//! Vector (`vec` register) instructions of `#asm`.
//!
//! A vector register is a 64-byte local. Every instruction is lowered lane by
//! lane on memory: each lane of the sources is loaded, combined with ordinary
//! scalar IR, and stored into a 64-byte scratch buffer that is then copied to
//! the destination. Going through the buffer makes aliasing operands
//! (`addps v, v, v`) behave like the hardware.
//!
//! - The vector size is the `.x/.y/.z` suffix (16/32/64 bytes), else the block's
//!   default (`AsmCtx::vec_width`).
//! - A leading `v` (`vaddps`) is accepted: VEX encoding follows the feature set.
//! - Two operands mean `dst op= src`; three mean `dst = a op b`.
//! - A write of N bytes to a register clears the rest of it with VEX encoding
//!   (any AVX feature in the block) and keeps it otherwise, as on hardware.
//! - Operands: vector registers, memory (`[ptr]`), Jai variables (as memory at
//!   their address), general-purpose registers for `movd`/`movq`/`movmsk*`,
//!   integer immediates, and vector-indexed memory (`[base + vindex*4]`) for
//!   gathers.
use super::{
    AsmCtx, AsmInst, AsmMem, AsmOperand, AsmReg, AsmSize, BinOp, CmpOp, Compiler, ConvOp, E,
    EntityKind, Expr, FnCtx, Found, Imm, Intrinsic, Opd, Operand, Result, ScopeId, Span, Ty, UnOp,
    Val, Value, bin, bits, cmp, err, is_neg, konst, ptr_of, resize, select, simd,
};
use crate::diagnostics::Code;

mod groups;
use super::simd::{AesOp, B2, Conv, DupKind, FmaKind, GfOp, Sat, ShaOp, U1};
use crate::ast::AsmMemTerm;

#[derive(Clone, Copy)]
pub(super) enum VOpd {
    /// A vector register (`Ptr` to its 64 bytes).
    Reg(Val),
    /// An op-mask register (`Ptr` to its 64-bit local).
    Mask(Val),
    /// Memory at an `I64` address (also a Jai variable).
    Mem(Val),
    /// A general-purpose register or scalar variable.
    Gpr(Opd),
    Imm(i128),
    /// `[base + vindex*scale + disp]`: `base` includes `disp`, `index` is a register.
    Vsib {
        base: Val,
        index: Val,
        scale: u64,
    },
}

