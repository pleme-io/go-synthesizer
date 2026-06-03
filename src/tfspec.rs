//! Typed terraform-plugin-framework resource-spec model.
//!
//! This is the **typed authoring surface** for a Terraform provider resource:
//! a structural description of one `terraform-plugin-framework` resource (its
//! Go package, type names, schema attributes) that the [`crate::tfemit`]
//! emitter lowers to a buildable Go [`crate::GoFile`]. Like the rest of
//! go-synthesizer, the rule is the same one stated in [`crate::file`]: a
//! consumer never hand-writes provider Go via `format!()` — it builds a typed
//! [`TfResourceSpec`] and emits.
//!
//! ## Why a spec layer (not just the AST)
//!
//! The [`crate::file`] AST is general Go. A terraform-plugin-framework resource
//! has a *fixed shape*: a model struct whose fields carry `tfsdk:"…"` tags and
//! `types.String`/`types.Int64`/… framework types, a `Resource`-implementing
//! struct, and the five framework methods (`Metadata`/`Schema`/`Create`/
//! `Read`/`Update`/`Delete`). Encoding that shape *once* as a typed spec means
//! every provider resource is emitted the same way — the
//! drift-killer the registry (gap E3) demands.
//!
//! ## Worlds-separate
//!
//! This model names **Terraform** primitives only. It never names or imports
//! akeyless — an akeyless-shaped provider is just a `TfResourceSpec` whose
//! `type_name`/attributes happen to describe akeyless resources, authored by a
//! private consumer. The public shape stays generic.

/// The terraform-plugin-framework attribute value type for one schema
/// attribute. Mirrors the `github.com/hashicorp/terraform-plugin-framework/
/// types` family used in a resource model struct.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TfType {
    /// `types.String` — schema `schema.StringAttribute`.
    String,
    /// `types.Bool` — schema `schema.BoolAttribute`.
    Bool,
    /// `types.Int64` — schema `schema.Int64Attribute`.
    Int64,
    /// `types.Float64` — schema `schema.Float64Attribute`.
    Float64,
    /// `types.Number` — schema `schema.NumberAttribute`.
    Number,
    /// `types.List` — schema `schema.ListAttribute`.
    List,
    /// `types.Map` — schema `schema.MapAttribute`.
    Map,
    /// `types.Set` — schema `schema.SetAttribute`.
    Set,
    /// `types.Object` — schema `schema.ObjectAttribute`.
    Object,
}

impl TfType {
    /// The `types.<X>` Go type name used in the model struct field.
    #[must_use]
    pub fn model_type(self) -> &'static str {
        match self {
            TfType::String => "types.String",
            TfType::Bool => "types.Bool",
            TfType::Int64 => "types.Int64",
            TfType::Float64 => "types.Float64",
            TfType::Number => "types.Number",
            TfType::List => "types.List",
            TfType::Map => "types.Map",
            TfType::Set => "types.Set",
            TfType::Object => "types.Object",
        }
    }

    /// The `schema.<X>Attribute` Go type name used in the schema's
    /// `Attributes` map.
    #[must_use]
    pub fn schema_attribute(self) -> &'static str {
        match self {
            TfType::String => "schema.StringAttribute",
            TfType::Bool => "schema.BoolAttribute",
            TfType::Int64 => "schema.Int64Attribute",
            TfType::Float64 => "schema.Float64Attribute",
            TfType::Number => "schema.NumberAttribute",
            TfType::List => "schema.ListAttribute",
            TfType::Map => "schema.MapAttribute",
            TfType::Set => "schema.SetAttribute",
            TfType::Object => "schema.ObjectAttribute",
        }
    }
}

/// One schema attribute on a terraform-plugin-framework resource.
///
/// The four boolean knobs are the framework's behaviour flags. They are
/// mutually constrained by Terraform's own rules (`required` and `optional` are
/// mutually exclusive; a `computed` attribute may also be `optional`) — the
/// spec records intent; [`TfResourceSpec::validate`] enforces the constraints.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TfAttribute {
    /// The attribute name as written in HCL and the `tfsdk:"…"` tag, e.g.
    /// `"name"`, `"access_id"`. Snake-case by Terraform convention.
    pub name: String,
    /// The Go field name in the model struct, e.g. `"Name"`, `"AccessID"`.
    /// PascalCase by Go convention.
    pub go_field: String,
    /// The framework value type.
    pub ty: TfType,
    /// `Required: true` in the schema.
    pub required: bool,
    /// `Optional: true` in the schema.
    pub optional: bool,
    /// `Computed: true` in the schema (provider-set / read-only value).
    pub computed: bool,
    /// `Sensitive: true` in the schema (redacted in plan output).
    pub sensitive: bool,
    /// Optional `MarkdownDescription` for the attribute.
    pub description: Option<String>,
}

