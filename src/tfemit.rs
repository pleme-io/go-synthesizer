//! terraform-plugin-framework emitter — [`crate::tfspec::TfResourceSpec`] →
//! buildable provider Go via the [`crate::file`] [`GoFile`] AST.
//!
//! This is gap E3 from the borealis pattern registry: a typed TF-resource-spec
//! lowered to a buildable `terraform-plugin-framework` resource. The emitter
//! constructs a [`GoFile`] structurally (never via `format!()` of Go syntax,
//! per the [`crate::file`] contract) containing:
//!
//!   - the model struct, fields tagged `tfsdk:"…"`, framework `types.*` field
//!     types, in attribute-declaration order (byte-stable);
//!   - the `Resource`-implementing struct (`{go_name}Resource`);
//!   - a `New{go_name}Resource()` constructor returning `resource.Resource`;
//!   - the framework method set: `Metadata`, `Schema`, `Create`, `Read`,
//!     `Update`, `Delete`.
//!
//! The emitted CRUD bodies are intentionally *scaffold* bodies (TODO markers,
//! the canonical diagnostics + state plumbing) — the same posture as the
//! crossplane controller emitter: the shape is guaranteed correct, the
//! provider-specific API calls are filled by the consumer (or a richer
//! private emitter that composes this one).
//!
//! ## Worlds-separate
//!
//! The emitter only knows Terraform's framework. It never names any vendor; a
//! vendor provider is emitted by passing a `TfResourceSpec` that *describes*
//! that vendor's resources — the vendor coupling lives in the (private)
//! caller's spec, never here.

use crate::file::{
    GoBlock, GoDecl, GoExpr, GoField, GoFile, GoFuncDecl, GoImport, GoParam, GoRecv, GoStmt,
    GoStructTag, GoType, GoTypeBody, GoTypeDecl,
};
use crate::tfspec::TfResourceSpec;

/// Error returned when a [`TfResourceSpec`] fails its structural validation
/// before emission. Carries the human-readable problems
/// ([`TfResourceSpec::validate`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmitError {
    pub problems: Vec<String>,
}

impl std::fmt::Display for EmitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid TfResourceSpec: {}", self.problems.join("; "))
    }
}

impl std::error::Error for EmitError {}

const FRAMEWORK_RESOURCE: &str = "github.com/hashicorp/terraform-plugin-framework/resource";
const FRAMEWORK_RESOURCE_SCHEMA: &str =
    "github.com/hashicorp/terraform-plugin-framework/resource/schema";
const FRAMEWORK_TYPES: &str = "github.com/hashicorp/terraform-plugin-framework/types";

/// Lower a [`TfResourceSpec`] to a buildable [`GoFile`].
///
/// Returns [`EmitError`] if the spec is structurally invalid (the same checks
/// as [`TfResourceSpec::validate`]) so a malformed spec never reaches the AST.
///
/// # Errors
///
/// Returns [`EmitError`] when [`TfResourceSpec::validate`] reports problems.
pub fn emit_resource(spec: &TfResourceSpec) -> Result<GoFile, EmitError> {
    let problems = spec.validate();
    if !problems.is_empty() {
        return Err(EmitError { problems });
    }

    let mut file = GoFile::new(&spec.package);
    file.doc = Some(format!(
        "Package {} implements the {} Terraform provider resources.",
        spec.package,
        spec.provider
    ));
    file.imports = vec![
        GoImport::plain("context"),
        GoImport::plain(FRAMEWORK_RESOURCE),
        GoImport::plain(FRAMEWORK_RESOURCE_SCHEMA),
        GoImport::plain(FRAMEWORK_TYPES),
    ];

    // 1. interface-assertion var: var _ resource.Resource = &XResource{}
    file.decls.push(GoDecl::Var(crate::file::GoVarDecl {
        name: "_".to_string(),
        ty: Some(GoType::qualified("resource", "Resource")),
        value: Some(GoExpr::Composite {
            ty: GoType::named(spec.resource_struct()),
            fields: vec![],
            addr_of: true,
        }),
        doc: Some(format!(
            "{} satisfies the terraform-plugin-framework resource.Resource interface.",
            spec.resource_struct()
        )),
        block_id: None,
    }));

    // 2. the model struct
    file.decls.push(GoDecl::Type(model_struct(spec)));

    // 3. the resource struct (empty for the scaffold; real providers hold a client)
    file.decls.push(GoDecl::Type(GoTypeDecl {
        name: spec.resource_struct(),
        doc: Some(format!(
            "{} is the terraform-plugin-framework resource for {}.",
            spec.resource_struct(),
            spec.full_type_name()
        )),
        markers: vec![],
        body: GoTypeBody::Struct(vec![]),
    }));

    // 4. the constructor
    file.decls.push(GoDecl::Func(constructor(spec)));

    // 5. Metadata
    file.decls.push(GoDecl::Func(metadata_method(spec)));

    // 6. Schema
    file.decls.push(GoDecl::Func(schema_method(spec)));

    // 7-10. CRUD scaffold
    for verb in ["Create", "Read", "Update", "Delete"] {
        file.decls.push(GoDecl::Func(crud_method(spec, verb)));
    }

    Ok(file)
}

