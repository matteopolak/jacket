# Plan: N-bit, big-endian and embedded targets

Status: N0 landed (`DataLayout`); N1 (wasm32) landed: the stdlib passes under node and wasmtime. N2 (i686 Linux, glibc) landed: 93 stdlib roots pass (`tools/dev/i686-roots.txt`, run in CI); the rest of the roots need per-CPU socket ABIs and 32-bit builds of the third-party native libraries (see below); musl is not built yet. N3 (s390x) landed in part: it builds, links and runs under qemu-user, and 69 stdlib roots pass on it (`tools/dev/s390x-roots.txt`, run in CI); N4 (armv7 Linux, hard-float) landed: it builds, links and runs under `qemu-arm` (`tools/dev/armv7-roots.txt`, run in CI). N5 (AVR) landed in part: the target, 16-bit layout, register ABI and the 48 KiB interpreter window exist, but the prelude does not yet fit that window and there is no linker, so nothing runs end to end until the embedded milestone E5; AVR code pointers (program memory) are not modelled. powerpc64 is not started. The rest is plan. Each milestone updates this page and the docs it names when it lands.

Jacket builds for 64-bit little-endian machines with an OS: x86-64 and arm64 on macOS, Linux and Windows, plus wasm64. This plan adds:

- **N-bit:** 16-, 32- and 64-bit pointers, little- and big-endian byte order, and per-target alignment, for every part of the compiler.
- **Embedded:** freestanding programs with no OS or libc, built for microcontrollers and bare-metal boards and tested in emulators.

No milestone may special-case a target, stub out a failing path or loosen a test. Breaking our own APIs is fine. Where a milestone turns out bigger than written, land its correct core and log the rest in `TODO.md`.

## What the language fixes and what the target decides

The specs fix these on every target, and they stay that way:

- `int` is `s64`. Integer literals default to `s64`.
- `string` and `[] T` are a `count: s64` and a data pointer. `[..] T` adds `allocated: s64` and an `Allocator`.
- `CPU_Tag` has only `X64`, `ARM64` and `CUSTOM`. `Operating_System_Tag` has `NONE`.

The target decides everything else: pointer size and alignment, byte order, the alignment of each scalar, the C ABI, C type widths (`long`, `size_t`, `long double`), and whether code and data share an address space.

So `size_of(*void)` is 2, 4 or 8. `size_of(string)` is `8 + pointer size` plus padding, for example 12 on i686, 16 on armv7 and 10 on AVR. `size_of(Type)`, `size_of(Any)` and `size_of(Allocator)` follow the pointer size. Code that needs the pointer size reads `size_of(*void)`. Nothing new is added to the language for it.

## Part 1: data layout per target

### The model

`crates/jaic/src/layout.rs` becomes a `DataLayout` value, computed from `Target` and stored once in `sema::Options`.

```rust
pub struct DataLayout {
    pub pointer_size: u8,          // 2, 4 or 8
    pub pointer_align: u8,
    pub code_pointer_size: u8,     // AVR: program-memory addresses
    pub endian: Endian,            // Little | Big
    pub int_align: [u8; 4],        // s8, s16, s32, s64 (i686 SysV: s64 aligns to 4; AVR: all 1)
    pub float_align: [u8; 2],      // float32, float64
    pub stack_align: u8,
    pub c: CModel,                 // int, long, long long, size_t, long double format
}
```

The model works like this:

- **Who decides:** `Target::data_layout()` is the only place a layout is chosen.
- **Builtin layouts:** the tables of offsets and sizes for `string`, views, `[..]`, `Any`, `Allocator` and `Context_Base` become one `BuiltinLayouts` struct per target.
- **No constant:** `POINTER_SIZE` is deleted outright, not deprecated, so every use becomes a compile error and gets fixed.
- **IR and backends:** `ir::Ty::size` and `align` take a `&DataLayout`. `Ty::Ptr` means a target pointer, never "8 bytes". `F80` is 12 bytes on i686 and 16 on x86-64.
- **One reader:** bytes of program data are read and written only through `DataLayout`, with `read_int(bytes, size)`, `write_ptr(...)` and so on, which respect width and byte order.
  - `clippy.toml` `disallowed-methods` bans `u{16,32,64}::{from,to}_{le,ne}_bytes` in `crates/jaic` and `crates/jaic-llvm`.
  - The allowed exceptions are the layout module, plus host-only code that reads the host's own memory (`interp/native`, `interp/threads`, `stack_trace.rs`, `record_build.rs`). Each gets an `#[allow]` with a one-line reason.
  - There are about 94 call sites today. Each is either target data, which moves to `DataLayout`, or host data, which gets the allow.
