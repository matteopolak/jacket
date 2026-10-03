use super::*;

impl<P: ProcedureProvider + ?Sized, E: CompilerEffects> Vm<'_, P, E> {
    pub(super) fn value(&mut self, expression: &ValueExpr, depth: usize) -> Result<Value> {
        self.step(depth)?;
        let value = match expression {
            ValueExpr::StorageBitcast {
                source,
                cast,
            } => match source {
                StorageBitcastSource::Place(place) => {
                    let source = self.place(*place, depth + 1)?;
                    self.storage_bitcast_place(&source, *cast)?
                }
                StorageBitcastSource::Value(value) => {
                    let source = self.value(value, depth + 1)?;
                    self.storage_bitcast_value(source, *cast, depth + 1)?
                }
            },
            ValueExpr::Bind {
                bindings,
                body,
                ty,
            } => {
                let scope = self.begin_bindings(0)?;
                let result: Result<Value> = (|| {
                    for (binding, producer) in bindings {
                        let value = self.value(producer, depth + 1)?;
                        self.capture_binding(*binding, value, 0)?;
                    }
                    let value = self.value(body, depth + 1)?;
                    value.validate(
                        self.provider.types(),
                        *ty,
                        self.limits.evaluation_depth.min(256),
                    )?;
                    Ok(value)
                })();
                self.end_bindings(scope)?;
                result?
            }
            ValueExpr::Bound {
                binding, ..
            } => self.bound_value(*binding, 0)?,
            ValueExpr::NativePointer(value) => crate::constants::native_pointer(
                self.provider.types(),
                value,
                self.memory.target(),
            )?,
            ValueExpr::RuntimeType(value) => self.runtime_type_constant(value, depth + 1)?,
            ValueExpr::TypeDescriptor {
                value,
                ty,
            } => {
                let value = self.value(value, depth + 1)?;
                let TypeKind::Pointer(header) = self.provider.types().kind(*ty)? else {
                    return Err(
                        Error::InvalidIr("Type descriptor requires a pointer result").into(),
                    );
                };
                let Value::Type {
                    descriptor,
                } = &value
                else {
                    return Err(Error::InvalidIr("Type descriptor requires a runtime Type").into());
                };
                let pointer = if let Some(pointer) = descriptor {
                    self.runtime_type_identity(&value)?;
                    pointer.clone()
                } else {
                    Pointer::null(*header)
                };
                Value::Pointer(pointer)
            }
            ValueExpr::Context {
                ty,
            } => self.context_value(*ty)?,
            ValueExpr::StaticAddress {
                data,
                address,
                ty,
            } => {
                let pointer = self.static_address(data, address, depth + 1)?;
                let value = Value::Pointer(pointer);
                value.validate(
                    self.provider.types(),
                    *ty,
                    self.limits.evaluation_depth.min(256),
                )?;
                value
            }
            ValueExpr::Conditional {
                ty,
                expression,
            } => {
                let value = if self.boolean(&expression.condition, depth + 1)? {
                    self.value(&expression.then_value, depth + 1)?
                } else {
                    self.value(&expression.else_value, depth + 1)?
                };
                value.validate(
                    self.provider.types(),
                    *ty,
                    self.limits.evaluation_depth.min(256),
                )?;
                value
            }
            ValueExpr::Union {
                ty,
                field,
                value,
            } => {
                self.provider.types().validate_field(*ty, *field)?;
                let value = Value::Union {
                    ty: *ty,
                    field: field.index(),
                    value: Box::new(self.value(value, depth + 1)?),
                };
                value.validate(
                    self.provider.types(),
                    *ty,
                    self.limits.evaluation_depth.min(256),
                )?;
                value
            }
            ValueExpr::SequenceView {
                sequence,
                ty,
            } => {
                let (value, storage) = self.sequence_parts(sequence, depth + 1)?;
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
                    Value::String(bytes) => (
                        self.memory.sequence_data(
                            self.provider.types(),
                            &storage.ok_or(Error::InvalidIr("string view has no storage"))?,
                        )?,
                        i64::try_from(bytes.len()).map_err(|_| Error::CheckedCast)?,
                    ),
                    _ => {
                        return Err(Error::InvalidIr(
                            "sequence view requires string or descriptor",
                        )
                        .into());
                    }
                };
                let value = Value::Slice {
                    ty: *ty,
                    pointer,
                    count,
                };
                value.validate(
                    self.provider.types(),
                    *ty,
                    self.limits.evaluation_depth.min(256),
                )?;
                value
            }
            ValueExpr::Float(expression) => Value::Float(self.float(expression, depth + 1)?),
            ValueExpr::Array {
                ty,
                elements,
            } => {
                if elements.len() > self.limits.value_cells {
                    return Err(Error::Limit(LimitKind::ValueCells).into());
                }
                let mut values = Vec::with_capacity(elements.len());
                let mut cells = 1;
                for element in elements {
                    let value = self.value(element, depth + 1)?;
                    self.push_value(&mut values, &mut cells, value)?;
                }
                let value = Value::Array {
                    ty: *ty,
                    elements: values,
                };
                value.validate(
                    self.provider.types(),
                    *ty,
                    self.limits.evaluation_depth.min(256),
                )?;
                value
            }
            ValueExpr::StringBytes {
                ty,
                bytes,
            } => {
                if bytes.len() >= self.limits.value_cells {
                    return Err(Error::Limit(LimitKind::ValueCells).into());
                }
                let value = Value::String(bytes.clone());
                value.validate(
                    self.provider.types(),
                    *ty,
                    self.limits.evaluation_depth.min(256),
                )?;
                value
            }
            ValueExpr::SequenceField {
                base,
                field,
                ty,
            } => {
                let value = self.sequence_field(base, *field, depth + 1)?;
                value.validate(
                    self.provider.types(),
                    *ty,
                    self.limits.evaluation_depth.min(256),
                )?;
                value
            }
            ValueExpr::ArrayToSlice {
                array,
                ty,
            } => self.array_view(&ValueExpr::Load(*array), *ty, depth + 1)?,
            ValueExpr::ArrayView {
                array,
                ty,
            } => self.array_view(array, *ty, depth + 1)?,
            ValueExpr::Index {
                base,
                index,
                ty,
                check,
            } => self.sequence_index(base, index, *ty, *check, depth + 1)?,
            ValueExpr::SequenceBuild {
                ty,
                initializers,
            } => self.sequence_build(*ty, initializers, depth + 1)?,
            ValueExpr::AddressOfValue {
                value,
                ty,
            } => self.address_of_value(value, *ty, depth + 1)?,
            ValueExpr::AddressOf {
                place,
                ty,
            } => {
                let pointer = self.place(*place, depth + 1)?;
                let value = Value::Pointer(pointer);
                value.validate(
                    self.provider.types(),
                    *ty,
                    self.limits.evaluation_depth.min(256),
                )?;
                value
            }
            ValueExpr::PointerFromInteger {
                value,
                ty,
                mode,
            } => {
                let number = self.integer(value, depth + 1)?;
                let TypeKind::Pointer(pointee) = self.provider.types().kind(*ty)? else {
                    return Err(Error::InvalidIr(
                        "integer pointer cast target is not pointer type",
                    )
                    .into());
                };
                self.charge_work(number.metadata_cells())?;
                self.prepare_number_address(&number)?;
                Value::Pointer(self.memory.integer_to_pointer(
                    self.provider.types(),
                    number,
                    *pointee,
                    *mode,
                )?)
            }
            ValueExpr::PointerOffsetLeft {
                offset,
                pointer,
                ty,
            } => {
                let offset = self.integer(offset, depth + 1)?.portable_integer()?.value();
                let pointer = self.value(pointer, depth + 1)?.pointer()?.clone();
                self.prepare_pointer_layouts(&pointer, true)?;
                self.charge_work(pointer.metadata_cells())?;
                let offset = isize::try_from(offset).map_err(|_| Error::CheckedCast)?;
                let value = Value::Pointer(self.memory.offset(
                    self.provider.types(),
                    &pointer,
                    offset,
                )?);
                value.validate(
                    self.provider.types(),
                    *ty,
                    self.limits.evaluation_depth.min(256),
                )?;
                value
            }
            ValueExpr::SequenceConcat {
                ty,
                parts,
            } => self.sequence_concat(*ty, parts, depth + 1)?,
            ValueExpr::PointerCast {
                value,
                ty,
                mode,
            } => {
                let value = self.value(value, depth + 1)?;
                let pointer = value.pointer()?;
                self.prepare_pointer_layouts(pointer, false)?;
                self.charge_work(pointer.metadata_cells())?;
                let TypeKind::Pointer(pointee) = self.provider.types().kind(*ty)? else {
                    return Err(Error::InvalidIr("pointer cast target is not pointer type").into());
                };
                Value::Pointer(self.memory.cast_pointer(
                    self.provider.types(),
                    pointer,
                    *pointee,
                    *mode,
                )?)
            }
            ValueExpr::PointerOffset {
                pointer,
                offset,
                subtract,
                ty,
            } => {
                let pointer = self.value(pointer, depth + 1)?.pointer()?.clone();
                let offset = self.integer(offset, depth + 1)?.portable_integer()?.value();
                self.prepare_pointer_layouts(&pointer, true)?;
                self.charge_work(pointer.metadata_cells())?;
                let offset = if *subtract {
                    -offset
                } else {
                    offset
                };
                let offset = isize::try_from(offset).map_err(|_| Error::OutOfBounds {
                    index: usize::MAX,
                    length: 0,
                })?;
                let value = Value::Pointer(self.memory.offset(
                    self.provider.types(),
                    &pointer,
                    offset,
                )?);
                value.validate(
                    self.provider.types(),
                    *ty,
                    self.limits.evaluation_depth.min(256),
                )?;
                value
            }
            ValueExpr::Distinct {
                ty,
                value,
            } => {
                let value = Value::Distinct {
                    ty: *ty,
                    value: Box::new(self.value(value, depth + 1)?),
                };
                value.validate(
                    self.provider.types(),
                    *ty,
                    self.limits.evaluation_depth.min(256),
                )?;
                value
            }
            ValueExpr::UnwrapDistinct {
                value,
                ty,
            } => {
                let value = match self.value(value, depth + 1)? {
                    Value::StoredAggregate(snapshot) => {
                        self.charge_work(snapshot.storage_cells())?;
                        snapshot.representation(self.provider.types(), self.limits.value_cells)?
                    }
                    Value::Distinct {
                        value, ..
                    } => *value,
                    _ => {
                        return Err(
                            Error::InvalidIr("distinct unwrap requires distinct value").into()
                        );
                    }
                };
                value.validate(
                    self.provider.types(),
                    *ty,
                    self.limits.evaluation_depth.min(256),
                )?;
                value
            }
            ValueExpr::ProcedureValue {
                procedure,
                ty,
            } => Value::Procedure {
                signature: *ty,
                procedure: Some(*procedure),
            },
            ValueExpr::IndirectCall {
                callee,
                arguments,
                ty,
                ..
            } => {
                let mut values = self.indirect_call(callee, arguments, depth + 1)?;
                if values.len() != 1 {
                    return Err(Error::InvalidIr("indirect value call requires one result").into());
                }
                let value = values.remove(0);
                value.validate(
                    self.provider.types(),
                    *ty,
                    self.limits.evaluation_depth.min(256),
                )?;
                value
            }
            ValueExpr::Int(expression) => self.integer(expression, depth + 1)?.into_value(),
            ValueExpr::Bool(expression) => Value::Bool(self.boolean(expression, depth + 1)?),
            ValueExpr::Load(place) => self.load(*place, depth + 1)?,
            ValueExpr::Zero(ty) => self.zero_value(*ty)?,
            ValueExpr::Record {
                ty,
                fields,
            } => {
                if fields.len() > self.limits.value_cells {
                    return Err(Error::Limit(LimitKind::ValueCells).into());
                }
                let mut values = Vec::with_capacity(fields.len());
                let mut cells = 1;
                for field in fields {
                    let value = self.value(field, depth + 1)?;
                    self.push_value(&mut values, &mut cells, value)?;
                }
                let value = Value::Record {
                    ty: *ty,
                    fields: values,
                };
                value.validate(
                    self.provider.types(),
                    *ty,
                    self.limits.evaluation_depth.min(256),
                )?;
                value
            }
            ValueExpr::OrderedRecord {
                ty,
                backing,
                initializers,
            } => {
                self.prepare_ordered_record(
                    *ty,
                    initializers.iter().map(|(path, _)| path.as_ref()),
                )?;
                let mut state = self.start_ordered_record(
                    *ty,
                    *backing,
                    initializers.iter().map(|(path, _)| path.as_ref()),
                    0,
                )?;
                for (index, (_, expression)) in initializers.iter().enumerate() {
                    let value = self.ordered_record_initializer(&state, expression, depth + 1)?;
                    self.write_ordered_record(&mut state, index, &value, 0)?;
                }
                self.finish_ordered_record(state, 0)?
            }
            ValueExpr::RecordBuild {
                ty,
                initializers,
            } => {
                let mut value = self.zero_value(*ty)?;
                let mut cells = value.cells(self.limits.value_cells)?;
                let mut seen = std::collections::HashSet::new();
                for (field, expression) in initializers {
                    if !seen.insert(*field) {
                        return Err(Error::InvalidIr("duplicate record field initializer").into());
                    }
                    let field_ty = self.provider.types().validate_field(*ty, *field)?;
                    let field_value = self.value(expression, depth + 1)?;
                    field_value.validate(
                        self.provider.types(),
                        field_ty,
                        self.limits.evaluation_depth.min(256),
                    )?;
                    match &mut value {
                        Value::StoredAggregate(snapshot) => {
                            self.charge_work(snapshot.storage_cells())?;
                            let updated = snapshot.with_field(
                                self.provider.types(),
                                field.index(),
                                &field_value,
                                self.limits.value_cells,
                            )?;
                            cells = updated.cells(self.limits.value_cells)?;
                            value = updated;
                        }
                        Value::Record {
                            fields, ..
                        } => {
                            let length = fields.len();
                            let destination =
                                fields.get_mut(field.index()).ok_or(Error::OutOfBounds {
                                    index: field.index(),
                                    length,
                                })?;
                            let replacement = field_value.cells(self.limits.value_cells)?;
                            cells = cells
                                .checked_sub(destination.cells(self.limits.value_cells)?)
                                .and_then(|cells| cells.checked_add(replacement))
                                .filter(|cells| *cells <= self.limits.value_cells)
                                .ok_or(Error::Limit(LimitKind::ValueCells))?;
                            self.charge_work(replacement)?;
                            *destination = field_value;
                        }
                        Value::Union {
                            field: active,
                            value,
                            ..
                        } if initializers.len() == 1 => {
                            *active = field.index();
                            **value = field_value;
                        }
                        _ => {
                            return Err(
                                Error::InvalidIr("record initializer requires a record").into()
                            );
                        }
                    }
                }
                value
            }
            ValueExpr::Field {
                base,
                field,
                ty,
            } => {
                let base = self.value(base, depth + 1)?;
                let base_ty = match base.semantic() {
                    Value::StoredAggregate(snapshot) => snapshot.ty(),
                    Value::Record {
                        ty, ..
                    }
                    | Value::Union {
                        ty, ..
                    } => *ty,
                    _ => return Err(Error::InvalidIr("field access requires a record").into()),
                };
                if self.provider.types().validate_field(base_ty, *field)? != *ty {
                    return Err(Error::TypeMismatch {
                        expected: *ty,
                    }
                    .into());
                }
                match base {
                    Value::StoredAggregate(snapshot) => {
                        self.charge_work(snapshot.storage_cells())?;
                        snapshot.field(
                            self.provider.types(),
                            field.index(),
                            self.limits.value_cells,
                        )?
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
                        let value = self.normalize_storage_value(value, depth + 1)?;
                        self.memory.union_value_field(
                            self.provider.types(),
                            base_ty,
                            &value,
                            field.index(),
                        )?
                    }
                    _ => {
                        return Err(Error::InvalidIr("field access requires record storage").into());
                    }
                }
            }
            ValueExpr::Call {
                call,
                ty,
            } => {
                let value = self.one(call, depth + 1)?;
                value.validate(
                    self.provider.types(),
                    *ty,
                    self.limits.evaluation_depth.min(256),
                )?;
                value
            }
            ValueExpr::Enum {
                ty,
                value,
            } => {
                let value = Value::Enum {
                    ty: *ty,
                    value: *value,
                };
                value.validate(
                    self.provider.types(),
                    *ty,
                    self.limits.evaluation_depth.min(256),
                )?;
                value
            }
            ValueExpr::EnumFromInt {
                ty,
                value,
            } => {
                let value = Value::Enum {
                    ty: *ty,
                    value: {
                        let number = self.integer(value, depth + 1)?;
                        if number.provenance().is_some() {
                            return Err(Error::UnsupportedPointerOperation(
                                "address-derived integer cannot convert to an enum",
                            )
                            .into());
                        }
                        number.integer()
                    },
                };
                value.validate(
                    self.provider.types(),
                    *ty,
                    self.limits.evaluation_depth.min(256),
                )?;
                value
            }
        };
        value.cells(self.limits.value_cells)?;
        Ok(value)
    }
}
