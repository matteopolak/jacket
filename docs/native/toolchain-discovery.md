# Plan: toolchain discovery and `jacket toolchain`

Status: plan, being built milestone by milestone. Each milestone updates this page and the docs it names when it lands.

Cross and bare-metal builds need tools beyond `jaic`: a linker for the target, the target's C library (a sysroot) for hosted targets, and the compiler builtins for freestanding ones. Today the user names some of these in environment variables (`JAIC_LINKER=arm-linux-gnueabihf-gcc`, `JAIC_COMPILER_RT`). This plan finds them from the host and the triple instead, with no configuration, and adds one command for the pieces no package manager provides.

## Rules

- **`jacket.toml` never names a machine's tools.** A project file says what the project is (`triple`, `board`, `runner`), never how one machine builds it: no linker, archiver, sysroot or builtins path. Anyone on any OS can build a project from its `jacket.toml`.
- **The system's tools come first.** Jacket ships no linker. It uses the installed `cc`, `ld`, `ld.lld`, `link.exe` or a GNU cross driver.
- **Builds never touch the network.** Anything that must be downloaded is fetched only by `jacket toolchain add`.
- **Jacket never installs system packages, and never offers to.** A missing tool is an error whose note gives the command that fixes it.
- **No silent fallbacks.** The lookup order below is fixed. Each step either finds the tool or ends in an error; it never switches to a different toolchain without saying so.

## Lookup order

For each link, in this order:

1. An override: the `-linker`, `-sysroot` or `-compiler_rt` flag, else `JAIC_LINKER`, `JAIC_SYSROOT` or `JAIC_COMPILER_RT`. These are for CI and scripts, and never come from `jacket.toml`.
2. The system linker, if it handles the triple. Each candidate is asked, not judged by name: `ld.lld --version` and its targets, `cc -print-targets` or a dry run, `link.exe` through `vswhere`.
3. Another installed tool that handles it: `clang --target=<triple>` with an `ld.lld` on `PATH`, or the GNU driver named for the triple (`arm-linux-gnueabihf-gcc`, `arm-none-eabi-gcc`, `avr-gcc`).
4. The jacket cache, filled by `jacket toolchain add`.
5. An error with a note.

