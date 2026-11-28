# LLVM setup

`crates/jaic-llvm` uses Inkwell ([dependency policy](development.md#dependency-policy)) over a separately installed LLVM 23.1, and `jaic` links with the host's Clang. Building `jaic-cli` needs LLVM; `jaic -check` and `-interp` don't use it at run time. `jai-wasm` doesn't depend on it.

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
2. the prefix the build saw (`crates/jaic-native/build.rs`; Rust only);
3. `llvm-config --prefix` (`llvm-config-<major>` first);
4. Homebrew's `llvm@<major>` and Debian's `/usr/lib/llvm-<major>`;
5. `<tool>-<major>` or `<tool>` on `PATH`.

A variable of step 1 that is set to a folder with no `bin/` is an error in `jaic` before any tool is looked up (`jacket_config::llvm::check_configured`), so a wrong prefix never leaves the search to find another LLVM.

A Python script that needs one tool calls `toolkit.llvm.require(name)`, which exits naming the tool and `LLVM_PREFIX` when none of these has it. Scripts do not fall back to a look-alike such as `ar` for `llvm-ar`.

The Rust side takes the version from `crates/jacket-config/src/llvm/version.rs`; a test fails when it disagrees with the pin.

### CI installs

- **macOS arm64:** Homebrew via `tools/ci/install_ci_llvm_macos.sh`. **macOS Intel:** Homebrew has no bottles, so the script installs conda-forge's LLVM with a pinned micromamba and puts only LLVM's tools on `PATH`.
- **Linux (Ubuntu 24.04):** `tools/ci/install_ci_llvm_linux.sh` adds the signed apt.llvm.org archive after checking its key fingerprint. Fresh CI runners only; it needs `sudo`.
- **Windows MSVC:** the official archives aren't linkable by llvm-sys as shipped (`llvm-config --system-libs` prints absolute build-machine paths and some `.lib` files are missing). `tools/ci/windows-llvm/prepare.sh <llvm dir>` adds the missing libraries, builds zlib and zstd, and compiles an `llvm-config-<major>.exe` shim that rewrites the paths. Locally, run it from Git Bash, set the `llvm-sys` variable with forward slashes and build with `+crt-static` ([Windows](../native/windows.md#configuration)). If the flag comes from `CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS`, also pass `--target x86_64-pc-windows-msvc`, or the CRT isn't linked statically.

`jaic -os wasm` also needs LLD's `wasm-ld` (`lld`, `lld-<major>`; [wasm target](../native/wasm-target.md)).

### Backends

`jaic` registers only the backends its targets use (`target_machine` in `crates/jaic-llvm/src/machine.rs`), not all of LLVM's, which would make the binary huge. The asm parser stays because inline asm is assembled by the backend.

| Backend | Registered by | Needs |
| --- | --- | --- |
| x86, AArch64, WebAssembly, ARM | inkwell's `target-*` features in the root `Cargo.toml` | nothing: every LLVM release has them |
| SystemZ, PowerPC, AVR | `initialize_optional_backends` | an LLVM built with the backend |

`crates/jaic-llvm/build.rs` runs the linked LLVM's `llvm-config --targets-built` and sets `llvm_backend = "systemz"`, `"powerpc"` or `"avr"` for each one present. The official Windows releases have none of the three, so a Windows `jaic` can't build for s390x, PowerPC64 or AVR: asking for one fails with "the LLVM this jaic was built with has no backend for '<target>'". `-interp -target <triple>` still works, since it needs no backend.

The release workflow strips packaged binaries, so their backtraces have no function names; build from source for symbolised ones.

## How to change it

- **New target architecture:** an `Arch` variant in `crates/jaic/src/abi.rs`, then either the `target-<name>` inkwell feature and `Target::initialize_<name>` in `target_machine` (if every LLVM release has the backend), or an `OPTIONAL` entry in `crates/jaic-llvm/build.rs` and a `register_backend!` call in `initialize_optional_backends`. Expect the binary to grow.
- **New LLVM version:** change `llvm` in `.github/pins.json`; for a patch release that's the only edit. A new major or minor also changes the inkwell feature (`llvm23-1`; then `cargo update -p inkwell` and `check_dependency_age.py`), `crates/jacket-config/src/llvm/version.rs`, the apt key fingerprint in `install_ci_llvm_linux.sh`, the Homebrew formula in `release.yml`, `llvmPackages_NN` in `flake.nix` and `nix/jaic.nix`, and the Windows shim in `prepare.sh`. Recheck the workarounds in [LLVM backend](../native/llvm-backend.md). Search for `LLVM_SYS_`, `llvm@` and `llvm-[0-9]+` to find the rest.
