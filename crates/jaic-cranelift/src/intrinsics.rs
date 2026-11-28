//! Lowering compiler intrinsics and the `#intrinsic "llvm.*"` procedures the standard library
//! declares.
use cranelift_codegen::ir::{
    self, AtomicRmwOp, InstBuilder, MemFlagsData, Value, condcodes::IntCC, types,
};
use jaic::ir::{AtomicOp, CheckReason, Intrinsic, Ty, Val};

use super::R;
use super::clif_ty;
use super::data::Helper;
use super::function::Cx;

impl Cx<'_, '_> {
    /// Trap, and go on lowering in a block nothing reaches (a trap ends a Cranelift block, an IR
    /// intrinsic does not).
    pub(crate) fn trap_here(&mut self) {
        self.b.ins().trap(super::checks::TRAP);
        let rest = self.b.create_block();

        self.b.switch_to_block(rest);
    }

    /// The Cranelift integer type an atomic access of IR class `ty` goes through: atomic
    /// operations compare and exchange bits, so floats and pointers use the integer of their width.
    fn atomic_ty(&self, ty: Ty) -> R<ir::Type> {
        Ok(match ty {
            Ty::I8 => types::I8,
            Ty::I16 => types::I16,
            Ty::I32 | Ty::F32 => types::I32,
            Ty::I64 | Ty::F64 | Ty::Ptr => types::I64,
            _ => return Err("an atomic operation takes 1, 2, 4 or 8 bytes".into()),
        })
    }

    /// `v` as the integer of its width.
    fn bits(&mut self, v: Value) -> Value {
        let ty = self.b.func.dfg.value_type(v);

        if ty.is_float() {
            let int = ir::Type::int(ty.bits() as u16).expect("an integer as wide as a float");
            self.b.ins().bitcast(int, MemFlagsData::new(), v)
        } else {
            v
        }
    }

    fn constant(&self, v: Val, what: &str) -> R<u64> {
        self.consts[v.0 as usize].ok_or_else(|| format!("{what} is not a constant").into())
    }

    fn call_helper(&mut self, helper: Helper, args: &[Value]) -> Vec<Value> {
        let f = self.helper(helper);
        let call = self.b.ins().call(f, args);

        self.b.inst_results(call).to_vec()
    }

    /// `ByteLookup`: each 16-byte block of the indices looks up the one 16-byte table; an index
    /// of 16 or more gives 0, as `swizzle` does.
    fn byte_lookup(&mut self, width: u8, dst: Value, table: Value, indices: Value) {
        let flags = MemFlagsData::new();
        let table = self.b.ins().load(types::I8X16, flags, table, 0);

        for k in 0..i32::from(width) / 16 {
            let idx = self.b.ins().load(types::I8X16, flags, indices, 16 * k);
            let out = self.b.ins().swizzle(table, idx);

            self.b.ins().store(flags, out, dst, 16 * k);
        }
    }

    /// `SignMask`: bit i of the result is the top bit of byte i.
    fn sign_mask(&mut self, width: u8, src: Value) -> Value {
        let flags = MemFlagsData::new();
        let mut mask = self.b.ins().iconst(types::I64, 0);

        for k in 0..i32::from(width) / 16 {
            let v = self.b.ins().load(types::I8X16, flags, src, 16 * k);
            let bits = self.b.ins().vhigh_bits(types::I64, v);
            let bits = self.b.ins().ishl_imm_u(bits, i64::from(16 * k));

            mask = self.b.ins().bor(mask, bits);
        }

        mask
    }