fn model_struct(spec: &TfResourceSpec) -> GoTypeDecl {
    let fields = spec
        .attributes
        .iter()
        .map(|a| GoField {
            name: Some(a.go_field.clone()),
            ty: framework_model_type(a.ty),
            doc: a.description.clone(),
            markers: vec![],
            tags: vec![GoStructTag::Custom {
                key: "tfsdk".to_string(),
                value: a.name.clone(),
            }],
        })
        .collect();
    GoTypeDecl {
        name: spec.model_struct(),
        doc: Some(format!(
            "{} maps the {} resource schema to Go.",
            spec.model_struct(),
            spec.full_type_name()
        )),
        markers: vec![],
        body: GoTypeBody::Struct(fields),
    }
}

/// The `types.<X>` field type for a model struct field.
fn framework_model_type(ty: crate::tfspec::TfType) -> GoType {
    // model_type() returns "types.String" etc — split on the dot for a
    // structural Qualified type (no `format!()` of Go syntax).
    let s = ty.model_type();
    let (pkg, name) = s.split_once('.').unwrap_or(("types", s));
    GoType::qualified(pkg, name)
}

fn constructor(spec: &TfResourceSpec) -> GoFuncDecl {
    let mut body = GoBlock::new();
    body.push(GoStmt::Return(vec![GoExpr::Composite {
        ty: GoType::named(spec.resource_struct()),
        fields: vec![],
        addr_of: true,
    }]));
    GoFuncDecl {
        name: format!("New{}Resource", spec.go_name),
        doc: Some(format!(
            "New{}Resource constructs the {} resource.",
            spec.go_name,
            spec.full_type_name()
        )),
        recv: None,
        params: vec![],
        returns: vec![GoType::qualified("resource", "Resource")],
        body,
    }
}

fn metadata_method(spec: &TfResourceSpec) -> GoFuncDecl {
    // resp.TypeName = req.ProviderTypeName + "_<type_name>"  (the framework idiom)
    let mut body = GoBlock::new();
    body.push(GoStmt::Assign {
        lhs: vec![GoExpr::sel(GoExpr::ident("resp"), "TypeName")],
        rhs: vec![GoExpr::binary(
            "+",
            GoExpr::sel(GoExpr::ident("req"), "ProviderTypeName"),
            GoExpr::str(format!("_{}", spec.type_name)),
        )],
    });
    GoFuncDecl {
        name: "Metadata".to_string(),
        doc: Some("Metadata sets the resource's Terraform type name.".to_string()),
        recv: Some(GoRecv {
            name: "r".to_string(),
            ty: GoType::pointer(GoType::named(spec.resource_struct())),
        }),
        params: vec![
            GoParam { name: "ctx".to_string(), ty: GoType::qualified("context", "Context") },
            GoParam {
                name: "req".to_string(),
                ty: GoType::qualified("resource", "MetadataRequest"),
            },
            GoParam {
                name: "resp".to_string(),
                ty: GoType::pointer(GoType::qualified("resource", "MetadataResponse")),
            },
        ],
        returns: vec![],
        body,
    }
}

