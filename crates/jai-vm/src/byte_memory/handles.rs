use super::provenance::ByteProvenance;
use super::{ByteImage, Endian, NEXT_HANDLE, Relocation};
use crate::{AddressProvenance, Error, LimitKind, Pointer, Value};
use jai_types::TypeId;
use std::collections::HashMap;
use std::sync::atomic::Ordering;

impl ByteImage {
    /// Align canonical handles with an existing image using one borrowed token map.
    /// The caller admits the reference metadata work before invoking this proof.
    pub(crate) fn retokenize_handles_from(&mut self, reference: &Self) -> Result<(), Error> {
        let mut tokens = HashMap::with_capacity(reference.relocations.len());
        for relocation in &reference.relocations {
            let token = reference.bits(relocation.offset, relocation.length)?;
            let receipt = reference.certified_code_pointer(relocation.offset, relocation.length)?;
            if let Some(previous) = tokens.insert(&relocation.value, (token, receipt))
                && previous != (token, receipt)
            {
                return Err(Error::UnsupportedPointerOperation(
                    "aggregate snapshot has inconsistent tokens for one handle",
                ));
            }
        }
        // Collect certified metadata before changing bytes, so proof failure is atomic.
        let mut certifications = Vec::new();
        for relocation in &self.relocations {
            if let Some((_, Some(pointer))) = tokens.get(&relocation.value) {
                let index = self.exact_span_index(relocation.offset, relocation.length)?;
                certifications.push((index, (**pointer).clone()));
            }
        }
        self.retokenize_handles(|value| {
            tokens
                .get(value)
                .map(|(token, _)| *token)
                .ok_or(Error::UnsupportedPointerOperation(
                    "canonical aggregate encoding requires a missing handle",
                ))
        })?;
        for (index, pointer) in certifications {
            self.provenance[index].provenance =
                ByteProvenance::Address(AddressProvenance::Pointer(pointer));
        }
        Ok(())
    }
    fn exact_span_index(&self, offset: usize, length: usize) -> Result<usize, Error> {
        let index = self.provenance.partition_point(|span| span.offset < offset);
        let span = self.provenance.get(index).ok_or(Error::InvalidIr(
            "complete handle is missing its provenance span",
        ))?;
        if span.offset != offset || span.length != length || !span.complete {
            return Err(Error::InvalidIr("handle provenance span is incomplete"));
        }
        Ok(index)
    }
    /// Validate an existing sealed code receipt against the complete visible slot.
    fn certified_code_pointer(
        &self,
        offset: usize,
        length: usize,
    ) -> Result<Option<&Pointer>, Error> {
        let index = self.exact_span_index(offset, length)?;
        let ByteProvenance::Address(AddressProvenance::Pointer(pointer)) =
            &self.provenance[index].provenance
        else {
            return Ok(None);
        };
        let Some(code) = pointer.code_pointer() else {
            return Ok(None);
        };
        self.initialized_range(offset, length)?;
        if length as u64 != self.target.policy.pointer().size
            || self.bits(offset, length)? != code.token()
        {
            return Err(Error::UnsupportedPointerOperation(
                "code receipt does not match visible handle bytes",
            ));
        }
        let relocation = self.handle(offset, length).ok_or(Error::InvalidIr(
            "code receipt requires a complete relocation",
        ))?;
        match &relocation.value {
            Value::Procedure {
                signature,
                procedure: Some(procedure),
            } if *signature == code.signature() && *procedure == code.procedure() => {}
            Value::Pointer(original) if original.code_pointer() == Some(code) => {}
            _ => {
                return Err(Error::InvalidIr(
                    "code receipt does not match relocation identity",
                ));
            }
        }
        Ok(Some(pointer))
    }
    /// Memory alone issues these receipts after canonical token admission.
    /// Every callback and identity check precedes any provenance mutation.
    pub(crate) fn certify_code_handles(
        &mut self,
        mut certify: impl FnMut(&Value) -> Result<Pointer, Error>,
    ) -> Result<(), Error> {
        let mut patches = Vec::new();
        for relocation in &self.relocations {
            let Value::Procedure {
                signature,
                procedure: Some(procedure),
            } = &relocation.value
            else {
                continue;
            };
            let index = self.exact_span_index(relocation.offset, relocation.length)?;
            self.initialized_range(relocation.offset, relocation.length)?;
            let pointer = certify(&relocation.value)?;
            let code = pointer.code_pointer().ok_or(Error::InvalidIr(
                "procedure certification requires a sealed code pointer",
            ))?;
            if code.signature() != *signature
                || code.procedure() != *procedure
                || relocation.length as u64 != self.target.policy.pointer().size
                || self.bits(relocation.offset, relocation.length)? != code.token()
            {
                return Err(Error::UnsupportedPointerOperation(
                    "procedure certification does not match canonical handle",
                ));
            }
            match &self.provenance[index].provenance {
                ByteProvenance::Procedure => {}
                ByteProvenance::Address(AddressProvenance::Pointer(old))
                    if old.code_pointer() == Some(code) => {}
                _ => {
                    return Err(Error::InvalidIr(
                        "procedure certification cannot replace unrelated provenance",
                    ));
                }
            }
            patches.push((index, pointer));
        }
        for (index, pointer) in patches {
            self.provenance[index].provenance =
                ByteProvenance::Address(AddressProvenance::Pointer(pointer));
        }
        Ok(())
    }
    pub(crate) fn validate_complete_handles(
        &self,
        mut validate: impl FnMut(&Value) -> Result<(), Error>,
    ) -> Result<(), Error> {
        for relocation in &self.relocations {
            validate(&relocation.value)?;
        }
        Ok(())
    }
    pub(super) fn encode_handle(
        &mut self,
        offset: usize,
        length: usize,
        value: &Value,
    ) -> Result<(), Error> {
        self.range(offset, length)?;
        if length == 0 || length > 8 {
            return Err(Error::InvalidIr(
                "virtual handle storage width is unsupported",
            ));
        }
        if let Value::Pointer(pointer) = value
            && let Some((bits, width)) = pointer.opaque_address_bits()
        {
            if u64::from(width) != self.target.policy.pointer().size * 8
                || length as u64 != self.target.policy.pointer().size
            {
                return Err(Error::UnsupportedPointerOperation(
                    "numeric pointer belongs to another target width",
                ));
            }
            return self.put_bits(offset, length, bits);
        }
        match value {
            Value::Pointer(pointer) if pointer.is_null() => return Ok(()),
            Value::Procedure {
                procedure: None, ..
            } => return Ok(()),
            Value::Pointer(_)
            | Value::Procedure {
                ..
            } => {}
            _ => {
                return Err(Error::InvalidIr(
                    "handle encoding requires pointer or procedure",
                ));
            }
        }
        // A reboxed sealed code pointer already has canonical visible bits.
        let token = match value {
            Value::Pointer(pointer) if pointer.code_pointer().is_some() => {
                pointer.code_pointer().unwrap().token()
            }
            _ => NEXT_HANDLE
                .try_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_add(1))
                .map_err(|_| Error::Limit(LimitKind::Allocations))?,
        };
        if length < 8 && token >= (1u64 << (length * 8)) {
            return Err(Error::Limit(LimitKind::Allocations));
        }
        self.put_bits(offset, length, token)?;
        self.mark_handle_provenance(offset, length, value);
        self.relocations.push(Relocation {
            offset,
            length,
            value: value.clone(),
        });
        Ok(())
    }
    pub(super) fn decode_pointer(
        &self,
        offset: usize,
        length: usize,
        pointee: TypeId,
        remaining: &mut usize,
    ) -> Result<Pointer, Error> {
        self.initialized_range(offset, length)?;
        if let Some(relocation) = self.handle(offset, length) {
            let pointer = match &relocation.value {
                Value::Pointer(pointer) => {
                    if pointer.code_pointer().is_some() {
                        self.certified_code_pointer(offset, length)?
                            .ok_or(Error::InvalidIr(
                                "code pointer lacks sealed byte provenance",
                            ))?;
                    }
                    pointer
                }
                Value::Procedure {
                    ..
                } => self.certified_code_pointer(offset, length)?.ok_or(
                    Error::UnsupportedPointerOperation(
                        "unowned procedure bytes cannot construct a code pointer",
                    ),
                )?,
                _ => return Err(Error::InvalidIr("unexpected pointer relocation class")),
            };
            *remaining = remaining
                .checked_sub(pointer.metadata_cells())
                .ok_or(Error::Limit(LimitKind::ValueCells))?;
            Ok(pointer.retype(pointee))
        } else {
            self.reject_address_view(
                offset,
                length,
                "tagged integer bytes require an explicit integer-to-pointer cast",
            )?;
            Pointer::opaque(
                self.bits(offset, length)?,
                u32::try_from(self.target.policy.pointer().size * 8)
                    .map_err(|_| Error::CheckedCast)?,
                pointee,
            )
        }
    }
    pub(super) fn decode_procedure(
        &self,
        offset: usize,
        length: usize,
        signature: TypeId,
    ) -> Result<Value, Error> {
        self.initialized_range(offset, length)?;
        if let Some(relocation) = self.handle(offset, length) {
            match &relocation.value {
                Value::Procedure {
                    signature: original,
                    ..
                } => {
                    if *original != signature {
                        return Err(Error::TypeMismatch {
                            expected: signature,
                        });
                    }
                    Ok(relocation.value.clone())
                }
                Value::Pointer(_) => {
                    let pointer = self.certified_code_pointer(offset, length)?.ok_or(
                        Error::UnsupportedPointerOperation(
                            "data pointer cannot construct a procedure",
                        ),
                    )?;
                    let code = pointer
                        .code_pointer()
                        .expect("checked sealed code provenance");
                    if code.signature() != signature {
                        return Err(Error::TypeMismatch {
                            expected: signature,
                        });
                    }
                    Ok(Value::Procedure {
                        signature,
                        procedure: Some(code.procedure()),
                    })
                }
                _ => Err(Error::InvalidIr("unexpected procedure relocation class")),
            }
        } else {
            self.reject_address_view(
                offset,
                length,
                "tagged address bytes cannot construct procedure values",
            )?;
            if self.bits(offset, length)? == 0 {
                Ok(Value::Procedure {
                    signature,
                    procedure: None,
                })
            } else {
                Err(Error::InvalidIr(
                    "byte storage cannot forge procedure provenance",
                ))
            }
        }
    }
    /// Replace visible handle bytes using the Memory owner's stable address tokens.
    ///
    /// The callback must map one canonical virtual address or procedure identity to
    /// one nonzero token, and keep different identities distinct. Different pointer
    /// views may share an address token. Complete relocations retain their original
    /// typed values; bytes lacking provenance are never repaired by this operation.
    /// A callback error or invalid/inconsistent token leaves this image unchanged.
    pub fn retokenize_handles(
        &mut self,
        mut token: impl FnMut(&Value) -> Result<u64, Error>,
    ) -> Result<(), Error> {
        let mut seen = HashMap::new();
        let mut patches = Vec::with_capacity(self.relocations.len());
        for relocation in &self.relocations {
            let range = self.range(relocation.offset, relocation.length)?;
            if relocation.length == 0 || relocation.length > 8 {
                return Err(Error::InvalidIr(
                    "virtual handle storage width is unsupported",
                ));
            }
            let bits = token(&relocation.value)?;
            if bits == 0 {
                return Err(Error::InvalidIr(
                    "live virtual handle cannot use a null byte token",
                ));
            }
            if relocation.length < 8 && bits >= (1u64 << (relocation.length * 8)) {
                return Err(Error::InvalidIr(
                    "virtual handle byte token exceeds target width",
                ));
            }
            if let Some(previous) = seen.insert(&relocation.value, bits)
                && previous != bits
            {
                return Err(Error::InvalidIr(
                    "identical virtual handles received inconsistent byte tokens",
                ));
            }
            let bytes = match self.target.endian {
                Endian::Little => bits.to_le_bytes(),
                Endian::Big => bits.to_be_bytes(),
            };
            let bytes = match self.target.endian {
                Endian::Little => bytes[..relocation.length].to_vec(),
                Endian::Big => bytes[8 - relocation.length..].to_vec(),
            };
            patches.push((range, bytes));
        }
        // All callbacks, bounds, widths and consistency checks precede mutation.
        for (range, bytes) in patches {
            self.bytes[range].copy_from_slice(&bytes);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
