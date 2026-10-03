//! Apply frozen operations to captured operands; never evaluate their source IR.
use super::super::*;
use super::{Operand, plan::*};

pub(super) fn replace_record_field<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
    vm: &mut Vm<'_, P, E>,
    record: Value,
    ty: TypeId,
    field: jai_types::FieldId,
    value: Value,
    _depth: usize,
) -> Result<Value> {
    vm.provider.types().validate_field(ty, field)?;
    let expected = vm.provider.types().field_type(field)?;
    value.validate(
        vm.provider.types(),
        expected,
        vm.limits.evaluation_depth.min(256),
    )?;
    let record = match record {
        Value::StoredAggregate(snapshot) => {
            vm.charge_work(
                snapshot
                    .storage_cells()
                    .checked_add(value.cells(vm.limits.value_cells)?)
                    .ok_or(Error::Limit(LimitKind::Fuel))?,
            )?;
            snapshot.with_field(
                vm.provider.types(),
                field.index(),
                &value,
                vm.limits.value_cells,
            )?
        }
        Value::Record {
            ty: actual,
            mut fields,
        } if actual == ty => {
            let slot = fields
                .get_mut(field.index())
                .ok_or(Error::InvalidIr("record initializer field missing"))?;
            *slot = value;
            Value::Record {
                ty,
                fields,
            }
        }
        _ => return Err(Error::InvalidIr("record build requires a record value").into()),
    };
    vm.charge_work(record.cells(vm.limits.value_cells)?)?;
    Ok(record)
}

/// Capture descriptor storage before evaluating an index that may mutate its base.
pub(super) fn snapshot_index_base<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
    vm: &mut Vm<'_, P, E>,
    base: &Pointer,
    base_ty: TypeId,
    depth: usize,
) -> Result<Option<Value>> {
    if matches!(
        vm.provider.types().kind(base_ty)?,
        TypeKind::FixedArray { .. }
    ) {
        return Ok(None);
    }
    vm.ensure_string_storage(base, depth + 1)?;
    Ok(Some(vm.load_sequence_value(base)?))
}

pub(super) fn index_place<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
    vm: &mut Vm<'_, P, E>,
    base: Pointer,
    snapshot: Option<Value>,
    index: Value,
    base_ty: TypeId,
    ty: TypeId,
    check: CheckMode,
) -> Result<Pointer> {
    let index = usize::try_from(number(vm, index)?.portable_integer()?.value()).map_err(|_| {
        Error::OutOfBounds {
            index: usize::MAX,
            length: 0,
        }
    })?;
    let pointer = if matches!(
        vm.provider.types().kind(base_ty)?,
        TypeKind::FixedArray { .. }
    ) {
        vm.prepare_pointer_layouts(&base, true)?;
        vm.memory.index(vm.provider.types(), &base, index)?
    } else {
        let (pointer, count) =
            match snapshot.ok_or(Error::InvalidIr("missing indexed descriptor snapshot"))? {
                Value::Slice {
                    pointer,
                    count,
                    ..
                }
                | Value::DynamicArray {
                    pointer,
                    count,
                    ..
                }
                | Value::StringView {
                    pointer,
                    count,
                } => (pointer, Some(count)),
                Value::Pointer(pointer) => (pointer, None),
                _ => {
                    return Err(
                        Error::InvalidIr("index place requires array, slice or pointer").into(),
                    );
                }
            };
        if let Some(count) = count
            && check.enabled()
            && (count < 0 || index as u128 >= count as u128)
        {
            return Err(Error::OutOfBounds {
                index,
                length: usize::try_from(count.max(0)).unwrap_or(usize::MAX),
            }
            .into());
        }
        vm.indexed_sequence_pointer(&pointer, index)?
    };
    if pointer.pointee() != ty {
        return Err(Error::TypeMismatch {
            expected: ty,
        }
        .into());
    }
    Ok(pointer)
}

fn number<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
    vm: &mut Vm<'_, P, E>,
    value: Value,
) -> Result<Number> {
    let cells = value.cells(vm.limits.value_cells)?;
    // Number::number clones address provenance. Admit and charge before that clone.
    vm.charge_work(cells)?;
    Ok(value.number()?)
}

fn pointer<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
    vm: &mut Vm<'_, P, E>,
    value: Value,
) -> Result<Pointer> {
    let Value::Pointer(pointer) = value else {
        return Err(Error::InvalidIr("expected pointer value").into());
    };
    vm.charge_work(pointer.metadata_cells())?;
    Ok(pointer)
}

fn value_operand(operands: &mut std::vec::IntoIter<Operand>) -> Result<Value> {
    match operands.next() {
        Some(Operand::Value(value)) => Ok(value),
        _ => Err(Error::InvalidIr("operation requires a captured value operand").into()),
    }
}
fn place_operand(operands: &mut std::vec::IntoIter<Operand>) -> Result<Pointer> {
    match operands.next() {
        Some(Operand::Place(pointer)) => Ok(pointer),
        _ => Err(Error::InvalidIr("operation requires a captured place operand").into()),
    }
}
fn checked<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
    vm: &Vm<'_, P, E>,
    value: Value,
    ty: TypeId,
) -> Result<Value> {
    value.cells(vm.limits.value_cells)?;
    value.validate(vm.provider.types(), ty, vm.limits.evaluation_depth.min(256))?;
    Ok(value)
}