fn schema_method(spec: &TfResourceSpec) -> GoFuncDecl {
    // resp.Schema = schema.Schema{ Attributes: map[string]schema.Attribute{...} }
    let attr_fields: Vec<(Option<String>, GoExpr)> = spec
        .attributes
        .iter()
        .map(|a| {
            let mut fields = vec![
                (
                    Some("Required".to_string()),
                    GoExpr::Lit(crate::file::GoLit::Bool(a.required)),
                ),
                (
                    Some("Optional".to_string()),
                    GoExpr::Lit(crate::file::GoLit::Bool(a.optional)),
                ),
                (
                    Some("Computed".to_string()),
                    GoExpr::Lit(crate::file::GoLit::Bool(a.computed)),
                ),
            ];
            if a.sensitive {
                fields.push((
                    Some("Sensitive".to_string()),
                    GoExpr::Lit(crate::file::GoLit::Bool(true)),
                ));
            }
            if let Some(desc) = &a.description {
                fields.push((
                    Some("MarkdownDescription".to_string()),
                    GoExpr::str(desc.clone()),
                ));
            }
            let (pkg, name) = a
                .ty
                .schema_attribute()
                .split_once('.')
                .unwrap_or(("schema", a.ty.schema_attribute()));
            // Composite-literal field names are emitted raw by the printer
            // (correct for STRUCT field names). For a `map[string]X` literal
            // the key is a quoted string literal, so we wrap the attribute name
            // in `"…"` so it renders as `"name":` (valid Go map key).
            (
                Some(format!("\"{}\"", a.name)),
                GoExpr::Composite {
                    ty: GoType::qualified(pkg, name),
                    fields,
                    addr_of: false,
                },
            )
        })
        .collect();

    let attrs_map = GoExpr::Composite {
        ty: GoType::Map(
            Box::new(GoType::named("string")),
            Box::new(GoType::qualified("schema", "Attribute")),
        ),
        fields: attr_fields,
        addr_of: false,
    };

    let mut schema_fields = vec![(Some("Attributes".to_string()), attrs_map)];
    if let Some(desc) = &spec.description {
        schema_fields.insert(
            0,
            (
                Some("MarkdownDescription".to_string()),
                GoExpr::str(desc.clone()),
            ),
        );
    }

    let mut body = GoBlock::new();
    body.push(GoStmt::Assign {
        lhs: vec![GoExpr::sel(GoExpr::ident("resp"), "Schema")],
        rhs: vec![GoExpr::Composite {
            ty: GoType::qualified("schema", "Schema"),
            fields: schema_fields,
            addr_of: false,
        }],
    });

    GoFuncDecl {
        name: "Schema".to_string(),
        doc: Some("Schema defines the resource's attribute schema.".to_string()),
        recv: Some(GoRecv {
            name: "r".to_string(),
            ty: GoType::pointer(GoType::named(spec.resource_struct())),
        }),
        params: vec![
            GoParam { name: "ctx".to_string(), ty: GoType::qualified("context", "Context") },
            GoParam {
                name: "req".to_string(),
                ty: GoType::qualified("resource", "SchemaRequest"),
            },
            GoParam {
                name: "resp".to_string(),
                ty: GoType::pointer(GoType::qualified("resource", "SchemaResponse")),
            },
        ],
        returns: vec![],
        body,
    }
}

