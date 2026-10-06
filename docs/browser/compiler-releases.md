# Browser compiler releases

## What it is

`tools/package_browser_release.py` turns one clean compiler commit into a relocatable, digest-inventoried browser bundle: the [WebAssembly compiler](playground.md), its `engine.mjs` glue, the jaifmt driver, the [language tour](tour.md), release metadata and a README. The portfolio site consumes it to serve the hosted playground (https://matteopolak.com/playground/jai). There is no UI in the bundle.

## How it works

1. **Preconditions.** The checkout must be clean, including untracked files. The dated Rust nightly in `rust-toolchain.toml` must be at least 14 days old, every locked Cargo registry dependency passes `tools/check_dependency_age.py`, and `tools/check_ci_sources.py` checks literal `include_*!` inputs. Child processes run with compiler/linker wrappers, Rust flags and Node preload options cleared, and with `CARGO_INCREMENTAL=0`.
2. **Build.** It runs `build_scripting_wasm.py --release` into a temporary stage. It checks the staged module against the build receipt's SHA-256, then replaces the receipt, which names host paths, with a relocatable `build-metadata.json`:

   ```json
   { "commit": "<40 hex>", "schema_version": 1, "toolchain": "nightly-2026-08-29", "wasm_sha256": "<64 hex>" }
   ```

3. **Inventory.** The stage must contain `jai_wasm.wasm`, `engine.mjs`, `jaifmt-playground.jai`, `build-metadata.json`, `README.md`, `tour.json` and `tour/main.jai` (the rest of the tour sits under `tour/`). Only `.wasm`, `.mjs`, `.jai`, `.json` and `.md` regular files with normalized relative paths are accepted. The bundle is limited to 64 files, 64 MiB per file and 128 MiB total. Each file's size and SHA-256 are recorded.
4. **Probe.** `node tools/check_browser_release.mjs <stage> --report <json>` imports the **staged** `engine.mjs` and Wasm. It requires the exact top-level inventory (the five files plus `tour.json`), a `tour/` folder whose files are exactly those `tour.json` lists, a matching `wasm_sha256` and a self-contained `engine.mjs`. It then runs execution probes (exit codes, compile-time/runtime phases, nested `#load`, diagnostics, Hash_Table, a `#run` workspace message loop, the virtual clock, separate stdout/stderr, the execution budget, a refused `#foreign`), runs the staged tour within the playground budget and checks its key output lines (`tests/examples.json`), formats a file with the staged driver and, when the module exports it, checks the language server (initialize, definition, hover, completion, versioned diagnostics). Last, it runs `check_playground_stdlib.mjs`, whose pass set must match `tools/playground_stdlib_expected.json`. Any failure means nothing is published.
5. **Archive.** Source cleanliness and the commit are checked again. The ZIP uses fixed timestamps and modes, so the same inputs give identical bytes, and every member is read back and checked against the inventory. The archive and manifest are moved into the output directory together. The output directory must be absent or empty, and existing releases are never overwritten.

The producer outputs exactly:

- `jai-playground.zip`, whose root holds the bundle files and the `tour/` tree.
- `jai-playground.manifest.json`: `schema_version: 2`, `commit`, `dirty_checkout: false`, `files: [{path, size, sha256}]` (every ZIP member), `archive: {name, size, sha256}` and `capabilities: {runtime: true, lsp: <probe result>}`. The JSON schema is [`tools/browser-release-manifest.schema.json`](../../tools/browser-release-manifest.schema.json).

Schema v2 removed the standalone UI (`index.html`, `worker.mjs`, editor files, `release.json`) and the `entrypoint` field. A v1 consumer that requires `index.html` rejects v2 bundles. The portfolio's `scripts/verify-jai-bundle.py` must accept v2 before its pointer is bumped to a compiler commit that has this layout.

`.github/workflows/browser-release.yml` builds one reviewed full SHA on manual dispatch, runs the helper tests and the Wasm/LSP crate tests, packages the bundle and uploads the two producer assets as a workflow artifact. It has read-only permissions and publishes nothing. The portfolio's `jai-web.yml` checks out a compiler commit, runs the same packager, verifies the bundle, publishes immutable `jai-web-<sha>` releases in `matteopolak/portfolio` and pins their hashes in `jai-web-release.json`.

## How to change it

- **Bundle contents:** keep `BUNDLED_GLUE` and the bundled examples (`build_scripting_wasm.py`, `tests/examples.json`), `REQUIRED` and `MAX_FILES` (`package_browser_release.py`), `BUNDLE_FILES`/`BUNDLE_EXAMPLES` (`check_browser_release.mjs`), the schema's `files` bounds (7 to 64) and the tests in step. Adding the tour needed no portfolio verifier change (it already accepted nested `.jai`/`.md`/`.json` paths); the portfolio's starter falls back to its built-in files for releases without `tour.json`. Coordinate any change with the portfolio's verifier and sync script.
- **Archive policy:** `package_browser_release.py`. Keep exhaustive inventories, bounded paths and bytes, refusal of dirty or changing sources, and atomic output.
- **Probes:** `check_browser_release.mjs`. Probes must run the staged files, not the source tree.
- Tests: the Python tests (`test_package_browser_release.py`, `test_build_scripting_wasm.py`) use inert fixtures and mock the build and Node calls. They prove archive, staging and refusal behavior, not compilation. `test_browser_release.mjs` proves the inventory rules and that a header-only module fails real execution.

## Configuration

```sh
python3 tools/package_browser_release.py --toolchain          # validate and print the pinned nightly
CARGO_TARGET_DIR=/path/to/target \
  python3 tools/package_browser_release.py --output artifacts/browser-release

python3 -m unittest discover -s tools -p 'test_*.py'
node --test tools/test_browser_release.mjs
```

`--output` must name a new or empty directory; inside the checkout it must be gitignored. `--target-dir` overrides the environment and Cargo configuration through the [shared helper](../tools/build-storage.md). The workflow's `compiler_revision` must be a lowercase full commit SHA.

To run the probe on a local, uncommitted build: stage it with `build_scripting_wasm.py`, replace its `build-metadata.json` with the relocatable form above (any 40-hex commit), then run `node tools/check_browser_release.mjs <dir>`.

## Dependencies

Python 3.11+ (`tomllib`, `zipfile`, `json`), independently installed `git` and Node (tool paths inside protected trees are refused), pinned Rustup/Cargo with `wasm32-unknown-unknown`, and the dependency-age, source-input and build-path helpers in `tools/`. There are no npm dependencies. The build needs no LLVM.
