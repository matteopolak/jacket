//! Per-function lowering state and the null-check facts tracked through it.
use crate::debuginfo::FnDebug;
use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::values::{BasicValueEnum, FunctionValue, IntValue, PointerValue};
use jaic::abi::Piece;
use jaic::ir::{Func, Inst, Val, inst_vals, term_uses};

/// Per-function lowering state.
pub(super) struct FnState<'ctx> {
    pub(super) function: FunctionValue<'ctx>,
    pub(super) vals: Vec<Option<BasicValueEnum<'ctx>>>,
    pub(super) slots: Vec<PointerValue<'ctx>>,
    pub(super) blocks: Vec<Option<BasicBlock<'ctx>>>,
    /// Builder positioned in the dedicated alloca block.
    pub(super) allocas: Builder<'ctx>,
    /// A C aggregate returned in registers: its pieces and the memory the IR writes it to.
    pub(super) reg_ret: Option<(Vec<Piece>, PointerValue<'ctx>)>,
    /// Debug locations and variables of this function, with debug info on.
    pub(super) dbg: Option<FnDebug<'ctx>>,
    /// (file, line) of the last `Inst::Loc`: where a failed check reports itself.
    pub(super) loc: (u32, u32),
    /// Check pointers before loads, stores and copies (`Func::null_checks`).
    pub(super) null_checks: bool,
    /// Vals known to hold a real address: a slot, global or function, or an offset into one.
    pub(super) nonnull: Vec<bool>,
    /// Pointers already checked in this block (`NullFacts`).
    pub(super) facts: NullFacts,
    /// The `i1` a comparison produced, for the `Val` holding it widened to a byte: a branch on
    /// that byte tests the `i1` itself, which keeps the compare and the branch adjacent for
    /// instruction selection.
    pub(super) flags: Vec<Option<IntValue<'ctx>>>,
}

/// What lowering has learned about pointers within the IR block it is in, so that a null check
/// that cannot fail is not emitted. A check is remembered until the block ends: a pointer that
/// passed one is not in the first page, nor is a small constant offset from it, nor what a local
/// variable still holds after the pointer was loaded from it and checked. Variables whose address
/// is used for anything but a direct load or store (`escaped`) are never tracked, since any write
/// might change them.
#[derive(Default)]
pub(super) struct NullFacts {
    /// Number of the block being lowered; facts of earlier blocks have expired.
    pub(super) epoch: u32,
    /// Per `Val`: the epoch in which it is known to be non-null (0: unknown).
    checked: Vec<u32>,
    /// Per `Val`: 1 + the slot it is the address of (0: not a slot address).
    slot_addr: Vec<u32>,
    /// Per `Val`: 1 + the slot it was loaded from, and that slot's version at the time.
    pub(super) loaded_from: Vec<(u32, u32)>,
    /// Per `Val`: the value of an integer constant.
    pub(super) constant: Vec<Option<u64>>,
    /// Per `Val`: 1 + the pointer it is a small constant offset from (0: neither).
    pub(super) derived: Vec<u32>,
    /// Per slot: the epoch in which what it holds is known to be non-null (0: unknown).
    pub(super) slot_checked: Vec<u32>,
    /// Per slot: counts the stores to it, so a pointer loaded before a store is not mistaken
    /// for what the slot holds after it.
    pub(super) slot_version: Vec<u32>,
    /// Per slot: the address is used other than as the address of a load or store.
    escaped: Vec<bool>,
}

impl NullFacts {
    pub(super) fn new(func: &Func) -> Self {
        let mut facts = NullFacts {
            epoch: 0,
            checked: vec![0; func.vals.len()],
            slot_addr: vec![0; func.vals.len()],
            loaded_from: vec![(0, 0); func.vals.len()],
            constant: vec![None; func.vals.len()],
            derived: vec![0; func.vals.len()],
            slot_checked: vec![0; func.slots.len()],
            slot_version: vec![0; func.slots.len()],
            escaped: vec![false; func.slots.len()],
        };
        for block in &func.blocks {
            for inst in &block.insts {
                match inst {
                    Inst::SlotAddr {
                        dst,
                        slot,
                    } => facts.slot_addr[dst.0 as usize] = slot.0 + 1,
                    Inst::IConst {
                        dst,
                        value,
                        ..
                    } => facts.constant[dst.0 as usize] = Some(*value),
                    _ => {}
                }
            }
        }
        for block in &func.blocks {
            let mut escape = |v: Val| {
                if let Some(slot) = facts.slot_addr[v.0 as usize].checked_sub(1) {
                    facts.escaped[slot as usize] = true;
                }
            };
            for inst in &block.insts {
                match inst {
                    Inst::Load {
                        ..
                    } => {}
                    Inst::Store {
                        value, ..
                    } => escape(*value),
                    _ => inst_vals(inst, &mut |_| {}, &mut escape),
                }
            }
            term_uses(&block.term, &mut escape);
        }
        facts
    }

    /// Start a new block: nothing learned so far holds.
    pub(super) fn enter_block(&mut self) {
        self.epoch += 1;
    }

    pub(super) fn mark(&mut self, v: Val) {
        if let Some(c) = self.checked.get_mut(v.0 as usize) {
            *c = self.epoch;
        }
    }

    /// The slot a direct load or store of `addr` reads or writes, if it is tracked.
    pub(super) fn tracked_slot(&self, addr: Val) -> Option<usize> {
        let slot = self.slot_addr.get(addr.0 as usize)?.checked_sub(1)? as usize;
        (!self.escaped[slot]).then_some(slot)
    }
}

impl FnState<'_> {
    /// Whether `v` is known to be a real address at this point: a slot, global or function, a
    /// pointer that passed a check in this block, a small offset from one, or a pointer loaded
    /// from a variable that has not changed since another load of it passed a check.
    pub(super) fn known_nonnull(&self, v: Val) -> bool {
        let facts = &self.facts;
        let mut v = v.0 as usize;
        loop {
            if self.nonnull.get(v).copied().unwrap_or(false)
                || facts.checked.get(v) == Some(&facts.epoch)
            {
                return true;
            }
            let (slot, version) = facts.loaded_from.get(v).copied().unwrap_or((0, 0));
            if let Some(slot) = slot.checked_sub(1) {
                let slot = slot as usize;
                if facts.slot_version[slot] == version && facts.slot_checked[slot] == facts.epoch {
                    return true;
                }
            }
            match facts.derived.get(v).and_then(|d| d.checked_sub(1)) {
                Some(base) => v = base as usize,
                None => return false,
            }
        }
    }
}