/// Lane-wise binary operations.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Lane {
    Add,
    Sub,
    Mul,
    And,
    Or,
    Xor,
    AndNot,
    CmpEq,
    CmpGt,
    MinS,
    MaxS,
    MinU,
    MaxU,
    FAdd,
    FSub,
    FMul,
    FDiv,
    FMin,
    FMax,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Shift {
    Left,
    Logical,
    Arith,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Cvt {
    /// `cvtdq2ps`
    IntToFloat,
    /// `cvtps2dq`: round to nearest even.
    FloatToInt,
    /// `cvttps2dq`: truncate.
    FloatToIntTrunc,
}

/// Instruction semantics: what a mnemonic does (see `lookup_vector`). The first group is the
/// core vector set, the rest the wider SSE..AVX-512 set (`asm/simd.rs`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(in crate::sema) enum VOp {
    Move,
    /// `movd` / `movq` between a vector register and a scalar (bytes moved).
    MovScalarInt(u64),
    /// `movss` / `movsd`.
    MovScalarFloat(Ty),
    /// Fill every lane of this many bytes from the source's first element.
    Broadcast(u64),
    /// (operation, lane type, scalar: only lane 0, the rest from the first source).
    Lanes(Lane, Ty, bool),
    Sqrt(Ty, bool),
    Abs(Ty),
    Shift(Shift, Ty),
    Pshufd,
    Shufps,
    CvtDq(Cvt),
    /// Sign bits of each lane of this size into a general-purpose register.
    Movmsk(u64),
    /// (index size, element size).
    Gather(u64, u64),
    Nop,

    Bin(B2, Ty),
    /// `prold`/`prorq` by an immediate: (left, lane).
    RotImm(bool, Ty),
    Unary(U1, Ty),
    /// `roundps`/`rndscaleps` (lane, scalar).
    Round(Ty, bool),
    /// (kind, operand order 132/213/231, lane, scalar).
    Fma(FmaKind, u16, Ty, bool),
    /// Pairwise horizontal add/sub (`phaddw`, `haddps`): (subtract, saturate, lane).
    Horizontal(bool, bool, Ty),
    AddSub(Ty),
    Pmaddwd,
    Pmaddubsw,
    Psadbw,
    /// `mpsadbw`: sums of absolute differences of 4-byte groups, picked by the immediate.
    Mpsadbw,
    /// `pmadd52luq` (false) / `pmadd52huq`: 52-bit multiply, low or high half added to qwords.
    Madd52(bool),
    /// `pmuludq` / `pmuldq` (signed).
    MulEven(bool),
    /// `pdpbusd` (bytes) / `pdpwssd` (words): dot products accumulated into dword lanes.
    DotAcc(Ty),
    Phminposuw,
    /// `punpckl*`/`punpckh*`/`unpck*`: (high, lane).
    Unpack(bool, Ty),
    /// `pshufhw` (true) / `pshuflw`.
    ShufHalf(bool),
    Shufpd,
    Palignr,
    /// `alignd`/`alignq`: element-granular `palignr` across the whole vector.
    Valign(Ty),
    /// `pslldq` (true) / `psrldq`.
    ByteShift(bool),
    BlendImm(Ty),
    BlendVar(Ty),
    BlendMask(Ty),
    Pshufb,
    /// `permd`/`permps`/`permb`/`permw`/`permq`/`permpd`: (lane). `permq`/`permpd` with an
    /// immediate permute 64-bit lanes within each 256-bit half.
    Perm(Ty),
    /// `permt2*` (false) / `permi2*` (true).
    PermTwo(Ty, bool),
    /// `permilps`/`permilpd` with an immediate or a vector control.
    PermilImm(Ty),
    Perm2x128,
    /// `shufi32x4`...: 128-bit chunks from both sources (lane size for masking).
    Shuf128(Ty),
    /// Insert / extract a chunk of this many bytes (lane size for masking).
    Insert(u64, Ty),
    Extract(u64, Ty),
    Pinsr(Ty),
    Pextr(Ty),
    Insertps,
    Extractps,
    /// `movhps`/`movhpd` (true) or `movlps`/`movlpd`: 64-bit half loads and stores.
    MovHalf(bool),
    Movhlps,
    Movlhps,
    Dup(DupKind),
    /// `pmovsx*` / `pmovzx*`: (from, to, signed).
    Extend(Ty, Ty, bool),
    /// `pmov*` down-conversions: (from, to, saturation).
    Narrow(Ty, Ty, Sat),
    /// `pack*`: (from lane, unsigned saturation).
    Pack(Ty, bool),
    Cvt(Conv),
    /// `cmpps` & co. with a predicate immediate: (lane, scalar).
    FCmp(Ty, bool),
    /// AVX-512 integer compares into a mask: (lane, unsigned, fixed predicate).
    IntCmp(Ty, bool, Option<u8>),
    /// `comiss`/`ucomiss`/`comisd`/`ucomisd`.
    Comis(Ty),
    Ptest,
    /// `pcmpistri/pcmpestri/pcmpistrm/pcmpestrm`: (explicit lengths, mask result).
    Pcmpstr(bool, bool),
    /// `ptestm*` (false) / `ptestnm*` (true).
    Testm(Ty, bool),
    MaskToVec(Ty),
    VecToMask(Ty),
    BroadcastMask(Ty),
    Compress(Ty),
    Expand(Ty),
    Ternlog(Ty),
    /// `dpps` / `dppd`.
    Dp(Ty),
    /// `pclmulqdq imm` (None) or `pclmul{l,h}q{l,h}qdq` (the immediate).
    Pclmul(Option<u8>),
    Aes(AesOp),
    /// F16C `cvtph2ps` (false) / `cvtps2ph` (true).
    Half(bool),
    Sha(ShaOp),
    Gf(GfOp),
    /// `pscatter*`/`scatter*`: (index size, element size).
    Scatter(u64, u64),
    /// `pmaskmovd/q`, `maskmovps/pd`: masked load or store with a vector mask.
    MaskMov(Ty),
}

/// Lane size in bytes, for EVEX broadcast and masking.
fn elem_size(op: VOp) -> u64 {
    match op {
        VOp::Lanes(_, ty, _) | VOp::Sqrt(ty, _) | VOp::Abs(ty) | VOp::Shift(_, ty) => ty.size(),
        VOp::Broadcast(size) | VOp::Movmsk(size) => size,
        _ => simd::elem_size(op),
    }
}

