//! Compare serialized descriptors with the checked reflection graph. References
//! check another immutable binding without traversing the descriptor graph.
use super::*;
use jai_types::TypeKind;
use jai_types::{CallingConvention, ContextMode, DistinctKind, IntegerType, ScalarType};

pub(super) fn charge(remaining: &mut usize, count: usize) -> Result<(), StaticDataError> {
    *remaining = remaining
        .checked_sub(count)
        .ok_or(StaticDataError::Limit("descriptor work count"))?;
    Ok(())
}
pub(super) fn metadata_nodes(descriptor: &TypeDescriptor) -> Result<usize, StaticDataError> {
    let maximum = crate::StaticDataLimits::default().value_nodes;
    let mut remaining = maximum;
    charge(&mut remaining, 1)?;
    charge(
        &mut remaining,
        descriptor.name.as_ref().map_or(0, |name| name.len()),
    )?;
    charge(
        &mut remaining,
        descriptor
            .layout
            .as_ref()
            .map_or(0, |layout| layout.field_offsets.len()),
    )?;
    match &descriptor.kind {
        DescriptorKind::Procedure {
            parameters,
            results,
            ..
        } => {
            charge(&mut remaining, parameters.len())?;
            charge(&mut remaining, results.len())?;
        }
        DescriptorKind::Record {
            fields,
            constants,
            metadata,
            ..
        } => {
            charge(&mut remaining, fields.len())?;
            for field in fields {
                charge(
                    &mut remaining,
                    field.name.as_ref().map_or(0, |name| name.len()),
                )?;
                charge(&mut remaining, field.notes.len())?;
                for note in &field.notes {
                    charge(&mut remaining, note.len())?;
                }
            }
            charge(&mut remaining, metadata.notes.len())?;
            charge(&mut remaining, constants.len())?;
            for constant in constants {
                charge(&mut remaining, constant.name.len())?;
            }
            for note in &metadata.notes {
                charge(&mut remaining, note.len())?;
            }
        }
        DescriptorKind::Enum { members, .. } => {
            charge(&mut remaining, members.len())?;
            for member in members {
                charge(
                    &mut remaining,
                    member.name.as_ref().map_or(0, |name| name.len()),
                )?;
            }
        }
        _ => {}
    }
    Ok(maximum - remaining)
}

