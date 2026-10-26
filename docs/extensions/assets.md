# Assets (`Extensions/Assets`)

## What it is

`load_asset(path) -> bytes, ok` loads a file a program ships with (a texture, a sound, a level) by a relative path. The same call works natively, in compiled `-os wasm` programs and in the browser playground, where the page fetches it. `examples/textured-cube` uses it to load its PNG texture.

```jai
#import "Extensions/Assets";

bytes, ok := load_asset("assets/jacket.png");
defer array_free(bytes);
```

## How it works

- **Paths.** `asset_path_is_relative` accepts only relative paths: no leading `/`, no `..` or empty component, no `\`, `:` or NUL. Anything else fails on every platform, so a program behaves the same everywhere.
- **Natively,** the path is read from the working directory, then from next to the executable.
- **In the playground,** the module calls the host procedure `jai_asset_fetch` (a `#foreign` with no library, answered by the page), which suspends the program while the page fetches. `jai_asset_size` and `jai_asset_take` then copy the bytes into a buffer the program allocates. `engine.mjs` provides all three when it is created with `assets: { base, limit }`:
  - It resolves the path against `base` (a directory URL) and refuses a result outside it.
  - It fetches with `credentials: "omit"` and `redirect: "error"`, so a shared playground link cannot read other same-origin data with the visitor's cookies.
  - It refuses a response over `limit` bytes (16 MiB by default).
  - Fetching needs JSPI (`playAsync`), like WebGPU.
- **No asset host,** meaning a page without `assets`, or a WASI command, where `Wasi_Runtime` exports stubs and `jai_host_provides` answers 0: the path is read with `read_entire_file` from the program's file system instead.
- **Playground bundles:** an example's `assets/` folder is listed under `"assets"` in its `<name>.json` index rather than opened as source ([playground](../browser/playground.md)). The playground passes `assets: { base: "<bundle>/<name>/" }`.

## How to change it

- **Host protocol:** `jai_asset_*` in `stdlib/Extensions/Assets/module.jai`, `assetFunctions` in `crates/jai-wasm/js/engine.mjs` and the stubs in `stdlib/Extensions/Wasi_Runtime/module.jai` must agree. `tools/browser/scripting_io_wasm.mjs` checks the browser path with a stubbed `fetch`.
- **Path rule:** `asset_path_is_relative` and `relative` in `engine.mjs` are the same rule; change both.
- **Bundled asset types and size:** `ASSET_SUFFIXES`/`MAX_ASSET_BYTES` in `tools/browser/build_scripting_wasm.py`, `ASSET_SUFFIXES` in `package_browser_release.py`, and the check in `check_browser_release.mjs`.

## Configuration

`createEngine(bytes, { assets: { base, limit } })`. There are no environment variables.

## Dependencies

`Basic`, `File`, `String` and natively `System` (the executable's path). In the browser: `engine.mjs`, `fetch` and JSPI.
