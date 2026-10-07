# LLVM setup

## What it is

`crates/jaic-llvm` uses Inkwell (`llvm23-1-prefer-dynamic`, from a pinned commit of its main branch) over a separately installed LLVM 23.1, and `jaic build` links with the host's Clang. Building `jaic-cli` needs LLVM; `jaic check` and `jaic run` don't use it at run time. `jai-wasm` doesn't depend on LLVM.

## How it works

`llvm-sys` finds LLVM through `LLVM_SYS_231_PREFIX` (a directory containing `bin/llvm-config` and `lib/`). CI sets it per platform: Homebrew's `llvm@23` on arm64 macOS (`tools/install_ci_llvm_macos.sh`, which runs `brew update` and retries when a runner image's Homebrew predates the `llvm@23` alias); on Intel macOS, where Homebrew has no bottles and would build LLVM from source past the job's timeout, the same script installs conda-forge's LLVM 23.1 packages (`llvmdev`, `libllvm23`, `llvm-tools`, `clang`, `compiler-rt`, `lld`) with a pinned, checksummed micromamba into the runner's temp directory, links the binaries it builds with an rpath to that `lib/` (`RUSTFLAGS`, since conda's libraries load through `@rpath`), sets `SDKROOT`, and puts only LLVM's own tools on `PATH` so conda's `cc`, `ld` and other cctools do not replace the system's (with `--static`, for the release's `macos-x64` archive, it installs only the static libraries and links nothing from the environment at run time; see [releases](releases.md#how-to-change-it)); and `tools/install_ci_llvm_linux.sh` on Ubuntu 24.04 (x86-64 or ARM64), which adds the signed apt.llvm.org archive after checking its signing-key fingerprint and installs the 23.1 packages into `/usr/lib/llvm-23`. The runners' Ubuntu mirror sometimes stalls mid-transfer and apt would wait on it forever, so the script also writes `/etc/apt/apt.conf.d/80jai-ci-network` (30-second network timeouts, five retries; it applies to the job's later `apt-get` calls too) and runs `apt-get update` and `install` under a ten-minute `timeout`, three attempts each; the workflow steps that call it have `timeout-minutes: 15`, so a stall fails the step instead of using up the job. The script needs `sudo` and writes system apt configuration; use it on fresh CI runners only.

`jaic build -os wasm` also needs LLD's `wasm-ld`, which Homebrew and Debian package separately from LLVM (`lld`, `lld-23`); CI installs both ([wasm target](../native/wasm-target.md)).

The same prefix is read at run time, to find the sanitizer link driver (`bin/clang`, see [sanitizers](../native/sanitizers.md)), `llvm-symbolizer` in the sweep and `llvm-dwarfdump` in `tests/debug_info.rs`.

### Windows MSVC builds

The official `clang+llvm-23.1.2-{x86_64,aarch64}-pc-windows-msvc` archives are not linkable by llvm-sys as shipped. Their `llvm-config --link-static --system-libs` prints

```
psapi.lib shell32.lib ole32.lib uuid.lib advapi32.lib ws2_32.lib ntdll.lib zs.lib S:/llvm/utils/release/llvm_package_23.1.2/build_amd64_stage0/zstdbuild/install/lib/zstd_static.lib xml2s.lib
```

llvm-sys 231 only strips `.lib` from each entry on MSVC and emits `cargo:rustc-link-lib=S:/llvm/.../zstd_static`; rustc reads `S:` as a library rename and fails with "renaming of the library `S` was specified". llvm-sys has no variable to override or filter system libraries, and the archive ships none of `zs.lib` (zlib), `zstd_static.lib` and `xml2s.lib`.

`tools/windows-llvm/prepare.sh <llvm dir>`, run by `windows-native.yml` and `release.yml` right after unpacking the archive, fixes this:

- `xml2s.lib`: an empty library. Only LLVMWindowsManifest (lld, llvm-mt) uses libxml2 and jaic links nothing from it.
- `zs.lib` and `zstd_static.lib`: zlib and zstd (versions and SHA-256s pinned in the script) compiled with the release's clang against the static CRT, into the archive's `lib/`, which is already on llvm-sys's search path. LLVM's MC and Object libraries reference the compression API, so an empty stub would not link.
- `bin/llvm-config-23.exe`: `llvm-config-shim.c`, compiled there. llvm-sys tries `llvm-config-<LLVM major>.exe` (23, derived from its own version 231) before `llvm-config.exe`, so it runs the shim, which runs the real `llvm-config.exe` and, for `--system-libs` only, rewrites path-qualified `*.lib` entries to their file names. The script then checks that every system library is a Windows import library or exists in `lib/`, and fails otherwise.

Locally, the same steps apply from Git Bash after unpacking the archive; then set `LLVM_SYS_231_PREFIX` to it (with forward slashes) and build with `+crt-static` ([Windows](../native/windows.md#configuration)).

Inkwell 0.10.0 on crates.io stops at LLVM 22, so the root `Cargo.toml` takes Inkwell from git at an exact commit that satisfies the [dependency policy](dependency-policy.md). That revision's `const_int` masks values to the type's width (LLVM 23 no longer truncates them itself), so jaic's callers, which pass sign-extended `u64`s for narrow types, behave as before.

## How to change it

For a new LLVM major version:

- `Cargo.toml` (workspace `inkwell` feature) and `crates/jaic-llvm/Cargo.toml` (`dynamic`/`static` features), then `cargo update -p inkwell` and `python3 tools/check_dependency_age.py`.
- The `LLVM_SYS_*_PREFIX` name and the versioned tool names (`llvm-config-NN`, `clang-NN`, `llvm-symbolizer-NN`, `wasm-ld-NN`, the `lld@NN` keg and `/usr/lib/llvm-NN`) in `crates/jaic-llvm/src/lib.rs`, `crates/jaic-llvm/src/wasm.rs`, `crates/jaic-cli/tests/debug_info.rs` and `tools/jaic-sweep.py`.
- CI: `ci.yml`, `compile-bench.yml`, `nix.yml`, `release.yml` and `windows-native.yml` (`LLVM_VERSION`, prefix variable, Homebrew formula), and the archive and version check in `install_ci_llvm_linux.sh`. Review the archive key fingerprint whenever the script is updated.
- Windows: the shim's file name in `tools/windows-llvm/prepare.sh` (`llvm-config-23.exe`, the LLVM major version) and its known import-library list; re-check the archive's `--system-libs` output (printed by the workflows) and drop the shim once llvm-sys or the archive stop emitting absolute paths.
- Nix: `llvmPackages_NN` in `flake.nix` and `nix/jaic.nix` ([Nix](nix.md)).
- Check the workarounds in [LLVM backend](../native/llvm-backend.md) against the new release, and compare compile times ([compile-time benchmark](compile-time-benchmark.md)).

`git grep -n -E 'LLVM_SYS_|llvm@|llvm-[0-9]+'` finds the places that name a version.

## Configuration

```sh
brew install llvm@23 lld   # lld: wasm-ld, only for `jaic build -os wasm`
export LLVM_SYS_231_PREFIX="$(brew --prefix llvm@23)"
export PATH="$LLVM_SYS_231_PREFIX/bin:$PATH"
cargo build -p jaic-cli --locked
```

Homebrew's plain `llvm` is LLVM 23 as well, so `/opt/homebrew/opt/llvm` works as the prefix. Any LLVM 23.1 install with `llvm-config` and the shared `libLLVM` works, for example a distribution package. The official release tarballs ship only static, LTO-bitcode libraries on macOS, which only the release workflow's lld-based link can use ([releases](releases.md)).

## Dependencies

Inkwell (git, `b7cbeed24af8`), llvm-sys 231, LLVM and Clang 23.1, and LLD 23 for wasm. Windows CI also downloads zlib's and zstd's sources from their GitHub releases. The code generator is described in [LLVM backend](../native/llvm-backend.md).
