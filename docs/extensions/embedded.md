# Embedded

`Extensions/Embedded` is what a program with no operating system (`os_target = .NONE`) needs: a runtime to start, memory maps and linker scripts, volatile access, the section and interrupt notes, CPU hints, flash image formats, and boards. [Embedded and freestanding targets](../native/embedded.md) covers the target itself.

> [!WARNING]
> Experimental: this module's API may change in any release.

```jai
#import "Basic";
#import "Extensions/Embedded";
```

## What runs where

Part of the module runs **on the target** (`OS == .NONE`) and part runs **in the metaprogram, on the host**:

| Where | Items |
| --- | --- |
| Target | `init_runtime`, `static_context`, `set_console_hook`, `set_panic_hook`, `set_exit_hook`, `exit`, `set_arguments`, `arguments`, `Heap`, `make_heap`, `heap_allocator` |
| Host | `Memory_Map`, `linker_script`, `link_for`, `write_binary` |
| Both | `volatile_load`, `volatile_store`, the barriers and hints |

## The runtime

`init_runtime()` copies `.data` from its load address, zeroes `.bss` and sets up the static `Context` with `TEMPORARY_STORAGE_SIZE` bytes of temporary storage. Call it first from the reset handler. `static_context()` returns that context for `push_context`.

```jai
reset :: () #c_call #no_context {
    init_runtime();
    set_console_hook(uart_write);
    push_context static_context().* { main(); }
}
```

- `set_console_hook` receives everything `print` and `write_string` write. With none set, output is dropped.
- `set_exit_hook` sets what `exit(status)` does: a board ends the run, for example through semihosting. `set_arguments` gives the program the command line the board was started with, as `arguments()` returns it (`Extensions/Testing` reads it on a board).
- `set_panic_hook` runs when a check or assert fails or the default allocator is used. It must not return. With none set, the reason goes to the console and the program halts.
- The default allocator has no heap and panics. Give the context one: `heap_allocator(*heap)` over `make_heap()` hands out the RAM between `.bss` and the stack in order. It never reuses memory, except that freeing or resizing the newest block gives the space back.

## Memory maps and linker scripts

```jai
#run {
    w := compiler_create_workspace("app");
    options := get_build_options(w);
    options.os_target = .NONE;
    options.cpu_target = .ARM64;
    options.llvm_options.target_system_triple = "aarch64-none-elf";
    options.runtime_support_definitions = .ONLY_INIT;
    link_for(*options, Memory_Map.{ ram = .{ 0x4000_0000, 0x400_0000 } });
    set_build_options(options, w);
    add_build_file("app.jai", w);
}
```

`Memory_Map` has `ram`, `flash` (leave it empty to run from RAM), `stack_size` (default 8 KiB) and `entry` (default `_start`). `link_for` writes `<executable>.ld` (or the `script_path` you pass) and adds `-T` to `additional_linker_arguments`. `linker_script(map)` returns the text.

The script puts `.vector_table`, `.text` and `.rodata` in flash (or RAM), `.data` and `.bss` in RAM, and defines `__data_start`, `__data_end`, `__data_load`, `__bss_start`, `__bss_end`, `__heap_start`, `__heap_end` and `__stack_top`. `.vector_table` is kept even though no code names it.

## Notes and builtins

These are Jacket-only; the official compiler ignores the notes.

- **`@Section(".name")`** after a variable or procedure puts it in that linker section: `table: [4] u32 = .[1, 2, 3, 4]; @Section(".vector_table")`. A variable with a section is always kept, as the linker script is what wants it.
- **`@Module_Asm`** after a constant string adds the string to the module's assembly. A board's reset code and the semihosting call are written this way, since there is no `#asm` for AArch64. The assembly is added once, however it is referenced.
- **`@Interrupt`** after a procedure marks it as entered by hardware. It must be `#c_call #no_context` with no parameters or results, or the compile fails (`interrupt-signature`). On AArch64 the note is checked and nothing more: the vector stub saves registers.
- **`volatile_load(p)` and `volatile_store(p, v)`** read and write a 1, 2, 4 or 8-byte scalar exactly once, in order with other volatile accesses. At compile time the address must be ordinary memory; a device register is an error.
- **Barriers and hints** (AArch64 and 32-bit ARM): `wait_for_interrupt`, `wait_for_event`, `send_event`, `data_sync_barrier`, `data_memory_barrier` and `instruction_barrier`.

