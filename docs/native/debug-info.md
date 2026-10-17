# Native debug information

## What it is

`jaic` emits DWARF (CodeView for `-windows-msvc` triples), so lldb and gdb can debug compiled Jai programs: breakpoints by `file.jai:line` or procedure name, stepping, backtraces with Jai names, and locals, parameters and globals with their Jai types. It is on by default at every optimisation level.

```
$ jaic main.jai -exe prog
$ lldb --batch -o 'b helper.jai:23' -o run -o bt -o 'frame variable' ./prog
```

## How it works

1. **Sema records** (`crates/jaic/src/sema/debug_info.rs`), only when `Options::debug_info` is set (never for `-interp`/`-check`):
   - `begin_func_debug` gives each lowered procedure an `ir::FuncDebug` side table (name, declaration line, lexical scopes, variables) as an `Option<Box<_>>` on `ir::Func`.
   - `debug_var` records a named local, parameter, named result or loop variable with its `TypeId`, line and the IR `Val` holding its address. Temporaries (names starting with `\0`) and `_` are skipped.
   - `debug_scope` maps sema block/macro scopes to `FuncDebug::scopes`; every `Inst::Loc` carries a scope index so the backend can use the right `DILexicalBlock`.
   - `collect_debug_types` (from `prepare_compiled_output`) converts each referenced `TypeId` into an `ir::DebugType` in `Program::debug_types`. Built-in aggregates become structs (`string` is `{count, data}`, `[..]` arrays `{count, data, allocated, allocator}`); `Type`, `Code` and procedure types become typedefs of `*void`.

2. **LLVM backend** (`crates/jaic-llvm/src/debuginfo.rs`, hooks in `lower.rs`):
   - One DIBuilder and compile unit per LLVM module; each split codegen unit has its own complete CU. Every function gets a `DISubprogram` with no linkage name.
   - Locations follow `Inst::Loc`. Blocks are lowered in reverse post-order, so each block starts from the location its first lowered predecessor ended with. Code before the first statement (parameter spills, stack-trace bookkeeping) is line 0, which debuggers skip.
   - Prologue: allocas have no location; scalar parameters are stored in the entry block without one; the entry branch carries the first statement's line, so `b proc` stops with parameters readable.
   - A variable whose address is a `SlotAddr` gets a `#dbg_declare` on that alloca. Aggregate parameters arrive by pointer, so the pointer is stored in a `dbg.addr` slot declared with `DW_OP_deref`. Other addresses are copied to such a slot (`Backend::set`).
   - Scalars are typedefs over DWARF base types, because LLDB names base types after the C type of the same size (`s64` would print as `long`). Structs are created as forward declarations first, so self-referential types work.

3. **CLI** (`crates/jaic-cli/src/main.rs`), macOS: the linker leaves DWARF in the object files and records their paths. An optimised build (`-O1`+) runs `write_dsym` after linking (`dsymutil` into `output.dSYM`; failure is a warning). An `-O0` build skips that and keeps its objects in `.build/` beside the output, which lldb reads through the debug map; this takes `dsymutil` (about 10% of a small build) off the edit-build-run loop. Each build deletes only its own stale objects, never `.build/` itself (parallel builds would race). Add `.build/` to `.gitignore`. A sanitized build writes the `.dSYM` at `-O0` too (the sanitizer symbolizer does not follow the debug map on every macOS). `JAIC_DSYM=1` writes the `.dSYM` and deletes the objects at any level, for binaries that leave the machine or tools needing a bundle (`atos`, samply). On Linux the linker copies the `.debug_*` sections into the executable.

## How to change it

- **New named variable**: call `self.debug_var(f, scope, name, span, ty, addr, 0)` where the entity is added; the address must stay valid for the variable's scope.
- **New type kind**: extend `describe_debug_type` (sema) and `DebugInfo::ty` (backend). Keys are `TypeId`s or the synthetic `ir::DEBUG_CHAR*` keys.
- Lines only come from statements in the procedure's own file; code from macros or `#insert` in other files keeps the caller's line.
- **Windows**: `DebugFormat::for_triple` selects `CodeView` (the `"CodeView"` module flag instead of `"Dwarf Version"`); `link` passes `/DEBUG /PDB:<stem>.pdb` because the objects are deleted after linking.
- Gotchas: in a function with a `DISubprogram` every call to a function with debug info needs a location (the verifier rejects it otherwise), so never build calls while the builder has no location. Do not add locations to allocas. Use the `raw` helpers, not inkwell's `insert_declare_*`, which wrap LLVM 19+ debug records as instructions.
- ELF: GNU ld and lld tail-merge `.debug_str`, so a DWARF 5 string offset may point into a longer string. That is valid, but `llvm-dwarfdump --verify` reports it; the test's `verify` helper tolerates exactly that diagnostic on non-Apple targets.

## Configuration

- Off with `-no_debug_info`, or `options.emit_debug_info = .NONE` in a metaprogram (also `set_optimization(*options, type, preserve_debug_info=false)`). `jaic_llvm::Options::debug_info`; `jaic::sema::Options::debug_info`.
- DWARF 4 on Apple, 5 elsewhere. The compile unit language is C99, so lldb expressions use C syntax (`p xs.data[1]`).
- `JAIC_DSYM=1` as above.

## Dependencies

LLVM's DIBuilder (through `inkwell::llvm_sys`); `dsymutil` on macOS. Tests: `crates/jaic-cli/tests/debug_info.rs` builds `tests/native/debug-info/`, checks `llvm-dwarfdump --verify` and drives `lldb --batch` (with `--no-lldbinit`, so user startup files cannot change launch or expression display); each test skips when its tool is missing. Related: [LLVM backend](llvm-backend.md), [IR](../compiler/ir.md), [stack traces](../compiler/stack-traces.md).
