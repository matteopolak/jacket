# Cranelift backend

`crates/jaic-cranelift` is an experimental code generator for unoptimized native builds. `jaic file.jai -backend cranelift` writes the objects with [Cranelift](https://cranelift.dev) instead of LLVM; linking is the same as for the [LLVM backend](llvm-backend.md). A build Cranelift cannot write is written by LLVM. The default stays `-backend llvm`.

What to expect:

- **No debug information.** Cranelift builds have no DWARF line tables and no unwind tables, so a debugger shows no source lines. Frame pointers are kept, so stack traces still work.
- **Slower programs.** Generated code takes 1.0 to 1.7 times the cycles of LLVM's `-O0` output.
- **Single-pass register allocation.** Cranelift's fastest allocator, which is why code generation is fast and the code is slower.

## How it works

`emit_objects` in `lib.rs`:

1. Checks the build options and the target (`check_options`, `isa::target_isa`). Anything it cannot build is an `Error::Unsupported`, here or later while lowering; nothing is written.
2. Declares every function, foreign symbol, global and helper in one `ObjectModule` (`data::Symbols::declare`).
3. Compiles the functions on jaic's thread pool (`jaic::pool::units`), largest first, in tasks of about 2,000 IR instructions. Each task lowers IR into a Cranelift `Function` (`function.rs`, `inst.rs`, `intrinsics.rs`, `checks.rs`) and compiles it to machine code with relocations.
4. Defines the compiled bytes in the module in order, then globals and their relocations, and writes one object file.

The IR maps directly: an IR value is defined once and blocks are in reverse post-order, so IR values become Cranelift values with no variables or phis. Aggregates stay in memory. A slot aligned to more than 16 bytes, the frame alignment Cranelift gives, is a larger slot with its address rounded up.

**C ABI** (`abi.rs`). Aggregate classification is `jaic::abi`'s, shared with LLVM. Cranelift passes scalars only, so a by-value aggregate becomes its register pieces, `StructArgument` (x86-64 stack copies) or a pointer. On AArch64 the all-or-nothing register rule and Apple's stack-only variadic arguments are done by passing padding arguments that use up the registers. Jai-convention procedures use the platform's C convention.

**x86-64 variadic calls.** System V wants `%al` to hold an upper bound on the vector registers a variadic call uses, and Cranelift cannot set it. Such a call goes through `jaic.variadic_call` (`mov eax, 8; jmp r15`) with the callee in Cranelift's pinned register, `r15`, which the caller saves and restores around the call.

**Checks.** Null checks use `jaic::ir::NullChecks`, the same elision as LLVM. A failed check calls `Runtime_Support`'s reporter and traps; a null check calls the local `jaic.null_fail`, which takes the location packed in one argument.

**Helpers.** Instructions Cranelift has no operation for (`rdtsc`, `cntvct_el0`, `pause`, `yield`) are small functions written as bytes (`data::Helper::machine_code`). `sin`, `cos` and `round` call libc.

## Speed

On the generated 100,000-line benchmark (`bench compare --modes build --sizes 100000`, Apple M5, 10 cores), Cranelift's codegen phase takes 0.54 to 0.56 of LLVM's `-O0` time (0.08 s against 0.15 s) and the whole build 0.82 to 0.88, with or without LLVM's debug info. With Cranelift's default allocator, codegen was no faster than LLVM's. Executables are 3 to 14% larger. Generated programs take 1.04 to 1.73 times LLVM's `-O0` cycles on `benchmarks/runtime/`.

## What it does not do

- optimized builds (`-o`, `-od`, `-O1` and above), sanitizers, coverage, LLVM IR or bitcode output, `-cpu` features
- targets other than x86-64 and arm64 on Linux and macOS
- `#asm` instructions with no portable lowering (`StringCompare`, `cpuid`, `syscall`, x86 timestamp and random instructions), fibers, `long double`, runtime-sized stack allocations, `@Section` on procedures
- Microsoft C++ member results

Each of these is an `Error::Unsupported`. The driver (`generate::with` in `crates/jaic-cli/src/backend.rs`) then has LLVM write the build and says so on the `Backend` line:

```text
   Compiling hello (hello.jai)
     Backend llvm 23.1.6, since Cranelift does not support optimized builds (`-o`, `-od`, `-O1` and above)
    Finished optimized in 0.38s
```

A Cranelift build shows `Backend cranelift 0.134.4`; a default build shows the LLVM line under `-verbose` only.

`-codegen_units` does not apply: there is one object. `#target_features` procedures are compiled for the baseline, since this backend emits no vector instructions that the features would allow.

## How to change it

- A new IR instruction or intrinsic: add it to `inst.rs` or `intrinsics.rs`. If it cannot be lowered, return `self.unsupported(...)`.
- A new option LLVM honours: honour it, or return `unsupported(...)` from `check_options` so LLVM builds it. Never ignore an option that changes behaviour.
- `Error::Failed` is for faults (I/O, a Cranelift error); it stops the build instead of handing it to LLVM.
- Debug info would go in `lib.rs` after compilation: the source line of each instruction is already set with `set_srcloc`.
- Test with `cargo nextest run -p jaic-cli --test native cranelift`. Those tests run the corpus, the compiler regressions, the C ABI fixtures (x86-64 too, under Rosetta) and the examples with Cranelift.

## Configuration

- `-backend llvm|cranelift` (`jaic-cli-args`, default `llvm`).
- The `cranelift` feature of `jaic-cli` (on by default) builds it in. Without it, `-backend cranelift` builds with LLVM and says why.
- `MACOSX_DEPLOYMENT_TARGET` sets the macOS version recorded in objects, as for LLVM (`jaic_native::macos_deployment_target`).

## Dependencies

- `cranelift-codegen`, `cranelift-frontend`, `cranelift-module`, `cranelift-object` (Apache-2.0 WITH LLVM-exception), pinned to one version.
- `jaic` (IR, ABI classification, null-check elision, thread pool) and `jaic-native` (options, link).
