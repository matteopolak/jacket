# Releases

## What it is

`.github/workflows/release.yml` builds `jaic` and `jai-lsp` for macOS (Apple silicon), Linux (x86-64) and Windows (x86-64), smoke-tests each archive, and publishes a GitHub release when a `v*` tag is pushed. Release notes come from the tag's section of `CHANGELOG.md`.

## How it works

Each platform job:

1. Downloads the official LLVM 22 release for the platform (`LLVM-<version>-macOS-ARM64.tar.xz`, `LLVM-<version>-Linux-X64.tar.xz`, `clang+llvm-<version>-x86_64-pc-windows-msvc.tar.xz`) and points `LLVM_SYS_221_PREFIX` at it. Those tarballs carry LLVM's static libraries. Homebrew's and apt's LLVM link Z3 and zstd as shared libraries, so a binary built against them would only run where those are installed.
2. Builds `jaic-cli --no-default-features --features static-llvm`, which links LLVM statically, on every platform. A matrix row with an empty `llvm` would build without the backend (`--no-default-features`).
3. Packages `jaic`, `jai-lsp`, `stdlib/`, `README.md` and `CHANGELOG.md` as `jaic-<platform>.tar.gz` (`.zip` on Windows).
4. Smoke-tests the packaged `jaic` from a directory outside the checkout: `run` (and `build`, where LLVM is linked) of `examples/compile-time-record.jai` must exit with 42. It also checks that the macOS binary links nothing from Homebrew and the Linux one no shared LLVM.

The `publish` job then collects the archives, writes `SHA256SUMS`, extracts the `## [x.y.z]` section of `CHANGELOG.md` and runs `gh release create`.

A packaged `jaic` finds its standard library through `jaic::stdlib_dir`: `JAIC_STDLIB`, else `stdlib/` next to the executable (when it has `module.jai`), else the repository's `stdlib/` (development builds).

## How to change it

To cut a release:

1. Add a `## [x.y.z] - YYYY-MM-DD` section at the top of `CHANGELOG.md`. Write it by hand, for users: what changed, not a commit list.
2. Bump `version` in the root `Cargo.toml`.
3. Commit, then tag and push:

```sh
git tag v0.2.0
git push origin v0.2.0
```

To test the build without publishing, run the workflow by hand (Actions → release → Run workflow) with an empty tag. The archives are kept as workflow artifacts. To publish an existing tag after a fix to the workflow, run it with that tag.

Gotchas:

- Linking static LLVM also needs LLVM's system libraries (`llvm-config --link-static --system-libs`, printed in the job log). On Linux the job installs zlib, zstd and libxml2; on macOS the official LLVM names Homebrew's `/opt/homebrew/lib/libzstd.a`, so the job installs Homebrew's zstd (the archive is linked in, so the binary does not need Homebrew). The macOS release's static libraries are LTO bitcode, which Xcode's `ld` cannot parse (`could not parse bitcode object file ... Unknown attribute`), so the macOS job links with the release's own `clang -fuse-ld=lld` against the Xcode SDK (`CARGO_TARGET_AARCH64_APPLE_DARWIN_LINKER`, `..._RUSTFLAGS`, `SDKROOT`). The release's own libc++ (`libc++.a`, and `libc++.1.dylib` behind `@rpath`) is deleted first, so `-lc++` links the system `/usr/lib/libc++.1.dylib` through the SDK.
- Windows: the LLVM archive is built with the static C runtime (`/MT`), so `jaic` is built for an explicit `x86_64-pc-windows-msvc` target with `+crt-static` (`CARGO_BUILD_TARGET`, `CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS`; binaries land in `target/x86_64-pc-windows-msvc/release`). Mixing runtimes links but `jaic build` crashes at once with `0xC0000005`. `llvm-config` also names a static libxml2 (`xml2s.lib`) that the archive does not ship; the job links an empty one. The archive's `bin/` goes on `PATH` so the smoke test's `jaic build` finds `clang` as its linker driver. See [Windows](../native/windows.md).
- Adding a platform means adding a matrix row with its LLVM tarball name. Check the LLVM release page for the exact asset name; it changes between major versions.

## Configuration

- `LLVM_VERSION` in the workflow: the LLVM 22 patch release to download.
- `jaic-cli` features: `dynamic-llvm` (default), `static-llvm`, `llvm` (backend without a link preference).
- `jaic-llvm` features: `dynamic` (default), `static`.

## Dependencies

GitHub-hosted runners (`macos-15`, `ubuntu-24.04`, `windows-2025`), the official LLVM release assets on github.com/llvm/llvm-project, and the pinned `checkout`, `setup-python`, `upload-artifact` and `download-artifact` actions.
