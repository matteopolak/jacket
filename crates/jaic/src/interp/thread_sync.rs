//! Mutex and condition-variable bookkeeping shared by both thread schedulers.
//!
//! The native scheduler (`threads/`) and the inline one (`threads_inline.rs`) emulate the same
//! pthread and Win32 primitives; they differ only in how a thread that cannot go on gives up
//! its turn (blocking its OS thread, or suspending its frames). Who owns a mutex, how many
//! levels it is held, and which threads queue on a condition live here so the two cannot
//! disagree about them. Thread identities are scheduler slot indices.
use super::errno::EPERM;
use crate::fxhash::HashMap;
use std::collections::VecDeque;

#[derive(Default)]
struct Held {
    owner: Option<usize>,
    count: u32,
}

/// The mutexes a program has used, keyed by their address.
#[derive(Default)]
pub(super) struct Mutexes {
    held: HashMap<u64, Held>,
}

impl Mutexes {
    /// `pthread_mutex_init`: the mutex starts free, whatever its address held before.
    pub(super) fn init(&mut self, addr: u64) {
        self.held.insert(addr, Held::default());
    }

    pub(super) fn destroy(&mut self, addr: u64) {
        self.held.remove(&addr);
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
        if state.owner.is_none() || state.owner == Some(thread) {
            state.owner = Some(thread);
            state.count += levels;
            true
        } else {
            false
        }
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
                levels
            }
            _ => 0,
        }
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
