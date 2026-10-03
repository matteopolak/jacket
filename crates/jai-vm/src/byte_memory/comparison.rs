//! Portable equality proofs for ranges containing opaque virtual handles.
use super::ByteImage;
use crate::{Error, Value};

impl ByteImage {
    pub fn range_has_provenance(&self, offset: usize, count: usize) -> Result<bool, Error> {
        let range = self.initialized_range(offset, count)?;
        Ok(!self.address_spans(&range).is_empty())
    }

    /// Prove equality without comparing virtual address tokens numerically.
    ///
    /// The callback compares canonical handle identities. Every tagged span must
    /// have its complete opaque relocation; derived integer bytes and fragments
    /// cannot establish portable equality. Untagged gaps compare as raw bytes.
    pub fn range_provenance_equivalent(
        &self,
        other: &Self,
        offset: usize,
        other_offset: usize,
        count: usize,
        mut equivalent: impl FnMut(&Value, &Value) -> Result<bool, Error>,
    ) -> Result<bool, Error> {
        self.check_target(other.target)?;
        let range = self.initialized_range(offset, count)?;
        let other_range = other.initialized_range(other_offset, count)?;
        let left = self.address_spans(&range);
        let right = other.address_spans(&other_range);
        // Validate the entire ranges before returning any equality result. A
        // differing plain prefix cannot make later derived bytes portable.
        for (image, spans, range) in [(self, left, &range), (other, right, &other_range)] {
            for span in spans {
                if !span.complete
                    || span.offset < range.start
                    || span.offset + span.length > range.end
                    || image.handle(span.offset, span.length).is_none()
                {
                    return Err(Error::UnsupportedPointerOperation(
                        "byte comparison requires complete opaque handle identities",
                    ));
                }
            }
        }
        if left.len() != right.len() {
            return Ok(false);
        }
        let mut cursor = 0;
        for (left, right) in left.iter().zip(right) {
            let relative = left.offset - offset;
            if relative != right.offset - other_offset || left.length != right.length {
                return Ok(false);
            }
            if self.bytes[offset + cursor..offset + relative]
                != other.bytes[other_offset + cursor..other_offset + relative]
            {
                return Ok(false);
            }
            let left = self
                .handle(left.offset, left.length)
                .expect("validated relocation");
            let right = other
                .handle(right.offset, right.length)
                .expect("validated relocation");
            if !equivalent(&left.value, &right.value)? {
                return Ok(false);
            }
            cursor = relative + left.length;
        }
        Ok(self.bytes[offset + cursor..range.end]
            == other.bytes[other_offset + cursor..other_range.end])
    }
}

#[cfg(test)]
mod tests;
