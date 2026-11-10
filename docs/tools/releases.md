# Releases and installation

`.github/workflows/release.yml` builds `jaic`, `jailsp`, `jailint`, `jaifmt` and `jacket` for macOS (arm64, Intel), Linux (x86-64, arm64) and Windows (x86-64, arm64), packages six archives (`jacket-<platform>.tar.gz`, `.zip` on Windows), smoke-tests each, and publishes a GitHub release from a pushed `v*` tag. Release notes are the tag's section of `CHANGELOG.md`. Every install channel installs these archives; versions and checksums come from the release, never from hand-kept files.

## Install channels

| Channel | Platforms | Command |
| --- | --- | --- |
| Homebrew tap | macOS, Linux | `brew install matteopolak/tap/jacket` |
| `install.sh` | macOS, Linux | `curl -fsSL https://jacket.matteopolak.com/install.sh \| sh` |
| `install.ps1` | Windows x64, arm64 | `irm https://jacket.matteopolak.com/install.ps1 \| iex` |
| VS Code, Open VSX | any | the `matteopolak.jacket` extension ([VS Code](vscode-extension.md)) |
| Neovim | any | the `matteopolak/jacket.nvim` plugin ([Neovim](neovim-plugin.md)) |
| Nix flake | Linux, macOS (from source) | `nix profile install github:matteopolak/jacket` |
| GitHub Action | CI runners | `uses: matteopolak/setup-jacket@v1` |
| Archive / source | all | unpack and put `jaic` on `PATH`; or `cargo build -p jaic-cli -p jai-language-server -p jailint --release --locked` with LLVM 23 |

- **Archive names** are written once in each consumer: `install.sh`, `install.ps1`, `RELEASE_ASSETS` in the extension's `src/toolchain.ts`, the Neovim `install.asset_for`, `PLATFORM_ASSETS` in `tools/dev/render_packages.py`, the formula template and setup-jacket's `resolve.sh`. Renaming means changing all together.
- **install.sh** maps `uname` to an archive, finds the latest version from the `releases/latest` redirect, verifies `SHA256SUMS`, unpacks to `~/.local/share/jai/<version>`, symlinks the tools into `~/.local/bin` and removes older versions it installed. It never overwrites a bin entry that isn't a symlink. `JAI_VERSION`, `JAI_INSTALL_DIR` and `JAI_BIN_DIR` configure it; `JAI_BASE_URL` replaces the download URL in both scripts (`file://` allowed, needs `JAI_VERSION`).
- **install.ps1** works in Windows PowerShell 5.1 and 7 and installs under `%LOCALAPPDATA%\Programs\jai\<version>` with a `current` junction on the user `PATH` (no admin rights).
- **Homebrew.** `tools/dev/render_packages.py --version v0.4.0 --sums SHA256SUMS --out out` fills `packaging/homebrew/jacket.rb.tmpl`. The `homebrew` job (after `publish`; skipped without `HOMEBREW_TAP_TOKEN`) audits, installs from a throwaway tap, then pushes `Formula/jacket.rb` to `matteopolak/homebrew-tap`. A new placeholder needs a value in `substitutions()`. Never re-upload an archive: that changes its hash and breaks that version's formula.
- **No winget.** Unsigned binaries failed Defender validation on every submission; don't re-add without code signing.
- **CI:** the `install-sh`, `install-ps1` and `homebrew` jobs in `ci.yml` run when the installers or `packaging/` change, against the latest published release (and skip those steps if there is none). They publish nothing.

### Nix

`flake.nix` packages `jaic` (with `jailsp` and `jailint`, `nix/jaic.nix`) and `jaifmt` (`nix/jaifmt.nix`), with `apps`, an overlay and a dev shell.

```sh
nix run github:matteopolak/jacket -- hello.jai -interp
nix develop     # then cargo build -p jaic-cli --locked
```

