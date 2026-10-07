# Stdlib target check

## What it is

A test that type-checks every stdlib module for every target, including the code nothing calls. `crates/jaic-cli/tests/stdlib_targets.rs` runs `jaic check <file> -no_dce -os <os> -cpu <cpu>` for linux, macos and windows on x64 and arm64, and for wasm, on a program that only imports the module. The CPU is always given, so the result does not depend on the host's. CI runs it as the `stdlib-targets` step of `ci.yml` ([continuous integration](continuous-integration.md)).

Without it, a module's code for other platforms (`#if OS == .MACOS`) and the procedures no test calls are never type-checked, because [dead-code elimination](../language/dead-code-elimination.md) only checks module code a program reaches. A type error there stays hidden until a user on that platform calls the procedure. One example is the macOS input adapter, which used a constant from the wrong module.

## How it works

1. The test lists `stdlib/X.jai` files and `stdlib/X/module.jai` folders by import name (and `stdlib/Extensions/X/module.jai` as `Extensions/X`), minus the `skip` lines of `tests/stdlib-targets.txt`, which name the metaprograms (`Default_Metaprogram`, `Minimal_Metaprogram`) that build the command line's program when imported.
2. For each module and target it writes `#import "X"; main :: () {}` and checks it with `-no_dce`, which checks every declaration and procedure body in modules too, and `-os` and `-cpu`, which select the `#if OS` and `#if CPU` branches. Checks run in parallel, one per CPU; the whole run takes a few seconds with a release `jaic`.
3. A failed check is reduced to its first error as `file: message`, with the stdlib path and the line and column removed so that unrelated edits do not change it.
4. The result must match `tests/stdlib-targets.txt` exactly: a failure that is not listed, a listed failure with a different first error, and a listed module that now passes all fail the test, and the message lists each one.

The listed failures are intentional:

- modules for other platforms or CPUs, which stop with `#assert` (`Windows` on linux, `Objective_C` off Apple platforms, `Wasi_Runtime` off wasm, `nvtt` off x64);
- procedures that stop the build on a target they do not support (`Debug`'s `is_valid_pointer` off Windows);
- bindings to native libraries with no wasm build (`SDL`, `ImGui`, `Curl`, `meshoptimizer`).

Checking stops at the first error, so a listed module is only checked up to its `#assert`. Every other module and target is checked completely.

## How to change it

- After fixing a listed module, or when a change makes one fail differently, edit its line in `tests/stdlib-targets.txt`. The test's message prints the new first error in the file's format.
- A new platform-only module should stop on the wrong target with an `#assert` that says why, and get a line for each target it refuses.
- To add a target, extend `TARGETS` in the test (`os-cpu`, or a bare OS for no `-cpu`) and list that target's intentional failures.

## Configuration

- `cargo test -p jaic-cli --test stdlib_targets` (add `--release` for speed, `--no-default-features` to build without LLVM). The check needs no native libraries or toolchains.
- `tests/stdlib-targets.txt`: `skip Module` and `Module target: file: first error` lines, where the target is `linux-x64`, `macos-arm64`, ... or `wasm`; `#` starts a comment.
