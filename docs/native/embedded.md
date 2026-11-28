# Embedded and freestanding targets

A freestanding target has no operating system: `Operating_System_Tag.NONE`, a triple such as `aarch64-none-elf`, no libc, no threads and no system entry point. The program supplies its own reset code, and the board supplies memory, console and heap.

> [!WARNING]
> Experimental: these targets and their APIs may change in any release.

`aarch64-none-elf` and the Cortex-M triples `thumbv7em-none-eabihf` (Cortex-M4F) and `thumbv8m.main-none-eabihf` (Cortex-M33) build. They are 32-bit targets: 4-byte pointers, Thumb-2 code, the hard-float AAPCS. 16-bit AVR waits on the N-bit layout work.

## How it works

- **Target.** `TargetOs::None` (`crates/jaic/src/target.rs`) maps to `OS == .NONE`. `-os none` selects it for checking, `-target aarch64-none-elf` or a workspace with `os_target = .NONE` and `cpu_target = .ARM64` builds it. `TargetCpu::Arm` with a `TargetAbi::CortexM` core is the Cortex-M family: the triple names the core (`cortex-m4`, `cortex-m33`) and LLVM schedules for it. Other CPUs are an error.
- **Metaprogram and target.** A target's pointers decide where compile-time code runs: a 32-bit target's runs in the interpreter's address window with the target's `OS` and `CPU`, so host-only code (a board's `configure`, the linker-script writer) cannot be compiled into it. A board therefore sets its triple on the workspace it builds (`llvm_options.target_system_triple`; the triple wins over `os_target` and `cpu_target`): `jaic app.jai +Extensions/Embedded/Boards/Mps2_An386`. `jacket --target <board>` does this. With `+Plugin`, `-target` names the program's target only: the plugins' metaprogram is compiled for the host either way (`crates/jaic-cli/src/run.rs`).
- **Linking.** `crates/jaic-native/src/freestanding.rs` runs `ld.lld` directly: `-static -nostdlib --gc-sections`, the objects, any static archives, the compiler-rt builtins, then `additional_linker_arguments`. One of those must be a linker script (`-T script.ld`); without it the build fails and says so. There is no C driver and no other linker to fall back to.
- **Libraries.** A `#library` or `#system_library` must be a static archive or object next to the declaring file (`start.a`, `libstart.a`, `start.o`). Anything else, `libc` included, is a link error naming it.
- **Builtins.** `libclang_rt.builtins` for the exact target (64-bit division, soft float, atomics outlines) comes from `JAIC_COMPILER_RT` (one archive), else `lib/clang/<version>/lib/<triple>/` or `lib/baremetal/` of an LLVM install, else `libgcc.a` of an installed `arm-none-eabi-gcc` or `avr-gcc`, else the cache `jacket toolchain add` fills (`crates/jaic-toolchain/src/builtins.rs`). An archive built for an OS is never used. Cortex-M builtins must be the soft-float ones (`libclang_rt.builtins-armv7em.a`): LLVM calls `__muldf3` and the like with the base calling convention, which the hard-float `armhf` archive does not follow. `jacket toolchain add` builds the right one. The Cortex-M runtime defines `abort` (it ends the run like a fault) because a GNU `libgcc.a` calls it. If the link needs a builtin and none was found, the error says how to get one.
- **Runtime.** `Runtime_Support` has an `OS == .NONE` variant, `stdlib/internal/runtime-freestanding`:
  - `.AUTO` and `.ONLY_INIT` do not define a system entry point; `main` is called by your reset code.
  - There is no default heap. The default allocator panics with a message saying the board must set `context.allocator`.
  - `write_string` and `print` go through `console_hook`. With none set, output is dropped.
  - Failed asserts and checks write their detail to the console, then call `panic_hook`, then halt in a breakpoint loop.

## How to run one

The `Qemu_Virt_Aarch64` and `Mps2_An386` boards in [Extensions/Embedded](../extensions/embedded.md) need no startup file or linker script of its own. `tests/freestanding` in `crates/jaic-cli` also has a hand-written one: `start.S` sets the stack and calls Jai, the program exits through the semihosting `SYS_EXIT_EXTENDED` call, and QEMU turns that into its own exit code.

```sh
jaic build.jai    # writes prog.elf
qemu-system-aarch64 -M virt -cpu cortex-a53 -nographic -nic none \
    -semihosting-config enable=on,target=native -kernel prog.elf
```

The Cortex-M boards run under `qemu-system-arm` (`-M mps2-an386 -cpu cortex-m4`, `-M mps2-an505 -cpu cortex-m33`) with the same semihosting options and `-kernel prog.elf`.

With jacket, name the board in `jacket.toml` and let it do the rest (`[target.dev] board = "Qemu_Virt_Aarch64"`, [targets](../tools/jacket.md#targets)):

```sh
jacket run --target dev    # builds with the board plugin, runs it under QEMU
jacket toolchain           # what --target accepts, and whether this machine can build and run each
```

## How to change it

- Another board for `--target`: give its module a `get_plugin` that runs its `configure` (see `Qemu_Virt_Aarch64`) give it string constants `triple` (which `configure` uses) and `runner`, and add its module to `BOARD_SOURCES` in `crates/jacket/src/target.rs`.
- Another OS-only procedure on NONE: put it behind `#if OS != .NONE`, so calling it is a compile error naming it. Never add a stub that fails at run time.
- Another freestanding CPU or core: `Target::parse` (and `CortexM` for another Cortex-M), `emulation` in `jaic-native/src/freestanding.rs`, the LLVM CPU in `baseline_cpu` of `jaic-llvm/src/machine.rs`, and the compiler-rt file name in `toolchain/freestanding.rs`.

## Configuration

- `JAIC_LINKER`, `-linker`: the `ld.lld` to use. Default: next to the build's LLVM, an LLD install, then `PATH`.
- `JAIC_COMPILER_RT`, `-compiler_rt`: the builtins archive (one file).
- `os_target`, `cpu_target`, `llvm_options.target_system_triple`, `runtime_support_definitions` and `additional_linker_arguments` in `Build_Options`.

## Dependencies

`ld.lld` (Homebrew `lld`, Debian `lld-<llvm major>`), compiler-rt builtins for the target when the program needs them, and for the tests `clang`, `qemu-system-aarch64` and `qemu-system-arm`.

Build the builtins with `jacket toolchain add TRIPLE`. It downloads the pinned compiler-rt source and compiles it with the installed `clang` and `llvm-ar`, into jacket's cache; later builds find it there. See [toolchain discovery](toolchain-discovery.md).

```bash
jacket toolchain add aarch64-none-elf
```

## AVR (Arduino Uno)

`Extensions/Embedded/Boards/Arduino_Uno` builds for `avr-none-elf` (ATmega328P: 32 KiB of flash, 2 KiB of RAM, 2-byte pointers). Compiler flags go before the source, because a board's arguments follow its name:

```sh
jaic -exe app app.jai +Extensions/Embedded/Boards/Arduino_Uno
qemu-system-avr -M uno -bios app -nographic -serial stdio -monitor none
```

`print("hello\n")` builds to about 5.7 KB of code, 0.45 KB of `.data` and 0.4 KB of `.bss`. The qemu tests in `crates/jaic-cli/tests/avr.rs` run it, and a program that calls through `#c_call` and Jai procedure pointers.

How it differs from the other boards:

- **Memory.** Flash and RAM are separate address spaces, so ordinary loads cannot read flash. `.data` and `.rodata` live in RAM, and the reset code in `Extensions/Embedded/avr.jai` copies them from flash with `lpm`. Constant data therefore costs RAM; keep it small.
- **Code pointers** are word addresses in flash (address space 1). The compiler casts them to and from plain pointers and casts the callee back before an indirect call.
- **Linking.** The linker script places flash at 0 and RAM at `0x800100`, and lld truncates addresses to 16 bits. `ld.lld` needs no `-m` for AVR. The Windows `jaic` cannot build for AVR: the LLVM it links has no AVR backend.
- **Builtins.** `__mulsi3` and `abort` come from `avr.jai`. The rest of compiler-rt comes from `avr-gcc`'s `libgcc.a`, else `jacket toolchain add avr-none-elf`, built for `atmega328p` (other AVR cores are rejected at link time). A `JAIC_COMPILER_RT` path that does not exist is an error that says so.
- **Build options.** The board sets `set_optimization(.OPTIMIZED_SMALL)` (an unoptimized image keeps compile-time scratch buffers and does not fit), `stack_trace = false` (trace text is RAM), `null_pointer_check = .OFF` (RAM starts at data address 0x100, so no low address is unmapped) and a 64-byte temporary storage.
- **Size.** A `String_Builder` is 256 bytes where pointers are 2 bytes wide (4 KiB elsewhere). `print` with no `%` and no values writes the text directly, so none of the formatter is linked; a `print` with values pulls in the formatter, which does not fit 32 KiB of flash with the rest of `Basic`. `init_runtime` leaves `context_info` empty, because describing `Context` links about 17 KB of type information. Call `describe_context()` after `init_runtime()` when something needs it, such as `Remap_Context`.
- **C ABI.** The runtime's `memset`, `memcmp` and `bcmp` take `int` and `size_t` at the target's width (16 bits on AVR), as LLVM's calls pass them.
- **Compile time.** The interpreter runs the program's own layout in a 48 KiB window, and the compile-time buffers shrink to fit, so `-check` works. Compile-time code at host width with values relaid out when they reach the program is not built (see [n-bit and embedded](n-bit-and-embedded.md)).

Required tools: `ld.lld` built with AVR support, the compiler-rt archive, and `qemu-system-avr` (Debian `qemu-system-misc`) to run it. Without a tool the tests skip with its install note; under `CI` they fail.
