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
use super::*;
use crate::ast::AsmMemTerm;

#[derive(Clone, Copy)]
enum VOpd {
    /// A vector register (`Ptr` to its 64 bytes).
    Reg(Val),
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
#[derive(Clone, Copy, PartialEq, Eq)]
enum Lane {
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum Shift {
    Left,
    Logical,
    Arith,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Cvt {
    /// `cvtdq2ps`
    IntToFloat,
    /// `cvtps2dq`: round to nearest even.
    FloatToInt,
    /// `cvttps2dq`: truncate.
    FloatToIntTrunc,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum VOp {
    Move,
    /// `movd` / `movq` between a vector register and a scalar (bytes moved).
    MovScalarInt(u64),
    /// `movss` / `movsd`.
    MovScalarFloat(Ty),
    /// Fill every lane of this many bytes from the source's first element.
    Broadcast(u64),
    /// (operation, lane type, scalar: only lane 0, the rest from the first source).
    Bin(Lane, Ty, bool),
    Sqrt(Ty, bool),
    Abs(Ty),
    Shift(Shift, Ty),
    Pshufd,
    Shufps,
    Cvt(Cvt),
    /// Sign bits of each lane of this size into a general-purpose register.
    Movmsk(u64),
    /// (index size, element size).
    Gather(u64, u64),
    Nop,
    /// `kmovb/w/d/q`: mask registers are 64-bit vector-class locals (bytes moved).
    Kmov(u64),
}

/// Lane size in bytes, for EVEX broadcast and masking.
fn elem_size(op: VOp) -> u64 {
    match op {
        VOp::Bin(_, ty, _) | VOp::Sqrt(ty, _) | VOp::Abs(ty) | VOp::Shift(_, ty) => ty.size(),
        VOp::Broadcast(size) | VOp::Movmsk(size) => size,
        _ => 4,
    }
}

fn lookup_vec(name: &str) -> Option<VOp> {
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
        "broadcasti128" | "broadcastf128" | "broadcasti32x4" | "broadcastf32x4" => {
            VOp::Broadcast(16)
        }
        "andps" | "andpd" | "pand" | "pandd" | "pandq" => VOp::Bin(Lane::And, Ty::I64, false),
        "orps" | "orpd" | "por" | "pord" | "porq" => VOp::Bin(Lane::Or, Ty::I64, false),
        "xorps" | "xorpd" | "pxor" | "pxord" | "pxorq" => VOp::Bin(Lane::Xor, Ty::I64, false),
        "andnps" | "andnpd" | "pandn" | "pandnd" | "pandnq" => {
            VOp::Bin(Lane::AndNot, Ty::I64, false)
        }
        "pmullw" => VOp::Bin(Lane::Mul, Ty::I16, false),
        "pmulld" => VOp::Bin(Lane::Mul, Ty::I32, false),
        "pmullq" => VOp::Bin(Lane::Mul, Ty::I64, false),
        "pshufd" => VOp::Pshufd,
        "shufps" => VOp::Shufps,
        "cvtdq2ps" => VOp::Cvt(Cvt::IntToFloat),
        "cvtps2dq" => VOp::Cvt(Cvt::FloatToInt),
        "cvttps2dq" => VOp::Cvt(Cvt::FloatToIntTrunc),
        "movmskps" => VOp::Movmsk(4),
        "movmskpd" => VOp::Movmsk(8),
        "pmovmskb" => VOp::Movmsk(1),
        "gatherdps" | "pgatherdd" => VOp::Gather(4, 4),
        "gatherdpd" | "pgatherdq" => VOp::Gather(4, 8),
        "gatherqps" | "pgatherqd" => VOp::Gather(8, 4),
        "gatherqpd" | "pgatherqq" => VOp::Gather(8, 8),
        "kmovb" => VOp::Kmov(1),
        "kmovw" => VOp::Kmov(2),
        "kmovd" => VOp::Kmov(4),
        "kmovq" => VOp::Kmov(8),
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
                    VOp::Bin(lane, ty, scalar)
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
                    Some(lane) => VOp::Bin(lane, ty, false),
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
            return None;
        }
    })
}

fn lane_addr(f: &mut FnCtx, base: Val, offset: u64) -> Val {
    if offset == 0 {
        base
    } else {
        f.b.ptr_offset(base, offset)
    }
}
fn load_lane(f: &mut FnCtx, base: Val, i: u64, ty: Ty) -> Val {
    let p = lane_addr(f, base, i * ty.size());
    f.b.load(ty, p)
}
fn store_lane(f: &mut FnCtx, base: Val, i: u64, ty: Ty, v: Val) {
    let p = lane_addr(f, base, i * ty.size());
    f.b.store(ty, p, v);
}
fn bitcast(f: &mut FnCtx, from: Ty, to: Ty, v: Val) -> Val {
    f.b.conv(ConvOp::Bitcast, from, to, v)
}
/// `cond ? a : b` for a float lane.
fn fselect(f: &mut FnCtx, ty: Ty, cond: Val, a: Val, b: Val) -> Val {
    let it = Ty::int(ty.size());
    let a = bitcast(f, ty, it, a);
    let b = bitcast(f, ty, it, b);
    let r = select(f, it, cond, a, b);
    bitcast(f, it, ty, r)
}
fn all_ones_if(f: &mut FnCtx, ty: Ty, cond: Val) -> Val {
    let ones = konst(f, ty, u64::MAX);
    let zero = konst(f, ty, 0);
    select(f, ty, cond, ones, zero)
}
/// A float math intrinsic computed in `f64` (the interpreter's float intrinsics are 64-bit).
fn float_unary(f: &mut FnCtx, op: Intrinsic, ty: Ty, x: Val) -> Val {
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

fn lane_op(f: &mut FnCtx, op: Lane, ty: Ty, a: Val, b: Val) -> Val {
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
fn float_to_s32(f: &mut FnCtx, x: Val, mode: char) -> Val {
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
    let lo = f.b.fconst(Ty::F64, -2147483648.0);
    let hi = f.b.fconst(Ty::F64, 2147483648.0);
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

impl Compiler {
    /// Lower `inst` if it is a vector instruction; false when `base` is not one.
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
        let span = inst.span;
        let name = inst.mnemonic.name.as_str();
        let width = match &inst.size {
            None => cx.vec_width,
            Some(AsmSize::Suffix(s)) => match s.name.as_str() {
                "x" => 16,
                "y" => 32,
                "z" => 64,
                // `movq.q` / `movd.d` and friends: the vector size is the default.
                "b" | "w" | "d" | "q" | "8" | "16" | "32" | "64" => cx.vec_width,
                other => {
                    return err(s.span, format!("unknown vector size suffix '.{other}'"));
                }
            },
            Some(AsmSize::Dynamic(e)) => {
                return err(e.span, "vector instructions take a .x/.y/.z size suffix");
            }
        };
        let n = inst.operands.len();
        let want = |lo: usize, hi: usize| -> Result<()> {
            if n < lo || n > hi {
                return err(
                    span,
                    format!("'{name}' takes {lo} to {hi} operands, found {n}"),
                );
            }
            Ok(())
        };
        let mut ops: Vec<VOpd> = Vec::with_capacity(n);
        for o in &inst.operands {
            ops.push(self.vec_operand(f, cx, o)?);
        }
        let es = elem_size(op);
        for (i, o) in inst.operands.iter().enumerate() {
            if let AsmOperand::Mem(m) = o
                && m.broadcast
                && let VOpd::Mem(addr) = ops[i]
            {
                let src = ptr_of(f, addr);
                let buf = f.b.alloca(64, 16);
                for k in 0..64 / es {
                    let p = lane_addr(f, buf, k * es);
                    f.b.copy(p, src, es);
                }
                ops[i] = VOpd::Reg(buf);
            }
        }
        // `{k}` masking: remember the destination, then merge or zero the masked-off lanes.
        let masked = match (&inst.evex.mask, ops.first()) {
            (Some((m, zeroing)), Some(&VOpd::Reg(dst))) => {
                let mask = match self.vec_operand(f, cx, m)? {
                    VOpd::Reg(p) => f.b.load(Ty::I64, p),
                    VOpd::Gpr(opd) => self.asm_read(f, opd, Ty::I64, span)?,
                    _ => return err(span, "expected a mask register"),
                };
                let old = f.b.alloca(64, 16);
                f.b.copy(old, dst, 64);
                Some((mask, *zeroing, dst, old))
            }
            (Some(_), _) => return err(span, "a masked instruction needs a vector destination"),
            _ => None,
        };
        let tmp = f.b.alloca(64, 16);
        match op {
            VOp::Nop => want(0, 0)?,
            VOp::Kmov(size) => {
                want(2, 2)?;
                let ty = Ty::int(size);
                let v = match ops[1] {
                    VOpd::Reg(p) => f.b.load(ty, p),
                    src => self.vec_scalar_read(f, src, ty, span)?,
                };
                match ops[0] {
                    VOpd::Reg(p) => {
                        f.b.zero(p, 8);
                        f.b.store(ty, p, v);
                    }
                    dst => self.vec_scalar_write(f, dst, ty, v, span)?,
                }
            }
            VOp::Move => {
                want(2, 2)?;
                let src = self.vec_ptr(f, ops[1], span)?;
                f.b.copy(tmp, src, width);
                self.vec_store(f, cx, ops[0], tmp, width, span)?;
            }
            VOp::MovScalarInt(size) => {
                want(2, 2)?;
                let it = Ty::int(size);
                match (ops[0], ops[1]) {
                    (VOpd::Reg(dst), VOpd::Reg(src)) => {
                        f.b.zero(tmp, 16);
                        f.b.copy(tmp, src, size);
                        self.vec_store(f, cx, VOpd::Reg(dst), tmp, 16, span)?;
                    }
                    (VOpd::Reg(_), src) => {
                        let v = self.vec_scalar_read(f, src, it, span)?;
                        f.b.zero(tmp, 16);
                        f.b.store(it, tmp, v);
                        self.vec_store(f, cx, ops[0], tmp, 16, span)?;
                    }
                    (dst, VOpd::Reg(src)) => {
                        let v = f.b.load(it, src);
                        self.vec_scalar_write(f, dst, it, v, span)?;
                    }
                    _ => return err(span, format!("'{name}' needs a vector register operand")),
                }
            }
            VOp::MovScalarFloat(ty) => {
                want(2, 3)?;
                let size = ty.size();
                match (n, ops[0], ops[1]) {
                    (2, VOpd::Reg(dst), VOpd::Reg(src)) => {
                        // Register to register keeps the destination's other lanes.
                        f.b.copy(tmp, dst, 16);
                        f.b.copy(tmp, src, size);
                        self.vec_store(f, cx, ops[0], tmp, 16, span)?;
                    }
                    (2, VOpd::Reg(_), src) => {
                        let p = self.vec_ptr(f, src, span)?;
                        f.b.zero(tmp, 16);
                        f.b.copy(tmp, p, size);
                        self.vec_store(f, cx, ops[0], tmp, 16, span)?;
                    }
                    (2, dst, VOpd::Reg(src)) => {
                        let p = self.vec_ptr(f, dst, span)?;
                        f.b.copy(p, src, size);
                    }
                    (3, VOpd::Reg(_), a) => {
                        let a = self.vec_ptr(f, a, span)?;
                        let b = self.vec_ptr(f, ops[2], span)?;
                        f.b.copy(tmp, a, 16);
                        f.b.copy(tmp, b, size);
                        self.vec_store(f, cx, ops[0], tmp, 16, span)?;
                    }
                    _ => return err(span, format!("'{name}' needs a vector register operand")),
                }
            }
            VOp::Broadcast(size) => {
                want(2, 2)?;
                let src = self.vec_ptr(f, ops[1], span)?;
                let mut at = 0;
                while at < width {
                    let p = lane_addr(f, tmp, at);
                    f.b.copy(p, src, size);
                    at += size;
                }
                self.vec_store(f, cx, ops[0], tmp, width, span)?;
            }
            VOp::Bin(lane, ty, scalar) => {
                want(2, 3)?;
                let (a, b) = self.vec_sources(f, &ops, span)?;
                if scalar {
                    f.b.copy(tmp, a, 16);
                    let x = load_lane(f, a, 0, ty);
                    let y = load_lane(f, b, 0, ty);
                    let r = lane_op(f, lane, ty, x, y);
                    store_lane(f, tmp, 0, ty, r);
                    self.vec_store(f, cx, ops[0], tmp, 16, span)?;
                } else {
                    for i in 0..width / ty.size() {
                        let x = load_lane(f, a, i, ty);
                        let y = load_lane(f, b, i, ty);
                        let r = lane_op(f, lane, ty, x, y);
                        store_lane(f, tmp, i, ty, r);
                    }
                    self.vec_store(f, cx, ops[0], tmp, width, span)?;
                }
            }
            VOp::Sqrt(ty, scalar) => {
                want(2, 3)?;
                if scalar {
                    // `sqrtss dst, src` / `sqrtss dst, a, src`: lane 0 from src, the rest from a.
                    let (a, b) = self.vec_sources(f, &ops, span)?;
                    f.b.copy(tmp, a, 16);
                    let x = load_lane(f, b, 0, ty);
                    let r = float_unary(f, Intrinsic::Sqrt, ty, x);
                    store_lane(f, tmp, 0, ty, r);
                    self.vec_store(f, cx, ops[0], tmp, 16, span)?;
                } else {
                    want(2, 2)?;
                    let src = self.vec_ptr(f, ops[1], span)?;
                    for i in 0..width / ty.size() {
                        let x = load_lane(f, src, i, ty);
                        let r = float_unary(f, Intrinsic::Sqrt, ty, x);
                        store_lane(f, tmp, i, ty, r);
                    }
                    self.vec_store(f, cx, ops[0], tmp, width, span)?;
                }
            }
            VOp::Abs(ty) => {
                want(2, 2)?;
                let src = self.vec_ptr(f, ops[1], span)?;
                for i in 0..width / ty.size() {
                    let x = load_lane(f, src, i, ty);
                    let neg = is_neg(f, ty, x);
                    let zero = konst(f, ty, 0);
                    let minus = bin(f, BinOp::Sub, ty, zero, x);
                    let r = select(f, ty, neg, minus, x);
                    store_lane(f, tmp, i, ty, r);
                }
                self.vec_store(f, cx, ops[0], tmp, width, span)?;
            }
            VOp::Shift(kind, ty) => {
                want(2, 3)?;
                // (dst, count) or (dst, src, count); the count is an immediate or a register's low quadword.
                let src = if n == 3 {
                    self.vec_ptr(f, ops[1], span)?
                } else {
                    self.vec_ptr(f, ops[0], span)?
                };
                let count = match ops[n - 1] {
                    VOpd::Imm(v) => konst(f, Ty::I64, (v as u64) & 0xff),
                    other => {
                        let p = self.vec_ptr(f, other, span)?;
                        f.b.load(Ty::I64, p)
                    }
                };
                let lane_bits = bits(ty);
                let max = konst(f, Ty::I64, lane_bits - 1);
                let over = cmp(f, CmpOp::UGt, Ty::I64, count, max);
                let clamped = select(f, Ty::I64, over, max, count);
                let amount = resize(f, clamped, Ty::I64, ty, false);
                for i in 0..width / ty.size() {
                    let x = load_lane(f, src, i, ty);
                    let r = match kind {
                        Shift::Left | Shift::Logical => {
                            let op = if kind == Shift::Left {
                                BinOp::Shl
                            } else {
                                BinOp::LShr
                            };
                            let s = bin(f, op, ty, x, amount);
                            let zero = konst(f, ty, 0);
                            select(f, ty, over, zero, s)
                        }
                        // An oversized arithmetic shift fills with the sign: shifting by bits-1 does that.
                        Shift::Arith => bin(f, BinOp::AShr, ty, x, amount),
                    };
                    store_lane(f, tmp, i, ty, r);
                }
                self.vec_store(f, cx, ops[0], tmp, width, span)?;
            }
            VOp::Pshufd => {
                want(3, 3)?;
                let src = self.vec_ptr(f, ops[1], span)?;
                let imm = self.vec_imm(ops[2], span)?;
                for i in 0..width / 4 {
                    let group = i / 4 * 4;
                    let pick = (imm >> (2 * (i % 4))) & 3;
                    let v = load_lane(f, src, group + pick, Ty::I32);
                    store_lane(f, tmp, i, Ty::I32, v);
                }
                self.vec_store(f, cx, ops[0], tmp, width, span)?;
            }
            VOp::Shufps => {
                want(3, 4)?;
                let (a, b) = if n == 4 {
                    (
                        self.vec_ptr(f, ops[1], span)?,
                        self.vec_ptr(f, ops[2], span)?,
                    )
                } else {
                    (
                        self.vec_ptr(f, ops[0], span)?,
                        self.vec_ptr(f, ops[1], span)?,
                    )
                };
                let imm = self.vec_imm(ops[n - 1], span)?;
                for i in 0..width / 4 {
                    let group = i / 4 * 4;
                    let from = if i % 4 < 2 {
                        a
                    } else {
                        b
                    };
                    let pick = (imm >> (2 * (i % 4))) & 3;
                    let v = load_lane(f, from, group + pick, Ty::I32);
                    store_lane(f, tmp, i, Ty::I32, v);
                }
                self.vec_store(f, cx, ops[0], tmp, width, span)?;
            }
            VOp::Cvt(kind) => {
                want(2, 2)?;
                let src = self.vec_ptr(f, ops[1], span)?;
                for i in 0..width / 4 {
                    let r = match kind {
                        Cvt::IntToFloat => {
                            let x = load_lane(f, src, i, Ty::I32);
                            f.b.conv(ConvOp::SToF, Ty::I32, Ty::F32, x)
                        }
                        Cvt::FloatToInt | Cvt::FloatToIntTrunc => {
                            let x = load_lane(f, src, i, Ty::F32);
                            let mode = match (kind, inst.evex.round) {
                                (_, Some(Some(m))) => m,
                                (Cvt::FloatToIntTrunc, _) => 'z',
                                _ => 'n',
                            };
                            let v = float_to_s32(f, x, mode);
                            bitcast(f, Ty::I32, Ty::F32, v)
                        }
                    };
                    store_lane(f, tmp, i, Ty::F32, r);
                }
                self.vec_store(f, cx, ops[0], tmp, width, span)?;
            }
            VOp::Movmsk(size) => {
                want(2, 2)?;
                let src = self.vec_ptr(f, ops[1], span)?;
                let ty = Ty::int(size);
                let mut mask = konst(f, Ty::I64, 0);
                for i in 0..width / size {
                    let x = load_lane(f, src, i, ty);
                    let neg = is_neg(f, ty, x);
                    let bit = resize(f, neg, Ty::I8, Ty::I64, false);
                    let at = konst(f, Ty::I64, i);
                    let bit = bin(f, BinOp::Shl, Ty::I64, bit, at);
                    mask = bin(f, BinOp::Or, Ty::I64, mask, bit);
                }
                self.vec_scalar_write(f, ops[0], Ty::I64, mask, span)?;
            }
            VOp::Gather(index_size, elem_size) => {
                want(3, 3)?;
                let VOpd::Vsib {
                    base,
                    index,
                    scale,
                } = ops[1]
                else {
                    return err(
                        span,
                        format!(
                            "'{name}' needs a vector-indexed memory operand ([base + vindex*scale])"
                        ),
                    );
                };
                let mask = self.vec_ptr(f, ops[2], span)?;
                let dst = self.vec_ptr(f, ops[0], span)?;
                let (it, et) = (Ty::int(index_size), Ty::int(elem_size));
                let lanes = width / index_size.max(elem_size);
                f.b.copy(tmp, dst, 64);
                let scale = konst(f, Ty::I64, scale);
                for i in 0..lanes {
                    // Only lanes whose mask element has its sign bit set load.
                    let m = load_lane(f, mask, i, et);
                    let on = is_neg(f, et, m);
                    let idx = load_lane(f, index, i, it);
                    let idx = resize(f, idx, it, Ty::I64, true);
                    let off = bin(f, BinOp::Mul, Ty::I64, idx, scale);
                    let addr = bin(f, BinOp::Add, Ty::I64, base, off);
                    let old = load_lane(f, tmp, i, et);
                    let take = f.b.new_block();
                    let join = f.b.new_block();
                    let slot = f.b.alloca(elem_size, elem_size);
                    f.b.store(et, slot, old);
                    f.b.branch(on, take, join);
                    f.b.switch_to(take);
                    let p = ptr_of(f, addr);
                    let v = f.b.load(et, p);
                    f.b.store(et, slot, v);
                    f.b.jump(join);
                    f.b.switch_to(join);
                    let v = f.b.load(et, slot);
                    store_lane(f, tmp, i, et, v);
                }
                self.vec_store(f, cx, ops[0], tmp, lanes * elem_size, span)?;
                // The mask is cleared once every element has been gathered.
                let zeros = f.b.alloca(64, 16);
                f.b.zero(zeros, 64);
                self.vec_store(f, cx, ops[2], zeros, 64, span)?;
            }
        }
        if let Some((mask, zeroing, dst, old)) = masked {
            let one = konst(f, Ty::I64, 1);
            let ity = Ty::int(es);
            for k in 0..width / es {
                let at = konst(f, Ty::I64, k);
                let bit = bin(f, BinOp::LShr, Ty::I64, mask, at);
                let bit = bin(f, BinOp::And, Ty::I64, bit, one);
                let zero = konst(f, Ty::I64, 0);
                let on = cmp(f, CmpOp::Ne, Ty::I64, bit, zero);
                let new = load_lane(f, dst, k, ity);
                let prev = if zeroing {
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
    fn vec_sources(&mut self, f: &mut FnCtx, ops: &[VOpd], span: Span) -> Result<(Val, Val)> {
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

    fn vec_imm(&self, o: VOpd, span: Span) -> Result<u64> {
        match o {
            VOpd::Imm(v) if (0..=255).contains(&v) => Ok(v as u64),
            _ => err(span, "expected an 8-bit immediate operand"),
        }
    }

    /// A pointer to the bytes of a register or memory operand.
    fn vec_ptr(&mut self, f: &mut FnCtx, o: VOpd, span: Span) -> Result<Val> {
        match o {
            VOpd::Reg(p) => Ok(p),
            VOpd::Mem(addr) => Ok(ptr_of(f, addr)),
            VOpd::Gpr(Opd::Reg(r)) => Ok(r.addr),
            _ => err(span, "expected a vector register or memory operand"),
        }
    }

    /// Write `size` bytes from `src` to a register or memory destination.
    fn vec_store(
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
            _ => err(span, "the destination must be a vector register or memory"),
        }
    }

    fn vec_scalar_read(&mut self, f: &mut FnCtx, o: VOpd, ty: Ty, span: Span) -> Result<Val> {
        match o {
            VOpd::Gpr(opd) => self.asm_read(f, opd, ty, span),
            VOpd::Mem(addr) => {
                let p = ptr_of(f, addr);
                Ok(f.b.load(ty, p))
            }
            VOpd::Imm(v) => Ok(konst(f, ty, v as u64)),
            _ => err(
                span,
                "expected a general-purpose register or memory operand",
            ),
        }
    }

    fn vec_scalar_write(
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
                span,
                "expected a general-purpose register or memory destination",
            ),
        }
    }

    fn vec_operand(&mut self, f: &mut FnCtx, cx: &AsmCtx, o: &AsmOperand) -> Result<VOpd> {
        match o {
            AsmOperand::Decl(d) if d.colon => {
                let (kind, addr) = self.asm_declare(f, cx, d, "vec")?;
                match kind {
                    AsmReg::Vec => Ok(VOpd::Reg(addr)),
                    _ => Ok(VOpd::Gpr(self.asm_gpr_opd(kind, addr, d.name.span)?)),
                }
            }
            AsmOperand::Decl(d) => err(
                d.name.span,
                "a register pin is only valid as a separate statement",
            ),
            AsmOperand::Mem(m) => self.vec_mem(f, cx, m),
            AsmOperand::Value(e) => {
                if let Some((kind, addr)) = self.asm_reg_named(cx.scope, e)? {
                    return match kind {
                        AsmReg::Vec => Ok(VOpd::Reg(addr)),
                        _ => Ok(VOpd::Gpr(self.asm_value(f, cx.scope, e, 0)?)),
                    };
                }
                let op = self.check_expr(f, cx.scope, e, None)?;
                match op {
                    // A Jai variable: integers act as general-purpose registers, anything
                    // else (floats, arrays, vectors) is memory at its address.
                    Operand::Place {
                        ty,
                        addr,
                    } => {
                        if self.ir_ty(ty).is_some_and(|t| !t.is_float()) {
                            Ok(VOpd::Gpr(self.asm_place(ty, addr, true, e.span)?))
                        } else {
                            Ok(VOpd::Mem(bitcast(f, Ty::Ptr, Ty::I64, addr)))
                        }
                    }
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
    fn asm_reg_named(&mut self, scope: ScopeId, e: &Expr) -> Result<Option<(AsmReg, Val)>> {
        if let E::Ident(name) = &e.kind
            && let Found::Entities(ids) = self.lookup_full(scope, *name)?
            && let Some(&id) = ids.last()
            && let Some(&kind) = self.asm_regs.get(&id)
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
                        _ => return err(e.span, "a vector index scale must be 1, 2, 4 or 8"),
                    },
                    None => 1,
                };
                if term.negate || vindex.is_some() {
                    return err(term.value.span, "a memory operand takes one vector index");
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
