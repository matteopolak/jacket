# Releases and installation

## What it is

`.github/workflows/release.yml` builds `jaic`, `jailsp`, `jailint` and `jaifmt` for macOS (arm64, Intel), Linux (x86-64, arm64) and Windows (x86-64, arm64), packages six archives (`jacket-<platform>.tar.gz`, `.zip` on Windows, unpacking to `jacket-<platform>/`), smoke-tests each, and publishes a GitHub release from a pushed `v*` tag (or a manual run for the tag matching the workflow's Cargo version). Release notes are the tag's section of `CHANGELOG.md`. Install channels (Homebrew, `install.sh`, `install.ps1`, Nix, the GitHub Action) all install these archives; versions and checksums come from the release itself, never from hand-maintained files.

## How it works

Each platform job:

1. Downloads the official LLVM 23 release (static libraries) and sets `LLVM_SYS_231_PREFIX`. Intel macOS has no official build, so it uses conda-forge's `llvmdev` via `tools/ci/install_ci_llvm_macos.sh --static` ([LLVM setup](llvm-setup.md)). Homebrew's and apt's LLVM link Z3/zstd dynamically, so binaries built against them would need those installed.
2. Builds `jaic-cli --no-default-features --features static-llvm`, `jailsp` and `jailint` through `tools/dev/build_pgo.py --llvm static` (below), strips symbols ([LLVM setup](llvm-setup.md#registered-backends-and-binary-size)).
3. Builds `jaifmt` with the packaged `jaic` (`-O2 -no_debug_info`) and packages the four tools, `stdlib/`, `native-libs/<platform>/` (built by `tools/native/build_native_libs.py --out` with `MACOSX_DEPLOYMENT_TARGET` 11.0 arm64 / 10.12 Intel, plus wgpu-native; see [native libraries](native-libs.md)), checked against what the stdlib binds by `tools/ci/check_native_libs.py`, `README.md` and `CHANGELOG.md`. `tools/ci/package_bindings_helpers.py` adds what `generate_bindings` needs (below).
4. Windows only: builds the crash runtime the standard library binds (`crash-native/platform_io_windows_crash`: a DLL for the interpreter and metaprograms, plus a static `.lib` that built programs link, so they need no DLL) into the packaged `stdlib/` with `tools/stdlib/build_platform_io_runtime.py --static-library`. The repository holds only its C source.
5. Smoke-tests the package from outside the checkout, including through a symlinked install with the checkout's `stdlib/` moved away: `-interp` and a plain build of `examples/compile-time-record.jai` exit 42, `jailint -D warnings` is clean, `jaifmt --stdin` formats, `stdlib/Simp/tests/image.jai` and the FreeType, ImGui and MojoShader runtime tests pass (SDL2 on Windows x64), the WebGPU triangle builds, `Sound_Player`'s device test runs (and, outside Windows, an `rpmalloc` allocation; on Linux a `Clipboard` program is linked), and no Homebrew or shared-LLVM dependency is linked. Then `tools/ci/release_smoke.py project` runs on the package itself (`jacket new`, `build`, `run`, a language-server hover) and `release_smoke.py bindings` runs `generate_bindings` on a C header and on issue 26's C++ class, so an archive that lacks a tool or a native library fails the build job before anything is published ([release smoke tests](release-smoke.md)).

The `vscode` job packages the [VS Code extension](vscode-extension.md); `publish` writes `toolchain-manifest.json` and `SHA256SUMS`, creates or updates the release (replacing assets only when the release is mutable; per-asset replacement is not atomic, so a failed upload needs a rerun), and the `editor-store` (Marketplace and Open VSX, one matrix job), Neovim mirror and Homebrew jobs follow. `release-meta` computes the version, tag, whether the tag is the current release, the dry-run and store-publishing gates and the editor version once, and every job's `if:` and environment read those outputs (a new publishing rule belongs there). Historical releases are not rebuilt.

### Bindings_Generator

`B.generate_bindings` needs three things at run time, none of which a checkout's developer build records in the archive:

| Needs | Where it is found | Shipped as |
| --- | --- | --- |
| `generator_platform`, the helper behind library loading, symbol lookup and `getenv` (`stdlib/internal/developer-tools/generator-platform-native.c`) | `native-libs/<platform>/` like every native library | `libgenerator_platform.{dylib,so,dll}` for `jaic -interp` and `libgenerator_platform.a` (`generator_platform.lib` on Windows) for built programs, a few KiB, every platform |
| libclang, bound as `../internal/developer-tools/clang-bindings/clang` | beside the stdlib, else `native-libs/<platform>/`, else (macOS) the Xcode toolchain | Linux and Windows: `libclang.so` plus the LLVM libraries it names (`$ORIGIN` run path) or `libclang.dll` from the pinned LLVM. macOS: nothing, jaic searches the selected Xcode or Command Line Tools `usr/lib` (`apple_toolchain_lib_dir`, only when it holds `libclang.dylib`; `DEVELOPER_DIR` is honoured) |
| A C compiler for the default include directories, and `llvm-nm` for static-library inputs | `clang-bindings/compiler-path.txt` and `generator-platform-nm.txt` (written by a checkout's build only), else `clang`/`cc` and `llvm-nm` on `PATH`, else `xcrun --find` on macOS | the system's |

Why macOS ships no libclang: Homebrew's `libclang.dylib` is 41 MB plus `libLLVM` at 161 MB, about 67 MB compressed, which would double the 68 MB macOS arm64 archive for a rarely used tool, while a Mac that runs `generate_bindings` already needs the Xcode toolchain (`xcrun --show-sdk-path` supplies the libc headers). Xcode's libclang parses with the same C API. Linux and Windows have no toolchain every user has, so they carry the pinned one; the job log prints each helper's size.

The failures when one is missing now say what and where: a foreign call into a missing library names the library and every directory searched, the link error lists them, and the include and `llvm-nm` lookups say what they tried. A checkout still builds the developer copies with `tools/native/ci_native_deps.sh developer`.

A packaged binary finds its stdlib through `jaic::stdlib_dir`: `JAIC_STDLIB`, else `stdlib/` beside the executable (checked both at the started path and behind symlinks), else the build-time repository path. If none has `prelude/module.jai`, `jaic` and `jailint` stop with an error naming the directory. `jailsp` is the exception for a file inside a folder that holds `prelude/module.jai` (`jaic::enclosing_stdlib`, for example the checkout's own `stdlib/`): that file is analysed against that library, so its internal imports are allowed.

### PGO and BOLT

Our Rust code is built with profile-guided optimisation by `tools/dev/build_pgo.py`: an instrumented build, a training run (the sweep over corpus, negative, stdlib, modules and examples, native sweeps at `-O0`/`-O2`, the short interpreter programs at the top of `benchmarks/` (`benchmarks/*.jai`; longer benchmarks go in subfolders so training stays quick), a `jaifmt` build, `jailint` over the stdlib, and a scripted `jailsp` session in `lsp_session()`), `llvm-profdata` merge, and an optimised rebuild. Only our code is profiled, not LLVM. This gains 6 to 9% on front-end work and 2 to 5% on the interpreter; `-O2` builds do not change. On Linux x86-64, BOLT then reorders code (`--bolt`, from the `llvm-bolt` in the LLVM tarball, instrumentation mode because runners lack `perf` LBR).

- Linux arm64 gets PGO only: Ubuntu's linker adds Cortex-A53 erratum 843419 veneers that `llvm-bolt` refuses, and dropping them would be unsafe on A53 cores. macOS and Windows get PGO only (BOLT rewrites ELF only). Windows arm64 gets a plain release build because the profile runtime crashes there (`pgo_works()`; recheck after toolchain bumps).
- Raw profiles must be merged by the `llvm-profdata` of rustc's LLVM (rustup `llvm-tools`), not the LLVM `jaic` links. The `cc` crate copies profile flags into clang-compiled C code, so the script appends `-fno-profile-generate -fno-profile-use` to `CFLAGS_<triple>`. `RUSTFLAGS` overrides every `CARGO_TARGET_<TRIPLE>_RUSTFLAGS`, so the script extends whichever is in effect; paths with spaces are refused.
- Change the training in `train()` and `LSP_FILES`/`lsp_session()` (keep it inside the repository; heavier training slows the release, twice on Linux). Locally: `python3 tools/dev/build_pgo.py --jobs 4` then run the sweep and `jaic_diff.py` with the result; compare against a plain release build as in [benchmarks](benchmarks.md#benchpy-and-profiling). Options: `--llvm dynamic|static|none`, `--bolt`, `--llvm-bin`, `--jobs`, `--target`, `--target-dir`, `--out`, `--profile` (reuse a profile), `--step-timeout`, `--strict`. Tests: `tools/dev/test_build_pgo.py`.

## Install channels

| Channel | Platforms | Command |
| --- | --- | --- |
| Homebrew tap | macOS, Linux | `brew install matteopolak/tap/jacket` |
| `install.sh` | macOS, Linux | `curl -fsSL https://raw.githubusercontent.com/matteopolak/jacket/main/install.sh \| sh` |
| `install.ps1` | Windows x64, arm64 | `irm https://raw.githubusercontent.com/matteopolak/jacket/main/install.ps1 \| iex` |
| Nix flake | Linux, macOS (from source) | `nix profile install github:matteopolak/jacket` |
| GitHub Action | CI runners | `uses: matteopolak/setup-jacket@v1` (own repository; resolves the platform, caches, verifies `SHA256SUMS`, adds tools to `PATH`, sets `JAIC_STDLIB`) |
| Archive / source | all | unpack and put `jaic` on `PATH`; or `cargo build -p jaic-cli -p jai-language-server -p jailint --release --locked` with LLVM 23 (`just build --release` also builds `target/jaifmt`) |

- **Archive names** are written once in each consumer: `install.sh`, `install.ps1`, `RELEASE_ASSETS` in the extension's `src/toolchain.ts`, the Neovim `install.asset_for`, `PLATFORM_ASSETS` in `tools/dev/render_packages.py`, the formula template and setup-jacket's `resolve.sh`. Renaming means changing all together.
- **Homebrew.** `tools/dev/render_packages.py --version v0.4.0 --sums SHA256SUMS --out out` fills `packaging/homebrew/jacket.rb.tmpl` (`@VERSION@`, `@BASE_URL@`, `@SHA256_<PLATFORM>@`; `--archives DIR` hashes local archives for dry runs). `NEWER_PLATFORMS` (`macos-x64`, `linux-arm64`) are required only after 0.4.1; older releases fall back to the nearest build plus a `depends_on arch:` line. The `homebrew` job (after `publish`; skipped without `HOMEBREW_TAP_TOKEN`) runs `brew style`, `brew audit --strict --online`, install and test from a throwaway tap, then pushes `Formula/jacket.rb` to `matteopolak/homebrew-tap`. The formula installs the tools, `stdlib/` and `native-libs/` into `libexec` and links binaries into `bin`; `jaic` resolves symlinks to find `libexec/stdlib`. It is not bottled. A new placeholder needs a value in `substitutions()`. Tests: `tools/dev/test_render_packages.py`. Never re-upload an archive: that changes its hash and breaks that version's formula.
- **install.sh** (POSIX sh, shellcheck-clean, one function called last so truncated downloads run nothing): maps `uname` to an archive (Rosetta-aware), finds the latest version from the `releases/latest` redirect, verifies against `SHA256SUMS`, unpacks to `~/.local/share/jai/<version>`, symlinks the tools into `~/.local/bin`, and removes older versions it installed. It never overwrites a bin entry that is not a symlink. `JAI_VERSION`, `JAI_INSTALL_DIR`, `JAI_BIN_DIR` (also read as `JAIC_*`) configure it; `JAI_BASE_URL` replaces the release download URL (`<base>/v<version>/<asset>`, `file://` allowed, needs `JAI_VERSION`) in both scripts. **install.ps1** works in Windows PowerShell 5.1 and 7, installs under `%LOCALAPPDATA%\Programs\jai\<version>` with a `current` directory junction (no admin rights) added to the user `PATH`.
- **Crash runtime:** on macOS and Linux the private crash runtime is compiled by `crates/jaic-cli/build.rs` and embedded in `jaic`, so archives need no extra file or release step for it ([crash runtime build](../native/crash-runtime-build.md)).
- No winget: unsigned binaries failed Microsoft Defender validation on every submission. Do not re-add without Windows code signing.
- **CI:** the `install-sh`, `install-ps1` and `homebrew` jobs in `ci.yml` run when `install.sh`, `install.ps1`, `packaging/` or `tools/dev/render_packages.py` change, against the latest published release ([development](development.md#continuous-integration)). Each job first runs `gh release view`: with no published release it emits a `::notice::` and a job-summary line and skips the steps that install a release (any other error fails). Release-independent steps always run, including installing stub archives with a real `SHA256SUMS` through `JAI_BASE_URL=file://...` (install, upgrade, damaged archive, occupied bin entry). They publish nothing.

### Nix

`flake.nix` packages `jaic` (with `jailsp` and `jailint`, `nix/jaic.nix`) and `jaifmt` (`nix/jaifmt.nix`, compiled by the `jaic` package), with `apps`, an `overlays.default` (`pkgs.jaic`, `pkgs.jaifmt`) and a dev shell (pinned Rust, LLVM 23, Python, Node), for x86_64/aarch64 Linux and macOS (Intel macOS evaluates but CI does not build it).

```sh
nix run github:matteopolak/jacket -- hello.jai -interp
nix run github:matteopolak/jacket#jaifmt -- --check src
nix develop     # then cargo build -p jaic-cli --locked
```

- `buildRustPackage` over the workspace with `cargoLock.lockFile`; `inkwell` is a git dependency fetched through `allowBuiltinFetchGit` (remove once it is back on crates.io). Rust comes from `rust-toolchain.toml` via oxalica's `rust-overlay`; `.cargo/config.toml` is left out of the build source. LLVM is nixpkgs' `llvmPackages_23` (`LLVM_SYS_231_PREFIX` points at `llvm.dev`). The flake follows `nixpkgs-unstable` because the 26.05 branch has no cached LLVM 23 for macOS.
- Layout matches the archives: real binaries in `libexec/jaic/` beside `stdlib/`, with `makeWrapper` scripts in `bin/` that set `JAI_LIBCLANG` if unset and append the stdenv C compiler, binutils and (macOS) `dsymutil` to `PATH` (a user's own `cc` wins). The install check runs `examples/compile-time-record.jai` (exit 42), a hello world interpreted and built, and `jailint`.
- **Change it:** bump inputs with `nix flake update` (the `nix` job fails if `flake.lock` is stale); a new nightly works if the locked `rust-overlay` has its manifest, else `nix flake update rust-overlay`; a new LLVM major changes `llvmPackages_NN` and the prefix name in both nix files; anything `jaic` reads at run time must be in the `fileset` in `nix/jaic.nix` and copied in `postInstall`.
- Gotchas: programs built by the Nix `jaic` use the store's glibc and loader, so they are not portable to non-Nix systems (use release archives); the bindings generator needs system headers supplied by the shell (`nix shell nixpkgs#glibc.dev`). Env: `JAIC_STDLIB`, `JAI_LIBCLANG`, `JAIC_LINKER`, `JAIC_AR`. The `nix` job in `ci.yml` runs `nix flake check` and `nix build` when the flake, `nix/`, `Cargo.lock` or `rust-toolchain.toml` change.

## How to change it

**Cut a release:**

1. Add `## [x.y.z] - YYYY-MM-DD` at the top of `CHANGELOG.md`. It is the release page, so keep it short and written for users: one or two sentences, optionally one image, `### Highlights` (2 to 6 one-line bullets), `### Breaking changes` (a Was/Now table, only if any), `### Fixes` (one line each). Leave out internal tooling and flag lists; link to docs with absolute `https://github.com/matteopolak/jacket/blob/main/docs/...` URLs. Use `###` or lower, since a `## [` line ends the extracted notes. Images are small self-contained SVGs under `docs/images/releases/<version>-<topic>.svg`, referenced by absolute `raw.githubusercontent.com` URL and committed before tagging.
2. Run `just bump-version X.Y.Z` (`python3 tools/bump_version.py X.Y.Z`). It sets `[workspace.package] version` in the root `Cargo.toml`, the workspace packages in `Cargo.lock` and `fuzz/Cargo.lock`, `editors/nvim/lua/jai/version.lua` and `jaifmt/version.jai`, adds a `## [X.Y.Z] - <today>` header to `CHANGELOG.md` if there is none, then checks both lock files with `cargo metadata --locked --offline` (`--no-verify` skips that). Leave `editors/vscode/package.json` alone: the release workflow stamps its version. `tools/bump_version.py --check`, run by the tools tests, fails when `version.lua`, `jaifmt/version.jai` or a lock file disagrees with `Cargo.toml`. Commit, then `git tag -s vX.Y.Z -m 'Jacket X.Y.Z'` and `git push origin vX.Y.Z`.

**Gotchas:**

- PGO roughly doubles the build, so the job timeout is 90 minutes. A link failure triggers a verbose relink step to show the linker command.
- Static LLVM needs its system libraries (`llvm-config --link-static --system-libs`, in the job log). Linux installs zlib, zstd, libxml2. macOS arm64 installs Homebrew zstd and deletes its `libzstd*.dylib` links so `ld` picks the archive; the official macOS libraries are LTO bitcode that Xcode's `ld` cannot parse, so the job links with the release's `clang -fuse-ld=lld` against the Xcode SDK and deletes the release's `libc++` so the system one is used. Intel macOS deletes every dylib in the conda environment's `lib/` so conda's `libiconv` is not linked, links its static `libc++.a` and adds `-lc++abi`.
- Windows builds for an explicit `rust_target` with `+crt-static`, because the LLVM archive uses the static C runtime (mixing crashes `jaic` with `0xC0000005`); `tools/ci/windows-llvm/prepare.sh` fixes the archive ([LLVM setup](llvm-setup.md)). It is also built once with LLVM off `PATH` so the link goes through Visual Studio's `link.exe`.
- A new platform needs a matrix row with its LLVM tarball name (and `rust_target` on Windows) and updates to every archive-name consumer above.
- A tag push publishes everything: the GitHub release, the VS Code Marketplace, Open VSX, the Homebrew tap and the Neovim mirror. A manual run publishes to the stores only with `publish_integrations`.
- The editor can be released independently of the compiler with `editor_version`; stores keep versions immutable, so a repin needs a new editor version.

## Configuration

`LLVM_VERSION` in the workflow (LLVM 23 patch release); `jaic-cli` features `dynamic-llvm` (default), `static-llvm`, `llvm`; `jaic-llvm` features `dynamic`, `static`. Secrets: `HOMEBREW_TAP_TOKEN` (fine-grained token, `matteopolak/homebrew-tap` only, Contents read/write), `VSCE_PAT`, `OVSX_PAT`, `NVIM_MIRROR_DEPLOY_KEY`. The `matteopolak/homebrew-tap` repository already exists.

## Dependencies

GitHub-hosted runners (`macos-15`, `macos-15-intel`, `ubuntu-24.04`, `ubuntu-24.04-arm`, `windows-2025`, `windows-11-arm`), official LLVM release assets, conda-forge (Intel macOS), rustup's `llvm-tools`, Homebrew on the runners, and for installers `curl`/`wget`, `tar`, `sha256sum`/`shasum` on users' machines.

## Toolchain manifest

`publish` writes `toolchain-manifest.json` next to `SHA256SUMS` with `tools/ci/toolchain_manifest.py`: `{"version": "0.20.0", "sha256": {"jacket-linux-x64.tar.gz": "<hex>", ...}}`, covering the `jacket-<platform>` archives only. The VS Code extension (`vscode` job, which generates the same file from the build's archives) and the Neovim plugin (`neovim` job, which downloads the release's) ship it and refuse any archive that does not match; a development build with no shipped manifest fetches the release's. To change the format, edit the script, `parseManifest` in `editors/vscode/src/toolchain.ts` and `parse_manifest` in `editors/nvim/lua/jai/install.lua`; all three tests read the fixtures in `editors/fixtures/` (`SHA256SUMS` and the manifest generated from it), so they cannot drift. The `publish` job also takes the release notes from `tools/ci/release_notes.py`.

## Installer test vectors

`tests/installers/vectors.json` holds the cases every installer must agree on: which archive each platform gets (or that it has none) and which `SHA256SUMS` line formats are accepted (two spaces, the `*` binary marker, upper-case hex) or refused. It is read by `tools/ci/test_install_sh.py` (install.sh, with a stub `uname`), `tools/ci/test_install_ps1.ps1` (install.ps1; runs in the `install-ps1` job), `editors/vscode/test/unit/toolchain.test.ts` and `editors/nvim/tests/install_spec.lua`. A new platform or checksum format goes into the file first, then into whichever installer fails.
