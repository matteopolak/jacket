# Release smoke tests

`.github/workflows/release-smoke.yml` checks that a published release installs and works from every channel on every OS. It runs when `release.yml` finishes for a tag push (a `workflow_run` trigger) and by hand (Actions, "release smoke", with a `version` tag; empty means the latest).

## How it works

`tools/ci/release_smoke.py` holds the shared pieces. Every install job ends with the same checks on the installed `bin` folder:

- `version`: `jacket version` lists jacket and each tool it fronts, all at the release's version.
- `bindings`: `B.generate_bindings` through `jaic -interp` on a C header, and on a C++ class with and without `-x c`.
- `project`: `jacket new smoke`, `jacket build`, `jacket run` (prints `Hello from smoke!`), then `jailsp` is initialized, given the project's `main.jai` and asked for a hover on `main`.
- `backend`: `-check` and `-interp` never load the backend library ([lazy backend](../compiler/lazy-backend.md)) and work with `JAIC_BACKEND` pointing at nothing; a build with that setting fails naming the library; a normal build loads it and runs.

| Job | Runners | Installs from |
| --- | --- | --- |
| `archive` | ubuntu x64/arm64, macOS arm64/Intel, Windows x64/arm64 | the platform's `jacket-<platform>` archive, verified against `SHA256SUMS` |
| `install-sh` | the Unix runners | the published `install.sh` |
| `install-ps1` | Windows x64/arm64 | the published `install.ps1` under Windows PowerShell 5.1 |
| `homebrew` | ubuntu, macOS arm64 | `brew install matteopolak/tap/jacket` and `brew test` (latest release only) |
| `vscode` | ubuntu | the release's `.vsix` (checksum, version, bundled files) and Open VSX serving the same version |
| `neovim` | ubuntu | the newest tag of the `jacket.nvim` mirror, running the plugin's own `jai.install` in a pinned headless Neovim |

`release.yml` runs the `project` check on each archive before upload, fails a package whose `native-libs/<platform>` lacks a library the stdlib binds (`tools/ci/check_native_libs.py`), and builds small programs against the shipped libraries. A new native module should get one ([releases](releases.md)). The PowerShell installer's own tests (`tools/ci/test_install_ps1.ps1`) run in `ci.yml`'s `install-ps1` job.

## How to change it

- New channel: add a job that puts the tools in a folder and runs `version` and `project`.
- New platform: add its runner to the matrices; archive names come from `platform_name()` in `release_smoke.py` ([releases](releases.md#install-channels)).
- The project check: `project()` and `hover()` in `release_smoke.py`, using the small JSON-RPC client in `tools/toolkit/lsp.py`.
- By hand (needs `gh` and `GH_TOKEN`): `PYTHONPATH=tools python3 tools/ci/release_smoke.py fetch --tag v0.20.0 --dest /tmp/smoke`, then `project --bin <printed folder>`.

## Configuration

`NVIM_VERSION` and `NVIM_SHA256` pin Neovim (kept in step with `ci.yml` by `tools/ci/pins.py`). Everything else comes from the release.
