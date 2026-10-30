# Fiber stack switching

## What it is

`fiber_init` / `fiber_switch` (see [interpreter](interpreter.md)) need a way to swap stacks. That lives in `crates/jaic/src/fiber.rs` plus one small assembly file per CPU and calling convention in `crates/jaic/src/fiber/`.

## How it works

Two routines, `jaic_fiber_switch` and `jaic_fiber_prepare`, work on a four-word `Context` that is also the layout of `Fiber_Context` in the Jai standard library. The same text is used in two places:

- inside `jaic`, where `fiber::host` assembles it with `global_asm!` for the interpreter's own native stacks;
- inside every program the LLVM backend compiles, where `jaic-llvm` pastes it into the module as module-level asm (`fiber::module_asm`) for each target it can build for, including ones `jaic` itself is never built for (Windows on arm64, Linux on arm64, and so on).

## Why not a coroutine crate such as `corosensei`

The second use is the blocker. The compiled program is plain Jai with no Rust in it, so a Rust crate cannot supply its switch routine. The assembly has to exist as text that the backend can emit for any target, and as long as it does, replacing only the `jaic` copy would leave two mechanisms for the same job and keep every `.s` file.

Smaller mismatches would also have to be bridged:

- the stack belongs to the caller (the Jai program passes `stack` and `size`, and the context is four words the program owns), while a crate allocates and owns its own stacks;
- the context also carries the Windows stack bounds (`StackBase`, `StackLimit`, `DeallocationStack`) that the program reads back, so the layout is part of the language-facing contract;
- the interpreter's fibers need a native stack, a guard and a value stack in one mapping.

So the assembly stays. Revisit only if the backend stops emitting the routines into compiled programs.

## How to change it

A new target needs a `.s` file, a case in `module_asm` and in the `host` module. Keep the offsets of `Context` (0, 8, 16, 24) in sync with the assembly. Tests: `fiber::tests` and `crates/jaic-llvm/tests/fiber_asm.rs`.

## Configuration

None. Stack sizes for interpreter fibers are constants in `interp/fibers.rs`.

## Dependencies

`jaic-llvm` (consumes `module_asm` and the offsets), `interp/fibers.rs` (the host routines).
