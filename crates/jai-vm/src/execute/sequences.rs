use super::*;
#[cfg(test)]
#[path = "allocator_tests.rs"]
mod allocator_tests;
#[path = "materialize.rs"]
mod materialize;

impl<P: ProcedureProvider + ?Sized, E: CompilerEffects> Vm<'_, P, E> {
    pub(super) fn temporary(&mut self, ty: TypeId, value: Value, depth: usize) -> Result<Pointer> {
        let value = self.normalize_storage_value(value, depth + 1)?;
        self.prepare_layout(ty)?;
        let pointer = self
            .memory
            .allocate(self.provider.types(), ty, Some(value))?;
        if let Some(frame) = self.frames.last_mut() {
            frame.temporaries.push(pointer.clone());
        } else {
            self.root_temporaries.push(pointer.clone());
        }
        Ok(pointer)
    }
    fn backing(
        &mut self,
        expression: &ValueExpr,
        ty: TypeId,
        value: Value,
        depth: usize,
    ) -> Result<Pointer> {
        if jai_ir::is_static_value(expression) {
            let value = self.normalize_storage_value(value, depth + 1)?;
            self.immutable_backing(ty, value, depth + 1)
        } else {
            self.temporary(ty, value, depth + 1)
        }
    }
    /// Literal bytes and static arrays outlive frames and retain stable addresses.
    pub(super) fn immutable_backing(
        &mut self,
        ty: TypeId,
        value: Value,
        depth: usize,
    ) -> Result<Pointer> {
        self.step(depth)?;
        let cells = value.cells(self.limits.value_cells)?;
        self.charge_work(cells)?;
        let key = (ty, value);
        if let Some(pointer) = self.literal_backing.get(&key) {
            return Ok(pointer.clone());
        }
        // A miss clones the allocation value and hashes the key again on insert.
        self.charge_work(cells.checked_mul(2).ok_or(Error::Limit(LimitKind::Fuel))?)?;
        if !self.literal_backing.is_empty()
            && self.literal_backing.len() == self.literal_backing.capacity()
        {
            // Growing the table may rehash every prior value key. Their matching
            // backing allocations provide a cached upper bound on that work.
            self.charge_work(self.memory.value_cells())?;
        }
        self.prepare_layout(ty)?;
        let pointer = self
            .memory
            .allocate(self.provider.types(), ty, Some(key.1.clone()))?;
        self.memory.freeze(&pointer)?;
        self.literal_backing.insert(key, pointer.clone());
        Ok(pointer)
    }
    pub(super) fn string_descriptor(&mut self, bytes: Vec<u8>, depth: usize) -> Result<Value> {
        let count = bytes.len();
        let pointer = if bytes.is_empty() {
            Pointer::null(
                self.provider
                    .types()
                    .scalar(jai_types::ScalarType::Int(jai_types::IntegerType::U8)),
            )
        } else {
            let ty = self
                .provider
                .types()
                .lookup(&TypeKind::String)
                .ok_or(Error::InvalidIr("string type is not registered"))?;
            let storage = self.immutable_backing(ty, Value::String(bytes), depth + 1)?;
            self.prepare_pointer_layouts(&storage, false)?;
            self.memory.cast_pointer(
                self.provider.types(),
                &storage,
                self.provider
                    .types()
                    .scalar(jai_types::ScalarType::Int(jai_types::IntegerType::U8)),
                CastMode::Checked,
            )?
        };
        Ok(Value::StringView {
            pointer,
            count: i64::try_from(count).map_err(|_| Error::CheckedCast)?,
        })
    }
    /// Convert a mutable descriptor slot, keeping literal backing independent of it.
    pub(super) fn ensure_string_storage(&mut self, storage: &Pointer, depth: usize) -> Result<()> {
        if matches!(
            self.provider.types().kind(storage.pointee())?,
            TypeKind::String
        ) && let Value::String(bytes) = self.load_sequence_value(storage)?
        {
            let descriptor = self.string_descriptor(bytes, depth + 1)?;
            self.store_pointer(storage, descriptor, depth + 1)?;
        }
        Ok(())
    }
    /// Storage contains descriptors; only the pool's byte allocations own string bytes.
    pub(super) fn normalize_storage_value(&mut self, value: Value, depth: usize) -> Result<Value> {
        value.cells(self.limits.value_cells)?;
        self.normalize_storage_inner(value, depth + 1)
    }
    fn normalize_storage_inner(&mut self, value: Value, depth: usize) -> Result<Value> {
        self.step(depth)?;
        Ok(match value {
            Value::String(bytes) => self.string_descriptor(bytes, depth + 1)?,
            Value::Record {
                ty,
                fields,
            } => Value::Record {
                ty,
                fields: fields
                    .into_iter()
                    .map(|field| self.normalize_storage_inner(field, depth + 1))
                    .collect::<Result<_>>()?,
            },
            Value::Array {
                ty,
                elements,
            } => Value::Array {
                ty,
                elements: elements
                    .into_iter()
                    .map(|element| self.normalize_storage_inner(element, depth + 1))
                    .collect::<Result<_>>()?,
            },
            Value::Distinct {
                ty,
                value,
            } => Value::Distinct {
                ty,
                value: Box::new(self.normalize_storage_inner(*value, depth + 1)?),
            },
            Value::Union {
                ty,
                field,
                value,
            } => Value::Union {
                ty,
                field,
                value: Box::new(self.normalize_storage_inner(*value, depth + 1)?),
            },
            Value::DynamicArray {
                ty,
                pointer,
                count,
                allocated,
                allocator,
            } => Value::DynamicArray {
                ty,
                pointer,
                count,
                allocated,
                allocator: allocator
                    .map(|value| {
                        self.normalize_storage_inner(*value, depth + 1)
                            .map(Box::new)
                    })
                    .transpose()?,
            },
            value => value,
        })
    }
    pub(super) fn sequence_parts(
        &mut self,
        base: &ValueExpr,
        depth: usize,
    ) -> Result<(Value, Option<Pointer>)> {
        let storage = if let ValueExpr::Load(place) = base {
            Some(self.place(*place, depth + 1)?)
        } else {
            None
        };
        let value = if let Some(pointer) = &storage {
            self.load_sequence_value(pointer)?
        } else {
            self.value(base, depth + 1)?
        };
        let storage = match (value.semantic(), storage) {
            (
                Value::Array {
                    elements, ..
                },
                None,
            ) if elements.is_empty() => None,
            (
                Value::Array {
                    ty, ..
                },
                None,
            ) => {
                self.charge_work(value.cells(self.limits.value_cells)?)?;
                Some(self.backing(base, *ty, value.clone(), depth + 1)?)
            }
            (_, storage) => storage,
        };
        let value = match value {
            Value::String(bytes) => self.string_descriptor(bytes, depth + 1)?,
            value => value,
        };
        Ok((value, storage))
    }
    pub(super) fn sequence_field(
        &mut self,
        base: &ValueExpr,
        field: SequenceField,
        depth: usize,
    ) -> Result<Value> {
        if let ValueExpr::Load(place) = base
            && let TypeKind::FixedArray {
                count, ..
            } = *self.provider.types().kind(place.ty())?
        {
            let storage = self.place(*place, depth + 2)?;
            return Ok(match field {
                SequenceField::Count => Value::Int(
                    Integer::checked(jai_types::IntegerType::S64, i128::from(count))
                        .ok_or(Error::CheckedCast)?,
                ),
                SequenceField::Data => {
                    Value::Pointer(self.array_data(place.ty(), count == 0, Some(storage))?)
                }
                SequenceField::Allocated => {
                    return Err(Error::InvalidIr("allocated field requires dynamic array").into());
                }
            });
        }
        let (value, storage) = self.sequence_parts(base, depth + 1)?;
        if let Value::Array {
            ty,
            elements,
        } = value.semantic()
        {
            return Ok(match field {
                SequenceField::Count => Value::Int(
                    Integer::checked(
                        jai_types::IntegerType::S64,
                        i128::try_from(elements.len()).map_err(|_| Error::CheckedCast)?,
                    )
                    .ok_or(Error::CheckedCast)?,
                ),
                SequenceField::Data => {
                    Value::Pointer(self.array_data(*ty, elements.is_empty(), storage)?)
                }
                SequenceField::Allocated => {
                    return Err(Error::InvalidIr("allocated field requires dynamic array").into());
                }
            });
        }
        let (count, allocated, pointer) = match value {
            Value::String(bytes) => {
                let storage = storage.ok_or(Error::InvalidIr("string has no storage"))?;
                self.prepare_pointer_layouts(&storage, true)?;
                let pointer = self.memory.sequence_data(self.provider.types(), &storage)?;
                (
                    i64::try_from(bytes.len()).map_err(|_| Error::CheckedCast)?,
                    None,
                    pointer,
                )
            }
            Value::StringView {
                pointer,
                count,
            }
            | Value::Slice {
                pointer,
                count,
                ..
            } => (count, None, pointer),
            Value::DynamicArray {
                pointer,
                count,
                allocated,
                ..
            } => (count, Some(allocated), pointer),
            _ => {
                return Err(
                    Error::InvalidIr("sequence field requires array, string or slice").into(),
                );
            }
        };
        Ok(match field {
            SequenceField::Count => Value::Int(
                Integer::checked(jai_types::IntegerType::S64, count as i128)
                    .ok_or(Error::CheckedCast)?,
            ),
            SequenceField::Data => Value::Pointer(pointer),
            SequenceField::Allocated => Value::Int(
                Integer::checked(
                    jai_types::IntegerType::S64,
                    allocated.ok_or(Error::InvalidIr("allocated field requires dynamic array"))?
                        as i128,
                )
                .ok_or(Error::CheckedCast)?,
            ),
        })
    }
    pub(super) fn array_view(
        &mut self,
        base: &ValueExpr,
        ty: TypeId,
        depth: usize,
    ) -> Result<Value> {
        if let ValueExpr::Load(place) = base
            && let TypeKind::FixedArray {
                count, ..
            } = *self.provider.types().kind(place.ty())?
        {
            let storage = self.place(*place, depth + 2)?;
            let value = Value::Slice {
                ty,
                pointer: self.array_data(place.ty(), count == 0, Some(storage))?,
                count: i64::try_from(count).map_err(|_| Error::CheckedCast)?,
            };
            value.validate(
                self.provider.types(),
                ty,
                self.limits.evaluation_depth.min(256),
            )?;
            return Ok(value);
        }
        let (value, storage) = self.sequence_parts(base, depth + 1)?;
        let Value::Array {
            ty: array_ty,
            elements,
        } = value.semantic()
        else {
            return Err(Error::InvalidIr("array view requires fixed array").into());
        };
        let pointer = self.array_data(*array_ty, elements.is_empty(), storage)?;
        let value = Value::Slice {
            ty,
            pointer,
            count: i64::try_from(elements.len()).map_err(|_| Error::CheckedCast)?,
        };
        value.validate(
            self.provider.types(),
            ty,
            self.limits.evaluation_depth.min(256),
        )?;
        Ok(value)
    }
    fn array_data(&mut self, ty: TypeId, empty: bool, storage: Option<Pointer>) -> Result<Pointer> {
        if empty {
            let TypeKind::FixedArray {
                element, ..
            } = self.provider.types().kind(ty)?
            else {
                return Err(Error::InvalidIr("array value has incorrect type").into());
            };
            Ok(Pointer::null(*element))
        } else {
            let storage = storage.ok_or(Error::InvalidIr("array has no backing storage"))?;
            self.prepare_pointer_layouts(&storage, false)?;
            self.prepare_layout(ty)?;
            Ok(self.memory.sequence_data(self.provider.types(), &storage)?)
        }
    }
    pub(super) fn sequence_index(
        &mut self,
        base: &ValueExpr,
        index: &IntExpr,
        ty: TypeId,
        check: CheckMode,
        depth: usize,
    ) -> Result<Value> {
        let base = self.value(base, depth + 1)?;
        let index = self.integer(index, depth + 1)?.portable_integer()?.value();
        let index = usize::try_from(index).map_err(|_| Error::OutOfBounds {
            index: usize::MAX,
            length: 0,
        })?;
        let value = match base {
            Value::StoredAggregate(snapshot) => {
                let element = snapshot.element_type(self.provider.types(), index)?;
                self.prepare_layout(element)?;
                self.charge_work(
                    snapshot
                        .storage_cells()
                        .checked_add(
                            self.memory
                                .prepared_decoded_cells(self.provider.types(), element)?,
                        )
                        .ok_or(Error::Limit(LimitKind::Fuel))?,
                )?;
                self.prepare_layout(snapshot.ty())?;
                snapshot.index(self.provider.types(), index, self.limits.value_cells)?
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
                let pointer = self.indexed_sequence_pointer(&pointer, index)?;
                self.load_sequence_value(&pointer)?
            }
            Value::Pointer(pointer) => {
                let index = isize::try_from(index).map_err(|_| Error::OutOfBounds {
                    index,
                    length: 0,
                })?;
                self.prepare_pointer_layouts(&pointer, true)?;
                let pointer = self.memory.offset(self.provider.types(), &pointer, index)?;
                self.load_sequence_value(&pointer)?
            }
            _ => return Err(Error::InvalidIr("index requires sequence or pointer").into()),
        };
        value.validate(
            self.provider.types(),
            ty,
            self.limits.evaluation_depth.min(256),
        )?;
        Ok(value)
    }
    pub(super) fn load_sequence_value(&mut self, pointer: &Pointer) -> Result<Value> {
        self.prepare_pointer_layouts(pointer, true)?;
        let work = self.memory.load_work_cost(self.provider.types(), pointer)?;
        self.charge_work(work)?;
        Ok(self.memory.load(self.provider.types(), pointer)?)
    }
    pub(super) fn indexed_sequence_pointer(
        &mut self,
        pointer: &Pointer,
        index: usize,
    ) -> Result<Pointer> {
        self.prepare_pointer_layouts(pointer, true)?;
        let size = self
            .memory
            .prepared_layout(self.provider.types(), pointer.pointee())?
            .size;
        if size == 0 {
            Ok(self.memory.cast_pointer(
                self.provider.types(),
                pointer,
                pointer.pointee(),
                CastMode::Checked,
            )?)
        } else {
            Ok(self.memory.offset(
                self.provider.types(),
                pointer,
                isize::try_from(index).map_err(|_| Error::CheckedCast)?,
            )?)
        }
    }
    pub(super) fn sequence_build(
        &mut self,
        ty: TypeId,
        initializers: &[(SequenceField, ValueExpr)],
        depth: usize,
    ) -> Result<Value> {
        let element = match self.provider.types().kind(ty)? {
            TypeKind::String => self
                .provider
                .types()
                .scalar(jai_types::ScalarType::Int(jai_types::IntegerType::U8)),
            TypeKind::Slice(element) | TypeKind::DynamicArray(element) => *element,
            _ => return Err(Error::UnsupportedType(ty).into()),
        };
        let (mut count, mut allocated, mut pointer) = (0, 0, Pointer::null(element));
        let allocator = self.default_sequence_allocator(ty)?;
        let mut seen = std::collections::HashSet::new();
        for (field, expression) in initializers {
            if !seen.insert(*field) {
                return Err(Error::InvalidIr("duplicate sequence field initializer").into());
            }
            let value = self.value(expression, depth + 1)?;
            if matches!(value, Value::AddressInteger(_)) {
                return Err(Error::UnsupportedPointerOperation(
                    "address-derived integer cannot become a sequence descriptor count",
                )
                .into());
            }
            match field {
                SequenceField::Data => pointer = value.pointer()?.clone(),
                SequenceField::Count => {
                    count =
                        i64::try_from(value.integer()?.value()).map_err(|_| Error::CheckedCast)?
                }
                SequenceField::Allocated => {
                    allocated =
                        i64::try_from(value.integer()?.value()).map_err(|_| Error::CheckedCast)?
                }
            }
        }
        let value = match self.provider.types().kind(ty)? {
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
        value.validate(
            self.provider.types(),
            ty,
            self.limits.evaluation_depth.min(256),
        )?;
        Ok(value)
    }

    pub(super) fn default_sequence_allocator(&mut self, ty: TypeId) -> Result<Option<Box<Value>>> {
        if !matches!(self.provider.types().kind(ty)?, TypeKind::DynamicArray(_)) {
            return Ok(None);
        }
        crate::value::allocator_schema(self.provider.types())?
            .map(|schema| self.zero_value(schema.ty()).map(Box::new))
            .transpose()
    }

    /// Consume the actual typed payload, preserving its complete stored image.
    #[cfg(test)]
    pub(super) fn sequence_allocator(&mut self, value: Value) -> Result<Value> {
        let schema = crate::value::allocator_schema(self.provider.types())?.ok_or(
            Error::InvalidIr("dynamic array allocator role is unavailable"),
        )?;
        let Value::DynamicArray {
            ty,
            allocator: Some(allocator),
            ..
        } = value
        else {
            return Err(Error::InvalidIr("allocator field requires a typed dynamic array").into());
        };
        if !matches!(self.provider.types().kind(ty)?, TypeKind::DynamicArray(_)) {
            return Err(Error::InvalidIr("allocator field requires a dynamic array type").into());
        }
        allocator.validate(
            self.provider.types(),
            schema.ty(),
            self.limits.evaluation_depth.min(256),
        )?;
        Ok(*allocator)
    }

    #[cfg(test)]
    pub(super) fn adopt_sequence_allocator(&self, value: Value) -> Result<Box<Value>> {
        let schema = crate::value::allocator_schema(self.provider.types())?.ok_or(
            Error::InvalidIr("dynamic array allocator role is unavailable"),
        )?;
        value.validate(
            self.provider.types(),
            schema.ty(),
            self.limits.evaluation_depth.min(256),
        )?;
        Ok(Box::new(value))
    }

    /// A field address does not read unrelated, possibly unwritten descriptor slots.
    #[cfg(test)]
    pub(super) fn load_sequence_allocator(&mut self, storage: &Pointer) -> Result<Value> {
        let schema = crate::value::allocator_schema(self.provider.types())?.ok_or(
            Error::InvalidIr("dynamic array allocator role is unavailable"),
        )?;
        self.prepare_pointer_layouts(storage, false)?;
        self.prepare_layout(storage.pointee())?;
        self.prepare_layout(schema.ty())?;
        let allocator = self
            .memory
            .sequence_allocator(self.provider.types(), storage)?;
        self.load_sequence_value(&allocator)
    }
}
