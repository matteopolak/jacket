# Toolchain discovery and `jacket toolchain`

Cross and bare-metal builds need tools beyond `jaic`: a linker for the target, the target's C library (a sysroot) for hosted targets, and the compiler builtins for freestanding ones. Jacket finds them from the host and the triple, with no configuration. `jacket toolchain` shows what was found and fetches the pieces no package manager provides.

## Rules

- **`jacket.toml` never names a machine's tools.** It holds `triple`, `board`, `runner` (tool names with placeholders) and link settings written by meaning. Never a linker, archiver, sysroot or builtins path.
- **The system's tools come first.** Jacket ships no linker. It uses the installed `cc`, `ld`, `ld.lld`, `lld-link`, `link.exe`, `wasm-ld` or a GNU cross driver.
- **Builds never touch the network.** Only `jacket toolchain add` downloads. Every download is pinned, sha256-checked and written only to jacket's cache.
- **Jacket never installs system packages.** A missing tool is an error whose `note:` gives the package-manager command for the detected OS, or the `jacket toolchain add` command.
- **No silent fallbacks.** The order below is fixed. Each step finds the tool or ends in the error.

## Lookup order

1. An override: the `-linker`, `-sysroot` or `-compiler_rt` flag, else `JAIC_LINKER`, `JAIC_SYSROOT` or `JAIC_COMPILER_RT`. These are for CI and scripts, never for `jacket.toml`. A named path that does not exist is an error.
2. The system tool that handles the triple. A candidate is asked (`ld.lld --version`, `cc -dumpmachine`), not judged by name.
3. Another installed tool that handles it: `clang --target=<triple>` with an `ld.lld`, or the GNU driver for the triple (`arm-linux-gnueabihf-gcc`, `arm-none-eabi-gcc`, `avr-gcc`).
4. The jacket cache, filled by `jacket toolchain add`.
5. An error with a note.

| Target | Linker | Libraries |
| --- | --- | --- |
| Host | macOS: `cc`; Linux: `cc`; Windows: `link.exe` | the system's |
| Cross Linux | `cc --target=T --ld-path=<ld.lld>`, or the GNU cross driver | the system sysroot (`/usr/<gnu-triple>`), else the cached one |
| Windows from macOS or Linux | `lld-link` or the MinGW driver | MinGW import libraries |
| wasm32, wasm64 | `wasm-ld` | none |
| Bare metal and boards | `ld.lld` | builtins, below; the board's linker script |

Builtins for a freestanding target, in order: the override file; the LLVM install's `libclang_rt.builtins.a` for the triple; `libgcc.a` from an installed `arm-none-eabi-gcc` or `avr-gcc` (asked with `-print-libgcc-file-name`); the cached compiler-rt build. Thumb-1 (`thumbv6m`) has no backend and is reported as a missing linker.

## Error notes

```
error: no linker for thumbv7em-none-eabihf
note: install one with `brew install lld`, or set JAIC_LINKER

error: no Linux C library for armv7-unknown-linux-gnueabihf
note: run `jacket toolchain add armv7-unknown-linux-gnueabihf`
```

The note names the package for the manager jacket detects (`brew`, `apt`, `dnf`, `pacman`, `winget`), or the package and its download page when none is detected. The table is `Need` in `crates/jaic-toolchain/src/package.rs`; a test checks that every row has text for every manager.

## Status lines

Cross and bare-metal builds print where each tool came from, in the bold right-aligned style of `Compiling` and `Finished`. Host builds print them only with `-v` (`-verbose` in jaic). An override is labelled. `-quiet` hides them.

```
   Compiling blink (thumbv7em-none-eabihf, board Mps2_An386)
      Linker /usr/bin/ld.lld 19.1.7
    Builtins ~/Library/Caches/jacket/toolchains/thumbv7em-none-eabihf/builtins/libclang_rt.builtins.a compiler-rt 21.1.8 (cached)
      Script board.ld
    Finished debug in 0.92s
```

Cross Linux links add a `Sysroot` line. Home folders are shortened to `~` on screen only.

## `jacket toolchain`

```sh
jacket toolchain                                  # every target: linker, libraries, runner
jacket toolchain check [TARGET...] [--tool NAME]  # exit 1 when anything is missing, with notes
jacket toolchain add TARGET...                    # fetch a sysroot or build builtins into the cache
jacket toolchain list                             # what the cache holds
jacket toolchain remove TARGET... | --all
```

With no subcommand it lists the host, the project's `[target.NAME]` tables and the built-in boards. `check --tool node` checks a program on `PATH` (CI uses it for `node`, `qemu-*`).

