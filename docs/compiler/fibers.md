# Fiber intrinsics

## What it is

Three intrinsics that start a procedure on a stack of its own and switch between stacks: `fiber_init`, `fiber_switch` and `fiber_free`. They are the only compiler support `stdlib/Extensions/Io` ([fibers](../specs/stdlib/README.md)) needs. A module declares them itself (they are not in the prelude), with a context struct of four pointers:

```jai
Fiber_Context :: struct { sp, stack_low, stack_high, stack_dealloc: *void; }

fiber_init   :: (fiber: *Fiber_Context, stack: *void, size: s64, entry: (data: *void) #c_call, data: *void) #intrinsic;
fiber_switch :: (from: *Fiber_Context, to: *Fiber_Context) #intrinsic;
fiber_free   :: (fiber: *Fiber_Context) #intrinsic;
```

`fiber_init` prepares `fiber` so the first switch to it calls `entry(data)` on `stack[0..size]`. `fiber_switch` saves the running stack's registers into `from` and resumes `to`; it returns when somebody switches back to `from`. `fiber_free` releases whatever the backend kept for a context (nothing natively). `entry` must never return: finish by switching away for good. `from` needs no `fiber_init`; it is just where the running stack is saved.

## How it works

`emit_intrinsic` (`sema/calls.rs`) maps the procedure names to `ir::Intrinsic::{FiberInit, FiberSwitch, FiberFree}`.

Native: `crates/jaic/src/fiber.rs` holds one assembly routine pair per ABI, `jaic_fiber_switch(from, to)` and `jaic_fiber_prepare(top, entry, arg) -> sp`. `jaic-llvm` emits it as module-level inline asm (local symbols, so each codegen unit has its own copy) and lowers `FiberSwitch` to a call, `FiberInit` to `prepare` plus stores of `sp`, `stack_low`, `stack_high` and `stack_dealloc` into the context. A switch pushes the callee-saved registers, stores `sp`, loads the other `sp` and pops.

| ABI | Saved by a switch |
| --- | --- |
| x64 SysV (Linux, macOS) | rbx, rbp, r12-r15 |
| Win64 | the above plus rdi, rsi, xmm6-15, and the TIB `StackBase`, `StackLimit` and `DeallocationStack` (gs:8, gs:0x10, gs:0x1478) |
| arm64 AAPCS64 (macOS, Linux) | x19-x30, d8-d15 |
| arm64 Windows | the above plus the same three TIB fields through x18 |

A fresh stack starts at a trampoline that moves `data` into the argument register and calls `entry`.

Interpreter: `interp/fibers.rs` gives every fiber a mapping `[guard][native stack][value stack]` and its own `ExecState`, and reuses the same assembly (as `global_asm!`) to switch Rust stacks. A trap inside a fiber is carried to the root fiber, whose `fiber_switch` then reports it. The stack the program passes to `fiber_init` is not used by the interpreter, so a runaway recursion shows up as the interpreter's "stack overflow" trap instead of a guard page hit. Fibers are not available in the sandbox host (`jaic -interp -os wasm`, the playground): the call traps.

wasm: all three fail to compile with "fibers are not supported when compiling for WebAssembly: wasm code cannot switch stacks".

## How to change it

A new target needs a new `asm_*` macro in `fiber.rs` (plus its entry in `module_asm` and the `host` module for the interpreter). Keep the `Context` offsets (0, 8, 16, 24) in sync with the assembly. Tests: `tests/native/fibers/` run by `crates/jaic-cli/tests/native.rs` (`fibers_*`), and the `io-fibers-*` stdlib tests.

## Configuration

None. The interpreter's fiber stack sizes are constants in `interp/fibers.rs` (`NATIVE_STACK`, `VALUE_STACK`, `BYTES_PER_CALL`).

## Dependencies

LLVM inline assembly (native), `mmap`/`VirtualAlloc` (interpreter fiber mappings).
