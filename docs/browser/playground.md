# Browser compiler (WebAssembly bundle)

`crates/jai-wasm` compiles the compiler core `crates/jaic` and the shared language server to `wasm32-unknown-unknown`, with the full `stdlib/` embedded, so `#import "Basic"` works offline. This repository ships the module, a small JavaScript glue file (`engine.mjs`), the example workspaces and release packaging, not a UI. The hosted playground at https://matteopolak.com/docs/jacket/playground lives in the portfolio repository (`matteopolak/portfolio`), which has the editor, worker and engine wrapper.

```js
const engine = await createEngine(wasmBytes);
const result = engine.play({ "main.jai": "main :: () { print(\"hi\\n\"); }" }, "main.jai", { budget: 1_000_000 });
// result: { exitCode, stdout, stderr, output, rendered, diagnostics }
```

A bundle (`tools/browser/build_scripting_wasm.py --output <dir>`) holds:

| File | Purpose |
| --- | --- |
| `jai_wasm.wasm` | Compiler, interpreter and `jai_lsp_*` language server; its only imports are the optional `jai_host` page functions |
| `webgpu_host.mjs`, `webgpu_bindings.generated.mjs` | Page host functions for [WebGPU](../extensions/webgpu.md) and canvas input |
| `engine.mjs` | `createEngine(bytes)` returns `play` and `lsp` |
| `jaifmt-playground.jai`, `jaifmt.wasm` | The formatter ([jaifmt](../tools/jaifmt.md#browser)); the wasm64 build is much faster and needs Memory64 |
| `build-metadata.json` | commit, toolchain, Wasm SHA-256 |
| `examples.json`, `<name>/**` | The [examples](#examples) the playground lists |

## How it works

1. `crates/jai-wasm/build.rs` embeds `stdlib/` with `include_bytes!` and links a 256 MiB shadow stack, because the compiler recurses deeply.
2. `play.rs::run(files, main)` builds a `VirtualFs` with the stdlib at `/stdlib/...` and user files at `/workspace/<path>`, uses `Options::for_program` (target WASM, `OS == .WASM`), installs a `SandboxHost` (libc shims, in-memory files, captured output), compiles, runs, and returns a JSON `PlayResult`. `exitCode` is null both when compilation failed and when the program hit a runtime error; the diagnostic `code` (`runtime-error`) tells them apart. `PlayOptions { budget }` bounds the interpreter in steps.
3. `play_exports.rs` is a pointer-free scalar ABI (`jai_play_reset`, `jai_play_push`, `jai_play_run`, ...). `engine.mjs` wraps it. Files are strings or `Uint8Array`s.
4. **Page host functions** (`host_bridge.rs`): a `#foreign` procedure the sandbox does not implement is offered to `host.functions[name]` through the `jai_host.call` import. `engine.mjs` provides `getentropy` itself. A promise result suspends the module through JSPI (`playAsync`).
5. **Standard input.** The sandbox never waits; the browser host (`PlayHost`) flushes output and calls the page function `jai_stdin_read(buffer, capacity)`, a promise settling with the byte count (0 is Ctrl+D). Without it or JSPI, input is closed.
6. `exit(n)` ends the run with `exitCode = n` and no diagnostic.

`play` is synchronous: run it in a Web Worker and terminate the worker to cancel. Without a `budget`, an infinite loop hangs the thread.

### Building and checking

```sh
rustup target add wasm32-unknown-unknown --toolchain nightly-2026-08-29
PYTHONPATH=tools python3 tools/browser/build_scripting_wasm.py --release --output target/scripting-runtime
node tools/browser/check_scripting_wasm.mjs target/scripting-runtime/jai_wasm.wasm
node tools/browser/check_jaifmt_wasm.mjs engine target/scripting-runtime
node tools/browser/check_playground_stdlib.mjs target/scripting-runtime
```

The build is offline, locked and non-incremental, and needs 2 GiB free. `--jaic <native jaic>` also builds `jaifmt.wasm`. To try a build in the hosted UI, run the portfolio's sync with `JAI_WEB_LOCAL=<bundle dir>`. Native test: `cargo test -p jai-wasm play`. A release module over 24 MiB fails (`MAX_RELEASE_WASM_BYTES`), because Cloudflare Pages rejects 25 MiB.

`check_playground_stdlib.mjs` runs every stdlib and regression test through `engine.play` with a fresh instance. A test needing something the browser lacks skips that part with a runtime `if OS == .WASM`; one with nothing left to run lists the modes in `unsupported_modes` of its `tests/manifest.json` row. `PLAYGROUND_VERBOSE=1` prints one test's output.

### Examples

Every folder of `examples/` is a small standalone workspace: `main.jai`, usually a `README.md`, more `.jai` files, an optional `assets/`. Names are kebab-case. `examples/jacket.toml` is shared; the bundle copies it into each.

Each example has a case in `tests/corpus/examples.json`. With `bundle: true` it is listed in the bundle with its `title` and `summary`. Examples that cannot run in a browser (`http`, `foreign-libraries`, windowed WebGPU) have no `bundle` and still run natively in `tools/differential/jaic_sweep.py examples`.

To add one: create `examples/<name>/main.jai`, add a row to `PROGRAMS` in `crates/jaic-cli/tests/examples.rs` and a case to `tests/corpus/examples.json` (`stdout_contains`; for the browser also `bundle: true`, `title`, `summary`, `args: ["-os", "wasm"]`). Gotchas:

- All files of a workspace share one global scope, so helper names must be distinct (`scale` collides with `Math`'s).
- A section that shows a failure sits behind a constant the reader flips (`BREAK_ON_PURPOSE`) so the default run stays green.
- Keep it browser-safe (no processes, sockets, windows or native libraries), short-running (auto-run reruns on every edit) and run `jaic jaifmt/main.jai -interp -- --check "$PWD/examples"`.
- `builder_to_string`, `NewArray` and `String` helpers take no allocator; wrap them in `push_allocator(temp)`.

### Browser releases

`tools/browser/package_browser_release.py` turns one clean commit into a relocatable bundle with a SHA-256 inventory, which the portfolio's `jai-web.yml` publishes as `jai-web-<sha>`.

1. The checkout must be clean, the pinned nightly at least 14 days old, and dependencies pass `tools/ci/check_dependency_age.py`.
2. It stages `build_scripting_wasm.py --release` output with a relocatable `build-metadata.json`. Only `.wasm`, `.mjs`, `.jai`, `.json`, `.md`, `.toml` (and `.png` assets) are accepted; at most 160 files, 64 MiB each, 128 MiB total.
3. `node tools/browser/check_browser_release.mjs <stage>` runs the staged engine through the exact inventory, every example, execution probes, formatting, the language server and `check_playground_stdlib.mjs`. Any failure publishes nothing.
4. The ZIP has fixed timestamps (identical inputs, identical bytes) and ships with `jai-playground.manifest.json`.

```sh
PYTHONPATH=tools python3 tools/browser/package_browser_release.py --output target/browser-release --jaic target/debug/jaic
```

`.github/workflows/browser-release.yml` runs this for one SHA on manual dispatch.

## How to change it

- Target or options: `options()` in `play.rs`. More host services: `SandboxHost::foreign` in `crates/jaic/src/interp/`. Result shape: `PlayResult::to_json`, `engine.mjs` and the portfolio's `src/lib/jai/engine.ts` together.
- Bundle contents: `BUNDLED_GLUE` in `build_scripting_wasm.py`, `REQUIRED` and `MAX_FILES` in `package_browser_release.py` and `BUNDLE_FILES` in `check_browser_release.mjs` must agree; coordinate with the portfolio. Probes must run staged files, not the source tree.
- Panics abort on wasm32, so a compiler bug traps the instance. `engine.mjs::play` throws `The compiler crashed: <message>` with `compilerCrashed = true`; recreate the engine.
- There is no real clock: `#cycle_counter` counts calls, and `SandboxHost` gives a virtual clock (+1 microsecond per call) whose sleeps never block. Other native `#foreign` symbols fail with ``foreign procedure `x` is not available here``.
- Files: reads see `/workspace` (the cwd) and `/stdlib`; writes go to an in-memory overlay (`/tmp` exists) that lasts one `play`.
- Example assets (`.png`, at most 256 KiB) are not sources; the program fetches them with `load_asset` when the page creates the engine with `assets: { base: "<name>/" }` ([assets](../extensions/assets.md)).
- Threads take turns on one host thread ([interpreter](../compiler/interpreter.md#threads)); a busy wait must poll (atomic read, `pause`, `sched_yield`).

## Configuration

- `cargo build -p jai-wasm --release --target wasm32-unknown-unknown`, or the build script. `--target-dir`/`CARGO_TARGET_DIR` select the build directory.
- Tests: `python3 tools/run_tests.py -p 'test_*.py'`, `node --test tools/browser/test_*.mjs`.
- `tests/corpus/examples.json` keys: `id`, `directory`, `main`, `title`, `summary`, `args`, `budget`, `bundle`, `mode`, `exit_code`, `stdout_contains`, `stdout_excludes`.

## Dependencies

`jaic` (no external crates) and `jai-language-server`. No `SharedArrayBuffer`, so any static host works. Tooling needs Python 3.11+, Node 24 (Memory64) and a pinned Rust nightly with `wasm32-unknown-unknown`; no npm dependencies.
