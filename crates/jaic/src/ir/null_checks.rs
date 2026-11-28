//! Which loads, stores and copies of a function native code must null-check
//! (`Func::null_checks`), shared by the native backends.
//!
//! A backend lowers a function's blocks in its own order and tells [`NullChecks`] what each
//! instruction does; [`NullChecks::check`] then says whether an access through an address needs
//! a check that can fail. Facts learned from a check hold until the block ends: a pointer that
//! passed one is not in the first page, nor is a small constant offset from it, nor what a local
//! variable still holds after the pointer was loaded from it and checked. Variables whose address
//! is used for anything but a direct load or store (`escaped`) are never tracked, since any write
//! might change them.
use super::{Func, Inst, Ty, Val, inst_vals, term_uses};

/// The null-check state of one function while it is lowered.
pub struct NullChecks {
    /// The function checks its accesses at all.
    enabled: bool,
    /// Per `Val`: known to hold a real address (a slot, global, function or trusted value, or an
    /// offset into one).
    nonnull: Vec<bool>,
    /// Number of the block being lowered; facts of earlier blocks have expired.
    epoch: u32,
    /// Per `Val`: the epoch in which it is known to be non-null (0: unknown).
    checked: Vec<u32>,
    /// Per `Val`: 1 + the slot it is the address of (0: not a slot address).
    slot_addr: Vec<u32>,
    /// Per `Val`: 1 + the slot it was loaded from, and that slot's version at the time.
    loaded_from: Vec<(u32, u32)>,
    /// Per `Val`: the value of an integer constant.
    constant: Vec<Option<u64>>,
    /// Per `Val`: 1 + the pointer it is a small constant offset from (0: neither).
    derived: Vec<u32>,
    /// Per slot: the epoch in which what it holds is known to be non-null (0: unknown).
    slot_checked: Vec<u32>,
    /// Per slot: counts the stores to it, so a pointer loaded before a store is not mistaken
    /// for what the slot holds after it.
    slot_version: Vec<u32>,
    /// Per slot: the address is used other than as the address of a load or store.
    escaped: Vec<bool>,
}

/// Offsets below this from a real address stay out of the never-mapped first page.
const FIRST_PAGE: u64 = 4096;

impl NullChecks {
    /// The state at the start of `func`. A function without null checks keeps no facts.
    pub fn new(func: &Func) -> Self {
        let enabled = func.null_checks;
        let vals = if enabled {
            func.vals.len()
        } else {
            0
        };
        let slots = if enabled {
            func.slots.len()
        } else {
            0
        };
        let mut facts = NullChecks {
            enabled,
            nonnull: vec![false; vals],
            epoch: 0,
            checked: vec![0; vals],
            slot_addr: vec![0; vals],
            loaded_from: vec![(0, 0); vals],
            constant: vec![None; vals],
            derived: vec![0; vals],
            slot_checked: vec![0; slots],
            slot_version: vec![0; slots],
            escaped: vec![false; slots],
        };

        if !enabled {
            return facts;
        }

        for v in &func.trusted {
            facts.nonnull[v.0 as usize] = true;
        }

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

    /// Start a new block: nothing learned in the previous one holds.
    pub fn enter_block(&mut self) {
        self.epoch += 1;
    }

    /// `v` is the address of a slot, global, function or stack allocation.
    pub fn address(&mut self, v: Val) {
        if let Some(known) = self.nonnull.get_mut(v.0 as usize) {
            *known = true;
        }
    }

    /// `dst = base + offset`.
    pub fn ptr_add(&mut self, dst: Val, base: Val, offset: Val) {
        if !self.enabled {
            return;
        }

        if self.nonnull[base.0 as usize] {
            self.nonnull[dst.0 as usize] = true;
        } else if self.constant[offset.0 as usize].is_some_and(|c| c < FIRST_PAGE) {
            // A real address plus a field offset is not in the first page either.
            self.derived[dst.0 as usize] = base.0 + 1;
        }
    }

    /// `dst` is loaded (as `ty`) from `addr`.
    pub fn load(&mut self, dst: Val, ty: Ty, addr: Val) {
        if self.enabled
            && ty == Ty::Ptr
            && let Some(slot) = self.tracked_slot(addr)
        {
            self.loaded_from[dst.0 as usize] = (slot as u32 + 1, self.slot_version[slot]);
        }
    }

    /// `value` is stored (as `ty`) to `addr`.
    pub fn store(&mut self, ty: Ty, addr: Val, value: Val) {
        if self.enabled
            && let Some(slot) = self.tracked_slot(addr)
        {
            let known = ty == Ty::Ptr && self.known_nonnull(value);

            self.slot_version[slot] += 1;
            self.slot_checked[slot] = if known {
                self.epoch
            } else {
                0
            };
        }
    }

    /// Whether an access through `addr` needs a check: null checks are on and `addr` is not known
    /// to be a real address here. Past a check `addr` is known to be one, and so is what the slot
    /// it came from holds, until the next store to that slot.
    pub fn check(&mut self, addr: Val) -> bool {
        if !self.enabled || self.known_nonnull(addr) {
            return false;
        }

        self.checked[addr.0 as usize] = self.epoch;
        let (from, version) = self.loaded_from[addr.0 as usize];

        if let Some(slot) = from.checked_sub(1)
            && self.slot_version[slot as usize] == version
        {
            self.slot_checked[slot as usize] = self.epoch;
        }

        true
    }

    /// The slot a direct load or store of `addr` reads or writes, if it is tracked.
    fn tracked_slot(&self, addr: Val) -> Option<usize> {
        let slot = self.slot_addr.get(addr.0 as usize)?.checked_sub(1)? as usize;
        (!self.escaped[slot]).then_some(slot)
    }

    /// Whether `v` is known to be a real address at this point: a slot, global or function, a
    /// pointer that passed a check in this block, a small offset from one, or a pointer loaded
    /// from a variable that has not changed since another load of it passed a check.
    fn known_nonnull(&self, v: Val) -> bool {
        let mut v = v.0 as usize;

        loop {
            if self.nonnull.get(v).copied().unwrap_or(false)
                || self.checked.get(v) == Some(&self.epoch)
            {
                return true;
            }

            let (slot, version) = self.loaded_from.get(v).copied().unwrap_or((0, 0));

            if let Some(slot) = slot.checked_sub(1) {
                let slot = slot as usize;

                if self.slot_version[slot] == version && self.slot_checked[slot] == self.epoch {
                    return true;
                }
            }

            match self.derived.get(v).and_then(|d| d.checked_sub(1)) {
                Some(base) => v = base as usize,
                None => return false,
            }
        }
    }
}
