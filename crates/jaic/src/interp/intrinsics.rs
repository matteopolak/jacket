//! Compiler intrinsics and wide-float operations.
use super::rets::Rets;
use super::trap::null_access;
use super::{Interp, Res, host_insts, native};
use crate::ir::{self, CmpOp, Ty};

impl Interp {
    pub(super) fn intrinsic(
        &mut self,
        op: ir::Intrinsic,
        a: &[u64],
        result_type: Option<Ty>,
    ) -> Res<Rets> {
        use ir::Intrinsic as I;
        let unary_float = |value: u64, narrow: fn(f32) -> f32, wide: fn(f64) -> f64| {
            if result_type == Some(Ty::F32) {
                narrow(f32::from_bits(value as u32)).to_bits() as u64
            } else {
                wide(f64::from_bits(value)).to_bits()
            }
        };
        Ok(match op {
            I::Memcpy => {
                if a[2] > 0 {
                    if a[0] < 4096 || a[1] < 4096 {
                        return self.null_trap("memcpy through a null pointer");
                    }
                    let Ok(len) = usize::try_from(a[2]) else {
                        return self.trap("memcpy length exceeds the host address space");
                    };
                    self.check_writable(a[0], len)?;
                    // Like a load or store, a copy goes through the program's pointers directly,
                    // so a bad one faults as it would natively.
                    // SAFETY: the program's own memmove.
                    unsafe { std::ptr::copy(a[1] as *const u8, a[0] as *mut u8, len) };
                }
                Rets::default()
            }
            I::Memset => {
                if a[2] > 0 {
                    if a[0] < 4096 {
                        return self.null_trap("memset through a null pointer");
                    }
                    let Ok(len) = usize::try_from(a[2]) else {
                        return self.trap("memset length exceeds the host address space");
                    };
                    self.check_writable(a[0], len)?;
                    // SAFETY: the program's own memset, as for memcpy.
                    unsafe { std::ptr::write_bytes(a[0] as *mut u8, a[1] as u8, len) };
                }
                Rets::default()
            }
            I::Memcmp => {
                let Ok(n) = usize::try_from(a[2]) else {
                    return self.trap("memcmp length exceeds the host address space");
                };
                // An empty comparison may pass null pointers, which a slice must not hold.
                let r: i16 = if n == 0 {
                    0
                } else {
                    if a[0] < 4096 || a[1] < 4096 {
                        return self.null_trap("memcmp through a null pointer");
                    }
                    if n > isize::MAX as usize {
                        return self.trap("memcmp length exceeds the host address space");
                    }
                    // SAFETY: the program's own memcmp, as for memcpy.
                    let ordering = unsafe {
                        std::slice::from_raw_parts(a[0] as *const u8, n)
                            .cmp(std::slice::from_raw_parts(a[1] as *const u8, n))
                    };
                    match ordering {
                        std::cmp::Ordering::Less => -1,
                        std::cmp::Ordering::Equal => 0,
                        std::cmp::Ordering::Greater => 1,
                    }
                };
                Rets::one(r as u16 as u64)
            }
            I::CompareAndSwap
            | I::AtomicLoad(_)
            | I::AtomicStore(_)
            | I::AtomicRmw(..)
            | I::AtomicCas(..) => {
                let result = self.atomic_access(op, a)?;
                // Poll after the indivisible operation, so waiting cooperative
                // threads can make progress without weakening native atomicity.
                if self.multi && self.host.cooperative_threads() {
                    self.inline_poll();
                }
                result
            }
            I::DebugBreak => return self.trap("debug_break() was called"),
            I::Trap => {
                let reason = a.first().copied().and_then(ir::CheckReason::from_raw);
                return self.check_trap(reason, 0, 0);
            }
            I::BoundsCheck => {
                let (index, count) = (a[0] as i64, a[1] as i64);
                if index < 0 || index >= count {
                    return self.check_trap(ir::CheckReason::Bounds, a[0], a[1]);
                }
                Rets::default()
            }
            // Needs the program for its location: `step` handles it.
            I::CheckFailed => unreachable!("CheckFailed is handled by `step`"),
            I::FiberInit | I::FiberSwitch | I::FiberFree => {
                unreachable!("fibers are handled by `step`")
            }
            I::CompilerWrite => {
                let bytes = self.fetch(a[0], a[1] as usize)?;
                self.effects += 1;
                self.host.write(&bytes, a.get(2).is_some_and(|&v| v != 0));
                Rets::default()
            }
            I::Sqrt => Rets::one(unary_float(a[0], f32::sqrt, f64::sqrt)),
            I::Sin => Rets::one(unary_float(a[0], f32::sin, f64::sin)),
            I::Cos => Rets::one(unary_float(a[0], f32::cos, f64::cos)),
            I::Floor => Rets::one(unary_float(a[0], f32::floor, f64::floor)),
            I::Ceil => Rets::one(unary_float(a[0], f32::ceil, f64::ceil)),
            I::Round => Rets::one(unary_float(a[0], f32::round, f64::round)),
            I::Trunc => Rets::one(unary_float(a[0], f32::trunc, f64::trunc)),
            I::Fabs => Rets::one(unary_float(a[0], f32::abs, f64::abs)),
            I::Fma => Rets::one(if result_type == Some(Ty::F32) {
                f32::from_bits(a[0] as u32)
                    .mul_add(f32::from_bits(a[1] as u32), f32::from_bits(a[2] as u32))
                    .to_bits() as u64
            } else {
                f64::from_bits(a[0])
                    .mul_add(f64::from_bits(a[1]), f64::from_bits(a[2]))
                    .to_bits()
            }),
            I::ReturnAddress => Rets::one(0),
            I::CycleCounter => {
                self.effects += 1;
                Rets::one(host_insts::cycle_counter()?)
            }
            I::X86Timestamp {
                ..
            }
            | I::X86Random {
                ..
            }
            | I::X86Xgetbv
            | I::X86Fence(_)
            | I::X86Pause
            | I::X86Prefetch(_) => {
                self.effects += 1;
                let results = host_insts::execute(op, a)?;
                if op == I::X86Pause && self.multi && self.host.cooperative_threads() {
                    self.inline_poll();
                }
                results
            }
            I::CpuId => {
                self.effects += 1;
                #[cfg(target_arch = "x86_64")]
                {
                    // CPUID is mandatory on x86-64; the standard wrapper preserves ABI registers.
                    let info = std::arch::x86_64::__cpuid_count(a[0] as u32, a[1] as u32);
                    Rets::collect(
                        [info.eax, info.ebx, info.ecx, info.edx]
                            .into_iter()
                            .map(u64::from),
                    )
                }
                #[cfg(not(target_arch = "x86_64"))]
                return self.trap("CPUID execution requires an x86-64 interpreter host");
            }
            I::Syscall => {
                // The sandbox makes only system calls that map or release memory; any other
                // gets EPERM.
                let sandboxed = !self.host.native_linking();
                let ret = if sandboxed && !native::memory_only_syscall(a) {
                    None
                } else {
                    native::syscall(a)
                };
                match ret {
                    Some(ret) => Rets::one(ret as u64),
                    None if sandboxed => Rets::one(-1i64 as u64),
                    None => {
                        return self.trap(
                            "a `syscall` runs only on a Linux x86-64 host (compile-time execution here cannot make system calls)",
                        );
                    }
                }
            }
            I::BlackBox => Rets::one(a[0]),
            I::BlackBoxMemory => Rets::default(),
            I::Pause => {
                std::hint::spin_loop();
                if self.multi && self.host.cooperative_threads() {
                    self.inline_poll();
                }
                Rets::default()
            }
            I::Popcount => Rets::one(a[0].count_ones() as u64),
            I::Ctlz => {
                let bits = a[1] as u32;
                let lead = if a[0] == 0 {
                    64
                } else {
                    a[0].leading_zeros()
                };
                Rets::one((lead - (64 - bits)) as u64)
            }
            I::Cttz => Rets::one((a[0].trailing_zeros()).min(a[1] as u32) as u64),
            I::Bswap => Rets::one(a[0].swap_bytes() >> (64 - a[1] as u32)),
            I::IsCompileTime => Rets::one(self.compile_time as u64),
            I::SAddOverflow | I::SSubOverflow | I::SMulOverflow => {
                let bits = (a[2] as u32 * 8).min(64);
                let wide = |v: u64| ((v as i128) << (128 - bits)) >> (128 - bits);
                let (x, y) = (wide(a[0]), wide(a[1]));
                let r = match op {
                    I::SAddOverflow => x + y,
                    I::SSubOverflow => x - y,
                    _ => x * y,
                };
                let limit = 1i128 << (bits - 1);
                Rets::one((r < -limit || r >= limit) as u64)
            }
            I::UAddOverflow | I::USubOverflow | I::UMulOverflow => {
                let bits = (a[2] as u32 * 8).min(64);
                let keep = if bits == 64 {
                    u64::MAX
                } else {
                    (1u64 << bits) - 1
                };
                let (x, y) = ((a[0] & keep) as i128, (a[1] & keep) as i128);
                let r = match op {
                    I::UAddOverflow => x + y,
                    I::USubOverflow => x - y,
                    _ => x * y,
                };
                Rets::one((r < 0 || r > keep as i128) as u64)
            }
            I::Wide(op, fmt) => self.wide(op, fmt, a)?.into(),
            I::ByteLookup(width) => {
                let table = self.fetch(a[1], 16)?;
                let indices = self.fetch(a[2], width as usize)?;
                let out: Vec<u8> = indices
                    .iter()
                    .map(|&i| {
                        if i < 16 {
                            table[i as usize]
                        } else {
                            0
                        }
                    })
                    .collect();
                self.put(a[0], &out)?;
                Rets::default()
            }
            I::SignMask(width) => {
                let bytes = self.fetch(a[0], width as usize)?;
                let mut mask = 0u64;
                for (i, b) in bytes.iter().enumerate() {
                    mask |= ((b >> 7) as u64) << i;
                }
                Rets::one(mask)
            }
        })
    }

