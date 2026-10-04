# Browser playground (jaic backend)

## What it is

The browser Run button compiles and interprets the workspace with the new compiler core `crates/jaic`
(lexer, parser, sema, interpreter) compiled to WebAssembly inside `crates/jai-wasm`. The full `stdlib/`
(and `prelude/`) are embedded in the wasm module, so `#import "Basic"` and friends work offline.
The shared language server (`jai_lsp_*`, `crates/jai-language-server`, built on the same `jaic` lexer and parser) is the only other export.

## How it works

1. `crates/jai-wasm/build.rs` walks `stdlib/` and generates `$OUT_DIR/stdlib_files.rs`
   (`BUNDLED: &[(&str, &[u8])]` of `include_bytes!`). `stdlib/prelude/module.jai` does `#load "../stdlib/prelude/module.jai"`,
   hence both trees. On wasm32 the script also links with a 256 MiB shadow stack (`-zstack-size`), because
   the compiler recurses deeply.
2. `src/play.rs::run(files, main)` builds a `jaic::sema::VirtualFs` with the bundled files at `/stdlib/...`
   and the user files at `/workspace/<path>`. Options: import paths `<main dir>/modules` then `/stdlib` (like the command
   line, so a `modules/` folder of the workspace works), preload `/stdlib/prelude/module.jai`, target **WASM / Wasm**
   (`OS == .WASM`). It installs a `SharedHost` (wraps
   `jaic::interp::SandboxHost`, which provides libc shims, an in-memory file system and captures stdout/stderr) as the interpreter host,
   calls `compile_program` then `run_program`, and returns a `PlayResult`
   `{exitCode|null, stdout, stderr, rendered, diagnostics[{severity,file,line,column,message}]}` as JSON.
   Diagnostics with a `path:line:col:` message prefix (runtime traps) are split apart; others use the span.
3. `src/play_exports.rs` is the pointer-free scalar ABI (same style as the other exports):
   `jai_play_reset`, `jai_play_push(channel, byte)` (0 = path, 1 = contents, 2 = main path),
   `jai_play_finish_file`, `jai_play_run`, then `jai_play_output_len/byte` (JSON) and `jai_play_error_len/byte`.
4. `web/scripting-runtime/engine.mjs` exposes `play(files, main)` (and `lsp(message)`); `worker.mjs` answers `run` messages with
   `{type:"run", play}`; `editor.mjs::showPlay` prints stdout, stderr, rendered errors and
   the exit code in the Output panel, and puts positioned diagnostics in the Problems panel and editor
   (kept in `runDiagnostics`, separate from language-server diagnostics, cleared when the file is edited).

## Building and staging

`python3 tools/build_scripting_wasm.py [--release]` runs an offline, locked, single-job, non-incremental `cargo build -p jai-wasm --target wasm32-unknown-unknown`, checks the `\0asm` header, and copies the module plus every file in `web/scripting-runtime/` into the output directory (default `artifacts/scripting-runtime`, ignored by git). It writes `build-metadata.json` with the build command, target directory selection, paths and the module SHA-256. It refuses to start below 2 GiB free on the source, build or staging volumes.

`engine.mjs::createEngine(wasmBytes)` instantiates the module (rejecting any host import) and returns `play(files, main)` and, when the module exports it, `lsp(message)`. Both exchange bounded scalar bytes with the module; no pointers cross the boundary.

```sh
rustup target add wasm32-unknown-unknown --toolchain nightly-2026-08-29
python3 tools/build_scripting_wasm.py --release
node tools/check_scripting_wasm.mjs artifacts/scripting-runtime/jai_wasm.wasm
node tools/check_playground_worker.mjs artifacts/scripting-runtime
python3 -m http.server 8080 --bind 127.0.0.1 --directory artifacts/scripting-runtime
```

Then open `http://127.0.0.1:8080/`. `tools/check_browser_release.mjs <staged-dir>` additionally needs the `release.json` written by `tools/package_browser_release.py`; see [compiler releases](compiler-releases.md).

## How to change it

- Different target or options: `options()` in `play.rs`.
- More host services (files, time): extend `SharedHost`/`SandboxHost::foreign` in `crates/jaic/src/interp/mod.rs`.
- Result shape: change `PlayResult::to_json` and `showPlay` together.
- Gotchas: under `OS == .WASM`, Runtime_Support writes output through the foreign
  `wasm_write_string(count, data, to_standard_error)`, which `SandboxHost` implements. The jaic interpreter stores
  host pointers in 64-bit slots, which works on wasm32. There is no step limit or argument list: the playground has no run options.
  Infinite loops hang the worker; use Cancel (terminates the worker).