| Target | Linker | Libraries |
| --- | --- | --- |
| The host | macOS: Xcode `ld` (today's direct path); Linux: `cc`; Windows: `link.exe` | the system's |
| Cross Linux | `ld.lld`, `clang --target`, or the GNU cross driver | the system sysroot (`/usr/<gnu-triple>`), else the cached one |
| Windows from macOS or Linux | `lld-link` or the MinGW driver | MinGW import libraries |
| macOS from Linux or Windows | `ld.lld` | an installed Apple SDK only: it cannot be redistributed |
| wasm32, wasm64 | `wasm-ld` | the WASI sysroot |
| Bare metal and boards | `ld.lld` | builtins: `libgcc.a` from an installed GNU embedded toolchain, else cached compiler-rt; the board's linker script |

Runners are found the same way: `qemu-*` on `PATH`, the Homebrew prefix, Debian's locations and `Program Files\qemu`; `wasmtime` or `node` for wasm; `avrdude`, `probe-rs` or `picotool` for flashing.

## Error notes

The note names the package for the package manager jacket detects (`brew`, `apt`, `dnf`, `pacman`, `winget`). With none detected, it names the package and its download page. When no package manager provides the piece, it gives the `jacket toolchain` command.

```
error: no linker for thumbv7em-none-eabihf
note: install one with `brew install lld`, or set JAIC_LINKER

error: no Linux C library for armv7-unknown-linux-gnueabihf
note: run `jacket toolchain add armv7-unknown-linux-gnueabihf`
```

| Need | macOS | Debian, Ubuntu | Fedora | Arch | Windows |
| --- | --- | --- | --- | --- | --- |
| `ld.lld`, `lld-link`, `wasm-ld` | `brew install lld` | `apt install lld` | `dnf install lld` | `pacman -S lld` | `winget install LLVM.LLVM` |
| Cross Linux libc (armhf) | `jacket toolchain add` | `apt install gcc-arm-linux-gnueabihf` | `dnf install gcc-arm-linux-gnu` | `jacket toolchain add` | `jacket toolchain add` |
| Windows import libraries | `brew install mingw-w64` | `apt install gcc-mingw-w64` | `dnf install mingw64-gcc` | `pacman -S mingw-w64-gcc` | MSVC, or `winget install MartinStorsjo.LLVM-MinGW.UCRT` |
| Cortex-M builtins | `brew install --cask gcc-arm-embedded` | `apt install gcc-arm-none-eabi` | `dnf install arm-none-eabi-gcc-cs` | `pacman -S arm-none-eabi-gcc` | `winget install Arm.GnuArmEmbeddedToolchain` |
| AVR builtins | `brew install avr-gcc` (tap) | `apt install gcc-avr avr-libc` | `dnf install avr-gcc` | `pacman -S avr-gcc` | Microchip's toolchain download |
| QEMU | `brew install qemu` | `apt install qemu-user qemu-system-arm` | `dnf install qemu` | `pacman -S qemu-full` | `winget install SoftwareFreedomConservancy.QEMU` |

The table lives in one place in the code, with a test that every target `jacket toolchain` lists has a row.

## `jacket toolchain`

Fetches what no package manager provides, into jacket's cache only (`%LOCALAPPDATA%\jacket\toolchains` on Windows, the user cache folder elsewhere). It never installs system packages.

```sh
jacket toolchain add armv7-unknown-linux-gnueabihf   # a pinned Debian sysroot, sha256-checked
jacket toolchain add thumbv6m-none-eabi              # compiler-rt builtins, built with the installed clang
jacket toolchain list                                # what the cache holds
jacket toolchain remove armv7-unknown-linux-gnueabihf
```

- **Sysroots.** A pinned set of Debian `libc6-dev` packages for the architecture, unpacked to a sysroot. Needed for cross Linux from macOS and Windows; on Linux the distribution's cross packages come first.
- **Builtins.** compiler-rt's source for the pinned LLVM tag is downloaded and its builtins are compiled for the triple with the installed `clang`, directly from the source list, without cmake, ninja, git or bash, so it works on Windows. The AVR core comes from the board, not a fixed `-mmcu`. This replaces `tools/embedded/build-compiler-rt.sh`, which CI then calls through the same code.
- Every download is pinned and checked by sha256. An entry records its source, so `list` shows where each came from.

### What it replaces

Every place that finds a tool today folds into the one lookup. There were five separate `PATH` walkers (`jaic-native/src/linker.rs`, `wasm.rs`, `freestanding.rs`, `jaic/src/toolchain/discover.rs`, `test-support/src/toolchain.rs`) and six separate install hints.

| Today | After |
| --- | --- |
| `jacket targets` | `jacket toolchain` with no subcommand: every target with its triple, board, linker, libraries and runner, found or missing; `jacket toolchain check NAME...` exits non-zero when something is missing |
| `tools/embedded/build-compiler-rt.sh`, and `JAIC_COMPILER_RT` naming a folder of builds | `jacket toolchain add <triple>`; `JAIC_COMPILER_RT` stays as the one-file override |
| `tools/dev/cross_link.sh` (a `JAIC_LINKER` that links inside the cross container) | the lookup order; the container stays only to run programs under qemu-user (`tools/dev/qemu_run.sh`) |
| `JAIC_LLD`, `JAIC_WASM_LD` | removed; `-linker` / `JAIC_LINKER` is the one override for whichever linker the target uses |
| `JAIC_SANITIZER_CC` | removed; the sanitizer driver is the build's own LLVM `clang`, else `clang` on `PATH`, else the error note |
| `JAIC_CROSS_LIBS` (cross-built third-party libraries for MinGW) | removed; cross builds read the same native-library roots as host builds, from the folder named for the target (`windows-x64-mingw`) |
| `JAIC_REQUIRE_TOOLCHAIN` in tests | removed; tests find tools through the same lookup, a missing one skips (fails under `CI`), and CI runs `jacket toolchain check` first so a missing tool names itself |
| `jacket::target::installed` (a `PATH`-only check of the runner) | the lookup, which also searches Homebrew's prefix and `Program Files\qemu` |
| The install hints in `linker_not_run`, `LLD_INSTALL`, `WASM_LD_INSTALL`, `compiler_rt_install`, the MinGW and MSVC messages, `test-support`'s `install_hint` | the one package table |
| 32-bit and cross builds of the native libraries (`native-libs/<os>-<arch>`), built by hand | `jacket toolchain add <triple>` fetches the release's native libraries for that target, once releases publish them per target |

Kept: `JAIC_LINKER` and `JAIC_AR` (overrides for CI and scripts), `JAIC_NATIVE_LIBS`, and `JAIC_CROSS_ARCH` for `tools/dev/qemu_run.sh`, which runs programs in a container and is not part of the lookup.

`jacket upgrade` and `jacket version` stay: they are about jacket's own install, not a target's tools.

## Status lines

Cross and bare-metal builds print where each tool came from, in the same bold, right-aligned style as `Compiling` and `Finished`. Host builds print them only with `-v`. An override is always shown and labelled. `-quiet` hides them; plain mode (CI, `NO_COLOR`) keeps them without colour.

```
   Compiling hello (armv7-unknown-linux-gnueabihf)
      Linker /opt/homebrew/bin/ld.lld 21.1.8
     Sysroot ~/Library/Caches/jacket/toolchains/armv7-unknown-linux-gnueabihf (jacket toolchain)
    Finished release in 1.84s
     Running qemu-arm -L ~/Library/Caches/jacket/toolchains/armv7-unknown-linux-gnueabihf ./hello

   Compiling blink (thumbv7em-none-eabihf, board Mps2_An386)
      Linker /usr/bin/ld.lld 19.1.7
    Builtins libgcc.a from arm-none-eabi-gcc 14.2
      Script board Mps2_An386
    Finished debug in 0.92s
```

`jacket toolchain add` shows a live `Building compiler-rt 21.1.8 for <triple> (112/158)` line while it builds, then `Builtins compiler-rt 21.1.8 (built in 14.2s)`. Later builds say `(cached)`. Home folders are shortened to `~` on screen only.

`jacket toolchain` uses the same lookup, so its listing and a build always agree.

## `jacket.toml`

```toml
[target.pi]
triple = "armv7-unknown-linux-gnueabihf"
runner = "qemu-arm -L {sysroot} {exe}"
```

- **Allowed:** `triple`, `board`, and `runner` as tool names with `{exe}`, `{args}` and `{sysroot}`, which jacket fills in.
- **Link settings** are written by meaning (`gc_sections`, `stack_size`, `entry`), not as linker flags, because `-Wl,...` works only through a C driver. Raw arguments, if still needed, use `ld.lld`'s spelling and are translated for the linker in use.
- **Flashing** takes the port from `--port` or `JACKET_PORT`, never from the project file.

## Design

One new crate, `jaic-toolchain`, depends on `jaic` (for `Target`) and `jacket-config`. `jaic-native` uses it to link, `jacket` uses it to list, check and fill the cache, and `test-support` uses it to find tools. It reads the machine through a `Host` value (the OS, `PATH`, environment, file checks, and a probe that runs a tool and returns its output). The real `Host` uses the process; tests build one from a table, so package-manager detection, probing and every error note are tested without the real tools.

- `which`: the one `PATH` search (`PATHEXT`, `.exe`, versioned names such as `ld.lld-23`, Homebrew kegs, Debian's `/usr/lib/llvm-N/bin`, `Program Files\LLVM`).
- `PackageManager::detect(host)`: `brew`, `apt`, `dnf`, `pacman` or `winget`, from the OS and the tools on `PATH`.
- `Need`: one variant per piece (`Lld`, `WasmLd`, `CrossLinuxLibc(arch)`, `MingwImportLibs`, `ArmEmbedded`, `Avr`, `Qemu(kind)`, `Wasmtime`, ...) with its `install(manager)` text. A test checks that every target `jacket toolchain` lists has a row.
- `find_linker(host, target)`, `find_sysroot`, `find_builtins`, `find_runner`: each returns `Found { path, version, origin }` or `Missing { need }`, and the origin is what the status line prints.

A tool is asked, not judged by its name: `ld.lld --version`, `cc -dumpmachine`, `clang --version`.

## Milestones

1. **T1, lookup and notes.** The `jaic-toolchain` crate, the package table and error notes, the lookup for cross Linux, MinGW, wasm and freestanding links, `-linker`/`-sysroot`/`-compiler_rt` and their variables, and `jacket toolchain` (replacing `jacket targets`). Removes `JAIC_LLD`, `JAIC_WASM_LD`, `JAIC_SANITIZER_CC`, `JAIC_CROSS_LIBS`, `JAIC_REQUIRE_TOOLCHAIN` and `cross_link.sh`.
2. **T2, status lines.** `Linker`, `Sysroot`, `Builtins` and `Script` lines in jaic's progress output (`crates/jaic/src/progress.rs`), shown for cross and bare-metal builds and with `-verbose` for host builds.
3. **T3, system builtins.** Link `libgcc.a` from an installed GNU embedded toolchain for Cortex-M and AVR.
4. **T4, `jacket toolchain add`.** Cached compiler-rt builds and Debian sysroots, `list` and `remove`; `build-compiler-rt.sh` goes away.
5. **T5, `jacket.toml` link settings.** `stack_size`, `entry` and `gc_sections` under `[target.NAME]`, rendered per linker, raw `ld.lld` spellings translated, and `{sysroot}` in runners. `jacket.toml` has no machine-specific keys today; a test keeps it that way.

## Files

- Linker choice: `crates/jaic-native/src/` (`freestanding.rs`, `apple_ld.rs` and the hosted link driver).
- Builtins lookup: `crates/jaic/src/toolchain/freestanding.rs`.
- Targets and runners: `crates/jacket/src/target.rs`.
- Related: [embedded and freestanding targets](embedded.md), [linking](native-linking.md), [targets](../compiler/targets.md), [N-bit and embedded plan](n-bit-and-embedded.md).