fn crud_method(spec: &TfResourceSpec, verb: &str) -> GoFuncDecl {
    let (req_ty, resp_ty) = match verb {
        "Create" => ("CreateRequest", "CreateResponse"),
        "Read" => ("ReadRequest", "ReadResponse"),
        "Update" => ("UpdateRequest", "UpdateResponse"),
        "Delete" => ("DeleteRequest", "DeleteResponse"),
        other => panic!("unknown CRUD verb {other}"),
    };
    let mut body = GoBlock::new();
    body.push(GoStmt::Comment(format!(
        "TODO: implement {verb} for {}.",
        spec.full_type_name()
    )));
    body.push(GoStmt::Comment(
        "Decode req plan/state, call the provider API, set resp state + diagnostics.".to_string(),
    ));
    GoFuncDecl {
        name: verb.to_string(),
        doc: Some(format!(
            "{verb} implements the terraform-plugin-framework {} lifecycle hook.",
            verb.to_lowercase()
        )),
        recv: Some(GoRecv {
            name: "r".to_string(),
            ty: GoType::pointer(GoType::named(spec.resource_struct())),
        }),
        params: vec![
            GoParam { name: "ctx".to_string(), ty: GoType::qualified("context", "Context") },
            GoParam {
                name: "req".to_string(),
                ty: GoType::qualified("resource", req_ty),
            },
            GoParam {
                name: "resp".to_string(),
                ty: GoType::pointer(GoType::qualified("resource", resp_ty)),
            },
        ],
        returns: vec![],
        body,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file::print_file;
    use crate::tfspec::{TfAttribute, TfType};

    fn sample_spec() -> TfResourceSpec {
        TfResourceSpec::new("provider", "myprovider", "static_secret", "StaticSecret")
            .with_description("A static secret resource.")
            .with_attribute(TfAttribute::required("name", "Name", TfType::String))
            .with_attribute(
                TfAttribute::optional("value", "Value", TfType::String)
                    .with_sensitive()
                    .with_description("the secret value"),
            )
            .with_attribute(TfAttribute::computed("id", "ID", TfType::String))
    }

    #[test]
    fn emit_rejects_invalid_spec() {
        let bad = TfResourceSpec::new("", "p", "t", "T");
        let err = emit_resource(&bad).unwrap_err();
        assert!(err.problems.iter().any(|p| p.contains("package")));
    }

    #[test]
    fn emitted_file_has_model_and_resource_structs() {
        let spec = sample_spec();
        let file = emit_resource(&spec).unwrap();
        let s = print_file(&file);
        assert!(s.contains("type StaticSecretResourceModel struct {"));
        assert!(s.contains("type StaticSecretResource struct {"));
    }

    #[test]
    fn model_fields_carry_tfsdk_tags_and_framework_types() {
        let spec = sample_spec();
        let s = print_file(&emit_resource(&spec).unwrap());
        assert!(s.contains("Name types.String `tfsdk:\"name\"`"));
        assert!(s.contains("Value types.String `tfsdk:\"value\"`"));
        assert!(s.contains("ID types.String `tfsdk:\"id\"`"));
    }

    #[test]
    fn constructor_returns_resource_resource() {
        let spec = sample_spec();
        let s = print_file(&emit_resource(&spec).unwrap());
        assert!(s.contains("func NewStaticSecretResource() resource.Resource {"));
        assert!(s.contains("return &StaticSecretResource{}"));
    }

    #[test]
    fn schema_emits_required_optional_computed_flags() {
        let spec = sample_spec();
        let s = print_file(&emit_resource(&spec).unwrap());
        // the required `name` attribute, keyed by a quoted string map key
        assert!(s.contains("\"name\": schema.StringAttribute{"));
        assert!(s.contains("Required: true,"));
        // Metadata uses the canonical `req.ProviderTypeName + "_x"` idiom
        assert!(s.contains("resp.TypeName = req.ProviderTypeName + \"_static_secret\""));
        // the sensitive `value` attribute
        assert!(s.contains("Sensitive: true,"));
        // the computed `id` attribute
        assert!(s.contains("Computed: true,"));
    }

    #[test]
    fn all_crud_methods_present() {
        let spec = sample_spec();
        let s = print_file(&emit_resource(&spec).unwrap());
        for verb in ["Create", "Read", "Update", "Delete"] {
            assert!(
                s.contains(&format!("func (r *StaticSecretResource) {verb}(")),
                "missing {verb}"
            );
        }
    }

    #[test]
    fn interface_assertion_var_present() {
        let spec = sample_spec();
        let s = print_file(&emit_resource(&spec).unwrap());
        assert!(s.contains("var _ resource.Resource = &StaticSecretResource{}"));
    }

    #[test]
    fn deterministic_emit_for_identical_spec() {
        let a = print_file(&emit_resource(&sample_spec()).unwrap());
        let b = print_file(&emit_resource(&sample_spec()).unwrap());
        assert_eq!(a, b);
    }

    #[test]
    fn framework_imports_present() {
        let spec = sample_spec();
        let s = print_file(&emit_resource(&spec).unwrap());
        assert!(s.contains("github.com/hashicorp/terraform-plugin-framework/resource"));
        assert!(s.contains("github.com/hashicorp/terraform-plugin-framework/types"));
    }
}
