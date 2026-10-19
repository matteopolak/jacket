//! Reading operands and writing results of SIMD instructions.
use super::helpers::WriteMask;
use crate::sema::asm::vec::{VOpd, lane_addr};
use crate::sema::asm::{
    AsmCtx, BinOp, Compiler, FnCtx, Result, Span, Ty, Val, bin, err, konst, resize,
};

impl Compiler {
    /// Sources and immediate of `dst, src..., [imm8]`. With one source fewer than
    /// `nsrc`, the destination is also the first source.
    pub(super) fn simd_args(
        &mut self,
        f: &mut FnCtx,
        ops: &[VOpd],
        nsrc: usize,
        imm: bool,
        span: Span,
    ) -> Result<(Vec<Val>, u64)> {
        if ops.is_empty() {
            return err(span, "missing destination operand");
        }
        let mut rest = &ops[1..];
        let mut k = 0;
        if imm {
            match rest.last() {
                Some(&VOpd::Imm(v)) if (0..=255).contains(&v) => {
                    k = v as u64;
                    rest = &rest[..rest.len() - 1];
                }
                _ => return err(span, "expected an 8-bit immediate as the last operand"),
            }
        }
        let mut srcs = Vec::with_capacity(nsrc);
        if rest.len() + 1 == nsrc {
            srcs.push(self.vec_ptr(f, ops[0], span)?);
        } else if rest.len() != nsrc {
            return err(
                span,
                format!(
                    "expected {nsrc} source operand(s){}",
                    if imm {
                        " and an immediate"
                    } else {
                        ""
                    }
                ),
            );
        }
        for &o in rest {
            srcs.push(self.vec_ptr(f, o, span)?);
        }
        Ok((srcs, k))
    }

    /// Write `bytes` of the scratch buffer to the destination (at least the 16 bytes of
    /// an xmm register when the destination is a register).
    pub(super) fn simd_out(
        &mut self,
        f: &mut FnCtx,
        cx: &AsmCtx,
        dst: VOpd,
        tmp: Val,
        bytes: u64,
        span: Span,
    ) -> Result<()> {
        let size = match dst {
            VOpd::Reg(_) if bytes < 16 => {
                let rest = lane_addr(f, tmp, bytes);
                f.b.zero(rest, 16 - bytes);
                16
            }
            _ => bytes,
        };
        self.vec_store(f, cx, dst, tmp, size, span)
    }

    /// Write lane flags (bit i = lane i) to a mask destination, `ANDed` with the write mask.
    pub(super) fn simd_mask_out(
        &mut self,
        f: &mut FnCtx,
        dst: VOpd,
        flags: Vec<Val>,
        wm: Option<WriteMask>,
        span: Span,
    ) -> Result<()> {
        let mut m = konst(f, Ty::I64, 0);
        for (i, flag) in flags.into_iter().enumerate() {
            let b = resize(f, flag, Ty::I8, Ty::I64, false);
            let at = konst(f, Ty::I64, i as u64);
            let b = bin(f, BinOp::Shl, Ty::I64, b, at);
            m = bin(f, BinOp::Or, Ty::I64, m, b);
        }
        if let Some(wm) = wm {
            m = bin(f, BinOp::And, Ty::I64, m, wm.bits);
        }
        match dst {
            VOpd::Mask(p) => {
                f.b.store(Ty::I64, p, m);
                Ok(())
            }
            _ => err(span, "the destination must be a mask register"),
        }
    }
}