    pub(crate) fn intrinsic(&mut self, op: Intrinsic, args: &[Val], want: &[Ty]) -> R<Vec<Value>> {
        let arg = |cx: &mut Self, i: usize| cx.get(args[i]);
        let ptr = |cx: &mut Self, i: usize| cx.get_i64(args[i]);
        let flags = MemFlagsData::new();
        let first_want = || want.first().copied().ok_or("an intrinsic has no result");

        Ok(match op {
            Intrinsic::Memcpy => {
                let (d, s, n) = (ptr(self, 0)?, ptr(self, 1)?, ptr(self, 2)?);
                let config = self.sh.isa.frontend_config();

                self.b.call_memmove(config, d, s, n);
                vec![]
            }
            Intrinsic::Memset => {
                let d = ptr(self, 0)?;
                let byte = arg(self, 1)?;
                let byte = self.coerce(byte, types::I8);
                let n = ptr(self, 2)?;
                let config = self.sh.isa.frontend_config();

                self.b.call_memset(config, d, byte, n);
                vec![]
            }
            Intrinsic::Memcmp => {
                let (a, b, n) = (ptr(self, 0)?, ptr(self, 1)?, ptr(self, 2)?);
                let r = self.call_helper(Helper::Memcmp, &[a, b, n])[0];
                // Normalized to -1/0/1 as I16, like the interpreter.
                let lt = self.b.ins().icmp_imm_s(IntCC::SignedLessThan, r, 0);
                let gt = self.b.ins().icmp_imm_s(IntCC::SignedGreaterThan, r, 0);
                let pos = self.b.ins().uextend(types::I16, gt);
                let lt = self.b.ins().uextend(types::I16, lt);
                let neg = self.b.ins().ineg(lt);

                vec![self.b.ins().bor(pos, neg)]
            }
            Intrinsic::CompareAndSwap => {
                let p = ptr(self, 0)?;
                let old = arg(self, 1)?;
                let float = self.b.func.dfg.value_type(old);
                let old = self.bits(old);
                let new = arg(self, 2)?;
                let new = self.bits(new);
                let new = self.coerce(new, self.b.func.dfg.value_type(old));
                let prev = self.b.ins().atomic_cas(flags, p, old, new);
                let ok = self.b.ins().icmp(IntCC::Equal, prev, old);
                let prev = self.coerce(prev, float);

                vec![ok, prev]
            }
            Intrinsic::AtomicLoad(_) | Intrinsic::VolatileLoad => {
                let ty = first_want()?;
                let int = self.atomic_ty(ty)?;
                let p = ptr(self, 0)?;
                let v = if matches!(op, Intrinsic::AtomicLoad(_)) {
                    self.b.ins().atomic_load(int, flags, p)
                } else {
                    // Unoptimized code keeps every access as written.
                    self.b.ins().load(int, flags, p, 0)
                };

                vec![self.coerce(v, clif_ty(ty)?)]
            }
            Intrinsic::AtomicStore(_) | Intrinsic::VolatileStore => {
                let p = ptr(self, 0)?;
                let v = arg(self, 1)?;
                let v = self.bits(v);

                if matches!(op, Intrinsic::AtomicStore(_)) {
                    self.b.ins().atomic_store(flags, v, p);
                } else {
                    self.b.ins().store(flags, v, p, 0);
                }

                vec![]
            }
            Intrinsic::AtomicRmw(rmw, _) => {
                let ty = first_want()?;
                let p = ptr(self, 0)?;
                let v = arg(self, 1)?;
                let v = self.bits(v);
                let v = self.coerce(v, self.atomic_ty(ty)?);
                let rmw = match rmw {
                    AtomicOp::Xchg => AtomicRmwOp::Xchg,
                    AtomicOp::Add => AtomicRmwOp::Add,
                    AtomicOp::Sub => AtomicRmwOp::Sub,
                    AtomicOp::And => AtomicRmwOp::And,
                    AtomicOp::Or => AtomicRmwOp::Or,
                    AtomicOp::Xor => AtomicRmwOp::Xor,
                };
                let int = self.b.func.dfg.value_type(v);
                let old = self.b.ins().atomic_rmw(int, flags, rmw, p, v);

                vec![self.coerce(old, clif_ty(ty)?)]
            }
            Intrinsic::AtomicCas(..) => {
                let ty = want
                    .get(1)
                    .copied()
                    .ok_or("a compare-exchange has no value")?;
                let int = self.atomic_ty(ty)?;
                let p = ptr(self, 0)?;
                let old = arg(self, 1)?;
                let old = self.bits(old);
                let old = self.coerce(old, int);
                let new = arg(self, 2)?;
                let new = self.bits(new);
                let new = self.coerce(new, int);
                let prev = self.b.ins().atomic_cas(flags, p, old, new);
                let ok = self.b.ins().icmp(IntCC::Equal, prev, old);

                vec![ok, self.coerce(prev, clif_ty(ty)?)]
            }
            Intrinsic::DebugBreak => {
                self.b.ins().debugtrap();
                vec![]
            }
            Intrinsic::Trap => {
                if let Some(&reason) = args.first() {
                    let reason = self.constant(reason, "trap reason")?;
                    let zero = self.b.ins().iconst(types::I64, 0);
                    let fatal = self.b.ins().iconst(types::I8, 1);

                    self.report_check(reason, zero, zero, fatal)?;
                }

                self.trap_here();
                vec![]
            }
            Intrinsic::CheckFailed => {
                let reason = self.constant(args[0], "check reason")?;
                let (a, b) = (self.get_i64(args[1])?, self.get_i64(args[2])?);
                let fatal = self.constant(args[3], "check fatality")?;
                let fatal_value = self.b.ins().iconst(types::I8, (fatal != 0) as i64);

                self.report_check(reason, a, b, fatal_value)?;

                if fatal != 0 {
                    self.trap_here();
                }

                vec![]
            }
            Intrinsic::BoundsCheck => {
                // Unsigned compare: a negative index is out of range too.
                let (index, count) = (self.get_i64(args[0])?, self.get_i64(args[1])?);
                let out = self
                    .b
                    .ins()
                    .icmp(IntCC::UnsignedGreaterThanOrEqual, index, count);

                self.trap_if(out, CheckReason::Bounds, index, count)?;
                vec![]
            }
            Intrinsic::CompilerWrite => {
                // (ptr, count, to_stderr): write(2) on fd 1 or 2.
                let (p, n) = (ptr(self, 0)?, ptr(self, 1)?);
                let to_err = arg(self, 2)?;
                let to_err = self.is_nonzero(to_err);
                let two = self.b.ins().iconst(types::I32, 2);
                let one = self.b.ins().iconst(types::I32, 1);
                let fd = self.b.ins().select(to_err, two, one);

                self.call_helper(Helper::Write, &[fd, p, n]);
                vec![]
            }
            Intrinsic::Sqrt => vec![self.unary(args[0], |b, x| b.ins().sqrt(x))?],
            Intrinsic::Floor => vec![self.unary(args[0], |b, x| b.ins().floor(x))?],
            Intrinsic::Ceil => vec![self.unary(args[0], |b, x| b.ins().ceil(x))?],
            Intrinsic::Trunc => vec![self.unary(args[0], |b, x| b.ins().trunc(x))?],
            Intrinsic::Fabs => vec![self.unary(args[0], |b, x| b.ins().fabs(x))?],
            Intrinsic::Sin | Intrinsic::Cos | Intrinsic::Round => {
                let x = arg(self, 0)?;
                let f32 = self.b.func.dfg.value_type(x) == types::F32;
                let helper = match (op, f32) {
                    (Intrinsic::Sin, false) => Helper::Sin,
                    (Intrinsic::Sin, true) => Helper::Sinf,
                    (Intrinsic::Cos, false) => Helper::Cos,
                    (Intrinsic::Cos, true) => Helper::Cosf,
                    (_, false) => Helper::Round,
                    (_, true) => Helper::Roundf,
                };

                self.call_helper(helper, &[x])
            }
            Intrinsic::Fma => {
                let (a, b, c) = (arg(self, 0)?, arg(self, 1)?, arg(self, 2)?);
                vec![self.b.ins().fma(a, b, c)]
            }
            Intrinsic::Popcount => vec![self.unary(args[0], |b, x| b.ins().popcnt(x))?],
            Intrinsic::Ctlz => vec![self.unary(args[0], |b, x| b.ins().clz(x))?],
            Intrinsic::Cttz => vec![self.unary(args[0], |b, x| b.ins().ctz(x))?],
            Intrinsic::Bswap => {
                let x = arg(self, 0)?;

                if self.b.func.dfg.value_type(x) == types::I8 {
                    vec![x]
                } else {
                    vec![self.b.ins().bswap(x)]
                }
            }
            Intrinsic::SAddOverflow
            | Intrinsic::UAddOverflow
            | Intrinsic::SSubOverflow
            | Intrinsic::USubOverflow
            | Intrinsic::SMulOverflow
            | Intrinsic::UMulOverflow => {
                let x = arg(self, 0)?;
                let y = arg(self, 1)?;
                let y = self.coerce(y, self.b.func.dfg.value_type(x));
                let (_, flag) = match op {
                    Intrinsic::SAddOverflow => self.b.ins().sadd_overflow(x, y),
                    Intrinsic::UAddOverflow => self.b.ins().uadd_overflow(x, y),
                    Intrinsic::SSubOverflow => self.b.ins().ssub_overflow(x, y),
                    Intrinsic::USubOverflow => self.b.ins().usub_overflow(x, y),
                    Intrinsic::SMulOverflow => self.b.ins().smul_overflow(x, y),
                    _ => self.b.ins().umul_overflow(x, y),
                };

                vec![flag]
            }
            Intrinsic::ReturnAddress => vec![self.b.ins().get_return_address(types::I64)],
            Intrinsic::CycleCounter => self.call_helper(Helper::CycleCounter, &[]),
            Intrinsic::Pause => {
                self.call_helper(Helper::Pause, &[]);
                vec![]
            }
            // Unoptimized code folds nothing, so the value is already opaque.
            Intrinsic::BlackBox => vec![arg(self, 0)?],
            Intrinsic::BlackBoxMemory => vec![],
            Intrinsic::IsCompileTime => {
                let ty = clif_ty(want.first().copied().unwrap_or(Ty::I8))?;
                vec![self.b.ins().iconst(ty, 0)]
            }
            // Native fibers hold nothing but their stacks, which the program owns.
            Intrinsic::FiberFree => vec![],
            Intrinsic::FiberInit | Intrinsic::FiberSwitch => {
                return Err(self.unsupported("fibers (`FiberInit`, `FiberSwitch`)"));
            }
            Intrinsic::Wide(..) => return Err(self.unsupported("`long double` arithmetic")),
            Intrinsic::ByteLookup(width) => {
                let (dst, table, indices) = (ptr(self, 0)?, ptr(self, 1)?, ptr(self, 2)?);

                self.byte_lookup(width, dst, table, indices);
                vec![]
            }
            Intrinsic::SignMask(width) => {
                let src = ptr(self, 0)?;
                vec![self.sign_mask(width, src)]
            }
            Intrinsic::StringCompare(..) => {
                return Err(self.unsupported(format!("the `#asm` SIMD operation {op:?}")));
            }
            Intrinsic::CpuId
            | Intrinsic::X86Xgetbv
            | Intrinsic::X86Timestamp {
                ..
            }
            | Intrinsic::X86Random {
                ..
            }
            | Intrinsic::X86Fence(_)
            | Intrinsic::X86Pause
            | Intrinsic::X86Prefetch(_)
            | Intrinsic::Syscall => {
                return Err(self.unsupported(format!("the `#asm` instruction {op:?}")));
            }
        })
    }

