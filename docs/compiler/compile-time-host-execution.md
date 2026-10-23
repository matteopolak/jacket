# Compile-time code on the host

## What it is

When `jaic` cross-compiles (`-os linux` on a Mac, `-cpu x64` on arm64), compile-time code runs in the interpreter on the machine doing the compiling. Plain `#run` therefore sees the host: `OS` and `CPU` name the machine running `jaic`, stdlib `#if OS` branches select the host variants, and struct layouts and constants match the host libc. `BUILD_OS` and `BUILD_CPU` are the host too. `OS`, `CPU`, `Build_Options` and every other description of the program being built keep naming the target, and everything outside compile-time code (type checking of the program, `size_of`, `#assert`, `#if`, code generation) uses the target.

Native builds (target equals host) take none of this path and pay nothing for it.

## How it works

A cross build keeps a second compiler, the **host twin**, next to the target compiler. It is the same `Compiler` type with the host `OS`/`CPU`, its own type table, module instances, IR and interpreter. It never generates code.

```
target Compiler                          host twin Compiler (host OS/CPU)
  check_run / eval_const_all  --scope-->   check_run_inner_all(mapped scope)
        ^                                       runs in the twin's Interp
        |                                       (reads the shared workspace registry)
  import_host_result  <---Operand::Const---  results read back in host layout
```

1. **Creation, on demand.** The first plain `#run` of a cross build creates the twin (`host_twin.rs`, `Compiler::host_twin`). It loads bootstrap modules and the program sources the target was given (`SourceState::program_sources`, recorded only in cross builds, and forwarded by `add_source` later), expands declarations and applies `#poke_name`. It never settles top-level code: the twin's own queued `#run`s and `#assert`s are never executed. Procedure bodies are checked and lowered only when the interpreter reaches them, so the cost is the front end of the import graph plus the code compile-time code actually calls. No LLVM work happens for the twin.
2. **Where it applies.** `Compiler::check_run` and the `#run` arm of `eval_const_all` (constants, top-level `#run`, `#insert #run`) ask `run_on_host_twin`. It declines (the run then executes on the target compiler as before) when the build is not a cross build, when the interpreter's host is not native (wasm sandbox runs keep `OS == .WASM`), in editor sessions, for `#run,host`, when the run's scope cannot be mapped (below), when its text contains a `#code` literal, and when the type of a run expression cannot cross (below).
3. **Scope mapping.** A run is evaluated in the twin's scope for the same file. The target scope is walked up to its file scope; the file is found in the twin by canonical path and module instance (module entry path plus primitive module parameters). Block and procedure scopes between the run and the file are allowed as long as the run's source text mentions none of the names they declare (the run cannot refer to a local constant or polymorphic parameter that only the target instantiated). Otherwise the run executes on the target compiler. Files that only the target has (`#insert`ed or `add_build_string` texts with generated names) are mapped by their virtual path; when absent the run falls back.
4. **Shared state.** The twin shares the workspace registry with the target compiler (`Interp::workspaces`), so `Compiler`-module calls from a metaprogram (`add_build_string`, `compiler_wait_for_message`, `set_build_options`, ...) act on the real workspaces. Its own interpreter binds those operations against its own types. The twin never pulls sources out of the registry (`MetaState::twin`); the target does, and forwards each added source to the twin.
5. **Source ids.** Both compilers add files to one table (`SourceMap::share`, an append-only table behind an `Rc`), so a `FileId` and every `Span` mean the same file in both. The run's syntax tree is therefore usable in the twin as it is, and a diagnostic from twin code prints with its real snippet in the target.
6. **Results** (`host_import.rs`). A run returns `Operand`s in the twin's types. They are converted to the target:
   - Types are matched by declaration: the file path, the span of the declaration and its name. A declaration that the target keeps in another file of the same directory (an OS-specific copy, `types.jai` and `windows-types.jai`) matches when its name and layout are the same. Built-in types, pointers, arrays and procedure types are rebuilt structurally. Types declared inside procedures, distinct types, `Code` and polymorphic struct instances have no counterpart: a run expression whose type is one of these is checked before it runs (`Compiler::can_import_host_type`) and stays on the target compiler, as it did before the twin.
   - A type crosses by value only if its layout is **identical** on both sides: same size, alignment, field names, offsets and field types (recursively), same enum members. A type whose layout differs (an OS-dependent field, a pthread-sized member) is rejected with an error naming it; return its fields individually instead. This is deliberately stricter than field-wise conversion, which would silently reorder semantics; nothing in the standard library returns such types from `#run`.
   - Pure-language types (numbers, strings, arrays, views, user structs and enums without OS-dependent members) flow freely, unlike `#run,host`.
   - Pointers into frozen data (strings, array contents, linked structures) are copied as globals together with the objects they reach. A pointer to a global variable of the program becomes the target's own global of that name (`Output::declared_globals`, `target_variable`), so results that hold program state stay connected. Procedure values and addresses (function pointers in structs) are mapped to the target's procedure with the same name and declaration span; procedures local to other procedures, polymorphic instances, `Code` values and foreign storage cannot cross and are errors.
