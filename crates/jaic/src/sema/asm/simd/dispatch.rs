//! Routing a SIMD operation to the group that lowers it.
use super::VOp;
use super::helpers::WriteMask;
use crate::diagnostics::Code;
use crate::sema::asm::vec::{VOpd, VecCall};
use crate::sema::asm::{AsmCtx, AsmInst, Compiler, FnCtx, Result, Span, Val, err};

#[derive(Clone, Copy)]
pub(super) struct SimdLocals<'a> {
    pub(super) span: Span,
    pub(super) name: &'a str,
    pub(super) dst: VOpd,
    pub(super) tmp: Val,
    pub(super) block: u64,
    pub(super) last_imm: bool,
    pub(super) width: u64,
    pub(super) wm: Option<WriteMask>,
    pub(super) ops: &'a [VOpd],
    pub(super) inst: &'a AsmInst,
    pub(super) op: VOp,
}

impl Compiler {
    pub(in crate::sema::asm) fn asm_simd(
        &mut self,
        f: &mut FnCtx,
        cx: &mut AsmCtx,
        inst: &AsmInst,
        call: &VecCall,
    ) -> Result<()> {
        let (op, ops, width, wm) = (call.op, &call.ops[..], call.width, call.writemask);
        let span = inst.span;
        let name = inst.mnemonic.name.as_str();
        let Some(&dst) = ops.first() else {
            return err(
                Code::AsmOperandCount,
                span,
                format!("`{name}` needs operands"),
            );
        };
        let tmp = f.b.alloca(64, 16);
        f.b.zero(tmp, 64);
        // In-lane operations work per 128-bit block (8 bytes for MMX).
        let block = width.min(16);
        let last_imm = matches!(ops.last(), Some(VOpd::Imm(_)));
        let c = SimdLocals {
            span,
            name,
            dst,
            tmp,
            block,
            last_imm,
            width,
            wm,
            ops,
            inst,
            op,
        };
        match op {
            VOp::Bin(..)
            | VOp::RotImm(..)
            | VOp::Unary(..)
            | VOp::Round(..)
            | VOp::Fma(..)
            | VOp::Horizontal(..)
            | VOp::AddSub(..) => self.simd_lanes(f, cx, &c)?,
            VOp::Pmaddwd
            | VOp::Pmaddubsw
            | VOp::Psadbw
            | VOp::Mpsadbw
            | VOp::Madd52(..)
            | VOp::MulEven(..)
            | VOp::DotAcc(..)
            | VOp::Phminposuw => self.simd_multiply(f, cx, &c)?,
            VOp::Unpack(..)
            | VOp::ShufHalf(..)
            | VOp::Shufpd
            | VOp::Palignr
            | VOp::Valign(..)
            | VOp::ByteShift(..)
            | VOp::BlendImm(..)
            | VOp::BlendVar(..)
            | VOp::BlendMask(..)
            | VOp::Pshufb
            | VOp::Perm(..)
            | VOp::PermTwo(..)
            | VOp::PermilImm(..)
            | VOp::Perm2x128
            | VOp::Shuf128(..) => self.simd_shuffle(f, cx, &c)?,
            VOp::Insert(..)
            | VOp::Extract(..)
            | VOp::Pinsr(..)
            | VOp::Pextr(..)
            | VOp::Extractps
            | VOp::Insertps
            | VOp::MovHalf(..)
            | VOp::Movhlps
            | VOp::Movlhps
            | VOp::Dup(..)
            | VOp::Extend(..)
            | VOp::Narrow(..)
            | VOp::Pack(..)
            | VOp::Cvt(..)
            | VOp::Half(..)
            | VOp::Sha(..)
            | VOp::Gf(..) => self.simd_insert(f, cx, &c)?,
            VOp::FCmp(..)
            | VOp::IntCmp(..)
            | VOp::Comis(..)
            | VOp::Pcmpstr(..)
            | VOp::Ptest
            | VOp::Testm(..)
            | VOp::VecToMask(..)
            | VOp::MaskToVec(..)
            | VOp::BroadcastMask(..) => self.simd_compare(f, cx, &c)?,
            VOp::Compress(..)
            | VOp::Expand(..)
            | VOp::Ternlog(..)
            | VOp::Dp(..)
            | VOp::Pclmul(..)
            | VOp::Aes(..)
            | VOp::Scatter(..)
            | VOp::MaskMov(..) => self.simd_masks(f, cx, &c)?,
            _ => unreachable!("core vector ops are lowered by the vector table"),
        }
        Ok(())
    }
}