pub(super) fn lookup_vec(name: &str) -> Option<VOp> {
    let int_lane = |c: &str| -> Option<Ty> {
        Some(match c {
            "b" => Ty::I8,
            "w" => Ty::I16,
            "d" => Ty::I32,
            "q" => Ty::I64,
            _ => return None,
        })
    };
    Some(match name {
        "movups" | "movaps" | "movupd" | "movapd" | "movdqu" | "movdqa" | "movdqu8"
        | "movdqu16" | "movdqu32" | "movdqu64" | "movdqa32" | "movdqa64" | "lddqu" | "movntdq"
        | "movntps" | "movntpd" | "movntdqa" => VOp::Move,
        "movd" => VOp::MovScalarInt(4),
        "movq" => VOp::MovScalarInt(8),
        "movss" => VOp::MovScalarFloat(Ty::F32),
        "movsd" => VOp::MovScalarFloat(Ty::F64),
        "broadcastss" => VOp::Broadcast(4),
        "broadcastsd" => VOp::Broadcast(8),
        "pbroadcastb" => VOp::Broadcast(1),
        "pbroadcastw" => VOp::Broadcast(2),
        "pbroadcastd" => VOp::Broadcast(4),
        "pbroadcastq" => VOp::Broadcast(8),
        "broadcasti32x2" | "broadcastf32x2" => VOp::Broadcast(8),
        "broadcasti128" | "broadcastf128" | "broadcasti32x4" | "broadcastf32x4"
        | "broadcasti64x2" | "broadcastf64x2" => VOp::Broadcast(16),
        "broadcasti32x8" | "broadcastf32x8" | "broadcasti64x4" | "broadcastf64x4" => {
            VOp::Broadcast(32)
        }
        "andps" | "andpd" | "pand" | "pandd" | "pandq" => VOp::Lanes(Lane::And, Ty::I64, false),
        "orps" | "orpd" | "por" | "pord" | "porq" => VOp::Lanes(Lane::Or, Ty::I64, false),
        "xorps" | "xorpd" | "pxor" | "pxord" | "pxorq" => VOp::Lanes(Lane::Xor, Ty::I64, false),
        "andnps" | "andnpd" | "pandn" | "pandnd" | "pandnq" => {
            VOp::Lanes(Lane::AndNot, Ty::I64, false)
        }
        "pmullw" => VOp::Lanes(Lane::Mul, Ty::I16, false),
        "pmulld" => VOp::Lanes(Lane::Mul, Ty::I32, false),
        "pmullq" => VOp::Lanes(Lane::Mul, Ty::I64, false),
        "pshufd" => VOp::Pshufd,
        "shufps" => VOp::Shufps,
        "cvtdq2ps" => VOp::CvtDq(Cvt::IntToFloat),
        "cvtps2dq" => VOp::CvtDq(Cvt::FloatToInt),
        "cvttps2dq" => VOp::CvtDq(Cvt::FloatToIntTrunc),
        "movmskps" => VOp::Movmsk(4),
        "movmskpd" => VOp::Movmsk(8),
        "pmovmskb" => VOp::Movmsk(1),
        "gatherdps" | "pgatherdd" => VOp::Gather(4, 4),
        "gatherdpd" | "pgatherdq" => VOp::Gather(4, 8),
        "gatherqps" | "pgatherqd" => VOp::Gather(8, 4),
        "gatherqpd" | "pgatherqq" => VOp::Gather(8, 8),
        "zeroupper" | "zeroall" | "emms" => VOp::Nop,
        _ => {
            // Float arithmetic: add/sub/mul/div/min/max/sqrt + ps/pd/ss/sd.
            for (op, lane) in [
                ("add", Lane::FAdd),
                ("sub", Lane::FSub),
                ("mul", Lane::FMul),
                ("div", Lane::FDiv),
                ("min", Lane::FMin),
                ("max", Lane::FMax),
                ("sqrt", Lane::FAdd),
            ] {
                let Some(form) = name.strip_prefix(op) else {
                    continue;
                };
                let (ty, scalar) = match form {
                    "ps" => (Ty::F32, false),
                    "pd" => (Ty::F64, false),
                    "ss" => (Ty::F32, true),
                    "sd" => (Ty::F64, true),
                    _ => continue,
                };
                return Some(if op == "sqrt" {
                    VOp::Sqrt(ty, scalar)
                } else {
                    VOp::Lanes(lane, ty, scalar)
                });
            }
            // Integer lanes: p<op><b|w|d|q>.
            for (op, lane) in [
                ("padd", Some(Lane::Add)),
                ("psub", Some(Lane::Sub)),
                ("pcmpeq", Some(Lane::CmpEq)),
                ("pcmpgt", Some(Lane::CmpGt)),
                ("pmins", Some(Lane::MinS)),
                ("pmaxs", Some(Lane::MaxS)),
                ("pminu", Some(Lane::MinU)),
                ("pmaxu", Some(Lane::MaxU)),
                ("pabs", None),
            ] {
                let Some(ty) = name.strip_prefix(op).and_then(int_lane) else {
                    continue;
                };
                return Some(match lane {
                    Some(lane) => VOp::Lanes(lane, ty, false),
                    None => VOp::Abs(ty),
                });
            }
            for (op, kind) in [
                ("psll", Shift::Left),
                ("psrl", Shift::Logical),
                ("psra", Shift::Arith),
            ] {
                if let Some(ty) = name.strip_prefix(op).and_then(int_lane)
                    && ty != Ty::I8
                {
                    return Some(VOp::Shift(kind, ty));
                }
            }
            return simd::lookup_simd(name);
        }
    })
}

/// The class of a fresh `name:` destination of `op` at this vector width.
pub(super) fn dst_class(op: VOp, width: u64) -> &'static str {
    match op {
        VOp::Lanes(Lane::CmpEq | Lane::CmpGt, ..) if width == 64 => "omr",
        // The sign-bit mask lands in a general-purpose register.
        VOp::Movmsk(_) => "gpr",
        _ => simd::dst_class(op, width),
    }
}

