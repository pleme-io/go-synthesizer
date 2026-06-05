//! go-synthesizer — the canonical typed Go-source AST for the pleme-io fleet.
//!
//! # Dual-layer design (deliberate, with a planned unification follow-up)
//!
//! This crate currently carries **two** typed Go-source models that coexist
//! at the crate root. This is an intentional intermediate state: the richer
//! "file" layer was relocated **verbatim** from `iac-forge/src/goast.rs` so
//! that downstream consumers (notably `crossplane-forge`, which drives
//! `crossplane-akeyless`) can import every item from `go_synthesizer` with
//! the **same name and signature** they previously imported from
//! `iac_forge::goast` — making the downstream migration a pure path swap
//! with **byte-identical** generated output. Full unification of the two
//! models into a single AST is a deliberate **follow-up**, not in scope for
//! this relocation.
//!
//! ## Layer 1 — the "file" layer ([`mod@file`], + [`mod@kube`])
//!
//! The whole-file model: [`GoFile`] + [`GoPrinter`] (import grouping /
//! sorting, the `// Code generated ... DO NOT EDIT.` header, multi-line
//! struct / composite emission, byte-equal determinism), [`GoDecl`] /
//! [`GoTypeDecl`] / [`GoTypeBody`], [`GoField`] with typed [`GoStructTag`]
//! ([`JsonTag`] / [`YamlTag`] / `Custom`), [`GoFuncDecl`] / [`GoRecv`] /
//! [`GoParam`], [`GoVarDecl`], the file-layer [`GoBlock`] / [`GoStmt`] /
//! [`GoExpr`] / [`GoLit`], [`GoImport`], and the kubebuilder markers
//! ([`KubeMarker`] / [`SubresourceKind`] / [`ResourceScope`]).
//!
//! **These names own the crate root.** Where a name collides with the
//! primitive layer (`GoImport`, `GoField`, `GoParam`, `GoType`, `GoStmt`,
//! `GoExpr`), the crate-root name resolves to **this** layer, so the
//! `iac_forge::goast::X` → `go_synthesizer::X` rewrite is mechanical.
//!
//! ## Layer 2 — the primitive "node" layer ([`mod@node`])
//!
//! The original flat per-node model: [`node::GoNode`] (the
//! `synthesizer-core`-conformant top-level node with per-node
//! `emit(indent)`), plus its [`node::GoExpr`] / [`node::GoStmt`] /
//! [`node::GoType`] / [`node::GoImport`] / [`node::GoField`] /
//! [`node::GoParam`] / [`node::GoMethodSig`] / [`node::GoCase`] /
//! [`node::ChanDir`] and [`node::emit_file`]. This layer is preserved
//! unchanged and continues to power the `synthesizer-core` conformance
//! impl. Its non-colliding items ([`GoNode`], [`GoMethodSig`], [`GoCase`],
//! [`ChanDir`], [`emit_file`]) are also re-exported at the crate root for
//! convenience; the colliding ones remain reachable via `go_synthesizer::node::*`.
//!
//! ## Layer 3 — the typed `go.mod` node ([`mod@gomod`])
//!
//! [`GoMod`] (+ [`GoRequire`] / [`GoReplace`] / [`GoExclude`] /
//! [`GoRetract`]) emits a canonical, deterministic `go.mod`.

pub mod file;
pub mod gomod;
pub mod hashkinds;
pub mod kube;
pub mod node;
pub mod tfemit;
pub mod tfspec;

mod synthesizer_core_impl;

// ── Crate-root re-exports ──────────────────────────────────────────────────

// Layer 1 owns the crate root. Colliding names (GoImport, GoField, GoParam,
// GoType, GoStmt, GoExpr) resolve to the file layer here, matching the
// original `iac_forge::goast` surface so downstream imports are a pure path
// swap.
pub use file::{
    GoBlock, GoDecl, GoExpr, GoField, GoFile, GoFuncDecl, GoIfaceMethod, GoImport, GoLit, GoParam,
    GoPrinter, GoRecv, GoSelectCase, GoStmt, GoStructTag, GoType, GoTypeBody, GoTypeDecl, GoVarDecl,
    JsonTag, YamlTag, print_file,
};
pub use kube::{KubeMarker, ResourceScope, SubresourceKind};

// Layer 3.
pub use gomod::{GoExclude, GoMod, GoReplace, GoRequire, GoRetract};

// ── New emitters / specs / registries (additive) ───────────────────────────
//
// The terraform-plugin-framework emitter (gap E3), its typed TF-resource-spec
// model, and the builder→required-hash-kinds registry (gap A4). None collide
// with existing crate-root names, so all are re-exported here.
pub use hashkinds::{BuilderKind, HashKind, all_builders, required_hashes, requires};
pub use tfemit::{EmitError, emit_resource};
pub use tfspec::{TfAttribute, TfResourceSpec, TfType};

// Layer 2 — only the names that do NOT collide with the file layer are
// re-exported at the crate root. The colliding primitive types remain
// available via `go_synthesizer::node::{GoExpr, GoStmt, GoType, GoImport,
// GoField, GoParam}`.
pub use node::{ChanDir, GoCase, GoMethodSig, GoNode, emit_file};
