//! Retain exact VM frames and their open effect checkpoint between drives.
use super::*;

pub(super) struct Request<'a> {
    pub(super) context: &'a Context<'a>,
    pub(super) key: &'a RunCacheKey,
    pub(super) origin: jai_vm::SourceOrigin,
    pub(super) location: SourceSpan,
    pub(super) temporary: Option<&'a Procedure>,
    pub(super) proof: &'a CallbackProof,
}
pub(super) enum Input<'a> {
    Expression(&'a ValueExpr),
    Call(&'a Call),
}
#[derive(Clone)]
enum OwnedInput {
    Expression(ValueExpr),
    Call(Call),
}
impl OwnedInput {
    fn borrowed(&self) -> Input<'_> {
        match self {
            Self::Expression(value) => Input::Expression(value),
            Self::Call(value) => Input::Call(value),
        }
    }
}
pub(super) enum Publication {
    Complete(Vec<ConstantValue>),
    Pending(Vec<Dependency>),
    Failed(jai_vm::Error),
}

pub(super) fn evaluate(
    provider: &(impl ProcedureProvider + ?Sized),
    request: Request<'_>,
    input: Input<'_>,
    results: &[TypeId],
    locals: &crate::local_declarations::LocalDeclarationRegistry,
) -> Publication {
    let context = request.context;
    let owned_input = match &input {
        Input::Expression(value) => OwnedInput::Expression((*value).clone()),
        Input::Call(value) => OwnedInput::Call((*value).clone()),
    };
    if let Some(pending) = context.cache.pending_execution()
        && !context
            .cache
            .continuations
            .borrow()
            .contains_key(request.key)
    {
        return Publication::Pending(pending.dependencies);
    }
    for &ty in results {
        if let Err(error) =
            materializable_type(provider.types(), ty, context.limits.evaluation_depth)
        {
            return match error {
                jai_vm::Error::Type(jai_types::TypeError::Incomplete(ty)) => {
                    Publication::Pending(vec![Dependency::Type(ty)])
                }
                error => Publication::Failed(error),
            };
        }
    }
    let retained = context.cache.continuations.borrow_mut().remove(request.key);
    let resume = retained.is_some();
    let vm = match retained {
        Some(run) => Vm::with_continuation(provider, BorrowedEffects(context.effects), run.state),
        None => new_vm(
            provider,
            BorrowedEffects(context.effects),
            context.limits,
            context.target,
            Some(&context.cache.state),
        ),
    };
    let mut vm = match vm {
        Ok(vm) => vm,
        Err(error) => return Publication::Failed(error),
    };
    let mut progress = if resume {
        vm.resume_resumable()
    } else {
        match input {
            Input::Expression(expression) => {
                vm.start_resumable_expression_owned_at(context.owner, expression, request.origin)
            }
            Input::Call(call) => {
                vm.start_resumable_call_owned_at(context.owner, call, request.origin)
            }
        }
    };
    loop {
        match progress.outcome {
            jai_vm::ResumableOutcome::Suspended(dependencies) => {
                if !request.key.flags.stallable
                    && dependencies.iter().any(|dependency| {
                        matches!(dependency, Dependency::Effect(_) | Dependency::Host(_))
                    })
                {
                    let canceled = vm.cancel_resumable();
                    context.cache.state.replace(Some(vm.into_state()));
                    return match canceled {
                        Ok(()) => Publication::Pending(dependencies),
                        Err(error) => Publication::Failed(error),
                    };
                }
                match context.effects.service_pending(&dependencies) {
                    Ok(true) => progress = vm.resume_resumable(),
                    Ok(false) => {
                        let state = match vm.into_continuation() {
                            Ok(state) => state,
                            Err(error) => return Publication::Failed(error),
                        };
                        context.cache.continuations.borrow_mut().insert(
                            request.key.clone(),
                            SuspendedRun {
                                state,
                                dependencies: dependencies.clone(),
                                location: request.location,
                                input: owned_input,
                                results: results.to_vec(),
                                temporary: request.temporary.cloned(),
                            },
                        );
                        return Publication::Pending(dependencies);
                    }
                    Err(error) => {
                        let canceled = vm.cancel_resumable();
                        context.cache.state.replace(Some(vm.into_state()));
                        return Publication::Failed(canceled.err().unwrap_or(error));
                    }
                }
            }
            jai_vm::ResumableOutcome::AwaitingPublication => {
                let mut constants = vec![];
                let execution = vm.finish_resumable_validated(|vm, values| {
                    CallbackCheck {
                        proof: request.proof,
                        context,
                        locals,
                    }
                    .validate()?;
                    if values.len() != results.len() {
                        return Err(jai_vm::Error::InvalidIr(
                            "#run result count does not match publication request",
                        ));
                    }
                    constants = vm
                        .materialize_values(values)?
                        .into_iter()
                        .zip(results)
                        .map(|(value, &ty)| {
                            materialize_with_runtime_types(
                                provider.types(),
                                value,
                                ty,
                                context.limits.evaluation_depth,
                                &mut |value| vm.runtime_type_constant_value(value),
                            )
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    Ok(())
                });
                context.cache.state.replace(Some(vm.into_state()));
                return match execution.outcome {
                    Outcome::Complete(_) => Publication::Complete(constants),
                    Outcome::Pending(dependencies) => Publication::Pending(dependencies),
                    Outcome::Failed(error) => Publication::Failed(error),
                };
            }
            jai_vm::ResumableOutcome::Failed(error) => {
                context.cache.state.replace(Some(vm.into_state()));
                return Publication::Failed(error);
            }
        }
    }
}

pub(super) fn scalar(publication: Publication, location: SourceSpan) -> RunOutcome {
    match publication {
        Publication::Complete(mut values) => RunOutcome::Complete(values.pop()),
        Publication::Pending(dependencies) => RunOutcome::Pending(dependencies),
        Publication::Failed(error) => failure(location, error),
    }
}

pub(super) struct SuspendedRun {
    pub(super) state: jai_vm::ContinuationState,
    pub(super) dependencies: Vec<Dependency>,
    pub(super) location: SourceSpan,
    input: OwnedInput,
    results: Vec<TypeId>,
    temporary: Option<Procedure>,
}

impl crate::Resolver<'_> {
    pub(super) fn resume_compile_time(
        &mut self,
        key: &RunCacheKey,
        context: &Context<'_>,
        location: SourceSpan,
    ) -> Option<Result<Publication, jai_source::Diagnostic>> {
        if !context.cache.continuations.borrow().contains_key(key) {
            return None;
        }
        let proof = context
            .cache
            .proof(key, context, &self.meta.local_declarations);
        if let Err(error) = proof.check(context, &self.meta.local_declarations) {
            return match error {
                callback_readiness::CallbackFailure::Pending(dependencies) => {
                    context
                        .cache
                        .continuations
                        .borrow_mut()
                        .get_mut(key)
                        .unwrap()
                        .dependencies = dependencies.clone();
                    Some(Ok(Publication::Pending(dependencies)))
                }
                callback_readiness::CallbackFailure::Failed(error) => {
                    context
                        .cache
                        .callback_failure
                        .borrow_mut()
                        .get_or_insert_with(|| error.clone());
                    Some(Err(error))
                }
            };
        }
        let (input, results, temporary, origin) = {
            let runs = context.cache.continuations.borrow();
            let run = runs.get(key)?;
            (
                run.input.clone(),
                run.results.clone(),
                run.temporary.clone(),
                run.state.source_origin().cloned()?,
            )
        };
        let mut signatures = context.signatures.clone();
        signatures.extend(context.generics.borrow().signature_snapshot());
        signatures.extend(self.meta.local_declarations.signature_snapshot());
        let mut procedures = context.procedures.clone();
        procedures.extend(self.meta.local_declarations.ready_snapshot());
        if let Some(procedure) = &temporary {
            signatures.insert(procedure.id, procedure.signature);
            procedures.insert(procedure.id, procedure.clone());
        }
        let bindings = prototypes::snapshot(context, self.meta);
        let globals = match self.meta.external_globals.snapshot(context.globals) {
            Ok(globals) => globals,
            Err(_) => {
                return Some(Ok(Publication::Failed(jai_vm::Error::InvalidIr(
                    "continuation global definitions changed",
                ))));
            }
        };
        let places = self.places.snapshot();
        let provider = ReadyProcedures::new_with_context(
            self.types,
            &procedures,
            &signatures,
            &globals,
            &places,
            context.context,
        )
        .map(|provider| {
            provider
                .with_foreign(&bindings.foreign)
                .with_storage_alignments(&self.meta.storage_alignments)
                .with_pending_global_alignments(context.pending_global_alignments)
                .with_generic_readiness(context.generics)
                .with_local_readiness(&self.meta.local_declarations)
                .with_callback_proof(&proof)
        })
        .and_then(|provider| provider.with_compiler(context.compiler))
        .and_then(|provider| provider.with_runtime(&bindings.runtime))
        .and_then(|provider| provider.with_file_abi(&bindings.file_abi))
        .and_then(|provider| provider.with_heap_abi(&bindings.heap_abi))
        .and_then(|provider| provider.with_process_abi(&bindings.process_abi));
        Some(Ok(match provider {
            Ok(provider) => evaluate(
                &provider,
                Request {
                    context,
                    key,
                    origin,
                    location,
                    temporary: temporary.as_ref(),
                    proof: &proof,
                },
                input.borrowed(),
                &results,
                &self.meta.local_declarations,
            ),
            Err(jai_ir::IrError::Type(jai_types::TypeError::Incomplete(ty))) => {
                Publication::Pending(vec![Dependency::Type(ty)])
            }
            Err(_) => Publication::Failed(jai_vm::Error::InvalidIr(
                "continuation provider failed verification",
            )),
        }))
    }
}

impl Cache {
    pub(crate) fn pending_execution(&self) -> Option<crate::LibraryPending> {
        let continuations = self.continuations.borrow();
        let compiler = self.compiler_continuations.borrow();
        let first = continuations
            .values()
            .map(|run| run.location)
            .chain(compiler.values().map(|run| run.location))
            .min_by_key(|location| {
                (
                    location.source.index(),
                    location.span.start,
                    location.span.end,
                )
            })?;
        let mut dependencies = vec![];
        for run_dependencies in continuations
            .values()
            .map(|run| &run.dependencies)
            .chain(compiler.values().map(|run| &run.dependencies))
        {
            for dependency in run_dependencies {
                if !dependencies.contains(dependency) {
                    dependencies.push(dependency.clone());
                }
            }
        }
        Some(crate::LibraryPending {
            source: None,
            dependencies,
            diagnostic: LocatedDiagnostic {
                location: first,
                message: "#run is suspended with its original VM execution and effect transaction"
                    .into(),
            },
        })
    }

    pub(crate) fn cancel(&self, effects: &dyn EffectService) -> Result<(), jai_vm::Error> {
        let mut first_error = None;
        for (_, run) in self.continuations.borrow_mut().drain() {
            if let Err(error) = run.state.cancel(&mut BorrowedEffects(effects)) {
                first_error.get_or_insert(error);
            }
        }
        for (_, run) in self.compiler_continuations.borrow_mut().drain() {
            if let Err(error) = run.state.cancel(&mut BorrowedEffects(effects)) {
                first_error.get_or_insert(error);
            }
        }
        match first_error {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }
}
