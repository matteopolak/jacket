# Compile-time code on the host

When `jaic` cross-compiles (`-os linux` on a Mac, `-cpu x64` on arm64), compile-time code runs on the machine doing the compiling. So a plain `#run` sees the host: `OS` and `CPU` name the machine running `jaic`, stdlib `#if OS` branches pick host variants, and struct layouts are the host's.

Native builds (target equals host) take none of this path.

## Host and target

| Question | Answered by |
|---|---|
| `OS`, `CPU`, `size_of`, `#if`, `#assert`, layouts and codegen of the program | the target: `Options::os`, `cpu`, `long_double` |
| `OS`, `CPU` and layouts seen by `#run`, `BUILD_OS`, `BUILD_CPU`, `IS_CROSS_COMPILING` | the host: `Options::host` |

Sema never asks the operating system what the host is; it reads `Options::host`. A test can replace it.

## How it works

A cross build keeps a second compiler, the **host twin**: the same `Compiler` type with the host `OS`/`CPU`, its own type table, modules, IR and interpreter. It never generates code.

```
target Compiler                          host twin Compiler (host OS/CPU)
  check_run / eval_const_all  --scope-->   check_run_inner_all(mapped scope)
  import_host_result  <---Operand::Const---  results read back in host layout
```

- **Created on demand** by the first plain `#run` of a cross build (`host_twin.rs`). It loads bootstrap modules and the program sources the target was given.
- **Where it applies**: `check_run` and the `#run` arm of `eval_const_all`. A run stays on the target compiler when any loaded source mentions `#no_reset`, when its text names `compiler_set_type_info_flags`, `add_data_segment`, `add_global_data`, `get_runtime_info` or `get_type_table`, or when the twin reaches one of those operations while running. In that last case the twin is dropped and every later run stays on the target.
- **Scope mapping**: a run is evaluated in the twin's scope for the same file, matched by canonical path and module instance.
- **Shared state**: both compilers share the workspace registry, so `add_build_string`, `compiler_wait_for_message` and friends act on the real workspaces. They also share one append-only source table, so a `FileId` or `Span` means the same file in both.
- **Results** (`host_import.rs`) are converted to the target:
  - Types match by declaration (file path, span, name). A type crosses by value only if its layout is identical on both sides; otherwise it is an error naming the type. Return its fields instead.
  - `long double` is converted from the host's format to the target's.
  - Pointers into frozen data are copied as globals with the objects they reach. A pointer to a program global becomes the target's own global of that name. `*Type_Info` becomes the target's descriptor. A `*void` (or a pointer inside a union) into memory only the interpreter holds becomes null, because a host address would differ from build to build.
  - Numbers, strings, arrays and user structs without OS-dependent members flow freely.
- Executions, traps and placeholder misses the twin saw are added to the target's counters.

## Limits

- Code that inspects the target program through the interpreter (`Type_Info` of target types, `Code` values, `#modify`, `#insert` of `Code`) is not moved; it runs on the target compiler.
- `#assert`, `#if` and constant folding without `#run` run on the target, so `size_of` and `OS` there describe the target.
- `Code`-valued results, procedures without a file-level declaration, and results with differing layouts are errors.

## How to change it

- Entry point: `Compiler::run_on_host_twin` (`sema/host_twin.rs`), called from `check_run` and `eval_const_all`. To make another kind of compile-time execution see the host, call it where the thunk is built and import with `import_host_result`.
- To widen what crosses back, extend `import_type` / `import_value` in `sema/host_import.rs`.
- The twin is a separate type universe: a `TypeId`, `ProcId`, `ScopeId` or `FileId` from one compiler means nothing in the other. Keep `host_twin.rs` the only place that moves values across.
- State written by `#run` code lives in the compiler that ran it. Anything compile-time code writes that the program must see belongs on the target.
- Test a cross build without running it with `-interp -os linux`; `crates/jaic-cli/tests/host_runs.rs` does this.
- `#run,host` (`sema/host_run.rs`) is the older strict form: a fresh compiler with no program, primitives only.

## Configuration

No flags. `IS_CROSS_COMPILING` is true exactly when the twin can be used (false inside it). `JAIC_NO_HOST_TWIN=1` turns it off; `JAIC_HOST_TWIN_DEBUG=1` prints why a twin could not be made or a run stayed on the target.

## Cost

A native build pays nothing. In a cross build the twin costs a front end (bootstrap modules, the program's imports, bodies reached by `#run`) and no codegen. It parses its own copies of files, since library syntax trees are platform-specific.

## Dependencies

`sema/host_run.rs`, `build/registry.rs`, `interp` (`Host::native_linking`), `source.rs`.
