# Nix flake

## What it is

`flake.nix` packages `jaic` (with the LLVM 23 backend), the `jailsp` language server, the `jailint` linter and the `jaifmt` formatter for Nix users, and provides a development shell with the pinned toolchain. It supports `x86_64-linux`, `aarch64-linux`, `x86_64-darwin` and `aarch64-darwin`.

| Output | Contents |
| --- | --- |
| `packages.<system>.jaic` (also `default`) | `bin/jaic`, `bin/jailsp` and `bin/jailint`, with `stdlib/` and `prelude/` |
| `packages.<system>.jaifmt` | `bin/jaifmt`, compiled from `jaifmt/main.jai` by the `jaic` package |
| `apps.<system>.{default,jaic,jailsp,jailint,jaifmt}` | `nix run` entry points |
| `devShells.<system>.default` | pinned nightly Rust, LLVM 23 and Clang, Python 3.14, Node, the env vars below |
| `overlays.default` | adds `pkgs.jaic` and `pkgs.jaifmt` |
| `checks.<system>` | builds both packages (their install checks run programs) |

## Using it

Run without installing:

```sh
nix run github:matteopolak/jai -- run hello.jai
nix run github:matteopolak/jai -- build hello.jai -o hello
nix run github:matteopolak/jai#jaifmt -- --check src
nix run github:matteopolak/jai#jailint -- src
```

Install into a profile: `nix profile install github:matteopolak/jai`.

As a flake input, in a NixOS (or nix-darwin) configuration:

```nix
{
  inputs.jai.url = "github:matteopolak/jai";

  outputs = { nixpkgs, jai, ... }: {
    nixosConfigurations.myhost = nixpkgs.lib.nixosSystem {
      modules = [
        ./configuration.nix
        { nixpkgs.overlays = [ jai.overlays.default ]; }
        ({ pkgs, ... }: { environment.systemPackages = [ pkgs.jaic pkgs.jaifmt ]; })
      ];
    };
  };
}
```

With home-manager, use the same overlay and `home.packages = [ pkgs.jaic pkgs.jaifmt ];`, or skip the overlay and reference `jai.packages.${pkgs.stdenv.hostPlatform.system}.jaic` directly.

Development shell (from a checkout): `nix develop`, then `cargo build -p jaic-cli --locked` as usual.

## How it works

`nix/jaic.nix` is a `buildRustPackage` over the workspace (`-p jaic-cli -p jai-language-server -p jailint`, default `dynamic-llvm` feature) using `cargoLock.lockFile = ./Cargo.lock`, so the build is offline against vendored crates. `inkwell` is a git dependency pinned to a commit ([dependency policy](dependency-policy.md)); `cargoLock.allowBuiltinFetchGit = true` lets Nix fetch that exact revision with `builtins.fetchGit` instead of keeping an `outputHashes` entry in sync with `Cargo.lock`.

- **Rust**: `rust-toolchain.toml` pins a nightly, which nixpkgs does not ship. The flake reads that file through oxalica's `rust-overlay` (`rust-bin.fromRustupToolchainFile`) and builds with `makeRustPlatform`, so Nix and rustup use the same compiler. `.cargo/config.toml` (the crate publish-age policy, which needs crates.io) is left out of the build source.
- **LLVM**: nixpkgs' `llvmPackages_23`. `LLVM_SYS_231_PREFIX` points at `llvm.dev`, which holds `bin/llvm-config`; `libLLVM` is linked dynamically from the store.
- **Layout**: `jaic` finds its standard library in `stdlib/` next to the executable (`jaic::stdlib_dir`; `JAIC_STDLIB` overrides it), and `stdlib/prelude/module.jai` loads `prelude`. The real binaries therefore live in `libexec/jaic/` beside `stdlib/` and `prelude/`, the same layout as the release archives ([releases](releases.md)). `bin/` holds `makeWrapper` scripts that exec the real path.
- **Wrapper**: `bin/jaic` sets `JAI_LIBCLANG` (only if unset) to nixpkgs' libclang 23 for the bindings generator, and appends the stdenv C compiler, its binutils (`cc`, `ar`) and, on macOS, LLVM's `dsymutil` to `PATH`. `jaic build` needs those to link. They are appended, so a `cc` already on the user's `PATH` wins.
- **Install check**: after installing, the derivation runs `examples/compile-time-record.jai` (exit 42) and a hello world under `jaic run`, then builds and runs both natively, and lints the hello world with `jailint -D warnings`. A broken wrapper or layout fails the build.
- **jaifmt**: `nix/jaifmt.nix` runs `jaic build jaifmt/main.jai -O2 --no-debug-info` with the `jaic` package as a build input, then formats a snippet as its check.
- **Overlay**: `overlays.default` re-exports this flake's packages, built from the flake's own nixpkgs. It does not rebuild against the consumer's nixpkgs, because older releases have neither LLVM 23 nor the overlay's toolchain.

CI: `.github/workflows/nix.yml` runs on ubuntu and macOS when the flake, `nix/`, `Cargo.lock` or `rust-toolchain.toml` change. It checks that `flake.lock` is current, runs `nix flake check` and `nix build`, then runs the result outside the checkout and enters the dev shell.

## How to change it

- **Bump inputs**: `nix flake update`, then build. Without Nix locally, push to a branch and let the `nix` workflow run (its first step fails if `flake.lock` does not match `flake.nix`).
- **New nightly**: editing `rust-toolchain.toml` is enough as long as the locked `rust-overlay` already has that date's manifest; otherwise run `nix flake update rust-overlay`.
- **New LLVM major**: change `llvmPackages_23` in `nix/jaic.nix` and `flake.nix` and the `LLVM_SYS_*_PREFIX` name in both, along with the Cargo side ([LLVM setup](llvm-setup.md)).
- **New runtime file**: anything `jaic` reads at run time must be added to the `fileset` in `nix/jaic.nix` and copied in `postInstall`. Files not in the fileset are not in the build sandbox.
- **Git dependencies** in `Cargo.lock` are fetched by `allowBuiltinFetchGit`. Once `inkwell` is back on crates.io and nothing else comes from git, remove that flag so the build is again fixed-output only.

Gotchas:

- Programs built by the Nix `jaic` link with the Nix C toolchain. On Linux they use the store's glibc and dynamic loader, so they run on the machine that built them but are not portable to non-Nix systems. Use the release archives for redistributable binaries.
- The bindings generator parses headers with libclang, but it does not provide system headers. On NixOS, run it inside a shell that has the headers you are binding (for example `nix shell nixpkgs#glibc.dev`), or pass include paths explicitly.
- `x86_64-darwin` evaluates, but CI does not build it, and nixpkgs is phasing the platform out.

## Configuration

- `JAIC_STDLIB`: overrides the stdlib directory (unset by the wrapper).
- `JAI_LIBCLANG`: libclang for the bindings generator (the wrapper sets a default).
- `JAIC_LINKER`, `JAIC_AR`: override the linker and archiver (otherwise `cc` and `ar` from `PATH`).
- Dev shell sets `LLVM_SYS_231_PREFIX` and `JAI_LIBCLANG`.

## Dependencies

nixpkgs `nixos-26.05` (`llvmPackages_23`, stdenv, `makeWrapper`, libffi, libxml2, ncurses, zlib, zstd), `oxalica/rust-overlay`, and the `cachix/install-nix-action` action for CI.
