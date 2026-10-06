//! Foreign calls involving a C `long double` wider than `f64` (`Long_Double` from
//! `Jaic_Extensions`), which the Rust prototypes of `call_as` cannot express:
//!
//! - x86-64 System V returns an x87 `long double` in `st(0)`. Its arguments need nothing
//!   special (they are 16-byte aligned stack slots, which `Regs` lays out).
//! - AArch64 Linux passes and returns binary128 in whole 128-bit `q` registers, of which
//!   `call_as` only sets the low 64 bits.
//!
//! Both use a small assembly sequence that loads the argument registers from a block, copies
//! the stack words, calls, and stores the result registers back into the block.
#![allow(unsafe_code)]
use super::Regs;
#[cfg(target_arch = "aarch64")]
use super::write_bytes;
use crate::abi::{Piece, PieceTy};
use crate::ir::Sig;
#[cfg(target_arch = "aarch64")]
use crate::ir::Ty;

/// Call `addr`, whose arguments are in `regs`, and store its result: through `out_ptr` for an
/// aggregate result (`pieces` in registers, or `sret`), else as the scalar result words.
///
/// SAFETY: `addr` is a C function whose signature `sig` describes, with arguments laid out
/// in `regs`; `out_ptr` addresses the aggregate result when there is one.
pub(super) unsafe fn call(
    addr: u64,
    regs: &Regs,
    sig: &Sig,
    ret: Option<(u64, Option<&[Piece]>)>,
    out_ptr: u64,
) -> Result<Vec<u64>, String> {
    #[cfg(target_arch = "x86_64")]
    {
        let _ = sig;
        // Only a result in `st(0)` brings a call here on x86-64.
        let Some((size, Some([piece]))) = ret else {
            return Err("unexpected long double call shape".into());
        };
        debug_assert_eq!(piece.ty, PieceTy::X87);
        // SAFETY: as this function's contract.
        let bytes = unsafe { x87::call(addr, regs) };
        let mut value = [0u8; 16];
        value[..10].copy_from_slice(&bytes[..10]);
        // SAFETY: the out-pointer addresses `size` (16) bytes.
        unsafe {
            std::ptr::copy_nonoverlapping(value.as_ptr(), out_ptr as *mut u8, size.min(16) as usize)
        };
        Ok(Vec::new())
    }
    #[cfg(target_arch = "aarch64")]
    {
        let sret = match ret {
            Some((_, None)) => out_ptr,
            _ => 0,
        };
        // SAFETY: as this function's contract.
        let (x, q) = unsafe { vector::call(addr, regs, sret) };
        match ret {
            None => Ok(match sig.returns.first() {
                None => Vec::new(),
                Some(Ty::F32) => vec![q[0] as u64 & 0xffff_ffff],
                Some(t) if t.is_float() => vec![q[0] as u64],
                Some(t) => vec![super::mask_int(*t, x[0])],
            }),
            Some((_, None)) => Ok(Vec::new()),
            Some((size, Some(pieces))) => {
                let (mut ni, mut nq) = (0, 0);
                for p in pieces {
                    let at = out_ptr + p.offset;
                    let room = size - p.offset;
                    // SAFETY (each write): the out-pointer addresses `size` bytes.
                    match p.ty {
                        PieceTy::I64 => {
                            unsafe { write_bytes(at, x[ni], room.min(8)) };
                            ni += 1;
                        }
                        PieceTy::F128 => {
                            let bytes = q[nq].to_le_bytes();
                            let n = room.min(16) as usize;
                            unsafe {
                                std::ptr::copy_nonoverlapping(bytes.as_ptr(), at as *mut u8, n)
                            };
                            nq += 1;
                        }
                        other => {
                            let width = if other == PieceTy::F32 {
                                4
                            } else {
                                8
                            };
                            unsafe { write_bytes(at, q[nq] as u64, room.min(width)) };
                            nq += 1;
                        }
                    }
                }
                Ok(Vec::new())
            }
        }
    }
}

