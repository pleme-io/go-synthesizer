/// Go AST — irreducible primitives for Go source code generation.
///
/// Go's algebraic basis: 10 declarations + 12 expressions + 8 statements + 10 types = 40 primitives.
/// gofmt-compatible output with tab indentation.
///
/// This is the 22nd AST domain in the typescape.

// ══════════════════════════════════════════════════════════════
// Declarations — the top-level constructs
// ══════════════════════════════════════════════════════════════

/// Top-level Go AST node.
#[derive(Debug, Clone, PartialEq)]
pub enum GoNode {
    /// `// comment`
    Comment(String),
    /// Empty line
    Blank,
    /// `package name`
    Package(String),
    /// `import "path"` or `import ( ... )`
    Import(Vec<GoImport>),
    /// `type Name struct { ... }`
    Struct {
        name: String,
        fields: Vec<GoField>,
        doc: Option<String>,
    },
    /// `type Name interface { ... }`
    Interface {
        name: String,
        methods: Vec<GoMethodSig>,
        doc: Option<String>,
    },
    /// `func Name(args) returns { body }`
    Func {
        name: String,
        args: Vec<GoParam>,
        returns: Vec<GoType>,
        body: Vec<GoStmt>,
        doc: Option<String>,
    },
    /// `func (recv Type) Name(args) returns { body }`
    Method {
        receiver: GoParam,
        name: String,
        args: Vec<GoParam>,
        returns: Vec<GoType>,
        body: Vec<GoStmt>,
        doc: Option<String>,
    },
    /// `var name Type = value`
    VarDecl {
        name: String,
        var_type: Option<GoType>,
        value: Option<GoExpr>,
    },
    /// `const name Type = value`
    ConstDecl {
        name: String,
        const_type: Option<GoType>,
        value: GoExpr,
    },
    /// `type Name = Type` or `type Name Type`
    TypeAlias {
        name: String,
        target: GoType,
        is_alias: bool,
    },
}

/// An import entry: `"path"` or `alias "path"`.
#[derive(Debug, Clone, PartialEq)]
pub struct GoImport {
    pub path: String,
    pub alias: Option<String>,
}

/// A struct field: `Name Type \`json:"name"\``
#[derive(Debug, Clone, PartialEq)]
pub struct GoField {
    pub name: String,
    pub field_type: GoType,
    pub tag: Option<String>,
    pub doc: Option<String>,
}

/// An interface method signature.
#[derive(Debug, Clone, PartialEq)]
pub struct GoMethodSig {
    pub name: String,
    pub args: Vec<GoParam>,
    pub returns: Vec<GoType>,
}

/// A function/method parameter.
#[derive(Debug, Clone, PartialEq)]
pub struct GoParam {
    pub name: String,
    pub param_type: GoType,
}

// ══════════════════════════════════════════════════════════════
// Expressions — the value primitives
// ══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq)]
pub enum GoExpr {
    /// Bare identifier
    Ident(String),
    /// `"string"`
    Str(String),
    /// Integer literal
    Int(i64),
    /// Float literal
    Float(f64),
    /// `true` / `false`
    Bool(bool),
    /// `nil`
    Nil,
    /// `func(args...)`
    Call {
        func: Box<GoExpr>,
        args: Vec<GoExpr>,
    },
    /// `expr.field` or `pkg.Func`
    Selector {
        expr: Box<GoExpr>,
        field: String,
    },
    /// `expr[index]`
    Index {
        expr: Box<GoExpr>,
        index: Box<GoExpr>,
    },
    /// `Type{ field: value, ... }` — composite literal
    Composite {
        comp_type: GoType,
        fields: Vec<(String, GoExpr)>,
    },
    /// `expr[low:high]`
    Slice {
        expr: Box<GoExpr>,
        low: Option<Box<GoExpr>>,
        high: Option<Box<GoExpr>>,
    },
    /// `&expr`
    Addr(Box<GoExpr>),
    /// `*expr`
    Deref(Box<GoExpr>),
    /// `fmt.Sprintf("format", args...)` — format string
    Sprintf {
        format: String,
        args: Vec<GoExpr>,
    },
    /// Binary operator: `left op right`
    BinOp {
        left: Box<GoExpr>,
        op: String,
        right: Box<GoExpr>,
    },
    /// `expr.(Type)` — type assertion
    TypeAssert {
        expr: Box<GoExpr>,
        assert_type: GoType,
    },
    /// `expr...` — variadic unpacking (spread)
    Spread(Box<GoExpr>),
    /// `map[K]V{ key: value, ... }` — map literal
    MapLit {
        key_type: GoType,
        val_type: GoType,
        entries: Vec<(GoExpr, GoExpr)>,
    },
    /// `fmt.Errorf("message: %w", err)` — error wrapping
    Errorf {
        format: String,
        args: Vec<GoExpr>,
    },
}

