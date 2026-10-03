use super::*;
use crate::compiler_code_plan::CompilerCodePlan;

impl<'a, P: ProcedureProvider + ?Sized, E: CompilerEffects> Vm<'a, P, E> {
    /// Source-only compiler entry. The plan is checked against this actual VM
    /// provider before its single outer effect transaction begins.
    pub fn start_resumable_compiler_code(
        &mut self,
        source: &CompilerCodePlan,
        ambient_owner: ProcedureId,
        origin: crate::SourceOrigin,
    ) -> ResumableExecution {
        if self.continuation.is_some() {
            return self.progress_failed(Error::InvalidIr("a continuation is already active"));
        }
        if !self.execution_phase().is_compile_time() {
            return self.progress_failed(Error::InvalidIr(
                "compiler Code plans require compile-time execution",
            ));
        }
        let empty = Places::default();
        let controller = source
            .verify(
                self.provider.types(),
                self.provider.signatures(),
                self.provider.globals(),
                self.provider.places().unwrap_or(&empty),
                ambient_owner,
                self.provider.context(),
            )
            .and_then(|checked| CompilerController::compile(checked, self.limits));
        match controller {
            Ok(controller) => {
                self.begin_resumable_root(SessionRoot::Compiler(Box::new(controller)), Some(origin))
            }
            Err(error) => self.progress_failed(error),
        }
    }

    /// No frame or Code selection is converted to a runtime Value/result ABI.
    /// The source validator must materialize selected native captures and check
    /// the immutable quotation plus graph publication before accepting it.
    pub fn finish_resumable_compiler_code<T>(
        &mut self,
        validator: impl FnOnce(&Self, &CompilerCodeSelection<'_>) -> std::result::Result<T, Error>,
    ) -> std::result::Result<T, Error> {
        let session = self
            .continuation
            .as_ref()
            .ok_or(Error::InvalidIr("no continuation is active"))?;
        if !matches!(session.phase, Phase::AwaitingPublication) {
            return Err(Error::InvalidIr(
                "compiler Code has not reached publication",
            ));
        }
        let SessionRoot::Compiler(controller) = &session.root else {
            return Err(Error::InvalidIr("continuation is not a compiler Code plan"));
        };
        let selection = controller
            .selection()
            .ok_or(Error::InvalidIr("compiler Code has no selected quotation"))?;
        let work = controller.retained_cells()?;
        let validation = validator(self, &selection)
            .and_then(|value| {
                self.host_files.require_closed()?;
                Ok(value)
            })
            .and_then(|value| {
                if let Some(processes) = &self.processes {
                    processes
                        .world
                        .require_quiescent(processes.branch.current())
                        .map_err(|error| Error::EffectRejected(error.to_string()))?;
                }
                Ok(value)
            });
        if let Err(halt) = self.flush_publication_work() {
            let error = publication_admission_error(halt);
            self.fail_resumable(error.clone().into());
            return Err(error);
        }
        let value = match validation {
            Ok(value) => value,
            Err(Error::Type(TypeError::Incomplete(ty))) => {
                if let Err(error) = self.effects.suspend() {
                    self.fail_resumable(error.clone().into());
                    return Err(error);
                }
                self.continuation.as_mut().unwrap().phase = Phase::Suspended(Dependency::Type(ty));
                return Err(Error::Type(TypeError::Incomplete(ty)));
            }
            Err(error) => {
                self.fail_resumable(error.clone().into());
                return Err(error);
            }
        };
        if let Err(halt) = self.charge_work(work) {
            let error = match halt {
                Halt::Failed(error) => error,
                Halt::Pending(_) => unreachable!(),
            };
            self.fail_resumable(error.clone().into());
            return Err(error);
        }
        let session = self.continuation.take().unwrap();
        if let Some(origin) = &session.origin {
            self.effects.set_source_origin(origin.clone());
        }
        let mut result = Ok(());
        for pointer in self.root_temporaries.drain(..) {
            if let Err(error) = self.memory.release(&pointer) {
                result = Err(error);
                break;
            }
        }
        if result.is_ok() {
            result = self.effects.finish(true);
        } else {
            let _ = self.effects.finish(false);
        }
        self.current_context = None;
        self.process_branch_execution = false;
        self.root_sequence_temp_bytes = 0;
        self.expression_bindings.clear();
        self.effects.clear_journal();
        if let Err(error) = result {
            self.restore_checkpoint(session.checkpoint);
            return Err(error);
        }
        Ok(value)
    }
}
