//! Bound metadata before cloning origin sets or splitting existing intervals.
use super::{ByteImage, ByteSpan, Relocation, overlap};
use crate::{AddressProvenance, Error, LimitKind, Value};
use std::ops::Range;

impl ByteSpan {
    pub(super) fn metadata_cells(&self) -> usize {
        match &self.provenance {
            super::provenance::ByteProvenance::Address(address) => address_metadata_cells(address),
            _ => 1,
        }
    }
}
/// A retained interval occupies a record even when code-derived data origins are empty.
pub(super) fn address_metadata_cells(address: &AddressProvenance) -> usize {
    match address {
        AddressProvenance::Derived { allocations, .. } => derived_metadata_cells(allocations.len()),
        AddressProvenance::Pointer(pointer) => 1usize.saturating_add(pointer.metadata_cells()),
    }
}
pub(super) fn derived_metadata_cells(origins: usize) -> usize {
    origins.max(1)
}
impl Relocation {
    pub(super) fn metadata_cells(&self) -> usize {
        1usize.saturating_add(match &self.value {
            Value::Pointer(pointer) => pointer.metadata_cells(),
            _ => 0,
        })
    }
}
impl ByteImage {
    /// Constant-time admission cost before visiting retained metadata records.
    pub(crate) fn metadata_inspection_work(&self) -> usize {
        self.provenance
            .len()
            .saturating_add(self.relocations.len())
            .saturating_add(self.unions.len())
    }
    /// Work/storage cells for origin IDs, complete relocations and union views.
    /// Callers doing bulk assembly charge these cells in addition to byte work.
    pub fn metadata_cells(&self) -> usize {
        self.provenance.iter().fold(
            self.relocations.iter().fold(self.unions.len(), |sum, r| {
                sum.saturating_add(r.metadata_cells())
            }),
            |sum, span| sum.saturating_add(span.metadata_cells()),
        )
    }
    pub(super) fn check_metadata_cells(&self, cells: usize) -> Result<(), Error> {
        if cells > self.limit {
            return Err(Error::Limit(LimitKind::ValueCells));
        }
        Ok(())
    }
    pub(super) fn retained_provenance_cells(&self, range: &Range<usize>) -> usize {
        self.provenance.iter().fold(0usize, |sum, span| {
            let end = span.offset + span.length;
            let fragments = if !overlap(range.start, range.end, span.offset, end) {
                1
            } else {
                usize::from(span.offset < range.start) + usize::from(end > range.end)
            };
            sum.saturating_add(span.metadata_cells().saturating_mul(fragments))
        })
    }
    pub(super) fn check_range_replacement(
        &self,
        range: &Range<usize>,
        incoming: usize,
    ) -> Result<(), Error> {
        let handles = self
            .relocations
            .iter()
            .filter(|r| !overlap(range.start, range.end, r.offset, r.offset + r.length))
            .fold(0usize, |sum, r| sum.saturating_add(r.metadata_cells()));
        let unions = self
            .unions
            .iter()
            .filter(|u| !overlap(range.start, range.end, u.offset, u.offset + u.length))
            .count();
        self.check_metadata_cells(
            self.retained_provenance_cells(range)
                .saturating_add(handles)
                .saturating_add(unions)
                .saturating_add(incoming),
        )
    }
    /// Metadata work in a selected range, computed by borrowing existing records.
    pub(crate) fn range_metadata_work(&self, offset: usize, bytes: usize) -> Result<usize, Error> {
        Ok(self.range_metadata_cells(&self.range(offset, bytes)?))
    }
    pub(super) fn range_metadata_cells(&self, range: &Range<usize>) -> usize {
        let origins = self.address_spans(range).iter().fold(0usize, |sum, span| {
            let cells = match &span.provenance {
                super::provenance::ByteProvenance::Address(AddressProvenance::Pointer(_))
                    if span.offset < range.start || span.offset + span.length > range.end =>
                {
                    1
                }
                _ => span.metadata_cells(),
            };
            sum.saturating_add(cells)
        });
        let handles = self
            .selected_relocations(range)
            .iter()
            .filter(|r| super::ranges::contained(range, r.offset, r.length))
            .fold(0usize, |sum, r| sum.saturating_add(r.metadata_cells()));
        let unions = self
            .selected_unions(range)
            .iter()
            .filter(|u| super::ranges::contained(range, u.offset, u.length))
            .count();
        origins.saturating_add(handles).saturating_add(unions)
    }
}

/// Preflight both independent pointer clones created by the encoder.
pub(super) fn encoded_metadata_cells(value: &Value, limit: usize) -> Result<usize, Error> {
    let mut pending = vec![value];
    let mut total = 0usize;
    while let Some(value) = pending.pop() {
        let cells = match value {
            Value::StoredAggregate(snapshot) => snapshot.image().metadata_cells(),
            Value::DynamicArray {
                pointer, allocator, ..
            } => {
                if let Some(allocator) = allocator {
                    pending.push(allocator.as_ref());
                }
                if pointer.is_null() {
                    0
                } else {
                    1usize
                        .saturating_add(pointer.metadata_cells())
                        .saturating_mul(2)
                }
            }
            Value::Pointer(pointer)
            | Value::Slice { pointer, .. }
            | Value::StringView { pointer, .. }
            | Value::Type {
                descriptor: Some(pointer),
            } if !pointer.is_null() && !pointer.is_opaque() => 1usize
                .saturating_add(pointer.metadata_cells())
                .saturating_mul(2),
            Value::Procedure {
                procedure: Some(_), ..
            } => 2,
            Value::AddressInteger(number) => number.provenance().map_or(0, address_metadata_cells),
            Value::Record { fields, .. }
            | Value::Array {
                elements: fields, ..
            } => {
                pending.extend(fields);
                0
            }
            Value::Distinct { value, .. } => {
                pending.push(value);
                0
            }
            Value::Union { value, .. } => {
                pending.push(value);
                1
            }
            _ => 0,
        };
        total = total
            .checked_add(cells)
            .filter(|n| *n <= limit)
            .ok_or(Error::Limit(LimitKind::ValueCells))?;
    }
    Ok(total)
}

#[cfg(test)]
mod tests;
