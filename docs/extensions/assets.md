# Assets

`load_asset(path)` loads a file a program ships with (texture, sound, level) by relative path. The same call works natively, in `-os wasm` programs and in the browser playground, where the page fetches it.

```jai
#import "Extensions/Assets";

bytes, ok := load_asset("assets/jacket.png");
defer array_free(bytes);
```

## How it works

- Only relative paths are accepted (no leading `/`, `..`, empty component, `\`, `:` or NUL), so behaviour is the same everywhere.
- Natively the file is read from the working directory, then from next to the executable.
- In the playground the module calls the host procedure `jai_asset_fetch`, which suspends the program while the page fetches (needs JSPI). `engine.mjs` resolves the path under `assets.base`, refuses results outside it, fetches with `credentials: "omit"` and `redirect: "error"`, and caps size at `assets.limit` (16 MiB default).
- Without an asset host (no `assets` option, or a WASI command) it falls back to `read_entire_file`.
- Playground bundles list an example's `assets/` folder in the `"assets"` key of its `<name>.json` index.

## How to change it

- The host protocol lives in three places that must agree: `stdlib/Extensions/Assets/module.jai`, `assetFunctions` in `crates/jai-wasm/js/engine.mjs`, and the stubs in `Wasi_Runtime`.
- The path rule is duplicated: `asset_path_is_relative` and `relative` in `engine.mjs`.
- Bundled asset types and size limits: `tools/browser/build_scripting_wasm.py`, `package_browser_release.py`, `check_browser_release.ts`.

Configure the browser side with `createEngine(bytes, { assets: { base, limit } })`.
