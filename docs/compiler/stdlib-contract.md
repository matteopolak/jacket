# Compiler and standard-library contract

The compiler reaches into the Jai prelude and a few modules by name and by byte layout. `crates/jaic/src/lang_items.rs` lists every name; `crates/jaic/src/layout.rs` computes every size and offset of the built-in aggregates (`string`, array views, resizable arrays, `Any`, `Allocator`) from the target's `DataLayout`. Nothing else writes these down.

## How it works

- `lang_items::modules` names the modules the compiler loads itself (`prelude`, `Preload`, `Runtime_Support`) and which are never offered as imports.
- `lang_items::record_fields` names the fields (`__address`, `__builtin`) of the records a metaprogram receives.
- `lang_items::PreludeType` is the closed set of prelude types the compiler builds values of (`Allocator`, `Type_Info` variants, `Source_Code_Location`, ...), looked up with `Compiler::preload_type`.
- `DataLayout::builtins()` gives the tables: a view is an `s64` count then the data pointer, a resizable array adds the `s64` capacity and an `Allocator` (two pointers), `Any` is a `*Type_Info` then a `*void`. Offsets follow the target's pointer size and `s64` alignment.

## How to change it

- A new prelude type: add a `PreludeType` member. `crates/jaic/tests/lang_items.rs` resolves every member against the real prelude.
- A changed prelude struct that a table mirrors fails `the_builtin_layouts_match_the_prelude` until the table is updated.
- Read the pointer size from `DataLayout`, never a constant. Read and write program bytes through `DataLayout::read_uint`/`write_uint` and friends; clippy bans `from_/to_{le,ne}_bytes` outside host-only code, which carries an `#[allow]` with a reason.

## `#compiler` builtins

Public `#compiler` operations bind directly to Rust handlers. Three private syntax queries remain for `Program_Print`; there are no Jai-facing workspace or record-builder transports.

- `build/public_api/mod.rs` lists the public operation names. An explicit `#compiler "name"` selector beats the procedure name.
- Binding checks the parameter and result shapes and captures the declaration's real aggregate layouts and defaults. Compile-time calls enter Rust; runtime calls run the Jai body, or fail as compile-time only when there is none.
- Workspace options are decoded from public named fields into typed Rust settings (`builtin_keys.rs`). Target enums are translated by member name, so unsupported targets give option diagnostics.
- `__jaic_syntax_int`, `__jaic_syntax_string` and `__jaic_syntax_child` expose compiler-owned attributes missing from public code records. `Syntax_Key` in `stdlib/internal/program-print/attributes.jai` must match Rust `SyntaxKey`.
- Query strings, views and message records belong to the compiler. Copy them before owning or freeing; `Default_Metaprogram` borrows `get_toplevel_command_line` rather than freeing it.

To change them:

- Public operation: update the registry, binding validation and handler under `build/public_api`, plus the Jai declaration.
- New syntax form for `compiler_get_code` / `compiler_modify_procedure`: add a mapping in `public_api/convert`, reading values only through the `Convert` readers so replay fingerprints stay complete. `code_roundtrips.rs` covers it.
- Syntax attribute: add the same key in Jai and Rust, and update the exporter and printers. `typed_builtin_keys.rs` and `meta_op_names.rs` guard both.