pub(super) fn lane_addr(f: &mut FnCtx, base: Val, offset: u64) -> Val {
    if offset == 0 {
        base
    } else {
        f.b.ptr_offset(base, offset)
    }
}

pub(super) fn load_lane(f: &mut FnCtx, base: Val, i: u64, ty: Ty) -> Val {
    let p = lane_addr(f, base, i * ty.size());
    f.b.load(ty, p)
}

pub(super) fn store_lane(f: &mut FnCtx, base: Val, i: u64, ty: Ty, v: Val) {
    let p = lane_addr(f, base, i * ty.size());
    f.b.store(ty, p, v);
}

pub(super) fn bitcast(f: &mut FnCtx, from: Ty, to: Ty, v: Val) -> Val {
    f.b.conv(ConvOp::Bitcast, from, to, v)
}

/// `cond ? a : b` for a float lane.
pub(super) fn fselect(f: &mut FnCtx, ty: Ty, cond: Val, a: Val, b: Val) -> Val {
    let it = Ty::int(ty.size());
    let a = bitcast(f, ty, it, a);
    let b = bitcast(f, ty, it, b);
    let r = select(f, it, cond, a, b);
    bitcast(f, it, ty, r)
}

pub(super) fn all_ones_if(f: &mut FnCtx, ty: Ty, cond: Val) -> Val {
    let ones = konst(f, ty, u64::MAX);
    let zero = konst(f, ty, 0);
    select(f, ty, cond, ones, zero)
}

/// A float math intrinsic computed in `f64` (the interpreter's float intrinsics are 64-bit).
pub(super) fn float_unary(f: &mut FnCtx, op: Intrinsic, ty: Ty, x: Val) -> Val {
    let wide = if ty == Ty::F32 {
        f.b.conv(ConvOp::FExt, Ty::F32, Ty::F64, x)
    } else {
        x
    };
    let r = f.b.intrinsic(op, vec![wide], &[Ty::F64])[0];
    if ty == Ty::F32 {
        f.b.conv(ConvOp::FTrunc, Ty::F64, Ty::F32, r)
    } else {
        r
    }
}

pub(super) fn lane_op(f: &mut FnCtx, op: Lane, ty: Ty, a: Val, b: Val) -> Val {
    match op {
        Lane::Add => bin(f, BinOp::Add, ty, a, b),
        Lane::Sub => bin(f, BinOp::Sub, ty, a, b),
        Lane::Mul => bin(f, BinOp::Mul, ty, a, b),
        Lane::And => bin(f, BinOp::And, ty, a, b),
        Lane::Or => bin(f, BinOp::Or, ty, a, b),
        Lane::Xor => bin(f, BinOp::Xor, ty, a, b),
        Lane::AndNot => {
            let na = f.b.un(UnOp::Not, ty, a);
            bin(f, BinOp::And, ty, na, b)
        }
        Lane::CmpEq => {
            let c = cmp(f, CmpOp::Eq, ty, a, b);
            all_ones_if(f, ty, c)
        }
        Lane::CmpGt => {
            let c = cmp(f, CmpOp::SGt, ty, a, b);
            all_ones_if(f, ty, c)
        }
        Lane::MinS | Lane::MaxS | Lane::MinU | Lane::MaxU => {
            let op = match op {
                Lane::MinS => CmpOp::SLt,
                Lane::MaxS => CmpOp::SGt,
                Lane::MinU => CmpOp::ULt,
                _ => CmpOp::UGt,
            };
            let c = cmp(f, op, ty, a, b);
            select(f, ty, c, a, b)
        }
        Lane::FAdd => bin(f, BinOp::FAdd, ty, a, b),
        Lane::FSub => bin(f, BinOp::FSub, ty, a, b),
        Lane::FMul => bin(f, BinOp::FMul, ty, a, b),
        Lane::FDiv => bin(f, BinOp::FDiv, ty, a, b),
        // As on hardware: the second operand when either is NaN or both are zero.
        Lane::FMin => {
            let c = cmp(f, CmpOp::FLt, ty, a, b);
            fselect(f, ty, c, a, b)
        }
        Lane::FMax => {
            let c = cmp(f, CmpOp::FGt, ty, a, b);
            fselect(f, ty, c, a, b)
        }
    }
}

