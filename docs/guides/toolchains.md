# Toolchains

To build for another OS or a microcontroller, `jaic` needs more than itself: a linker for the target, the target's C library for Linux targets, and the compiler builtins for targets with no OS. Jacket finds these on your machine with no configuration. This page shows what it looks for, how to see what it found, and how to get what's missing.

> [!WARNING]
> Cross and bare-metal targets are experimental and may change in any release.

## What jacket looks for

| Target | Linker | Libraries |
| --- | --- | --- |
| Your own machine | `cc` on macOS and Linux, `link.exe` on Windows | the system's |
| Linux on another CPU | `clang` with `ld.lld`, or a GNU cross compiler such as `arm-linux-gnueabihf-gcc` | the system's cross packages, else a sysroot from `jacket toolchain add` |
| Windows from macOS or Linux | `lld-link`, or the MinGW compiler | MinGW's import libraries |
| WebAssembly | `wasm-ld` | none |
| Boards and bare metal | `ld.lld` | compiler builtins and the board's linker script |

Each tool is asked what it can build for (`ld.lld --version`, `cc -dumpmachine`), so it is found whatever its install path. Jacket ships no linker and never installs system packages.

The compiler builtins cover operations a small chip has no instruction for, such as 64-bit division or `float64` math on a 32-bit core. Jacket uses LLVM's builtins when your LLVM has them for the target, else `libgcc.a` from an installed `arm-none-eabi-gcc` or `avr-gcc`, else ones that `jacket toolchain add` built.

## See what was found