fn admit_wrapper<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
    vm: &mut Vm<'_, P, E>,
    child: &Value,
) -> Result<()> {
    child
        .cells(vm.limits.value_cells)?
        .checked_add(1)
        .filter(|cells| *cells <= vm.limits.value_cells)
        .ok_or(Error::Limit(LimitKind::ValueCells))?;
    vm.charge_work(1)
}

fn compare_strings<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
    vm: &mut Vm<'_, P, E>,
    relation: Equality,
    left: Value,
    right: Value,
    depth: usize,
) -> Result<bool> {
    let length = |value: &Value| match value {
        Value::String(bytes) => i64::try_from(bytes.len()).map_err(|_| Error::CheckedCast),
        Value::StringView {
            count, ..
        } => Ok(*count),
        _ => Err(Error::InvalidIr(
            "string comparison requires string descriptors",
        )),
    };
    let count = length(&left)?;
    let mut equal = count == length(&right)?;
    if equal {
        let count = crate::checked_sequence_count(count)?;
        if count != 0 {
            for value in [&left, &right] {
                if let Value::StringView {
                    pointer, ..
                } = value
                {
                    vm.prepare_pointer_layouts(pointer, true)?;
                    vm.memory
                        .validate_slice(vm.provider.types(), pointer, count)?;
                }
            }
        }
        for index in 0..count {
            vm.step(depth + 1)?;
            let byte = |vm: &mut Vm<'_, P, E>, value: &Value| -> Result<u8> {
                match value {
                    Value::String(bytes) => bytes
                        .get(index)
                        .copied()
                        .ok_or_else(|| Error::InvalidIr("string byte is outside its count").into()),
                    Value::StringView {
                        pointer, ..
                    } => {
                        vm.prepare_pointer_layouts(pointer, true)?;
                        let pointer = vm.memory.offset(
                            vm.provider.types(),
                            pointer,
                            isize::try_from(index).map_err(|_| Error::CheckedCast)?,
                        )?;
                        let value = vm.load_sequence_value(&pointer)?;
                        let number = number(vm, value)?;
                        let integer = number.portable_integer()?;
                        if integer.ty() != jai_types::IntegerType::U8 {
                            return Err(
                                Error::InvalidIr("string comparison has non-byte backing").into()
                            );
                        }
                        Ok(u8::try_from(integer.value()).map_err(|_| Error::CheckedCast)?)
                    }
                    _ => Err(
                        Error::InvalidIr("string comparison requires string descriptors").into(),
                    ),
                }
            };
            if byte(vm, &left)? != byte(vm, &right)? {
                equal = false;
                break;
            }
        }
    }
    Ok(if relation == Equality::Equal {
        equal
    } else {
        !equal
    })
}

fn sequence_parts<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
    vm: &mut Vm<'_, P, E>,
    operand: Operand,
    base_ty: TypeId,
    from_place: bool,
    static_backing: bool,
    depth: usize,
) -> Result<(Value, Option<Pointer>)> {
    let (value, mut storage) = if from_place {
        let Operand::Place(pointer) = operand else {
            return Err(Error::InvalidIr("sequence operation requires captured storage").into());
        };
        (vm.load_sequence_value(&pointer)?, Some(pointer))
    } else {
        let Operand::Value(value) = operand else {
            return Err(Error::InvalidIr("sequence operation requires captured value").into());
        };
        (value, None)
    };
    if matches!(value.semantic(), Value::Array { elements, .. } if !elements.is_empty())
        && storage.is_none()
    {
        vm.charge_work(value.cells(vm.limits.value_cells)?)?;
        storage = Some(if static_backing {
            vm.immutable_backing(base_ty, value.clone(), depth + 1)?
        } else {
            vm.temporary(base_ty, value.clone(), depth + 1)?
        });
    }
    let value = match value {
        Value::String(bytes) => vm.string_descriptor(bytes, depth + 1)?,
        value => value,
    };
    Ok((value, storage))
}
fn array_data<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
    vm: &mut Vm<'_, P, E>,
    ty: TypeId,
    empty: bool,
    storage: Option<Pointer>,
) -> Result<Pointer> {
    if empty {
        let TypeKind::FixedArray {
            element, ..
        } = vm.provider.types().kind(ty)?
        else {
            return Err(Error::InvalidIr("array value has incorrect type").into());
        };
        Ok(Pointer::null(*element))
    } else {
        let pointer = storage.ok_or(Error::InvalidIr("array has no backing storage"))?;
        vm.prepare_pointer_layouts(&pointer, true)?;
        Ok(vm.memory.sequence_data(vm.provider.types(), &pointer)?)
    }
}

