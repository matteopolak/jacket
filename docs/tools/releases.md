# Releases

## What it is

`.github/workflows/release.yml` builds `jaic`, `jailsp`, `jailint` and `jaifmt` for macOS (Apple silicon), Linux (x86-64) and Windows (x86-64 and arm64), smoke-tests each archive, and publishes a GitHub release when a `v*` tag is pushed. Release notes come from the tag's section of `CHANGELOG.md`.

## How it works

Each platform job:

1. Downloads the official LLVM 23 release for the platform (`LLVM-<version>-macOS-ARM64.tar.xz`, `LLVM-<version>-Linux-X64.tar.xz`, `clang+llvm-<version>-x86_64-pc-windows-msvc.tar.xz`, `clang+llvm-<version>-aarch64-pc-windows-msvc.tar.xz`) and points `LLVM_SYS_231_PREFIX` at it. Those tarballs carry LLVM's static libraries. Homebrew's and apt's LLVM link Z3 and zstd as shared libraries, so a binary built against them would only run where those are installed.
2. Builds `jaic-cli --no-default-features --features static-llvm`, which links LLVM statically, on every platform, together with `jailsp` and `jailint`, through `tools/build_pgo.py --llvm static`: an instrumented build, a training run over the repository's tests, examples and benchmarks, and a profile-guided rebuild (Windows arm64 gets a plain release build; see [PGO and BOLT](pgo-and-bolt.md#gotchas)). On Linux the result is also optimised with BOLT, using the `llvm-bolt` in the LLVM tarball. See [PGO and BOLT](pgo-and-bolt.md). The binaries land in `target/pgo/dist`. A matrix row with an empty `llvm` would build without the backend (`--llvm none`).
3. Builds `jaifmt` with the packaged `jaic` (`jaic build jaifmt/main.jai -O2 --no-debug-info`: optimised but with the default runtime checks, unlike `jaifmt/build.jai`, and without a `.pdb`/`.dSYM` beside it), then packages `jaic`, `jailsp`, `jailint`, `jaifmt`, `stdlib/`, `README.md` and `CHANGELOG.md` as `jaic-<platform>.tar.gz` (`.zip` on Windows).
4. Smoke-tests the packaged `jaic` from a directory outside the checkout: `run` (and `build`, where LLVM is linked) of `examples/compile-time-record.jai` must exit with 42, and the packaged `jailint -D warnings` must find nothing in it (which also checks it finds the packaged stdlib), and `echo 'main::(){x:=1;}' | jaifmt --stdin` must print the formatted program. It also checks that the macOS binary links nothing from Homebrew and the Linux one no shared LLVM (with `if ...; then exit 1`, since `set -e` ignores a failing `! cmd`).

