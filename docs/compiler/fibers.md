# Fiber stack switching

`fiber_init` / `fiber_switch` (see [interpreter](interpreter.md)) swap stacks with two small assembly routines, `jaic_fiber_switch` and `jaic_fiber_prepare`, one file per CPU and calling convention in `crates/jaic/src/fiber/`. They work on a four-word `Context`, which is also the layout of `Fiber_Context` in the standard library.

## How it works

The same assembly text is used twice:

- inside `jaic`, assembled with `global_asm!` for the interpreter's native stacks (`fiber::host`);
- inside every compiled program, pasted into the LLVM module as module-level asm (`fiber::module_asm`), for every target the backend supports, including ones `jaic` itself isn't built for.

Compiled programs contain no Rust, so a coroutine crate can't supply the switch routine. The caller also owns the stack and the context (which carries Windows stack bounds), which crates don't allow.

## How to change it

A new target needs a `.s` file and a case in `module_asm` and `host`. Keep the `Context` offsets (0, 8, 16, 24) in sync with the assembly. Tests: `fiber::tests`, `crates/jaic-llvm/tests/fiber_asm.rs`. Interpreter fiber stack sizes are constants in `interp/fibers.rs`.