- The layout matches the archives: real binaries in `libexec/jaic/` beside `stdlib/`, with `makeWrapper` scripts in `bin/`. Rust comes from `rust-toolchain.toml` via `rust-overlay`; LLVM is nixpkgs' `llvmPackages_23`.
- **Change it:** `nix flake update` (the `nix` job fails if `flake.lock` is stale); a new LLVM major changes `llvmPackages_NN` and the prefix name in both nix files; anything `jaic` reads at run time must be in the `fileset` in `nix/jaic.nix` and copied in `postInstall`.
- Programs built by the Nix `jaic` use the store's glibc, so they aren't portable to non-Nix systems.

## How a release is built

Each platform job:

1. Downloads the pinned official LLVM (static libraries). Intel macOS uses conda-forge's `llvmdev` ([LLVM setup](llvm-setup.md)).
2. Builds `jaic-cli` (`lazy-llvm`), the backend library `jaic-backend` (static LLVM inside), `jailsp` and `jailint` through `tools/dev/build_pgo.py --llvm lazy` and strips symbols.
3. Builds `jaifmt` with the packaged `jaic` and packages the tools, the backend library beside `jaic` (`libjaic_backend.dylib`, `.so` or `jaic_backend.dll`; [lazy backend](../compiler/lazy-backend.md)), `stdlib/`, `native-libs/<platform>/` ([native libraries](native-libs.md)), `README.md` and `CHANGELOG.md`.
4. Windows only: builds the crash runtime into the packaged `stdlib/`.
5. Smoke-tests the package from outside the checkout ([release smoke tests](release-smoke.md)), so an archive missing a tool or library fails before anything is published.

`publish` downloads an allowlist of platform archives, the VSIX and compressed docs bundle. It excludes the Pages `artifact.tar` and `pgo-training-*` diagnostic artifacts, writes `toolchain-manifest.json` and `SHA256SUMS`, and creates or updates the release; the store (Marketplace and Open VSX), Neovim mirror and Homebrew jobs follow. `release-meta` computes the version, tag, dry-run and publishing gates once, and every job's `if:` reads those outputs (a new publishing rule belongs there).

A packaged binary finds its stdlib through `jaic::stdlib_dir`: `JAIC_STDLIB`, else `stdlib/` beside the executable (also behind symlinks), else the build-time repository path ([prelude selection](../compiler/prelude-selection.md)).

### Bindings_Generator

