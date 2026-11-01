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
//!   one hash lookup in a thread-local table, as it was when the interner was per thread.
//! - A spelling is hashed once. The shards and the caches are the same open-addressing `Table`,
//!   which stores each hash, so neither a miss in the cache nor a hit in a shard hashes again,
//!   and growing a table moves entries without touching the spellings.
//!
//! **Publication.** A slot is written before its shard unlocks, and a thread gets a `Sym` only
//! from that shard (under the lock), from its own cache (filled from the shard), or from another
//! thread through something that synchronises (a lock, a join, a channel). Each of those orders
//! the slot's writes before any `as_str`, so the slot fields can be relaxed atomics.
use crate::fxhash::FxHasher;
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

/// A symbol's number marking a free place in a `Table` (never a real one: there are at most
/// `SEGMENT * SEGMENTS` of those).
const FREE: u32 = u32::MAX;

#[derive(Clone, Copy)]
struct Entry {
    hash: u64,
    text: &'static str,
    sym: Sym,
}

/// Spellings to symbols: open addressing with linear probing, at most half full.
struct Table {
    entries: Vec<Entry>,
    len: usize,
}

impl Table {
    const fn new() -> Table {
        Table {
            entries: Vec::new(),
            len: 0,
        }
    }

    fn find(&self, hash: u64, text: &str) -> Option<Entry> {
        let mask = self.entries.len().checked_sub(1)?;
        let mut at = self.home(hash);
        loop {
            let entry = &self.entries[at];
            if entry.sym.0 == FREE {
                return None;
            }
            if entry.hash == hash && entry.text == text {
                return Some(*entry);
            }
            at = (at + 1) & mask;
        }
    }

    /// Where probing for `hash` starts. The multiply spreads the whole hash into the top bits,
    /// which the index takes: the low bits of `FxHasher` alone cluster badly under linear probing.
    fn home(&self, hash: u64) -> usize {
        (hash.wrapping_mul(0x9e37_79b9_7f4a_7c15) >> (64 - self.entries.len().trailing_zeros()))
            as usize
    }

    /// Adds an entry whose spelling is not in the table.
    fn insert(&mut self, entry: Entry) {
        if (self.len + 1) * 2 > self.entries.len() {
            self.grow();
        }
        self.place(entry);
        self.len += 1;
    }

    fn place(&mut self, entry: Entry) {
        let mask = self.entries.len() - 1;
        let mut at = self.home(entry.hash);
        while self.entries[at].sym.0 != FREE {
            at = (at + 1) & mask;
        }
        self.entries[at] = entry;
    }

    fn grow(&mut self) {
        let free = Entry {
            hash: 0,
            text: "",
            sym: Sym(FREE),
        };
        let capacity = (self.entries.len() * 2).max(64);
        let old = std::mem::replace(&mut self.entries, vec![free; capacity]);
        for entry in old.into_iter().filter(|entry| entry.sym.0 != FREE) {
            self.place(entry);
        }
    }

    fn clear(&mut self) {
        *self = Table::new();
    }
}

static TABLE: [AtomicPtr<Segment>; SEGMENTS] = [const { AtomicPtr::new(null_mut()) }; SEGMENTS];

static SHARD: [Mutex<Table>; SHARDS] = [const { Mutex::new(Table::new()) }; SHARDS];

static NEXT: AtomicU32 = AtomicU32::new(0);

thread_local! {
    static LOCAL: RefCell<Table> = const { RefCell::new(Table::new()) };
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
        let hash = BuildHasherDefault::<FxHasher>::new().hash_one(text);
        LOCAL.with_borrow_mut(|local| {
            if let Some(entry) = local.find(hash, text) {
                return entry.sym;
            }
            let entry = global(hash, text, leak);
            if local.len >= LOCAL_LIMIT {
                local.clear();
            }
            local.insert(entry);
            entry.sym
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

/// The entry for `text` from its shard, numbering it (and leaking the spelling) the first time.
fn global(hash: u64, text: &str, leak: impl FnOnce() -> &'static str) -> Entry {
    let mut shard = SHARD[(hash >> 58) as usize % SHARDS]
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    if let Some(entry) = shard.find(hash, text) {
        return entry;
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
    let entry = Entry {
        hash,
        text: spelling,
        sym: Sym(index),
    };
    shard.insert(entry);
    entry
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