fn sequence_index<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
    vm: &mut Vm<'_, P, E>,
    base: Value,
    index: Value,
    ty: TypeId,
    check: CheckMode,
) -> Result<Value> {
    let index = usize::try_from(number(vm, index)?.portable_integer()?.value()).map_err(|_| {
        Error::OutOfBounds {
            index: usize::MAX,
            length: 0,
        }
    })?;
    let value = match base {
        Value::StoredAggregate(snapshot) => {
            let element = snapshot.element_type(vm.provider.types(), index)?;
            vm.prepare_layout(element)?;
            vm.charge_work(
                snapshot
                    .storage_cells()
                    .checked_add(
                        vm.memory
                            .prepared_decoded_cells(vm.provider.types(), element)?,
                    )
                    .ok_or(Error::Limit(LimitKind::Fuel))?,
            )?;
            snapshot.index(vm.provider.types(), index, vm.limits.value_cells)?
        }
        Value::Array {
            elements, ..
        } => {
            let length = elements.len();
            elements.into_iter().nth(index).ok_or(Error::OutOfBounds {
                index,
                length,
            })?
        }
        Value::String(bytes) => {
            let byte = *bytes.get(index).ok_or(Error::OutOfBounds {
                index,
                length: bytes.len(),
            })?;
            Value::Int(Integer::wrapping(
                jai_types::IntegerType::U8,
                i128::from(byte),
            ))
        }
        Value::StringView {
            pointer,
            count,
        }
        | Value::Slice {
            pointer,
            count,
            ..
        }
        | Value::DynamicArray {
            pointer,
            count,
            ..
        } => {
            if check.enabled() && i64::try_from(index).map_or(true, |index| index >= count) {
                return Err(Error::OutOfBounds {
                    index,
                    length: usize::try_from(count).unwrap_or(0),
                }
                .into());
            }
            vm.charge_work(pointer.metadata_cells())?;
            let pointer = vm.indexed_sequence_pointer(&pointer, index)?;
            vm.load_sequence_value(&pointer)?
        }
        Value::Pointer(pointer) => {
            vm.charge_work(pointer.metadata_cells())?;
            vm.prepare_pointer_layouts(&pointer, true)?;
            let pointer = vm.memory.offset(
                vm.provider.types(),
                &pointer,
                isize::try_from(index).map_err(|_| Error::OutOfBounds {
                    index,
                    length: 0,
                })?,
            )?;
            vm.load_sequence_value(&pointer)?
        }
        _ => return Err(Error::InvalidIr("index requires sequence or pointer").into()),
    };
    checked(vm, value, ty)
}

fn field<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
    vm: &mut Vm<'_, P, E>,
    base: Value,
    field: jai_types::FieldId,
    ty: TypeId,
    depth: usize,
) -> Result<Value> {
    let base_ty = match &base {
        Value::StoredAggregate(snapshot) => snapshot.ty(),
        Value::Record {
            ty, ..
        }
        | Value::Union {
            ty, ..
        } => *ty,
        _ => return Err(Error::InvalidIr("field access requires a record").into()),
    };
    if vm.provider.types().validate_field(base_ty, field)? != ty {
        return Err(Error::TypeMismatch {
            expected: ty,
        }
        .into());
    }
    let value = match base {
        Value::StoredAggregate(snapshot) => {
            vm.charge_work(snapshot.storage_cells())?;
            snapshot.field(vm.provider.types(), field.index(), vm.limits.value_cells)?
        }
        Value::Record {
            fields, ..
        } => {
            let length = fields.len();
            fields
                .into_iter()
                .nth(field.index())
                .ok_or(Error::OutOfBounds {
                    index: field.index(),
                    length,
                })?
        }
        Value::Union {
            field: active,
            value,
            ..
        } if active == field.index() => *value,
        value @ Value::Union {
            ..
        } => {
            let value = vm.normalize_storage_value(value, depth + 1)?;
            vm.charge_work(value.cells(vm.limits.value_cells)?)?;
            vm.memory
                .union_value_field(vm.provider.types(), base_ty, &value, field.index())?
        }
        _ => return Err(Error::InvalidIr("field access requires record storage").into()),
    };
    checked(vm, value, ty)
}

fn sequence_build<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
    vm: &mut Vm<'_, P, E>,
    ty: TypeId,
    fields: &[SequenceField],
    operands: &mut std::vec::IntoIter<Operand>,
) -> Result<Value> {
    let element = match vm.provider.types().kind(ty)? {
        TypeKind::String => vm
            .provider
            .types()
            .scalar(jai_types::ScalarType::Int(jai_types::IntegerType::U8)),
        TypeKind::Slice(element) | TypeKind::DynamicArray(element) => *element,
        _ => return Err(Error::UnsupportedType(ty).into()),
    };
    let (mut count, mut allocated, mut pointer) = (0, 0, Pointer::null(element));
    let allocator = vm.default_sequence_allocator(ty)?;
    let mut seen = std::collections::HashSet::new();
    for field in fields {
        if !seen.insert(*field) {
            return Err(Error::InvalidIr("duplicate sequence field initializer").into());
        }
        let value = value_operand(operands)?;
        match field {
            SequenceField::Data => pointer = self::pointer(vm, value)?,
            SequenceField::Count => {
                count = i64::try_from(number(vm, value)?.portable_integer()?.value())
                    .map_err(|_| Error::CheckedCast)?
            }
            SequenceField::Allocated => {
                allocated = i64::try_from(number(vm, value)?.portable_integer()?.value())
                    .map_err(|_| Error::CheckedCast)?
            }
        }
    }
    let value = match vm.provider.types().kind(ty)? {
        TypeKind::String => Value::StringView {
            pointer,
            count,
        },
        TypeKind::Slice(_) => Value::Slice {
            ty,
            pointer,
            count,
        },
        TypeKind::DynamicArray(_) => Value::DynamicArray {
            ty,
            pointer,
            count,
            allocated,
            allocator,
        },
        _ => return Err(Error::UnsupportedType(ty).into()),
    };
    checked(vm, value, ty)
}

