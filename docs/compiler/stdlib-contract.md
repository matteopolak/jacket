# Compiler and standard-library contract

## What it is

The compiler is written in Rust but reaches into the Jai prelude and a few modules by name and by byte layout. `crates/jaic/src/lang_items.rs` lists every name; `crates/jaic/src/layout.rs` lists every size and offset of the aggregates the language builds in (`string`, array views, resizable arrays, `Any`). Nothing else in the compiler writes these down.

## How it works

- `lang_items::modules` names the modules the compiler loads itself (`prelude`, `Preload`, `Runtime_Support`; `INTERNAL` is the list that is never offered as an import) and the `Runtime_Support` type its compile-time context keeps temporary storage in. `BootstrapModule`, `Compiler::load_bootstrap`, the standard-library index and the language server's auto-import read these.
- `lang_items::record_fields` names the fields (`__address`, `__builtin`) the type records a metaprogram receives carry; `record_build.rs` reads them back and `sema/code_export` writes them.
- `lang_items::PreludeType` is the closed set of prelude types the compiler builds values of (`Allocator`, `Type_Info` and its variants, `Source_Code_Location`, ...), looked up with `Compiler::preload_type`.
- `layout` is 64-bit only, stated once: `POINTER_SIZE` is a constant (there is no per-target pointer size), `view` is a count then a data pointer (`string` and `[] T`), `resizable` adds the allocated capacity and an `Allocator`, `any` is a `*Type_Info` then a `*void`. `TypeStore::size_of`/`align_of`, the builtin members (`count`, `data`, `allocated`, `allocator`, `type`, `value_pointer`), constant freezing and the type-info writers use these names, not numbers.

## How to change it

A new prelude type the compiler needs: add a `PreludeType` member and its name; a unit test in `lang_items.rs` checks the prelude's sources declare it, and `crates/jaic/tests/lang_items.rs` resolves every member against the real prelude. A changed prelude struct that a table mirrors (`Allocator` is `layout::ALLOCATOR_SIZE` bytes) fails `the_builtin_layouts_match_the_prelude` until the table is updated. Supporting a 32-bit target would mean replacing `POINTER_SIZE` with a target setting everywhere the interpreter, IR and both backends assume eight-byte pointers; today they all do.

## Configuration

None. These are constants of the language, not options.

## Dependencies

`stdlib/prelude`, `stdlib/Runtime_Support`, and the metaprogram records of the `Compiler` module (see [compiler builtins](compiler-builtins.md)).