// ══════════════════════════════════════════════════════════════
// Statements — the execution primitives
// ══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq)]
pub enum GoStmt {
    /// `name = value`
    Assign {
        target: GoExpr,
        value: GoExpr,
    },
    /// `name := value`
    ShortAssign {
        name: String,
        value: GoExpr,
    },
    /// `return expr, ...`
    Return(Vec<GoExpr>),
    /// `if cond { then } else { else }`
    If {
        init: Option<Box<GoStmt>>,
        cond: GoExpr,
        body: Vec<GoStmt>,
        else_body: Option<Vec<GoStmt>>,
    },
    /// `for init; cond; post { body }` or `for range`
    For {
        init: Option<Box<GoStmt>>,
        cond: Option<GoExpr>,
        post: Option<Box<GoStmt>>,
        body: Vec<GoStmt>,
    },
    /// `for key, value := range expr { body }`
    ForRange {
        key: Option<String>,
        value: Option<String>,
        expr: GoExpr,
        body: Vec<GoStmt>,
    },
    /// `switch expr { case ... }`
    Switch {
        expr: Option<GoExpr>,
        cases: Vec<GoCase>,
    },
    /// `defer func() { body }()`
    Defer(GoExpr),
    /// `go func() { body }()`
    Go(GoExpr),
    /// Expression as statement (function calls, etc.)
    Expr(GoExpr),
    /// Blank line
    Blank,
}

/// A switch case.
#[derive(Debug, Clone, PartialEq)]
pub struct GoCase {
    pub exprs: Vec<GoExpr>, // empty = default
    pub body: Vec<GoStmt>,
}