pub(super) fn apply<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
    vm: &mut Vm<'_, P, E>,
    op: &ApplyOp,
    operands: Vec<Operand>,
    depth: usize,
) -> Result<Operand> {
    let mut operands = operands.into_iter();
    let value = match op {
        ApplyOp::Bound(binding) => vm.bound_value(*binding, 0)?,
        ApplyOp::Literal(value) => {
            vm.charge_work(value.cells(vm.limits.value_cells)?)?;
            value.clone()
        }
        ApplyOp::StringBytes {
            ty,
            bytes,
        } => {
            if bytes.len() >= vm.limits.value_cells {
                return Err(Error::Limit(LimitKind::ValueCells).into());
            }
            vm.charge_work(bytes.len())?;
            checked(vm, Value::String(bytes.to_vec()), *ty)?
        }
        ApplyOp::NativePointer(value) => {
            crate::constants::native_pointer(vm.provider.types(), value, vm.memory.target())?
        }
        ApplyOp::RuntimeType(value) => vm.runtime_type_constant(value, depth + 1)?,
        ApplyOp::StaticAddress {
            data,
            address,
        } => Value::Pointer(vm.static_address(data, address, depth + 1)?),
        ApplyOp::Context(ty) => vm.context_value(*ty)?,
        ApplyOp::Zero(ty) => vm.zero_value(*ty)?,
        ApplyOp::Load => vm.load_sequence_value(&place_operand(&mut operands)?)?,
        ApplyOp::StorageBitcast {
            cast,
            from_place,
        } => {
            if *from_place {
                vm.storage_bitcast_place(&place_operand(&mut operands)?, *cast)?
            } else {
                vm.storage_bitcast_value(value_operand(&mut operands)?, *cast, depth + 1)?
            }
        }
        ApplyOp::Int {
            ty,
            op,
        } => {
            let result = match op {
                IntApply::FromValue => number(vm, value_operand(&mut operands)?)?,
                IntApply::EnumValue => match value_operand(&mut operands)? {
                    Value::Enum {
                        value, ..
                    } => Number::plain(value),
                    _ => {
                        return Err(
                            Error::InvalidIr("enum conversion requires an enum value").into()
                        );
                    }
                },
                IntApply::PointerDifference => {
                    let left = pointer(vm, value_operand(&mut operands)?)?;
                    let right = pointer(vm, value_operand(&mut operands)?)?;
                    vm.prepare_pointer_layouts(&left, true)?;
                    vm.prepare_pointer_layouts(&right, true)?;
                    Number::plain(Integer::wrapping(
                        *ty,
                        i128::from(vm.memory.distance(vm.provider.types(), &left, &right)?),
                    ))
                }
                IntApply::FromFloat(mode) => Number::plain(crate::floats::to_integer(
                    value_operand(&mut operands)?.float()?,
                    *ty,
                    *mode,
                )?),
                IntApply::FromPointer(mode) => {
                    let pointer = pointer(vm, value_operand(&mut operands)?)?;
                    vm.prepare_pointer_layouts(&pointer, false)?;
                    vm.memory
                        .pointer_to_integer(vm.provider.types(), &pointer, *ty, *mode)?
                }
                IntApply::FromBool => Number::plain(Integer::wrapping(
                    *ty,
                    i128::from(value_operand(&mut operands)?.boolean()?),
                )),
                IntApply::Cast(mode) => {
                    let source = number(vm, value_operand(&mut operands)?)?;
                    vm.number_cast(*ty, source, *mode)?
                }
                IntApply::InvalidCheckedCast => return Err(Error::CheckedCast.into()),
                IntApply::Negate(check) => {
                    let source = number(vm, value_operand(&mut operands)?)?;
                    scalar::negate_number(*ty, source, *check)?
                }
                IntApply::Complement => {
                    let source = number(vm, value_operand(&mut operands)?)?;
                    scalar::complement_number(*ty, source)?
                }
                IntApply::Binary(op, check) => {
                    let left = number(vm, value_operand(&mut operands)?)?;
                    let right = number(vm, value_operand(&mut operands)?)?;
                    vm.number_binary(*ty, *op, left, right, *check)?
                }
            };
            if result.ty() != *ty {
                return Err(Error::InvalidIr("integer expression result type differs").into());
            }
            result.into_value()
        }
        ApplyOp::Bool(op) => Value::Bool(match op {
            BoolApply::CompileTime => vm.execution_phase.is_compile_time(),
            BoolApply::FromValue => value_operand(&mut operands)?.boolean()?,
            BoolApply::FromInt => {
                let value = number(vm, value_operand(&mut operands)?)?;
                vm.number_truth(&value)?
            }
            BoolApply::FromPointer => match value_operand(&mut operands)? {
                Value::Pointer(pointer) => vm.pointer_truth(&pointer)?,
                Value::Procedure {
                    procedure, ..
                } => procedure.is_some(),
                _ => {
                    return Err(
                        Error::InvalidIr("pointer truth requires a pointer or procedure").into(),
                    );
                }
            },
            BoolApply::Not => !value_operand(&mut operands)?.boolean()?,
            BoolApply::CompareInts(op) => {
                let left = number(vm, value_operand(&mut operands)?)?;
                let right = number(vm, value_operand(&mut operands)?)?;
                vm.compare_numbers(*op, &left, &right)?
            }
            BoolApply::CompareFloats(op) => crate::floats::compare(
                *op,
                value_operand(&mut operands)?.float()?,
                value_operand(&mut operands)?.float()?,
            )?,
            BoolApply::ComparePointers(op) => {
                let left = value_operand(&mut operands)?;
                let right = value_operand(&mut operands)?;
                let equal = match (left, right) {
                    (Value::Pointer(left), Value::Pointer(right)) => {
                        vm.prepare_pointer_layouts(&left, false)?;
                        vm.prepare_pointer_layouts(&right, false)?;
                        vm.charge_work(
                            left.metadata_cells().saturating_add(right.metadata_cells()),
                        )?;
                        vm.memory.same_address(vm.provider.types(), &left, &right)?
                    }
                    (
                        Value::Procedure {
                            signature: ls,
                            procedure: l,
                        },
                        Value::Procedure {
                            signature: rs,
                            procedure: r,
                        },
                    ) if ls == rs => l == r,
                    _ => {
                        return Err(Error::InvalidIr(
                            "pointer comparison requires one exact pointer or procedure type",
                        )
                        .into());
                    }
                };
                if *op == Equality::Equal {
                    equal
                } else {
                    !equal
                }
            }
            BoolApply::CompareStrings(op) => compare_strings(
                vm,
                *op,
                value_operand(&mut operands)?,
                value_operand(&mut operands)?,
                depth + 1,
            )?,
            BoolApply::CompareBools(op) => {
                let equal = value_operand(&mut operands)?.boolean()?
                    == value_operand(&mut operands)?.boolean()?;
                if *op == Equality::Equal {
                    equal
                } else {
                    !equal
                }
            }
        }),
        ApplyOp::Float {
            ty,
            op,
        } => {
            let result = match op {
                FloatApply::FromValue => value_operand(&mut operands)?.float()?,
                FloatApply::Negate => value_operand(&mut operands)?.float()?.negate(),
                FloatApply::Binary(op) => crate::floats::binary(
                    *ty,
                    *op,
                    value_operand(&mut operands)?.float()?,
                    value_operand(&mut operands)?.float()?,
                )?,
                FloatApply::Cast => value_operand(&mut operands)?.float()?.convert(*ty),
                FloatApply::FromInt => {
                    let source = number(vm, value_operand(&mut operands)?)?;
                    if source.provenance().is_some() {
                        return Err(Error::UnsupportedPointerOperation(
                            "address-derived integer cannot convert to a float",
                        )
                        .into());
                    }
                    jai_types::FloatValue::from_integer(*ty, source.integer())
                }
            };
            if result.ty() != *ty {
                return Err(
                    Error::InvalidIr("floating-point expression result type differs").into(),
                );
            }
            Value::Float(result)
        }
        ApplyOp::TypeDescriptor(ty) => {
            let value = value_operand(&mut operands)?;
            let TypeKind::Pointer(header) = vm.provider.types().kind(*ty)? else {
                return Err(Error::InvalidIr("Type descriptor requires a pointer result").into());
            };
            let Value::Type {
                descriptor,
            } = &value
            else {
                return Err(Error::InvalidIr("Type descriptor requires a runtime Type").into());
            };
            let pointer = if let Some(pointer) = descriptor {
                vm.runtime_type_identity(&value)?;
                vm.charge_work(pointer.metadata_cells())?;
                pointer.clone()
            } else {
                Pointer::null(*header)
            };
            Value::Pointer(pointer)
        }
        ApplyOp::Array(ty) | ApplyOp::Record(ty) => {
            if operands.len() > vm.limits.value_cells {
                return Err(Error::Limit(LimitKind::ValueCells).into());
            }
            let mut values = Vec::with_capacity(operands.len());
            let mut cells = 1;
            while operands.len() != 0 {
                vm.push_value(&mut values, &mut cells, value_operand(&mut operands)?)?;
            }
            checked(
                vm,
                if matches!(op, ApplyOp::Array(_)) {
                    Value::Array {
                        ty: *ty,
                        elements: values,
                    }
                } else {
                    Value::Record {
                        ty: *ty,
                        fields: values,
                    }
                },
                *ty,
            )?
        }
        ApplyOp::Union {
            ty,
            field,
        } => {
            vm.provider.types().validate_field(*ty, *field)?;
            let child = value_operand(&mut operands)?;
            admit_wrapper(vm, &child)?;
            checked(
                vm,
                Value::Union {
                    ty: *ty,
                    field: field.index(),
                    value: Box::new(child),
                },
                *ty,
            )?
        }
        ApplyOp::Field {
            ty,
            field: field_id,
        } => field(vm, value_operand(&mut operands)?, *field_id, *ty, depth + 1)?,
        ApplyOp::SequenceBuild {
            ty,
            fields,
        } => sequence_build(vm, *ty, fields, &mut operands)?,
        ApplyOp::SequenceIndex {
            ty,
            check,
        } => sequence_index(
            vm,
            value_operand(&mut operands)?,
            value_operand(&mut operands)?,
            *ty,
            *check,
        )?,
        ApplyOp::SequenceField {
            base_type,
            field,
            from_place,
            static_backing,
        } => {
            let operand = operands
                .next()
                .ok_or(Error::InvalidIr("missing sequence operand"))?;
            sequence_field(
                vm,
                operand,
                *base_type,
                *field,
                *from_place,
                *static_backing,
                depth + 1,
            )?
        }
        ApplyOp::ArrayView {
            base_type,
            ty,
            from_place,
            static_backing,
        } => {
            let operand = operands
                .next()
                .ok_or(Error::InvalidIr("missing array operand"))?;
            array_view(
                vm,
                operand,
                *base_type,
                *ty,
                *from_place,
                *static_backing,
                depth + 1,
            )?
        }
        ApplyOp::SequenceView {
            ty,
            from_place,
        } => {
            let value = if *from_place {
                vm.load_sequence_value(&place_operand(&mut operands)?)?
            } else {
                value_operand(&mut operands)?
            };
            let value = match value {
                Value::String(bytes) => vm.string_descriptor(bytes, depth + 1)?,
                value => value,
            };
            let (pointer, count) = match value {
                Value::Slice {
                    pointer,
                    count,
                    ..
                }
                | Value::DynamicArray {
                    pointer,
                    count,
                    ..
                }
                | Value::StringView {
                    pointer,
                    count,
                } => (pointer, count),
                _ => {
                    return Err(
                        Error::InvalidIr("sequence view requires string or descriptor").into(),
                    );
                }
            };
            checked(
                vm,
                Value::Slice {
                    ty: *ty,
                    pointer,
                    count,
                },
                *ty,
            )?
        }
        ApplyOp::AddressOf => Value::Pointer(place_operand(&mut operands)?),
        ApplyOp::AddressOfValue(ty) => {
            let TypeKind::Pointer(pointee) = *vm.provider.types().kind(*ty)? else {
                return Err(
                    Error::InvalidIr("materialized address requires a pointer type").into(),
                );
            };
            let value = checked(vm, value_operand(&mut operands)?, pointee)?;
            Value::Pointer(vm.temporary(pointee, value, depth + 1)?)
        }
        ApplyOp::PointerCast {
            ty,
            mode,
        } => {
            let pointer = pointer(vm, value_operand(&mut operands)?)?;
            vm.prepare_pointer_layouts(&pointer, false)?;
            let TypeKind::Pointer(pointee) = vm.provider.types().kind(*ty)? else {
                return Err(Error::InvalidIr("pointer cast target is not pointer type").into());
            };
            Value::Pointer(vm.memory.cast_pointer(
                vm.provider.types(),
                &pointer,
                *pointee,
                *mode,
            )?)
        }
        ApplyOp::PointerOffset {
            ty,
            subtract,
            integer_first,
        } => {
            let left = value_operand(&mut operands)?;
            let right = value_operand(&mut operands)?;
            let (pointer_value, offset_value) = if *integer_first {
                (right, left)
            } else {
                (left, right)
            };
            let pointer = pointer(vm, pointer_value)?;
            let offset = number(vm, offset_value)?.portable_integer()?.value();
            let offset = if *subtract {
                offset.checked_neg().ok_or(Error::CheckedCast)?
            } else {
                offset
            };
            let offset = isize::try_from(offset).map_err(|_| {
                if *integer_first {
                    Error::CheckedCast
                } else {
                    Error::OutOfBounds {
                        index: usize::MAX,
                        length: 0,
                    }
                }
            })?;
            vm.prepare_pointer_layouts(&pointer, true)?;
            checked(
                vm,
                Value::Pointer(vm.memory.offset(vm.provider.types(), &pointer, offset)?),
                *ty,
            )?
        }
        ApplyOp::PointerFromInteger {
            ty,
            mode,
        } => {
            let source = number(vm, value_operand(&mut operands)?)?;
            vm.prepare_number_address(&source)?;
            let TypeKind::Pointer(pointee) = vm.provider.types().kind(*ty)? else {
                return Err(
                    Error::InvalidIr("integer pointer cast target is not pointer type").into(),
                );
            };
            Value::Pointer(vm.memory.integer_to_pointer(
                vm.provider.types(),
                source,
                *pointee,
                *mode,
            )?)
        }
        ApplyOp::Distinct(ty) => {
            let child = value_operand(&mut operands)?;
            admit_wrapper(vm, &child)?;
            checked(
                vm,
                Value::Distinct {
                    ty: *ty,
                    value: Box::new(child),
                },
                *ty,
            )?
        }
        ApplyOp::UnwrapDistinct(ty) => {
            let value = match value_operand(&mut operands)? {
                Value::StoredAggregate(snapshot) => {
                    vm.charge_work(snapshot.storage_cells())?;
                    snapshot.representation(vm.provider.types(), vm.limits.value_cells)?
                }
                Value::Distinct {
                    value, ..
                } => *value,
                _ => return Err(Error::InvalidIr("distinct unwrap requires distinct value").into()),
            };
            checked(vm, value, *ty)?
        }
        ApplyOp::EnumFromInt(ty) => {
            let source = number(vm, value_operand(&mut operands)?)?;
            if source.provenance().is_some() {
                return Err(Error::UnsupportedPointerOperation(
                    "address-derived integer cannot convert to an enum",
                )
                .into());
            }
            checked(
                vm,
                Value::Enum {
                    ty: *ty,
                    value: source.integer(),
                },
                *ty,
            )?
        }
    };
    if operands.next().is_some() {
        return Err(Error::InvalidIr("operation received extra operands").into());
    }
    value.cells(vm.limits.value_cells)?;
    Ok(Operand::Value(value))
}

