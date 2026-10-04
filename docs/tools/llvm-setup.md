# LLVM setup

## What it is

`crates/jaic-llvm` uses Inkwell 0.10 (`llvm22-1-prefer-dynamic`) over an independently installed LLVM 22.1, and `jaic build` links with the host's Clang. Building `jaic-cli` needs LLVM; `jaic check` and `jaic run` do not need it at run time. The `jai-wasm` crate does not depend on LLVM.

## How it works

`llvm-sys` finds LLVM through `LLVM_SYS_221_PREFIX` (a directory containing `bin/llvm-config` and `lib/`). CI sets it per platform: Homebrew's `llvm@22` on macOS, and `tools/install_ci_llvm_linux.sh` on Ubuntu 24.04 (x86-64 or ARM64), which adds the signed apt.llvm.org archive after checking its signing-key fingerprint and installs the 22.1 packages into `/usr/lib/llvm-22`. The script needs `sudo` and writes system apt configuration; use it on fresh CI runners only.

## How to change it

For a new LLVM major version, bump the `inkwell` feature in the root `Cargo.toml`, the `LLVM_SYS_*_PREFIX` variable name in `ci.yml`, and the version checks in `install_ci_llvm_linux.sh`. Review the archive key fingerprint whenever the script is updated.

## Configuration

```sh
brew install llvm@22
export LLVM_SYS_221_PREFIX="$(brew --prefix llvm@22)"
export PATH="$LLVM_SYS_221_PREFIX/bin:$PATH"
cargo build -p jaic-cli --locked
```

## Dependencies

Inkwell 0.10.0, llvm-sys 221, LLVM and Clang 22.1. The code generator itself is documented in the native-build docs under `docs/native/`.
