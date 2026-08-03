# go-synthesizer

Typed AST for structurally correct Go source code generation.

# Dual-layer design (deliberate, with a planned unification follow-up)
This crate currently carries **two** typed Go-source models that coexist
at the crate root. This is an intentional intermediate state: the richer
"file" layer was relocated **verbatim** from `iac-forge/src/goast.rs` so
that downstream consumers (notably `crossplane-forge`, which drives
a downstream vendor provider) can import every item from `go_synthesizer` with

Part of the pleme-io code-generation pipeline:

```
sekkei -> takumi -> openapi-forge / iac-forge -> backend renderers
```

## License

MIT — see [LICENSE](./LICENSE).
