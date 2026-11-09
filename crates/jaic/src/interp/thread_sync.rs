//! Slot, mutex and condition-variable bookkeeping shared by both thread schedulers.
//!
//! The native scheduler (`threads/`) and the inline one (`threads_inline.rs`) emulate the same
//! pthread and Win32 primitives; they differ only in how a thread that cannot go on gives up
//! its turn (blocking its OS thread, or suspending its frames). Which slot a thread lives in
//! and what its handle is (`Slots`), who owns a mutex, how many levels it is held, and which
//! threads queue on a condition live here so the two cannot disagree about them. Thread
//! identities are scheduler slot indices.
use super::errno::EPERM;
use crate::fxhash::{HashMap, HashSet};
use std::collections::VecDeque;

/// The thread slots, and the handles Jai code holds to them (`pthread_t`).
///
/// A slot is free for the next thread once the thread has finished and its handle is gone
/// (joined or detached). A handle is the slot plus one with the slot's generation in the high
/// half, so one kept after its slot was reused names nothing instead of the new thread; the
/// first owner of a slot has just the slot plus one. The schedulers keep the thread records
/// themselves, indexed by slot.
#[derive(Default)]
pub(super) struct Slots {
    /// Per slot: its generation, and whether the handle to its thread is gone.
    owners: Vec<(u32, bool)>,
    spare: Vec<usize>,
}

impl Slots {
    /// A slot for a new thread: a free one, or else a new one after the last. The caller puts
    /// the thread's record there: pushed when the slot is new (one past the last record).
    pub(super) fn acquire(&mut self) -> usize {
        if let Some(slot) = self.spare.pop() {
            self.owners[slot].1 = false;
            slot
        } else {
            self.owners.push((0, false));
            self.owners.len() - 1
        }
    }

    /// The slots free for `acquire`.
    #[cfg(test)]
    pub(super) fn spare(&self) -> &[usize] {
        &self.spare
    }

    /// The number Jai code sees for the thread in `slot`.
    pub(super) fn handle_of(&self, slot: usize) -> u64 {
        (u64::from(self.owners[slot].0) << 32) | (slot as u64 + 1)
    }

    /// The slot a handle names, if its thread (or its result) is still there.
    pub(super) fn resolve(&self, handle: u64) -> Option<usize> {
        let slot = usize::try_from((handle & 0xffff_ffff).checked_sub(1)?).ok()?;
        let &(generation, _) = self.owners.get(slot)?;
        (u64::from(generation) == handle >> 32).then_some(slot)
    }

    /// Whether nothing can ask for the slot's thread any more.
    pub(super) fn is_released(&self, slot: usize) -> bool {
        self.owners[slot].1
    }

    /// The handle to the thread in `slot` is gone. `true` when the thread has `finished` and
    /// this is the first release: the caller then ends the slot's use (`free`).
    pub(super) fn release(&mut self, slot: usize, finished: bool) -> bool {
        !std::mem::replace(&mut self.owners[slot].1, true) && finished
    }

    /// End every handle to the finished thread in `slot` and make the slot free for `acquire`.
    pub(super) fn free(&mut self, slot: usize) {
        debug_assert!(!self.spare.contains(&slot));
        let owner = &mut self.owners[slot];
        owner.0 = owner.0.wrapping_add(1);
        owner.1 = true;
        self.spare.push(slot);
    }
}

/// Owner of the mutexes of a thread that is gone: no thread, whatever slot it is given.
const NOBODY: usize = usize::MAX;

#[derive(Default)]
struct Held {
    owner: Option<usize>,
    count: u32,
}

/// The mutexes a program has used, keyed by their address.
#[derive(Default)]
pub(super) struct Mutexes {
    held: HashMap<u64, Held>,
    /// The mutexes each thread holds, so a thread's end costs what it held, not every mutex.
    owned: HashMap<usize, HashSet<u64>>,
}

/// `addr` no longer belongs to `owner`.
fn disown(owned: &mut HashMap<usize, HashSet<u64>>, owner: Option<usize>, addr: u64) {
    let Some(owner) = owner else {
        return;
    };
    if let Some(addrs) = owned.get_mut(&owner) {
        addrs.remove(&addr);
        if addrs.is_empty() {
            owned.remove(&owner);
        }
    }
}

