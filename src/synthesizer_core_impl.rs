//! Conformance to [`synthesizer_core`] traits.
//!
//! Wave 2 of the compound-knowledge refactor: purely additive. No behavior
//! change to go-synthesizer's existing APIs — this module only adds trait
//! impls that downstream generic code can consume.

use crate::node::GoNode;
use synthesizer_core::{NoRawAttestation, SynthesizerNode};

impl SynthesizerNode for GoNode {
    fn emit(&self, indent: usize) -> String {
        // Delegate to the inherent `GoNode::emit` — inherent takes
        // priority over trait methods in UFCS path lookup.
        GoNode::emit(self, indent)
    }

    fn indent_unit() -> &'static str {
        "\t"
    }

    fn variant_id(&self) -> u8 {
        match self {
            Self::Comment(_) => 0,
            Self::Blank => 1,
            Self::Package(_) => 2,
            Self::Import(_) => 3,
            Self::Struct { .. } => 4,
            Self::Interface { .. } => 5,
            Self::Func { .. } => 6,
            Self::Method { .. } => 7,
            Self::VarDecl { .. } => 8,
            Self::ConstDecl { .. } => 9,
            Self::TypeAlias { .. } => 10,
        }
    }
}

impl NoRawAttestation for GoNode {
    fn attestation() -> &'static str {
        "GoNode has no raw escape-hatch variant. The enum declaration in \
         src/node.rs lists exactly 11 typed variants (Comment, Blank, \
         Package, Import, Struct, Interface, Func, Method, VarDecl, \
         ConstDecl, TypeAlias) — none of them accepts untyped source \
         fragments. The conformance test \
         no_raw_constructor_in_production_source (in \
         tests/synthesizer_core_conformance.rs) scans src/ for any \
         accidental introduction of a raw-string constructor and fails CI \
         if one appears. Every Go source construct is produced through \
         typed GoExpr / GoStmt / GoType primitives, never through raw \
         strings. The 22nd AST domain of the typescape is closed under \
         its primitives."
    }
}
