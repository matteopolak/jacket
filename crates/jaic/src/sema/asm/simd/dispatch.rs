//! Routing a SIMD operation to the group that lowers it.
use super::SOp;
use super::helpers::WriteMask;
use crate::sema::asm::vec::VOpd;
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
    pub(super) op: SOp,
}

impl Compiler {
    #[allow(clippy::too_many_arguments)]
    pub(in crate::sema::asm) fn asm_simd(
        &mut self,
        f: &mut FnCtx,
        cx: &mut AsmCtx,
        inst: &AsmInst,
        op: SOp,
        ops: &[VOpd],
        width: u64,
        wm: Option<WriteMask>,
    ) -> Result<()> {
        let span = inst.span;
        let name = inst.mnemonic.name.as_str();
        let Some(&dst) = ops.first() else {
            return err(span, format!("`{name}` needs operands"));
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
            SOp::Bin(..)
            | SOp::RotImm(..)
            | SOp::Unary(..)
            | SOp::Round(..)
            | SOp::Fma(..)
            | SOp::Horizontal(..)
            | SOp::AddSub(..) => self.simd_lanes(f, cx, &c)?,
            SOp::Pmaddwd
            | SOp::Pmaddubsw
            | SOp::Psadbw
            | SOp::Mpsadbw
            | SOp::Madd52(..)
            | SOp::MulEven(..)
            | SOp::DotAcc(..)
            | SOp::Phminposuw => self.simd_multiply(f, cx, &c)?,
            SOp::Unpack(..)
            | SOp::ShufHalf(..)
            | SOp::Shufpd
            | SOp::Palignr
            | SOp::Valign(..)
            | SOp::ByteShift(..)
            | SOp::BlendImm(..)
            | SOp::BlendVar(..)
            | SOp::BlendMask(..)
            | SOp::Pshufb
            | SOp::Perm(..)
            | SOp::PermTwo(..)
            | SOp::PermilImm(..)
            | SOp::Perm2x128
            | SOp::Shuf128(..) => self.simd_shuffle(f, cx, &c)?,
            SOp::Insert(..)
            | SOp::Extract(..)
            | SOp::Pinsr(..)
            | SOp::Pextr(..)
            | SOp::Extractps
            | SOp::Insertps
            | SOp::MovHalf(..)
            | SOp::Movhlps
            | SOp::Movlhps
            | SOp::Dup(..)
            | SOp::Extend(..)
            | SOp::Narrow(..)
            | SOp::Pack(..)
            | SOp::Cvt(..)
            | SOp::Half(..)
            | SOp::Sha(..)
            | SOp::Gf(..) => self.simd_insert(f, cx, &c)?,
            SOp::FCmp(..)
            | SOp::IntCmp(..)
            | SOp::Comis(..)
            | SOp::Pcmpstr(..)
            | SOp::Ptest
            | SOp::Testm(..)
            | SOp::VecToMask(..)
            | SOp::MaskToVec(..)
            | SOp::BroadcastMask(..) => self.simd_compare(f, cx, &c)?,
            SOp::Compress(..)
            | SOp::Expand(..)
            | SOp::Ternlog(..)
            | SOp::Dp(..)
            | SOp::Pclmul(..)
            | SOp::Aes(..)
            | SOp::Scatter(..)
            | SOp::MaskMov(..) => self.simd_masks(f, cx, &c)?,
        }
        Ok(())
    }
}