- **Checked against LLVM:** a test builds an LLVM `TargetMachine` for every supported triple. It checks that `DataLayout` agrees with LLVM's data layout on the size and alignment of `i8` through `i64`, `ptr`, `f32` and `f64`, and on byte order.

### Compile-time code

Two kinds of compile-time code exist, as described in [compile-time code on the host](../compiler/compile-time-host-execution.md):

1. **A plain `#run`** runs in the host twin, which has the host layout. `host_import.rs` today refuses a type whose layout differs between host and target. It changes to convert by type instead:
   - It walks both compilers' layouts of the same declaration, field by field.
   - Integers and floats are rewritten in the target's width position and byte order.
   - Pointers into frozen data are relocated as now. Interpreter-only addresses become null, as now.
   - Padding is recomputed from the target layout.
   - OS-dependent members are still an error.
2. **Code that inspects the target program** (`#no_reset`, `Type_Info`, `Code`, `#modify`, `add_global_data`) runs on the target compiler in the **target's data model**. Its results then need no translation. That requires the interpreter changes below.

#### The interpreter in the target's data model

Interpreter memory is real host memory today, and program addresses are host addresses. For a target whose pointers are narrower than the host's:

- **An address window.** The interpreter reserves one contiguous host region as the program's address space: up to 4 GiB for 32-bit targets, and 64 KiB data plus 64 KiB code for AVR.
  - Every interpreter allocation for that program comes from the window: stack, globals, `Type_Info` tables and heap.
  - A program address is the offset of the host address from the window base.
  - When the target width equals the host's, the window is the whole address space and the offset is 0, so 64-bit targets keep today's code path and speed.
- **Byte order.** For a target whose byte order differs from the host's, `code::build` emits byte-swapping load and store ops. Targets with the host's byte order emit the same ops as today.
- **Foreign calls.** A narrow or byte-swapped program cannot call host C code directly: host pointers don't fit, and host structs have the wrong layout. Such programs reach C through the `Host` shim, which the interpreter already uses for wasm.
  - The shim implements a fixed set of calls inside the window: memory allocation, `memcpy` and friends, writes to stdout and stderr, time, and `abort`.
  - Any other foreign call made at compile time is a compile error naming the procedure and the target, with the hint to do that work in a plain `#run` (on the host).
  - None of this is a fallback. The same program always gets the same answer.
- **The browser engine** already runs the interpreter on a 32-bit host. It now also runs 32-bit targets natively (window offset 0) and 64-bit targets through the 64-bit path. Both are tested.

### Emitting data

`sema/freeze.rs`, `sema/typeinfo/build.rs`, `sema/runtime_info.rs` and `sema/consteval.rs` write global bytes through `DataLayout`.

Relocations carry the pointer width. A union word that might hold a pointer is resolved by the union's static type, never by guessing from the value.

### The C ABI

`crates/jaic/src/abi.rs` grows an `Arch` for each new ABI, each with tests copied from the published ABI documents, not from another compiler's output:

| Arch | Used by |
|---|---|
| `X86` | i686 System V (cdecl; `s64` aligns to 4 in structs; small structs are returned in memory on Linux) |
| `Win32` | i686 Windows (cdecl and the `#c_call` default; no stdcall unless a spec asks for it) |
| `Arm32` | AAPCS32, soft-float and hard-float (VFP) variants |
| `Riscv32`, `Riscv64` | ILP32/ILP32F/ILP32D and LP64/LP64D |
| `PowerPc64Be` | ELFv1 (big-endian) |
| `S390x` | z/Architecture ELF (big-endian, 64-bit) |
| `Avr` | avr-gcc ABI (16-bit) |
| `Wasm32` | Clang's basic C ABI, as for wasm64 |

`docs/native/c-abi.md` gets a row per arch.

### Targets

