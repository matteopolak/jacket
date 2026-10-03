//! Sealed retained allocation footprint, independent of validation work.
use super::*;
use jai_types::{DescriptorKind, TypeDescriptor};
use std::mem::size_of;

// Includes the two strong/weak counters preceding each Arc allocation.
const ARC_HEADER: usize = 2 * size_of::<usize>();
struct Footprint {
    bytes: usize,
    limit: usize,
}
impl Footprint {
    fn add(&mut self, bytes: usize) -> Result<(), StaticDataError> {
        self.bytes = self
            .bytes
            .checked_add(bytes)
            .filter(|bytes| *bytes <= self.limit)
            .ok_or(StaticDataError::Limit("retained bytes"))?;
        Ok(())
    }
    fn array<T>(&mut self, capacity: usize) -> Result<(), StaticDataError> {
        self.add(
            capacity
                .checked_mul(size_of::<T>())
                .ok_or(StaticDataError::Limit("retained bytes"))?,
        )
    }
    fn address(&mut self, address: &StaticAddress) -> Result<(), StaticDataError> {
        self.array::<StaticProjection>(address.path.len())?;
        for projection in address.path.iter() {
            if let StaticProjection::ByteView(_) = projection {
                // Conservatively counts repeated view references within one object.
                self.add(ARC_HEADER + size_of::<crate::StaticByteView>())?;
            }
        }
        Ok(())
    }
    fn value(&mut self, value: &StaticValue, depth: usize) -> Result<(), StaticDataError> {
        if depth > 256 {
            return Err(StaticDataError::Limit("value depth"));
        }
        match &value.kind {
            StaticValueKind::Constant(value) => self.constant(value, depth)?,
            StaticValueKind::Record(values) | StaticValueKind::Array(values) => {
                self.array::<StaticValue>(values.capacity())?;
                for child in values {
                    self.value(child, depth + 1)?;
                }
            }
            StaticValueKind::Address(address)
            | StaticValueKind::Slice {
                data: Some(address),
                ..
            } => self.address(address)?,
            StaticValueKind::Slice {
                data: None, ..
            } => {}
        }
        Ok(())
    }
    fn constant(&mut self, value: &ConstantValue, depth: usize) -> Result<(), StaticDataError> {
        if depth > 256 {
            return Err(StaticDataError::Limit("value depth"));
        }
        match &value.kind {
            ConstantKind::StringBytes(bytes) => self.add(bytes.capacity())?,
            ConstantKind::Array(values) | ConstantKind::Record(values) => {
                self.array::<ConstantValue>(values.capacity())?;
                for child in values {
                    self.constant(child, depth + 1)?;
                }
            }
            ConstantKind::Distinct(child)
            | ConstantKind::Union {
                value: child, ..
            } => {
                self.add(size_of::<ConstantValue>())?;
                self.constant(child, depth + 1)?;
            }
            ConstantKind::RuntimeType(_) => return Err(StaticDataError::InvalidValue(value.ty)),
            ConstantKind::Int(_)
            | ConstantKind::Float(_)
            | ConstantKind::Bool(_)
            | ConstantKind::Enum(_)
            | ConstantKind::Procedure(_)
            | ConstantKind::NativePointer(_)
            | ConstantKind::Zero => {}
        }
        Ok(())
    }
    fn notes(&mut self, notes: &[Box<[u8]>]) -> Result<(), StaticDataError> {
        self.array::<Box<[u8]>>(notes.len())?;
        for note in notes {
            self.add(note.len())?;
        }
        Ok(())
    }
    fn descriptor(&mut self, descriptor: &TypeDescriptor) -> Result<(), StaticDataError> {
        if let Some(name) = &descriptor.name {
            self.add(name.len())?;
        }
        if let Some(layout) = &descriptor.layout {
            self.array::<u64>(layout.field_offsets.len())?;
        }
        match &descriptor.kind {
            DescriptorKind::Procedure {
                parameters,
                results,
                ..
            } => {
                self.array::<jai_types::DescriptorId>(parameters.len())?;
                self.array::<jai_types::DescriptorId>(results.len())?;
            }
            DescriptorKind::Record {
                fields,
                constants,
                metadata,
                ..
            } => {
                self.array::<jai_types::ReflectedField>(fields.len())?;
                for field in fields {
                    if let Some(name) = &field.name {
                        self.add(name.len())?;
                    }
                    self.notes(&field.notes)?;
                }
                self.array::<jai_types::ReflectedTypeConstant>(constants.len())?;
                for constant in constants {
                    self.add(constant.name.len())?;
                }
                self.notes(&metadata.notes)?;
            }
            DescriptorKind::Enum {
                members, ..
            } => {
                self.array::<jai_types::ReflectedEnumMember>(members.len())?;
                for member in members {
                    if let Some(name) = &member.name {
                        self.add(name.len())?;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
}

/// Prove a moved value and its immutable descriptor clone before retaining either.
pub(super) fn object(
    value: &StaticValue,
    descriptor: Option<(&StaticAddress, &TypeDescriptor)>,
    limit: usize,
) -> Result<usize, StaticDataError> {
    let mut footprint = Footprint {
        bytes: 0,
        limit,
    };
    footprint.add(ARC_HEADER + size_of::<StaticObject>())?;
    footprint.value(value, 0)?;
    if let Some((header, descriptor)) = descriptor {
        footprint.address(header)?;
        footprint.descriptor(descriptor)?;
    }
    Ok(footprint.bytes)
}

/// Header paths in RuntimeTypeBinding have at most one projection. This bound is
/// admitted before RuntimeTypeBinding::new clones the source descriptor.
pub(super) fn descriptor_preflight(
    value: &StaticValue,
    descriptor: &TypeDescriptor,
    limit: usize,
) -> Result<(), StaticDataError> {
    let mut footprint = Footprint {
        bytes: 0,
        limit,
    };
    footprint.add(ARC_HEADER + size_of::<StaticObject>() + size_of::<StaticProjection>())?;
    footprint.value(value, 0)?;
    footprint.descriptor(descriptor)
}

pub(super) fn table(objects: usize) -> Result<usize, StaticDataError> {
    objects
        .checked_mul(size_of::<Arc<StaticObject>>())
        .and_then(|bytes| bytes.checked_add(ARC_HEADER + size_of::<StaticData>()))
        .ok_or(StaticDataError::Limit("retained bytes"))
}

#[cfg(test)]
mod tests;
