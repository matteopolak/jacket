// The compiler's stack-switching primitives (docs/compiler/interpreter.md) and the state they save.
// Internal to the module: programs use `spawn` and `io_run`. A program that wants the primitives
// itself declares them (tests/native/fibers/switch.jai shows how).

#scope_module

// What one suspended fiber leaves behind. Zero it before first use. The fields belong to the
// compiler: native code keeps the saved stack pointer in `sp` (and, on Windows, the stack bounds the
// thread information block holds), and the interpreter keeps nothing in the program's memory but
// identifies a fiber by this context's address.
Fiber_Context :: struct {
    sp: *void;
    stack_low: *void;
    stack_high: *void;
    stack_dealloc: *void;
}

// Prepares `fiber` so that the first `fiber_switch` to it calls `entry(data)` on the memory
// `stack` .. `stack + size` (the stack grows down from its end; keep it 16-byte aligned, and
// put a guard page at its low end). `entry` must never return: it ends by switching to another
// fiber for good. It runs without a context, so it pushes one before it calls Jai code. A fiber
// context that was prepared before may be prepared again once nothing runs on it.
fiber_init :: (fiber: *Fiber_Context, stack: *void, size: s64, entry: (data: *void) #c_call, data: *void) #intrinsic;

// Saves the registers and stack pointer of the running fiber into `from` and continues `to`, which
// is a prepared fiber or one that earlier switched away. It returns when something switches back
// to `from`. `from` may be a context that was never prepared: it stands for the stack the
// caller is running on (a thread's own stack), and later switches to it resume right here.
fiber_switch :: (from: *Fiber_Context, to: *Fiber_Context) #intrinsic;

// Gives back what the compiler keeps for `fiber` (nothing for native code; the interpreter's
// stacks for that context). Call it once a context is never used again, and not for the running one.
fiber_free :: (fiber: *Fiber_Context) #intrinsic;

#scope_export