- **CPUs and triples:** `TargetCpu` gains `X86`, `Arm`, `Riscv32`, `Riscv64`, `PowerPc64`, `S390x` and `Avr`, and `Target::parse` accepts their triples. `memory32` goes away, since `DataLayout` says it.
- **CPU tags:** each new CPU maps to `CPU_Tag.CUSTOM`, because the spec enum is fixed.
- **`Extensions/Target`:** a new module with constants for what code can't infer: `ARCHITECTURE: Architecture`, `BIG_ENDIAN: bool` and `HAS_FPU: bool`. They come from compiler builtin keys (`builtin_keys.rs`) and work in `#if`.
- **LLVM:** `jaic-llvm` initializes the ARM, RISC-V, PowerPC, SystemZ and AVR targets, and the inkwell features in the root `Cargo.toml` enable them. The release LLVM builds must include these targets. Check each install script and fail the build with a clear message when one is missing; never silently drop a target.

### The standard library

Most of the stdlib is target-neutral Jai already. The parts that aren't:

- **C type widths.** Bindings that write `s64`/`u64` for C `long`, `size_t`, `ssize_t`, `ptrdiff_t` or `intptr_t` are wrong on 32-bit targets. Add stdlib-internal aliases set from the target (`c_long`, `c_size`, ...) in `stdlib/internal`, and change the generators to emit them:
  - `Bindings_Generator`
  - the `socket-abi-*.jai` and other ABI generators
  - the Linux syscall tables

  Hand-written bindings get fixed module by module, each with a test that `size_of` of the record matches the C header for that target.
- **Per-(OS, CPU) ABI files.** The generated ABI files move from per-OS to per-(OS, CPU), for example `socket-abi-linux-x86.jai`. 32-bit Linux uses the 64-bit `time_t` ABI (the `*_time64` syscalls), so years past 2038 work.
- **Code that depends on pointer size** (hash mixing, `Bit_Array` words, allocator headers, `Atomics`) reads `size_of(*void)` or `size_of(int)` and gets a test on a 32-bit target.
- **`#asm` and `Machine_X64`** stay x86-64 only. Using them for another CPU is a compile error, as on arm64 today.

### Milestones

Each milestone is a series of one-change commits, starting with a failing test.

1. **N0, the refactor.** Add `DataLayout`, delete `POINTER_SIZE`, thread the layout everywhere, add the clippy ban and the LLVM agreement test.
   - No target is added yet. Done means every existing test, snapshot and benchmark output is byte-identical, and `tools/bench` compile times show no regression beyond noise.
2. **N1, wasm32.** `wasm32-wasi` and `wasm32-unknown-unknown` build and run.
   - This adds the interpreter address window, the 32-bit `freeze`/`Type_Info` work and `host_import` conversion.
   - Done means `jacket test` over the stdlib with `-target wasm32-wasi` passes under wasmtime and node, and the jaic error "jaic builds wasm64 only" is deleted.
3. **N2, i686 Linux.** `i686-linux-gnu` and `i686-linux-musl`.
   - Adds the `X86` ABI, 32-bit Linux syscall and ABI files, and the stdlib C-width aliases.
   - Done means the stdlib runtime tests run natively on the x64 Linux CI runner with multilib, plus a new CI leg.
   - Landed for glibc: the CI leg runs the roots in `tools/dev/i686-roots.txt`. Remaining gaps: musl; a per-CPU Linux socket ABI (`internal/net/socket-abi-linux.jai` is 64-bit only, so Socket, Http, Grpc, WebSocket and Tls fail); 32-bit builds of the third-party native libraries (X11, mbedtls, freetype, lz4, stb); and the `Syscall` intrinsic, which is x86-64 only.
   - 64-bit atomics need 8-byte alignment but i686 aligns `s64` to 4 inside structs: stack slots are raised to 8, and a struct member that is an atomic target takes `#align 8`.
4. **N3, big-endian.** `s390x-linux-gnu`, which is 64-bit, so byte order is tested alone.
   - Adds byte-swapping interpreter ops and big-endian emission.
   - Done means the stdlib runtime tests pass under `qemu-s390x` in CI. Then `powerpc64-linux-gnu`, under `qemu-ppc64`.
   - Landed for s390x: `TargetCpu::S390x`, the z/Architecture ABI, the SystemZ backend, per-CPU POSIX declarations, and an interpreter that swaps bytes on loads and stores and runs on the emulated C library. Remaining gaps: powerpc64; fibers (no s390x switch routine); the crash runtime; the sandbox's hard-coded record layouts (`stat`, `dirent`) are little-endian; compile-time atomics on big-endian memory; the same Socket ABI and third-party native library gaps as i686.
