# Browser compiler (WebAssembly bundle)

## What it is

`crates/jai-wasm` compiles the compiler core `crates/jaic` and the shared language server to `wasm32-unknown-unknown`, with the full `stdlib/` (including `stdlib/prelude/`) embedded, so `#import "Basic"` works offline. This repository ships the module, a small JavaScript glue file (`engine.mjs`), the example workspaces and the release packaging, not a UI. The hosted playground at https://matteopolak.com/playground/jacket lives in the portfolio repository (`matteopolak/portfolio`), which has the editor, worker and engine wrapper.

A bundle (`tools/browser/build_scripting_wasm.py --output <dir>`) holds:

| File | Source | Purpose |
| --- | --- | --- |
| `jai_wasm.wasm` | `crates/jai-wasm` | Compiler, interpreter and `jai_lsp_*` language server; only imports are the optional `jai_host` page functions |
| `webgpu_host.mjs`, `webgpu_bindings.generated.mjs` | `crates/jai-wasm/js/` | Page host functions for [WebGPU](../extensions/webgpu.md) and canvas input |
| `engine.mjs` | `crates/jai-wasm/js/` | `createEngine(bytes)` returns `play` and `lsp`; the Node checks use it too |
| `jaifmt-playground.jai` | `jaifmt/playground.jai` | Formatter driver ([jaifmt](../tools/jaifmt.md#browser-playground)) |
| `jaifmt.wasm` | `jaifmt/wasm.jai`, built by native `jaic -os wasm` (`--jaic`) | jaifmt as a wasm64 WASI module, about 35 times faster than the driver; needs Memory64 ([jaifmt](../tools/jaifmt.md#webassembly-build-jaifmtwasm)) |
| `build-metadata.json` | build script / packager | commit, toolchain and Wasm SHA-256 |
| `examples.json`, `<name>/**` | `examples/<name>/` | The [example workspaces](#examples) the playground lists, one folder each with the shared `examples/jacket.toml` copied in as `<name>/jacket.toml`, and their one index |

`examples.json` is `{"schema_version": 2, "examples": [...]}`, the default (first shown) example first. Each entry names a workspace and lists its files; `assets` are binary files the program fetches itself:

```json
{
  "name": "textured-cube",
  "title": "Textured cube",
  "summary": "A spinning WebGPU cube with a PNG texture loaded as an asset.",
  "main": "main.jai",
  "files": ["main.jai", "jacket.toml"],
  "assets": ["assets/jacket.png"]
}
```

## How it works

1. `crates/jai-wasm/build.rs` walks `stdlib/` and generates `$OUT_DIR/stdlib_files.rs` (`include_bytes!`). `stdlib/prelude/module.jai` does `#load "../stdlib/prelude/module.jai"`, hence both trees. It also links with a 256 MiB shadow stack because the compiler recurses deeply.
2. `src/play.rs::run(files, main)` builds a `jaic::sema::VirtualFs` with the stdlib at `/stdlib/...` and user files at `/workspace/<path>`. Import paths are `<main dir>/modules` then `/stdlib`, preload is `/stdlib/prelude/module.jai`, target is WASM (`OS == .WASM`). It installs a `SharedHost` (wraps `SandboxHost`: libc shims, in-memory file system, captured stdout/stderr), calls `compile_program` then `run_program`, and returns a JSON `PlayResult` (`exitCode`, `stdout`, `stderr`, ordered `output` runs, `rendered`, `diagnostics[{severity, file, line, column, message, code}]`).

   `code` is the error's stable name from `DiagnosticKind::code` (`unavailable`, `static-assert`, `runtime-error`). `exitCode` is null both when compilation failed and when the program stopped with a runtime error; `runtime-error` tells them apart. `output` is everything written, in order. `run_with(files, main, PlayOptions { budget })` bounds the interpreter to `budget` basic blocks, including workspaces a metaprogram compiles.
3. `src/play_exports.rs` is a pointer-free scalar ABI: `jai_play_reset`, `jai_play_push(channel, byte)` (0 path, 1 contents, 2 main path, 3 program arguments, 4 fixed standard input), `jai_play_finish_file`, `jai_play_set_budget(thousands)`, `jai_play_set_styled(1)` (errors in `rendered` with ANSI colour and box drawing), `jai_play_run`, then `jai_play_output_len/byte` and `jai_play_error_len/byte`.
4. `engine.mjs::createEngine(wasmBytes, { host, assets })` returns `play(files, main, { budget, args, stdin })`, `playAsync` and, when exported, `lsp(message)`. `files` maps each workspace path to its contents, as a string (sent as UTF-8) or a `Uint8Array`; the stdlib checks send every file as bytes, so binary fixtures arrive intact.
5. **Page host functions** (`src/host_bridge.rs`): a `#foreign` procedure the sandbox does not implement is offered to `host.functions[name]` through the `jai_host.call` import with 64-bit argument slots; pointers are addresses in the module's memory (`jai_host_alloc`/`jai_host_free` give the page memory to hand back). `engine.mjs` reads addresses unsigned and range-checks them. A program asks whether a name is provided with `jai_host_provides(name, count)`. `engine.mjs` itself provides `getentropy` (the OS random bytes of `internal/crypto/random`) from `crypto.getRandomValues`; a page's `host.functions.getentropy` replaces it, and `Wasi_Runtime` answers it from WASI `random_get` in compiled wasm programs. A promise result suspends the module through JSPI (`playAsync`); afterwards the budget refills and the virtual clock advances by the real time spent.
6. **Standard input and arguments**: `SandboxHost` (`interp/sandbox/stdin.rs`) keeps a byte queue behind descriptor 0 and C `stdin`; the sandbox itself never waits, so a read that finds nothing sees end of input. The browser host (`PlayHost`) waits for the page: when a read cannot be satisfied (`SandboxHost::stdin_wants_data`) it flushes pending output to `host.output`, then calls the page function `jai_stdin_read(buffer, capacity)`, a promise that settles with the byte count (0 is Ctrl+D). Without `jai_stdin_read` or JSPI, input is closed. Cancel a waiting run by terminating the worker.
7. `exit(n)` raises `TrapKind::Exit(n)` and the run ends with `exitCode = n`, output kept, no diagnostic.

`play` is synchronous; run it in a Web Worker and terminate the worker to cancel. Without a `budget`, an infinite loop hangs the thread.

### Building and checking

```sh
rustup target add wasm32-unknown-unknown --toolchain nightly-2026-08-29
PYTHONPATH=tools python3 tools/browser/build_scripting_wasm.py --release --output target/scripting-runtime
node tools/browser/check_scripting_wasm.mjs target/scripting-runtime/jai_wasm.wasm
node tools/browser/check_jaifmt_wasm.mjs engine target/scripting-runtime
node tools/browser/check_playground_stdlib.mjs target/scripting-runtime
```

`build_scripting_wasm.py` runs an offline, locked, non-incremental `cargo build -p jai-wasm --target wasm32-unknown-unknown`, checks the `\0asm` header, stages the bundle (default `target/scripting-runtime`) and refuses to start below 2 GiB free. With `--jaic <native jaic>` it also builds `jaifmt.wasm`. To try a local build in the hosted UI, run the portfolio's sync with `JAI_WEB_LOCAL=<bundle dir>`. Native test: `cargo test -p jai-wasm play`. The release module is about 19 MB, mostly the stdlib's `.jai` sources (`crates/jai-wasm/build.rs` bundles only those, never `tests/` or data files like the SDL3 binding metadata). A release build over 24 MiB fails, because Cloudflare Pages rejects files over 25 MiB (`MAX_RELEASE_WASM_BYTES`).

`check_playground_stdlib.mjs` runs every case of the test catalogue (`stdlib/**/tests` and `tests/compiler/regressions`, see [testing](../tools/testing.md#stdlib-runtime-tests)) through `engine.play` with a fresh instance each, and every test must pass. A test needing something the browser lacks skips that part with a runtime `if OS == .WASM` (preferred over `#if`, so the code stays type-checked); a test with nothing left to run lists the browser modes in `unsupported_modes` of its row in the module's `tests/manifest.json`. `PLAYGROUND_VERBOSE=1 ... name.jai` prints one test's output.

### Examples

Every folder of `examples/` is a small, standalone workspace: a `main.jai`, usually a short `README.md` guide, more `.jai` files it `#load`s, and an optional `assets/` folder. Names are kebab-case. `examples/jacket.toml` is shared by all of them: the `[format]` settings the playground's formatter reads and the lint settings, found by `jaifmt`, `jailint` and `jacket` walking up from a workspace. The bundle copies it into each workspace so a browser workspace is self-contained.

Each example has a case in `tests/corpus/examples.json` (`id` is the folder name). A case with `bundle: true` is listed in the bundle, in file order, with its `title` and one-sentence `summary`; `build_scripting_wasm.py` copies its folder and writes `examples.json`. Examples that cannot run in a browser (`http`, `foreign-libraries`, the windowed `webgpu-triangle` and `webgpu-raymarch`, programs that return a status) have no `bundle` and still run natively in `tools/differential/jaic_sweep.py examples` (`jaic main.jai -interp`, or only `-check` with `mode: "check"`). `check_scripting_wasm.mjs` and `check_browser_release.mjs` run the bundled ones in the wasm engine under the case's `budget` via `tools/browser/examples_wasm.mjs`. The portfolio's `starter.ts` fetches `examples.json`, then each listed file of the chosen example, falling back to a two-file starter on any failure.

To add an example: create `examples/<name>/main.jai` (plus a `README.md` if it needs a guide), add a row to `PROGRAMS` in `crates/jaic-cli/tests/examples.rs` and a case to `tests/corpus/examples.json` with `stdout_contains` lines, and for the browser set `bundle: true`, `title`, `summary` and `args: ["-os", "wasm"]`. All files of a workspace share one global scope, so helpers need distinct names (a helper named `scale` collides with `Math`'s). A section that shows a failure must sit behind a constant the reader flips (`BREAK_ON_PURPOSE`) so the default run stays green. Keep it browser-safe (no processes, sockets, windows or native libraries), short-running (auto-run reruns on every edit) and short-lined, and run jaifmt on it: `jaic jaifmt/main.jai -interp -- --check "$PWD/examples"`. `builder_to_string`, `NewArray` and `String` helpers take no allocator argument; wrap them in `push_allocator(temp)`.

### Browser releases

`tools/browser/package_browser_release.py` turns one clean compiler commit into a relocatable bundle with a SHA-256 inventory, which the portfolio consumes (its `jai-web.yml` runs the same packager and publishes `jai-web-<sha>` releases). Steps:

1. The checkout must be clean, the pinned nightly in `rust-toolchain.toml` at least 14 days old, locked dependencies pass `tools/ci/check_dependency_age.py`, and `tools/ci/check_ci_sources.py` checks literal `include_*!` inputs.
2. It runs `build_scripting_wasm.py --release` into a temporary stage and replaces the build metadata with a relocatable `build-metadata.json` (`commit`, `schema_version`, `toolchain`, `wasm_sha256`).
3. The stage must contain `jai_wasm.wasm`, `engine.mjs`, `jaifmt-playground.jai`, `jaifmt.wasm`, `build-metadata.json`, `README.md`, `examples.json` and `basics/main.jai`. Only `.wasm`, `.mjs`, `.jai`, `.json`, `.md` and `.toml` files are accepted (and `.png` in an example's `assets/`); at most 160 files, 64 MiB each, 128 MiB total.
4. `node tools/browser/check_browser_release.mjs <stage>` runs the **staged** engine: the exact inventory, every example folder matching `examples.json` (and carrying `jacket.toml`), execution probes (exit codes, phases, nested `#load`, diagnostics, a `#run` workspace message loop, the virtual clock, the budget, a refused `#foreign`), every bundled example, a format, the language server, and `check_playground_stdlib.mjs`. Any failure means nothing is published.
5. The ZIP uses fixed timestamps and modes (identical inputs give identical bytes), is read back against the inventory, and is written with `jai-playground.manifest.json` (`schema_version: 2`, `commit`, `files: [{path, size, sha256}]`, `archive`, `capabilities`) into an absent or empty output directory.

Run it with `PYTHONPATH=tools python3 tools/browser/package_browser_release.py --output target/browser-release --jaic target/debug/jaic`; `--jaic` is required (it builds `jaifmt.wasm`, needing LLVM 23 and `wasm-ld`; [wasm target](../native/wasm-target.md)). `.github/workflows/browser-release.yml` does this for one full SHA on manual dispatch with read-only permissions and uploads the two files as a workflow artifact.

## How to change it

- **Target or options**: `options()` in `play.rs`. **More host services** (files, time): `SharedHost`/`SandboxHost::foreign` in `crates/jaic/src/interp/`. **Result shape**: `PlayResult::to_json`, `engine.mjs` and the portfolio's `src/lib/jai/engine.ts` together.
- **Bundle contents**: `BUNDLED_GLUE` in `build_scripting_wasm.py`, `REQUIRED` and `MAX_FILES` in `package_browser_release.py`, `BUNDLE_FILES` in `check_browser_release.mjs` must agree; coordinate with the portfolio's verifier and sync script. Probes must run the staged files, not the source tree.
- **Panics**: wasm32 panics abort, so a compiler bug traps the instance. `jai_play_reset` installs a panic hook that records the message; `engine.mjs::play` throws `The compiler crashed: <message>` with `compilerCrashed = true`, and the engine must be recreated.
- **No real clock** on wasm32: `#cycle_counter` counts calls; `SandboxHost` provides a virtual deterministic clock (+1 microsecond per call), `nanosleep`/`usleep` that advance it and never block, and `wasm_debug_break`. Other native `#foreign` symbols fail with ``foreign procedure `x` is not available here`` or `unknown library`. Under `OS == .WASM`, Runtime_Support writes output through the foreign `wasm_write_string`.
- **Files**: runtime reads see the workspace (cwd `/workspace`) and `/stdlib`; writes go to an in-memory overlay (`/tmp` exists) that lasts one `play`. `normalize` clamps `..` at the root.
- **Example assets**: files in a bundled example's `assets/` folder (`.png`, at most 256 KiB each) are not workspace sources. The program fetches them with `load_asset` when the page creates the engine with `assets: { base: "<name>/" }` ([assets](../extensions/assets.md)).
- **Threads** take turns on one host thread ([interpreter](../compiler/interpreter.md#threads)); a busy wait must poll (atomic read, `pause`, `sched_yield`) for others to run, and sleeping only moves the virtual clock. Public `Thread` and `POSIX` modules are not browser WASM APIs.

## Configuration

- `cargo build -p jai-wasm --release --target wasm32-unknown-unknown`, or the build script to stage the bundle; `--target-dir` and `CARGO_TARGET_DIR` select the build directory.
- Tests: `python3 tools/run_tests.py -p 'test_*.py'`, `node --test tools/browser/test_*.mjs`. CI (`.github/workflows/ci.yml`) runs the Node checks on a debug build.
- Portfolio: per-example file and byte limits (`MAX_TOUR_FILES`, `MAX_TOUR_BYTES`) in `starter.ts`.
- `tests/corpus/examples.json` keys: `id`, `directory`, `main`, `title`, `summary`, `args`, `budget`, `bundle`, `mode`, `exit_code`, `stdout_contains`, `stdout_excludes`.

## Dependencies

`jaic` (no external crates) and `jai-language-server`. The interpreter's `SandboxHost` supplies the libc subset the stdlib needs; no `SharedArrayBuffer` is used, so any static host works. Tooling needs Python 3.11+, `git`, Node (24 for Memory64) and a pinned Rust nightly with `wasm32-unknown-unknown`; no npm dependencies.
