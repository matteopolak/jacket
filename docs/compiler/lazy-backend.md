# Lazy code generation backend

## What it is

A release `jaic` does not link LLVM. Native code generation lives in a shared library, `libjaic_backend` (`jaic_backend.dll` on Windows), that sits beside the executable and is opened the first time a build writes native code. `-check`, `-interp`, the language server, `jailint` and the browser build never open it.

## How it works

Only object emission needs LLVM. Everything else a native build does (build options, the system linker, `wasm-ld`, `dsymutil`, archives) runs in the driver. So the crates split like this:

| Crate | Holds |
| --- | --- |
| `crates/jaic-native` | No LLVM. `Options`, `Codegen`, `OptLevel`, `Sanitize`, the linker, `used_libraries`, and `backend`: the loader, the request types and the stamp. |
| `crates/jaic-llvm` | Lowering and `emit_object` / `emit_objects`. Re-exports the `jaic-native` names, so code that links LLVM directly needs only this crate. |
| `crates/jaic-backend` | A `cdylib` with two exports, `jaic_backend_stamp` and `jaic_backend_emit`, over `jaic-llvm`. |
| `crates/jaic-cli` | Links `jaic-native` always, and `jaic-llvm` only when LLVM is built in. |

The driver calls code generation from one place, `generate::object(s)` in `crates/jaic-cli/src/backend.rs`. With the `lazy-llvm` feature that call opens the library (once), then hands it an `EmitRequest`. `-timings` shows the open as a `load backend` phase; a run that prints no such line never loaded it.

**Sharing the IR.** The request holds `&ir::Program` and `&Options`, passed by pointer; nothing is serialized. That is only sound when both sides came from the same sources and compiler, so the library exports a stamp and `load` refuses a library whose stamp differs from the driver's. The stamp is the crate version, the `rustc` release, a hash of `crates/jaic/src/ir*`, `target.rs`, `abi.rs` and the `jaic-native` options and loader, and `size_of` of the shared types. `crates/jaic-native/build.rs` computes it, so both sides get it from the same crate.

**Memory and panics.** Nothing the library allocates is freed by the driver, or the other way round: object paths and error text come back through a closure in the request, and the driver copies them. A panic inside the library is caught there and returned as an error. The library has its own copy of `jaic` and its own thread pool, sized from the request.

**Finding the library.** `JAIC_BACKEND` (a file path) if set, else the library's name in the directory of the real `jaic` (symlinks are resolved, so `~/.local/bin/jaic` and Homebrew's `bin/jaic` work). A missing library is an error only when a build needs it:

```
error[cli-failure]: this jaic cannot write native code: the code generation library `libjaic_backend.dylib` was not found (looked at ...)
help: reinstall jaic, or set JAIC_BACKEND to the library's path
```

The loader is a few lines of `dlopen` / `LoadLibraryExW` (`crates/jaic-native/src/backend.rs`), so there is no new dependency.

## How to change it

- **A new field in the request or in `Options`.** Edit the type in `jaic-native` and both sides rebuild; the stamp changes with the source. Do not put an owning type in the reply: add a `Reply` variant that borrows.
- **A new thing the backend must do** (say, emit bitcode on its own). Add a field to `EmitRequest` or a second export, and serve it in `backend::serve`. Keep the call count low: each call is one load-bearing decision about what crosses.
- **Anything that does not need LLVM** belongs in `jaic-native`, not `jaic-llvm`, so the driver does not need the library for it.
- **Global state.** The library has its own copy of every `static` in `jaic`. If code generation reads one the driver sets (like the thread count), pass it in the request.

## Configuration

| Setting | Effect |
| --- | --- |
| `JAIC_BACKEND` | Path of the library to open instead of the one beside `jaic`. |
| `jaic-cli` feature `lazy-llvm` | Open the library (release archives). Not combinable with `dynamic-llvm` or `static-llvm`. |
| `jaic-cli` features `dynamic-llvm` (default), `static-llvm` | LLVM linked into `jaic`. `cargo build -p jaic-cli` is a complete compiler, and every test runs against it. |
| `jaic-backend` features `dynamic` (default), `static` | How LLVM links into the library. Release archives use `static`. |

Build the lazy pair by hand:

```sh
cargo build --release -p jaic-cli -p jaic-backend --no-default-features \
  --features jaic-cli/lazy-llvm,jaic-backend/dynamic
target/release/jaic hello.jai -timings   # prints "jaic-timing: load backend"
```

`tools/dev/build_pgo.py --llvm lazy` builds the release pair, with PGO (and BOLT on Linux x86-64) applied to the library too ([releases](../tools/releases.md)).

## Effect

A release `jaic` on macOS arm64 is 10 MB instead of 102 MB, and the library is 91 MB. `jaic hello.jai -check` no longer maps the static LLVM image at start-up: on an Apple M5 it takes 56 ms instead of 62 ms and uses 40 MiB of memory instead of 52. A 60k-line generated program checks in 124 ms instead of 130 ms. A native build opens the library once and takes the same time as before (hello 153 ms against 154 ms, the 60k-line program 439 ms against 437 ms). Measure with `tools/bench`: `compare` for `-check` and build of a generated program, `compile --only hello` for start-up.

## Why the default stays in-process

A `cdylib` cannot be a build dependency of the executable on stable Rust, so a plain `cargo build -p jaic-cli` cannot also produce the library. Keeping LLVM built in for development means every existing test, the sweep and `cargo run` work unchanged. The lazy path is tested by `crates/jaic-llvm/tests/lazy_backend.rs` (the library writes the same object as a direct call, and the loader refuses a file that is not the library) and, for the whole packaged flow, by each release leg: `tools/ci/release_smoke.py backend` checks that `-check` and `-interp` neither load the library nor need it, and that a build loads it.

## Dependencies

`jaic-native` uses `jaic` and `jacket-config`. `jaic-backend` uses `jaic-llvm` (Inkwell, LLVM 23) and `jaic-native`. See [LLVM backend](../native/llvm-backend.md) for what the library does with the IR and [releases](../tools/releases.md) for how it is packaged.