fn sequence_field<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
    vm: &mut Vm<'_, P, E>,
    operand: Operand,
    base_ty: TypeId,
    field: SequenceField,
    from_place: bool,
    static_backing: bool,
    depth: usize,
) -> Result<Value> {
    if from_place
        && let TypeKind::FixedArray {
            count, ..
        } = *vm.provider.types().kind(base_ty)?
    {
        let Operand::Place(storage) = operand else {
            return Err(Error::InvalidIr("array field requires captured storage").into());
        };
        return Ok(match field {
            SequenceField::Count => Value::Int(
                Integer::checked(jai_types::IntegerType::S64, i128::from(count))
                    .ok_or(Error::CheckedCast)?,
            ),
            SequenceField::Data => {
                Value::Pointer(array_data(vm, base_ty, count == 0, Some(storage))?)
            }
            SequenceField::Allocated => {
                return Err(Error::InvalidIr("allocated field requires dynamic array").into());
            }
        });
    }
    let (value, storage) =
        sequence_parts(vm, operand, base_ty, from_place, static_backing, depth + 1)?;
    let (count, allocated, pointer) = match &value {
        Value::StoredAggregate(snapshot) => {
            let jai_types::TypeKind::FixedArray {
                count, ..
            } = vm.provider.types().kind(snapshot.ty())?
            else {
                return Err(
                    Error::InvalidIr("sequence field requires array, string or slice").into(),
                );
            };
            (
                i64::try_from(*count).map_err(|_| Error::CheckedCast)?,
                None,
                array_data(vm, snapshot.ty(), *count == 0, storage)?,
            )
        }
        Value::Array {
            ty,
            elements,
        } => (
            i64::try_from(elements.len()).map_err(|_| Error::CheckedCast)?,
            None,
            array_data(vm, *ty, elements.is_empty(), storage)?,
        ),
        Value::StringView {
            pointer,
            count,
        }
        | Value::Slice {
            pointer,
            count,
            ..
        } => {
            vm.charge_work(pointer.metadata_cells())?;
            (*count, None, pointer.clone())
        }
        Value::DynamicArray {
            pointer,
            count,
            allocated,
            ..
        } => {
            vm.charge_work(pointer.metadata_cells())?;
            (*count, Some(*allocated), pointer.clone())
        }
        _ => return Err(Error::InvalidIr("sequence field requires array, string or slice").into()),
    };
    Ok(match field {
        SequenceField::Count => Value::Int(
            Integer::checked(jai_types::IntegerType::S64, i128::from(count))
                .ok_or(Error::CheckedCast)?,
        ),
        SequenceField::Data => Value::Pointer(pointer),
        SequenceField::Allocated => Value::Int(
            Integer::checked(
                jai_types::IntegerType::S64,
                i128::from(
                    allocated.ok_or(Error::InvalidIr("allocated field requires dynamic array"))?,
                ),
            )
            .ok_or(Error::CheckedCast)?,
        ),
    })
}

