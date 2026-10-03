//! Provenance overlays follow bytes independently of complete opaque relocations.
use super::{ByteImage, overlap};
use crate::{AddressProvenance, Error, LimitKind, Number, Value};
use jai_types::{Integer, IntegerType};
use std::collections::BTreeSet;
use std::ops::Range;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) enum ByteProvenance {
    Address(AddressProvenance),
    Procedure,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct ByteSpan {
    pub offset: usize,
    pub length: usize,
    pub complete: bool,
    pub provenance: ByteProvenance,
}
impl ByteSpan {
    fn allocation_ids(&self) -> Vec<u64> {
        match &self.provenance {
            ByteProvenance::Address(AddressProvenance::Pointer(pointer)) => pointer
                .data_allocation_key()
                .map(|(_, id)| id)
                .into_iter()
                .collect(),
            ByteProvenance::Address(AddressProvenance::Derived {
                allocations, ..
            }) => allocations.to_vec(),
            ByteProvenance::Procedure => vec![],
        }
    }
    pub(super) fn memory_identity(&self) -> Option<u64> {
        match &self.provenance {
            ByteProvenance::Address(AddressProvenance::Pointer(pointer)) => {
                Some(pointer.memory_identity())
            }
            ByteProvenance::Address(AddressProvenance::Derived {
                memory, ..
            }) => Some(*memory),
            ByteProvenance::Procedure => None,
        }
    }
}
impl ByteImage {
    /// Inspect allocation origins in every retained byte, including inactive union bytes.
    /// Visits borrow metadata and may repeat an origin shared by several intervals.
    /// Callers charge `metadata_cells()` before traversing an untrusted image.
    pub fn visit_address_origins(
        &self,
        mut visit: impl FnMut(u64, u64) -> Result<(), Error>,
    ) -> Result<(), Error> {
        for span in &self.provenance {
            match &span.provenance {
                ByteProvenance::Address(AddressProvenance::Pointer(pointer)) => {
                    if let Some((memory, allocation)) = pointer.data_allocation_key() {
                        visit(memory, allocation)?;
                    }
                }
                ByteProvenance::Address(AddressProvenance::Derived {
                    memory,
                    allocations,
                }) => {
                    for allocation in allocations {
                        visit(*memory, *allocation)?;
                    }
                }
                ByteProvenance::Procedure => {}
            }
        }
        Ok(())
    }
    // Spans never overlap. Mutations restore ordering before any public read.
    pub(super) fn address_spans(&self, range: &Range<usize>) -> &[ByteSpan] {
        if range.is_empty() {
            return &[];
        }
        let start = self
            .provenance
            .partition_point(|p| p.offset + p.length <= range.start);
        let end = self.provenance.partition_point(|p| p.offset < range.end);
        &self.provenance[start..end]
    }
    pub(super) fn mark_handle_provenance(&mut self, offset: usize, length: usize, value: &Value) {
        let provenance = match value {
            Value::Pointer(pointer) => {
                ByteProvenance::Address(AddressProvenance::Pointer(pointer.clone()))
            }
            Value::Procedure {
                ..
            } => ByteProvenance::Procedure,
            _ => unreachable!("handle encoder validated the relocation class"),
        };
        self.provenance.push(ByteSpan {
            offset,
            length,
            complete: true,
            provenance,
        });
    }
    pub(super) fn mark_number_provenance(
        &mut self,
        offset: usize,
        length: usize,
        number: &Number,
    ) -> Result<(), Error> {
        if number
            .provenance()
            .map_or(0, super::metadata::address_metadata_cells)
            > self.limit
        {
            return Err(Error::Limit(LimitKind::ValueCells));
        }
        if let Some(provenance) = number.provenance() {
            self.provenance.push(ByteSpan {
                offset,
                length,
                complete: true,
                provenance: ByteProvenance::Address(provenance.clone()),
            });
        }
        Ok(())
    }
    pub(super) fn decode_number(
        &self,
        offset: usize,
        length: usize,
        ty: IntegerType,
        remaining: &mut usize,
    ) -> Result<Value, Error> {
        let range = self.initialized_range(offset, length)?;
        let integer = Integer::wrapping(ty, i128::from(self.bits(offset, length)?));
        let touching = self.address_spans(&range);
        if touching.is_empty() {
            return Ok(Value::Int(integer));
        }
        let memory = touching[0]
            .memory_identity()
            .ok_or(Error::UnsupportedPointerOperation(
                "procedure address bytes cannot be interpreted as integers",
            ))?;
        for span in touching {
            if span.memory_identity() != Some(memory) {
                return Err(Error::UnsupportedPointerOperation(
                    "integer byte view combines incompatible address provenance",
                ));
            }
        }
        let mut origins = BTreeSet::new();
        for span in touching {
            for allocation in span.allocation_ids() {
                origins.insert(allocation);
                if origins.len() > self.limit {
                    return Err(Error::Limit(LimitKind::ValueCells));
                }
            }
        }
        let exact_pointer = if touching.len() == 1
            && touching[0].offset == offset
            && touching[0].length == length
            && touching[0].complete
            && length as u64 >= self.target.policy.pointer().size
        {
            match &touching[0].provenance {
                ByteProvenance::Address(AddressProvenance::Pointer(pointer)) => Some(pointer),
                _ => None,
            }
        } else {
            None
        };
        let cells = if let Some(pointer) = exact_pointer {
            1usize.saturating_add(pointer.metadata_cells())
        } else {
            origins.len()
        };
        *remaining = remaining
            .checked_sub(cells)
            .ok_or(Error::Limit(LimitKind::ValueCells))?;
        let provenance = if let Some(pointer) = exact_pointer {
            AddressProvenance::Pointer(pointer.clone())
        } else {
            AddressProvenance::Derived {
                memory,
                allocations: origins.into_iter().collect(),
            }
        };
        Ok(Number::address(integer, provenance).into_value())
    }
    pub(super) fn reject_address_view(
        &self,
        offset: usize,
        length: usize,
        reason: &'static str,
    ) -> Result<(), Error> {
        let range = self.range(offset, length)?;
        if !self.address_spans(&range).is_empty() {
            return Err(Error::UnsupportedPointerOperation(reason));
        }
        Ok(())
    }
    /// Integer byte fills retain address provenance instead of exposing guessed bits.
    pub fn fill_range_number(
        &mut self,
        offset: usize,
        count: usize,
        number: Number,
    ) -> Result<(), Error> {
        let range = self.range(offset, count)?;
        if number.metadata_cells() > self.limit {
            return Err(Error::Limit(LimitKind::ValueCells));
        }
        let incoming = if range.is_empty() || number.provenance().is_none() {
            0
        } else {
            super::metadata::derived_metadata_cells(number.origin_count())
        };
        self.check_range_replacement(&range, incoming)?;
        let allocations = number.allocation_ids();
        self.fill_range(offset, count, number.bits() as u8)?;
        if let Some(memory) = number.memory_identity()
            && !range.is_empty()
        {
            self.provenance.push(ByteSpan {
                offset,
                length: count,
                complete: false,
                provenance: ByteProvenance::Address(AddressProvenance::Derived {
                    memory,
                    allocations: allocations.into_boxed_slice(),
                }),
            });
            self.normalize_metadata();
        }
        Ok(())
    }
    /// Clear only written bytes, retaining the provenance of untouched fragments.
    pub(super) fn clear_provenance(&mut self, range: &Range<usize>) {
        if range.is_empty() {
            return;
        }
        let mut remaining = Vec::with_capacity(self.provenance.len());
        for span in self.provenance.drain(..) {
            let end = span.offset + span.length;
            if !overlap(range.start, range.end, span.offset, end) {
                remaining.push(span);
                continue;
            }
            if span.offset < range.start {
                remaining.push(ByteSpan {
                    offset: span.offset,
                    length: range.start - span.offset,
                    complete: false,
                    provenance: span.provenance.clone(),
                });
            }
            if end > range.end {
                remaining.push(ByteSpan {
                    offset: range.end,
                    length: end - range.end,
                    complete: false,
                    provenance: span.provenance,
                });
            }
        }
        self.provenance = remaining;
    }
    pub(super) fn capture_provenance(&self, range: &Range<usize>) -> Vec<ByteSpan> {
        if range.is_empty() {
            return vec![];
        }
        self.address_spans(range)
            .iter()
            .filter_map(|span| {
                let start = range.start.max(span.offset);
                let end = range.end.min(span.offset + span.length);
                if start >= end {
                    return None;
                }
                let provenance = if start != span.offset || end != span.offset + span.length {
                    match &span.provenance {
                        ByteProvenance::Address(_) => {
                            ByteProvenance::Address(AddressProvenance::Derived {
                                memory: span
                                    .memory_identity()
                                    .expect("address provenance has a memory identity"),
                                allocations: span.allocation_ids().into_boxed_slice(),
                            })
                        }
                        ByteProvenance::Procedure => ByteProvenance::Procedure,
                    }
                } else {
                    span.provenance.clone()
                };
                Some(ByteSpan {
                    offset: start - range.start,
                    length: end - start,
                    complete: span.complete
                        && start == span.offset
                        && end == span.offset + span.length,
                    provenance,
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests;
