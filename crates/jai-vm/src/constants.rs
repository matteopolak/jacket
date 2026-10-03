use crate::{Error, LimitKind, Limits, Pointer, Value};
use jai_ir::{ConstantKind, ConstantValue, ContextDefinition, Global, GlobalInitializer};
use jai_types::{FloatType, FloatValue, Integer, RecordKind, TypeId, TypeKind, TypeView};

struct Budget {
    remaining: usize,
    depth: usize,
}

/// Bound borrowed definitions before VM state snapshots recursively clone or compare
/// them. This checks only tree shape, so unused nominal types can remain pending.
pub(crate) fn preflight_provider_constants(
    globals: &[Global],
    context: Option<&ContextDefinition>,
    limits: Limits,
) -> Result<(), Error> {
    let mut budget = ShapeBudget {
        remaining: limits.value_cells,
        depth: limits.evaluation_depth.min(256),
    };
    for global in globals {
        match global.initializer() {
            GlobalInitializer::Value(value) => budget.constant(value)?,
            GlobalInitializer::Int(_) | GlobalInitializer::Bool(_) => budget.reserve(1)?,
            GlobalInitializer::External(data) => {
                budget.reserve(1)?;
                budget.reserve(data.symbol().len())?;
                if let jai_ir::ExternalDataSource::Library(binding) = data.source() {
                    budget.reserve(match &binding.kind {
                        jai_ir::ForeignLibraryKind::System {
                            name,
                        } => name.len(),
                        jai_ir::ForeignLibraryKind::Local {
                            path,
                        } => path.as_os_str().len(),
                    })?;
                }
            }
        }
    }
    if let Some(context) = context {
        budget.constant(&context.default)?;
    }
    Ok(())
}

struct ShapeBudget {
    remaining: usize,
    depth: usize,
}
impl ShapeBudget {
    fn reserve(&mut self, count: usize) -> Result<(), Error> {
        self.remaining = self
            .remaining
            .checked_sub(count)
            .ok_or(Error::Limit(LimitKind::ValueCells))?;
        Ok(())
    }

    fn constant(&mut self, root: &ConstantValue) -> Result<(), Error> {
        self.reserve(1)?;
        let mut pending = vec![(root, 0usize)];
        while let Some((value, depth)) = pending.pop() {
            if depth > self.depth {
                return Err(Error::Limit(LimitKind::EvaluationDepth));
            }
            match &value.kind {
                ConstantKind::Record(children) | ConstantKind::Array(children) => {
                    self.reserve(children.len())?;
                    if !children.is_empty() && depth == self.depth {
                        return Err(Error::Limit(LimitKind::EvaluationDepth));
                    }
                    pending.extend(children.iter().map(|child| (child, depth + 1)));
                }
                ConstantKind::Distinct(child)
                | ConstantKind::Union {
                    value: child, ..
                } => {
                    self.reserve(1)?;
                    if depth == self.depth {
                        return Err(Error::Limit(LimitKind::EvaluationDepth));
                    }
                    pending.push((child, depth + 1));
                }
                ConstantKind::StringBytes(bytes) => self.reserve(bytes.len())?,
                ConstantKind::Int(_)
                | ConstantKind::Float(_)
                | ConstantKind::Bool(_)
                | ConstantKind::Enum(_)
                | ConstantKind::Procedure(_)
                | ConstantKind::NativePointer(_)
                | ConstantKind::RuntimeType(_)
                | ConstantKind::Zero => {}
            }
        }
        Ok(())
    }
}

