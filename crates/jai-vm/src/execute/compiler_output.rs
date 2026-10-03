//! Read checked variadic string slices before publishing an owned output request.
use super::*;
impl<P: ProcedureProvider + ?Sized, E: CompilerEffects> Vm<'_, P, E> {
    pub(super) fn compiler_write_strings(&mut self, arguments: &[Value]) -> Result<Vec<Value>> {
        let [Value::Slice { pointer, count, .. }, Value::Bool(error)] = arguments else {
            return Err(Error::InvalidIr(
                "write_strings requires its checked string slice and standard-error flag",
            )
            .into());
        };
        let count = crate::checked_sequence_count(*count)?;
        if count > self.limits.value_cells {
            return Err(Error::Limit(LimitKind::ValueCells).into());
        }
        if !matches!(
            self.provider.types().kind(pointer.pointee())?,
            TypeKind::String
        ) {
            return Err(Error::InvalidIr("write_strings slice must contain source strings").into());
        }
        if count != 0 {
            self.prepare_pointer_layouts(pointer, true)?;
        }
        self.memory
            .validate_slice(self.provider.types(), pointer, count)?;
        let mut bytes = Vec::new();
        for index in 0..count {
            self.step(0)?;
            let pointer = self.memory.offset(
                self.provider.types(),
                pointer,
                isize::try_from(index).map_err(|_| Error::CheckedCast)?,
            )?;
            self.charge_work(
                self.memory
                    .load_work_cost(self.provider.types(), &pointer)?,
            )?;
            let value = self.memory.load(self.provider.types(), &pointer)?;
            let Value::String(string) = self.materialize_value(&value)? else {
                return Err(
                    Error::InvalidIr("write_strings slice contained a non-string value").into(),
                );
            };
            let count = bytes
                .len()
                .checked_add(string.len())
                .filter(|count| *count <= self.limits.value_cells)
                .ok_or(Error::Limit(LimitKind::ValueCells))?;
            let work = u64::try_from(string.len()).map_err(|_| Error::Limit(LimitKind::Fuel))?;
            self.statistics.steps = self
                .statistics
                .steps
                .checked_add(work)
                .filter(|work| *work <= self.limits.fuel)
                .ok_or(Error::Limit(LimitKind::Fuel))?;
            bytes
                .try_reserve(count - bytes.len())
                .map_err(|_| Error::Limit(LimitKind::ValueCells))?;
            bytes.extend_from_slice(&string);
        }
        match self.effects.request(crate::CompilerRequest::WriteOutput {
            stream: if *error {
                crate::CompilerOutputStream::StandardError
            } else {
                crate::CompilerOutputStream::StandardOutput
            },
            bytes,
        }) {
            crate::EffectOutcome::Ready(crate::CompilerResponse::Unit) => Ok(vec![]),
            crate::EffectOutcome::Ready(_) => {
                Err(Error::EffectResponse("write_strings requires a unit response").into())
            }
            crate::EffectOutcome::Pending(key) => Err(Halt::Pending(Dependency::Effect(key))),
            crate::EffectOutcome::Rejected(reason) => Err(Error::EffectRejected(reason).into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CompilerRequest, CompilerResponse, EffectOutcome};
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
    #[derive(Default)]
    struct Output(Vec<CompilerRequest>);
    impl CompilerEffects for Output {
        fn begin(&mut self) {}
        fn request(&mut self, request: CompilerRequest) -> EffectOutcome {
            self.0.push(request);
            EffectOutcome::Ready(CompilerResponse::Unit)
        }
        fn finish(&mut self, _: bool) -> std::result::Result<(), Error> {
            Ok(())
        }
    }

    #[test]
    fn descriptor_copy_work_is_charged_before_output() {
        let mut types = TypeRegistry::new();
        let string = types.string();
        let array = types.fixed_array(string, 1).unwrap();
        let slice = types.slice(string).unwrap();
        let provider = Provider(types);
        for fuel in [5, 100] {
            let mut vm = Vm::new(
                &provider,
                Output::default(),
                Limits {
                    fuel,
                    ..Limits::default()
                },
            )
            .unwrap();
            let backing = vm
                .memory
                .allocate(
                    &provider.0,
                    array,
                    Some(Value::Array {
                        ty: array,
                        elements: vec![Value::String(b"abc".to_vec())],
                    }),
                )
                .unwrap();
            let pointer = vm.memory.index(&provider.0, &backing, 0).unwrap();
            let result = vm.compiler_write_strings(&[
                Value::Slice {
                    ty: slice,
                    pointer,
                    count: 1,
                },
                Value::Bool(false),
            ]);
            if fuel == 5 {
                assert!(matches!(
                    result,
                    Err(Halt::Failed(Error::Limit(LimitKind::Fuel)))
                ));
                assert!(vm.effects().0.is_empty());
                // Pointer-layout admission consumes its bounded traversal work
                // before a descriptor load or any output can occur.
                assert_eq!(vm.statistics.steps, fuel);
            } else {
                assert!(result.is_ok());
                assert_eq!(
                    vm.effects().0,
                    vec![CompilerRequest::WriteOutput {
                        stream: crate::CompilerOutputStream::StandardOutput,
                        bytes: b"abc".to_vec(),
                    }]
                );
            }
        }
    }
}