5. **N4, armv7 Linux.** `armv7-linux-gnueabihf` with AAPCS32 hard-float, under `qemu-arm`. This covers 32-bit and big-endian ARM ABI code before embedded work uses it.
6. **N5, 16-bit.** AVR data layout, ABI and the 64 KiB interpreter window. Tested in the embedded milestones (E5), since AVR has no OS.

## Part 2: embedded

### Freestanding targets

`TargetOs::None` maps to `Operating_System_Tag.NONE`. It takes triples such as `aarch64-none-elf`, `thumbv7em-none-eabihf`, `thumbv8m.main-none-eabihf`, `riscv32imac-unknown-none-elf` and `avr-none`. A freestanding target has:

- **No libc, no dynamic linking and no system entry point.** `#foreign` declarations bind to symbols in other linked objects or the linker script. A `#library` naming a shared library is a compile error.
- **No threads and no TLS.** `Context` lives in a static, and thread-local data becomes plain globals. `Thread` and `Process` are compile errors when imported.
- **Linking with `ld.lld` only,** with a linker script, writing an ELF file. If `ld.lld` isn't installed, the build fails with an install hint. It never falls back to another linker.
- **Builtins from compiler-rt** (64-bit division, soft float) for the exact target, located by the toolchain module. If it's missing, the build fails with an install hint.

### The runtime

`Runtime_Support` gets an `OS == .NONE` variant:

- **Startup.** With `runtime_support_definitions = .OMIT`, the program supplies its own reset handler. With `.ONLY_INIT`, the runtime provides `init_runtime()`, which copies `.data` from flash, zeroes `.bss` and sets up the static `Context`, for the program to call from reset. `.AUTO` means `.ONLY_INIT` on `NONE`.
- **Allocation.** The default allocator has no heap. Allocating through it calls the panic hook with a message saying the board must provide a heap. `Extensions/Embedded` provides heap allocators over a linker-defined region (`__heap_start`/`__heap_end`), and a board sets one in its context.
- **Temporary storage** uses `Build_Options.temporary_storage_size`, which may be 0, with no overflow page from the heap unless a heap exists.
- **Panics and asserts** call `Embedded.panic_hook`. The default hook writes the message to the console if there is one, then halts in a breakpoint loop.
- **Console.** `print` and `write_string` go through a console hook that the board sets (UART, semihosting or none). With none set, output is dropped, and that's documented, not a fallback.
- **`Basic` builds on `NONE`.** Each procedure that needs an OS (files, time, environment, process) is behind `#if OS != .NONE`. Calling it on `NONE` is a compile error naming the procedure, never a stub that returns an error at run time.

### Language support, all in `Extensions/Embedded`

These are jaic-only. The official compiler ignores unknown notes.

- **`@Section(".name")`** on a global or procedure places it in that section. The linker script decides where the section goes, so a vector table is a global with `@Section(".vector_table")`.
- **`volatile_load(p)` and `volatile_store(p, v)`** are compiler builtins that lower to volatile loads and stores. In the interpreter, an address outside the window is a compile-time error (MMIO can't run at compile time).
- **`@Interrupt`** on a `#c_call #no_context` procedure emits the arch's interrupt-entry ABI (LLVM's `interrupt` attribute on RISC-V and AVR). On Cortex-M it's checked and otherwise a no-op, because the hardware already follows AAPCS. `@Interrupt` on any other kind of procedure is a compile error.
- **Barriers and hints** (`wfi`, `dsb`, `isb`, `cpsid`/`cpsie`, `fence`) go through `#intrinsic "llvm.<name>"`, already used for wasm, extended to each arch's intrinsics.
- **Memory maps.** `Embedded.Memory_Map` describes flash and RAM regions. `Embedded.link_for(*options, map)` writes the linker script at compile time and adds `-T` to the workspace's link line. A board passes its own map.
- **Output formats.** `Embedded.write_binary(elf_path, out_path, .BIN | .HEX | .UF2)` is Jai code in the module that reads the ELF, so there's no `objcopy` dependency.

Boards live in `stdlib/Extensions/Embedded/Boards/<Name>`. A board holds its memory map, vector table type, startup code, console and clocks. The first boards are QEMU machines, so CI can run them.

### Testing

`jacket test` gains a runner per target in `jacket.toml`. A test compiled for that target is run through the runner, and its result comes from the semihosting exit code. Output also goes through semihosting.

