//! Constants needing virtual storage are hydrated by the actual interpreter.
use super::*;

impl<P: ProcedureProvider + ?Sized, E: CompilerEffects> Vm<'_, P, E> {
    pub(super) fn global_value(
        &mut self,
        initializer: &GlobalInitializer,
        depth: usize,
    ) -> Result<Value> {
        match initializer {
            GlobalInitializer::External(_) => {
                Err(Error::InvalidIr("external data has no constant initializer").into())
            }
            GlobalInitializer::Value(value) => self.constant_value(value, depth),
            GlobalInitializer::Int(value) => Ok(Value::Int(*value)),
            GlobalInitializer::Bool(value) => Ok(Value::Bool(*value)),
        }
    }
    pub(super) fn constant_value(
        &mut self,
        constant: &ConstantValue,
        depth: usize,
    ) -> Result<Value> {
        self.step(depth)?;
        let value = match &constant.kind {
            ConstantKind::Zero => self.zero_value(constant.ty)?,
            ConstantKind::NativePointer(value) => crate::constants::native_pointer(
                self.provider.types(),
                value,
                self.memory.target(),
            )?,
            ConstantKind::RuntimeType(value) => self.runtime_type_constant(value, depth + 1)?,
            ConstantKind::Record(fields) | ConstantKind::Array(fields) => {
                if fields.len() > self.limits.value_cells {
                    return Err(Error::Limit(LimitKind::ValueCells).into());
                }
                let mut values = Vec::with_capacity(fields.len());
                let mut cells = 1;
                for field in fields {
                    let value = self.constant_value(field, depth + 1)?;
                    self.push_value(&mut values, &mut cells, value)?;
                }
                if matches!(constant.kind, ConstantKind::Record(_)) {
                    Value::Record {
                        ty: constant.ty,
                        fields: values,
                    }
                } else {
                    Value::Array {
                        ty: constant.ty,
                        elements: values,
                    }
                }
            }
            ConstantKind::Union {
                field,
                value,
            } => Value::Union {
                ty: constant.ty,
                field: field.index(),
                value: Box::new(self.constant_value(value, depth + 1)?),
            },
            ConstantKind::Distinct(value) => Value::Distinct {
                ty: constant.ty,
                value: Box::new(self.constant_value(value, depth + 1)?),
            },
            _ => crate::constants::global(
                self.provider.types(),
                &GlobalInitializer::Value(constant.clone()),
                self.limits,
            )?,
        };
        value.cells(self.limits.value_cells)?;
        value.validate(
            self.provider.types(),
            constant.ty,
            self.limits.evaluation_depth,
        )?;
        self.memory
            .validate_runtime_type_values(self.provider.types(), &value)?;
        Ok(value)
    }
}