fn array_view<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
    vm: &mut Vm<'_, P, E>,
    operand: Operand,
    base_ty: TypeId,
    ty: TypeId,
    from_place: bool,
    static_backing: bool,
    depth: usize,
) -> Result<Value> {
    if from_place
        && let TypeKind::FixedArray {
            count, ..
        } = *vm.provider.types().kind(base_ty)?
    {
        let Operand::Place(storage) = operand else {
            return Err(Error::InvalidIr("array view requires captured storage").into());
        };
        let pointer = array_data(vm, base_ty, count == 0, Some(storage))?;
        return checked(
            vm,
            Value::Slice {
                ty,
                pointer,
                count: i64::try_from(count).map_err(|_| Error::CheckedCast)?,
            },
            ty,
        );
    }
    let (value, storage) =
        sequence_parts(vm, operand, base_ty, from_place, static_backing, depth + 1)?;
    let Value::Array {
        ty: array_ty,
        elements,
    } = value.semantic()
    else {
        return Err(Error::InvalidIr("array view requires fixed array").into());
    };
    let pointer = array_data(vm, *array_ty, elements.is_empty(), storage)?;
    checked(
        vm,
        Value::Slice {
            ty,
            pointer,
            count: i64::try_from(elements.len()).map_err(|_| Error::CheckedCast)?,
        },
        ty,
    )
}

