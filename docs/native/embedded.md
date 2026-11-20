# Embedded and freestanding targets

A freestanding target has no operating system: `Operating_System_Tag.NONE`, a triple such as `aarch64-none-elf`, no libc, no threads and no system entry point. The program supplies its own reset code, and the board supplies memory, console and heap.

Only `aarch64-none-elf` builds today. 32-bit and 16-bit microcontrollers wait on the N-bit layout work.

## How it works

- **Target.** `TargetOs::None` (`crates/jaic/src/target.rs`) maps to `OS == .NONE`. `-os none` selects it for checking, `-target aarch64-none-elf` or a workspace with `os_target = .NONE` and `cpu_target = .ARM64` builds it. Only AArch64 is accepted; other CPUs are an error.
- **Linking.** `crates/jaic-native/src/freestanding.rs` runs `ld.lld` directly: `-static -nostdlib --gc-sections`, the objects, any static archives, the compiler-rt builtins, then `additional_linker_arguments`. One of those must be a linker script (`-T script.ld`); without it the build fails and says so. There is no C driver and no other linker to fall back to.
- **Libraries.** A `#library` or `#system_library` must be a static archive or object next to the declaring file (`start.a`, `libstart.a`, `start.o`). Anything else, `libc` included, is a link error naming it.
- **Builtins.** `libclang_rt.builtins` for the exact target (64-bit division, soft float, atomics outlines) comes from `JAIC_COMPILER_RT`, else `lib/clang/<version>/lib/<triple>/` or `lib/baremetal/` of an LLVM install (`crates/jaic/src/toolchain/freestanding.rs`). An archive built for an OS is never used. If the link needs a builtin and none was found, the error says how to get one.
- **Runtime.** `Runtime_Support` has an `OS == .NONE` variant, `stdlib/internal/runtime-freestanding`:
  - `.AUTO` and `.ONLY_INIT` do not define a system entry point; `main` is called by your reset code.
  - There is no default heap. The default allocator panics with a message saying the board must set `context.allocator`.
  - `write_string` and `print` go through `console_hook`. With none set, output is dropped.
  - Failed asserts and checks write their detail to the console, then call `panic_hook`, then halt in a breakpoint loop.

## How to run one

`tests/freestanding` in `crates/jaic-cli` is a complete example: `start.S` sets the stack and calls Jai, the program exits through the semihosting `SYS_EXIT_EXTENDED` call, and QEMU turns that into its own exit code.

```sh
jaic build.jai    # writes prog.elf
qemu-system-aarch64 -M virt -cpu cortex-a53 -nographic \
    -semihosting-config enable=on,target=native -kernel prog.elf
```

## How to change it

- Another OS-only procedure on NONE: put it behind `#if OS != .NONE`, so calling it is a compile error naming it. Never add a stub that fails at run time.
- Another freestanding CPU: `Target::parse`, `emulation` in `jaic-native/src/freestanding.rs`, and the compiler-rt file name in `toolchain/freestanding.rs`.

## Configuration

- `JAIC_LLD`: the `ld.lld` to use. Default: next to the build's LLVM, an LLD install, then `PATH`.
- `JAIC_COMPILER_RT`: the builtins archive.
- `os_target`, `cpu_target`, `llvm_options.target_system_triple`, `runtime_support_definitions` and `additional_linker_arguments` in `Build_Options`.

## Dependencies

`ld.lld` (Homebrew `lld`, Debian `lld-<llvm major>`), compiler-rt builtins for the target when the program needs them, and for the tests `clang` and `qemu-system-aarch64`.