// ══════════════════════════════════════════════════════════════
// Types — the type system primitives
// ══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq)]
pub enum GoType {
    /// Named type: `string`, `int`, `error`, `context.Context`
    Named(String),
    /// `*Type`
    Pointer(Box<GoType>),
    /// `[]Type`
    Slice(Box<GoType>),
    /// `map[Key]Value`
    Map(Box<GoType>, Box<GoType>),
    /// `chan Type` or `chan<- Type` or `<-chan Type`
    Chan {
        dir: ChanDir,
        elem: Box<GoType>,
    },
    /// `func(args) returns`
    Func {
        args: Vec<GoType>,
        returns: Vec<GoType>,
    },
    /// `struct { fields }`
    Struct(Vec<GoField>),
    /// `interface { methods }`
    Interface(Vec<GoMethodSig>),
    /// `[N]Type`
    Array(usize, Box<GoType>),
    /// `any` / `interface{}`
    Any,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChanDir {
    Both,
    Send,
    Recv,
}

// ══════════════════════════════════════════════════════════════
// Emission — deterministic, gofmt-compatible (tab indentation)
// ══════════════════════════════════════════════════════════════

impl GoNode {
    pub fn emit(&self, indent: usize) -> String {
        let pad = "\t".repeat(indent);
        match self {
            Self::Comment(text) => {
                if text.contains('\n') {
                    text.lines().map(|l| format!("{pad}// {l}")).collect::<Vec<_>>().join("\n")
                } else {
                    format!("{pad}// {text}")
                }
            }
            Self::Blank => String::new(),
            Self::Package(name) => format!("{pad}package {name}"),
            Self::Import(imports) => {
                if imports.len() == 1 {
                    let i = &imports[0];
                    match &i.alias {
                        Some(a) => format!("{pad}import {a} \"{path}\"", path = i.path),
                        None => format!("{pad}import \"{path}\"", path = i.path),
                    }
                } else {
                    let mut out = format!("{pad}import (\n");
                    for i in imports {
                        match &i.alias {
                            Some(a) => out.push_str(&format!("{pad}\t{a} \"{path}\"\n", path = i.path)),
                            None => out.push_str(&format!("{pad}\t\"{path}\"\n", path = i.path)),
                        }
                    }
                    out.push_str(&format!("{pad})"));
                    out
                }
            }
            Self::Struct { name, fields, doc } => {
                let mut out = String::new();
                if let Some(d) = doc {
                    out.push_str(&format!("{pad}// {d}\n"));
                }
                out.push_str(&format!("{pad}type {name} struct {{\n"));
                for f in fields {
                    if let Some(d) = &f.doc {
                        out.push_str(&format!("{pad}\t// {d}\n"));
                    }
                    let tag = f.tag.as_ref().map(|t| format!(" `{t}`")).unwrap_or_default();
                    out.push_str(&format!("{pad}\t{} {}{tag}\n", f.name, f.field_type.emit()));
                }
                out.push_str(&format!("{pad}}}"));
                out
            }
            Self::Interface { name, methods, doc } => {
                let mut out = String::new();
                if let Some(d) = doc {
                    out.push_str(&format!("{pad}// {d}\n"));
                }
                out.push_str(&format!("{pad}type {name} interface {{\n"));
                for m in methods {
                    let args = m.args.iter().map(|a| format!("{} {}", a.name, a.param_type.emit())).collect::<Vec<_>>().join(", ");
                    let rets = emit_go_returns(&m.returns);
                    out.push_str(&format!("{pad}\t{name}({args}){rets}\n", name = m.name));
                }
                out.push_str(&format!("{pad}}}"));
                out
            }
            Self::Func { name, args, returns, body, doc } => {
                let mut out = String::new();
                if let Some(d) = doc {
                    out.push_str(&format!("{pad}// {d}\n"));
                }
                let args_str = args.iter().map(|a| format!("{} {}", a.name, a.param_type.emit())).collect::<Vec<_>>().join(", ");
                let rets = emit_go_returns(returns);
                out.push_str(&format!("{pad}func {name}({args_str}){rets} {{\n"));
                for s in body {
                    out.push_str(&s.emit(indent + 1));
                    out.push('\n');
                }
                out.push_str(&format!("{pad}}}"));
                out
            }
            Self::Method { receiver, name, args, returns, body, doc } => {
                let mut out = String::new();
                if let Some(d) = doc {
                    out.push_str(&format!("{pad}// {d}\n"));
                }
                let recv = format!("{} {}", receiver.name, receiver.param_type.emit());
                let args_str = args.iter().map(|a| format!("{} {}", a.name, a.param_type.emit())).collect::<Vec<_>>().join(", ");
                let rets = emit_go_returns(returns);
                out.push_str(&format!("{pad}func ({recv}) {name}({args_str}){rets} {{\n"));
                for s in body {
                    out.push_str(&s.emit(indent + 1));
                    out.push('\n');
                }
                out.push_str(&format!("{pad}}}"));
                out
            }
            Self::VarDecl { name, var_type, value } => {
                let ty = var_type.as_ref().map(|t| format!(" {}", t.emit())).unwrap_or_default();
                let val = value.as_ref().map(|v| format!(" = {}", v.emit())).unwrap_or_default();
                format!("{pad}var {name}{ty}{val}")
            }
            Self::ConstDecl { name, const_type, value } => {
                let ty = const_type.as_ref().map(|t| format!(" {}", t.emit())).unwrap_or_default();
                format!("{pad}const {name}{ty} = {}", value.emit())
            }
            Self::TypeAlias { name, target, is_alias } => {
                if *is_alias {
                    format!("{pad}type {name} = {}", target.emit())
                } else {
                    format!("{pad}type {name} {}", target.emit())
                }
            }
        }
    }
}

impl GoExpr {
    pub fn emit(&self) -> String {
        match self {
            Self::Ident(s) => s.clone(),
            Self::Str(s) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")),
            Self::Int(n) => n.to_string(),
            Self::Float(f) => format!("{f}"),
            Self::Bool(b) => b.to_string(),
            Self::Nil => "nil".into(),
            Self::Call { func, args } => {
                let a = args.iter().map(|a| a.emit()).collect::<Vec<_>>().join(", ");
                format!("{}({a})", func.emit())
            }
            Self::Selector { expr, field } => format!("{}.{field}", expr.emit()),
            Self::Index { expr, index } => format!("{}[{}]", expr.emit(), index.emit()),
            Self::Composite { comp_type, fields } => {
                if fields.is_empty() {
                    format!("{}{{}}", comp_type.emit())
                } else {
                    let f = fields.iter().map(|(k, v)| format!("{k}: {}", v.emit())).collect::<Vec<_>>().join(", ");
                    format!("{}{{ {f} }}", comp_type.emit())
                }
            }
            Self::Slice { expr, low, high } => {
                let l = low.as_ref().map(|e| e.emit()).unwrap_or_default();
                let h = high.as_ref().map(|e| e.emit()).unwrap_or_default();
                format!("{}[{l}:{h}]", expr.emit())
            }
            Self::Addr(expr) => format!("&{}", expr.emit()),
            Self::Deref(expr) => format!("*{}", expr.emit()),
            Self::Sprintf { format: fmt, args } => {
                let a = std::iter::once(GoExpr::Str(fmt.clone()))
                    .chain(args.iter().cloned())
                    .map(|a| a.emit())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("fmt.Sprintf({a})")
            }
            Self::BinOp { left, op, right } => format!("{} {op} {}", left.emit(), right.emit()),
            Self::TypeAssert { expr, assert_type } => format!("{}.({})", expr.emit(), assert_type.emit()),
            Self::Spread(expr) => format!("{}...", expr.emit()),
            Self::MapLit { key_type, val_type, entries } => {
                if entries.is_empty() {
                    format!("map[{}]{}{{}}", key_type.emit(), val_type.emit())
                } else {
                    let inner = entries.iter()
                        .map(|(k, v)| format!("\t{}: {},", k.emit(), v.emit()))
                        .collect::<Vec<_>>()
                        .join("\n");
                    format!("map[{}]{}{{\n{}\n}}", key_type.emit(), val_type.emit(), inner)
                }
            }
            Self::Errorf { format: fmt, args } => {
                let all: Vec<String> = std::iter::once(GoExpr::Str(fmt.clone()))
                    .chain(args.iter().cloned())
                    .map(|a| a.emit())
                    .collect();
                format!("fmt.Errorf({})", all.join(", "))
            }
        }
    }
}

impl GoStmt {
    pub fn emit(&self, indent: usize) -> String {
        let pad = "\t".repeat(indent);
        match self {
            Self::Assign { target, value } => format!("{pad}{} = {}", target.emit(), value.emit()),
            Self::ShortAssign { name, value } => format!("{pad}{name} := {}", value.emit()),
            Self::Return(exprs) => {
                if exprs.is_empty() {
                    format!("{pad}return")
                } else {
                    let vals = exprs.iter().map(|e| e.emit()).collect::<Vec<_>>().join(", ");
                    format!("{pad}return {vals}")
                }
            }
            Self::If { init, cond, body, else_body } => {
                let init_str = init.as_ref().map(|s| format!("{}; ", s.emit(0).trim())).unwrap_or_default();
                let mut out = format!("{pad}if {init_str}{} {{\n", cond.emit());
                for s in body { out.push_str(&s.emit(indent + 1)); out.push('\n'); }
                match else_body {
                    Some(eb) => {
                        out.push_str(&format!("{pad}}} else {{\n"));
                        for s in eb { out.push_str(&s.emit(indent + 1)); out.push('\n'); }
                        out.push_str(&format!("{pad}}}"));
                    }
                    None => out.push_str(&format!("{pad}}}")),
                }
                out
            }
            Self::For { init, cond, post, body } => {
                let parts: Vec<String> = vec![
                    init.as_ref().map(|s| s.emit(0).trim().to_string()).unwrap_or_default(),
                    cond.as_ref().map(|e| e.emit()).unwrap_or_default(),
                    post.as_ref().map(|s| s.emit(0).trim().to_string()).unwrap_or_default(),
                ];
                let header = parts.join("; ").trim_end_matches("; ").trim().to_string();
                let mut out = format!("{pad}for {header} {{\n");
                for s in body { out.push_str(&s.emit(indent + 1)); out.push('\n'); }
                out.push_str(&format!("{pad}}}"));
                out
            }
            Self::ForRange { key, value, expr, body } => {
                let binding = match (key, value) {
                    (Some(k), Some(v)) => format!("{k}, {v}"),
                    (Some(k), None) => k.clone(),
                    (None, Some(v)) => format!("_, {v}"),
                    (None, None) => "_".into(),
                };
                let mut out = format!("{pad}for {binding} := range {} {{\n", expr.emit());
                for s in body { out.push_str(&s.emit(indent + 1)); out.push('\n'); }
                out.push_str(&format!("{pad}}}"));
                out
            }
            Self::Switch { expr, cases } => {
                let header = expr.as_ref().map(|e| format!(" {}", e.emit())).unwrap_or_default();
                let mut out = format!("{pad}switch{header} {{\n");
                for c in cases {
                    if c.exprs.is_empty() {
                        out.push_str(&format!("{pad}default:\n"));
                    } else {
                        let vals = c.exprs.iter().map(|e| e.emit()).collect::<Vec<_>>().join(", ");
                        out.push_str(&format!("{pad}case {vals}:\n"));
                    }
                    for s in &c.body { out.push_str(&s.emit(indent + 1)); out.push('\n'); }
                }
                out.push_str(&format!("{pad}}}"));
                out
            }
            Self::Defer(expr) => format!("{pad}defer {}", expr.emit()),
            Self::Go(expr) => format!("{pad}go {}", expr.emit()),
            Self::Expr(expr) => format!("{pad}{}", expr.emit()),
            Self::Blank => String::new(),
        }
    }
}

impl GoType {
    pub fn emit(&self) -> String {
        match self {
            Self::Named(name) => name.clone(),
            Self::Pointer(inner) => format!("*{}", inner.emit()),
            Self::Slice(inner) => format!("[]{}", inner.emit()),
            Self::Map(key, val) => format!("map[{}]{}", key.emit(), val.emit()),
            Self::Chan { dir, elem } => match dir {
                ChanDir::Both => format!("chan {}", elem.emit()),
                ChanDir::Send => format!("chan<- {}", elem.emit()),
                ChanDir::Recv => format!("<-chan {}", elem.emit()),
            },
            Self::Func { args, returns } => {
                let a = args.iter().map(|t| t.emit()).collect::<Vec<_>>().join(", ");
                let r = emit_go_returns(returns);
                format!("func({a}){r}")
            }
            Self::Struct(fields) => {
                let f = fields.iter().map(|f| format!("{} {}", f.name, f.field_type.emit())).collect::<Vec<_>>().join("; ");
                format!("struct {{ {f} }}")
            }
            Self::Interface(methods) => {
                if methods.is_empty() {
                    "interface{}".into()
                } else {
                    let m = methods.iter().map(|m| {
                        let a = m.args.iter().map(|a| a.param_type.emit()).collect::<Vec<_>>().join(", ");
                        let r = emit_go_returns(&m.returns);
                        format!("{}({a}){r}", m.name)
                    }).collect::<Vec<_>>().join("; ");
                    format!("interface {{ {m} }}")
                }
            }
            Self::Array(n, inner) => format!("[{n}]{}", inner.emit()),
            Self::Any => "any".into(),
        }
    }
}

fn emit_go_returns(returns: &[GoType]) -> String {
    match returns.len() {
        0 => String::new(),
        1 => format!(" {}", returns[0].emit()),
        _ => {
            let r = returns.iter().map(|t| t.emit()).collect::<Vec<_>>().join(", ");
            format!(" ({r})")
        }
    }
}

// ══════════════════════════════════════════════════════════════
// Constructors
// ══════════════════════════════════════════════════════════════

impl GoImport {
    #[must_use]
    pub fn new(path: &str) -> Self { Self { path: path.into(), alias: None } }