    fn unary(
        &mut self,
        v: Val,
        f: impl FnOnce(&mut cranelift_frontend::FunctionBuilder, Value) -> Value,
    ) -> R<Value> {
        let x = self.get(v)?;
        Ok(f(&mut self.b, x))
    }

    /// A call to `#intrinsic "llvm.<name>"`: the operation it names.
    pub(crate) fn llvm_intrinsic(&mut self, name: &str, args: &[Value]) -> R<Vec<Value>> {
        let (op, ty) = name.rsplit_once('.').unwrap_or((name, ""));
        let float = matches!(ty, "f32" | "f64");
        let x = args.first().copied();

        Ok(match (op, x) {
            ("trap", _) if ty.is_empty() => {
                self.trap_here();
                vec![]
            }
            ("debugtrap", _) => {
                self.b.ins().debugtrap();
                vec![]
            }
            ("sqrt", Some(x)) if float => vec![self.b.ins().sqrt(x)],
            ("floor", Some(x)) if float => vec![self.b.ins().floor(x)],
            ("ceil", Some(x)) if float => vec![self.b.ins().ceil(x)],
            ("trunc", Some(x)) if float => vec![self.b.ins().trunc(x)],
            ("fabs", Some(x)) if float => vec![self.b.ins().fabs(x)],
            // Rounds to nearest, ties to even, in the default rounding mode.
            ("nearbyint" | "rint" | "roundeven", Some(x)) if float => vec![self.b.ins().nearest(x)],
            ("ctpop", Some(x)) => vec![self.b.ins().popcnt(x)],
            _ => return Err(self.unsupported(format!("the intrinsic `llvm.{name}`"))),
        })
    }
}
