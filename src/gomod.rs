//! Typed `go.mod` node.
//!
//! A new shared primitive (not relocated from iac-forge) for emitting a
//! canonical, deterministic `go.mod`. Like the rest of go-synthesizer, the
//! authoring surface is a typed value and the printer is the single source
//! of layout truth — no `format!()`-string go.mod assembly in consumers.
//!
//! Determinism guarantees:
//!   - `require` entries are emitted in a stable, sorted order (by module
//!     path, then version) inside a single `require ( ... )` block.
//!   - indirect requirements are marked with the canonical `// indirect`
//!     trailing comment.
//!   - `replace`, `exclude`, `retract` directives are each emitted in
//!     sorted order in their own grouped block.
//!   - Two structurally identical [`GoMod`] values render byte-equal.

/// A single `require` directive entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoRequire {
    /// Module path, e.g. `github.com/crossplane/crossplane-runtime`.
    pub path: String,
    /// Semantic version, e.g. `v1.15.1`.
    pub version: String,
    /// `// indirect` marker.
    pub indirect: bool,
}

impl GoRequire {
    #[must_use]
    pub fn new(path: impl Into<String>, version: impl Into<String>) -> Self {
        Self { path: path.into(), version: version.into(), indirect: false }
    }

    #[must_use]
    pub fn indirect(path: impl Into<String>, version: impl Into<String>) -> Self {
        Self { path: path.into(), version: version.into(), indirect: true }
    }
}

/// A single `replace` directive: `replace old [oldv] => new [newv]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoReplace {
    /// Module path being replaced.
    pub old_path: String,
    /// Optional version of the module being replaced.
    pub old_version: Option<String>,
    /// Replacement module path or filesystem path.
    pub new_path: String,
    /// Optional version of the replacement (omitted for filesystem
    /// replacements like `./local`).
    pub new_version: Option<String>,
}

/// A single `exclude` directive: `exclude path version`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoExclude {
    pub path: String,
    pub version: String,
}

/// A single `retract` directive: `retract version` or
/// `retract [low, high]`, optionally with a rationale comment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoRetract {
    /// Low version of the range (or the single version when `high` is None).
    pub low: String,
    /// High version of the range; `None` means a single-version retract.
    pub high: Option<String>,
    /// Optional rationale, emitted as a `// <reason>` comment on the line
    /// above the directive (go convention).
    pub rationale: Option<String>,
}

/// A typed `go.mod` file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoMod {
    /// `module <path>`.
    pub module: String,
    /// `go <version>`, e.g. `1.22`.
    pub go: String,
    pub require: Vec<GoRequire>,
    pub replace: Vec<GoReplace>,
    pub exclude: Vec<GoExclude>,
    pub retract: Vec<GoRetract>,
}

impl GoMod {
    #[must_use]
    pub fn new(module: impl Into<String>, go: impl Into<String>) -> Self {
        Self {
            module: module.into(),
            go: go.into(),
            require: Vec::new(),
            replace: Vec::new(),
            exclude: Vec::new(),
            retract: Vec::new(),
        }
    }

