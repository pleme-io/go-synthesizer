/// go-synthesizer — typed AST for structurally correct Go source generation.
///
/// Go's irreducible primitives (22nd AST domain in the typescape):
/// - 10 declaration nodes: Package, Import, Struct, Interface, Func, Method, VarDecl, ConstDecl, TypeAlias, InitFunc
/// - 12 expression nodes: Ident, Str, Int, Float, Bool, Nil, Call, Selector, Index, Composite, Slice, Addr
/// - 8 statement nodes: Assign, ShortAssign, Return, If, For, Switch, Defer, Go
/// - 10 type nodes: Named, Pointer, Slice, Map, Chan, Func, Struct, Interface, Array, Error
///
/// ANY valid Go can be expressed as compositions of these primitives.

mod node;
pub use node::*;
