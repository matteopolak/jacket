//! Opaque identities for immutable compiler code values, independent of host pointers.
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT_CODE_ARENA: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CodeValueId {
    arena: u64,
    index: usize,
}
impl CodeValueId {
    pub fn index(self) -> usize {
        self.index
    }
}

pub struct CodeValueIds {
    arena: u64,
    next: usize,
}
impl Default for CodeValueIds {
    fn default() -> Self {
        Self::new()
    }
}
impl CodeValueIds {
    pub fn new() -> Self {
        let arena = NEXT_CODE_ARENA
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |arena| {
                arena.checked_add(1)
            })
            .expect("code value identity space exhausted");
        Self {
            arena,
            next: 0,
        }
    }
    pub fn allocate(&mut self) -> CodeValueId {
        let id = CodeValueId {
            arena: self.arena,
            index: self.next,
        };
        self.next = self
            .next
            .checked_add(1)
            .expect("code value identity space exhausted");
        id
    }
    pub fn owns(&self, id: CodeValueId) -> bool {
        id.arena == self.arena && id.index < self.next
    }
}
