//! The stack-switching code behind fibers (`docs/compiler/interpreter.md`).
//!
//! A fiber's saved state is a [`Context`] (`Fiber_Context` in `stdlib/Extensions/Io/fiber_context.jai`):
//! the stack pointer, with the callee-saved registers pushed on that stack, plus the stack bounds
//! that Windows keeps in the thread information block. Two routines, written once per CPU and
//! calling convention, do all of it:
//!
//! - `jaic_fiber_switch(from, to)` pushes the callee-saved registers, stores the stack pointer in
//!   `from`, loads it from `to`, pops the registers it finds there and returns on the new stack;
//! - `jaic_fiber_prepare(top, entry, arg) -> sp` lays out the frame a switch pops for a new fiber, so
//!   the first switch to it calls `entry(arg)` on the stack that ends at `top`. `entry` never returns.
//!
//! The same text serves the LLVM backend, which puts it into every module that switches fibers (as
//! module assembly with local symbols: each object file has its own copy), and the interpreter,
//! which assembles it into `jaic` itself and runs its own interpretation state on separate native
//! stacks (`interp/fibers.rs`).

/// What the routines save per fiber. The layout is `Fiber_Context`'s.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Context {
    /// The stack pointer while the fiber is not running.
    pub sp: u64,
    /// Lowest address of the stack (Windows `StackLimit`).
    pub stack_low: u64,
    /// One past the highest address (Windows `StackBase`).
    pub stack_high: u64,
    /// Windows `DeallocationStack`.
    pub stack_dealloc: u64,
}

/// Byte offsets of `Context`'s fields.
pub const SP: u64 = 0;

pub const STACK_LOW: u64 = 8;
pub const STACK_HIGH: u64 = 16;
pub const STACK_DEALLOC: u64 = 24;

/// The symbols the assembly defines (without the Mach-O underscore).
pub const SWITCH: &str = "jaic_fiber_switch";

pub const PREPARE: &str = "jaic_fiber_prepare";

/// x86-64, System V (macOS, Linux). `$u` prefixes the symbol names (`_` for Mach-O), `$decl` is
/// the visibility directives and `$tail` closes the text.
macro_rules! asm_x64_sysv {
    ($u:literal, $l:literal, $decl:literal, $tail:literal) => {
        concat!(
            ".text\n.p2align 4\n",
            $decl,
            $u,
            "jaic_fiber_switch:\n",
            "    pushq %rbp\n",
            "    pushq %rbx\n",
            "    pushq %r12\n",
            "    pushq %r13\n",
            "    pushq %r14\n",
            "    pushq %r15\n",
            "    movq %rsp, (%rdi)\n",
            "    movq (%rsi), %rsp\n",
            "    popq %r15\n",
            "    popq %r14\n",
            "    popq %r13\n",
            "    popq %r12\n",
            "    popq %rbx\n",
            "    popq %rbp\n",
            "    ret\n",
            ".p2align 4\n",
            $u,
            "jaic_fiber_prepare:\n",
            "    andq $-16, %rdi\n",
            "    leaq ",
            $l,
            "jaic_fiber_trampoline(%rip), %rax\n",
            "    movq %rax, -8(%rdi)\n",
            "    movq $0, -16(%rdi)\n",
            "    movq $0, -24(%rdi)\n",
            "    movq %rsi, -32(%rdi)\n",
            "    movq %rdx, -40(%rdi)\n",
            "    movq $0, -48(%rdi)\n",
            "    movq $0, -56(%rdi)\n",
            "    leaq -56(%rdi), %rax\n",
            "    ret\n",
            ".p2align 4\n",
            $l,
            "jaic_fiber_trampoline:\n",
            "    movq %r13, %rdi\n",
            "    callq *%r12\n",
            "    ud2\n",
            $tail
        )
    };
}