Cross and bare-metal builds print where each tool came from. Here the [bare-metal program](other-targets.md#bare-metal-a-program-with-no-operating-system) from Other targets, built for a Cortex-M4F board:

```console cwd=../examples/targets/board/blink cache
$ jacket build hello.jai --target Mps2_An386
   Compiling blink (hello.jai)
      Linker /opt/homebrew/opt/lld/bin/ld.lld 23.1.6
    Builtins compiler-rt 21.1.8 (cached)
      Script hello.ld
    Finished debug [unoptimized] in 0.16s
```

Builds for your own machine print these lines only with `-v`.

`jacket toolchain` lists every target you can build for, with what each needs and whether this machine has it:

```console cwd=../examples/targets/board/blink cache
$ jacket toolchain
TARGET             FROM      TRIPLE                     LINKER                                   LIBRARIES                    RUNNER
host               -         aarch64-apple-darwin       /usr/bin/cc 21.0.0                       -                            -
Arduino_Uno        built in  avr-none-elf               /opt/homebrew/opt/lld/bin/ld.lld 23.1.6  missing                      /opt/homebrew/bin/qemu-system-avr 11.1.3
Qemu_Virt_Aarch64  built in  aarch64-none-elf           /opt/homebrew/opt/lld/bin/ld.lld 23.1.6  missing                      /opt/homebrew/bin/qemu-system-aarch64 11.1.3
Mps2_An386         built in  thumbv7em-none-eabihf      /opt/homebrew/opt/lld/bin/ld.lld 23.1.6  compiler-rt 21.1.8 (cached)  /opt/homebrew/bin/qemu-system-arm 11.1.3
Mps2_An505         built in  thumbv8m.main-none-eabihf  /opt/homebrew/opt/lld/bin/ld.lld 23.1.6  missing                      /opt/homebrew/bin/qemu-system-arm 11.1.3

Arduino_Uno: no builtins for avr-none-elf
note: run `jacket toolchain add avr-none-elf` to build its builtins, or install the AVR GNU toolchain with `brew install osx-cross/avr/avr-gcc`

Qemu_Virt_Aarch64: no builtins for aarch64-none-elf
note: run `jacket toolchain add aarch64-none-elf` to build its builtins

Mps2_An505: no builtins for thumbv8m.main-none-eabihf
note: run `jacket toolchain add thumbv8m.main-none-eabihf` to build its builtins, or install the GNU Arm embedded toolchain with `brew install --cask gcc-arm-embedded`
```

## When something is missing

The build stops with a note that says how to get it, for the package manager it finds (`brew`, `apt`, `dnf`, `pacman` or `winget`). Run the command, then build again.

Two things have no package on most systems: a Linux C library for another CPU (on macOS and Windows), and compiler builtins for a chip. For those the note names a `jacket toolchain add` command:

```console cwd=../examples/targets/linux/hello cache
$ jacket build hello.jai --target aarch64-unknown-linux-gnu
   Compiling hello (hello.jai)
error[cli-failure]: no Linux C library for aarch64-unknown-linux-gnu
note: run `jacket toolchain add aarch64-unknown-linux-gnu` to fetch its C library
```

`jacket toolchain check` reports the same for any target without building, here an Arduino Uno with no AVR builtins installed:

```console cwd=../examples/targets/board/blink cache
$ jacket toolchain check Arduino_Uno
TARGET       FROM      TRIPLE        LINKER                                   LIBRARIES  RUNNER
Arduino_Uno  built in  avr-none-elf  /opt/homebrew/opt/lld/bin/ld.lld 23.1.6  missing    /opt/homebrew/bin/qemu-system-avr 11.1.3

Arduino_Uno: no builtins for avr-none-elf
note: run `jacket toolchain add avr-none-elf` to build its builtins, or install the AVR GNU toolchain with `brew install osx-cross/avr/avr-gcc`
```

## `jacket toolchain add`

```console cwd=../examples/targets/board/blink cache
$ jacket toolchain add armv7-unknown-linux-gnueabihf thumbv7em-none-eabihf
     Sysroot ~/Library/Caches/jacket/toolchains/armv7-unknown-linux-gnueabihf/sysroot (cached)
    Builtins compiler-rt 21.1.8 (cached)
```

- **A Linux target** gets a sysroot: a fixed set of Debian packages (the C library, its headers and the kernel headers) downloaded and unpacked into jacket's cache.
- **A target with no OS** gets the compiler builtins, compiled from a fixed LLVM release with your installed `clang` and `llvm-ar` (`brew install llvm`, or `apt install clang llvm`). It takes a minute or two; later runs say `(cached)`.

An `add` for something the cache already holds says `(cached)` and downloads nothing. This is the only jacket command that downloads anything. Builds never do. Every download is a pinned version checked against its SHA-256.

```console cwd=../examples/targets/board/blink cache
$ jacket toolchain list
~/Library/Caches/jacket/toolchains
TARGET                         SYSROOT   BUILTINS
armv7-unknown-linux-gnueabihf  yes       -
i686-unknown-linux-gnu         yes       -
thumbv7em-none-eabihf          -         compiler-rt 21.1.8
```

`jacket toolchain remove TARGET` deletes one target's entry, and `--all` empties the cache.

The cache is `jacket/toolchains` in your user cache folder (`~/Library/Caches` on macOS, `$XDG_CACHE_HOME` or `~/.cache` on Linux, `%LOCALAPPDATA%` on Windows).

## In CI

`jacket toolchain check` exits with a failure, and the same notes, when something a target needs is missing. `--tool` adds a program such as an emulator:

```console cwd=../examples/targets/board/blink cache
$ jacket toolchain check thumbv7em-none-eabihf --tool qemu-system-arm
TARGET                 FROM      TRIPLE                 LINKER                                   LIBRARIES                    RUNNER
thumbv7em-none-eabihf  built in  thumbv7em-none-eabihf  /opt/homebrew/opt/lld/bin/ld.lld 23.1.6  compiler-rt 21.1.8 (cached)  -
qemu-system-arm: /opt/homebrew/bin/qemu-system-arm
```

## `jacket.toml` stays portable

`jacket.toml` names targets, never a machine's tools. Anyone can build the same project with the tools they have:

```toml
[target.pi]
triple = "armv7-unknown-linux-gnueabihf"
runner = "qemu-arm -L {sysroot} {exe}"
stack-size = 8192
gc-sections = true
```

- `stack-size`, `entry` and `gc-sections` are written once, and jacket passes them in the form the chosen linker expects.
- `{sysroot}` in a runner is the sysroot jacket found, so the same line works with the system's cross packages or a cached sysroot.

To use a specific tool for one build, use a flag or environment variable instead. These belong in scripts and CI, not in `jacket.toml`:

| Flag | Variable | Picks |
| --- | --- | --- |
| `-linker` | `JAIC_LINKER` | the linker or compiler driver |
| `-sysroot` | `JAIC_SYSROOT` | the sysroot folder for a Linux target |
| `-compiler_rt` | `JAIC_COMPILER_RT` | the builtins archive for a target with no OS |

A path that doesn't exist is an error, never a reason to look elsewhere.

## Where to go next

- [Other targets](other-targets.md): build and run a program on WebAssembly, ARM Linux and a board.
- [Toolchain discovery](../native/toolchain-discovery.md): the full lookup order, how the downloads are pinned, and how to change it.
- [Tools](../reference/tools.md#compiling): the `jacket toolchain` flags.
