//! Integration tests proving `GoNode` conforms to `synthesizer_core` traits.
//!
//! Wave 2 of the compound-knowledge refactor. Every test calls one of
//! `synthesizer_core::node::laws::*` on a real `GoNode` value, compounding
//! proof surface: the same laws prove properties of every synthesizer that
//! conforms.

// Conformance is proven on the primitive "node" layer (`GoNode`), so import
// every node type from `go_synthesizer::node` — the colliding names
// (GoExpr/GoField/GoImport/GoParam/GoStmt/GoType) resolve to the richer
// "file" layer at the crate root after canonicalization.
use go_synthesizer::node::{
    ChanDir, GoExpr, GoField, GoImport, GoMethodSig, GoNode, GoParam, GoStmt, GoType,
};
use synthesizer_core::node::laws;
use synthesizer_core::{NoRawAttestation, SynthesizerNode};

// ─── Trait shape ────────────────────────────────────────────────────

#[test]
fn indent_unit_is_tab() {
    // Go's gofmt convention: tab indentation, not spaces.
    assert_eq!(<GoNode as SynthesizerNode>::indent_unit(), "\t");
}

#[test]
fn variant_ids_distinct_across_disjoint_variants() {
    let samples: Vec<GoNode> = vec![
        GoNode::Comment("hello".into()),
        GoNode::Blank,
        GoNode::Package("main".into()),
        GoNode::Import(vec![GoImport::new("fmt")]),
        GoNode::Struct {
            name: "S".into(),
            fields: vec![],
            doc: None,
        },
        GoNode::Interface {
            name: "I".into(),
            methods: vec![],
            doc: None,
        },
        GoNode::Func {
            name: "f".into(),
            args: vec![],
            returns: vec![],
            body: vec![],
            doc: None,
        },
        GoNode::Method {
            receiver: GoParam::new("r", GoType::Named("T".into())),
            name: "m".into(),
            args: vec![],
            returns: vec![],
            body: vec![],
            doc: None,
        },
        GoNode::VarDecl {
            name: "x".into(),
            var_type: None,
            value: Some(GoExpr::Int(1)),
        },
        GoNode::ConstDecl {
            name: "C".into(),
            const_type: None,
            value: GoExpr::Int(1),
        },
        GoNode::TypeAlias {
            name: "A".into(),
            target: GoType::Named("int".into()),
            is_alias: true,
        },
    ];
    let before = samples.len();
    let mut ids: Vec<u8> = samples.iter().map(SynthesizerNode::variant_id).collect();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(
        ids.len(),
        before,
        "variant_id must be distinct for disjoint variants"
    );
}

// ─── SynthesizerNode laws ───────────────────────────────────────────

#[test]
fn law_determinism_holds_on_simple_nodes() {
    for n in [
        GoNode::Blank,
        GoNode::Comment("x".into()),
        GoNode::Package("main".into()),
        GoNode::Import(vec![GoImport::new("fmt")]),
    ] {
        assert!(laws::is_deterministic(&n, 0));
        assert!(laws::is_deterministic(&n, 3));
    }
}

#[test]
fn law_determinism_holds_on_struct() {
    let n = GoNode::Struct {
        name: "Config".into(),
        fields: vec![
            GoField::new("Name", GoType::Named("string".into())).with_tag("json:\"name\""),
            GoField::new("Port", GoType::Named("int".into())),
        ],
        doc: Some("Config holds config.".into()),
    };
    assert!(laws::is_deterministic(&n, 2));
}

#[test]
fn law_determinism_holds_on_func() {
    let n = GoNode::Func {
        name: "Main".into(),
        args: vec![GoParam::new("x", GoType::Named("int".into()))],
        returns: vec![GoType::Named("error".into())],
        body: vec![GoStmt::Return(vec![GoExpr::Nil])],
        doc: None,
    };
    assert!(laws::is_deterministic(&n, 1));
}

#[test]
fn law_determinism_holds_on_method() {
    let n = GoNode::Method {
        receiver: GoParam::new("c", GoType::Pointer(Box::new(GoType::Named("Config".into())))),
        name: "String".into(),
        args: vec![],
        returns: vec![GoType::Named("string".into())],
        body: vec![GoStmt::Return(vec![GoExpr::Str("cfg".into())])],
        doc: None,
    };
    assert!(laws::is_deterministic(&n, 0));
}

