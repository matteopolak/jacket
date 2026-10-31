# Compiler builtins

Public `#compiler` operations bind directly to Rust handlers. Three private syntax queries remain for `Program_Print`; there are no Jai-facing workspace or record-builder transports.

## How it works

- `build/public_api/mod.rs` lists the public operation names. An explicit `#compiler "name"` selector beats the procedure name.
- Binding checks the parameter and result shapes and captures the declaration's real aggregate layouts and defaults. Compile-time calls enter Rust; runtime calls run the Jai body, or fail as compile-time only when there is none.
- Workspace options are decoded from public named fields into typed Rust settings (`builtin_keys.rs`). Target enums are translated by member name, so unsupported targets give option diagnostics.
- `__jaic_syntax_int`, `__jaic_syntax_string` and `__jaic_syntax_child` expose compiler-owned attributes missing from public code records. `Syntax_Key` in `stdlib/internal/program-print/attributes.jai` must match Rust `SyntaxKey`.
- Query strings, views and message records belong to the compiler. Copy them before owning or freeing; `Default_Metaprogram` borrows `get_toplevel_command_line` rather than freeing it.

## How to change it

- Public operation: update the registry, binding validation and handler under `build/public_api`, plus the Jai declaration.
- New syntax form for `compiler_get_code` / `compiler_modify_procedure`: add a mapping in `public_api/convert`, reading values only through the `Convert` readers so replay fingerprints stay complete. `code_roundtrips.rs` covers it.
- Syntax attribute: add the same key in Jai and Rust, and update the exporter and printers. `typed_builtin_keys.rs` and `meta_op_names.rs` guard both.