impl TfAttribute {
    /// A required, plain attribute of the given type.
    #[must_use]
    pub fn required(
        name: impl Into<String>,
        go_field: impl Into<String>,
        ty: TfType,
    ) -> Self {
        Self {
            name: name.into(),
            go_field: go_field.into(),
            ty,
            required: true,
            optional: false,
            computed: false,
            sensitive: false,
            description: None,
        }
    }

    /// An optional, plain attribute of the given type.
    #[must_use]
    pub fn optional(
        name: impl Into<String>,
        go_field: impl Into<String>,
        ty: TfType,
    ) -> Self {
        Self {
            name: name.into(),
            go_field: go_field.into(),
            ty,
            required: false,
            optional: true,
            computed: false,
            sensitive: false,
            description: None,
        }
    }

    /// A computed-only attribute (provider-set, read-only — e.g. an `id`).
    #[must_use]
    pub fn computed(
        name: impl Into<String>,
        go_field: impl Into<String>,
        ty: TfType,
    ) -> Self {
        Self {
            name: name.into(),
            go_field: go_field.into(),
            ty,
            required: false,
            optional: false,
            computed: true,
            sensitive: false,
            description: None,
        }
    }

    /// Builder: mark the attribute `Sensitive: true`.
    #[must_use]
    pub fn with_sensitive(mut self) -> Self {
        self.sensitive = true;
        self
    }

    /// Builder: attach a `MarkdownDescription`.
    #[must_use]
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
}

/// A whole terraform-plugin-framework resource specification.
///
/// One value of this type describes one Go file emitting one Terraform
/// resource. [`crate::tfemit::emit_resource`] turns it into a [`crate::GoFile`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TfResourceSpec {
    /// The Go package the emitted file belongs to, e.g. `"provider"`.
    pub package: String,
    /// The Terraform type name *without* the provider prefix, e.g. `"static_secret"`
    /// for a resource addressed as `myprovider_static_secret`.
    pub type_name: String,
    /// The provider's type-name prefix, e.g. `"myprovider"`. The full
    /// Terraform resource type is `"{provider}_{type_name}"`.
    pub provider: String,
    /// The Go identifier base for the resource + model structs, e.g.
    /// `"StaticSecret"` → `StaticSecretResource` + `StaticSecretResourceModel`.
    pub go_name: String,
    /// The schema attributes, in declaration order (the emitter preserves order
    /// for byte-stable output).
    pub attributes: Vec<TfAttribute>,
    /// Optional resource-level `MarkdownDescription`.
    pub description: Option<String>,
}

impl TfResourceSpec {
    /// Construct a spec with no attributes; push [`TfAttribute`]s onto
    /// [`Self::attributes`] or use [`Self::with_attribute`].
    #[must_use]
    pub fn new(
        package: impl Into<String>,
        provider: impl Into<String>,
        type_name: impl Into<String>,
        go_name: impl Into<String>,
    ) -> Self {
        Self {
            package: package.into(),
            provider: provider.into(),
            type_name: type_name.into(),
            go_name: go_name.into(),
            attributes: Vec::new(),
            description: None,
        }
    }

    /// Builder: append one attribute.
    #[must_use]
    pub fn with_attribute(mut self, attr: TfAttribute) -> Self {
        self.attributes.push(attr);
        self
    }

    /// Builder: attach a resource-level description.
    #[must_use]
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// The fully-qualified Terraform resource type, `"{provider}_{type_name}"`.
    #[must_use]
    pub fn full_type_name(&self) -> String {
        format!("{}_{}", self.provider, self.type_name)
    }

    /// The Go identifier for the resource struct, `"{go_name}Resource"`.
    #[must_use]
    pub fn resource_struct(&self) -> String {
        format!("{}Resource", self.go_name)
    }

    /// The Go identifier for the model struct, `"{go_name}ResourceModel"`.
    #[must_use]
    pub fn model_struct(&self) -> String {
        format!("{}ResourceModel", self.go_name)
    }