```toml
[target.mps2-an386]
triple = "thumbv7em-none-eabihf"
cpu = "cortex-m4"
board = "Extensions/Embedded/Boards/Mps2_An386"
runner = ["qemu-system-arm", "-M", "mps2-an386", "-nographic", "-semihosting-config", "enable=on,target=native", "-kernel", "{exe}"]
```

`jacket test --target mps2-an386` builds and runs every test whose notes allow `OS == .NONE`. If the runner's program is missing, the run fails with an install hint.

Boards and emulators, in the order they're added:

| Board | CPU | Emulator |
|---|---|---|
| `Qemu_Virt_Aarch64` | Cortex-A53, 64-bit | `qemu-system-aarch64 -M virt` |
| `Mps2_An386` | Cortex-M4F, 32-bit | `qemu-system-arm -M mps2-an386` |
| `Mps2_An505` | Cortex-M33, 32-bit | `qemu-system-arm -M mps2-an505` |
| `Qemu_Virt_Riscv32` | rv32imac | `qemu-system-riscv32 -M virt` |
| `Arduino_Uno` | ATmega328P, 8-bit CPU with 16-bit pointers | `qemu-system-avr -M uno` |

Renode comes in later for boards with real peripherals (STM32, nRF52). `qemu -s` plus `gdb` debugs firmware with our DWARF.

### Size

On small parts, code size is correctness. Each board's CI leg builds `examples/embedded/blinky` and fails if the flash size grows more than 5% over a committed baseline (`tools/bench/gate.py` style).

Size work:

- `Type_Info` only for types reflected at run time (`runtime_storageless_type_info`).
- `print` without pulling in every formatter.
- Dead-code elimination over the runtime.

### Milestones

1. **E0, freestanding aarch64.** Add `TargetOs::None`, `aarch64-none-elf`, ld.lld linking and compiler-rt lookup.
   - Done means a program with `.OMIT` and a hand-written reset handler runs on QEMU `virt` and exits through semihosting with code 0.
2. **E1, the runtime.** The `NONE` runtime, `Basic` building on `NONE`, panic and console hooks, and the `Extensions/Embedded` heap.
3. **E2, embedded language support.** `@Section`, `volatile_*`, `@Interrupt`, intrinsics, `Memory_Map`/`link_for` and `write_binary`.
4. **E3, the test runner.** Runners in `jacket test`, a semihosting console, and a CI leg running the `NONE`-compatible stdlib tests on QEMU `virt` aarch64.
5. **E4, 32-bit microcontrollers.** Needs N-bit N4. Add the `Mps2_An386`, `Mps2_An505` and `Qemu_Virt_Riscv32` boards, with blinky and the stdlib tests on each.
6. **E5, 16-bit.** Needs N-bit N5. The `Arduino_Uno` board on `qemu-system-avr`, and the size gate.

## Part 3: lints

Two jailint rules in the `restriction` category, so both are off by default. A project turns them on with `[lint.rules] narrow_pointers = "warn"` or `-W freestanding`.

They catch code that compiles for the target being linted but would be wrong on a narrow or freestanding one. Anything that simply fails to compile for that target is not their job: `jailint --target <triple>` compiles for the target, and the compiler reports it.

### `narrow_pointers`

Flags code whose meaning depends on 8-byte pointers. Lands with the N-bit work, after N0, because it uses `DataLayout`.

- **Constants that change with pointer size.** The rule re-evaluates each `#assert`, `#if` condition, array length and constant that reads `size_of` or `align_of` of a pointer-holding type, under the 4- and 2-byte layouts. It flags each one whose value or outcome changes. The finding names the type and the sizes, for example `size_of(Node) is 24 here, 12 with 4-byte pointers`. This is real constant evaluation, not text matching.
- **Pointer bits treated as a 64-bit integer.**
  - Masks or shifts touching bits 32 and up on a value cast from a pointer (high-bit pointer tagging).
  - `cast(*u64)` or `cast(*s64)` of storage that holds a pointer.
  - A union member that overlays a pointer with an 8-byte integer or `[8] u8`.
- **Integer constants that can't be addresses or sizes on 32-bit.** Values of 2³² or more used as a pointer, or as an allocation or view size.
- **Foreign C width mismatches.** `#foreign` parameters and fields declared `s64`/`u64` where the binding's own type alias says C `long`/`size_t`. This uses the stdlib C-width aliases from N2, so it is exact, not a guess from names.

### `freestanding`

