# LLVM setup

## What it is

`crates/jaic-llvm` uses Inkwell (`llvm23-1-prefer-dynamic`, from a pinned commit of its main branch) over a separately installed LLVM 23.1, and `jaic build` links with the host's Clang. Building `jaic-cli` needs LLVM; `jaic check` and `jaic run` don't use it at run time. `jai-wasm` doesn't depend on LLVM.

## How it works

`llvm-sys` finds LLVM through `LLVM_SYS_231_PREFIX` (a directory containing `bin/llvm-config` and `lib/`). CI sets it per platform: Homebrew's `llvm@23` on macOS (`tools/install_ci_llvm_macos.sh`, which runs `brew update` and retries when a runner image's Homebrew predates the `llvm@23` alias), and `tools/install_ci_llvm_linux.sh` on Ubuntu 24.04 (x86-64 or ARM64), which adds the signed apt.llvm.org archive after checking its signing-key fingerprint and installs the 23.1 packages into `/usr/lib/llvm-23`. The script needs `sudo` and writes system apt configuration; use it on fresh CI runners only.

`jaic build -os wasm` also needs LLD's `wasm-ld`, which Homebrew and Debian package separately from LLVM (`lld`, `lld-23`); CI installs both ([wasm target](../native/wasm-target.md)).

The same prefix is read at run time, to find the sanitizer link driver (`bin/clang`, see [sanitizers](../native/sanitizers.md)), `llvm-symbolizer` in the sweep and `llvm-dwarfdump` in `tests/debug_info.rs`.

Inkwell 0.10.0 on crates.io stops at LLVM 22, so the root `Cargo.toml` takes Inkwell from git at an exact commit that satisfies the [dependency policy](dependency-policy.md). That revision's `const_int` masks values to the type's width (LLVM 23 no longer truncates them itself), so jaic's callers, which pass sign-extended `u64`s for narrow types, behave as before.

## How to change it

For a new LLVM major version:

- `Cargo.toml` (workspace `inkwell` feature) and `crates/jaic-llvm/Cargo.toml` (`dynamic`/`static` features), then `cargo update -p inkwell` and `python3 tools/check_dependency_age.py`.
- The `LLVM_SYS_*_PREFIX` name and the versioned tool names (`llvm-config-NN`, `clang-NN`, `llvm-symbolizer-NN`, `wasm-ld-NN`, the `lld@NN` keg and `/usr/lib/llvm-NN`) in `crates/jaic-llvm/src/lib.rs`, `crates/jaic-llvm/src/wasm.rs`, `crates/jaic-cli/tests/debug_info.rs` and `tools/jaic-sweep.py`.
- CI: `ci.yml`, `compile-bench.yml`, `nix.yml`, `release.yml` and `windows-native.yml` (`LLVM_VERSION`, prefix variable, Homebrew formula), and the archive and version check in `install_ci_llvm_linux.sh`. Review the archive key fingerprint whenever the script is updated.
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

Inkwell (git, `b7cbeed24af8`), llvm-sys 231, LLVM and Clang 23.1, and LLD 23 for wasm. The code generator is described in [LLVM backend](../native/llvm-backend.md).
