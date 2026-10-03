use super::*;
use crate::retained_metadata::{EvalRetainedMetadataError, admit, push};

impl WeakFloatKey {
    /// Inspect each genuine key allocation once, including exact decimal backing
    /// and child boxes. Cached fingerprints never stand in for owned storage.
    pub fn visit_retained_metadata<E>(
        &self,
        charge: &mut impl FnMut(usize, usize) -> Result<(), E>,
    ) -> Result<(), EvalRetainedMetadataError<E>> {
        let mut pending = Vec::new();
        let mut visited = Vec::new();
        push(&mut pending, self, charge)?;
        while let Some(key) = pending.pop() {
            admit(1, 0, charge)?;
            let pointer = std::sync::Arc::as_ptr(&key.0);
            let mut repeated = false;
            for previous in &visited {
                admit(1, 0, charge)?;
                if *previous == pointer {
                    repeated = true;
                    break;
                }
            }
            if repeated {
                continue;
            }
            push(&mut visited, pointer, charge)?;
            let children = key
                .0
                .children
                .len()
                .checked_mul(std::mem::size_of::<WeakFloatKey>())
                .ok_or(EvalRetainedMetadataError::CapacityOverflow)?;
            let bytes = (3 * std::mem::size_of::<usize>())
                .checked_add(std::mem::size_of::<KeyNode>())
                .and_then(|n| n.checked_add(children))
                .ok_or(EvalRetainedMetadataError::CapacityOverflow)?;
            admit(1, bytes, charge)?;
            if let Node::Decimal(value) = &key.0.tag {
                admit(1, value.retained_spelling_capacity(), charge)?;
            }
            for child in key.0.children.iter().rev() {
                admit(1, 0, charge)?;
                push(&mut pending, child, charge)?;
            }
        }
        Ok(())
    }
}
