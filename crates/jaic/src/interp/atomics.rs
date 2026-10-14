//! Real host atomics over validated scalar storage. The cooperative scheduler
//! does not replace atomicity: native foreign threads may touch the same memory.
use super::*;
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
            return self.trap(ir::check_message(ir::TRAP_ATOMIC_ALIGNMENT, address, width));
        }
        if !self.probe.accessible(address, width as usize, false)
            || (!matches!(op, I::AtomicLoad(_))
                && !self.probe.accessible(address, width as usize, true))
        {
            return self.trap(format!(
                "atomic access to inaccessible memory at {address:#x}"
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
        macro_rules! access {
            ($atomic:ty, $scalar:ty) => {{
                // SAFETY: the width, natural alignment and mapped permissions were
                // checked above. As for native pointers, callers must retain the
                // allocation and use compatible atomic accesses while it is shared.
                let cell = unsafe { <$atomic>::from_ptr(address as *mut $scalar) };
                match op {
                    I::AtomicLoad(_) => Rets::one(cell.load(success) as u64),
                    I::AtomicStore(_) => {
                        cell.store(args[1] as $scalar, success);
                        Rets::default()
                    }
                    I::AtomicRmw(rmw, _) => {
                        let value = args[1] as $scalar;
                        let old = match rmw {
                            ir::AtomicOp::Xchg => cell.swap(value, success),
                            ir::AtomicOp::Add => cell.fetch_add(value, success),
                            ir::AtomicOp::Sub => cell.fetch_sub(value, success),
                            ir::AtomicOp::And => cell.fetch_and(value, success),
                            ir::AtomicOp::Or => cell.fetch_or(value, success),
                            ir::AtomicOp::Xor => cell.fetch_xor(value, success),
                        };
                        Rets::one(old as u64)
                    }
                    _ => {
                        let (ok, old) = match cell.compare_exchange(
                            args[1] as $scalar,
                            args[2] as $scalar,
                            success,
                            failed,
                        ) {
                            Ok(old) => (true, old),
                            Err(old) => (false, old),
                        };
                        Rets::collect([ok as u64, old as u64].into_iter())
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