/// x86-64, Microsoft x64: also xmm6-xmm15 and the stack bounds in the thread information block
/// (`gs:8` StackBase, `gs:0x10` StackLimit, `gs:0x1478` DeallocationStack).
macro_rules! asm_x64_win {
    ($u:literal, $l:literal, $decl:literal, $tail:literal) => {
        concat!(
            ".text\n.p2align 4\n",
            $decl,
            $u,
            "jaic_fiber_switch:\n",
            "    pushq %rbp\n",
            "    pushq %rbx\n",
            "    pushq %rdi\n",
            "    pushq %rsi\n",
            "    pushq %r12\n",
            "    pushq %r13\n",
            "    pushq %r14\n",
            "    pushq %r15\n",
            "    subq $168, %rsp\n",
            "    movups %xmm6, 0(%rsp)\n",
            "    movups %xmm7, 16(%rsp)\n",
            "    movups %xmm8, 32(%rsp)\n",
            "    movups %xmm9, 48(%rsp)\n",
            "    movups %xmm10, 64(%rsp)\n",
            "    movups %xmm11, 80(%rsp)\n",
            "    movups %xmm12, 96(%rsp)\n",
            "    movups %xmm13, 112(%rsp)\n",
            "    movups %xmm14, 128(%rsp)\n",
            "    movups %xmm15, 144(%rsp)\n",
            "    movq %gs:8, %rax\n",
            "    movq %rax, 16(%rcx)\n",
            "    movq %gs:16, %rax\n",
            "    movq %rax, 8(%rcx)\n",
            "    movq %gs:0x1478, %rax\n",
            "    movq %rax, 24(%rcx)\n",
            "    movq %rsp, (%rcx)\n",
            "    movq (%rdx), %rsp\n",
            "    movq 16(%rdx), %rax\n",
            "    movq %rax, %gs:8\n",
            "    movq 8(%rdx), %rax\n",
            "    movq %rax, %gs:16\n",
            "    movq 24(%rdx), %rax\n",
            "    movq %rax, %gs:0x1478\n",
            "    movups 0(%rsp), %xmm6\n",
            "    movups 16(%rsp), %xmm7\n",
            "    movups 32(%rsp), %xmm8\n",
            "    movups 48(%rsp), %xmm9\n",
            "    movups 64(%rsp), %xmm10\n",
            "    movups 80(%rsp), %xmm11\n",
            "    movups 96(%rsp), %xmm12\n",
            "    movups 112(%rsp), %xmm13\n",
            "    movups 128(%rsp), %xmm14\n",
            "    movups 144(%rsp), %xmm15\n",
            "    addq $168, %rsp\n",
            "    popq %r15\n",
            "    popq %r14\n",
            "    popq %r13\n",
            "    popq %r12\n",
            "    popq %rsi\n",
            "    popq %rdi\n",
            "    popq %rbx\n",
            "    popq %rbp\n",
            "    ret\n",
            ".p2align 4\n",
            $u,
            "jaic_fiber_prepare:\n",
            "    andq $-16, %rcx\n",
            "    leaq ",
            $l,
            "jaic_fiber_trampoline(%rip), %rax\n",
            "    movq %rax, -8(%rcx)\n",
            "    movq $0, -16(%rcx)\n",
            "    movq $0, -24(%rcx)\n",
            "    movq $0, -32(%rcx)\n",
            "    movq $0, -40(%rcx)\n",
            "    movq %rdx, -48(%rcx)\n",
            "    movq %r8, -56(%rcx)\n",
            "    movq $0, -64(%rcx)\n",
            "    movq $0, -72(%rcx)\n",
            "    leaq -240(%rcx), %rax\n",
            "    ret\n",
            ".p2align 4\n",
            $l,
            "jaic_fiber_trampoline:\n",
            "    movq %r13, %rcx\n",
            "    subq $32, %rsp\n",
            "    callq *%r12\n",
            "    ud2\n",
            $tail
        )
    };
}

