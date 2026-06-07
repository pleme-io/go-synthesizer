//! Builder → required-hash-kinds registry — a small typed map.
//!
//! This is gap A4 from the borealis pattern registry: a *generic prefetch
//! shape for the matrix healer*. A Nix/Go build "builder" (the FOD-producing
//! derivation kind that wraps a fetch) requires zero, one, or two hashes to be
//! prefetched before it can build hermetically — e.g. a `buildGoModule`
//! consumer needs a **vendor hash**; a `fetchFromGitHub` consumer needs a
//! **source hash**; a `mkGoTool` consumer needs **both**. When a matrix healer
//! wants to prefetch hashes generically (rather than hard-coding "this builder
//! needs a vendorHash"), it asks this registry which [`HashKind`]s a
//! [`BuilderKind`] requires.
//!
//! The registry is a *typed map* with a single source of truth
//! ([`required_hashes`]) — adding a builder kind is one match arm, never a
//! scattered conditional. It is pure data: no IO, no vendor specifics, fully public.

/// A kind of content hash a builder may need prefetched.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum HashKind {
    /// The hash of the fetched source tree (`fetchFromGitHub`'s `hash` /
    /// `sha256`, the `src` FOD).
    Source,
    /// The hash of the vendored Go module set (`buildGoModule`'s
    /// `vendorHash`).
    Vendor,
    /// The hash of a Cargo vendor dir (`vendorCargoDeps` / `cargoHash`).
    Cargo,
    /// The hash of an npm dependency closure (`fetchNpmDeps`' `npmDepsHash`).
    Npm,
}

impl HashKind {
    /// The canonical attribute name a Nix builder expects this hash under
    /// (the name the matrix healer writes into the spec).
    #[must_use]
    pub fn attr_name(self) -> &'static str {
        match self {
            HashKind::Source => "hash",
            HashKind::Vendor => "vendorHash",
            HashKind::Cargo => "cargoHash",
            HashKind::Npm => "npmDepsHash",
        }
    }
}

/// A kind of builder whose hash requirements the healer prefetches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum BuilderKind {
    /// A plain source fetch (`fetchFromGitHub` / `fetchurl`) — source hash only.
    Fetch,
    /// A `buildGoModule` derivation — vendor hash only (source comes from
    /// `src`, already pinned).
    GoModule,
    /// A `mkGoTool`-style builder that both fetches source and vendors —
    /// source + vendor.
    GoTool,
    /// A `buildRustPackage` / Cargo builder — cargo hash only.
    RustPackage,
    /// A Node/npm builder — npm deps hash only.
    NodePackage,
}

/// The set of [`HashKind`]s a [`BuilderKind`] requires, in canonical order
/// (sorted, stable). This is the single source of truth — the typed map.
#[must_use]
pub fn required_hashes(builder: BuilderKind) -> Vec<HashKind> {
    match builder {
        BuilderKind::Fetch => vec![HashKind::Source],
        BuilderKind::GoModule => vec![HashKind::Vendor],
        BuilderKind::GoTool => vec![HashKind::Source, HashKind::Vendor],
        BuilderKind::RustPackage => vec![HashKind::Cargo],
        BuilderKind::NodePackage => vec![HashKind::Npm],
    }
}

/// `true` iff the builder requires the given hash kind.
#[must_use]
pub fn requires(builder: BuilderKind, kind: HashKind) -> bool {
    required_hashes(builder).contains(&kind)
}

/// Every builder kind in the registry, in canonical order. Lets a healer
/// iterate the whole registry (e.g. to validate a spec covers every builder).
#[must_use]
pub fn all_builders() -> Vec<BuilderKind> {
    vec![
        BuilderKind::Fetch,
        BuilderKind::GoModule,
        BuilderKind::GoTool,
        BuilderKind::RustPackage,
        BuilderKind::NodePackage,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn go_module_needs_only_vendor() {
        assert_eq!(required_hashes(BuilderKind::GoModule), vec![HashKind::Vendor]);
        assert!(requires(BuilderKind::GoModule, HashKind::Vendor));
        assert!(!requires(BuilderKind::GoModule, HashKind::Source));
    }

    #[test]
    fn go_tool_needs_source_and_vendor() {
        assert_eq!(
            required_hashes(BuilderKind::GoTool),
            vec![HashKind::Source, HashKind::Vendor]
        );
    }

    #[test]
    fn fetch_needs_only_source() {
        assert_eq!(required_hashes(BuilderKind::Fetch), vec![HashKind::Source]);
    }

    #[test]
    fn attr_names_are_canonical() {
        assert_eq!(HashKind::Vendor.attr_name(), "vendorHash");
        assert_eq!(HashKind::Source.attr_name(), "hash");
        assert_eq!(HashKind::Cargo.attr_name(), "cargoHash");
        assert_eq!(HashKind::Npm.attr_name(), "npmDepsHash");
    }

    #[test]
    fn all_builders_have_at_least_one_hash() {
        for b in all_builders() {
            assert!(
                !required_hashes(b).is_empty(),
                "{b:?} has no required hashes"
            );
        }
    }

    #[test]
    fn required_hashes_are_sorted_and_unique() {
        for b in all_builders() {
            let hs = required_hashes(b);
            let mut sorted = hs.clone();
            sorted.sort();
            sorted.dedup();
            assert_eq!(hs, sorted, "{b:?} hashes not sorted/unique");
        }
    }
}