7. **Build options and workspaces.** `get_build_options` reports the target for a workspace that has no saved options (the record's defaults would otherwise read `OS`, which is the host inside the twin). Sources the twin's code adds to a workspace (`add_build_string`, including with a `Code` scope) are loaded into the twin when it pulls them, then queued in `SourceState::twin_outbox` and loaded into the target after the run (`load_twin_sources`; a `Code` scope is mapped back to the target's file scope). A module-scoped string cannot be named from the twin and is an error.
8. **Counters.** Executions, traps and placeholder misses the twin saw are added to the target's, so dependency retries keep treating a run that already executed as started.

### Options considered

- **Per-call patches** (the earlier `compile_time_host_monotonic_nanoseconds`, host file query and CPU detection hooks). Each fixes one libc corner and leaves the rest of the host-dependent stdlib wrong (forking, pipes, locking, `stat`). Replaced by this design; the clock hook and `host-clock.jai` are gone.
- **One compiler, two OS views.** Key every module instance, entity cache and struct layout by view. Every cache in sema would gain a key and any missed one is a silent wrong-platform bug. Rejected as far more invasive than a second instance.
- **Host copies of library procedures called from target-compiled code.** Needs a marshalling layer for polymorphic instances, callbacks and every user type that library code touches. Rejected.
- **Extending `#run,host`.** It builds a fresh compiler with no program, so it cannot name program declarations, and transfers only primitives. Kept as the strict, explicit form; the twin is the program-aware generalisation.
- **Running metaprograms in the twin only when they use no `Code`/type values.** The workspace API is record- and string-based, so metaprograms work; what does not work is code that hands target `Type` or `Code` values through the interpreter (see limits).

## How to change it

- Entry points: `Compiler::run_on_host_twin` (`sema/host_twin.rs`), called from `check_run` and `eval_const_all` in `sema/consteval.rs` and `sema/decls.rs`.
- To make another kind of compile-time execution see the host (for example global initializers), call `run_on_host_twin` where it builds its thunk and import the result with `import_host_result`.
- To widen what crosses back, extend `import_type` / `import_value` in `sema/host_import.rs`; keep the identical-layout rule unless you also convert images.
- Gotchas: the twin is a separate type universe. A `TypeId`, `ProcId`, `ScopeId` or `FileId` from one compiler means nothing in the other. Keep `host_twin.rs` the only place that moves values across, and never store a twin id in target state.
- State written by `#run` code (globals, allocated memory) lives in the compiler that ran it. Runs the twin takes share one interpreter; a run that falls back sees different global state.

## Limits

- Compile-time code that inspects the target program through the interpreter (`Type_Info` of target types, `Code` values, `#modify`, `#insert` of `Code`) is not moved; those operations are not `#run` forms and keep running on the target compiler. A plain `#run` that passes a target type or `Code` value in fails with a message saying so.
- `#assert`, `#if` and constant folding that does not use `#run` execute on the target, so `size_of` and `OS` there keep describing the target.
- `Code`-valued results, procedures without a file-level declaration, and results whose layout differs between host and target, are errors.

## Configuration

No flags. `IS_CROSS_COMPILING` is true exactly when the twin can be used (and false inside it). `JAIC_NO_HOST_TWIN=1` turns the mechanism off (plain `#run` runs on the target compiler as before); it exists for timing and bisecting. `JAIC_HOST_TWIN_DEBUG=1` prints why a twin could not be made or a run fell back.

## Cost

A native build never creates a twin and pays nothing. In a cross build the twin costs its front end (bootstrap modules, the program's imports, bodies reached by `#run`) and no code generation. Measured with `-check` on an arm64 Mac targeting Linux x64: a program with a few `#run` constants 0.03 s native, 0.04 s cross; a build program with `run_command` and a child workspace 0.05 s native, 0.07 s cross; XChatger `first.jai` 0.14 s cross.

Not done: the twin parses its own copies of the files. Parsing skips dead `#if OS`/`#if CPU` branches of library files (`parse_file_for_target`), so a library syntax tree is specific to its platform and cannot be shared. Sharing the trees of user files, which are platform-neutral, is possible by keying a cache on the shared `FileId`, but the saving is the parse time of user files only.

## Dependencies

`sema/host_run.rs` (`#run,host`, `compiler_host`), `build/registry.rs` (shared workspaces, host factory), `interp` (`Host::native_linking`), `source.rs` (`SourceMap` id ranges).