pub(super) fn apply_place<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
    vm: &mut Vm<'_, P, E>,
    op: &PlaceOp,
    operands: Vec<Operand>,
    depth: usize,
) -> Result<Operand> {
    let mut operands = operands.into_iter();
    let pointer = match op {
        PlaceOp::Context(ty) => vm.context_pointer(*ty)?,
        PlaceOp::Local(id) => {
            let frame = vm.frame()?;
            if id.procedure() != frame.procedure.id {
                return Err(Error::InvalidIr("local belongs to another procedure").into());
            }
            let cells = frame
                .slots
                .get(id.index())
                .ok_or(Error::InvalidIr("missing local storage"))?
                .metadata_cells();
            vm.charge_work(cells)?;
            vm.frame()?.slots[id.index()].clone()
        }
        PlaceOp::Global(id) => {
            if vm.provider.global_alignment_pending(*id) {
                return Err(Halt::Pending(Dependency::GlobalAlignment(*id)));
            }
            if matches!(
                vm.provider
                    .globals()
                    .get(id.index())
                    .map(jai_ir::Global::initializer),
                Some(jai_ir::GlobalInitializer::External(_))
            ) {
                return Err(Error::UnsupportedExternalGlobal(*id).into());
            }
            let slot = vm
                .globals
                .get(id.index())
                .ok_or(Error::InvalidIr("missing global storage"))?;
            if let Some(pointer) = slot {
                let cells = pointer.metadata_cells();
                vm.charge_work(cells)?;
                vm.globals[id.index()]
                    .as_ref()
                    .ok_or(Error::InvalidIr("missing global storage"))?
                    .clone()
            } else {
                let global = vm
                    .provider
                    .globals()
                    .get(id.index())
                    .ok_or(Error::InvalidIr("missing global definition"))?;
                let value = vm.global_value(global.initializer(), depth + 1)?;
                let value = vm.normalize_storage_value(value, depth + 1)?;
                vm.prepare_layout(global.ty())?;
                let pointer = vm.memory.allocate_with_alignment(
                    vm.provider.types(),
                    global.ty(),
                    Some(value),
                    vm.provider
                        .storage_alignments()
                        .and_then(|alignments| alignments.global(global.id()))
                        .unwrap_or(1),
                )?;
                vm.charge_work(pointer.metadata_cells())?;
                vm.globals[id.index()] = Some(pointer.clone());
                pointer
            }
        }
        PlaceOp::Field(field) => {
            let base = place_operand(&mut operands)?;
            vm.provider.types().validate_field(base.pointee(), *field)?;
            vm.prepare_field_layouts(&base, field.index())?;
            vm.charge_work(base.metadata_cells())?;
            vm.memory.field(vm.provider.types(), &base, field.index())?
        }
        PlaceOp::Dereference => pointer(vm, value_operand(&mut operands)?)?,
        PlaceOp::SequenceField(field) => {
            let base = place_operand(&mut operands)?;
            vm.ensure_string_storage(&base, depth + 1)?;
            vm.prepare_pointer_layouts(&base, true)?;
            vm.prepare_layout(match field {
                jai_ir::SequenceField::Data => vm
                    .provider
                    .types()
                    .lookup(&TypeKind::Pointer(
                        match vm.provider.types().kind(base.pointee())? {
                            TypeKind::String => vm
                                .provider
                                .types()
                                .scalar(jai_types::ScalarType::Int(jai_types::IntegerType::U8)),
                            TypeKind::Slice(element) | TypeKind::DynamicArray(element) => *element,
                            _ => {
                                return Err(
                                    Error::InvalidIr("sequence field requires descriptor").into()
                                );
                            }
                        },
                    ))
                    .ok_or(Error::UnsupportedType(base.pointee()))?,
                _ => vm
                    .provider
                    .types()
                    .scalar(jai_types::ScalarType::Int(jai_types::IntegerType::S64)),
            })?;
            vm.charge_work(base.metadata_cells())?;
            vm.memory
                .sequence_field(vm.provider.types(), &base, *field)?
        }
    };
    if operands.next().is_some() {
        return Err(Error::InvalidIr("place operation received extra operands").into());
    }
    Ok(Operand::Place(pointer))
}