`add` takes a Linux triple or a freestanding one:

- **Sysroot** (`armv7-unknown-linux-gnueabihf`, `aarch64-unknown-linux-gnu`, `i686-...`, `x86_64-...`, `s390x-...`): a pinned set of Debian bookworm packages (`libc6`, `libc6-dev`, `linux-libc-dev`, `libcrypt`, `libgcc`) unpacked into `<cache>/jacket/toolchains/<triple>/sysroot`.
- **Builtins** (`aarch64-none-elf`, `thumbv7em-none-eabihf`, `thumbv8m.main-none-eabihf`, `avr-none-elf`): compiler-rt's pinned source, compiled file by file with the installed `clang` and archived with `llvm-ar`. No cmake, ninja, git or bash, so it works on Windows. A second `add` says `(cached)`; a build made from an older source list is rebuilt.

`XDG_CACHE_HOME` moves the cache. `remove` deletes only entries `add` made.

## `jacket.toml`

```toml
[target.pi]
triple = "armv7-unknown-linux-gnueabihf"
runner = "qemu-arm -L {sysroot} {exe}"
stack_size = 8192
entry = "main"
gc_sections = true
```

- `runner` words may use `{exe}`, `{args}` and `{sysroot}`. `{sysroot}` is the system's cross sysroot, else the cached one; none found is an error with the `add` note. It is looked up only when running.
- `stack_size`, `entry` and `gc_sections` are written by meaning. jaic renders them for the linker it uses:

| Meaning | `ld.lld` | MSVC / lld-link | Apple `ld` | GNU driver |
| --- | --- | --- | --- | --- |
| `gc_sections` | `--gc-sections` | `/OPT:REF` | `-Wl,-dead_strip` | `-Wl,--gc-sections` |
| `entry = S` | `--entry=S` | `/ENTRY:S` | `-Wl,-e,S` | `-Wl,--entry=S` |
| `stack_size = N` | `-z stack-size=N` | `/STACK:N` | `-Wl,-stack_size,0xN` | `-Wl,-z,stack-size=N` (MinGW: `-Wl,--stack,N`) |

- Raw linker arguments (`-link_arg`, `additional_linker_arguments`) use `ld.lld`'s spelling and go through the same translation.
- Flashing takes the port from `--port` or `JACKET_PORT`, never from the file.

## How to change it

- Lookup and notes: `crates/jaic-toolchain/src/` (`lookup.rs` finds programs, `linker.rs` linkers and sysroots, `builtins.rs` builtins, `package.rs` the notes, `pins.rs` the pins). It reads the machine through the `Host` trait, so tests build a `FakeHost` instead of needing real tools.
- Pins: `crates/jaic-toolchain/data/sysroots.txt` and `compiler-rt.txt` are generated. Rewrite them with `PYTHONPATH=tools python3 tools/embedded/pin_sysroots.py` or `pin_compiler_rt.py`; `--check` fails when they are stale.
- Status lines: `crates/jaic/src/progress.rs`; callers in `crates/jaic-native/src/`.
- `add`, `list`, `remove`: `crates/jacket/src/toolchain/`.
- Link-setting translation: `crates/jaic-native/src/link_options.rs`.
- Tests that need a tool call `test_support::toolchain::buildable(triple)` (`buildable_with_builtins` when the program needs the compiler builtins): it skips with the note when the tool is missing, and fails under `CI`.

## Configuration

| Name | Effect |
| --- | --- |
| `-linker`, `JAIC_LINKER` | the linker or driver to use |
| `-sysroot`, `JAIC_SYSROOT` | the sysroot folder for a Linux target |
| `-compiler_rt`, `JAIC_COMPILER_RT` | one builtins archive for a freestanding target |
| `JAIC_AR` | the archiver |
| `XDG_CACHE_HOME` | moves jacket's cache |

Removed: `JAIC_LLD`, `JAIC_WASM_LD`, `JAIC_SANITIZER_CC`, `JAIC_CROSS_LIBS`, `JAIC_REQUIRE_TOOLCHAIN`, the `cross_link.sh` and `build-compiler-rt.sh` scripts. `jacket targets` is now `jacket toolchain`.

## Dependencies

`jaic-toolchain` depends on `jaic` (for `Target`). `jaic-native` links with it, `jacket` lists and fills the cache with it, and `test-support` finds tools with it. Downloads use `curl`; `.deb` files are read by a small `ar` parser, then unpacked with `tar`.

Related: [embedded and freestanding targets](embedded.md), [linking](native-linking.md), [targets](../compiler/targets.md).
