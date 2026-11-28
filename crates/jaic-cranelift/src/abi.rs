//! IR signatures as Cranelift signatures: the C ABI for by-value aggregates and variadic calls.
//!
//! The classification is `jaic::abi`'s, shared with the LLVM backend. What LLVM's own lowering
//! does after it (an `AArch64` aggregate that does not fit in the registers left goes wholly on
//! the stack, Apple's variadic arguments always go on the stack) is done here by consuming the
//! registers with padding arguments, which is what the ABIs do to the following arguments too.
use cranelift_codegen::ir::{self, AbiParam, ArgumentPurpose, types};
use cranelift_codegen::isa::CallConv;
use jaic::abi::{self, Arch, Passing, Piece, PieceTy};
use jaic::ir::{AggLayout, Conv, Sig, Ty};

use super::{R, clif_ty, unsupported};

/// How one IR parameter crosses the call.
#[derive(Clone, Debug)]
pub(crate) enum ParamPlan {
    Scalar,
    /// A by-value C aggregate in register-sized pieces (one Cranelift parameter each); the IR
    /// passes a pointer to it.
    Pieces(AggLayout, Vec<Piece>),
    /// A by-value C aggregate copied to the argument area (`byval`, x86-64).
    ByVal,
    /// A by-value C aggregate passed as the address of a caller copy.
    Indirect(AggLayout),
    /// The IR out-pointer of a C aggregate result: the hidden result pointer, or memory the
    /// result registers are stored to.
    Dropped,
}

/// How the results come back.
#[derive(Clone, Debug)]
pub(crate) enum RetPlan {
    Scalars(Vec<Ty>),
    /// A C aggregate returned in register pieces, stored through the IR out-pointer.
    Pieces(AggLayout, Vec<Piece>),
    /// A C aggregate returned through the hidden result pointer, the first parameter.
    Sret,
}

/// One Cranelift parameter, in order.
#[derive(Clone, Copy, Debug)]
pub(crate) enum AbiArg {
    /// The hidden result pointer.
    Sret,
    /// A value that only consumes a register (zero when calling, ignored when called).
    Pad(ir::Type),
    /// IR parameter `ir`, or piece `piece` of the aggregate it points to.
    Param {
        ir: usize,
        piece: Option<usize>,
    },
}

pub(crate) struct Lowered {
    pub(crate) sig: ir::Signature,
    /// Per IR parameter (a call site's variadic arguments included).
    pub(crate) params: Vec<ParamPlan>,
    pub(crate) ret: RetPlan,
    /// What each Cranelift parameter is.
    pub(crate) args: Vec<AbiArg>,
}

/// The Cranelift type of a register piece.
fn piece_ty(ty: PieceTy) -> R<ir::Type> {
    Ok(match ty {
        PieceTy::I64 => types::I64,
        PieceTy::F32 => types::F32,
        PieceTy::F64 => types::F64,
        // Two packed floats share the low 64 bits of one SSE register: an 8-byte load of the
        // pair into it is what a `double` would be.
        PieceTy::V2F32 => types::F64,
        PieceTy::X87 | PieceTy::F128 => {
            return Err(unsupported("`long double` fields in C aggregates"));
        }
        PieceTy::Int(_) => return Err(unsupported("this target's aggregate passing")),
    })
}

/// Argument registers left while an `AArch64` parameter list is laid out (AAPCS64 NGRN, NSRN).
#[derive(Default)]
struct Aapcs {
    ints: u32,
    floats: u32,
}

const AAPCS_REGS: u32 = 8;

/// Lays out one signature's Cranelift parameters.
struct Builder<'a> {
    arch: Arch,
    apple: bool,
    params: &'a mut Vec<AbiParam>,
    args: &'a mut Vec<AbiArg>,
    aapcs: Aapcs,
}

impl Builder<'_> {
    fn push(&mut self, ty: ir::Type, arg: AbiArg) {
        if self.arch.is_aarch64() {
            if ty.is_float() {
                self.aapcs.floats += 1;
            } else {
                self.aapcs.ints += 1;
            }
        }

        self.params.push(AbiParam::new(ty));
        self.args.push(arg);
    }

    /// Use up the integer registers left (`floats`: the floating-point ones).
    fn exhaust(&mut self, floats: bool) {
        let (used, ty) = if floats {
            (self.aapcs.floats, types::F64)
        } else {
            (self.aapcs.ints, types::I64)
        };

        for _ in used..AAPCS_REGS {
            self.push(ty, AbiArg::Pad(ty));
        }
    }

    /// An aggregate in `pieces` on `AArch64`: in registers when all of them fit (a 16-byte
    /// aligned one starts at an even register), otherwise all on the stack, and the registers of
    /// its class are used up so that later arguments go on the stack too (AAPCS64 C.10-C.13).
    fn aarch64_pieces(&mut self, ir: usize, layout: &AggLayout, pieces: &[Piece]) -> R<()> {
        let floats = pieces
            .iter()
            .all(|p| matches!(p.ty, PieceTy::F32 | PieceTy::F64));
        let needed = pieces.len() as u32;
        let used = if floats {
            self.aapcs.floats
        } else {
            self.aapcs.ints
        };
        let mut start = used;

        if !floats && layout.align >= 16 && start % 2 == 1 {
            start += 1;
        }

        if start + needed > AAPCS_REGS {
            self.exhaust(floats);

            // On the stack the aggregate is its memory image in 8-byte units; Apple packs
            // `float`s as 4 bytes, as the image has them, but elsewhere each would take 8.
            if !self.apple
                && pieces.iter().any(|p| p.ty == PieceTy::F32)
                && self.aapcs.ints < AAPCS_REGS
            {
                return Err(unsupported(
                    "a C aggregate of `float32`s passed on the stack while integer registers are free",
                ));
            }
        } else if start > used {
            self.push(types::I64, AbiArg::Pad(types::I64));
        }

        for (k, p) in pieces.iter().enumerate() {
            self.push(
                piece_ty(p.ty)?,
                AbiArg::Param {
                    ir,
                    piece: Some(k),
                },
            );
        }

        Ok(())
    }
}

