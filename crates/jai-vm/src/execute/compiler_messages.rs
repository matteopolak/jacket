//! Reconstruct actual source event records in the receiving VM's memory arena.
use super::*;
use crate::{
    CompilerCompletion, CompilerEvent, CompilerMessageSchema, CompilerPhase, CompilerRequest,
    CompilerResponse, EffectOutcome,
};
use jai_types::{CastMode, FieldId, IntegerType, ScalarType};

impl<P: ProcedureProvider + ?Sized, E: CompilerEffects> Vm<'_, P, E> {
    pub(super) fn compiler_wait_for_message(
        &mut self,
        schema: CompilerMessageSchema,
        arguments: &[Value],
    ) -> Result<Vec<Value>> {
        if !arguments.is_empty() {
            return Err(Error::InvalidIr("compiler message wait requires no arguments").into());
        }
        self.validate_message_schema(schema)?;
        match self.effects.request(CompilerRequest::WaitForMessage) {
            EffectOutcome::Ready(CompilerResponse::Message(event)) => {
                self.materialize_compiler_message(schema, event)
            }
            EffectOutcome::Ready(_) => Err(Error::EffectResponse(
                "compiler wait requires an actual owned message response",
            )
            .into()),
            EffectOutcome::Pending(key) => Err(Halt::Pending(Dependency::Effect(key))),
            EffectOutcome::Rejected(reason) => Err(Error::EffectRejected(reason).into()),
        }
    }

    fn validate_message_schema(&self, schema: CompilerMessageSchema) -> Result<()> {
        let types = self.provider.types();
        let signed = types.scalar(ScalarType::Int(IntegerType::S64));
        let count = types.scalar(ScalarType::Int(IntegerType::S32));
        for (ty, field, expected) in [
            (schema.message, schema.workspace, signed),
            (schema.phase, schema.phase_base, schema.message),
            (schema.complete, schema.complete_base, schema.message),
            (schema.phase, schema.pending_count, count),
        ] {
            if types.validate_field(ty, field)? != expected {
                return Err(Error::InvalidIr(
                    "compiler event metadata has an incompatible source field",
                )
                .into());
            }
        }
        for (ty, field, representation) in [
            (schema.message, schema.kind, IntegerType::U8),
            (schema.phase, schema.phase_kind, IntegerType::U32),
            (schema.complete, schema.completion_error, IntegerType::U8),
        ] {
            let field_ty = types.validate_field(ty, field)?;
            let definition = types.enum_definition(field_ty)?;
            // Source enum/enum_flags syntax is verified by the semantic schema
            // binder; the canonical storage definition carries representation.
            if definition.representation != representation {
                return Err(Error::InvalidIr(
                    "compiler event metadata has an incompatible source enum",
                )
                .into());
            }
        }
        if schema.phase_base.index() != 0 || schema.complete_base.index() != 0 {
            return Err(Error::InvalidIr(
                "compiler event metadata requires its leading Message base",
            )
            .into());
        }
        Ok(())
    }

    pub(super) fn materialize_compiler_message(
        &mut self,
        schema: CompilerMessageSchema,
        event: CompilerEvent,
    ) -> Result<Vec<Value>> {
        self.validate_message_schema(schema)?;
        let (record, base_field, detail_field, kind, detail) = match event {
            CompilerEvent::Phase {
                phase, ..
            } => (
                schema.phase,
                schema.phase_base,
                schema.phase_kind,
                4,
                match phase {
                    CompilerPhase::SourceParsed => 0,
                    CompilerPhase::Typechecked {
                        ..
                    } => 1,
                    CompilerPhase::TargetCodeBuilt => 2,
                },
            ),
            CompilerEvent::Complete {
                error, ..
            } => (
                schema.complete,
                schema.complete_base,
                schema.completion_error,
                6,
                match error {
                    CompilerCompletion::None => 0,
                    CompilerCompletion::CompilationFailed => 1,
                    CompilerCompletion::CompilerShutdown => 2,
                },
            ),
        };
        let mut value = self.zero_value(record)?;
        let mut base = self.zero_value(schema.message)?;
        let workspace = Integer::checked(IntegerType::S64, i128::from(event.workspace().get()))
            .ok_or(Error::CheckedCast)?;
        let types = self.provider.types();
        set(
            &mut base,
            schema.kind,
            source_enum(types, schema.message, schema.kind, kind)?,
        )?;
        set(&mut base, schema.workspace, Value::Int(workspace))?;
        set(&mut value, base_field, base)?;
        set(
            &mut value,
            detail_field,
            source_enum(types, record, detail_field, detail)?,
        )?;
        if let CompilerEvent::Phase {
            phase: CompilerPhase::Typechecked {
                pending_count,
            },
            ..
        } = event
        {
            let count = Integer::checked(IntegerType::S32, i128::from(pending_count))
                .ok_or(Error::CheckedCast)?;
            set(&mut value, schema.pending_count, Value::Int(count))?;
        }
        let value = self.normalize_storage_value(value, 0)?;
        self.prepare_layout(record)?;
        let types = self.provider.types();
        let pointer = self.memory.allocate(types, record, Some(value))?;
        self.memory.freeze(&pointer)?;
        let base = self
            .memory
            .cast_pointer(types, &pointer, schema.message, CastMode::Checked)?;
        Ok(vec![Value::Pointer(base)])
    }
}

