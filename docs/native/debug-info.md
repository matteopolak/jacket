# Native debug information

## What it is

`jaic build` emits DWARF debug information, so lldb and gdb can debug compiled Jai programs: breakpoints by `file.jai:line` or procedure name, stepping by source line, backtraces with Jai procedure names and source locations, and locals, parameters and globals with their Jai types (integers, floats, bools, pointers, strings, arrays, views, `[..]` arrays, structs, unions, enums). Like `jai`, it is on by default at every optimization level.

```
$ jaic build main.jai -o prog
$ lldb --batch -o 'b helper.jai:23' -o run -o bt -o 'frame variable' ./prog
  * frame #0: 0x0000000100001c24 prog`sum(xs=[] s64 @ 0x000000016fdfe5f0) at helper.jai:23:9
    frame #1: 0x00000001000016f8 prog`main at main.jai:19:5
    ...
([] s64) xs = {
  count = 3
  data = 0x000000016fdfe660
}
(s64) total = 0
(s64) it = 10
```

## How it works

Three layers:

1. **Sema records** (`crates/jaic/src/sema/debug_info.rs`), only when `Options::debug_info` is set (`jaic build` sets it; `jaic run`/`check` never pay for it):
   - `begin_func_debug` gives each lowered procedure an `ir::FuncDebug` side table: its written name (polymorph instances share it), declaration line, lexical scopes and variables. It is a `Option<Box<_>>` on `ir::Func`, so the interpreter's hot data does not grow.
   - `debug_var` records a named local, parameter (`arg` = position), named result or loop variable (`it`, `it_index`, `while x := ...`) with its `TypeId`, line and the IR `Val` holding its address. Compiler temporaries (names starting with `\0`) and `_` are skipped.
   - `debug_scope` maps sema `Block`/`Macro` scopes to `FuncDebug::scopes` entries (entry 0 is the procedure). Every statement's `Inst::Loc` carries its scope index, so the backend can put each instruction in the right `DILexicalBlock` and a debugger only shows the variables in scope (several `it`s in one procedure do not collide).
   - `debug_global` records program globals (`Program::debug_globals`).
   - `collect_debug_types` (called from `prepare_compiled_output`) converts every referenced `TypeId` into an `ir::DebugType` in `Program::debug_types`, keyed by the `TypeId` itself, and fills `Program::file_paths`. Built-in aggregates become structs (`string` is `{count, data}` with `data: *u8` shown as characters, views `{count, data}`, `[..]` arrays `{count, data, allocated, allocator}`, `Any` `{type, value_pointer}`); `Type`, `Code` and procedure types become typedefs of `*void`; distinct types typedefs.

2. **The LLVM backend** (`crates/jaic-llvm/src/debuginfo.rs`, hooks in `lower.rs`):
   - `DebugInfo::new` creates one DIBuilder and compile unit per LLVM module. Split codegen (`emit_objects`) builds one module per thread, so each object has its own complete CU; types are rebuilt per module.
   - Every defined function gets a `DISubprogram` (no linkage name, so debuggers show `scale`, not the internal `scale.12`).
   - Locations: the builder's debug location follows `Inst::Loc`. Blocks are lowered in reverse post-order, not source order, so each block starts from the location its first lowered predecessor ended with (`enter_block`/`leave_block`). Code before the first statement (parameter spills, stack-trace bookkeeping) is line 0, which debuggers skip when stepping.
   - Prologue: allocas have no location (at `-O0` their addresses are materialized at each use and would otherwise be attributed to the entry line). Scalar parameters are also stored to their slots in the entry block, without a location, and the entry block's branch carries the first statement's line. LLVM ends the prologue there, so `b proc` stops at the first statement with parameters already readable.
   - Variables: a variable whose address is a `SlotAddr` gets a `#dbg_declare` on that alloca. Aggregate parameters arrive by pointer, so their pointer is stored in a `dbg.addr` slot declared with `DW_OP_deref`. Any other address (`watch`) is copied to such a slot when its instruction is lowered (`Backend::set`).
   - Scalars are typedefs over DWARF base types: LLDB names base types after the C type of the same size (`s64` would print as `long`) but keeps typedef names. Structs are created as replaceable forward declarations first, so self-referential types (`next: *Node`) work.
   - Globals get a `DIGlobalVariableExpression` in the module that defines them (unit 0).