    /// `long double` operations in software (`crate::wide_float`), whatever the host.
    fn wide(&mut self, op: ir::WideOp, fmt: ir::WideFloat, a: &[u64]) -> Res<Vec<u64>> {
        use crate::wide_float as w;
        use ir::WideOp as W;
        let value = |s: &Self, addr: u64| -> Res<w::Bytes> {
            if addr < 4096 {
                return s.null_trap(null_access("read", addr));
            }
            let mut out = [0u8; 16];
            out.copy_from_slice(&s.fetch(addr, 16)?);
            Ok(out)
        };
        let put = |s: &mut Self, addr: u64, v: w::Bytes| -> Res<Vec<u64>> {
            if addr < 4096 {
                return s.null_trap(null_access("write", addr));
            }
            if !s.write(addr, &v) {
                return Err(super::Trap::unwritable(addr, 16).at(s.loc));
            }
            Ok(Vec::new())
        };
        match op {
            W::Arith(kind) => {
                let r = w::arith(fmt, kind, &value(self, a[1])?, &value(self, a[2])?);
                put(self, a[0], r)
            }
            W::Neg => {
                let r = w::neg(fmt, &value(self, a[1])?);
                put(self, a[0], r)
            }
            W::Cmp(cmp) => {
                use std::cmp::Ordering as O;
                let order = w::compare(fmt, &value(self, a[0])?, &value(self, a[1])?);
                let yes = match (cmp, order) {
                    (CmpOp::FNe, None) => true,
                    (_, None) => false,
                    (CmpOp::FEq, Some(o)) => o == O::Equal,
                    (CmpOp::FNe, Some(o)) => o != O::Equal,
                    (CmpOp::FLt, Some(o)) => o == O::Less,
                    (CmpOp::FLe, Some(o)) => o != O::Greater,
                    (CmpOp::FGt, Some(o)) => o == O::Greater,
                    (CmpOp::FGe, Some(o)) => o != O::Less,
                    _ => return self.trap("invalid long double comparison"),
                };
                Ok(vec![yes as u64])
            }
            W::FromF64 => put(self, a[0], w::from_f64(fmt, f64::from_bits(a[1]))),
            W::FromF32 => put(self, a[0], w::from_f32(fmt, f32::from_bits(a[1] as u32))),
            W::FromS64 => put(self, a[0], w::from_i64(fmt, a[1] as i64)),
            W::FromU64 => put(self, a[0], w::from_u64(fmt, a[1])),
            W::ToF64 => Ok(vec![w::to_f64(fmt, &value(self, a[0])?).to_bits()]),
            W::ToF32 => Ok(vec![w::to_f32(fmt, &value(self, a[0])?).to_bits() as u64]),
            W::ToS64 => Ok(vec![w::to_i64(fmt, &value(self, a[0])?) as u64]),
            W::ToU64 => Ok(vec![w::to_u64(fmt, &value(self, a[0])?)]),
        }
    }
}
