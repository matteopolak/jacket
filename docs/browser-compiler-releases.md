# Browser compiler releases

## What it is

`tools/package_browser_release.py` produces relocatable browser compiler assets for the portfolio consumer. Each bundle belongs to one clean full compiler commit; it contains the actual Rust-generated WebAssembly and compiler-owned frontend, with an exhaustive digest inventory.

## How it works

The producer requires a clean Git checkout, including untracked source files. It checks public literal source inputs, validates the dated Rust nightly against the 14-day minimum, checks every locked Cargo registry dependency with the existing publication-age checker, and invokes `build_scripting_wasm.py --release`. The existing pinned Cargo and target-directory rules apply. Child builds/probes clear injected compiler/linker wrappers, Rust flags and Node preload options while preserving the selected Cargo target and enforcing nonincremental builds. CI fetches the locked graph before the offline build; local builds must explicitly select the Jai APFS cache documented in [build storage](build-storage.md).

The build stages regular browser assets recursively while excluding `node_modules`, Git metadata and target caches. The producer checks the Wasm digest against its successful build receipt, removes that host-path receipt, and adds `release.json` with the exact compiler revision. `check_browser_release.mjs` checks relative asset dependencies and imports the **staged** engine and Wasm. Its real execution probes cover scalar and wide signed results, 32-bit browser pointer layout, compile-time/runtime phase selection, a nested source bundle, fuel rejection and source diagnostics. A missing/stale/invalid module or failed probe prevents publication. This Node gate establishes runtime execution and literal asset dependencies; rendered editing, iframe initialization and full browser behavior require separate browser acceptance.

The compiler-owned prelude is embedded in Wasm. Current source bundles provide their own imports; the LSP uses open-document source files. No original/reference corpus, supplied native artifacts or unnecessary external module sources belong in the archive. The shared LSP uses the same Wasm module. When `lsp-client.mjs` is present, the gate requires genuine language exports; initialize, open, definition, hover, completion and versioned change diagnostics must pass before publishing. An independent runtime run then proves that LSP document state did not replace runtime source. Runtime-only builds record `capabilities.lsp: false`; rich-editor builds cannot publish missing language support. There is no placeholder LSP asset.

Source cleanliness and the commit are checked again after build/probes and before finalization. Assets are bounded to 512 files, 64 MiB per file and 128 MiB total. Only normalized relative POSIX regular-file paths are accepted; traversal, absolute/drive paths, symlinks, protected/generated trees, native executables, source maps and build receipts are rejected. Zip timestamps/modes are fixed. Every archive member is read back and checked against its recorded size/digest. The output volume must retain the 2 GiB free-space floor, with additional bounded staging/archive headroom checked before writes and finalization. The output directory must be absent or empty; both assets are finalized together by a directory rename, and existing releases are never overwritten.

The machine-readable contract is [manifest schema v1](../tools/browser-release-manifest.schema.json). Archive normalization, unique names, aggregate byte limits and file types remain mandatory producer/consumer checks beyond that JSON schema.

The producer outputs exactly:

- `jai-playground.zip`, whose root contains `index.html`, runtime/frontend assets and `release.json`.
- `jai-playground.manifest.json`, a separate JSON asset with `schema_version: 1`, `commit` (40 lowercase hex digits), `dirty_checkout: false`, `entrypoint: "index.html"`, `files: [{path, size, sha256}]`, the archive's `name`, `size` and SHA-256, and `capabilities: {runtime: true, lsp: <actual probe result>}`. `files` exhaustively inventories every ZIP member, excluding the external manifest itself.

`.github/workflows/browser-release.yml` builds one reviewed full SHA through manual dispatch, tests the portable runtime and helpers, executes the staged Wasm gate and uploads only the two verified producer assets. It has read-only repository permissions and no release-publishing step. The portfolio's `jai-web.yml` owns immutable `jai-web-<full-sha>` releases in `matteopolak/portfolio`, pins manifest/archive hashes, and mounts the verified files under `/jai/<full-sha>/`.

## How to change it

Extend runtime/frontend output in `web/scripting-runtime` and preserve base-relative URLs. Keep `release.json` available for the editor's revision-aware embed handshake. The editor owns `?embed=1` ready/error messages; `ready` must follow actual compiler-worker initialization. Package tests do not establish that UI handshake.

Change archive policy and schema in `package_browser_release.py` together with the portfolio consumer. Maintain exhaustive files, bounded paths/bytes, refusal of dirty/source-changing builds and atomic output. The Python tests use authored inert fixtures and mock builds/Node calls; they prove archive/refusal behavior, not compilation. The Node tests prove dependency relocation/refusal and that a header-only module fails actual execution.

The frontend owner supplies pinned root `package.json`/`package-lock.json` and the actual build/checker scripts. With that complete inventory, packaging runs `node tools/check_editor_dependencies.mjs` before `npm ci --ignore-scripts`, then `npm run build` and `npm test`. Partial configuration is rejected. The bundle and license notices must reproduce tracked bytes; new source changes fail the clean-release gate. `node_modules` and local editor build/age receipts are ignored and never packaged. The language protocol probe uses the actual `engine.lsp` bridge and source-syntax results, not semantic inference or compile-time execution.

## Configuration

```sh
# CI provides fetched locked dependencies and the installed wasm target.
python3 tools/package_browser_release.py --toolchain
CARGO_TARGET_DIR=/Volumes/CodexBuilds/targets/jai CARGO_INCREMENTAL=0 \
  python3 tools/package_browser_release.py --output artifacts/browser-release

# Helper tests perform no Cargo build.
python3 -m unittest discover -s tools -p test_package_browser_release.py -v
node --test tools/test_browser_release.mjs
```

`--output` selects a new/empty directory. Inside the checkout it must be gitignored, so generated output cannot falsify source cleanliness. `--target-dir` overrides the environment and actual Cargo configuration through the existing [shared helper](build-storage.md). `--toolchain` validates and prints the dated nightly without building. The manual workflow's `compiler_revision` must be an exact lowercase full commit SHA; no mutable branch/latest lookup is used. CI uses its own ephemeral explicitly selected target, while local Jai builds use the T7-backed APFS target.

## Dependencies

Python 3.11+ standard-library TOML/JSON/Zip support, independently installed Node/npm (protected original tool paths and aliases are refused), Node's standard WebAssembly/ES-module APIs, independently installed pinned Rustup/Cargo with `wasm32-unknown-unknown`, and the existing dependency-age/source-input/build-path helpers. This producer tooling introduces no registry dependency; the separate editor and language-server packets own their pinned npm/Cargo dependencies and publication-age checks. The runtime build depends only on the portable compiler/interpreter graph and needs no LLVM backend libraries. The frontend and portable LSP owners supply their actual implementations; the portfolio consumes and publishes verified assets separately.

The staged verification gate additionally executes the actual `worker.mjs` through a Node host adapter against the generated Wasm. It checks cloned multi-file snapshots, independent language-server state, execution-worker termination and a fresh-worker restart. This is a real worker/runtime gate; rendered browser editing and iframe behavior remain a separate acceptance check.