3. **The CLI** (`crates/jaic-cli/src/main.rs`): on macOS the linker leaves DWARF in the object files and only records their paths, and `jaic build` deletes the objects. After linking an executable or dynamic library, `jaic_llvm::write_dsym` runs `dsymutil` to collect it into `output.dSYM` next to the binary, where lldb finds it by UUID. If `dsymutil` fails the build still succeeds with a warning. On Linux the linker copies the `.debug_*` sections into the executable. Static libraries keep DWARF in their member objects.

## How to change it

- New kind of named variable: call `self.debug_var(f, scope, name, span, ty, addr, 0)` where the entity is added. It must be an address that stays valid for the variable's scope.
- New type kind: extend `describe_debug_type` (sema) and `DebugInfo::ty` (backend). Keys must be `TypeId`s or the synthetic `ir::DEBUG_CHAR*` keys.
- Lines only come from statements in the procedure's own file (`check_stmt`). Code from macros or `#insert` in other files keeps the caller's line, and their variables get no range.
- Windows (CodeView): `DebugFormat::for_triple` selects `CodeView` for `-windows-msvc` triples, which sets the `"CodeView"` module flag instead of `"Dwarf Version"`; the metadata is the same. The Windows backend still has to keep the `.pdb` path (link with `/DEBUG`).
- Gotchas: in a function with a `DISubprogram` every call to a function with debug info needs a location (the LLVM verifier rejects it otherwise), so lowering must never build calls while the builder has no location. Do not add a location to allocas. Use the `raw` helpers, not inkwell's `insert_declare_*`, which wrap LLVM 19+ debug records as instructions.

## Configuration

- On by default for `jaic build`. Turn it off with `--no-debug-info`, or from a metaprogram with `options.emit_debug_info = .NONE` (also what `set_optimization(*options, type, preserve_debug_info=false)` does). The workspace option is forwarded as `emit_debug_info` (`stdlib/Compiler/options.jai`, `BuildSettings::emit_debug_info`).
- `jaic_llvm::Options::debug_info`; `jaic::sema::Options::debug_info`.
- DWARF 4 on Apple targets, 5 elsewhere. The compile unit's language is C99, so lldb expressions use C syntax (`p xs.data[1]`, `p *p`).
- Cost: Focus (`first.jai -O0`) builds in about 2.5s instead of 2.3s, including `dsymutil`.

## Dependencies

- LLVM's DIBuilder (LLVM-C, through `inkwell::llvm_sys`); `dsymutil` on macOS (Xcode command line tools).
- Tests: `crates/jaic-cli/tests/debug_info.rs` builds `tests/native/debug-info/` and checks `llvm-dwarfdump --verify` and its output (single and split codegen, `--no-debug-info`), and drives `lldb --batch` through breakpoints in both files, a backtrace through a polymorphic procedure and `frame variable`. Each test skips when its tool is missing.
- ELF gotcha: GNU ld (the `cc` default on Linux, and lld at `-O2`) tail-merges `.debug_str`, so a DWARF 5 string offset may point into the middle of a longer string (`s64` stored inside `[..] s64`). That is valid, and gdb/lldb read it correctly, but `llvm-dwarfdump --verify` reports every such offset as "is neither zero nor immediately following a null character". The test's `verify` helper tolerates exactly that diagnostic (and its summary lines) on non-Apple targets, and fails on anything else; the name checks confirm the strings still resolve. Reproduce on macOS by rewriting `--emit-ir` output to `x86_64-unknown-linux-gnu` with `"Dwarf Version", i32 5`, compiling with `llc -filetype=obj`, and linking with `rust-lld -flavor gnu -O2 --unresolved-symbols=ignore-all -e main`.
- Related: [LLVM backend](llvm-backend.md), [IR](../compiler/ir.md), [stack traces](../compiler/stack-traces.md).