/// `f32` → `s32` the way `cvtps2dq` / `cvttps2dq` do: NaN and out-of-range give `0x8000_0000`.
pub(super) fn float_to_s32(f: &mut FnCtx, x: Val, mode: char) -> Val {
    let x = f.b.conv(ConvOp::FExt, Ty::F32, Ty::F64, x);
    let r = if matches!(mode, 'z' | 'd' | 'u') {
        let op = match mode {
            'd' => Intrinsic::Floor,
            'u' => Intrinsic::Ceil,
            _ => Intrinsic::Trunc,
        };
        f.b.intrinsic(op, vec![x], &[Ty::F64])[0]
    } else {
        // Round half to even: `Round` rounds half away from zero, so step back on odd ties.
        let away = f.b.intrinsic(Intrinsic::Round, vec![x], &[Ty::F64])[0];
        let whole = f.b.intrinsic(Intrinsic::Trunc, vec![x], &[Ty::F64])[0];
        let frac = bin(f, BinOp::FSub, Ty::F64, x, whole);
        let frac = f.b.intrinsic(Intrinsic::Fabs, vec![frac], &[Ty::F64])[0];
        let half = f.b.fconst(Ty::F64, 0.5);
        let tie = cmp(f, CmpOp::FEq, Ty::F64, frac, half);
        let as_int = f.b.conv(ConvOp::FToS, Ty::F64, Ty::I64, away);
        let one = konst(f, Ty::I64, 1);
        let low = bin(f, BinOp::And, Ty::I64, as_int, one);
        let odd = cmp(f, CmpOp::Eq, Ty::I64, low, one);
        let fix = bin(f, BinOp::And, Ty::I8, tie, odd);
        let neg = {
            let z = f.b.fconst(Ty::F64, 0.0);
            cmp(f, CmpOp::FLt, Ty::F64, x, z)
        };
        let up = f.b.fconst(Ty::F64, 1.0);
        let down = f.b.fconst(Ty::F64, -1.0);
        let step = fselect(f, Ty::F64, neg, down, up);
        let stepped = bin(f, BinOp::FSub, Ty::F64, away, step);
        fselect(f, Ty::F64, fix, stepped, away)
    };
    let lo = f.b.fconst(Ty::F64, -2_147_483_648.0);
    let hi = f.b.fconst(Ty::F64, 2_147_483_648.0);
    let ge = cmp(f, CmpOp::FGe, Ty::F64, r, lo);
    let lt = cmp(f, CmpOp::FLt, Ty::F64, r, hi);
    let ok = bin(f, BinOp::And, Ty::I8, ge, lt);
    // Clamp before converting so the conversion itself stays defined.
    let zero = f.b.fconst(Ty::F64, 0.0);
    let safe = fselect(f, Ty::F64, ok, r, zero);
    let v = f.b.conv(ConvOp::FToS, Ty::F64, Ty::I32, safe);
    let min = konst(f, Ty::I32, 0x8000_0000);
    select(f, Ty::I32, ok, v, min)
}

/// What the groups of `asm_vec_inst` share: the evaluated operands, the vector size and the
/// write mask.
pub(super) struct VecCall<'a> {
    pub(super) op: VOp,
    pub(super) span: Span,
    pub(super) name: &'a str,
    pub(super) width: u64,
    pub(super) ops: Vec<VOpd>,
    pub(super) es: u64,
    pub(super) writemask: Option<simd::WriteMask>,
    pub(super) masked: Option<(Val, bool, Val, Val)>,
    pub(super) tmp: Val,
}

impl VecCall<'_> {
    /// Checks the operand count against `lo..=hi`.
    fn want(&self, lo: usize, hi: usize) -> Result<()> {
        let n = self.ops.len();
        if n < lo || n > hi {
            return err(
                Code::AsmOperandCount,
                self.span,
                format!("`{}` takes {lo} to {hi} operands, found {n}", self.name),
            );
        }
        Ok(())
    }
}

impl Compiler {
    /// Lower `inst` if it is a vector instruction; false when `base` is not one.
    /// `asm_vec_inst`, then the store of a float-variable destination back to its variable.
    pub(super) fn asm_vec_inst_floats(
        &mut self,
        f: &mut FnCtx,
        cx: &mut AsmCtx,
        inst: &AsmInst,
        base: &str,
    ) -> Result<bool> {
        cx.float_vars.clear();
        cx.float_dst = None;
        let done = self.asm_vec_inst(f, cx, inst, base)?;
        if let Some((reg, addr, size)) = cx.float_dst.take() {
            f.b.copy(addr, reg, size);
        }
        cx.float_vars.clear();
        Ok(done)
    }

