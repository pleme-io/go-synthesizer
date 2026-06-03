//! Kubebuilder marker primitives (structural — not strings).
//!
//! Relocated **verbatim** from `iac-forge/src/goast.rs` so that the
//! canonical Go-source AST lives in `go-synthesizer`. These types are part
//! of the [`crate::file`] layer (the richer GoFile/GoPrinter model) and are
//! re-exported at the crate root for downstream consumers that previously
//! imported them from `iac_forge::goast`.
//!
//! Output emitted from these markers must remain **byte-identical** to the
//! original `iac_forge::goast` printer output — the printing logic in
//! [`crate::file::GoPrinter`] is preserved unchanged.

// ── Kubebuilder markers (structural — not strings) ────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum KubeMarker {
    Required,
    Optional,
    XValidationCEL { rule: String, message: String },
    ObjectGenerate(bool),
    ObjectRoot,
    Subresource(SubresourceKind),
    Resource { scope: ResourceScope, categories: Vec<String> },
    PrintColumn { name: String, ty: String, json_path: String, priority: Option<u32> },
    GroupName(String),
    /// Free-form fallback for kubebuilder markers we haven't structured
    /// yet. Discouraged when a structured variant exists; the goal is
    /// for every emitter to use only structured variants.
    Free(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SubresourceKind {
    Status,
    Scale,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ResourceScope {
    Cluster,
    Namespaced,
}