impl Budget {
    fn consume(&mut self, depth: usize) -> Result<(), Error> {
        if depth > self.depth {
            return Err(Error::Limit(LimitKind::EvaluationDepth));
        }
        self.remaining = self
            .remaining
            .checked_sub(1)
            .ok_or(Error::Limit(LimitKind::ValueCells))?;
        Ok(())
    }
}
pub(crate) fn zero(types: &dyn TypeView, ty: TypeId, limits: Limits) -> Result<Value, Error> {
    zero_inner(
        types,
        ty,
        &mut Budget {
            remaining: limits.value_cells,
            depth: limits.evaluation_depth.min(256),
        },
        0,
    )
}
fn zero_inner(
    types: &dyn TypeView,
    ty: TypeId,
    budget: &mut Budget,
    depth: usize,
) -> Result<Value, Error> {
    budget.consume(depth)?;
    Ok(match types.kind(ty)? {
        TypeKind::Type => Value::Type {
            descriptor: None,
        },
        TypeKind::Procedure(_) => Value::Procedure {
            signature: ty,
            procedure: None,
        },
        TypeKind::Distinct(id) => Value::Distinct {
            ty,
            value: Box::new(zero_inner(
                types,
                types.distinct(*id)?.representation,
                budget,
                depth + 1,
            )?),
        },
        TypeKind::Bool => Value::Bool(false),
        TypeKind::Integer(integer) => Value::Int(Integer::wrapping(*integer, 0)),
        TypeKind::Float(FloatType::F32) => Value::Float(FloatValue::F32(0)),
        TypeKind::Float(FloatType::F64) => Value::Float(FloatValue::F64(0)),
        TypeKind::String => Value::String(vec![]),
        TypeKind::Pointer(pointee) => Value::Pointer(Pointer::null(*pointee)),
        TypeKind::DynamicArray(element) => Value::DynamicArray {
            ty,
            pointer: Pointer::null(*element),
            count: 0,
            allocated: 0,
            allocator: crate::value::allocator_schema(types)?
                .map(|schema| zero_inner(types, schema.ty(), budget, depth + 1).map(Box::new))
                .transpose()?,
        },
        TypeKind::Slice(element) => Value::Slice {
            ty,
            pointer: Pointer::null(*element),
            count: 0,
        },
        TypeKind::Enum(id) => Value::Enum {
            ty,
            value: Integer::wrapping(types.enumeration(*id)?.representation, 0),
        },
        kind if kind.record_storage_id().is_some() => {
            let record = types.record_storage_definition(ty)?;
            if record.kind == RecordKind::Union {
                let first = record.fields.first().ok_or(Error::UnsupportedType(ty))?;
                Value::Union {
                    ty,
                    field: 0,
                    value: Box::new(zero_inner(types, *first, budget, depth + 1)?),
                }
            } else {
                if record.fields.len() > budget.remaining {
                    return Err(Error::Limit(LimitKind::ValueCells));
                }
                let mut fields = Vec::with_capacity(record.fields.len());
                for &field in &record.fields {
                    fields.push(zero_inner(types, field, budget, depth + 1)?);
                }
                Value::Record {
                    ty,
                    fields,
                }
            }
        }
        TypeKind::FixedArray {
            element,
            count,
        } => {
            let count = usize::try_from(*count)
                .ok()
                .filter(|count| *count <= budget.remaining)
                .ok_or(Error::Limit(LimitKind::ValueCells))?;
            let mut elements = Vec::with_capacity(count);
            for _ in 0..count {
                elements.push(zero_inner(types, *element, budget, depth + 1)?);
            }
            Value::Array {
                ty,
                elements,
            }
        }
        _ => return Err(Error::UnsupportedType(ty)),
    })
}
pub(crate) fn global(
    types: &dyn TypeView,
    initializer: &GlobalInitializer,
    limits: Limits,
) -> Result<Value, Error> {
    match initializer {
        GlobalInitializer::External(_) => Err(Error::InvalidIr(
            "external data has no constant initializer",
        )),
        GlobalInitializer::Int(value) => Ok(Value::Int(*value)),
        GlobalInitializer::Bool(value) => Ok(Value::Bool(*value)),
        GlobalInitializer::Value(value) => constant(
            types,
            value,
            &mut Budget {
                remaining: limits.value_cells,
                depth: limits.evaluation_depth.min(256),
            },
            0,
        ),
    }
}
fn constant(
    types: &dyn TypeView,
    constant: &ConstantValue,
    budget: &mut Budget,
    depth: usize,
) -> Result<Value, Error> {
    if matches!(constant.kind, ConstantKind::Zero) {
        return zero_inner(types, constant.ty, budget, depth);
    }
    budget.consume(depth)?;
    let value = match &constant.kind {
        ConstantKind::NativePointer(_) => {
            return Err(Error::InvalidIr(
                "native pointer constants require the selected VM target",
            ));
        }
        ConstantKind::RuntimeType(_) => {
            return Err(Error::InvalidIr(
                "runtime Type constants require VM static publication",
            ));
        }
        ConstantKind::Union {
            field,
            value,
        } => Value::Union {
            ty: constant.ty,
            field: field.index(),
            value: Box::new(self::constant(types, value, budget, depth + 1)?),
        },
        ConstantKind::Float(value) => Value::Float(*value),
        ConstantKind::StringBytes(bytes) => {
            budget.remaining = budget
                .remaining
                .checked_sub(bytes.len())
                .ok_or(Error::Limit(LimitKind::ValueCells))?;
            Value::String(bytes.clone())
        }
        ConstantKind::Distinct(value) => Value::Distinct {
            ty: constant.ty,
            value: Box::new(self::constant(types, value, budget, depth + 1)?),
        },
        ConstantKind::Array(elements) => {
            if elements.len() > budget.remaining {
                return Err(Error::Limit(LimitKind::ValueCells));
            }
            let mut values = Vec::with_capacity(elements.len());
            for element in elements {
                values.push(self::constant(types, element, budget, depth + 1)?);
            }
            Value::Array {
                ty: constant.ty,
                elements: values,
            }
        }
        ConstantKind::Int(value) => Value::Int(*value),
        ConstantKind::Bool(value) => Value::Bool(*value),
        ConstantKind::Procedure(procedure) => Value::Procedure {
            signature: constant.ty,
            procedure: Some(*procedure),
        },
        ConstantKind::Enum(value) => Value::Enum {
            ty: constant.ty,
            value: *value,
        },
        ConstantKind::Record(fields) => {
            if fields.len() > budget.remaining {
                return Err(Error::Limit(LimitKind::ValueCells));
            }
            let mut values = Vec::with_capacity(fields.len());
            for field in fields {
                values.push(self::constant(types, field, budget, depth + 1)?);
            }
            Value::Record {
                ty: constant.ty,
                fields: values,
            }
        }
        ConstantKind::Zero => return Err(Error::InvalidIr("unreachable zero constant")),
    };
    value.validate(types, constant.ty, budget.depth)?;
    Ok(value)
}

