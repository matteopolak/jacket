# Compiler and standard-library contract

The compiler reaches into the Jai prelude and a few modules by name and by byte layout. `crates/jaic/src/lang_items.rs` lists every name; `crates/jaic/src/layout.rs` lists every size and offset of the built-in aggregates (`string`, array views, resizable arrays, `Any`). Nothing else writes these down.

## How it works

- `lang_items::modules` names the modules the compiler loads itself (`prelude`, `Preload`, `Runtime_Support`) and which are never offered as imports.
- `lang_items::record_fields` names the fields (`__address`, `__builtin`) of the records a metaprogram receives.
- `lang_items::PreludeType` is the closed set of prelude types the compiler builds values of (`Allocator`, `Type_Info` variants, `Source_Code_Location`, ...), looked up with `Compiler::preload_type`.
- `layout` is 64-bit only: `POINTER_SIZE` is a constant, a view is count then data pointer, a resizable array adds capacity and an `Allocator`, `Any` is a `*Type_Info` then a `*void`.

## How to change it

- A new prelude type: add a `PreludeType` member. `crates/jaic/tests/lang_items.rs` resolves every member against the real prelude.
- A changed prelude struct that a table mirrors fails `the_builtin_layouts_match_the_prelude` until the table is updated.
- A 32-bit target would need `POINTER_SIZE` to become a target setting everywhere the interpreter, IR and backends assume eight-byte pointers.

See also [compiler builtins](compiler-builtins.md).