    /// Evaluates the operands, size and write mask of a vector instruction.
    fn vec_call<'a>(
        &mut self,
        f: &mut FnCtx,
        cx: &mut AsmCtx,
        inst: &'a AsmInst,
        op: VOp,
    ) -> Result<VecCall<'a>> {
        let span = inst.span;
        let name = inst.mnemonic.name.as_str();
        let width = match &inst.size {
            None => cx.vec_width,
            Some(AsmSize::Suffix(s)) => match s.name.as_str() {
                "x" => 16,
                "y" => 32,
                "z" => 64,
                // `.q` on a lane operation is the 64-bit MMX (`str` register) form.
                "q" if matches!(op, VOp::Lanes(..) | VOp::Abs(_) | VOp::Shift(..))
                    || simd::mmx_capable(op) =>
                {
                    8
                }
                // `movq.q` / `movd.d` and friends: the vector size is the default.
                "b" | "w" | "d" | "q" | "8" | "16" | "32" | "64" => cx.vec_width,
                other => {
                    return err(
                        Code::AsmVectorOperand,
                        s.span,
                        format!("unknown vector size suffix `.{other}`"),
                    );
                }
            },
            Some(AsmSize::Dynamic(e)) => {
                return err(
                    Code::AsmVectorOperand,
                    e.span,
                    "vector instructions take a .x/.y/.z size suffix",
                );
            }
        };
        let n = inst.operands.len();
        // AVX-512 compares write a mask register; a fresh `name:` destination is a mask
        // when the instruction produces one (always for 512-bit compares).
        let dst_class = dst_class(op, width);
        let mut ops: Vec<VOpd> = Vec::with_capacity(n);
        for (i, o) in inst.operands.iter().enumerate() {
            let class = if i == 0 {
                dst_class
            } else {
                "vec"
            };
            let before = cx.float_vars.len();
            ops.push(self.vec_operand_class(f, cx, o, class)?);
            if i == 0 && cx.float_vars.len() > before {
                cx.float_dst = cx.float_vars.last().copied();
            }
        }
        let op = match (op, ops.first()) {
            (VOp::Lanes(lane @ (Lane::CmpEq | Lane::CmpGt), ty, false), Some(VOpd::Mask(_))) => {
                VOp::IntCmp(
                    ty,
                    false,
                    if lane == Lane::CmpEq {
                        Some(0)
                    } else {
                        Some(6)
                    },
                )
            }
            _ => op,
        };
        let es = elem_size(op);
        for (i, o) in inst.operands.iter().enumerate() {
            if let AsmOperand::Mem(m) = o
                && m.broadcast
                && let VOpd::Mem(addr) = ops[i]
            {
                let src = ptr_of(f, addr);
                let buf = f.b.alloca(64, 16);
                let bs = match op {
                    // The affine matrix operand is one 64-bit matrix per qword.
                    VOp::Gf(GfOp::Affine(_)) => 8,
                    _ => es,
                };
                for k in 0..64 / bs {
                    let p = lane_addr(f, buf, k * bs);
                    f.b.copy(p, src, bs);
                }
                ops[i] = VOpd::Reg(buf);
            }
        }
        // `{k}` masking: remember the destination, then merge or zero the masked-off lanes.
        // Instructions that write a mask register AND their result with it instead.
        let writemask = match &inst.evex.mask {
            Some((m, zeroing)) => {
                let (bits, reg) = match self.vec_operand_class(f, cx, m, "omr")? {
                    VOpd::Mask(p) | VOpd::Reg(p) => (f.b.load(Ty::I64, p), Some(p)),
                    VOpd::Gpr(opd) => (self.asm_read(f, opd, Ty::I64, span)?, None),
                    _ => return err(Code::AsmMaskOperand, span, "expected a mask register"),
                };
                Some(simd::WriteMask {
                    bits,
                    zeroing: *zeroing,
                    reg,
                })
            }
            None => None,
        };
        let self_masked = match op {
            VOp::Gather(..) => true,
            _ => simd::consumes_mask(op),
        };
        let masked = match (writemask, ops.first()) {
            (Some(_), _) if self_masked => None,
            (
                Some(simd::WriteMask {
                    bits: mask,
                    zeroing,
                    ..
                }),
                Some(&(VOpd::Reg(_) | VOpd::Mem(_))),
            ) => {
                let dst = self.vec_ptr(f, ops[0], span)?;
                let old = f.b.alloca(64, 16);
                f.b.copy(old, dst, width);
                Some((mask, zeroing, dst, old))
            }
            (Some(_), Some(VOpd::Mask(_))) => None,
            (Some(_), _) => {
                return err(
                    Code::AsmVectorOperand,
                    span,
                    "a masked instruction needs a vector destination",
                );
            }
            _ => None,
        };
        let tmp = f.b.alloca(64, 16);
        Ok(VecCall {
            op,
            span,
            name,
            width,
            ops,
            es,
            writemask,
            masked,
            tmp,
        })
    }

    pub(super) fn asm_vec_inst(
        &mut self,
        f: &mut FnCtx,
        cx: &mut AsmCtx,
        inst: &AsmInst,
        base: &str,
    ) -> Result<bool> {
        let op = match lookup_vec(base) {
            Some(op) => op,
            None => match base.strip_prefix('v').and_then(lookup_vec) {
                Some(op) => op,
                None => return Ok(false),
            },
        };
        let c = self.vec_call(f, cx, inst, op)?;
        let op = c.op;
        match op {
            VOp::Move
            | VOp::MovScalarInt(..)
            | VOp::MovScalarFloat(..)
            | VOp::Broadcast(..)
            | VOp::Movmsk(..) => self.vec_moves(f, cx, &c, op)?,
            VOp::Lanes(..) | VOp::Sqrt(..) | VOp::Abs(..) | VOp::Shift(..) | VOp::CvtDq(..) => {
                self.vec_lane_ops(f, cx, inst, &c, op)?;
            }
            VOp::Pshufd | VOp::Shufps => self.vec_shuffles(f, cx, &c, op)?,
            VOp::Gather(..) => self.vec_gather(f, cx, &c, op)?,
            VOp::Nop => c.want(0, 0)?,
            _ => self.asm_simd(f, cx, inst, &c)?,
        }
        if let Some((mask, zeroing, dst, old)) = c.masked {
            let (es, width) = (c.es, c.width);
            let one = konst(f, Ty::I64, 1);
            let ity = Ty::int(es);
            for k in 0..width / es {
                let at = konst(f, Ty::I64, k);
                let bit = bin(f, BinOp::LShr, Ty::I64, mask, at);
                let bit = bin(f, BinOp::And, Ty::I64, bit, one);
                let zero = konst(f, Ty::I64, 0);
                let on = cmp(f, CmpOp::Ne, Ty::I64, bit, zero);
                let new = load_lane(f, dst, k, ity);
                let prev = if zeroing && !matches!(c.ops[0], VOpd::Mem(_)) {
                    konst(f, ity, 0)
                } else {
                    load_lane(f, old, k, ity)
                };
                let r = select(f, ity, on, new, prev);
                store_lane(f, dst, k, ity, r);
            }
        }
        Ok(true)
    }

    /// The two sources of a lane operation: `(dst, dst, src)` for two operands.
    pub(super) fn vec_sources(
        &mut self,
        f: &mut FnCtx,
        ops: &[VOpd],
        span: Span,
    ) -> Result<(Val, Val)> {
        if ops.len() == 3 {
            Ok((
                self.vec_ptr(f, ops[1], span)?,
                self.vec_ptr(f, ops[2], span)?,
            ))
        } else {
            Ok((
                self.vec_ptr(f, ops[0], span)?,
                self.vec_ptr(f, ops[1], span)?,
            ))
        }
    }

    pub(super) fn vec_imm(&self, o: VOpd, span: Span) -> Result<u64> {
        match o {
            VOpd::Imm(v) if (0..=255).contains(&v) => Ok(v as u64),
            _ => err(
                Code::AsmImmediate,
                span,
                "expected an 8-bit immediate operand",
            ),
        }
    }

    /// A pointer to the bytes of a register or memory operand.
    pub(super) fn vec_ptr(&mut self, f: &mut FnCtx, o: VOpd, span: Span) -> Result<Val> {
        match o {
            VOpd::Reg(p) => Ok(p),
            VOpd::Mem(addr) => Ok(ptr_of(f, addr)),
            VOpd::Gpr(Opd::Reg(r)) => Ok(r.addr),
            _ => err(
                Code::AsmVectorOperand,
                span,
                "expected a vector register or memory operand",
            ),
        }
    }

    /// Write `size` bytes from `src` to a register or memory destination.
    pub(super) fn vec_store(
        &mut self,
        f: &mut FnCtx,
        cx: &AsmCtx,
        dst: VOpd,
        src: Val,
        size: u64,
        span: Span,
    ) -> Result<()> {
        match dst {
            VOpd::Reg(p) => {
                f.b.copy(p, src, size);
                if cx.vex && size < 64 {
                    let rest = lane_addr(f, p, size);
                    f.b.zero(rest, 64 - size);
                }
                Ok(())
            }
            VOpd::Mem(addr) => {
                let p = ptr_of(f, addr);
                f.b.copy(p, src, size);
                Ok(())
            }
            _ => err(
                Code::AsmVectorOperand,
                span,
                "the destination must be a vector register or memory",
            ),
        }
    }

    pub(super) fn vec_scalar_read(
        &mut self,
        f: &mut FnCtx,
        o: VOpd,
        ty: Ty,
        span: Span,
    ) -> Result<Val> {
        match o {
            VOpd::Gpr(opd) => self.asm_read(f, opd, ty, span),
            VOpd::Mem(addr) => {
                let p = ptr_of(f, addr);
                Ok(f.b.load(ty, p))
            }
            VOpd::Imm(v) => Ok(konst(f, ty, v as u64)),
            _ => err(
                Code::AsmExpectedGeneralPurpose,
                span,
                "expected a general-purpose register or memory operand",
            ),
        }
    }

    pub(super) fn vec_scalar_write(
        &mut self,
        f: &mut FnCtx,
        o: VOpd,
        ty: Ty,
        v: Val,
        span: Span,
    ) -> Result<()> {
        match o {
            VOpd::Gpr(opd) => self.asm_write(f, opd, ty, v, span),
            VOpd::Mem(addr) => {
                let p = ptr_of(f, addr);
                f.b.store(ty, p, v);
                Ok(())
            }
            _ => err(
                Code::AsmExpectedGeneralPurposeRegister,
                span,
                "expected a general-purpose register or memory destination",
            ),
        }
    }

    /// A vector instruction operand; `name:` declares a register of `class`.
    pub(super) fn vec_operand_class(
        &mut self,
        f: &mut FnCtx,
        cx: &mut AsmCtx,
        o: &AsmOperand,
        class: &str,
    ) -> Result<VOpd> {
        match o {
            AsmOperand::Decl(d) if d.colon => {
                let (kind, addr) = self.asm_declare(f, cx, d, class)?;
                match kind {
                    AsmReg::Vec => Ok(VOpd::Reg(addr)),
                    AsmReg::Mask => Ok(VOpd::Mask(addr)),
                    AsmReg::Gpr => Ok(VOpd::Gpr(self.asm_gpr_opd(kind, addr, d.name.span)?)),
                }
            }
            AsmOperand::Decl(d) => err(
                Code::AsmRegisterPin,
                d.name.span,
                "a register pin is only valid as a separate statement",
            ),
            AsmOperand::Mem(m) => self.vec_mem(f, cx, m),
            AsmOperand::Value(e) => {
                if let Some((kind, addr)) = self.asm_reg_named(cx.scope, e)? {
                    return match kind {
                        AsmReg::Vec => Ok(VOpd::Reg(addr)),
                        AsmReg::Mask => Ok(VOpd::Mask(addr)),
                        AsmReg::Gpr => Ok(VOpd::Gpr(self.asm_value(f, cx.scope, e, 0)?)),
                    };
                }
                let op = self.check_expr(f, cx.scope, e, None)?;
                match op {
                    // A Jai variable: integers act as general-purpose registers, floats as
                    // vector registers (their low bytes), anything else (arrays, vectors) is
                    // memory at its address.
                    Operand::Place {
                        ty,
                        addr,
                    } => match self.ir_ty(ty) {
                        Some(t) if !t.is_float() => {
                            Ok(VOpd::Gpr(self.asm_place(ty, addr, true, e.span)?))
                        }
                        Some(t) => {
                            let size = t.size();
                            let reg = f.b.alloca(64, 16);
                            f.b.zero(reg, 64);
                            f.b.copy(reg, addr, size);
                            cx.float_vars.push((reg, addr, size));
                            Ok(VOpd::Reg(reg))
                        }
                        None => Ok(VOpd::Mem(bitcast(f, Ty::Ptr, Ty::I64, addr))),
                    },
                    Operand::Const {
                        value: Value::Int(v),
                        ..
                    } => Ok(VOpd::Imm(v)),
                    _ => Ok(VOpd::Gpr(self.asm_value(f, cx.scope, e, 0)?)),
                }
            }
        }
    }

    /// The register an identifier names, if it is one declared by `#asm`.
    pub(in crate::sema) fn asm_reg_named(
        &mut self,
        scope: ScopeId,
        e: &Expr,
    ) -> Result<Option<(AsmReg, Val)>> {
        if let E::Ident(name) = &e.kind
            && let Found::Entities(ids) = self.lookup_full(scope, *name)?
            && let Some(&id) = ids.last()
            && let Some(&kind) = self.pr.asm_regs.get(&id)
            && let EntityKind::Local {
                addr, ..
            } = self.entity(id).kind
        {
            return Ok(Some((kind, addr)));
        }
        Ok(None)
    }

    /// A memory operand, which may be vector-indexed (`[base + vindex*4]`).
    fn vec_mem(&mut self, f: &mut FnCtx, cx: &AsmCtx, m: &AsmMem) -> Result<VOpd> {
        let mut vindex = None;
        let mut rest: Vec<AsmMemTerm> = Vec::new();
        for term in &m.terms {
            if let Some((AsmReg::Vec, addr)) = self.asm_reg_named(cx.scope, &term.value)? {
                let scale = match &term.scale {
                    Some(e) => match self.asm_value(f, cx.scope, e, 0)? {
                        Opd::Imm(Imm::Int(s)) if matches!(s, 1 | 2 | 4 | 8) => s as u64,
                        _ => {
                            return err(
                                Code::AsmVectorOperand,
                                e.span,
                                "a vector index scale must be 1, 2, 4 or 8",
                            );
                        }
                    },
                    None => 1,
                };
                if term.negate || vindex.is_some() {
                    return err(
                        Code::AsmVectorOperand,
                        term.value.span,
                        "a memory operand takes one vector index",
                    );
                }
                vindex = Some((addr, scale));
            } else {
                rest.push(term.clone());
            }
        }
        let base_mem = AsmMem {
            terms: rest,
            ..m.clone()
        };
        let base = self.asm_mem(f, cx.scope, &base_mem)?;
        Ok(match vindex {
            Some((index, scale)) => VOpd::Vsib {
                base,
                index,
                scale,
            },
            None => VOpd::Mem(base),
        })
    }
}
