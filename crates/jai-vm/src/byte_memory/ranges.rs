use super::{ByteImage, ByteSpan, Relocation, UnionView, overlap};
use crate::{Error, LimitKind};
use std::ops::Range;

struct CopySnapshot {
    bytes: Vec<u8>,
    initialized: Vec<bool>,
    provenance: Vec<ByteSpan>,
    relocations: Vec<Relocation>,
    unions: Vec<UnionView>,
}

impl ByteImage {
    /// Assemble complete images in one pass without repeated whole-buffer cloning.
    pub fn concatenate(
        images: &[ByteImage],
        target: super::ByteTarget,
        limit: usize,
    ) -> Result<Self, Error> {
        let mut total = 0usize;
        let mut metadata = 0usize;
        for image in images {
            image.check_target(target)?;
            metadata = metadata
                .checked_add(image.metadata_cells())
                .filter(|n| *n <= limit)
                .ok_or(Error::Limit(LimitKind::ValueCells))?;
            total = total
                .checked_add(image.len())
                .filter(|n| *n <= limit)
                .ok_or(Error::Limit(LimitKind::ValueCells))?;
        }
        let mut result = Self {
            bytes: Vec::with_capacity(total),
            initialized: Vec::with_capacity(total),
            provenance: vec![],
            relocations: vec![],
            unions: vec![],
            target,
            limit,
        };
        for image in images {
            let offset = result.bytes.len();
            result.bytes.extend_from_slice(&image.bytes);
            result.initialized.extend_from_slice(&image.initialized);
            result
                .provenance
                .extend(image.provenance.iter().cloned().map(|mut p| {
                    p.offset += offset;
                    p
                }));
            result
                .relocations
                .extend(image.relocations.iter().cloned().map(|mut r| {
                    r.offset += offset;
                    r
                }));
            result
                .unions
                .extend(image.unions.iter().cloned().map(|mut u| {
                    u.offset += offset;
                    u
                }));
        }
        Ok(result)
    }
    /// Extract only the selected bytes, initialization and intersecting provenance.
    pub fn extract_range(&self, offset: usize, count: usize) -> Result<Self, Error> {
        let range = self.range(offset, count)?;
        let snapshot = self.capture_range(range);
        Ok(Self {
            bytes: snapshot.bytes,
            initialized: snapshot.initialized,
            provenance: snapshot.provenance,
            relocations: snapshot.relocations,
            unions: snapshot.unions,
            target: self.target,
            limit: self.limit,
        })
    }
    /// Raw bytes are observable; reading them does not transfer handle provenance.
    pub fn read_range(&self, offset: usize, count: usize) -> Result<&[u8], Error> {
        Ok(&self.bytes[self.initialized_range(offset, count)?])
    }
    /// Raw writes invalidate every overlapping handle and union reconstruction view.
    pub fn write_range(&mut self, offset: usize, bytes: &[u8]) -> Result<(), Error> {
        let range = self.range(offset, bytes.len())?;
        self.check_range_replacement(&range, 0)?;
        self.bytes[range.clone()].copy_from_slice(bytes);
        self.initialized[range.clone()].fill(true);
        self.invalidate_range(&range);
        Ok(())
    }
    pub fn fill_range(&mut self, offset: usize, count: usize, byte: u8) -> Result<(), Error> {
        let range = self.range(offset, count)?;
        self.check_range_replacement(&range, 0)?;
        self.bytes[range.clone()].fill(byte);
        self.initialized[range.clone()].fill(true);
        self.invalidate_range(&range);
        Ok(())
    }
    /// Copies complete source relocations, never partial virtual handles.
    ///
    /// This operates on a distinct immutable source image. The caller enforcing
    /// `memcpy` semantics must reject overlapping regions of one allocation.
    pub fn copy_range_from(
        &mut self,
        source: &Self,
        source_offset: usize,
        destination_offset: usize,
        count: usize,
    ) -> Result<(), Error> {
        self.check_target(source.target)?;
        let source_range = source.range(source_offset, count)?;
        let destination = self.range(destination_offset, count)?;
        if count == 0 {
            return Ok(());
        }
        self.check_range_replacement(&destination, source.range_metadata_cells(&source_range))?;
        let snapshot = source.capture_range(source_range);
        self.apply_copy(destination, snapshot);
        Ok(())
    }
    /// Same-image copy with explicit `memmove` overlap semantics.
    pub fn copy_range_within(
        &mut self,
        source_offset: usize,
        destination_offset: usize,
        count: usize,
    ) -> Result<(), Error> {
        let source = self.range(source_offset, count)?;
        let destination = self.range(destination_offset, count)?;
        if count == 0 {
            return Ok(());
        }
        // Capture bytes and complete metadata before invalidating the destination.
        self.check_range_replacement(&destination, self.range_metadata_cells(&source))?;
        let snapshot = self.capture_range(source);
        self.apply_copy(destination, snapshot);
        Ok(())
    }
    pub(super) fn selected_relocations(&self, range: &Range<usize>) -> &[Relocation] {
        let start = self.relocations.partition_point(|r| r.offset < range.start);
        let end = self.relocations.partition_point(|r| r.offset < range.end);
        &self.relocations[start..end]
    }
    pub(super) fn selected_unions(&self, range: &Range<usize>) -> &[UnionView] {
        let start = self.unions.partition_point(|u| u.offset < range.start);
        let end = self.unions.partition_point(|u| u.offset <= range.end);
        &self.unions[start..end]
    }
    fn capture_range(&self, range: Range<usize>) -> CopySnapshot {
        let relocations = self
            .selected_relocations(&range)
            .iter()
            .filter(|r| contained(&range, r.offset, r.length))
            .cloned()
            .map(|mut r| {
                r.offset -= range.start;
                r
            })
            .collect();
        let unions = self
            .selected_unions(&range)
            .iter()
            .filter(|u| contained(&range, u.offset, u.length))
            .cloned()
            .map(|mut u| {
                u.offset -= range.start;
                u
            })
            .collect();
        CopySnapshot {
            provenance: self.capture_provenance(&range),
            initialized: self.initialized[range.clone()].to_vec(),
            bytes: self.bytes[range].to_vec(),
            relocations,
            unions,
        }
    }
    fn apply_copy(&mut self, destination: Range<usize>, snapshot: CopySnapshot) {
        self.bytes[destination.clone()].copy_from_slice(&snapshot.bytes);
        self.initialized[destination.clone()].copy_from_slice(&snapshot.initialized);
        self.invalidate_range(&destination);
        self.provenance
            .extend(snapshot.provenance.into_iter().map(|mut span| {
                span.offset += destination.start;
                span
            }));
        self.relocations
            .extend(snapshot.relocations.into_iter().map(|mut r| {
                r.offset += destination.start;
                r
            }));
        self.unions.extend(snapshot.unions.into_iter().map(|mut u| {
            u.offset += destination.start;
            u
        }));
        self.normalize_metadata();
    }
    fn invalidate_range(&mut self, range: &Range<usize>) {
        self.clear_provenance(range);
        self.relocations
            .retain(|r| !overlap(range.start, range.end, r.offset, r.offset + r.length));
        self.unions
            .retain(|u| !overlap(range.start, range.end, u.offset, u.offset + u.length));
    }
}
pub(super) fn contained(range: &Range<usize>, offset: usize, length: usize) -> bool {
    offset >= range.start && offset <= range.end && length <= range.end - offset
}

#[cfg(test)]
mod tests;