/// AArch64, AAPCS64 (macOS, Linux, and Windows with `$tib`/`$load_tib` for the thread environment
/// block that `x18` holds): x19-x30 and d8-d15 live in a 160-byte frame.
macro_rules! asm_aarch64 {
    ($u:literal, $l:literal, $decl:literal, $save_tib:expr, $load_tib:expr, $tail:literal) => {
        concat!(
            ".text\n.p2align 2\n",
            $decl,
            $u,
            "jaic_fiber_switch:\n",
            "    sub sp, sp, #160\n",
            "    stp x19, x20, [sp, #0]\n",
            "    stp x21, x22, [sp, #16]\n",
            "    stp x23, x24, [sp, #32]\n",
            "    stp x25, x26, [sp, #48]\n",
            "    stp x27, x28, [sp, #64]\n",
            "    stp x29, x30, [sp, #80]\n",
            "    stp d8, d9, [sp, #96]\n",
            "    stp d10, d11, [sp, #112]\n",
            "    stp d12, d13, [sp, #128]\n",
            "    stp d14, d15, [sp, #144]\n",
            $save_tib,
            "    mov x9, sp\n",
            "    str x9, [x0]\n",
            "    ldr x9, [x1]\n",
            "    mov sp, x9\n",
            $load_tib,
            "    ldp x19, x20, [sp, #0]\n",
            "    ldp x21, x22, [sp, #16]\n",
            "    ldp x23, x24, [sp, #32]\n",
            "    ldp x25, x26, [sp, #48]\n",
            "    ldp x27, x28, [sp, #64]\n",
            "    ldp x29, x30, [sp, #80]\n",
            "    ldp d8, d9, [sp, #96]\n",
            "    ldp d10, d11, [sp, #112]\n",
            "    ldp d12, d13, [sp, #128]\n",
            "    ldp d14, d15, [sp, #144]\n",
            "    add sp, sp, #160\n",
            "    ret\n",
            ".p2align 2\n",
            $u,
            "jaic_fiber_prepare:\n",
            "    and x0, x0, #0xfffffffffffffff0\n",
            "    sub x0, x0, #160\n",
            "    stp xzr, xzr, [x0, #16]\n",
            "    stp xzr, xzr, [x0, #32]\n",
            "    stp xzr, xzr, [x0, #48]\n",
            "    stp xzr, xzr, [x0, #64]\n",
            "    stp xzr, xzr, [x0, #96]\n",
            "    stp xzr, xzr, [x0, #112]\n",
            "    stp xzr, xzr, [x0, #128]\n",
            "    stp xzr, xzr, [x0, #144]\n",
            "    stp x1, x2, [x0, #0]\n",
            "    adr x9, ",
            $l,
            "jaic_fiber_trampoline\n",
            "    stp xzr, x9, [x0, #80]\n",
            "    ret\n",
            ".p2align 2\n",
            $l,
            "jaic_fiber_trampoline:\n",
            "    mov x0, x20\n",
            "    blr x19\n",
            "    brk #0\n",
            $tail
        )
    };
}

/// Windows' thread environment block, through `x18`: StackBase +8, StackLimit +16 and
/// DeallocationStack +0x1478 (an offset too big for `ldr`'s immediate, so it goes through `x10`).
macro_rules! aarch64_save_tib {
    () => {
        concat!(
            "    ldr x10, [x18, #8]\n",
            "    str x10, [x0, #16]\n",
            "    ldr x10, [x18, #16]\n",
            "    str x10, [x0, #8]\n",
            "    mov x11, #0x1478\n",
            "    ldr x10, [x18, x11]\n",
            "    str x10, [x0, #24]\n",
        )
    };
}

macro_rules! aarch64_load_tib {
    () => {
        concat!(
            "    ldr x10, [x1, #16]\n",
            "    str x10, [x18, #8]\n",
            "    ldr x10, [x1, #8]\n",
            "    str x10, [x18, #16]\n",
            "    mov x11, #0x1478\n",
            "    ldr x10, [x1, #24]\n",
            "    str x10, [x18, x11]\n",
        )
    };
}

const ELF_NOTE: &str = "\n.pushsection .note.GNU-stack,\"\",@progbits\n.popsection\n";