impl Mutexes {
    /// `pthread_mutex_init`: the mutex starts free, whatever its address held before.
    pub(super) fn init(&mut self, addr: u64) {
        if let Some(old) = self.held.insert(addr, Held::default()) {
            disown(&mut self.owned, old.owner, addr);
        }
    }

    pub(super) fn destroy(&mut self, addr: u64) {
        if let Some(old) = self.held.remove(&addr) {
            disown(&mut self.owned, old.owner, addr);
        }
    }

    /// `thread` is gone: what it still holds stays held, but by nobody a later thread in its
    /// slot could be. Costs what the thread held.
    pub(super) fn orphan(&mut self, thread: usize) {
        let Some(addrs) = self.owned.remove(&thread) else {
            return;
        };
        for addr in addrs {
            if let Some(state) = self.held.get_mut(&addr) {
                state.owner = Some(NOBODY);
            }
        }
    }

    /// Whether `thread` could take the mutex now (it is free, or already its own).
    pub(super) fn available_to(&self, addr: u64, thread: usize) -> bool {
        self.held
            .get(&addr)
            .is_none_or(|m| m.owner.is_none() || m.owner == Some(thread))
    }

    /// Take `levels` levels of the mutex for `thread` if it is free or already its own.
    pub(super) fn try_lock(&mut self, addr: u64, thread: usize, levels: u32) -> bool {
        let state = self.held.entry(addr).or_default();
        if state.owner.is_none() {
            self.owned.entry(thread).or_default().insert(addr);
        } else if state.owner != Some(thread) {
            return false;
        }
        state.owner = Some(thread);
        state.count += levels;
        true
    }

    /// Give back one level. `Ok(true)` when the mutex became free (waiters can take it),
    /// `Err(EPERM)` when `thread` does not hold it.
    pub(super) fn unlock(&mut self, addr: u64, thread: usize) -> Result<bool, u64> {
        let Some(state) = self.held.get_mut(&addr) else {
            return Err(EPERM);
        };
        if state.owner != Some(thread) {
            return Err(EPERM);
        }
        state.count -= 1;
        let freed = state.count == 0;
        if freed {
            state.owner = None;
            disown(&mut self.owned, Some(thread), addr);
        }
        Ok(freed)
    }

    /// Give back every level `thread` holds (a condition wait does, and takes them back
    /// afterwards). Returns how many levels it held.
    pub(super) fn release_all(&mut self, addr: u64, thread: usize) -> u32 {
        match self.held.get_mut(&addr) {
            Some(state) if state.owner == Some(thread) => {
                let levels = state.count;
                state.count = 0;
                state.owner = None;
                disown(&mut self.owned, Some(thread), addr);
                levels
            }
            _ => 0,
        }
    }

    /// The threads that hold at least one mutex.
    #[cfg(test)]
    pub(super) fn owners(&self) -> usize {
        self.owned.len()
    }
}

/// The threads queued on each condition variable, oldest first. `W` identifies a waiter: the
/// native scheduler queues thread slots, the inline one wait tokens.
pub(super) struct CondQueues<W> {
    queues: HashMap<u64, VecDeque<W>>,
}

impl<W> Default for CondQueues<W> {
    fn default() -> Self {
        CondQueues {
            queues: HashMap::default(),
        }
    }
}

impl<W: Copy + PartialEq> CondQueues<W> {
    /// `pthread_cond_init` / `_destroy`: forget everything queued on the condition.
    pub(super) fn clear(&mut self, cond: u64) {
        self.queues.remove(&cond);
    }

    pub(super) fn enqueue(&mut self, cond: u64, waiter: W) {
        self.queues.entry(cond).or_default().push_back(waiter);
    }

    /// Take the oldest waiter, or every waiter, off the queue and hand each to `wake`.
    pub(super) fn dequeue(&mut self, cond: u64, all: bool, mut wake: impl FnMut(W)) {
        let Some(queue) = self.queues.get_mut(&cond) else {
            return;
        };
        while let Some(waiter) = queue.pop_front() {
            wake(waiter);
            if !all {
                break;
            }
        }
    }

