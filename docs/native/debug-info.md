# Native debug information

`jaic` emits DWARF (CodeView for `-windows-msvc` triples), so lldb and gdb can debug compiled Jai: breakpoints by `file.jai:line` or procedure name, stepping, backtraces with Jai names, and locals, parameters and globals with Jai types. It is on by default at every optimisation level.

```
$ jacket build main.jai -exe prog
$ lldb --batch -o 'b helper.jai:23' -o run -o bt -o 'frame variable' ./prog
```

## How it works

1. **Sema records** (`sema/debug_info.rs`), only when `Options::debug_info` is set (never for `-interp`/`-check`):
   - `begin_func_debug` gives each procedure an `ir::FuncDebug` table: name, line, lexical scopes, variables.
   - `debug_var` records a named local, parameter, result or loop variable with its type, line and the IR value holding its address. Temporaries and `_` are skipped.
   - Every `Inst::Loc` carries a scope index so the backend picks the right lexical block.
   - `collect_debug_types` turns each referenced type into an `ir::DebugType`. `string` is `{count, data}`, arrays are `{count, data, allocated, allocator}`, and `Type`, `Code` and procedure types are typedefs of `*void`.

2. **LLVM backend** (`jaic-llvm/src/debuginfo.rs`, hooks in `lower.rs`):
   - One DIBuilder and compile unit per module; each split codegen unit has its own. Each function gets a `DISubprogram`.
   - Code before the first statement (parameter spills, stack-trace bookkeeping) is line 0, which debuggers skip. The entry branch carries the first statement's line, so `b proc` stops with parameters readable.
   - A variable at a `SlotAddr` gets a `#dbg_declare` on that alloca. Aggregate parameters arrive by pointer, so the pointer is stored in a `dbg.addr` slot with `DW_OP_deref`.
   - Scalars are typedefs over DWARF base types, because LLDB names base types after the C type of the same size (`s64` would print as `long`).

3. **CLI, macOS.** The linker leaves DWARF in the object files. An optimised build (`-O1`+) runs `dsymutil` into `output.dSYM`. An `-O0` build skips that and keeps its objects in `.build/` beside the output, which lldb reads through the debug map; add `.build/` to `.gitignore`. A sanitized build writes the `.dSYM` at `-O0` too. `JAIC_DSYM=1` writes the `.dSYM` and deletes objects at any level. On Linux the linker copies `.debug_*` into the executable.

## How to change it

- New named variable: `self.debug_var(f, scope, name, span, ty, addr, 0)` where the entity is added. The address must stay valid for its scope.
- New type kind: `describe_debug_type` (sema) and `DebugInfo::ty` (backend).
- Lines come only from statements in the procedure's own file; macro or `#insert` code from other files keeps the caller's line.
- Windows: `DebugFormat::for_triple` selects CodeView, and `link` passes `/DEBUG /PDB:<stem>.pdb`.
- Gotchas: every call to a function with debug info inside a function with a `DISubprogram` needs a location, so never build calls while the builder has none. Allocas get no location. Use the `raw` helpers, not inkwell's `insert_declare_*`. On ELF, tail-merged `.debug_str` makes `llvm-dwarfdump --verify` complain; the tests tolerate exactly that.

## Configuration

- Off with `-no_debug_info` or `options.emit_debug_info = .NONE` in a metaprogram.
- The compile unit's file is named by its whole path. DWARF 5 (not Apple) keeps that file as line table entry 0 and LLVM folds every later file with the same name into it, so without a unique name the lines of every `module.jai` would read as the main file's and a coverage run would count none of them.
- DWARF 4 on Apple, 5 elsewhere. The compile unit language is C99, so lldb expressions use C syntax (`p xs.data[1]`).
- `JAIC_DSYM=1`.

## Dependencies

LLVM's DIBuilder; `dsymutil` on macOS. Tests: `crates/jaic-cli/tests/debug_info.rs` builds `tests/compiler/native/debug-info/`, checks `llvm-dwarfdump --verify` and drives `lldb --batch --no-lldbinit`; each test skips when its tool is missing. Related: [LLVM backend](llvm-backend.md), [IR](../compiler/ir.md), [stack traces](../compiler/stack-traces.md).