#[cfg(target_arch = "x86_64")]
mod x87 {
    use super::Regs;
    use std::mem::offset_of;

    #[repr(C)]
    struct Block {
        ints: [u64; 6],
        floats: [u64; 8],
        stack: *const u64,
        words: u64,
        target: u64,
        result: [u8; 16],
    }

    /// Call through registers and stack words from `regs`, returning `st(0)`'s 10 bytes.
    ///
    /// SAFETY: `addr` is a C function returning an x87 `long double` whose arguments are
    /// `regs` (six integer registers, eight SSE registers, then stack slots).
    pub(super) unsafe fn call(addr: u64, regs: &Regs) -> [u8; 16] {
        let mut block = Block {
            ints: [0; 6],
            floats: regs.floats.map(|f| f as u64),
            stack: regs.stack.as_ptr(),
            words: regs.ns as u64,
            target: addr,
            result: [0; 16],
        };
        block.ints.copy_from_slice(&regs.ints[..6]);
        let p: *mut Block = &mut block;
        // SAFETY: the sequence restores `rsp`; `r12` (callee-saved) keeps the block address
        // across the call, `r13` the caller's stack pointer.
        unsafe {
            std::arch::asm!(
                "mov r13, rsp",
                "mov rcx, [r12 + {words}]",
                "lea rax, [rcx*8 + 15]",
                "and rax, -16",
                "sub rsp, rax",
                "and rsp, -16",
                "mov rsi, [r12 + {stack}]",
                "xor eax, eax",
                "2:",
                "cmp rax, rcx",
                "jae 3f",
                "mov rdx, [rsi + rax*8]",
                "mov [rsp + rax*8], rdx",
                "inc rax",
                "jmp 2b",
                "3:",
                "movsd xmm0, [r12 + {floats}]",
                "movsd xmm1, [r12 + {floats} + 8]",
                "movsd xmm2, [r12 + {floats} + 16]",
                "movsd xmm3, [r12 + {floats} + 24]",
                "movsd xmm4, [r12 + {floats} + 32]",
                "movsd xmm5, [r12 + {floats} + 40]",
                "movsd xmm6, [r12 + {floats} + 48]",
                "movsd xmm7, [r12 + {floats} + 56]",
                "mov rdi, [r12 + {ints}]",
                "mov rsi, [r12 + {ints} + 8]",
                "mov rdx, [r12 + {ints} + 16]",
                "mov rcx, [r12 + {ints} + 24]",
                "mov r8, [r12 + {ints} + 32]",
                "mov r9, [r12 + {ints} + 40]",
                // Variadic callees read the number of vector registers used from `al`.
                "mov eax, 8",
                "call qword ptr [r12 + {target}]",
                "fstp tbyte ptr [r12 + {result}]",
                "mov rsp, r13",
                words = const offset_of!(Block, words),
                stack = const offset_of!(Block, stack),
                floats = const offset_of!(Block, floats),
                ints = const offset_of!(Block, ints),
                target = const offset_of!(Block, target),
                result = const offset_of!(Block, result),
                in("r12") p,
                out("r13") _,
                clobber_abi("C"),
            );
        }
        block.result
    }
}

#[cfg(target_arch = "aarch64")]
mod vector {
    use super::Regs;
    use std::mem::offset_of;

    #[repr(C)]
    struct Block {
        ints: [u64; 8],
        sret: u64,
        stack: *const u64,
        words: u64,
        target: u64,
        quads: [u128; 8],
        out_ints: [u64; 2],
        out_quads: [u128; 4],
    }

