//! Borrowed ownership traversal; the caller supplies its current root budget.
use crate::Value;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EvalRetainedMetadataError<E> {
    Admission(E),
    CapacityOverflow,
    Allocation,
}

pub(crate) fn admit<E>(
    work: usize,
    bytes: usize,
    charge: &mut (impl FnMut(usize, usize) -> Result<(), E> + ?Sized),
) -> Result<(), EvalRetainedMetadataError<E>> {
    charge(work, bytes).map_err(EvalRetainedMetadataError::Admission)
}

/// Reserve the requested allocation while its old backing is still live. There
/// is no sharing credit or refund: callers can use a cumulative admission sink.
pub(crate) fn reserve<T, E>(
    values: &mut Vec<T>,
    extra: usize,
    charge: &mut (impl FnMut(usize, usize) -> Result<(), E> + ?Sized),
) -> Result<(), EvalRetainedMetadataError<E>> {
    let wanted = values
        .len()
        .checked_add(extra)
        .ok_or(EvalRetainedMetadataError::CapacityOverflow)?;
    if wanted <= values.capacity() {
        return Ok(());
    }
    let capacity = wanted
        .max(
            values
                .capacity()
                .checked_mul(2)
                .ok_or(EvalRetainedMetadataError::CapacityOverflow)?,
        )
        .max(4);
    let bytes = capacity
        .checked_add(values.capacity())
        .and_then(|n| n.checked_mul(std::mem::size_of::<T>()))
        .ok_or(EvalRetainedMetadataError::CapacityOverflow)?;
    admit(values.len().saturating_add(1), bytes, charge)?;
    values
        .try_reserve_exact(capacity - values.len())
        .map_err(|_| EvalRetainedMetadataError::Allocation)?;
    if values.capacity() > capacity {
        return Err(EvalRetainedMetadataError::CapacityOverflow);
    }
    Ok(())
}

pub(crate) fn push<T, E>(
    values: &mut Vec<T>,
    value: T,
    charge: &mut (impl FnMut(usize, usize) -> Result<(), E> + ?Sized),
) -> Result<(), EvalRetainedMetadataError<E>> {
    reserve(values, 1, charge)?;
    values.push(value);
    Ok(())
}

impl Value {
    /// Visit the real owned backing without cloning a scalar or its shared DAG.
    /// The enclosing Value slot is excluded; weak-float Arc payloads and scratch
    /// allocations are included. Exact pointers deduplicate only this traversal.
    pub fn visit_retained_metadata<E>(
        &self,
        charge: &mut impl FnMut(usize, usize) -> Result<(), E>,
    ) -> Result<(), EvalRetainedMetadataError<E>> {
        admit(1, 0, charge)?;
        match self {
            Self::WeakFloat(value) => crate::floats::retained_metadata::visit(value, charge),
            Self::Literal(_) | Self::Int(_) | Self::Bool(_) | Self::Float(_) => Ok(()),
        }
    }
}