    #[must_use]
    pub fn with_alias(path: &str, alias: &str) -> Self { Self { path: path.into(), alias: Some(alias.into()) } }
}

impl GoField {
    #[must_use]
    pub fn new(name: &str, field_type: GoType) -> Self { Self { name: name.into(), field_type, tag: None, doc: None } }

    #[must_use]
    pub fn with_tag(mut self, tag: &str) -> Self { self.tag = Some(tag.into()); self }

    #[must_use]
    pub fn with_doc(mut self, doc: &str) -> Self { self.doc = Some(doc.into()); self }
}

impl GoParam {
    #[must_use]
    pub fn new(name: &str, param_type: GoType) -> Self { Self { name: name.into(), param_type } }
}

/// Emit a sequence of Go nodes as a complete file.
#[must_use]
pub fn emit_file(nodes: &[GoNode]) -> String {
    let mut out = nodes.iter().map(|n| n.emit(0)).collect::<Vec<_>>().join("\n");
    if !out.ends_with('\n') { out.push('\n'); }
    out
}

// ══════════════════════════════════════════════════════════════
// Tests
// ══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_emits() {
        assert_eq!(GoNode::Package("main".into()).emit(0), "package main");
    }

    #[test]
    fn single_import() {
        let node = GoNode::Import(vec![GoImport::new("fmt")]);
        assert_eq!(node.emit(0), "import \"fmt\"");
    }

    #[test]
    fn multi_import() {
        let node = GoNode::Import(vec![
            GoImport::new("fmt"),
            GoImport::new("os"),
            GoImport::with_alias("github.com/pkg/errors", "errors"),
        ]);
        let out = node.emit(0);
        assert!(out.contains("import ("));
        assert!(out.contains("\"fmt\""));
        assert!(out.contains("errors \"github.com/pkg/errors\""));
    }

    #[test]
    fn struct_emits() {
        let node = GoNode::Struct {
            name: "Config".into(),
            doc: Some("Config holds configuration.".into()),
            fields: vec![
                GoField::new("Name", GoType::Named("string".into())).with_tag("json:\"name\""),
                GoField::new("Port", GoType::Named("int".into())).with_tag("json:\"port\""),
            ],
        };
        let out = node.emit(0);
        assert!(out.contains("type Config struct {"));
        assert!(out.contains("Name string `json:\"name\"`"));
        assert!(out.contains("Port int `json:\"port\"`"));
    }

    #[test]
    fn func_emits() {
        let node = GoNode::Func {
            name: "main".into(),
            args: vec![],
            returns: vec![],
            doc: None,
            body: vec![
                GoStmt::Expr(GoExpr::Call {
                    func: Box::new(GoExpr::Selector {
                        expr: Box::new(GoExpr::Ident("fmt".into())),
                        field: "Println".into(),
                    }),
                    args: vec![GoExpr::Str("hello".into())],
                }),
            ],
        };
        let out = node.emit(0);
        assert!(out.contains("func main()"));
        assert!(out.contains("fmt.Println(\"hello\")"));
    }

    #[test]
    fn method_emits() {
        let node = GoNode::Method {
            receiver: GoParam::new("c", GoType::Pointer(Box::new(GoType::Named("Config".into())))),
            name: "String".into(),
            args: vec![],
            returns: vec![GoType::Named("string".into())],
            body: vec![GoStmt::Return(vec![GoExpr::Str("config".into())])],
            doc: None,
        };
        let out = node.emit(0);
        assert!(out.contains("func (c *Config) String() string {"));
    }

    #[test]
    fn if_else() {
        let stmt = GoStmt::If {
            init: None,
            cond: GoExpr::BinOp {
                left: Box::new(GoExpr::Ident("err".into())),
                op: "!=".into(),
                right: Box::new(GoExpr::Nil),
            },
            body: vec![GoStmt::Return(vec![GoExpr::Ident("err".into())])],
            else_body: None,
        };
        let out = stmt.emit(0);
        assert!(out.contains("if err != nil {"));
    }

    #[test]
    fn for_range() {
        let stmt = GoStmt::ForRange {
            key: Some("i".into()),
            value: Some("v".into()),
            expr: GoExpr::Ident("items".into()),
            body: vec![GoStmt::Blank],
        };
        let out = stmt.emit(0);
        assert!(out.contains("for i, v := range items {"));
    }

    #[test]
    fn all_types_emit() {
        let types = vec![
            GoType::Named("string".into()),
            GoType::Pointer(Box::new(GoType::Named("Config".into()))),
            GoType::Slice(Box::new(GoType::Named("int".into()))),
            GoType::Map(Box::new(GoType::Named("string".into())), Box::new(GoType::Named("int".into()))),
            GoType::Chan { dir: ChanDir::Both, elem: Box::new(GoType::Named("int".into())) },
            GoType::Array(10, Box::new(GoType::Named("byte".into()))),
            GoType::Any,
        ];
        for t in &types {
            assert!(!t.emit().is_empty(), "{t:?}");
        }
    }

    #[test]
    fn deterministic() {
        let node = GoNode::Func {
            name: "test".into(), args: vec![], returns: vec![], doc: None,
            body: vec![GoStmt::Return(vec![])],
        };
        assert_eq!(node.emit(0), node.emit(0));
    }

    #[test]
    fn balanced_braces() {
        let node = GoNode::Struct {
            name: "Nested".into(), doc: None,
            fields: vec![GoField::new("Inner", GoType::Struct(vec![
                GoField::new("X", GoType::Named("int".into())),
            ]))],
        };
        let out = node.emit(0);
        let opens = out.chars().filter(|c| *c == '{').count();
        let closes = out.chars().filter(|c| *c == '}').count();
        assert_eq!(opens, closes);
    }

    #[test]
    fn trailing_newline() {
        let out = emit_file(&[GoNode::Package("main".into())]);
        assert!(out.ends_with('\n'));
    }

    #[test]
    fn tab_indentation() {
        let node = GoNode::Func {
            name: "f".into(), args: vec![], returns: vec![], doc: None,
            body: vec![GoStmt::Return(vec![GoExpr::Int(1)])],
        };
        let out = node.emit(0);
        assert!(out.contains("\treturn 1"), "Go uses tab indentation: {out}");
    }

    #[test]
    fn composite_literal() {
        let expr = GoExpr::Composite {
            comp_type: GoType::Named("Config".into()),
            fields: vec![("Name".into(), GoExpr::Str("test".into()))],
        };
        assert!(expr.emit().contains("Config{ Name: \"test\" }"));
    }

    #[test]
    fn switch_stmt() {
        let stmt = GoStmt::Switch {
            expr: Some(GoExpr::Ident("x".into())),
            cases: vec![
                GoCase { exprs: vec![GoExpr::Int(1)], body: vec![GoStmt::Return(vec![GoExpr::Str("one".into())])] },
                GoCase { exprs: vec![], body: vec![GoStmt::Return(vec![GoExpr::Str("other".into())])] },
            ],
        };
        let out = stmt.emit(0);
        assert!(out.contains("switch x {"));
        assert!(out.contains("case 1:"));
        assert!(out.contains("default:"));
    }

    #[test]
    fn complete_go_file() {
        let nodes = vec![
            GoNode::Package("main".into()),
            GoNode::Blank,
            GoNode::Import(vec![GoImport::new("fmt")]),
            GoNode::Blank,
            GoNode::Func {
                name: "main".into(),
                args: vec![],
                returns: vec![],
                doc: Some("main is the entry point.".into()),
                body: vec![GoStmt::Expr(GoExpr::Call {
                    func: Box::new(GoExpr::Selector {
                        expr: Box::new(GoExpr::Ident("fmt".into())),
                        field: "Println".into(),
                    }),
                    args: vec![GoExpr::Str("Hello, World!".into())],
                })],
            },
        ];
        let out = emit_file(&nodes);
        assert!(out.contains("package main"));
        assert!(out.contains("import \"fmt\""));
        assert!(out.contains("func main()"));
        assert!(out.contains("Hello, World!"));
        assert!(out.ends_with('\n'));
    }

    // ── New variant tests ────────────────────────────────────

    #[test]
    fn type_assert() {
        let expr = GoExpr::TypeAssert {
            expr: Box::new(GoExpr::Ident("data".into())),
            assert_type: GoType::Pointer(Box::new(GoType::Named("Config".into()))),
        };
        assert_eq!(expr.emit(), "data.(*Config)");
    }

    #[test]
    fn spread_operator() {
        let expr = GoExpr::Spread(Box::new(GoExpr::Ident("diags".into())));
        assert_eq!(expr.emit(), "diags...");
    }

    #[test]
    fn map_literal_empty() {
        let expr = GoExpr::MapLit {
            key_type: GoType::Named("string".into()),
            val_type: GoType::Named("int".into()),
            entries: vec![],
        };
        assert_eq!(expr.emit(), "map[string]int{}");
    }

    #[test]
    fn map_literal_with_entries() {
        let expr = GoExpr::MapLit {
            key_type: GoType::Named("string".into()),
            val_type: GoType::Named("string".into()),
            entries: vec![
                (GoExpr::Str("key".into()), GoExpr::Str("value".into())),
            ],
        };
        let out = expr.emit();
        assert!(out.contains("map[string]string{"));
        assert!(out.contains("\"key\": \"value\""));
    }

    #[test]
    fn errorf() {
        let expr = GoExpr::Errorf {
            format: "failed to read: %w".into(),
            args: vec![GoExpr::Ident("err".into())],
        };
        assert_eq!(expr.emit(), "fmt.Errorf(\"failed to read: %w\", err)");
    }

    #[test]
    fn terraform_diagnostics_pattern() {
        // Real terraform-plugin-framework pattern:
        // resp.Diagnostics.Append(diags...)
        let stmt = GoStmt::Expr(GoExpr::Call {
            func: Box::new(GoExpr::Selector {
                expr: Box::new(GoExpr::Selector {
                    expr: Box::new(GoExpr::Ident("resp".into())),
                    field: "Diagnostics".into(),
                }),
                field: "Append".into(),
            }),
            args: vec![GoExpr::Spread(Box::new(GoExpr::Ident("diags".into())))],
        });
        let out = stmt.emit(0);
        assert!(out.contains("resp.Diagnostics.Append(diags...)"));
    }

    #[test]
    fn terraform_type_assert_pattern() {
        // req.ProviderData.(*AkeylessClient)
        let expr = GoExpr::TypeAssert {
            expr: Box::new(GoExpr::Selector {
                expr: Box::new(GoExpr::Ident("req".into())),
                field: "ProviderData".into(),
            }),
            assert_type: GoType::Pointer(Box::new(GoType::Named("AkeylessClient".into()))),
        };
        assert_eq!(expr.emit(), "req.ProviderData.(*AkeylessClient)");
    }
}