/// Native address capsules contain numeric bits, never virtual provenance.
pub(crate) fn native_pointer(
    types: &dyn TypeView,
    constant: &jai_ir::NativePointerConstant,
    target: crate::ByteTarget,
) -> Result<Value, Error> {
    use jai_ir::NativePointerConstantError;
    let error = |error| match error {
        NativePointerConstantError::Type(error) => Error::Type(error),
        NativePointerConstantError::CheckedCast => Error::CheckedCast,
        NativePointerConstantError::InvalidType(ty) => Error::TypeMismatch {
            expected: ty,
        },
        NativePointerConstantError::UnsupportedWidth(_) => {
            Error::UnsupportedPointerOperation("native pointer constant width is unsupported")
        }
        NativePointerConstantError::UnsupportedMode(_) => {
            Error::InvalidIr("native pointer constant has a nonnumeric cast mode")
        }
    };
    constant.validate(types).map_err(error)?;
    let bits = u32::try_from(target.policy.pointer().size)
        .ok()
        .and_then(|size| size.checked_mul(8))
        .ok_or(Error::UnsupportedPointerOperation(
            "native pointer constant width is unsupported",
        ))?;
    let address = constant.address(bits).map_err(error)?;
    let TypeKind::Pointer(pointee) = types.kind(constant.type_id())? else {
        return Err(Error::TypeMismatch {
            expected: constant.type_id(),
        });
    };
    Ok(Value::Pointer(Pointer::opaque(
        address.bits(),
        bits,
        *pointee,
    )?))
}
