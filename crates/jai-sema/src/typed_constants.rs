//! Semantic handles keep recursive constants out of lexical binding copies.
use jai_ir::ConstantValue;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct ConstantId {
    owner: u64,
    index: usize,
}

pub(crate) struct ConstantPool {
    owner: u64,
    values: Vec<ConstantValue>,
}

impl Default for ConstantPool {
    fn default() -> Self {
        static NEXT_OWNER: AtomicU64 = AtomicU64::new(1);
        Self {
            owner: NEXT_OWNER.fetch_add(1, Ordering::Relaxed),
            values: Vec::new(),
        }
    }
}

impl ConstantPool {
    pub(crate) fn insert(&mut self, value: ConstantValue) -> ConstantId {
        let id = ConstantId {
            owner: self.owner,
            index: self.values.len(),
        };
        self.values.push(value);
        id
    }
    pub(crate) fn get(&self, id: ConstantId) -> Option<&ConstantValue> {
        if id.owner != self.owner {
            return None;
        }
        self.values.get(id.index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn another_semantic_context_cannot_hydrate_an_existing_handle() {
        let types = jai_types::TypeRegistry::new();
        let ty = types.scalar(jai_types::ScalarType::Bool);
        let mut first = ConstantPool::default();
        let mut second = ConstantPool::default();
        let id = first.insert(ConstantValue {
            ty,
            kind: jai_ir::ConstantKind::Bool(true),
        });
        second.insert(ConstantValue {
            ty,
            kind: jai_ir::ConstantKind::Bool(false),
        });
        assert!(first.get(id).is_some());
        assert!(second.get(id).is_none());
    }
}