    /// Call with full 128-bit vector argument registers; returns `x0`, `x1` and `q0`..`q3`.
    ///
    /// SAFETY: `addr` is a C function whose arguments are `regs` (eight integer registers,
    /// eight vector registers, then stack slots), with `sret` in `x8` when nonzero.
    pub(super) unsafe fn call(addr: u64, regs: &Regs, sret: u64) -> ([u64; 2], [u128; 4]) {
        let mut block = Block {
            ints: regs.ints,
            sret,
            stack: regs.stack.as_ptr(),
            words: regs.ns as u64,
            target: addr,
            quads: regs.floats,
            out_ints: [0; 2],
            out_quads: [0; 4],
        };
        let p: *mut Block = &mut block;
        // SAFETY: the sequence restores `sp`; `x21` (callee-saved) keeps the block address
        // across the call, `x22` the caller's stack pointer.
        unsafe {
            std::arch::asm!(
                "mov x22, sp",
                "ldr x9, [x21, #{words}]",
                "lsl x10, x9, #3",
                "add x10, x10, #15",
                "and x10, x10, #-16",
                "sub sp, sp, x10",
                "ldr x11, [x21, #{stack}]",
                "mov x12, #0",
                "2:",
                "cmp x12, x9",
                "b.hs 3f",
                "ldr x13, [x11, x12, lsl #3]",
                "str x13, [sp, x12, lsl #3]",
                "add x12, x12, #1",
                "b 2b",
                "3:",
                "add x14, x21, #{quads}",
                "ldp q0, q1, [x14]",
                "ldp q2, q3, [x14, #32]",
                "ldp q4, q5, [x14, #64]",
                "ldp q6, q7, [x14, #96]",
                "add x14, x21, #{ints}",
                "ldp x0, x1, [x14]",
                "ldp x2, x3, [x14, #16]",
                "ldp x4, x5, [x14, #32]",
                "ldp x6, x7, [x14, #48]",
                "ldr x8, [x21, #{sret}]",
                "ldr x16, [x21, #{target}]",
                "blr x16",
                "stp x0, x1, [x21, #{out_ints}]",
                "add x14, x21, #{out_quads}",
                "stp q0, q1, [x14]",
                "stp q2, q3, [x14, #32]",
                "mov sp, x22",
                words = const offset_of!(Block, words),
                stack = const offset_of!(Block, stack),
                quads = const offset_of!(Block, quads),
                ints = const offset_of!(Block, ints),
                sret = const offset_of!(Block, sret),
                target = const offset_of!(Block, target),
                out_ints = const offset_of!(Block, out_ints),
                out_quads = const offset_of!(Block, out_quads),
                in("x21") p,
                out("x22") _,
                clobber_abi("C"),
            );
        }
        (block.out_ints, block.out_quads)
    }
}

#[cfg(test)]
mod tests {
    use crate::ir::{AggLayout, CAbi, Conv, Sig, Ty};
    #[cfg(target_arch = "x86_64")]
    use crate::wide_float::{self, WideFloat};

    fn wide_layout(ty: Ty) -> AggLayout {
        AggLayout {
            size: 16,
            align: 16,
            fields: vec![(0, ty)],
        }
    }

    /// The IR signature of a C function `(params...) -> long double`, where `wide[i]` marks the
    /// `long double` parameters (passed by address in the IR).
    fn sig(ty: Ty, wide: &[bool]) -> Sig {
        let mut params: Vec<Ty> = wide
            .iter()
            .map(|&w| {
                if w {
                    Ty::Ptr
                } else {
                    Ty::I64
                }
            })
            .collect();
        params.push(Ty::Ptr);
        let mut abi_params: Vec<Option<AggLayout>> =
            wide.iter().map(|&w| w.then(|| wide_layout(ty))).collect();
        abi_params.push(None);
        Sig {
            c_fixed: params.len() as u32,
            params,
            returns: Vec::new(),
            conv: Conv::C,
            c_varargs: false,
            c_abi: Some(Box::new(CAbi {
                params: abi_params,
                ret: Some(wide_layout(ty)),
                ret_indirect: false,
            })),
        }
    }