- Panics: wasm32 panics abort, so a compiler bug traps the instance (`RuntimeError: unreachable`). `jai_play_reset`
  installs a panic hook (wasm32 only) that records the message; `engine.mjs::play` catches the trap and throws
  `The compiler crashed: <message>` with `compilerCrashed = true`, and `worker.mjs` drops its engine so the next
  Run instantiates a fresh module. `jai_play_panic_len/byte` read the message.
- No real clock on wasm32 (`std::time::SystemTime::now` panics): `#cycle_counter` counts calls, and `SandboxHost`
  implements `clock_gettime`/`gettimeofday`/`time` (virtual, deterministic, +1 microsecond per call), `nanosleep`/`usleep`
  (advance the virtual clock, never block) and `wasm_debug_break` (runtime error), so `current_time_monotonic`,
  `random_seed` and friends work. Other native `#foreign` symbols fail with `foreign procedure 'x' is not available here`
  or `unknown library`.
- **POSIX on WASM**: `OS == .WASM` is treated like Linux by `stdlib/POSIX` (Linux x86-64 bindings and struct layouts),
  `File`, `File_Utilities`, `Thread`, `Basic` time, so those modules compile and call into `SandboxHost`. Windowing
  and native-library modules still do not: `Window_Type` is `*void`, `Clipboard` is an in-memory string, `Window_Creation`,
  FreeType, stb_image, libclang and `Process` have no browser backend.
- **Files**: reads see the workspace (cwd is `/workspace`, so `read_entire_file("lib/data.txt")` works) and the bundled
  stdlib (`/stdlib/...`); writes go to an in-memory overlay (`/tmp` exists) that lives for one Run. Nothing persists.
- **Threads**: run one after another on the interpreter stack, see [interpreter threads](../compiler/interpreter-threads.md).
  `Thread`, `Mutex`, `Condition_Variable`, `Semaphore` and `Thread_Group` work; a thread that busy-waits for a
  thread lower on the stack hangs the worker (Cancel).
- **Path semantics**: `normalize` clamps `..` at the root, so `#load "../../stdlib/X.jai"` from `/workspace/a.jai`
  reaches `/stdlib/X.jai` as on a real file system.
- The worker check (`tools/check_playground_worker.mjs`) smoke-tests Hash_Table, a `#run` workspace message loop, empty views and the virtual clock.
- Regression sweep for the wasm build: `node tools/check_playground_stdlib.mjs <staged-dir>` runs every
  `tests/stdlib/*.jai` through `engine.play` with a fresh wasm instance each (worker threads, 120 s timeout) and compares
  the pass set with `tools/playground_stdlib_expected.json` (`pass` list plus `excluded`: name to written reason). Any
  regression, any newly passing excluded test, or any test in neither list fails the check; `check_playground_worker.mjs`
  runs it. After intentionally changing the set: `--update` rewrites `pass` (new failures get a `TODO explain` reason you
  must replace). `PLAYGROUND_VERBOSE=1 ... name.jai` prints that test's output. Currently 136 of 146 pass. Excluded:
  `bindings-generator-c`/`-cpp` (dlopen of libclang), `bindings-generator-cpp-classes` and `buildcpp-api` (start a compiler
  process), `c-variadic-foreign-calls` (native C ABI test against libc, pipe, fcntl), `simp-compat-api` and
  `getrect-right-handed-api`/`getrect-right-handed-surface` (FreeType and stb_image C libraries),
  `getrect-text-display-compiles` (Window_Creation) and `getrect-rh-negative-control` (fails everywhere by design).
- Debug browser-only behavior natively: `jaic run test.jai -os wasm` uses the same `SandboxHost`.

## Configuration

- Build: `cargo build -p jai-wasm --release --target wasm32-unknown-unknown` (or
  `python3 tools/build_scripting_wasm.py --release`, which stages `web/scripting-runtime/` plus `jai_wasm.wasm`).
- Serve the staged directory with any static server and open `index.html`.
- Native test: `cargo test -p jai-wasm play`; `node tools/check_scripting_wasm.mjs <jai_wasm.wasm>` runs fixtures against the built module (hello world, sibling `#load`, positioned diagnostics, scalar ABI).
- The wasm is about 30 MB because the stdlib is embedded.

## Dependencies

`jaic` (no external crates), `jai-language-server`, CodeMirror bundle in
`web/scripting-runtime/editor.bundle.mjs`.
Rust's `wasm32-unknown-unknown` target has no filesystem or process services; the interpreter's
`SandboxHost` supplies the libc subset the stdlib needs (virtual clock, in-memory files, cooperative threads).
No Web Worker threads or `SharedArrayBuffer` are involved, so any static host works.