Freestanding code is compiled with `+strict-align`: with the MMU off, an unaligned access faults. A board that enables the MMU can pass `-strict-align` in `llvm_options.target_system_features`.

## Flash images

`write_binary(elf_path, base, .BIN | .HEX | .UF2, uf2_family = 0)` reads the loadable segments of a little-endian ELF32 or ELF64 file at their load addresses and writes `base.bin`, `base.hex` and `base.uf2`. There is no `objcopy`. BIN is the bytes from the lowest address, gaps zero. HEX is Intel HEX with a start address record. UF2 is 256-byte blocks; `uf2_family` sets the family id flag. `elf_segments`, `binary_image`, `intel_hex` and `uf2_image` are the pieces.

## Boards

A board lives in `Extensions/Embedded/Boards/<Name>` and holds its memory map and startup code:

| Board | Core | Runs on |
| --- | --- | --- |
| `Arduino_Uno` | ATmega328P, 8-bit AVR | `qemu-system-avr -M uno` |
| `Qemu_Virt_Aarch64` | Cortex-A53, AArch64 | `qemu-system-aarch64 -M virt` |
| `Mps2_An386` | Cortex-M4F, Thumb-2 | `qemu-system-arm -M mps2-an386 -cpu cortex-m4` |
| `Mps2_An505` | Cortex-M33 (secure state), Thumb-2 | `qemu-system-arm -M mps2-an505 -cpu cortex-m33` |

Every board has:

- `configure(*options)` sets the target and runtime. A metaprogram of your own then calls `Embedded.link_for(*options, Board.memory_map)` for the linker script. The board's plugin adds the script to the link line and writes it at `PRE_WRITE_EXECUTABLE`, so `jaic -check` writes none.
- `start(program)` runs from the reset function: it sets up the runtime, a heap, a semihosting console and panic handler, runs `program`, and exits with status 0. A panic exits with status 1.

The program exports `jai_start`, which the board's reset code calls:

```jai
Board :: #import "Extensions/Embedded/Boards/Qemu_Virt_Aarch64";

main :: () { print("hello\n"); }

#program_export "jai_start"
reset :: () #c_call #no_context { Board.start(main); }
```

`crates/jaic-cli/tests/freestanding/qemu_virt` is a complete project.

## How to change it

- A new board: add `Boards/<Name>/module.jai` with a `memory_map`, a `configure` for the host, a `get_plugin` that returns `Embedded.board_plugin("<import path>", configure)`, and a `start` for the target that calls `Embedded.start_semihosted` (or your own startup). Give it string constants `triple` (which `configure` uses) and `runner` (the emulator command, with `{exe}` and `{args}`), and add its module to `BOARD_SOURCES` in `crates/jacket/src/target.rs`. Boards that use another CPU need that CPU's compiler support first.
- A Cortex-M board needs nothing but its memory map and triple: `cortex-m.jai` has the vector table (the stack top, reset, and a fault handler that exits with status 1), the reset code that enables the FPU and calls `jai_start`, and the semihosting trap (`bkpt 0xab`). The core boots from address 0, so the memory map's `flash` region must be there.
- Semihosting (`semihosting.jai`: console, exit, command line) is shared by the boards that use it. A board with a UART sets its own `set_console_hook` instead.
- A new CPU's hints: add them under `#if CPU == ...` in `intrinsics.jai`.
- A new output format: add a flag to `Output_Formats` and a branch in `write_binary`.

## Dependencies

`ld.lld` and, for builtins such as 64-bit division, compiler-rt for the target (see [embedded and freestanding targets](../native/embedded.md); `jacket toolchain add` builds it). The tests also need `clang`, `qemu-system-aarch64` and `qemu-system-arm`.