    /// `(k: s64, x: long double, y: long double) -> long double`, returning `x + y + k` in
    /// `st(0)`; the two x87 arguments are in the caller's stack slots.
    #[cfg(target_arch = "x86_64")]
    #[unsafe(naked)]
    extern "C" fn x87_sum() {
        std::arch::naked_asm!(
            "fld tbyte ptr [rsp + 8]",
            "fld tbyte ptr [rsp + 24]",
            "faddp st(1), st",
            "mov [rsp - 8], rdi",
            "fild qword ptr [rsp - 8]",
            "faddp st(1), st",
            "ret",
        )
    }

    #[cfg(target_arch = "x86_64")]
    #[test]
    fn x87_arguments_and_result() {
        let fmt = WideFloat::X87;
        let one = wide_float::from_f64(fmt, 1.0);
        let x = wide_float::arith(
            fmt,
            wide_float::Arith::Div,
            &one,
            &wide_float::from_f64(fmt, 3.0),
        );
        let y = wide_float::from_f64(fmt, 2.0f64.powi(-62));
        let mut out = [0u8; 16];
        let args = [
            5,
            x.as_ptr() as u64,
            y.as_ptr() as u64,
            out.as_mut_ptr() as u64,
        ];
        super::super::call_with(
            x87_sum as *const () as u64,
            &args,
            &sig(Ty::F80, &[false, true, true]),
        )
        .unwrap();
        let sum = wide_float::arith(fmt, wide_float::Arith::Add, &x, &y);
        let want = wide_float::arith(
            fmt,
            wide_float::Arith::Add,
            &sum,
            &wide_float::from_f64(fmt, 5.0),
        );
        assert_eq!(out, want);
    }

    /// AAPCS64 binary128 values travel in whole `q` registers, like these 128-bit vectors
    /// (Apple's arm64 has no binary128 `long double`, but passes vectors the same way).
    #[cfg(target_arch = "aarch64")]
    mod vectors {
        use super::*;
        use std::arch::aarch64::{
            uint64x2_t, vaddq_u64, vdupq_n_u64, vgetq_lane_u64, vsetq_lane_u64,
        };

        /// Swaps the halves of `a`, adds `k` to both, and adds the ninth vector (on the stack).
        #[allow(clippy::too_many_arguments, improper_ctypes_definitions)]
        extern "C" fn mix(
            a: uint64x2_t,
            k: u64,
            b1: uint64x2_t,
            b2: uint64x2_t,
            b3: uint64x2_t,
            b4: uint64x2_t,
            b5: uint64x2_t,
            b6: uint64x2_t,
            b7: uint64x2_t,
            spilled: uint64x2_t,
        ) -> uint64x2_t {
            let _ = (b1, b2, b3, b4, b5, b6, b7);
            // SAFETY: NEON is always present on AArch64.
            unsafe {
                let swapped = vsetq_lane_u64::<1>(
                    vgetq_lane_u64::<0>(a),
                    vdupq_n_u64(vgetq_lane_u64::<1>(a)),
                );
                vaddq_u64(vaddq_u64(swapped, vdupq_n_u64(k)), spilled)
            }
        }

        #[test]
        fn quad_registers_and_stack() {
            let words =
                |lo: u64, hi: u64| -> [u8; 16] { (lo as u128 | (hi as u128) << 64).to_le_bytes() };
            let a = words(0x1111, 0x2222_0000_0000_0000);
            let filler = words(7, 7);
            let spilled = words(0x30, 0x40);
            let mut out = [0u8; 16];
            let mut args = vec![a.as_ptr() as u64, 0x100];
            args.extend((0..7).map(|_| filler.as_ptr() as u64));
            args.push(spilled.as_ptr() as u64);
            args.push(out.as_mut_ptr() as u64);
            let mut wide = vec![true, false];
            wide.extend([true; 8]);
            super::super::super::call_with(mix as *const () as u64, &args, &sig(Ty::F128, &wide))
                .unwrap();
            assert_eq!(
                out,
                words(0x2222_0000_0000_0000 + 0x100 + 0x30, 0x1111 + 0x100 + 0x40)
            );
        }
    }
}
