# Release smoke tests

## What it is

`.github/workflows/release-smoke.yml` checks that a published release can be installed and used from every channel on every operating system. It runs when `release.yml` finishes for a tag push (a `workflow_run` trigger, because a release created with the workflow token starts no other workflow itself) and by hand: Actions, "release smoke", with a `version` tag such as `v0.20.0` (empty: the latest release).

## How it works

`tools/ci/release_smoke.py` has the shared pieces. Every install job ends with the same two checks on the installed `bin` folder:

- `version`: `jacket version` lists jacket and each tool it fronts, and every line carries the release's version.
- `project`: `jacket new smoke`, `jacket build`, `jacket run` (prints `Hello from smoke!`), then `jailsp` is started, initialized, given the project's `main.jai` and asked for a hover on `main`; an empty answer fails.

| Job | Runners | What it installs from |
| --- | --- | --- |
| `archive` | ubuntu x64/arm64, macOS arm64/Intel, Windows x64/arm64 | `fetch`: this platform's `jacket-<platform>` archive from the release, verified against its `SHA256SUMS` line, unpacked |
| `install-sh` | the four Unix runners | the published `install.sh` (`curl ... \| JAI_VERSION=<tag> sh`), tools in `~/.local/bin` |
| `install-ps1` | Windows x64/arm64 | the published `install.ps1` under Windows PowerShell 5.1, tools in `%LOCALAPPDATA%\Programs\jai\current` |
| `homebrew` | ubuntu, macOS arm64 | `brew install matteopolak/tap/jacket` and `brew test`. The tap holds only the latest release; for an older tag the job says so and tests nothing |
| `vscode` | ubuntu | `vsix`: the release's `jacket-vscode-<editor version>.vsix` is verified against `SHA256SUMS`, its `package.json` version matches the file name, it holds the bundle and grammar, and any archive checksums pinned in the bundle are exactly the release's. `vsix-store`: Open VSX serves the same version (retried for five minutes, since it can lag) |
| `neovim` | ubuntu | the newest tag of the published `matteopolak/jacket.nvim` mirror, run in a pinned headless Neovim: `tools/ci/release_smoke.lua` calls the plugin's own `jai.install` with its stamped version and pinned checksums, then the `project` check runs on the result. A plugin stamped for another toolchain version is skipped with a notice |

The PowerShell installer's own tests (`tools/ci/test_install_ps1.ps1`: stub archives, upgrade, a damaged download, and the shared `tests/installers/vectors.json` checksum cases) run in the `install-ps1` job of `ci.yml` under Windows PowerShell 5.1 and PowerShell 7, whenever `install.ps1`, `packaging/` or the installer tests change and on the nightly run.

## How to change it

- A new channel: add a job that puts the tools in a folder and runs `version` and `project` on it.
- A new platform: add its runner to the matrices; archive names come from `platform_name()` in `release_smoke.py`, like every other consumer ([releases](releases.md#install-channels)).
- What the project check does: `project()` and `hover()` in `release_smoke.py`. They use `tools/toolkit/lsp.py`, a small standard-library JSON-RPC client.
- Running a piece by hand needs `gh` and `GH_TOKEN`: `python3 tools/ci/release_smoke.py fetch --tag v0.20.0 --dest /tmp/smoke`, then `project --bin <printed folder>`.

## Configuration

`NVIM_VERSION` and `NVIM_SHA256` pin Neovim (kept in step with `ci.yml` by `tools/ci/pins.py`). Everything else comes from the release: the tag, `SHA256SUMS`, the assets.

## Dependencies

The published release and its assets, `raw.githubusercontent.com` (installers), the Homebrew tap, `jacket.nvim` and Open VSX; `gh` on the runners.