/// The Cranelift signature of an IR signature for `call_conv` (the platform's C convention, which
/// Jai-convention procedures use too: both sides of those calls are this backend's).
pub(crate) fn lower_sig(
    sig: &Sig,
    arch: Arch,
    apple: bool,
    call_conv: CallConv,
    data: &jaic::layout::DataLayout,
) -> R<Lowered> {
    let cabi = if sig.conv == Conv::C {
        sig.c_abi.as_deref()
    } else {
        None
    };

    if cabi.is_some_and(|c| c.ret_after_this) {
        return Err(unsupported("Microsoft C++ member function results"));
    }

    let ret_agg = cabi.and_then(|c| c.ret.clone());
    let forced_sret = cabi.is_some_and(|c| c.ret_indirect);
    let mut out = ir::Signature::new(call_conv);
    let mut args = Vec::new();
    let ret = match &ret_agg {
        Some(layout) => {
            if let Some(pieces) = abi::classify_ret(arch, layout, data).filter(|_| !forced_sret) {
                RetPlan::Pieces(layout.clone(), pieces)
            } else {
                out.params
                    .push(AbiParam::special(types::I64, ArgumentPurpose::StructReturn));
                args.push(AbiArg::Sret);
                RetPlan::Sret
            }
        }
        None => RetPlan::Scalars(sig.returns.clone()),
    };

    match &ret {
        RetPlan::Scalars(tys) => {
            for &ty in tys {
                out.returns.push(AbiParam::new(clif_ty(ty)?));
            }
        }
        RetPlan::Pieces(_, pieces) => {
            for p in pieces {
                out.returns.push(AbiParam::new(piece_ty(p.ty)?));
            }
        }
        RetPlan::Sret => {}
    }

    let mut params = Vec::new();
    let mut budget = abi::RegBudget::new(
        arch,
        matches!(ret, RetPlan::Sret) && abi::sret_takes_int_register(arch, forced_sret),
    );
    let mut b = Builder {
        arch,
        apple,
        params: &mut out.params,
        args: &mut args,
        aapcs: Aapcs::default(),
    };

    for (i, &ty) in sig.params.iter().enumerate() {
        let variadic = sig.c_varargs && i >= sig.c_fixed as usize;

        if variadic && i == sig.c_fixed as usize && apple && arch.is_aarch64() {
            // Apple arm64 passes every variadic argument on the stack, each in 8 bytes.
            b.exhaust(false);
            b.exhaust(true);
        }

        if ret_agg.is_some() && i == sig.out_index() {
            params.push(ParamPlan::Dropped);
            continue;
        }

        let layout = cabi.and_then(|c| c.params.get(i).cloned().flatten());
        let Some(layout) = layout else {
            let mut cty = clif_ty(ty)?;

            if variadic && apple && arch.is_aarch64() {
                cty = if cty.is_float() {
                    types::F64
                } else {
                    types::I64
                };
            }

            budget.scalar(ty);
            b.push(
                cty,
                AbiArg::Param {
                    ir: i,
                    piece: None,
                },
            );
            params.push(ParamPlan::Scalar);

            continue;
        };
        let passing = if sig.c_varargs {
            abi::classify_vararg(arch, &layout, data)
        } else {
            abi::classify_arg(arch, &layout, data)
        };

        match budget.aggregate(passing) {
            Passing::Registers(_) if variadic && apple && arch.is_aarch64() => {
                // On the stack: the aggregate's memory image in 8-byte units.
                let pieces: Vec<Piece> = (0..layout.size.div_ceil(8))
                    .map(|k| Piece {
                        offset: k * 8,
                        ty: PieceTy::I64,
                    })
                    .collect();

                for k in 0..pieces.len() {
                    b.push(
                        types::I64,
                        AbiArg::Param {
                            ir: i,
                            piece: Some(k),
                        },
                    );
                }

                params.push(ParamPlan::Pieces(layout, pieces));
            }
            Passing::Registers(pieces) => {
                if arch.is_aarch64() {
                    b.aarch64_pieces(i, &layout, &pieces)?;
                } else {
                    for (k, p) in pieces.iter().enumerate() {
                        b.push(
                            piece_ty(p.ty)?,
                            AbiArg::Param {
                                ir: i,
                                piece: Some(k),
                            },
                        );
                    }
                }

                params.push(ParamPlan::Pieces(layout, pieces));
            }
            Passing::ByVal => {
                if layout.align > 8 {
                    return Err(unsupported(
                        "a C aggregate aligned to more than 8 bytes passed by value on the stack",
                    ));
                }

                let size = layout.size.next_multiple_of(8);
                let size = u32::try_from(size)
                    .map_err(|_| unsupported("a C aggregate this large passed by value"))?;

                b.params.push(AbiParam::special(
                    types::I64,
                    ArgumentPurpose::StructArgument(size),
                ));
                b.args.push(AbiArg::Param {
                    ir: i,
                    piece: None,
                });
                params.push(ParamPlan::ByVal);
            }
            Passing::Indirect => {
                b.push(
                    types::I64,
                    AbiArg::Param {
                        ir: i,
                        piece: None,
                    },
                );
                params.push(ParamPlan::Indirect(layout));
            }
        }
    }

    Ok(Lowered {
        sig: out,
        params,
        ret,
        args,
    })
}