    /// Render a canonical, deterministic `go.mod`.
    ///
    /// Layout mirrors `go mod tidy` conventions: `module` line, blank line,
    /// `go` line, then a grouped `require ( ... )` block (sorted), followed
    /// by grouped `replace` / `exclude` / `retract` blocks when present.
    #[must_use]
    pub fn print(&self) -> String {
        let mut out = String::new();
        out.push_str("module ");
        out.push_str(&self.module);
        out.push('\n');
        out.push('\n');
        out.push_str("go ");
        out.push_str(&self.go);
        out.push('\n');

        if !self.require.is_empty() {
            let mut reqs = self.require.clone();
            reqs.sort_by(|a, b| a.path.cmp(&b.path).then_with(|| a.version.cmp(&b.version)));
            out.push('\n');
            out.push_str("require (\n");
            for r in &reqs {
                out.push('\t');
                out.push_str(&r.path);
                out.push(' ');
                out.push_str(&r.version);
                if r.indirect {
                    out.push_str(" // indirect");
                }
                out.push('\n');
            }
            out.push_str(")\n");
        }

        if !self.replace.is_empty() {
            let mut reps = self.replace.clone();
            reps.sort_by(|a, b| {
                a.old_path
                    .cmp(&b.old_path)
                    .then_with(|| a.old_version.cmp(&b.old_version))
            });
            out.push('\n');
            out.push_str("replace (\n");
            for r in &reps {
                out.push('\t');
                out.push_str(&r.old_path);
                if let Some(v) = &r.old_version {
                    out.push(' ');
                    out.push_str(v);
                }
                out.push_str(" => ");
                out.push_str(&r.new_path);
                if let Some(v) = &r.new_version {
                    out.push(' ');
                    out.push_str(v);
                }
                out.push('\n');
            }
            out.push_str(")\n");
        }

        if !self.exclude.is_empty() {
            let mut exs = self.exclude.clone();
            exs.sort_by(|a, b| a.path.cmp(&b.path).then_with(|| a.version.cmp(&b.version)));
            out.push('\n');
            out.push_str("exclude (\n");
            for e in &exs {
                out.push('\t');
                out.push_str(&e.path);
                out.push(' ');
                out.push_str(&e.version);
                out.push('\n');
            }
            out.push_str(")\n");
        }

        if !self.retract.is_empty() {
            let mut rets = self.retract.clone();
            rets.sort_by(|a, b| a.low.cmp(&b.low).then_with(|| a.high.cmp(&b.high)));
            out.push('\n');
            out.push_str("retract (\n");
            for r in &rets {
                if let Some(reason) = &r.rationale {
                    out.push('\t');
                    out.push_str("// ");
                    out.push_str(reason);
                    out.push('\n');
                }
                out.push('\t');
                match &r.high {
                    Some(high) => {
                        out.push('[');
                        out.push_str(&r.low);
                        out.push_str(", ");
                        out.push_str(high);
                        out.push(']');
                    }
                    None => out.push_str(&r.low),
                }
                out.push('\n');
            }
            out.push_str(")\n");
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minimal_go_mod() {
        let m = GoMod::new("github.com/pleme-io/example", "1.22");
        let s = m.print();
        assert_eq!(s, "module github.com/pleme-io/example\n\ngo 1.22\n");
    }

    #[test]
    fn require_block_is_sorted_and_indirect_marked() {
        let mut m = GoMod::new("example.com/m", "1.22");
        m.require.push(GoRequire::indirect("github.com/z/last", "v1.0.0"));
        m.require.push(GoRequire::new("github.com/a/first", "v2.3.4"));
        let s = m.print();
        let expected = "module example.com/m\n\
            \n\
            go 1.22\n\
            \n\
            require (\n\
            \tgithub.com/a/first v2.3.4\n\
            \tgithub.com/z/last v1.0.0 // indirect\n\
            )\n";
        assert_eq!(s, expected);
    }

    #[test]
    fn deterministic_regardless_of_insertion_order() {
        let mut a = GoMod::new("m", "1.22");
        a.require.push(GoRequire::new("b/two", "v1.0.0"));
        a.require.push(GoRequire::new("a/one", "v1.0.0"));
        let mut b = GoMod::new("m", "1.22");
        b.require.push(GoRequire::new("a/one", "v1.0.0"));
        b.require.push(GoRequire::new("b/two", "v1.0.0"));
        assert_eq!(a.print(), b.print());
    }

    #[test]
    fn replace_block_filesystem_and_versioned() {
        let mut m = GoMod::new("m", "1.22");
        m.replace.push(GoReplace {
            old_path: "github.com/old/mod".into(),
            old_version: Some("v1.0.0".into()),
            new_path: "github.com/new/mod".into(),
            new_version: Some("v1.2.0".into()),
        });
        m.replace.push(GoReplace {
            old_path: "github.com/local/dep".into(),
            old_version: None,
            new_path: "./local".into(),
            new_version: None,
        });
        let s = m.print();
        assert!(s.contains("replace (\n"));
        assert!(s.contains("\tgithub.com/local/dep => ./local\n"));
        assert!(s.contains("\tgithub.com/old/mod v1.0.0 => github.com/new/mod v1.2.0\n"));
        // sorted: "github.com/local/..." before "github.com/old/..."
        let local = s.find("github.com/local/dep").unwrap();
        let old = s.find("github.com/old/mod").unwrap();
        assert!(local < old);
    }

    #[test]
    fn exclude_block_sorted() {
        let mut m = GoMod::new("m", "1.22");
        m.exclude.push(GoExclude { path: "z/mod".into(), version: "v1.0.0".into() });
        m.exclude.push(GoExclude { path: "a/mod".into(), version: "v1.0.0".into() });
        let s = m.print();
        assert!(s.contains("exclude (\n"));
        let a = s.find("a/mod").unwrap();
        let z = s.find("z/mod").unwrap();
        assert!(a < z);
    }

    #[test]
    fn retract_single_and_range_with_rationale() {
        let mut m = GoMod::new("m", "1.22");
        m.retract.push(GoRetract {
            low: "v1.0.0".into(),
            high: None,
            rationale: Some("published by mistake".into()),
        });
        m.retract.push(GoRetract {
            low: "v1.1.0".into(),
            high: Some("v1.2.0".into()),
            rationale: None,
        });
        let s = m.print();
        assert!(s.contains("retract (\n"));
        assert!(s.contains("\t// published by mistake\n\tv1.0.0\n"));
        assert!(s.contains("\t[v1.1.0, v1.2.0]\n"));
    }

    #[test]
    fn full_go_mod_byte_equal() {
        let mut m = GoMod::new("github.com/pleme-io/full", "1.22");
        m.require.push(GoRequire::new("github.com/crossplane/crossplane-runtime", "v1.15.1"));
        m.require.push(GoRequire::indirect("github.com/pkg/errors", "v0.9.1"));
        let s = m.print();
        let expected = "module github.com/pleme-io/full\n\
            \n\
            go 1.22\n\
            \n\
            require (\n\
            \tgithub.com/crossplane/crossplane-runtime v1.15.1\n\
            \tgithub.com/pkg/errors v0.9.1 // indirect\n\
            )\n";
        assert_eq!(s, expected);
    }
}
