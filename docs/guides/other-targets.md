# Other targets

Jacket builds for 32-bit WebAssembly, 32-bit x86 and ARM Linux, big-endian Linux, and microcontrollers with no operating system, down to the 8-bit AVR with 16-bit pointers. This page builds and runs a program on each, then links to the reference docs.

> [!WARNING]
> These targets and their APIs are experimental and may change in any release.

## What you can build for

| Target | `--target` | Pointers | Run it with |
| --- | --- | --- | --- |
| WebAssembly (WASI) | `wasm32-wasi` | 4 bytes | `wasmtime`, node |
| x86 Linux | `i686-unknown-linux-gnu` | 4 bytes | the machine itself, or `qemu-i386` |
| 32-bit ARM Linux (Raspberry Pi and friends) | `armv7-unknown-linux-gnueabihf` | 4 bytes | the machine itself, or `qemu-arm` |
| IBM Z Linux, big-endian | `s390x-unknown-linux-gnu` | 8 bytes | `qemu-s390x` |
| POWER Linux, big-endian | `powerpc64-unknown-linux-gnu` | 8 bytes | `qemu-ppc64` |
| Arduino Uno (AVR, 8-bit) | `Arduino_Uno` | 2 bytes | `qemu-system-avr` |
| Cortex-M4F board | `Mps2_An386` | 4 bytes | `qemu-system-arm` |
| Cortex-M33 board | `Mps2_An505` | 4 bytes | `qemu-system-arm` |
| AArch64 bare metal | `Qemu_Virt_Aarch64` | 8 bytes | `qemu-system-aarch64` |

The Windows `jaic` can't build for AVR, s390x or PowerPC64, because the LLVM it links has no backend for them ([backends](../tools/llvm-setup.md#backends)).

## Start here: a 32-bit program in WebAssembly

WebAssembly needs no cross toolchain, only a runtime. Install [wasmtime](https://wasmtime.dev), then:

```jai
#import "Basic";

main :: () {
    print("Pointers are % bits\n", size_of(*void) * 8);
}
```

```sh
jacket build hello.jai --target wasm32-wasi -exe hello.wasm
wasmtime hello.wasm
```

```text
Pointers are 32 bits
```

`print`, strings, arrays and allocators follow the target's pointer size.

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

For x86 on 64-bit x86 Linux, install `gcc-multilib`, build with `--target i686-unknown-linux-gnu`, and run the program directly.

Not all of the standard library works on these targets. The modules that pass their tests are listed per target in `tools/dev/` (`i686-roots.txt`, `armv7-roots.txt` and so on).

## Bare metal: a program with no operating system

A board module gives a program what an OS normally would: startup code, a memory map, a console and a heap. You write `main` as usual, plus one small function the board calls on reset.

You need:

- `ld.lld` (Homebrew `lld`, or Debian `lld`).
- QEMU (`qemu-system-arm` for the Cortex-M boards).
- The compiler-rt builtins for the chip, which cover things like 64-bit division and `float64` math on a 32-bit core. A GNU embedded toolchain's `libgcc.a` (`arm-none-eabi-gcc`) is used when installed. Otherwise `jacket toolchain add` compiles them in a minute or two with LLVM's `clang` and `llvm-ar` (`brew install llvm`, or `apt install clang llvm`).

```sh
jacket toolchain add thumbv7em-none-eabihf
```

Then the program:

```jai
#import "Basic";
Board :: #import "Extensions/Embedded/Boards/Mps2_An386";

main :: () {
    print("Hello, world!\n");
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

`jacket toolchain` lists every name `--target` accepts with the linker, libraries and runner it needs, and whether this machine has them.

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
- [Tests on a board](../tools/jacket-test.md#tests-on-a-board) and [jacket toolchain](../tools/jacket.md#targets).