/// The text for `arch`, to put into a module that switches fibers: the symbols are local to the
/// module, so every object file of a program may carry a copy. `macho` selects the Mach-O symbol
/// prefix; `windows` and `macho` otherwise pick the object format by the architecture.
pub fn module_asm(arch: crate::abi::Arch, macho: bool) -> Option<String> {
    use crate::abi::Arch;
    Some(match arch {
        Arch::X86_64 if macho => asm_x64_sysv!("_", "L", "", "").to_string(),
        Arch::X86_64 => format!("{}{ELF_NOTE}", asm_x64_sysv!("", ".L", "", "")),
        Arch::Win64 => asm_x64_win!("", ".L", "", "").to_string(),
        Arch::Aarch64 if macho => asm_aarch64!("_", "L", "", "", "", "").to_string(),
        Arch::Aarch64 => format!("{}{ELF_NOTE}", asm_aarch64!("", ".L", "", "", "", "")),
        Arch::Win64Arm => {
            asm_aarch64!("", ".L", "", aarch64_save_tib!(), aarch64_load_tib!(), "").to_string()
        }
        Arch::Wasm64 => return None,
    })
}

/// The routines linked into `jaic` for the interpreter (the host's own architecture only).
#[allow(unsafe_code)]
#[cfg(all(
    not(target_arch = "wasm32"),
    any(target_arch = "x86_64", target_arch = "aarch64")
))]
pub mod host {
    use super::Context;

    #[cfg(all(target_arch = "x86_64", target_os = "macos"))]
    std::arch::global_asm!(
        asm_x64_sysv!(
            "_",
            "L",
            ".globl _jaic_fiber_switch\n.globl _jaic_fiber_prepare\n",
            ""
        ),
        options(att_syntax)
    );

    #[cfg(all(target_arch = "x86_64", unix, not(target_os = "macos")))]
    std::arch::global_asm!(
        asm_x64_sysv!(
            "",
            ".L",
            ".globl jaic_fiber_switch\n.globl jaic_fiber_prepare\n",
            "\n.pushsection .note.GNU-stack,\"\",@progbits\n.popsection\n"
        ),
        options(att_syntax)
    );

    #[cfg(all(target_arch = "x86_64", windows))]
    std::arch::global_asm!(
        asm_x64_win!(
            "",
            ".L",
            ".globl jaic_fiber_switch\n.globl jaic_fiber_prepare\n",
            ""
        ),
        options(att_syntax)
    );

    #[cfg(all(target_arch = "aarch64", target_os = "macos"))]
    std::arch::global_asm!(asm_aarch64!(
        "_",
        "L",
        ".globl _jaic_fiber_switch\n.globl _jaic_fiber_prepare\n",
        "",
        "",
        ""
    ));

    #[cfg(all(target_arch = "aarch64", unix, not(target_os = "macos")))]
    std::arch::global_asm!(asm_aarch64!(
        "",
        ".L",
        ".globl jaic_fiber_switch\n.globl jaic_fiber_prepare\n",
        "",
        "",
        "\n.pushsection .note.GNU-stack,\"\",@progbits\n.popsection\n"
    ));

    #[cfg(all(target_arch = "aarch64", windows))]
    std::arch::global_asm!(asm_aarch64!(
        "",
        ".L",
        ".globl jaic_fiber_switch\n.globl jaic_fiber_prepare\n",
        aarch64_save_tib!(),
        aarch64_load_tib!(),
        ""
    ));

    unsafe extern "C" {
        /// Save the running fiber into `from` and run `to`; returns when something switches back.
        pub fn jaic_fiber_switch(from: *mut Context, to: *mut Context);

        /// The stack pointer to store in a `Context` so the first switch to it calls `entry(arg)`
        /// on the stack ending at `top`.
        pub fn jaic_fiber_prepare(top: u64, entry: extern "C" fn(u64) -> !, arg: u64) -> u64;
    }
}