    /// Validate the spec's structural invariants. Returns the list of problems
    /// found (empty = valid). This is the spec-layer guard the emitter relies on
    /// so a malformed spec never reaches the AST.
    ///
    /// Checks:
    ///   - non-empty package / provider / type_name / go_name,
    ///   - no attribute is simultaneously `required` and `optional`,
    ///   - every attribute is at least one of required/optional/computed,
    ///   - attribute names and go-field names are unique.
    #[must_use]
    pub fn validate(&self) -> Vec<String> {
        let mut errs = Vec::new();
        if self.package.is_empty() {
            errs.push("package must not be empty".to_string());
        }
        if self.provider.is_empty() {
            errs.push("provider must not be empty".to_string());
        }
        if self.type_name.is_empty() {
            errs.push("type_name must not be empty".to_string());
        }
        if self.go_name.is_empty() {
            errs.push("go_name must not be empty".to_string());
        }

        let mut seen_names = std::collections::BTreeSet::new();
        let mut seen_fields = std::collections::BTreeSet::new();
        for a in &self.attributes {
            if a.required && a.optional {
                errs.push(format!(
                    "attribute `{}` is both required and optional",
                    a.name
                ));
            }
            if !a.required && !a.optional && !a.computed {
                errs.push(format!(
                    "attribute `{}` must be at least one of required/optional/computed",
                    a.name
                ));
            }
            if !seen_names.insert(a.name.clone()) {
                errs.push(format!("duplicate attribute name `{}`", a.name));
            }
            if !seen_fields.insert(a.go_field.clone()) {
                errs.push(format!("duplicate go field `{}`", a.go_field));
            }
        }
        errs
    }

    /// `true` iff [`Self::validate`] returns no problems.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.validate().is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tf_type_model_and_schema_names() {
        assert_eq!(TfType::String.model_type(), "types.String");
        assert_eq!(TfType::String.schema_attribute(), "schema.StringAttribute");
        assert_eq!(TfType::Int64.model_type(), "types.Int64");
        assert_eq!(TfType::Int64.schema_attribute(), "schema.Int64Attribute");
    }

    #[test]
    fn full_type_name_joins_provider_and_type() {
        let s = TfResourceSpec::new("provider", "myprovider", "static_secret", "StaticSecret");
        assert_eq!(s.full_type_name(), "myprovider_static_secret");
        assert_eq!(s.resource_struct(), "StaticSecretResource");
        assert_eq!(s.model_struct(), "StaticSecretResourceModel");
    }

    #[test]
    fn valid_spec_reports_no_errors() {
        let s = TfResourceSpec::new("provider", "myprovider", "thing", "Thing")
            .with_attribute(TfAttribute::required("name", "Name", TfType::String))
            .with_attribute(TfAttribute::computed("id", "ID", TfType::String));
        assert!(s.is_valid(), "expected valid, got {:?}", s.validate());
    }

    #[test]
    fn required_and_optional_is_invalid() {
        let mut attr = TfAttribute::required("name", "Name", TfType::String);
        attr.optional = true;
        let s = TfResourceSpec::new("provider", "p", "t", "T").with_attribute(attr);
        let errs = s.validate();
        assert!(errs.iter().any(|e| e.contains("both required and optional")));
    }

    #[test]
    fn attribute_with_no_disposition_is_invalid() {
        let attr = TfAttribute {
            name: "x".into(),
            go_field: "X".into(),
            ty: TfType::String,
            required: false,
            optional: false,
            computed: false,
            sensitive: false,
            description: None,
        };
        let s = TfResourceSpec::new("provider", "p", "t", "T").with_attribute(attr);
        let errs = s.validate();
        assert!(errs.iter().any(|e| e.contains("at least one of")));
    }

    #[test]
    fn duplicate_attribute_name_is_invalid() {
        let s = TfResourceSpec::new("provider", "p", "t", "T")
            .with_attribute(TfAttribute::required("name", "Name", TfType::String))
            .with_attribute(TfAttribute::optional("name", "NameTwo", TfType::String));
        let errs = s.validate();
        assert!(errs.iter().any(|e| e.contains("duplicate attribute name")));
    }

    #[test]
    fn empty_package_is_invalid() {
        let s = TfResourceSpec::new("", "p", "t", "T");
        assert!(s.validate().iter().any(|e| e.contains("package")));
    }

    #[test]
    fn builder_chaining_sets_fields() {
        let attr = TfAttribute::optional("token", "Token", TfType::String)
            .with_sensitive()
            .with_description("the secret token");
        assert!(attr.sensitive);
        assert_eq!(attr.description.as_deref(), Some("the secret token"));
    }
}
