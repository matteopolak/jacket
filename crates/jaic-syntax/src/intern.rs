//! Process-wide string interner. Symbols are cheap copyable handles compared by number.
//!
//! Every thread interns into the same table, so a `Sym` means the same spelling wherever it goes:
//! parse workers, the checker, and the threads a compile-time run starts.
//!
//! - Spellings live in an append-only table of fixed segments that never move. `as_str` is two
//!   loads, with no lock and no thread-local.
//! - New spellings go through one of `SHARDS` maps, picked by hash, under its mutex. The number is
//!   taken from `NEXT`, so numbers are unique but their order follows thread timing.
//! - Each thread keeps a cache of the symbols it has seen (`LOCAL`), so after warm-up an intern is
//!   one hash lookup in a thread-local map, as it was when the interner was per thread.
//!
//! **Publication.** A slot is written before its shard unlocks, and a thread gets a `Sym` only
//! from that shard (under the lock), from its own cache (filled from the shard), or from another
//! thread through something that synchronises (a lock, a join, a channel). Each of those orders
//! the slot's writes before any `as_str`, so the slot fields can be relaxed atomics.
use crate::fxhash::{FxHasher, HashMap};
use std::cell::RefCell;
use std::fmt;
use std::hash::{BuildHasher, BuildHasherDefault};
use std::ptr::null_mut;
use std::sync::atomic::{AtomicPtr, AtomicU32, Ordering};
use std::sync::{Mutex, PoisonError};

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Sym(u32);

/// Slots per segment, and the most segments: room for 16M distinct spellings.
const SEGMENT: usize = 4096;

const SEGMENTS: usize = 4096;
const SHARDS: usize = 64;

/// A thread's cache is dropped when it grows past this (it refills from the shards).
const LOCAL_LIMIT: usize = 1 << 20;

struct Slot {
    ptr: AtomicPtr<u8>,
    len: AtomicU32,
}

type Segment = [Slot; SEGMENT];
type Map = HashMap<&'static str, Sym>;

static TABLE: [AtomicPtr<Segment>; SEGMENTS] = [const { AtomicPtr::new(null_mut()) }; SEGMENTS];

static SHARD: [Mutex<Map>; SHARDS] =
    [const { Mutex::new(Map::with_hasher(BuildHasherDefault::new())) }; SHARDS];

static NEXT: AtomicU32 = AtomicU32::new(0);

thread_local! {
    static LOCAL: RefCell<HashMap<&'static str, Sym>> = RefCell::default();
}

impl Sym {
    pub fn intern(text: &str) -> Sym {
        Sym::lookup(text, || Box::leak(text.to_owned().into_boxed_str()))
    }

    /// Interns a spelling that is already leaked, without copying it again.
    pub fn intern_static(text: &'static str) -> Sym {
        Sym::lookup(text, || text)
    }

    fn lookup(text: &str, leak: impl FnOnce() -> &'static str) -> Sym {
        LOCAL.with_borrow_mut(|local| {
            if let Some(&sym) = local.get(text) {
                return sym;
            }
            let (spelling, sym) = global(text, leak);
            if local.len() >= LOCAL_LIMIT {
                local.clear();
            }
            local.insert(spelling, sym);
            sym
        })
    }

    pub fn as_str(self) -> &'static str {
        let i = self.0 as usize;
        let segment = TABLE[i / SEGMENT].load(Ordering::Acquire);
        debug_assert!(!segment.is_null());
        // SAFETY: a `Sym` exists only once its slot is written and published (module docs), and
        // segments are never freed.
        #[allow(unsafe_code)]
        unsafe {
            let slot = &(*segment)[i % SEGMENT];
            let (ptr, len) = (
                slot.ptr.load(Ordering::Relaxed),
                slot.len.load(Ordering::Relaxed),
            );
            std::str::from_utf8_unchecked(std::slice::from_raw_parts(ptr, len as usize))
        }
    }

    pub fn index(self) -> u32 {
        self.0
    }
}

/// The symbol for `text` from its shard, numbering it (and leaking the spelling) the first time.
fn global(text: &str, leak: impl FnOnce() -> &'static str) -> (&'static str, Sym) {
    let hash = BuildHasherDefault::<FxHasher>::new().hash_one(text);
    let mut shard = SHARD[(hash >> 58) as usize % SHARDS]
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    if let Some((&spelling, &sym)) = shard.get_key_value(text) {
        return (spelling, sym);
    }
    let spelling = leak();
    let mut gap = 0;
    if let Some(seed) = crate::knobs::sym_scramble() {
        gap = crate::knobs::scramble_gap(seed, hash, 4);
    }
    let index = NEXT.fetch_add(1 + gap, Ordering::Relaxed) + gap;
    let slot = slot(index as usize);
    slot.ptr
        .store(spelling.as_ptr().cast_mut(), Ordering::Relaxed);
    slot.len.store(spelling.len() as u32, Ordering::Relaxed);
    let sym = Sym(index);
    shard.insert(spelling, sym);
    (spelling, sym)
}

/// The slot for symbol number `index`, installing its segment if this is the first.
fn slot(index: usize) -> &'static Slot {
    let Some(entry) = TABLE.get(index / SEGMENT) else {
        panic!("more than {} distinct identifiers", SEGMENT * SEGMENTS);
    };
    let mut segment = entry.load(Ordering::Acquire);
    if segment.is_null() {
        let slots: Box<[Slot]> = (0..SEGMENT)
            .map(|_| Slot {
                ptr: AtomicPtr::new(null_mut()),
                len: AtomicU32::new(0),
            })
            .collect();
        let fresh =
            Box::into_raw(Box::<Segment>::try_from(slots).unwrap_or_else(|_| unreachable!()));
        segment =
            match entry.compare_exchange(null_mut(), fresh, Ordering::AcqRel, Ordering::Acquire) {
                Ok(_) => fresh,
                Err(installed) => {
                    // SAFETY: `fresh` was never published.
                    #[allow(unsafe_code)]
                    drop(unsafe { Box::from_raw(fresh) });
                    installed
                }
            };
    }
    // SAFETY: installed segments live for the rest of the process.
    #[allow(unsafe_code)]
    unsafe {
        &(*segment)[index % SEGMENT]
    }
}

impl fmt::Debug for Sym {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "`{}`", self.as_str())
    }
}

impl fmt::Display for Sym {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Convenience: `sym!("name")` interns a literal.
#[macro_export]
macro_rules! sym {
    ($s:expr) => {
        $crate::intern::Sym::intern($s)
    };
}

#[cfg(test)]
mod tests {
    use super::Sym;

    /// Threads interning overlapping and distinct spellings agree on every symbol, and every
    /// symbol reads back its spelling on every thread.
    #[test]
    fn threads_share_one_table() {
        let threads: Vec<_> = (0..8)
            .map(|t| {
                std::thread::spawn(move || {
                    (0..20_000)
                        .map(|i| {
                            let text = if i % 2 == 0 {
                                format!("shared_{i}")
                            } else {
                                format!("thread_{t}_{i}")
                            };
                            let sym = Sym::intern(&text);
                            assert_eq!(sym.as_str(), text);
                            (text, sym)
                        })
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        let results: Vec<_> = threads.into_iter().map(|t| t.join().unwrap()).collect();
        for result in &results {
            for (text, sym) in result {
                assert_eq!(Sym::intern(text), *sym);
                assert_eq!(sym.as_str(), text);
            }
        }
    }
}
