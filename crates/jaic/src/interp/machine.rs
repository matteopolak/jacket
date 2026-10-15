//! Architectural host execution, separated from portable scheduler hints and compiler API hooks.
use super::{Res, Rets, Trap};
use crate::ir::Intrinsic;
#[cfg(target_arch = "x86_64")]
use crate::ir::{Ty, X86Fence, X86Prefetch};

pub(super) fn execute(op: Intrinsic, arguments: &[u64]) -> Res<Rets> {
    #[cfg(not(target_arch = "x86_64"))]
    {
        let _ = (op, arguments);
        Err(Trap::new(
            "x86 architectural instruction requires an x86-64 interpreter host",
        ))
    }
    #[cfg(target_arch = "x86_64")]
    {
        use std::arch::{asm, x86_64::__cpuid_count};
        let maximum = __cpuid_count(0, 0).eax;
        let leaf1 = (maximum >= 1).then(|| __cpuid_count(1, 0));
        match op {
            Intrinsic::X86Timestamp {
                aux,
            } => {
                let available = if aux {
                    __cpuid_count(0x80000000, 0).eax >= 0x80000001
                        && __cpuid_count(0x80000001, 0).edx & (1 << 27) != 0
                } else {
                    leaf1.is_some_and(|leaf| leaf.edx & (1 << 4) != 0)
                };
                if !available {
                    return Err(Trap::new(if aux {
                        "RDTSCP is unavailable on the executing x86-64 processor (CPUID)"
                    } else {
                        "RDTSC is unavailable on the executing x86-64 processor (CPUID)"
                    }));
                }
                let hi: u32;
                let lo: u32;
                let auxiliary: Option<u32>;
                // SAFETY: CPUID establishes instruction support. No process memory is read.
                unsafe {
                    if aux {
                        let value: u32;
                        asm!("rdtscp", out("eax") lo, out("edx") hi, out("ecx") value, options(nostack, preserves_flags));
                        auxiliary = Some(value);
                    } else {
                        asm!("rdtsc", out("eax") lo, out("edx") hi, options(nostack, preserves_flags));
                        auxiliary = None;
                    }
                }
                let timestamp = (u64::from(hi) << 32) | u64::from(lo);
                Ok(if let Some(value) = auxiliary {
                    Rets::collect([timestamp, u64::from(value)].into_iter())
                } else {
                    Rets::one(timestamp)
                })
            }
            Intrinsic::X86Random {
                seed,
                width,
            } => {
                let available = if seed {
                    maximum >= 7 && __cpuid_count(7, 0).ebx & (1 << 18) != 0
                } else {
                    leaf1.is_some_and(|leaf| leaf.ecx & (1 << 30) != 0)
                };
                if !available {
                    return Err(Trap::new(if seed {
                        "RDSEED is unavailable on the executing x86-64 processor (CPUID)"
                    } else {
                        "RDRAND is unavailable on the executing x86-64 processor (CPUID)"
                    }));
                }
                let carry: u8;
                let value: u64;
                // SAFETY: real CPUID guards the fixed instruction. Each attempt preserves
                // the hardware destination and CF, including resource-exhaustion failure.
                unsafe {
                    macro_rules! attempt {
                        ($instruction:literal, $modifier:literal, $ty:ty) => {{
                            let result: $ty;
                            asm!(concat!($instruction, " {result:", $modifier, "}"), "setc {carry}", result = out(reg) result, carry = out(reg_byte) carry, options(nomem, nostack));
                            result as u64
                        }};
                    }
                    value = match (seed, width) {
                        (false, Ty::I16) => attempt!("rdrand", "x", u16),
                        (false, Ty::I32) => attempt!("rdrand", "e", u32),
                        (false, Ty::I64) => attempt!("rdrand", "r", u64),
                        (true, Ty::I16) => attempt!("rdseed", "x", u16),
                        (true, Ty::I32) => attempt!("rdseed", "e", u32),
                        (true, Ty::I64) => attempt!("rdseed", "r", u64),
                        _ => return Err(Trap::new("invalid x86 random instruction width")),
                    };
                }
                Ok(Rets::collect([value, u64::from(carry)].into_iter()))
            }
            Intrinsic::X86Fence(kind) => {
                // SAFETY: SSE2 is required by the x86-64 ABI. Memory clobbers deliberately
                // prevent compiler motion across the actual architectural fence.
                unsafe {
                    match kind {
                        X86Fence::Load => asm!("lfence", options(nostack, preserves_flags)),
                        X86Fence::Store => asm!("sfence", options(nostack, preserves_flags)),
                        X86Fence::Full => asm!("mfence", options(nostack, preserves_flags)),
                    }
                }
                Ok(Rets::default())
            }
            Intrinsic::X86Pause => {
                // SAFETY: PAUSE is the x86-64 spin hint and preserves memory and flags.
                unsafe {
                    asm!("pause", options(nomem, nostack, preserves_flags));
                }
                Ok(Rets::default())
            }
            Intrinsic::X86Prefetch(hint) => {
                let [address] = arguments else {
                    return Err(Trap::new("invalid x86 prefetch operands"));
                };
                // SAFETY: x64 baseline cache hints preserve flags and do not perform a data
                // load. The operand is the interpreted program's own computed address.
                unsafe {
                    match hint {
                        X86Prefetch::T0 => {
                            asm!("prefetcht0 [{address}]",address=in(reg) *address,options(nostack, preserves_flags))
                        }
                        X86Prefetch::T1 => {
                            asm!("prefetcht1 [{address}]",address=in(reg) *address,options(nostack, preserves_flags))
                        }
                        X86Prefetch::T2 => {
                            asm!("prefetcht2 [{address}]",address=in(reg) *address,options(nostack, preserves_flags))
                        }
                        X86Prefetch::NonTemporal => {
                            asm!("prefetchnta [{address}]",address=in(reg) *address,options(nostack, preserves_flags))
                        }
                    }
                }
                Ok(Rets::default())
            }
            _ => Err(Trap::new("invalid x86 architectural operation")),
        }
    }
}

pub(super) fn cycle_counter() -> Res<u64> {
    #[cfg(target_arch = "x86_64")]
    {
        Ok(execute(
            Intrinsic::X86Timestamp {
                aux: false,
            },
            &[],
        )?[0])
    }
    #[cfg(target_arch = "aarch64")]
    {
        let counter: u64;
        // SAFETY: same user-space architectural virtual counter as native LLVM lowering.
        unsafe {
            std::arch::asm!("mrs {counter}, cntvct_el0", counter=out(reg) counter, options(nomem, nostack, preserves_flags));
        }
        Ok(counter)
    }
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    {
        Err(Trap::new(
            "no architectural cycle counter is available on this interpreter host",
        ))
    }
}