Flags code that can't work with `OS == .NONE`, or that misbehaves on a microcontroller. Lands with embedded E2, because it uses the NONE runtime and `@Interrupt`.

- **OS-only modules.** Importing a module that doesn't build on `NONE`. The rule reads the same per-module availability facts `jacket doc` computes (`Module::support`), so there is no list to maintain.
- **Allocation through the default allocator.** On `NONE` it has no heap: `alloc`, `New`, `array_add` and `tprint` calls reached with no allocator in scope. Calls inside a `push_allocator` or `push_context` that sets one are fine.
- **Interrupt handlers that do too much.** Inside an `@Interrupt` procedure and everything it calls: allocation, `print`, locks, and calls that may block.
- **Unbounded stack.** Recursion (cycles in the call graph) and stack frames over a configurable size, `[lint.freestanding] max_frame_bytes`, default 1024.
- **Float and wide-integer work the CPU can't do.** `float64` arithmetic, or `s64` division and remainder in a loop, when the linted target has no double-precision FPU or no 64-bit divide. This needs a freestanding target selected; without one, this part stays quiet.

Each rule gets a `### <rule>` section in `docs/tools/jailint.md`, good and bad fixtures under `tests/lint/<rule>/`, and a regenerated `docs/reference/lint-rules.md`.

## Example: blinky on a Cortex-M4

`blinky.jai`, for QEMU `mps2-an386`:

```jai
#import "Basic";
#import "Extensions/Embedded";
Board :: #import "Extensions/Embedded/Boards/Mps2_An386";

// Linker-script symbol: the top of RAM.
__stack_top: u8 #elsewhere;

@Section(".vector_table")
vectors := Board.Vector_Table.{
    initial_stack = *__stack_top,
    reset         = reset,
    systick       = on_systick,
};

ticks: u32;   // written by the interrupt, read with volatile_load

on_systick :: () #c_call #no_context @Interrupt {
    volatile_store(*ticks, volatile_load(*ticks) + 1);
}

reset :: () #c_call #no_context {
    init_runtime();                       // .data, .bss, the static Context
    push_context Board.context() {        // console = UART0, allocator = the board's heap
        Board.start_systick(hz = 1000);
        main();
    }
}

main :: () {
    print("blinky on %, % KiB RAM\n", Board.NAME, Board.RAM_SIZE / 1024);
    led := false;
    while true {
        next := volatile_load(*ticks) + 500;
        while volatile_load(*ticks) < next  wait_for_interrupt();
        led = !led;
        volatile_store(Board.LED0, cast(u32) led);
    }
}
```

`build.jai`, the metaprogram:

```jai
#import "Basic";
#import "Compiler";
Embedded :: #import "Extensions/Embedded";
Board    :: #import "Extensions/Embedded/Boards/Mps2_An386";

#run {
    set_build_options_dc(.{do_output = false});   // this file only builds; it is not the program

    w := compiler_create_workspace("blinky");
    options := get_build_options(w);
    options.output_executable_name = "blinky";
    options.os_target  = .NONE;
    options.cpu_target = .CUSTOM;
    options.llvm_options.target_system_triple = "thumbv7em-none-eabihf";
    options.llvm_options.target_system_cpu    = "cortex-m4";
    options.runtime_support_definitions = .ONLY_INIT;   // we write reset; the runtime gives init_runtime
    options.temporary_storage_size = 1024;
    Embedded.link_for(*options, Board.MEMORY);         // writes blinky.ld, adds -T blinky.ld
    set_build_options(options, w);

    add_build_file("blinky.jai", w);
}
```

Building and running it:

```sh
jaic build.jai                                   # writes blinky (ELF)
qemu-system-arm -M mps2-an386 -nographic -semihosting -kernel blinky
```

Or, with the `[target.mps2-an386]` table above in `jacket.toml`:

```sh
jacket build blinky.jai --target mps2-an386
jacket run   blinky.jai --target mps2-an386      # runs it in the runner
jacket test  --target mps2-an386
```

## Dependencies

- LLVM with the ARM, RISC-V, PowerPC, SystemZ and AVR targets.
- `ld.lld`, and compiler-rt builtins per target.
- QEMU user mode (`qemu-arm`, `qemu-s390x`, `qemu-ppc64`) and system mode (`qemu-system-aarch64`, `-arm`, `-riscv32`, `-avr`).
- wasmtime and node for wasm32.
- Linux multilib (`gcc-multilib`) and Debian cross sysroots for the Linux targets.
