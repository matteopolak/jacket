# LLVM setup

`crates/jaic-llvm` uses Inkwell (from a pinned commit; see [development](development.md#dependency-policy)) over a separately installed LLVM 23.1, and `jaic` links with the host's Clang. Building `jaic-cli` needs LLVM; `jaic -check` and `-interp` don't use it at run time. `jai-wasm` doesn't depend on it.

```sh
brew install llvm@23 lld   # lld: wasm-ld, only for `jaic -os wasm`
export LLVM_PREFIX="$(brew --prefix llvm@23)"
export LLVM_SYS_231_PREFIX="$LLVM_PREFIX"   # the name follows the LLVM version
export PATH="$LLVM_PREFIX/bin:$PATH"
cargo build -p jaic-cli --locked
```

Any LLVM 23.1 install with `llvm-config` and shared `libLLVM` works. The official macOS tarballs ship only static LTO-bitcode libraries, usable only by the release workflow's lld-based link ([releases](releases.md)).

## How it works

`llvm-sys` finds LLVM through `LLVM_SYS_<major><minor>_PREFIX`. CI sets it per platform through `.github/actions/setup-llvm` (`setup-windows` on Windows), which also exports `LLVM_VERSION`, `LLVM_MAJOR`, `LLVM_SYS_PREFIX_VAR` and `LLVM_PREFIX`. The version is `llvm` in `.github/pins.json` and nowhere else; `tools/ci/pins.py check` fails when the full version is spelled out in `.github/`, `tools/` or `docs/`.

### Where jaic finds LLVM

Programs used at run time (the sanitizer link driver, `wasm-ld`, `llvm-dwarfdump`, `llvm-symbolizer`) come from one search order, in `jacket_config::llvm` (Rust) and `tools/toolkit/llvm.py` (Python):

1. the `llvm-sys` variable, then `LLVM_PREFIX`;
2. the prefix the build saw (`crates/jaic-llvm/build.rs`; Rust only);
3. `llvm-config --prefix` (`llvm-config-<major>` first);
4. Homebrew's `llvm@<major>` and Debian's `/usr/lib/llvm-<major>`;
5. `<tool>-<major>` or `<tool>` on `PATH`.

The Rust side takes the version from `crates/jacket-config/src/llvm/version.rs`; a test fails when it disagrees with the pin.

### CI installs

- **macOS arm64:** Homebrew via `tools/ci/install_ci_llvm_macos.sh`. **macOS Intel:** Homebrew has no bottles, so the script installs conda-forge's LLVM with a pinned micromamba and puts only LLVM's tools on `PATH`.
- **Linux (Ubuntu 24.04):** `tools/ci/install_ci_llvm_linux.sh` adds the signed apt.llvm.org archive after checking its key fingerprint. Fresh CI runners only; it needs `sudo`.
- **Windows MSVC:** the official archives aren't linkable by llvm-sys as shipped (`llvm-config --system-libs` prints absolute build-machine paths and some `.lib` files are missing). `tools/ci/windows-llvm/prepare.sh <llvm dir>` adds the missing libraries, builds zlib and zstd, and compiles an `llvm-config-<major>.exe` shim that rewrites the paths. Locally, run it from Git Bash, set the `llvm-sys` variable with forward slashes and build with `+crt-static` ([Windows](../native/windows.md#configuration)). If the flag comes from `CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS`, also pass `--target x86_64-pc-windows-msvc`, or the CRT isn't linked statically.

`jaic -os wasm` also needs LLD's `wasm-ld` (`lld`, `lld-<major>`; [wasm target](../native/wasm-target.md)).

### Registered backends and binary size

`jaic` registers only the x86-64, AArch64 and WebAssembly backends (`target_machine` in `crates/jaic-llvm/src/lib.rs`), not all twenty, which would make the binary huge. The root `Cargo.toml` enables inkwell's `target-x86`, `target-aarch64` and `target-webassembly`. The asm parser stays because inline asm is assembled by the backend. Other architectures are rejected before LLVM sees them.

The release workflow strips packaged binaries, so their backtraces have no function names; build from source for symbolised ones.

## How to change it

- **New target architecture:** an `Arch` variant in `crates/jaic/src/abi.rs`, the `target-<name>` inkwell feature, and `Target::initialize_<name>` in `target_machine`. Expect the binary to grow.
- **New LLVM version:** change `llvm` in `.github/pins.json`; for a patch release that's the only edit. A new major or minor also changes the inkwell feature (`llvm23-1`; then `cargo update -p inkwell` and `check_dependency_age.py`), `crates/jacket-config/src/llvm/version.rs`, the apt key fingerprint in `install_ci_llvm_linux.sh`, the Homebrew formula in `release.yml`, `llvmPackages_NN` in `flake.nix` and `nix/jaic.nix`, and the Windows shim in `prepare.sh`. Recheck the workarounds in [LLVM backend](../native/llvm-backend.md). Search for `LLVM_SYS_`, `llvm@` and `llvm-[0-9]+` to find the rest.
- Inkwell's `const_int` masks values to the type's width at the pinned revision, so sign-extended `u64` arguments for narrow types behave as before.
