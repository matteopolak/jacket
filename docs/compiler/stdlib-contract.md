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

See also [compiler builtins](compiler-builtins.md).