fn source_enum(
    types: &dyn TypeView,
    record: TypeId,
    field: FieldId,
    number: i128,
) -> std::result::Result<Value, Error> {
    let ty = types.validate_field(record, field)?;
    let definition = types.enum_definition(ty)?;
    let value = Integer::checked(definition.representation, number).ok_or(Error::CheckedCast)?;
    if !definition.values.contains(&value) {
        return Err(Error::InvalidIr(
            "compiler message tag is absent from the checked source enum",
        ));
    }
    Ok(Value::Enum {
        ty,
        value,
    })
}
fn set(record: &mut Value, field: FieldId, value: Value) -> std::result::Result<(), Error> {
    let Value::Record {
        fields, ..
    } = record
    else {
        return Err(Error::InvalidIr(
            "compiler message must use source record storage",
        ));
    };
    let target = fields.get_mut(field.index()).ok_or(Error::InvalidIr(
        "compiler message field ordinal is invalid",
    ))?;
    *target = value;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EffectKey, WorkspaceId};
    use jai_types::{RecordKind, TypeRegistry};

    struct Provider(TypeRegistry);
    impl ProcedureProvider for Provider {
        fn types(&self) -> &dyn TypeView {
            &self.0
        }
        fn procedure(&self, _: ProcedureId) -> ProcedureAvailability<'_> {
            ProcedureAvailability::Missing
        }
    }
    fn enumeration(types: &mut TypeRegistry, repr: IntegerType, count: i128) -> TypeId {
        let ty = types.reserve_enum(repr);
        types
            .define_enum(
                ty,
                (0..count)
                    .map(|n| Integer::checked(repr, n).unwrap())
                    .collect::<Vec<_>>(),
            )
            .unwrap();
        ty
    }
    fn fixture() -> (Provider, CompilerMessageSchema) {
        let mut types = TypeRegistry::new();
        let kind = enumeration(&mut types, IntegerType::U8, 10);
        let phase_kind = enumeration(&mut types, IntegerType::U32, 6);
        let error = enumeration(&mut types, IntegerType::U8, 3);
        let signed = types.scalar(ScalarType::Int(IntegerType::S64));
        let count = types.scalar(ScalarType::Int(IntegerType::S32));
        let message = types.reserve_record(RecordKind::Struct);
        types.define_record(message, [kind, signed]).unwrap();
        let strings = types.slice(types.string()).unwrap();
        let phase = types.reserve_record(RecordKind::Struct);
        types
            .define_record(
                phase,
                [
                    message,
                    phase_kind,
                    types.string(),
                    types.scalar(ScalarType::Bool),
                    count,
                    count,
                    strings,
                    strings,
                    strings,
                    strings,
                ],
            )
            .unwrap();
        let complete = types.reserve_record(RecordKind::Struct);
        types.define_record(complete, [message, error]).unwrap();
        let schema = CompilerMessageSchema {
            message,
            kind: types.field(message, 0).unwrap().id,
            workspace: types.field(message, 1).unwrap().id,
            phase,
            phase_base: types.field(phase, 0).unwrap().id,
            phase_kind: types.field(phase, 1).unwrap().id,
            pending_count: types.field(phase, 5).unwrap().id,
            complete,
            complete_base: types.field(complete, 0).unwrap().id,
            completion_error: types.field(complete, 1).unwrap().id,
        };
        (Provider(types), schema)
    }
    fn pointer(values: Vec<Value>) -> Pointer {
        let [Value::Pointer(pointer)] = values.as_slice() else {
            panic!("actual Message pointer expected")
        };
        pointer.clone()
    }
    fn integer(value: &Value) -> i128 {
        match value {
            Value::Int(n)
            | Value::Enum {
                value: n, ..
            } => n.value(),
            _ => panic!("integer field expected"),
        }
    }
    #[test]
    fn fresh_phase_and_complete_snapshots_preserve_source_base_and_real_fields() {
        let (provider, schema) = fixture();
        let mut vm = Vm::new(&provider, crate::NoEffects, Limits::default()).unwrap();
        let workspace = WorkspaceId::from_raw(27).unwrap();
        let first = pointer(
            vm.materialize_compiler_message(
                schema,
                CompilerEvent::Phase {
                    workspace,
                    phase: CompilerPhase::Typechecked {
                        pending_count: 9,
                    },
                },
            )
            .unwrap(),
        );
        let second = pointer(
            vm.materialize_compiler_message(
                schema,
                CompilerEvent::Complete {
                    workspace,
                    error: CompilerCompletion::CompilationFailed,
                },
            )
            .unwrap(),
        );
        assert_ne!(first, second);
        for (base, concrete, tag, detail) in [
            (&first, schema.phase, 4, 1),
            (&second, schema.complete, 6, 1),
        ] {
            let Value::Record {
                ty,
                fields,
            } = vm.memory.load(&provider.0, base).unwrap()
            else {
                panic!("Message base record expected")
            };
            assert_eq!(ty, schema.message);
            assert_eq!(integer(&fields[0]), tag);
            assert_eq!(integer(&fields[1]), 27);
            let full = vm
                .memory
                .cast_pointer(&provider.0, base, concrete, CastMode::Checked)
                .unwrap();
            let Value::Record {
                fields, ..
            } = vm.memory.load(&provider.0, &full).unwrap()
            else {
                panic!("concrete message expected")
            };
            assert_eq!(integer(&fields[1]), detail);
            if concrete == schema.phase {
                assert_eq!(integer(&fields[5]), 9);
                assert_eq!(
                    vm.materialize_value(&fields[2]).unwrap(),
                    Value::String(vec![])
                );
            }
            assert!(matches!(
                vm.memory.store(
                    &provider.0,
                    &full,
                    Value::Record {
                        ty: concrete,
                        fields
                    }
                ),
                Err(Error::ReadOnlyStorage)
            ));
        }
    }
    struct Response(EffectOutcome);
    impl CompilerEffects for Response {
        fn begin(&mut self) {
        }
        fn request(&mut self, request: CompilerRequest) -> EffectOutcome {
            assert_eq!(request, CompilerRequest::WaitForMessage);
            self.0.clone()
        }
        fn finish(&mut self, _: bool) -> std::result::Result<(), Error> {
            Ok(())
        }
    }
    #[test]
    fn empty_wait_preserves_pending_identity_and_rejects_fake_unit() {
        let (provider, schema) = fixture();
        let key = EffectKey(119);
        let mut vm = Vm::new(
            &provider,
            Response(EffectOutcome::Pending(key)),
            Limits::default(),
        )
        .unwrap();
        assert!(
            matches!(vm.compiler_wait_for_message(schema, &[]), Err(Halt::Pending(Dependency::Effect(actual))) if actual == key)
        );
        vm.effects_mut().0 = EffectOutcome::Ready(CompilerResponse::Unit);
        assert!(matches!(
            vm.compiler_wait_for_message(schema, &[]),
            Err(Halt::Failed(Error::EffectResponse(_)))
        ));
    }
    #[test]
    fn oversized_source_count_and_foreign_field_metadata_are_rejected() {
        let (provider, mut schema) = fixture();
        let mut vm = Vm::new(&provider, crate::NoEffects, Limits::default()).unwrap();
        let event = CompilerEvent::Phase {
            workspace: WorkspaceId::from_raw(1).unwrap(),
            phase: CompilerPhase::Typechecked {
                pending_count: i32::MAX as u32 + 1,
            },
        };
        assert!(matches!(
            vm.materialize_compiler_message(schema, event),
            Err(Halt::Failed(Error::CheckedCast))
        ));
        schema.workspace = schema.phase_kind;
        assert!(
            vm.materialize_compiler_message(
                schema,
                CompilerEvent::Complete {
                    workspace: WorkspaceId::from_raw(1).unwrap(),
                    error: CompilerCompletion::None
                }
            )
            .is_err()
        );
    }

    #[test]
    fn checked_wait_call_dispatches_owned_event_to_actual_pointer_storage() {
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
        let (Provider(mut types), schema) = fixture();
        let result = types.pointer(schema.message).unwrap();
        let signature = types
            .procedure(ProcedureType {
                parameters: vec![].into(),
                results: vec![result].into(),
                return_abi: jai_types::ForeignReturnAbi::Natural,
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
                intrinsic: crate::CompilerIntrinsic::SourceWaitForMessage {
                    schema,
                },
            },
        };
        let effects = Response(EffectOutcome::Ready(CompilerResponse::Message(
            CompilerEvent::Complete {
                workspace: WorkspaceId::from_raw(13).unwrap(),
                error: CompilerCompletion::CompilerShutdown,
            },
        )));
        let mut vm = Vm::new(&provider, effects, Limits::default()).unwrap();
        let Outcome::Complete(values) = vm.execute(id, vec![]).outcome else {
            panic!("checked wait must complete with owned event")
        };
        let base = pointer(values);
        let concrete = vm
            .memory
            .cast_pointer(&provider.types, &base, schema.complete, CastMode::Checked)
            .unwrap();
        let Value::Record {
            fields, ..
        } = vm.memory.load(&provider.types, &concrete).unwrap()
        else {
            panic!("actual completion record expected")
        };
        assert_eq!(integer(&fields[1]), 2);
    }
}
