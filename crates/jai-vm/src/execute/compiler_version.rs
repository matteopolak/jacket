//! Publish this implementation's version through the checked source ABI.
use super::*;
use jai_types::{IntegerType, RecordKind, ScalarType};

const VERSION: &str = env!("CARGO_PKG_VERSION");

impl<P: ProcedureProvider + ?Sized, E: CompilerEffects> Vm<'_, P, E> {
    pub(super) fn compiler_version_info(
        &mut self,
        record: TypeId,
        arguments: &[Value],
    ) -> Result<Vec<Value>> {
        let [Value::Pointer(pointer)] = arguments else {
            return Err(
                Error::InvalidIr("version info requires its checked record pointer").into(),
            );
        };
        let types = self.provider.types();
        let shape = types.record_definition(record)?;
        let signed = types.scalar(ScalarType::Int(IntegerType::S32));
        if shape.kind != RecordKind::Struct
            || shape.fields.as_ref() != [signed, signed, signed]
            || pointer.pointee() != record
        {
            return Err(
                Error::InvalidIr("version info requires the checked s32 record shape").into(),
            );
        }
        if VERSION.len().saturating_add(1) > self.limits.value_cells {
            return Err(Error::Limit(LimitKind::ValueCells).into());
        }
        self.charge_work(VERSION.len())?;
        if !pointer.is_null() {
            self.prepare_pointer_layouts(pointer, true)?;
            // Preflight both backing replacement and the new four-node record
            // before allocating its fields or changing caller-owned memory.
            let work = self.memory.store_work_cost(types, pointer)?;
            self.charge_work(work.checked_add(4).ok_or(Error::Limit(LimitKind::Fuel))?)?;
            let mut fields = Vec::with_capacity(3);
            for part in [
                env!("CARGO_PKG_VERSION_MAJOR"),
                env!("CARGO_PKG_VERSION_MINOR"),
                env!("CARGO_PKG_VERSION_PATCH"),
            ] {
                let value = part
                    .parse::<i128>()
                    .ok()
                    .and_then(|value| Integer::checked(IntegerType::S32, value))
                    .ok_or(Error::InvalidIr(
                        "implementation version exceeds source s32 ABI",
                    ))?;
                fields.push(Value::Int(value));
            }
            self.memory
                .store(types, pointer, Value::Record { ty: record, fields })?;
        }
        Ok(vec![Value::String(VERSION.as_bytes().to_vec())])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_types::TypeRegistry;

    struct Provider(TypeRegistry);
    impl ProcedureProvider for Provider {
        fn types(&self) -> &dyn TypeView {
            &self.0
        }
        fn procedure(&self, _: ProcedureId) -> ProcedureAvailability<'_> {
            ProcedureAvailability::Missing
        }
    }
    fn provider() -> (Provider, TypeId) {
        let mut types = TypeRegistry::new();
        let record = types.reserve_record(RecordKind::Struct);
        let signed = types.scalar(ScalarType::Int(IntegerType::S32));
        types.define_record(record, [signed; 3]).unwrap();
        (Provider(types), record)
    }
    #[test]
    fn null_version_pointer_returns_the_actual_implementation_version() {
        let (provider, record) = provider();
        let mut vm = Vm::new(&provider, crate::NoEffects, Limits::default()).unwrap();
        assert_eq!(
            vm.compiler_version_info(record, &[Value::Pointer(Pointer::null(record))])
                .ok(),
            Some(vec![Value::String(VERSION.as_bytes().to_vec())]),
        );
    }
    #[test]
    fn version_fields_write_real_memory_and_fuel_failure_preserves_it() {
        let (provider, record) = provider();
        for fuel in [VERSION.len() as u64, 100] {
            let mut vm = Vm::new(
                &provider,
                crate::NoEffects,
                Limits {
                    fuel,
                    ..Limits::default()
                },
            )
            .unwrap();
            let old = Value::Record {
                ty: record,
                fields: vec![Value::Int(Integer::wrapping(IntegerType::S32, -1)); 3],
            };
            let pointer = vm
                .memory
                .allocate(&provider.0, record, Some(old.clone()))
                .unwrap();
            let result = vm.compiler_version_info(record, &[Value::Pointer(pointer.clone())]);
            let actual = vm.memory.load(&provider.0, &pointer).unwrap();
            if fuel == VERSION.len() as u64 {
                assert!(matches!(
                    result,
                    Err(Halt::Failed(Error::Limit(LimitKind::Fuel)))
                ));
                assert_eq!(actual, old);
            } else {
                assert!(result.is_ok());
                let fields = [
                    env!("CARGO_PKG_VERSION_MAJOR"),
                    env!("CARGO_PKG_VERSION_MINOR"),
                    env!("CARGO_PKG_VERSION_PATCH"),
                ]
                .into_iter()
                .map(|part| {
                    Value::Int(Integer::checked(IntegerType::S32, part.parse().unwrap()).unwrap())
                })
                .collect();
                assert_eq!(actual, Value::Record { ty: record, fields });
            }
        }
    }

    #[test]
    fn readonly_version_storage_rejects_without_replacing_the_record() {
        let (provider, record) = provider();
        let mut vm = Vm::new(&provider, crate::NoEffects, Limits::default()).unwrap();
        let old = Value::Record {
            ty: record,
            fields: vec![Value::Int(Integer::wrapping(IntegerType::S32, -1)); 3],
        };
        let pointer = vm
            .memory
            .allocate(&provider.0, record, Some(old.clone()))
            .unwrap();
        vm.memory.freeze(&pointer).unwrap();
        assert!(matches!(
            vm.compiler_version_info(record, &[Value::Pointer(pointer.clone())]),
            Err(Halt::Failed(Error::ReadOnlyStorage))
        ));
        assert_eq!(vm.memory.load(&provider.0, &pointer).unwrap(), old);
    }

    #[test]
    fn checked_compiler_call_dispatches_to_the_memory_adapter() {
        struct Callable {
            types: TypeRegistry,
            signatures: std::collections::HashMap<ProcedureId, TypeId>,
            compiler: crate::CompilerProcedure,
        }
        impl ProcedureProvider for Callable {
            fn types(&self) -> &dyn TypeView {
                &self.types
            }
            fn signatures(&self) -> &std::collections::HashMap<ProcedureId, TypeId> {
                &self.signatures
            }
            fn procedure(&self, _: ProcedureId) -> ProcedureAvailability<'_> {
                ProcedureAvailability::Compiler(self.compiler)
            }
        }
        let (Provider(mut types), record) = provider();
        let pointer_type = types.pointer(record).unwrap();
        let signature = types
            .procedure(ProcedureType {
                parameters: vec![pointer_type].into(),
                results: vec![types.string()].into(),
                convention: jai_types::CallingConvention::Jai,
                context: jai_types::ContextMode::Implicit,
                variadic: jai_types::Variadic::None,
            })
            .unwrap();
        let id = ProcedureId::new(0);
        let provider = Callable {
            types,
            signatures: [(id, signature)].into(),
            compiler: crate::CompilerProcedure {
                signature,
                intrinsic: crate::CompilerIntrinsic::SourceVersionInfo { record },
            },
        };
        let mut vm = Vm::new(&provider, crate::NoEffects, Limits::default()).unwrap();
        let pointer = vm.memory.allocate(&provider.types, record, None).unwrap();
        assert_eq!(
            vm.execute(id, vec![Value::Pointer(pointer.clone())])
                .outcome,
            Outcome::Complete(vec![Value::String(VERSION.as_bytes().to_vec())]),
        );
        let Value::Record { fields, .. } = vm.memory.load(&provider.types, &pointer).unwrap()
        else {
            panic!("version record must be initialized by the checked compiler call");
        };
        assert_eq!(fields.len(), 3);
        assert!(
            fields
                .iter()
                .all(|value| matches!(value, Value::Int(integer) if integer.value() >= 0))
        );
    }
}
