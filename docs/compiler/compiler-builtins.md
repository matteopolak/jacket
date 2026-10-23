# Compiler builtins

## What it is

Public `#compiler` operations bind directly to Rust compiler handlers. Three private syntax metadata queries remain for `Program_Print`; there are no Jai-facing workspace or record-builder transports.

## How it works

`build/public_api/mod.rs` lists public operation names. An explicit `#compiler "name"` selector takes precedence over the procedure name. Binding validates the parameter/result shapes and captures the selected declaration's actual aggregate layouts and defaults. Compile-time calls enter Rust; runtime calls execute the Jai body, or fail as compile-time only when there is none. Handlers read their arguments through those layouts and call typed engine functions in `build/engine.rs`; no argument is passed through interpreter memory the handler made itself.

Workspace options are decoded from public named fields into typed Rust settings. `builtin_keys.rs` keeps internal workspace selector enums and validates public option enum values. These internal selectors are not declared in Jai. Target enums are translated by their declared member names, so unsupported targets produce public option diagnostics instead of internal selector errors. Deferred custom-link messages retain the schema that owns their mutable exit-code field.

`__jaic_syntax_int`, `__jaic_syntax_string`, and `__jaic_syntax_child` expose compiler-owned attributes absent from public code-record layouts. `Syntax_Key` in `stdlib/internal/program-print/attributes.jai` matches Rust `SyntaxKey` in `builtin_keys.rs`. The exporter writes named syntax fields; the queries validate keys and find records by association without dereferencing arbitrary pointers.

Query strings, views and message records are compiler-owned and remain valid for the compilation. Consumers must copy storage before taking ownership or freeing it; in particular, `Default_Metaprogram` borrows `get_toplevel_command_line` instead of freeing it as a Jai allocation.

## How to change it

For public operations, update the registry, binding validation and handler under `build/public_api`, plus the public declaration. The record-to-tree converter in `public_api/convert` builds the trees `compiler_get_code` and `compiler_modify_procedure` return; a new syntax form needs a mapping there, and every value it reads goes through the `Convert` readers so replay fingerprints stay complete. `code_roundtrips.rs` covers edited and unsupported nodes through real compilation and execution. Test bodyless compile-time execution and runtime bodies. For a syntax attribute, add the same key/value in Jai and Rust and update the exporter and printers. `typed_builtin_keys.rs` checks shared enum values; `meta_op_names.rs` ensures only the retained syntax hooks are declared and bindable.

## Configuration

The selected stdlib and prelude supply the public declarations. No separate builtin configuration is required.

## Dependencies

Sema procedure binding, interpreter hooks, the Rust workspace engine, neutral records, and `stdlib/internal/program-print`.