fn invalid(value: &StaticValue) -> StaticDataError {
    StaticDataError::InvalidValue(value.ty)
}
fn fields(value: &StaticValue, count: usize) -> Result<&[StaticValue], StaticDataError> {
    match &value.kind {
        StaticValueKind::Record(fields) if fields.len() == count => Ok(fields),
        _ => Err(invalid(value)),
    }
}
pub(super) fn validate_shape(
    value: &StaticValue,
    kind: &DescriptorKind,
    schema: RuntimeTypeSchema,
) -> Result<(), StaticDataError> {
    let count = match kind {
        DescriptorKind::Void
        | DescriptorKind::Type
        | DescriptorKind::Code
        | DescriptorKind::Any
        | DescriptorKind::Bool => {
            if value.ty != schema.header_type() {
                return Err(invalid(value));
            }
            return fields(value, 2).map(|_| ());
        }
        DescriptorKind::Float { .. } | DescriptorKind::String => 1,
        DescriptorKind::Integer { .. } | DescriptorKind::Pointer { .. } => 2,
        DescriptorKind::Procedure { .. }
        | DescriptorKind::FixedArray { .. }
        | DescriptorKind::Slice { .. }
        | DescriptorKind::DynamicArray { .. }
        | DescriptorKind::Distinct { .. } => 4,
        DescriptorKind::Record { .. } => 11,
        DescriptorKind::Enum { .. } => 7,
    };
    let values = fields(value, count)?;
    if values[0].ty != schema.header_type() {
        return Err(invalid(value));
    }
    Ok(())
}
fn number(value: &StaticValue, expected: i128) -> Result<(), StaticDataError> {
    if matches!(&value.kind, StaticValueKind::Constant(crate::ConstantValue { kind: ConstantKind::Int(n) | ConstantKind::Enum(n), .. }) if n.value() == expected)
    {
        Ok(())
    } else {
        Err(invalid(value))
    }
}
fn boolean(value: &StaticValue, expected: bool) -> Result<(), StaticDataError> {
    if matches!(&value.kind, StaticValueKind::Constant(crate::ConstantValue { kind: ConstantKind::Bool(n), .. }) if *n == expected)
    {
        Ok(())
    } else {
        Err(invalid(value))
    }
}
fn string(value: &StaticValue, expected: &[u8]) -> Result<(), StaticDataError> {
    if matches!(&value.kind, StaticValueKind::Constant(crate::ConstantValue { kind: ConstantKind::StringBytes(bytes), .. }) if bytes == expected)
    {
        Ok(())
    } else {
        Err(invalid(value))
    }
}
fn zero(value: &StaticValue) -> Result<(), StaticDataError> {
    if matches!(
        &value.kind,
        StaticValueKind::Constant(crate::ConstantValue {
            kind: ConstantKind::Zero,
            ..
        })
    ) {
        Ok(())
    } else {
        Err(invalid(value))
    }
}
fn resolve<'a>(
    data: &'a StaticData,
    address: &StaticAddress,
) -> Result<&'a StaticValue, StaticDataError> {
    let mut value = data.object(address.object())?.value();
    for step in address.path() {
        value = match (step, &value.kind) {
            (StaticProjection::Field(field), StaticValueKind::Record(fields)) => {
                fields.get(field.index())
            }
            (StaticProjection::Index(index), StaticValueKind::Array(values)) => {
                usize::try_from(*index)
                    .ok()
                    .and_then(|index| values.get(index))
            }
            _ => None,
        }
        .ok_or_else(|| invalid(value))?;
    }
    Ok(value)
}
fn view<'a>(
    data: &'a StaticData,
    value: &'a StaticValue,
    expected: usize,
    remaining: &mut usize,
) -> Result<&'a [StaticValue], StaticDataError> {
    let StaticValueKind::Slice {
        data: address,
        count,
    } = &value.kind
    else {
        return Err(invalid(value));
    };
    if usize::try_from(*count).ok() != Some(expected) {
        return Err(invalid(value));
    }
    charge(remaining, expected)?;
    let Some(address) = address else {
        return if expected == 0 {
            Ok(&[])
        } else {
            Err(invalid(value))
        };
    };
    let Some((StaticProjection::Index(start), path)) = address.path().split_last() else {
        return Err(invalid(value));
    };
    charge(remaining, path.len())?;
    let parent = path
        .iter()
        .fold(StaticAddress::new(address.object()), |address, step| {
            address.project(step.clone())
        });
    let StaticValueKind::Array(values) = &resolve(data, &parent)?.kind else {
        return Err(invalid(value));
    };
    let start = usize::try_from(*start).map_err(|_| invalid(value))?;
    let end = start.checked_add(expected).ok_or_else(|| invalid(value))?;
    values.get(start..end).ok_or_else(|| invalid(value))
}
fn reference(
    data: &StaticData,
    value: &StaticValue,
    expected: TypeId,
    identity: RuntimeTypeIdentity,
    header: bool,
) -> Result<(), StaticDataError> {
    let StaticValueKind::Address(address) = &value.kind else {
        return Err(invalid(value));
    };
    let object = data.object(address.object())?;
    let binding = object.descriptor_binding().ok_or_else(|| invalid(value))?;
    let child = binding.identity();
    if child.ty() != expected
        || child.schema() != identity.schema()
        || child.policy() != identity.policy()
        || (header && binding.header() != address)
        || (!header && (!address.path().is_empty() || address.object() != child.object()))
    {
        return Err(invalid(value));
    }
    Ok(())
}
fn references(
    data: &StaticData,
    value: &StaticValue,
    expected: &[DescriptorId],
    identity: RuntimeTypeIdentity,
    remaining: &mut usize,
) -> Result<(), StaticDataError> {
    for (value, descriptor) in view(data, value, expected.len(), remaining)?
        .iter()
        .zip(expected)
    {
        reference(data, value, descriptor.represented_type(), identity, true)?;
    }
    Ok(())
}
fn strings(
    data: &StaticData,
    value: &StaticValue,
    expected: &[Box<[u8]>],
    remaining: &mut usize,
) -> Result<(), StaticDataError> {
    for (value, bytes) in view(data, value, expected.len(), remaining)?
        .iter()
        .zip(expected)
    {
        string(value, bytes)?;
    }
    Ok(())
}
pub(super) fn validate(
    data: &StaticData,
    value: &StaticValue,
    descriptor: &TypeDescriptor,
    identity: RuntimeTypeIdentity,
    types: &dyn TypeView,
    remaining: &mut usize,
) -> Result<(), StaticDataError> {
    validate_shape(value, &descriptor.kind, identity.schema())?;
    let StaticValueKind::Record(values) = &value.kind else {
        return Err(invalid(value));
    };
    match &descriptor.kind {
        DescriptorKind::Void
        | DescriptorKind::Type
        | DescriptorKind::Code
        | DescriptorKind::Any
        | DescriptorKind::Bool
        | DescriptorKind::Float { .. }
        | DescriptorKind::String => {}
        DescriptorKind::Integer { representation } => boolean(&values[1], representation.signed())?,
        DescriptorKind::Pointer { pointee } => {
            reference(data, &values[1], pointee.represented_type(), identity, true)?
        }
        DescriptorKind::FixedArray { element, .. }
        | DescriptorKind::Slice { element }
        | DescriptorKind::DynamicArray { element } => {
            reference(data, &values[1], element.represented_type(), identity, true)?;
            let (kind, count) = match descriptor.kind {
                DescriptorKind::FixedArray { count, .. } => (0, i128::from(count)),
                DescriptorKind::Slice { .. } => (1, -1),
                _ => (2, -1),
            };
            number(&values[2], kind)?;
            number(&values[3], count)?;
        }
        DescriptorKind::Procedure {
            parameters,
            results,
            convention,
            context,
            ..
        } => {
            references(data, &values[1], parameters, identity, remaining)?;
            references(data, &values[2], results, identity, remaining)?;
            let flags = if *context == ContextMode::None { 8 } else { 0 }
                | if *convention == CallingConvention::C {
                    32
                } else {
                    0
                }
                | if *convention == CallingConvention::CppMethod {
                    0x1000_0000
                } else {
                    0
                };
            number(&values[3], flags)?;
        }
        DescriptorKind::Distinct {
            kind,
            representation,
        } => {
            string(&values[1], descriptor.name.as_deref().unwrap_or_default())?;
            reference(
                data,
                &values[2],
                representation.represented_type(),
                identity,
                true,
            )?;
            number(
                &values[3],
                match kind {
                    DistinctKind::Distinct => 1,
                    DistinctKind::IsA => 2,
                },
            )?;
        }
        DescriptorKind::Enum {
            representation,
            members,
            flags,
        } => {
            string(&values[1], descriptor.name.as_deref().unwrap_or_default())?;
            reference(
                data,
                &values[2],
                types.scalar(ScalarType::Int(*representation)),
                identity,
                false,
            )?;
            for (value, member) in view(data, &values[3], members.len(), remaining)?
                .iter()
                .zip(members)
            {
                string(value, member.name.as_deref().unwrap_or_default())?;
            }
            for (value, member) in view(data, &values[4], members.len(), remaining)?
                .iter()
                .zip(members)
            {
                number(
                    value,
                    jai_types::Integer::wrapping(IntegerType::S64, member.value.value()).value(),
                )?;
            }
            number(&values[5], 0)?;
            number(&values[6], i128::from(*flags))?;
        }
        DescriptorKind::Record {
            fields: members,
            constants,
            metadata,
            ..
        } => {
            string(&values[1], descriptor.name.as_deref().unwrap_or_default())?;
            view(data, &values[2], 0, remaining)?;
            let all_members = view(
                data,
                &values[3],
                members
                    .len()
                    .checked_add(constants.len())
                    .ok_or(StaticDataError::Limit("record member count"))?,
                remaining,
            )?;
            let (physical, constant_members) = all_members.split_at(members.len());
            for (value, member) in physical.iter().zip(members) {
                let values = fields(value, 6)?;
                string(&values[0], member.name.as_deref().unwrap_or_default())?;
                reference(
                    data,
                    &values[1],
                    member.ty.represented_type(),
                    identity,
                    true,
                )?;
                number(&values[2], i128::from(member.offset_in_bytes))?;
                let flags = if member.using { 4 } else { 0 }
                    | if member.procedure_as_void_pointer(types, identity.ty())? {
                        8
                    } else {
                        0
                    };
                number(&values[3], flags)?;
                strings(data, &values[4], &member.notes, remaining)?;
                number(&values[5], -1)?;
            }
            number(&values[4], i128::from(metadata.status_flags))?;
            number(&values[5], i128::from(metadata.nontextual_flags))?;
            number(&values[6], i128::from(metadata.textual_flags))?;
            zero(&values[7])?;
            zero(&values[8])?;
            if constants.is_empty() {
                view(data, &values[9], 0, remaining)?;
            } else {
                let StaticValueKind::Slice {
                    data: Some(address),
                    count,
                } = &values[9].kind
                else {
                    return Err(invalid(&values[9]));
                };
                let [StaticProjection::ByteView(proof)] = address.path() else {
                    return Err(invalid(&values[9]));
                };
                let object = data.object(address.object())?;
                let TypeKind::FixedArray {
                    element,
                    count: cells,
                } = *types.kind(object.ty())?
                else {
                    return Err(invalid(object.value()));
                };
                if element != types.meta_type()
                    || usize::try_from(cells).ok() != Some(constants.len())
                    || proof.object() != address.object()
                    || proof.backing_type() != object.ty()
                    || proof.policy() != identity.policy()
                    || proof.offset() != 0
                    || proof.length() != *count
                    || data.address_type(address, types)?
                        != types.scalar(ScalarType::Int(IntegerType::U8))
                {
                    return Err(invalid(&values[9]));
                }
                let layout = jai_types::LayoutEngine::new(types, identity.policy())
                    .layout(object.ty())
                    .map_err(|_| invalid(&values[9]))?
                    .clone();
                if *count != layout.size {
                    return Err(invalid(&values[9]));
                }
                let stride = layout.array_stride.ok_or_else(|| invalid(&values[9]))?;
                let StaticValueKind::Array(stored) = &object.value().kind else {
                    return Err(invalid(object.value()));
                };
                if stored.len() != constants.len() {
                    return Err(invalid(object.value()));
                }
                charge(remaining, stored.len())?;
                for (index, ((row, stored), constant)) in constant_members
                    .iter()
                    .zip(stored)
                    .zip(constants)
                    .enumerate()
                {
                    let row = fields(row, 6)?;
                    string(&row[0], &constant.name)?;
                    reference(data, &row[1], types.meta_type(), identity, true)?;
                    number(&row[2], 0)?;
                    number(&row[3], 1)?;
                    view(data, &row[4], 0, remaining)?;
                    let offset = u64::try_from(index)
                        .ok()
                        .and_then(|index| index.checked_mul(stride))
                        .ok_or_else(|| invalid(&row[5]))?;
                    number(&row[5], i128::from(offset))?;
                    if stored.ty != types.meta_type() {
                        return Err(invalid(stored));
                    }
                    reference(data, stored, constant.represented_type, identity, true)?;
                }
            }
            strings(data, &values[10], &metadata.notes, remaining)?;
        }
    }
    Ok(())
}