    /// A waiter that stopped waiting without being signaled (a timeout).
    pub(super) fn remove(&mut self, cond: u64, waiter: W) {
        if let Some(queue) = self.queues.get_mut(&cond) {
            queue.retain(|&w| w != waiter);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_slot_handle_dies_when_the_slot_is_freed_and_the_next_owner_gets_a_new_one() {
        let mut slots = Slots::default();
        let first = slots.acquire();
        let old = slots.handle_of(first);
        assert_eq!(old, 1, "the first owner has the plain slot plus one");
        assert!(!slots.release(first, false), "still running");
        assert!(!slots.release(first, true), "released once");
        slots.free(first);
        assert_eq!(slots.resolve(old), None);
        let second = slots.acquire();
        assert_eq!(second, first);
        assert_ne!(slots.handle_of(second), old);
        assert_eq!(slots.resolve(slots.handle_of(second)), Some(second));
        assert!(slots.spare().is_empty());
        assert_eq!(slots.resolve(0), None);
    }

    #[test]
    fn a_mutex_stays_held_by_nobody_after_its_owner_is_orphaned() {
        let mut mutexes = Mutexes::default();
        assert!(mutexes.try_lock(0x10, 3, 2));
        assert!(mutexes.try_lock(0x20, 3, 1));
        assert!(mutexes.try_lock(0x30, 4, 1));
        mutexes.orphan(3);
        assert!(
            !mutexes.try_lock(0x10, 3, 1),
            "a thread in the slot is not the owner"
        );
        assert!(mutexes.unlock(0x20, 3).is_err());
        assert!(!mutexes.available_to(0x20, 5));
        assert_eq!(
            mutexes.unlock(0x30, 4),
            Ok(true),
            "other threads are untouched"
        );
    }

    #[test]
    fn ownership_is_tracked_only_while_a_mutex_is_held() {
        let mut mutexes = Mutexes::default();
        assert!(mutexes.try_lock(1, 7, 1));
        assert!(mutexes.try_lock(1, 7, 1));
        assert_eq!(mutexes.owners(), 1);
        assert_eq!(mutexes.unlock(1, 7), Ok(false));
        assert_eq!(mutexes.owners(), 1);
        assert_eq!(mutexes.unlock(1, 7), Ok(true));
        assert_eq!(mutexes.owners(), 0);
        assert!(mutexes.try_lock(2, 7, 3));
        assert_eq!(mutexes.release_all(2, 7), 3);
        assert_eq!(mutexes.owners(), 0);
        assert!(mutexes.try_lock(3, 7, 1));
        mutexes.destroy(3);
        assert_eq!(mutexes.owners(), 0);
        assert!(mutexes.try_lock(4, 7, 1));
        mutexes.init(4);
        assert_eq!(mutexes.owners(), 0);
    }

    #[test]
    fn many_mutexes_and_threads_coming_and_going_keep_each_lock_with_its_owner() {
        let mut mutexes = Mutexes::default();
        for addr in 0..10_000 {
            mutexes.init(addr);
        }
        // Thread 1 holds a few of them for good.
        for addr in [5, 500, 9_999] {
            assert!(mutexes.try_lock(addr, 1, 1));
        }
        // Many threads take the slot 2 in turn, hold one mutex and end; they hold what they
        // took, a few of them unlocked first.
        for round in 0..2_000u64 {
            let addr = 1_000 + round;
            assert!(mutexes.try_lock(addr, 2, 1));
            if round % 3 == 0 {
                assert_eq!(mutexes.unlock(addr, 2), Ok(true));
            }
            mutexes.orphan(2);
        }
        assert_eq!(
            mutexes.owners(),
            1,
            "only the live thread still owns anything"
        );
        for addr in [5, 500, 9_999] {
            assert_eq!(mutexes.unlock(addr, 1), Ok(true));
        }
        // A lock a gone thread kept is held for good; one it gave back is free.
        assert!(!mutexes.try_lock(1_001, 2, 1));
        assert!(mutexes.try_lock(1_000, 2, 1));
    }
}