#[test]
fn law_honors_indent_unit_on_comment() {
    // Comment is a great test case: at indent 0 it emits "// text"; at
    // indent n it emits "\t\t...// text". The law verifies indent_unit
    // is respected.
    assert!(laws::honors_indent_unit(&GoNode::Comment("hi".into()), 0));
    assert!(laws::honors_indent_unit(&GoNode::Comment("hi".into()), 2));
}

#[test]
fn law_honors_indent_unit_on_package() {
    assert!(laws::honors_indent_unit(&GoNode::Package("main".into()), 0));
    assert!(laws::honors_indent_unit(&GoNode::Package("main".into()), 4));
}

#[test]
fn law_indent_monotone_len_on_comment() {
    // Emitting with more indentation produces at least as many bytes.
    assert!(laws::indent_monotone_len(&GoNode::Comment("x".into()), 0));
    assert!(laws::indent_monotone_len(&GoNode::Comment("x".into()), 3));
}

#[test]
fn law_indent_monotone_len_on_chan_returning_func() {
    // Exercise a GoNode carrying a chan type (covers ChanDir + GoMethodSig
    // paths in the emitter under indentation).
    let n = GoNode::Interface {
        name: "Streamer".into(),
        methods: vec![GoMethodSig {
            name: "Recv".into(),
            args: vec![],
            returns: vec![GoType::Chan {
                dir: ChanDir::Recv,
                elem: Box::new(GoType::Named("int".into())),
            }],
        }],
        doc: None,
    };
    assert!(laws::indent_monotone_len(&n, 0));
    assert!(laws::indent_monotone_len(&n, 2));
}

#[test]
fn law_variant_id_valid_on_all_sample_variants() {
    let samples = [
        GoNode::Blank,
        GoNode::Comment("x".into()),
        GoNode::Package("main".into()),
        GoNode::Import(vec![GoImport::new("os")]),
        GoNode::VarDecl {
            name: "x".into(),
            var_type: None,
            value: Some(GoExpr::Bool(true)),
        },
        GoNode::ConstDecl {
            name: "K".into(),
            const_type: Some(GoType::Named("int".into())),
            value: GoExpr::Int(42),
        },
        GoNode::TypeAlias {
            name: "ID".into(),
            target: GoType::Named("string".into()),
            is_alias: false,
        },
    ];
    for n in &samples {
        assert!(laws::variant_id_is_valid(n));
    }
}

// ─── NoRawAttestation ───────────────────────────────────────────────

#[test]
fn attestation_is_nonempty() {
    assert!(!<GoNode as NoRawAttestation>::attestation().is_empty());
}

#[test]
fn attestation_mentions_raw() {
    let s = <GoNode as NoRawAttestation>::attestation();
    assert!(
        s.to_lowercase().contains("raw"),
        "attestation must explain how no-raw is enforced — got: {s}"
    );
}

// ─── No-raw source invariant ────────────────────────────────────────

#[test]
fn no_raw_constructor_in_production_source() {
    // Scan src/ for `GoNode::Raw(...)` or `Self::Raw(...)` constructor
    // uses. Legitimate non-constructions (match arms, comments,
    // attribute lines) are exempted. GoNode has no Raw variant today;
    // this test guards against accidental reintroduction.
    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut violations = Vec::new();
    for path in walk_rust_files(&src_dir) {
        let content = std::fs::read_to_string(&path).expect("read src file");
        let lines: Vec<&str> = content.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("//") || trimmed.starts_with("*") {
                continue;
            }
            // Variant declaration line.
            if line.contains("Raw(String)") {
                continue;
            }
            // Match arms (patterns, not constructions).
            if line.contains("=>") {
                continue;
            }
            // Attribute lines.
            if trimmed.starts_with("#[") {
                continue;
            }
            // Preceding #[allow(deprecated)] → intentional reference.
            let prev_allows = i > 0 && lines[i - 1].contains("#[allow(deprecated)]");
            if prev_allows {
                continue;
            }
            if line.contains("GoNode::Raw(") || line.contains("Self::Raw(") {
                violations.push(format!("{}:{}", path.display(), i + 1));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "GoNode::Raw construction in production source is forbidden \
         (use a typed variant). Violations: {violations:?}"
    );
}

fn walk_rust_files(root: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(root).expect("read src dir") {
        let entry = entry.expect("read dir entry");
        let path = entry.path();
        if path.is_dir() {
            out.extend(walk_rust_files(&path));
        } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            out.push(path);
        }
    }
    out
}
