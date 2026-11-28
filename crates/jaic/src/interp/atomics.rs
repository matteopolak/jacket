//! Real host atomics over validated scalar storage. The cooperative scheduler
//! does not replace atomicity: native foreign threads may touch the same memory.
use super::{Interp, Res, Rets, Trap, ir};
use std::sync::atomic::{AtomicU8, AtomicU16, AtomicU32, AtomicU64, Ordering};

fn order(value: u8) -> Res<Ordering> {
    Ok(match value {
        ir::ORDER_RELAXED => Ordering::Relaxed,
        ir::ORDER_ACQUIRE => Ordering::Acquire,
        ir::ORDER_RELEASE => Ordering::Release,
        ir::ORDER_ACQ_REL => Ordering::AcqRel,
        ir::ORDER_SEQ_CST => Ordering::SeqCst,
        _ => return Err(Trap::new("unknown atomic memory order")),
    })
}

/// The strongest ordering a failed compare-exchange may have when the successful one has
/// `success`: a failure only loads.
fn failure_of(success: Ordering) -> Ordering {
    match success {
        Ordering::Release | Ordering::Relaxed => Ordering::Relaxed,
        Ordering::AcqRel | Ordering::Acquire => Ordering::Acquire,
        _ => Ordering::SeqCst,
    }
}

impl Interp {
    pub(super) fn atomic_access(&mut self, op: ir::Intrinsic, args: &[u64]) -> Res<Rets> {
        use ir::Intrinsic as I;
        let (operands, ordering, failure) = match op {
            I::AtomicLoad(o) => (1, o, 0),
            I::AtomicStore(o) | I::AtomicRmw(_, o) => (2, o, 0),
            I::AtomicCas(s, f) => (3, s, f),
            I::CompareAndSwap => (3, ir::ORDER_SEQ_CST, ir::ORDER_SEQ_CST),
            _ => return self.trap("invalid atomic operation"),
        };

        if args.len() != operands + 1 {
            return self.trap("invalid atomic operands");
        }

        let width = args[operands];
        let address = args[0];

        if !matches!(width, 1 | 2 | 4 | 8) {
            return self.trap("invalid atomic scalar width");
        }

        if !address.is_multiple_of(width) {
            return self.trap(ir::check_message(
                ir::CheckReason::AtomicAlignment,
                address,
                width,
            ));
        }

        let success = order(ordering)?;
        let failed = order(failure)?;

        match op {
            I::AtomicLoad(2 | 3) | I::AtomicStore(1 | 3) => {
                return self.trap("invalid atomic load/store memory order");
            }
            I::AtomicCas(..)
                if !matches!((ordering, failure), (0..=4, 0) | (1 | 3 | 4, 1) | (4, 4)) =>
            {
                return self.trap("invalid compare-exchange failure memory order");
            }
            _ => {}
        }

        // A big-endian program's memory holds every scalar byte-swapped. Exchange,
        // compare-exchange and the bitwise operations act on the swapped bytes directly;
        // addition and subtraction carry across bytes, so they retry a compare-exchange.
        let swap = self.swap_bytes;

        macro_rules! access {
            ($atomic:ty, $scalar:ty) => {{
                let order = |v: $scalar| {
                    if swap {
                        v.swap_bytes()
                    } else {
                        v
                    }
                };
                // SAFETY: the width and natural alignment were checked above. Like the
                // interpreter's plain loads and stores, the access goes through the raw
                // pointer, so a bad address faults as it would natively (asking about the
                // mapping first read /proc/self/maps on Linux after every foreign call).
                // Callers must retain the allocation and use compatible atomic accesses
                // while it is shared.
                let cell = unsafe { <$atomic>::from_ptr(self.host_addr(address) as *mut $scalar) };
                match op {
                    I::AtomicLoad(_) => Rets::one(order(cell.load(success)) as u64),
                    I::AtomicStore(_) => {
                        cell.store(order(args[1] as $scalar), success);
                        Rets::default()
                    }
                    I::AtomicRmw(rmw @ (ir::AtomicOp::Add | ir::AtomicOp::Sub), _) if swap => {
                        let value = args[1] as $scalar;
                        let step = |old: $scalar| {
                            let old = old.swap_bytes();
                            let new = if rmw == ir::AtomicOp::Add {
                                old.wrapping_add(value)
                            } else {
                                old.wrapping_sub(value)
                            };
                            Some(new.swap_bytes())
                        };
                        let old = match cell.try_update(success, failure_of(success), step) {
                            Ok(old) | Err(old) => old,
                        };
                        Rets::one(old.swap_bytes() as u64)
                    }
                    I::AtomicRmw(rmw, _) => {
                        let value = order(args[1] as $scalar);
                        let old = match rmw {
                            ir::AtomicOp::Xchg => cell.swap(value, success),
                            ir::AtomicOp::Add => cell.fetch_add(value, success),
                            ir::AtomicOp::Sub => cell.fetch_sub(value, success),
                            ir::AtomicOp::And => cell.fetch_and(value, success),
                            ir::AtomicOp::Or => cell.fetch_or(value, success),
                            ir::AtomicOp::Xor => cell.fetch_xor(value, success),
                        };
                        Rets::one(order(old) as u64)
                    }
                    _ => {
                        let (ok, old) = match cell.compare_exchange(
                            order(args[1] as $scalar),
                            order(args[2] as $scalar),
                            success,
                            failed,
                        ) {
                            Ok(old) => (true, old),
                            Err(old) => (false, old),
                        };
                        Rets::collect([ok as u64, order(old) as u64].into_iter())
                    }
                }
            }};
        }

        Ok(match width {
            1 => access!(AtomicU8, u8),
            2 => access!(AtomicU16, u16),
            4 => access!(AtomicU32, u32),
            8 => access!(AtomicU64, u64),
            _ => unreachable!(),
        })
    }
}
