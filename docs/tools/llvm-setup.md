# LLVM setup

## What it is

`crates/jaic-llvm` uses Inkwell (`llvm23-1-prefer-dynamic`, from a pinned commit; see [development](development.md#dependency-policy)) over a separately installed LLVM 23.1, and `jaic` links with the host's Clang. Building `jaic-cli` needs LLVM; `jaic -check` and `-interp` do not use it at run time. `jai-wasm` does not depend on it.

```sh
brew install llvm@23 lld   # lld: wasm-ld, only for `jaic -os wasm`
export LLVM_SYS_231_PREFIX="$(brew --prefix llvm@23)"
export PATH="$LLVM_SYS_231_PREFIX/bin:$PATH"
cargo build -p jaic-cli --locked
```

Any LLVM 23.1 install with `llvm-config` and shared `libLLVM` works. The official macOS tarballs ship only static LTO-bitcode libraries, usable only by the release workflow's lld-based link ([releases](releases.md)).

## How it works

`llvm-sys` finds LLVM through `LLVM_SYS_231_PREFIX` (a directory with `bin/llvm-config` and `lib/`). The same prefix is read at run time for the sanitizer link driver (`bin/clang`, [sanitizers](../native/sanitizers.md)), `llvm-symbolizer` in the sweep and `llvm-dwarfdump` in `tests/debug_info.rs`. CI sets it per platform through the `.github/actions/setup-llvm` composite action (`setup-windows` on Windows), which exports `LLVM_VERSION`, `LLVM_MAJOR`, the variable name and `LLVM_PREFIX` (the install directory, for steps that should not spell the version-specific variable). The version is `llvm` in `.github/pins.json` and nowhere else: `tools/ci/pins.py llvm-env` derives the major version and the `LLVM_SYS_<major><minor>_PREFIX` name from it, the scripts source `tools/ci/llvm_env.sh`, and `install_pinned.py llvm-release` takes archive names with a `{version}` placeholder. `pins.py check` (run with the policy tests) fails when the full version is spelled out in `.github/`, `tools/` or `docs/`.


- **macOS arm64:** Homebrew `llvm@<major>` via `tools/ci/install_ci_llvm_macos.sh`. **macOS Intel:** Homebrew has no bottles, so the script installs conda-forge's LLVM of the pinned version with a pinned micromamba, links with an rpath to it, and puts only LLVM's tools on `PATH` so conda's `cc`/`ld` do not shadow the system's (`--static` installs only static libraries for the release's `macos-x64` archive).
- **Linux (Ubuntu 24.04, x86-64/ARM64):** `tools/ci/install_ci_llvm_linux.sh` adds the signed apt.llvm.org archive after checking its key fingerprint and installs into `/usr/lib/llvm-<major>`. It writes apt network timeouts and retries (the mirror sometimes stalls) and needs `sudo`; use it on fresh CI runners only.
- **Windows MSVC:** the official `clang+llvm-<version>-*-pc-windows-msvc` archives are not linkable by llvm-sys as shipped: `llvm-config --system-libs` prints absolute build-machine paths (`S:/llvm/.../zstd_static.lib`) that rustc reads as a library rename, and `zs.lib`, `zstd_static.lib`, `xml2s.lib` are missing. `tools/ci/windows-llvm/prepare.sh <llvm dir>` (run by the Windows and release workflows after unpacking) adds an empty `xml2s.lib`, builds zlib and zstd (pinned versions and SHA-256) into `lib/`, and compiles `bin/llvm-config-<major>.exe`, a shim llvm-sys picks first, which runs the real `llvm-config` and rewrites path-qualified `*.lib` entries to file names. It then verifies every system library resolves. Locally: run it from Git Bash, set `LLVM_SYS_231_PREFIX` (forward slashes) and build with `+crt-static` ([Windows](../native/windows.md#configuration)).

`jaic -os wasm` also needs LLD's `wasm-ld` (packaged separately: `lld`, `lld-23`; [wasm target](../native/wasm-target.md)).

### Registered backends and binary size

`jaic` registers only the x86-64, AArch64 and WebAssembly backends (`target_machine` in `crates/jaic-llvm/src/lib.rs`), not `Target::initialize_all`, which links all twenty (0.6.2's macOS arm64 `jaic` was 136 MiB). The root `Cargo.toml` enables inkwell's `target-x86`, `target-aarch64` and `target-webassembly` features. The disassembler is skipped; the asm parser stays because inline asm is assembled by the backend. Other architectures are rejected before LLVM sees them (`jaic::abi::Arch::from_triple`). Unreferenced archive members are never loaded and the linker drops the rest. `llvm-sys`'s `disable-alltargets-init` is deliberately not enabled: an unverified link failure on one of six release platforms costs more than it saves.

The release workflow strips packaged `jaic`, `jailsp` and `jailint` after the PGO/BOLT build (`strip -x` then ad-hoc `codesign` on macOS, `strip --strip-unneeded` on Linux; Windows symbols live in the unshipped `.pdb`). Backtraces from stripped binaries have no function names; build from source for symbolised ones. Internal-error reports are unaffected.

## How to change it

- **New target architecture:** an `Arch` variant in `crates/jaic/src/abi.rs`, the `target-<name>` inkwell feature, and `Target::initialize_<name>` in `target_machine`; expect the static binary to grow.
- **New LLVM version:** change `llvm` in `.github/pins.json`; for a patch release that is the only edit (`pins.py check` finds a stray copy of the old version). A new major or minor version also changes what llvm-sys expects, which CI cannot derive: the `llvm-sys` feature that `Cargo.toml` and `crates/jaic-llvm/Cargo.toml` select through inkwell (`llvm23-1`; then `cargo update -p inkwell` and `tools/ci/check_dependency_age.py`), the `LLVM_SYS_*_PREFIX` name and versioned tool names (`llvm-config-NN`, `clang-NN`, `llvm-symbolizer-NN`, `wasm-ld-NN`, `lld@NN`, `/usr/lib/llvm-NN`) in `crates/jaic-llvm/src/lib.rs`, `wasm.rs`, `crates/jaic-cli/tests/debug_info.rs`, `tools/differential/jaic_sweep.py` and `tools/fuzz/stdlib_fuzz.py`; the apt signing key fingerprint in `install_ci_llvm_linux.sh`; the Homebrew formula in `release.yml`; `llvmPackages_NN` in `flake.nix` and `nix/jaic.nix`; the shim in `tools/ci/windows-llvm/prepare.sh` (drop it once llvm-sys or the archive stop emitting absolute paths). Recheck the workarounds in [LLVM backend](../native/llvm-backend.md) and compile times. Searching the tree for `LLVM_SYS_`, `llvm@` and `llvm-[0-9]+` finds version mentions.
- Inkwell's `const_int` at the pinned revision masks values to the type's width (LLVM 23 no longer truncates), so jaic's sign-extended `u64` arguments for narrow types behave as before.

## Dependencies

Inkwell (git), llvm-sys 231, LLVM and Clang 23.1, LLD 23 for wasm, the platform's `strip` and `codesign`. Windows CI also downloads zlib and zstd sources.
