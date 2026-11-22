# 32-bit and embedded targets

Jacket doesn't stop at 64-bit desktops. The same program can build for 32-bit WebAssembly, 32-bit x86 and ARM Linux, big-endian Linux, and microcontrollers with no operating system at all. This page gets you from nothing to a running program on each, then points you at the deeper docs.

## What you can build for

| Target | `--target` | Pointers | Run it with |
| --- | --- | --- | --- |
| WebAssembly (WASI) | `wasm32-wasi` | 4 bytes | `wasmtime`, node |
| x86 Linux | `i686-unknown-linux-gnu` | 4 bytes | the machine itself, or `qemu-i386` |
| 32-bit ARM Linux (Raspberry Pi and friends) | `armv7-unknown-linux-gnueabihf` | 4 bytes | the machine itself, or `qemu-arm` |
| IBM Z Linux, big-endian | `s390x-unknown-linux-gnu` | 8 bytes | `qemu-s390x` |
| POWER Linux, big-endian | `powerpc64-unknown-linux-gnu` | 8 bytes | `qemu-ppc64` |
| Cortex-M4F board | `Mps2_An386` | 4 bytes | `qemu-system-arm` |
| Cortex-M33 board | `Mps2_An505` | 4 bytes | `qemu-system-arm` |
| AArch64 bare metal | `Qemu_Virt_Aarch64` | 8 bytes | `qemu-system-aarch64` |

## Start here: a 32-bit program in WebAssembly

WebAssembly needs no cross toolchain, so it's the quickest way to see a 32-bit build. Install [wasmtime](https://wasmtime.dev), then:

```jai
#import "Basic";

main :: () {
    print("Hello from a %-bit pointer!\n", size_of(*void) * 8);
}
```

```sh
jacket build hello.jai --target wasm32-wasi -exe hello.wasm
wasmtime hello.wasm
```

```text
Hello from a 32-bit pointer!
```

Nothing in the program changed. `print`, strings, arrays and allocators all follow the target's pointer size on their own.

## 32-bit Linux

Linux targets link against the target's C library, so you need its cross compiler and headers. On Debian or Ubuntu, for ARM:

```sh
sudo apt install gcc-arm-linux-gnueabihf libc6-dev-armhf-cross qemu-user
```

Then point Jacket at that compiler for linking, build, and run under QEMU (or copy the program to a Raspberry Pi and run it there):

```sh
JAIC_LINKER=arm-linux-gnueabihf-gcc jacket build hello.jai --target armv7-unknown-linux-gnueabihf
qemu-arm -L /usr/arm-linux-gnueabihf ./hello
```

For x86 on a 64-bit x86 Linux machine it's simpler: `sudo apt install gcc-multilib`, then `jacket build hello.jai --target i686-unknown-linux-gnu`, and the program runs directly.

Most of the standard library already works on these targets, but not all of it yet. The modules that pass their tests are listed per target in `tools/dev/` (`i686-roots.txt`, `armv7-roots.txt` and so on).

## Bare metal: a program with no operating system

A board module gives a program what an OS normally would: startup code, a memory map, a console and a heap. You write `main` as usual, plus one small function the board calls on reset.

You need:

- `ld.lld` (Homebrew `lld`, or Debian `lld`).
- QEMU (`qemu-system-arm` for the Cortex-M boards).
- The compiler-rt builtins for the chip, which cover things like 64-bit division and `float64` math on a 32-bit core. `tools/embedded/build-compiler-rt.sh` builds them in a minute or two, and prints the `JAIC_COMPILER_RT=` line to set. It needs LLVM's `clang` and `llvm-ar` (`brew install llvm`, or `apt install clang llvm`), `cmake` and `ninja`.

```sh
tools/embedded/build-compiler-rt.sh thumbv7em-none-eabihf
export JAIC_COMPILER_RT=$PWD/target/compiler-rt
```

Then the program:

```jai
#import "Basic";
Board :: #import "Extensions/Embedded/Boards/Mps2_An386";

main :: () {
    print("Hello from a Cortex-M4!\n");
}

#program_export "jai_start"
reset :: () #c_call #no_context { Board.start(main); }
```

```sh
jacket run hello.jai --target Mps2_An386
```

`jacket run` builds with the board, then starts it in QEMU. `print` goes out through the emulator's console, and when `main` returns the board exits with status 0. A failed assert exits with 1.

To give the target a short name, or to run on real hardware, add it to `jacket.toml`:

```toml
[target.dev]
board = "Mps2_An386"

[target.pico]
triple = "thumbv6m-none-eabi"
runner = "flash {exe}"     # whatever puts the program on your board
```

`jacket targets` lists every name `--target` accepts, and whether its runner is installed.

## Tests on the target

`jacket test` takes the same `--target`, and runs each test through that target's runner:

```sh
jacket test --target Mps2_An386 --path tests/firmware
```

On a board, only unit tests run, and everything they import has to build without an OS. Keep them in a folder of their own.

## Writing code that works on every target

- If you need the pointer size, read `size_of(*void)`. Don't assume 8.
- Put OS-only code behind `#if OS != .NONE`. Calling it on a board is then a compile error, not a crash.
- Turn on the lint rules made for this, in `jacket.toml`:

  ```toml
  [lint.rules]
  narrow_pointers = "warn"   # pointer tricks that break below 64 bits
  freestanding = "warn"      # heap use, OS-only imports and deep stacks on a board
  ```

## Where to go next

- [Extensions/Embedded](../extensions/embedded.md): the runtime, memory maps, `volatile_load` and `volatile_store`, interrupts, flash images (`.bin`, `.hex`, `.uf2`) and how to add your own board.
- [Embedded and freestanding targets](../native/embedded.md): how linking works with no OS, and every setting.
- [WebAssembly](../native/wasm-target.md): imports, exports and running in a browser.
- [Targets](../compiler/targets.md): the exact layout and calling convention of each target.
- [Tests on a board](../tools/jacket-test.md#tests-on-a-board) and [jacket targets](../tools/jacket.md#targets).
