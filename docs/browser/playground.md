# Browser compiler (WebAssembly bundle)

## What it is

`crates/jai-wasm` compiles the compiler core `crates/jaic` and the shared language server to `wasm32-unknown-unknown`, with the full `stdlib/` (including `stdlib/prelude/`) embedded, so `#import "Basic"` works offline. This repository ships the module, a small JavaScript glue file (`engine.mjs`), the default language tour and the release packaging, not a UI. The hosted playground at https://matteopolak.com/playground/jacket lives in the portfolio repository (`matteopolak/portfolio`), which has the editor, worker and engine wrapper.

A bundle (`tools/build_scripting_wasm.py --output <dir>`) holds:

| File | Source | Purpose |
| --- | --- | --- |
| `jai_wasm.wasm` | `crates/jai-wasm` | Compiler, interpreter and `jai_lsp_*` language server; only imports are the optional `jai_host` page functions |
| `webgpu_host.mjs`, `webgpu_bindings.generated.mjs` | `crates/jai-wasm/js/` | Page host functions for [WebGPU](../extensions/webgpu.md) and canvas input |
| `engine.mjs` | `crates/jai-wasm/js/` | `createEngine(bytes)` returns `play` and `lsp`; the Node checks use it too |
| `jaifmt-playground.jai` | `jaifmt/playground.jai` | Formatter driver ([jaifmt](../tools/jaifmt.md#browser-playground)) |
| `jaifmt.wasm` | `jaifmt/wasm.jai`, built by native `jaic -os wasm` (`--jaic`) | jaifmt as a wasm64 WASI module, about 35 times faster than the driver; needs Memory64 ([jaifmt](../tools/jaifmt.md#webassembly-build-jaifmtwasm)) |
| `build-metadata.json` | build script / packager | commit, toolchain and Wasm SHA-256 |
| `tour.json`, `tour/**` | `examples/tour/` | The [language tour](#language-tour) and its file index (`{schema_version, main, files[]}`) |

## How it works

1. `crates/jai-wasm/build.rs` walks `stdlib/` and generates `$OUT_DIR/stdlib_files.rs` (`include_bytes!`). `stdlib/prelude/module.jai` does `#load "../stdlib/prelude/module.jai"`, hence both trees. It also links with a 256 MiB shadow stack because the compiler recurses deeply.
2. `src/play.rs::run(files, main)` builds a `jaic::sema::VirtualFs` with the stdlib at `/stdlib/...` and user files at `/workspace/<path>`. Import paths are `<main dir>/modules` then `/stdlib`, preload is `/stdlib/prelude/module.jai`, target is WASM (`OS == .WASM`). It installs a `SharedHost` (wraps `SandboxHost`: libc shims, in-memory file system, captured stdout/stderr), calls `compile_program` then `run_program`, and returns a JSON `PlayResult` (`exitCode`, `stdout`, `stderr`, ordered `output` runs, `rendered`, `diagnostics[{severity, file, line, column, message, code}]`).

   `code` is the error's stable name from `DiagnosticKind::code` (`unavailable`, `static-assert`, `runtime-error`). `exitCode` is null both when compilation failed and when the program stopped with a runtime error; `runtime-error` tells them apart. `output` is everything written, in order. `run_with(files, main, PlayOptions { budget })` bounds the interpreter to `budget` basic blocks, including workspaces a metaprogram compiles.
3. `src/play_exports.rs` is a pointer-free scalar ABI: `jai_play_reset`, `jai_play_push(channel, byte)` (0 path, 1 contents, 2 main path, 3 program arguments, 4 fixed standard input), `jai_play_finish_file`, `jai_play_set_budget(thousands)`, `jai_play_set_styled(1)` (errors in `rendered` with ANSI colour and box drawing), `jai_play_run`, then `jai_play_output_len/byte` and `jai_play_error_len/byte`.
4. `engine.mjs::createEngine(wasmBytes, { host })` returns `play(files, main, { budget, args, stdin })`, `playAsync` and, when exported, `lsp(message)`.
5. **Page host functions** (`src/host_bridge.rs`): a `#foreign` procedure the sandbox does not implement is offered to `host.functions[name]` through the `jai_host.call` import with 64-bit argument slots; pointers are addresses in the module's memory (`jai_host_alloc`/`jai_host_free` give the page memory to hand back). `engine.mjs` reads addresses unsigned and range-checks them. A program asks whether a name is provided with `jai_host_provides(name, count)`. A promise result suspends the module through JSPI (`playAsync`); afterwards the budget refills and the virtual clock advances by the real time spent.
6. **Standard input and arguments**: `SandboxHost` (`interp/sandbox/stdin.rs`) keeps a byte queue behind descriptor 0 and C `stdin`; the sandbox itself never waits, so a read that finds nothing sees end of input. The browser host (`PlayHost`) waits for the page: when a read cannot be satisfied (`SandboxHost::stdin_wants_data`) it flushes pending output to `host.output`, then calls the page function `jai_stdin_read(buffer, capacity)`, a promise that settles with the byte count (0 is Ctrl+D). Without `jai_stdin_read` or JSPI, input is closed. Cancel a waiting run by terminating the worker.
7. `exit(n)` raises `TrapKind::Exit(n)` and the run ends with `exitCode = n`, output kept, no diagnostic.

`play` is synchronous; run it in a Web Worker and terminate the worker to cancel. Without a `budget`, an infinite loop hangs the thread.

### Building and checking

```sh
rustup target add wasm32-unknown-unknown --toolchain nightly-2026-08-29
python3 tools/build_scripting_wasm.py --release --output artifacts/scripting-runtime
node tools/check_scripting_wasm.mjs artifacts/scripting-runtime/jai_wasm.wasm
node tools/check_jai_format_wasm.mjs artifacts/scripting-runtime
node tools/check_playground_stdlib.mjs artifacts/scripting-runtime
```

`build_scripting_wasm.py` runs an offline, locked, non-incremental `cargo build -p jai-wasm --target wasm32-unknown-unknown`, checks the `\0asm` header, stages the bundle (default `artifacts/scripting-runtime`) and refuses to start below 2 GiB free. With `--jaic <native jaic>` it also builds `jaifmt.wasm`. To try a local build in the hosted UI, run the portfolio's sync with `JAI_WEB_LOCAL=<bundle dir>`. Native test: `cargo test -p jai-wasm play`. The release module is about 11 MB, mostly the stdlib.

`check_playground_stdlib.mjs` runs every `tests/stdlib/*.jai` through `engine.play` with a fresh instance each, and every test must pass. A test needing something the browser lacks skips that part with a runtime `if OS == .WASM` (preferred over `#if`, so the code stays type-checked); a test with nothing left to run gets a `playground` line in `tests/stdlib-runtime-skips.txt` (the check fails if it passes, so remove the line then). `PLAYGROUND_VERBOSE=1 ... name.jai` prints one test's output.

### Language tour

`examples/tour/` is a multi-file Jai program, one topic per file (`basics`, `types`, `data`, `memory`, `generics`, `meta`, `safety`, `threads`, `files`, `machine` for `#asm` and `Long_Double`, `finale/raymarch.jai`, and `gpu/raymarch_gpu.jai` as a WebGPU encore), loaded by `main.jai` from a table of `{title, procedure}` stops, with `tour.md` as a plain-text guide. The playground opens it as the default workspace; it ships in the bundle as `tour/` plus `tour.json`:

The tour's case in `tests/examples.json` has `bundle: "tour"`, which makes `build_scripting_wasm.py` copy the folder and write the index. `tools/jaic-sweep.py examples` runs it natively (`jaic main.jai -interp -os wasm`); `check_scripting_wasm.mjs` and `check_browser_release.mjs` run it in the wasm engine under the case's `budget` via `tools/examples_wasm.mjs`. The portfolio's `starter.ts::loadStarter` fetches `tour.json` and each listed file, falling back to a two-file starter on any failure.

To add a stop: write `examples/tour/<topic>/<file>.jai` with a `tour_<topic> :: ()` procedure, `#load` it in `main.jai`, add `.{ "Title", tour_<topic> }` to `stops`, and mention it in `tour.md`. All files share one global scope, so helpers need distinct names (a helper named `scale` collides with `Math`'s). A stop that shows a failure must sit behind a constant the reader flips (`BREAK_ON_PURPOSE`) so the default run stays green. Keep it browser-safe (no processes, sockets, windows or native libraries), short-running (auto-run reruns on every edit) and short-lined, and run jaifmt on it: `jaic jaifmt/main.jai -interp -- --check "$PWD/examples/tour"`. Update `stdout_contains` in `tests/examples.json` when output changes. `builder_to_string`, `NewArray` and `String` helpers take no allocator argument; wrap them in `push_allocator(temp)`.

### Browser releases

`tools/package_browser_release.py` turns one clean compiler commit into a relocatable bundle with a SHA-256 inventory, which the portfolio consumes (its `jai-web.yml` runs the same packager and publishes `jai-web-<sha>` releases). Steps:

1. The checkout must be clean, the pinned nightly in `rust-toolchain.toml` at least 14 days old, locked dependencies pass `tools/check_dependency_age.py`, and `tools/check_ci_sources.py` checks literal `include_*!` inputs.
2. It runs `build_scripting_wasm.py --release` into a temporary stage and replaces the build metadata with a relocatable `build-metadata.json` (`commit`, `schema_version`, `toolchain`, `wasm_sha256`).
3. The stage must contain `jai_wasm.wasm`, `engine.mjs`, `jaifmt-playground.jai`, `jaifmt.wasm`, `build-metadata.json`, `README.md`, `tour.json` and `tour/main.jai`. Only `.wasm`, `.mjs`, `.jai`, `.json` and `.md` files are accepted; at most 64 files, 64 MiB each, 128 MiB total.
4. `node tools/check_browser_release.mjs <stage>` runs the **staged** engine: the exact inventory, a `tour/` matching `tour.json`, execution probes (exit codes, phases, nested `#load`, diagnostics, a `#run` workspace message loop, the virtual clock, the budget, a refused `#foreign`), the tour, a format, the language server, and `check_playground_stdlib.mjs`. Any failure means nothing is published.
5. The ZIP uses fixed timestamps and modes (identical inputs give identical bytes), is read back against the inventory, and is written with `jai-playground.manifest.json` (`schema_version: 2`, `commit`, `files: [{path, size, sha256}]`, `archive`, `capabilities`) into an absent or empty output directory.

Run it with `python3 tools/package_browser_release.py --output artifacts/browser-release --jaic target/debug/jaic`; `--jaic` is required (it builds `jaifmt.wasm`, needing LLVM 23 and `wasm-ld`; [wasm target](../native/wasm-target.md)). `.github/workflows/browser-release.yml` does this for one full SHA on manual dispatch with read-only permissions and uploads the two files as a workflow artifact.

## How to change it

- **Target or options**: `options()` in `play.rs`. **More host services** (files, time): `SharedHost`/`SandboxHost::foreign` in `crates/jaic/src/interp/`. **Result shape**: `PlayResult::to_json`, `engine.mjs` and the portfolio's `src/lib/jai/engine.ts` together.
- **Bundle contents**: `BUNDLED_GLUE` in `build_scripting_wasm.py`, `REQUIRED` and `MAX_FILES` in `package_browser_release.py`, `BUNDLE_FILES`/`BUNDLE_EXAMPLES` in `check_browser_release.mjs` must agree; coordinate with the portfolio's verifier and sync script. Probes must run the staged files, not the source tree.
- **Panics**: wasm32 panics abort, so a compiler bug traps the instance. `jai_play_reset` installs a panic hook that records the message; `engine.mjs::play` throws `The compiler crashed: <message>` with `compilerCrashed = true`, and the engine must be recreated.
- **No real clock** on wasm32: `#cycle_counter` counts calls; `SandboxHost` provides a virtual deterministic clock (+1 microsecond per call), `nanosleep`/`usleep` that advance it and never block, and `wasm_debug_break`. Other native `#foreign` symbols fail with ``foreign procedure `x` is not available here`` or `unknown library`. Under `OS == .WASM`, Runtime_Support writes output through the foreign `wasm_write_string`.
- **Files**: runtime reads see the workspace (cwd `/workspace`) and `/stdlib`; writes go to an in-memory overlay (`/tmp` exists) that lasts one `play`. `normalize` clamps `..` at the root.
- **Threads** take turns on one host thread ([interpreter](../compiler/interpreter.md#threads)); a busy wait must poll (atomic read, `pause`, `sched_yield`) for others to run, and sleeping only moves the virtual clock. Public `Thread` and `POSIX` modules are not browser WASM APIs.

## Configuration

- `cargo build -p jai-wasm --release --target wasm32-unknown-unknown`, or the build script to stage the bundle; `--target-dir` and `CARGO_TARGET_DIR` select the build directory.
- Tests: `python3 -m unittest discover -s tools -p 'test_*.py'`, `node --test tools/test_browser_release.mjs`. CI (`.github/workflows/ci.yml`) runs the Node checks on a debug build.
- Portfolio: `MAX_TOUR_FILES` (64) and `MAX_TOUR_BYTES` (1 MiB) in `starter.ts`.
- `tests/examples.json` keys: `directory`, `main`, `args`, `budget`, `bundle`, `stdout_contains`, `stdout_excludes`.

## Dependencies

`jaic` (no external crates) and `jai-language-server`. The interpreter's `SandboxHost` supplies the libc subset the stdlib needs; no `SharedArrayBuffer` is used, so any static host works. Tooling needs Python 3.11+, `git`, Node (24 for Memory64) and a pinned Rust nightly with `wasm32-unknown-unknown`; no npm dependencies.