`generate_bindings` needs at run time: the `generator_platform` helper (in `native-libs/<platform>/` on every platform), libclang (shipped on Linux and Windows; on macOS `jaic` finds the Xcode toolchain's, since a Mac running it has Xcode and shipping libclang would double the archive), and a C compiler plus `llvm-nm` from `PATH`. Failures name the library and every directory searched. A checkout builds developer copies with `tools/native/ci_native_deps.sh developer`.

### PGO and BOLT

Our Rust code is built with profile-guided optimisation (`tools/dev/build_pgo.py`): an instrumented build, a training run (corpus sweeps, short `benchmarks/*.jai` programs, a `jaifmt` build, `jailint` over a stdlib subset, a scripted `jailsp` session), a merge, and an optimised rebuild. Only our code is profiled. It gains 6 to 9% on front-end work and 2 to 5% on the interpreter. On Linux x86-64, BOLT then reorders code (`--bolt`). Linux arm64, macOS arm64 and Windows x64 get PGO only. Windows arm64 gets a plain build because the profile runtime crashes there (`pgo_works()`), and macOS Intel gets one (`--no-pgo`, `pgo: false` in the matrix) because training on the slow Intel runner pushed the job past its timeout and few people use that build.

- Profiles must be merged by the `llvm-profdata` of rustc's LLVM (rustup `llvm-tools`), not the LLVM `jaic` links.
- Change the training in `train()` and `lsp_session()`; heavier training slows the release.
- Each training pass keeps its logs and a `summary.json` (commands, expected exits, durations, outcomes) in `<target-dir>/pgo/training/pgo/` (or `bolt/`). A new run clears them first, after checking that a supplied `--profile` is a readable file outside that folder. `completed` means the command exited 0, not that training covered everything.
- Training commands run in their own process group (a Job Object on Windows) that is killed on timeout or interruption. Programs that start another session are not covered.
- A failed step prints the last 16 KiB of its output; `--verbose` prints all of it. Any failed step (a non-zero exit or a timeout) stops the build before the optimised rebuild, so a broken workload cannot ship a profile that silently misses code.
- The `jailint` training lints `LINT_TRAINING_PATHS` (generic containers, format and parsing modules, time and compression code), about a tenth of the stdlib: the whole of it took 15 to 25 minutes on the Intel macOS runner and the same linter paths run on every module. Bindings and generated code add volume, not coverage. The second lint step runs the examples and compiler regressions with `--allow-incomplete`, because some regression programs deliberately do not compile fully.
- To check process cleanup on Windows x64 and arm64: `gh workflow run ci.yml --ref my-branch -f jobs=docs -f tier=fast -F pgo_processes=true`. It runs `tools/ci/check_native_python.py` and the focused Python tests, without building the compiler.
- Locally: `PYTHONPATH=tools python3 tools/dev/build_pgo.py --jobs 4`. Options: `--llvm dynamic|static|lazy|none`, `--bolt`, `--target`, `--out`, `--profile`. Tests: `tools/dev/test_build_pgo.py`.

## Cutting a release

1. Add `## [x.y.z] - YYYY-MM-DD` at the top of `CHANGELOG.md`: one or two sentences, `### Highlights` (2 to 6 one-liners), `### Breaking changes` (a Was/Now table, if any), `### Fixes`. Keep it for users; no internals. Use `###` or lower for subsections: the next level-two heading ends the extracted notes. Fenced examples do not define release sections. Inline code on its own line does not start a fence.
2. Alternatively, prepare notes under `## Unreleased` or `## [Unreleased]` (trailing whitespace is allowed; suffixes are not). Run `just bump-version X.Y.Z`; either heading becomes `## [X.Y.Z] - YYYY-MM-DD`. A prewritten section for that exact version is kept, including its date and notes. It edits the workspace version in `Cargo.toml`, the lock files and `CHANGELOG.md`, and regenerates `jaifmt/version.jai` and `editors/nvim/lua/jai/version.lua`. Leave `editors/vscode/package.json` alone: the workflow stamps it.
3. Commit, then `git tag -s vX.Y.Z -m 'Jacket X.Y.Z'` and `git push origin vX.Y.Z`.

A tag push publishes everything: the GitHub release, both extension stores, the Homebrew tap and the Neovim mirror. A manual run publishes to the stores only with `publish_integrations`. The editor can be released alone with `editor_version`; stores keep versions immutable, so a checksum repin needs a new editor version.

A forced tag update validates the release metadata but deliberately skips builds and publishing.
If that tag has not been published, resume it explicitly from its own workflow revision:

```sh
gh workflow run release.yml --ref vX.Y.Z -f tag=vX.Y.Z \
  -f publish_integrations=true -f editor_version=X.Y.Z
```

Confirm the build and publish jobs actually succeeded; a successful metadata-only run does not
establish publication. Run `release-smoke.yml` with `version=vX.Y.Z` after a manual release,
since its automatic trigger accepts tag-push releases only. Do not rebuild an immutable published
release or replace its archives and checksums.

The browser bundle has its own `browser-release.yml` validation. It builds a native compiler and
formatter before running the LLVM-free language-server and Wasm tests: formatting tests execute
the real `jaifmt` through `JAIFMT`. Packaging then uses that compiler to build the browser formatter.
To validate a released compiler with updated workflow setup, dispatch the workflow from the fixed
branch and set `compiler_revision` to the release's full commit SHA; checkout and bundle provenance
remain pinned to that compiler revision.

Gotchas:

- PGO roughly doubles the build, so the job timeout is 90 minutes.
- Static LLVM needs its system libraries (`llvm-config --link-static --system-libs`, in the job log); the per-OS workarounds are in `release.yml` and [LLVM setup](llvm-setup.md).
- Windows builds with `+crt-static` for an explicit `rust_target`, because mixing C runtimes crashes `jaic`.
- A new platform needs a matrix row with its LLVM tarball and updates to every archive-name consumer above.

### The release command

`tools/release.py <step>` holds every release step; `release.yml` calls it per job. `--list` names the steps, `--help` after a step its options. Options default to the workflow's environment variables; secrets are read from the environment only.

| Step | What it does |
| --- | --- |
| `meta` | derives the job outputs from the tag, event and inputs |
| `notes TAG` | prints the `CHANGELOG.md` section of the tag |
| `published [--formula]` | whether a release exists that installers can install |
| `publish` | creates the GitHub release, or edits a mutable one |
| `marketplace`, `open-vsx` | publish the VSIX |
| `homebrew` | pushes the rendered formula to the tap |
| `nvim-stamp`, `nvim-mirror` | stamp the version and manifest into `editors/nvim`, then mirror it to `matteopolak/jacket.nvim` |
| `portfolio` | `gh workflow run jacket-web.yml -R matteopolak/portfolio -f jai_ref=<tag> -f update_pointer=true`; run by the maintainer after the release |
| `all` | the publishing steps in order, for a release run by hand |

The portfolio step dispatches `jacket-web.yml` on the portfolio's default branch with `jai_ref=vX.Y.Z` and `update_pointer=true`. That workflow resolves the tag to a full compiler SHA, validates its bundle, then updates the deployment pointer. For example, inspect the intended command without publishing with `python3 tools/release.py portfolio --tag vX.Y.Z --dry-run` (substitute a numeric tag).

`--dry-run` prints each command (secrets as `***`) and runs nothing. To add a step: a `step_<name>(args, shell)` function and a `step(...)` row in `build_parser`; run every command through `shell.run`; add it to `PUBLISHING` if `all` should run it; `tools/test_release.py` has the fakes.

## Toolchain manifest

The release includes `toolchain-manifest.json` next to `SHA256SUMS` (`tools/ci/toolchain_manifest.py`): `{"version": "0.20.0", "sha256": {"jacket-linux-x64.tar.gz": "<hex>", ...}}`. The VS Code extension and the Neovim plugin ship it and refuse any archive that doesn't match; a development build with none fetches the release's. To change the format, edit the script, `parseManifest` in `editors/vscode/src/toolchain.ts` and `parse_manifest` in `editors/nvim/lua/jai/install.lua`. All three tests read `editors/fixtures/`, so they can't drift.

## Installer test vectors

`tests/installers/vectors.json` holds the cases every installer must agree on: which archive each platform gets and which `SHA256SUMS` line formats are accepted or refused. It is read by `tools/ci/test_install_sh.py`, `tools/ci/test_install_ps1.ps1`, `editors/vscode/test/unit/toolchain.test.ts` and `editors/nvim/tests/install_spec.lua`. A new platform or checksum format goes in the file first, then into whichever installer fails.

## Configuration

The `llvm` pin in `.github/pins.json`; `jaic-cli` features `dynamic-llvm` (default), `static-llvm`, `lazy-llvm`; `jaic-llvm` and `jaic-backend` features `dynamic`, `static`; secrets `HOMEBREW_TAP_TOKEN`, `VSCE_PAT`, `OVSX_PAT`, `NVIM_MIRROR_DEPLOY_KEY`.

## Dependencies

GitHub-hosted runners, official LLVM release assets, conda-forge (Intel macOS), rustup's `llvm-tools`; `curl`/`wget`, `tar` and `sha256sum`/`shasum` on users' machines.
