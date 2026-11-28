# Prerequisites

What jacket, jaic, jaifmt, jailint and jailsp need from the machine, and what each says when it is missing.

## The rule

A missing prerequisite is found before the command does any work or changes any state. The command exits with an `error:` that names the need and the command that needs it, then a `note:` with the next step:

```
jacket: error: `curl` is not installed, and `jacket toolchain add` needs it
note: install curl with `apt install curl`
```

The note is the install command for the detected package manager (`brew`, `apt`, `dnf`, `pacman`, `winget`), the flag or variable to set, or `jacket toolchain add`. Jacket never runs it. There is no fallback to another tool.

## How to change it

- Install advice lives in one table: `Need` in `crates/jaic-toolchain/src/package.rs`. Add a variant and a row for a new program.
- In `jacket`, `crates/jacket/src/prerequisite.rs` builds the error: `missing`, `require` (a program on `PATH`), `spawn_error` (for a spawn that fails), `runner` and `writable_dir`. Check before the first side effect, not at the spawn.
- Tests live in `crates/jacket/tests/prerequisites.rs` (a copy of `jacket` run with an empty `PATH`) and `crates/jaic-cli/tests/missing_prerequisites.rs`.

## Inventory

| Need | Needed by | When missing |
| --- | --- | --- |
| C driver, linker (`cc`, `ld.lld`, `lld-link`, `link.exe`, `wasm-ld`), cross driver, sysroot, builtins | `jaic`, `jacket build/run/test` for a native or cross target | `error: no linker for TRIPLE` with a package note or `jacket toolchain add`; see [toolchain discovery](../native/toolchain-discovery.md) |
| `clang`, `llvm-ar` | `jacket toolchain add` for a freestanding target | `Missing` with the `clang` package note |
| `dsymutil` | `jaic` builds with debug information on macOS (not an unoptimized build, which keeps its objects) | error before compiling, with the Xcode tools note and `-no_debug_info`; Homebrew's `llvm` is keg-only, so it is not suggested |
| `jaic`, `jaifmt`, `jailint`, `jailsp` | `jacket` commands that start them | next to `jacket`, then `PATH`; error names the command and points at the installers |
| standard library folder | every compile, `jailint`, `jailsp` and `jaic -print_stdlib` (`JAIC_STDLIB`, else `stdlib/` beside the executable) | `error[cli-failure]: cannot find the standard library` and a note naming `JAIC_STDLIB` |
| `git` | `jacket migrate --apply` in a repository | error and install note. `jacket dep` also needs it: see [dependencies](dependencies.md) |
| `curl`, `tar` | `jacket upgrade`, `jacket toolchain add` | error and install note, before any download or prompt; `toolchain add` also checks the cache folder first |
| user cache folder (`XDG_CACHE_HOME`, `LOCALAPPDATA`) | `jacket toolchain add`, `jacket test` | error naming the folder and the variable that moves it |
| runner (`wasmtime`, `qemu-*`, `node`, a path) | `jacket test --runner`, `--target`, board runners | checked before any build; error and install note for the program |
| `node` | `jacket doc --runtime` | error and install note when the arguments are read, or drop `--runtime` |
| `python3` | `jacket test` building native test libraries, only when one is missing | error with install note and the command to run by hand, before any test is built |
| `llvm-dwarfdump` | `jacket test --coverage` | error with the LLVM install note, or set `LLVM_PREFIX` |