The `vscode` job then packages the [VS Code extension](vscode-extension.md) as `jai-vscode-<version>.vsix`, with its version set from the tag (which must equal `Cargo.toml`'s) and the SHA-256 of the archives just built pinned inside, so it downloads exactly those.

The `publish` job then collects the archives and the `.vsix`, writes `SHA256SUMS`, extracts the `## [x.y.z]` section of `CHANGELOG.md` and runs `gh release create`. After it, the `marketplace` and `open-vsx` jobs publish the `.vsix` to the VS Code Marketplace and Open VSX when the `VSCE_PAT` and `OVSX_PAT` secrets are set (otherwise they say they skipped; see [publishing](vscode-extension.md#publishing)). They are separate jobs, so a store failing leaves the GitHub release and the other store alone. The `homebrew` and `winget` jobs, also after `publish`, render the Homebrew formula and the winget manifests from the tag and the release's `SHA256SUMS`, validate them, and push the formula to `matteopolak/homebrew-tap` and open a winget-pkgs pull request when `HOMEBREW_TAP_TOKEN` and `WINGET_TOKEN` are set; see [package managers](package-managers.md).

A packaged `jaic` (and `jailint`, `jailsp`) finds its standard library through `jaic::stdlib_dir`: `JAIC_STDLIB`, else `stdlib/` next to the executable (when it has `module.jai`), checked both beside the path it was started from and beside the real file behind any symlinks, else the repository's `stdlib/` (development builds; the path is baked in at build time, so on another machine it is the CI runner's). Symlinks matter on macOS, where the executable's path is the link's: unpacking into `/opt/jaic` and linking `~/bin/jaic` to it must still find `/opt/jaic/stdlib`. When none of these has `module.jai`, `jaic` and `jailint` stop with an error naming the directory and both fixes (`jaic::missing_stdlib`); `jailsp` logs it. The smoke test also runs the package through a symlink with the checkout's `stdlib/` moved away.

## How to change it

To cut a release:

1. Add a `## [x.y.z] - YYYY-MM-DD` section at the top of `CHANGELOG.md`. Write it by hand, for users: what changed, not a commit list.
2. Bump `version` in the root `Cargo.toml`.
3. Commit, then tag and push:

```sh
git tag -s v0.4.0 -m 'jaic 0.4.0'
git push origin v0.4.0
```

To test the build without publishing, run the workflow by hand (Actions → release → Run workflow) with an empty tag. The archives are kept as workflow artifacts. To publish an existing tag after a fix to the workflow, run it with that tag.

Gotchas:

- PGO roughly doubles the build (two release builds plus training; Linux trains twice for BOLT), so the job's timeout is 90 minutes. If the build step fails on a link error, a follow-up step relinks `jaic` with `-v` to show the linker command.
- Linking static LLVM also needs LLVM's system libraries (`llvm-config --link-static --system-libs`, printed in the job log). On Linux the job installs zlib, zstd and libxml2; on macOS the official LLVM names Homebrew's `/opt/homebrew/lib/libzstd.a`, so the job installs Homebrew's zstd and deletes the `libzstd*.dylib` links in `/opt/homebrew/lib`: llvm-sys passes that path on as `-L/opt/homebrew/lib -lzstd`, and `ld` would pick the dylib beside the archive, leaving `jaic` needing Homebrew's zstd at run time (as 0.3.0's did). With only the archive there, it is linked in. 
  The macOS release's static libraries are LTO bitcode, which Xcode's `ld` cannot parse (`could not parse bitcode object file ... Unknown attribute`), so the macOS job links with the release's own `clang -fuse-ld=lld` against the Xcode SDK (`CARGO_TARGET_AARCH64_APPLE_DARWIN_LINKER`, `..._RUSTFLAGS`, `SDKROOT`). The release's own libc++ (`libc++.a`, and `libc++.1.dylib` behind `@rpath`) is deleted first, so `-lc++` links the system `/usr/lib/libc++.1.dylib` through the SDK.
- Windows: the LLVM archive is built with the static C runtime (`/MT`), so `jaic` is built for an explicit target (the matrix row's `rust_target`, `x86_64-pc-windows-msvc` or `aarch64-pc-windows-msvc`) with `+crt-static` (`CARGO_BUILD_TARGET`, `CARGO_TARGET_<TRIPLE>_RUSTFLAGS`; binaries land in `target/<triple>/release`). Mixing runtimes links, but `jaic build` then crashes at once with `0xC0000005`. The arm64 archive is built natively on GitHub's `windows-11-arm` runner, whose image ships rustup and the Visual Studio ARM64 tools. `llvm-config` also names static libraries the archive does not ship (libxml2, zlib, and zstd by an absolute path llvm-sys cannot parse); `tools/windows-llvm/prepare.sh` supplies them ([LLVM setup](llvm-setup.md#windows-msvc-builds)). The archive's `bin/` goes on `PATH` so the smoke test's `jaic build` finds `clang` as its linker driver. See [Windows](../native/windows.md).
- There is no Intel macOS archive: LLVM publishes no macOS X64 build of LLVM 23, and Homebrew has no Intel bottles for it, so a static build would mean compiling LLVM from source in the job. Intel Macs build from source ([LLVM setup](llvm-setup.md)); CI still tests them with conda-forge's LLVM (`tools/install_ci_llvm_macos.sh`).
- Adding a platform means adding a matrix row with its LLVM tarball name (and, on Windows, its `rust_target`). Check the LLVM release page for the exact asset name; it changes between major versions.

## Configuration

- `LLVM_VERSION` in the workflow: the LLVM 23 patch release to download.
- `jaic-cli` features: `dynamic-llvm` (default), `static-llvm`, `llvm` (backend without a link preference).
- `jaic-llvm` features: `dynamic` (default), `static`.

## Dependencies

rustup's `llvm-tools` component (installed with the toolchain, for `llvm-profdata`), GitHub-hosted runners (`macos-15`, `ubuntu-24.04`, `windows-2025`, `windows-11-arm`), the official LLVM release assets on github.com/llvm/llvm-project, and the pinned `checkout`, `setup-python`, `upload-artifact` and `download-artifact` actions.
