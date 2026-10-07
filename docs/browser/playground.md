# Browser compiler (WebAssembly bundle)

## What it is

`crates/jai-wasm` compiles the compiler core `crates/jaic` (lexer, parser, sema, interpreter) and the shared language server to `wasm32-unknown-unknown`. The full `stdlib/` are embedded in the module, so `#import "Basic"` and friends work offline.

This repository ships the module and a small JavaScript glue file, not a UI. The hosted playground at https://matteopolak.com/playground/jai lives in the portfolio repository (`matteopolak/portfolio`), which has its own editor, worker and engine wrapper. From a bundle it uses `jai_wasm.wasm`, `jaifmt-playground.jai` and the [language tour](tour.md) (`tour.json`, `tour/`); it can use `jaifmt.wasm` for the Format button.

A bundle (`tools/build_scripting_wasm.py --output <dir>`) holds:

| File | Source | Purpose |
| --- | --- | --- |
| `jai_wasm.wasm` | `crates/jai-wasm` | Compiler, interpreter and `jai_lsp_*` language server. It takes no host imports. |
| `engine.mjs` | `crates/jai-wasm/js/engine.mjs` | Optional glue: `createEngine(bytes)` returns `play` and `lsp`. The Node checks use it too. |
| `jaifmt-playground.jai` | `jaifmt/playground.jai` | Formatter driver ([jaifmt](../tools/jaifmt.md#browser-playground)) |
| `jaifmt.wasm` | `jaifmt/wasm.jai`, built by a native `jaic -os wasm` (`--jaic`) | jaifmt as a wasm64 WASI module, about 35 times faster than the driver; needs Memory64 ([jaifmt](../tools/jaifmt.md#webassembly-build-jaifmtwasm)) |
| `build-metadata.json` | build script / packager | Local build receipt, or the commit, toolchain and Wasm SHA-256 in a release |
| `README.md` | `crates/jai-wasm/js/README.md` | Short notice for embedders |
| `tour.json`, `tour/**` | `examples/tour/` (cases with `bundle` in `tests/examples.json`) | The [language tour](tour.md) the playground opens with, and its file index |

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
   `{exitCode|null, stdout, stderr, output[{stream,text}], rendered, diagnostics[{severity,file,line,column,message,code?}]}`
   as JSON (`code`, when present, is the error's stable name from `DiagnosticKind::code`, such as `unavailable`, `static-assert` or `runtime-error`; `exitCode` is null both when compilation failed and when the running program stopped with a runtime error, and `runtime-error` tells the second apart). `output` is everything the program wrote in write order, as runs of `"stdout"` or `"stderr"`
   (`SandboxHost::order` records the run lengths). `run_with(files, main, PlayOptions { budget })` bounds the
   main compile's interpreter to `budget` basic blocks; past it the run fails with "execution budget exhausted".
   Diagnostics with a `path:line:col:` message prefix (runtime traps) are split apart; others use the span.
3. `src/play_exports.rs` is the pointer-free scalar ABI (same style as the other exports):
   `jai_play_reset`, `jai_play_push(channel, byte)` (0 = path, 1 = contents, 2 = main path),
   `jai_play_finish_file`, `jai_play_set_budget(thousands)` (0 = unbounded, kept across resets), `jai_play_set_styled(1)` (errors in `rendered` with ANSI colour and box drawing, as a terminal shows them, for an output pane that draws SGR codes; 0, the default, gives plain text; kept across resets), `jai_play_run`, then `jai_play_output_len/byte` (JSON) and `jai_play_error_len/byte`.
4. `crates/jai-wasm/js/engine.mjs::createEngine(wasmBytes)` instantiates the module (rejecting any host import) and returns `play(files, main, { budget })` and, when the module exports it, `lsp(message)`. Both exchange bounded scalar bytes with the module; no pointers cross the boundary.

### Embedding

`play` is synchronous and returns when the program ends; nothing streams while it runs, because the module takes no host imports. Run it in a Web Worker so the page stays responsive, and terminate the worker to cancel.

```js
import { createEngine } from "./engine.mjs";
const engine = await createEngine(await (await fetch("jai_wasm.wasm")).arrayBuffer());
const play = engine.play({ "main.jai": source }, "main.jai", { budget: 50_000_000 });
for (const { stream, text } of play.output) (stream === "stderr" ? console.error : console.log)(text);
```

## Building and checking

`python3 tools/build_scripting_wasm.py [--release] [--output <dir>]` runs an offline, locked, single-job, non-incremental `cargo build -p jai-wasm --target wasm32-unknown-unknown`, checks the `\0asm` header and stages the bundle (default `artifacts/scripting-runtime`, ignored by git). With `--jaic <native jaic>` it also compiles `jaifmt/wasm.jai` to `jaifmt.wasm` and records its SHA-256; without it no `jaifmt.wasm` is staged. Its `build-metadata.json` records the build command, target directory selection, paths and the module SHA-256. It refuses to start below 2 GiB free on the source, build or staging volumes. Release bundles are produced by `tools/package_browser_release.py`; see [compiler releases](compiler-releases.md).

```sh
rustup target add wasm32-unknown-unknown --toolchain nightly-2026-08-29
python3 tools/build_scripting_wasm.py --release --output artifacts/scripting-runtime
node tools/check_scripting_wasm.mjs artifacts/scripting-runtime/jai_wasm.wasm
node tools/check_jai_format_wasm.mjs artifacts/scripting-runtime
node tools/check_playground_stdlib.mjs artifacts/scripting-runtime
```

To try a local build in the hosted UI, run the portfolio's sync with `JAI_WEB_LOCAL=<bundle dir>` (see its `scripts/sync-jai-web.ts`).

## How to change it

- Different target or options: `options()` in `play.rs`.
- More host services (files, time): extend `SharedHost`/`SandboxHost::foreign` in `crates/jaic/src/interp/mod.rs`.
- Result shape: change `PlayResult::to_json`, then `engine.mjs` and the portfolio's engine wrapper (`src/lib/jai/engine.ts`) together.
- Bundle contents: `BUNDLED_GLUE` in `tools/build_scripting_wasm.py`, `REQUIRED` in `tools/package_browser_release.py` and `BUNDLE_FILES`/`BUNDLE_EXAMPLES` in `tools/check_browser_release.mjs` must agree. Example folders come from `tests/examples.json` (`bundle` key); see [language tour](tour.md).
- Gotchas: under `OS == .WASM`, Runtime_Support writes output through the foreign
  `wasm_write_string(count, data, to_standard_error)`, which `SandboxHost` implements. The jaic interpreter stores
  host pointers in 64-bit slots, which works on wasm32. Without a `budget`, an infinite loop hangs the calling thread.
- Panics: wasm32 panics abort, so a compiler bug traps the instance (`RuntimeError: unreachable`). `jai_play_reset`
  installs a panic hook (wasm32 only) that records the message; `engine.mjs::play` catches the trap and throws
  `The compiler crashed: <message>` with `compilerCrashed = true`. The instance is then unusable; create a new engine.
  `jai_play_panic_len/byte` read the message.
- No real clock on wasm32 (`std::time::SystemTime::now` panics): `#cycle_counter` counts calls, and `SandboxHost`
  implements `clock_gettime`/`gettimeofday`/`time` (virtual, deterministic, +1 microsecond per call), `nanosleep`/`usleep`
  (advance the virtual clock, never block) and `wasm_debug_break` (runtime error), so `current_time_monotonic`,
  `random_seed` and friends work. Other native `#foreign` symbols fail with ``foreign procedure `x` is not available here``
  or `unknown library`.
- **POSIX on WASM**: `OS == .WASM` is treated like Linux by `stdlib/POSIX` (Linux x86-64 bindings and struct layouts),
  `File`, `File_Utilities`, `Thread`, `Basic` time, so those modules compile and call into `SandboxHost`. Windowing
  and native-library modules still do not: `Window_Type` is `*void`, `Clipboard` is an in-memory string, `Window_Creation`,
  FreeType, stb_image, libclang and `Process` have no browser backend.
- **Files**: reads see the workspace (cwd is `/workspace`, so `read_entire_file("lib/data.txt")` works) and the bundled
  stdlib (`/stdlib/...`); writes go to an in-memory overlay (`/tmp` exists) that lives for one `play`. Nothing persists.
- **Threads**: run one after another on the interpreter stack, see [interpreter threads](../compiler/interpreter-threads.md).
  `Thread`, `Mutex`, `Condition_Variable`, `Semaphore` and `Thread_Group` work; a thread that busy-waits for a
  thread lower on the stack hangs.
- **Path semantics**: `normalize` clamps `..` at the root, so `#load "../../stdlib/X.jai"` from `/workspace/a.jai`
  reaches `/stdlib/X.jai` as on a real file system.
- Regression sweep for the wasm build: `node tools/check_playground_stdlib.mjs <staged-dir>` runs every
  `tests/stdlib/*.jai` through `engine.play` with a fresh wasm instance each (worker threads, 120 s timeout) and
  **every test must pass**: one failure exits non-zero, which fails CI and the release gate
  `check_browser_release.mjs`, so a broken bundle is never published. A test that needs something the browser
  lacks for only part of its work skips that part in Jai with `OS == .WASM` (the target the engine compiles for)
  and keeps the rest running: processes (`Process`, `BuildCpp`, the bindings generators' compiler runs),
  native C libraries (libc callbacks and variadics). Prefer a runtime `if OS == .WASM` over
  `#if` where the skipped code can still compile, so it stays type-checked in the browser. A test with nothing
  left to run there (sockets, windows, OpenGL, audio, libclang, Debug's stack capture, File_Async's worker
  thread) has a `playground` line in `tests/stdlib-runtime-skips.txt`
  instead; it still runs, and the check fails if it passes, so the line goes once it can. `PLAYGROUND_VERBOSE=1 ... name.jai`
  prints that test's output; passing test names runs only those.
- Debug browser-only behavior natively: `jaic run test.jai -os wasm` uses the same `SandboxHost`.

## Configuration

- Build: `cargo build -p jai-wasm --release --target wasm32-unknown-unknown`, or `tools/build_scripting_wasm.py` to also stage the bundle. `--target-dir` and `CARGO_TARGET_DIR` select the build directory ([build storage](../tools/build-storage.md)).
- Native test: `cargo test -p jai-wasm play`. `node tools/check_scripting_wasm.mjs <jai_wasm.wasm>` runs fixtures against the built module (exit codes, phases, nested `#load`, diagnostics) and the `tests/examples.json` programs (the tour) under their playground budget. `node tools/check_jai_format_wasm.mjs <jai_wasm.wasm | dir>` runs the formatter driver the way a Format button would. CI (`.github/workflows/ci.yml`) runs both on a debug build.
- The release module is about 11 MB, mostly the embedded stdlib.

## Dependencies

`jaic` (no external crates) and `jai-language-server`. Rust's `wasm32-unknown-unknown` target has no filesystem or process services; the interpreter's `SandboxHost` supplies the libc subset the stdlib needs (virtual clock, in-memory files, cooperative threads). No `SharedArrayBuffer` is involved, so any static host works. The Node checks need only Node's built-in WebAssembly, worker_threads and fs APIs; there are no npm dependencies.
