//! Publish one checked call's ordered results in one compiler transaction.
use super::*;
use jai_source::{Diagnostic, Span};
use jai_syntax::{ExpressionKind, NamePath};

impl Cache {
    fn completed_results(
        &self,
        key: &RunCacheKey,
        context: &Context<'_>,
        location: SourceSpan,
        locals: &crate::local_declarations::LocalDeclarationRegistry,
    ) -> Result<Option<Vec<ConstantValue>>, Diagnostic> {
        let Some(completed) = self.results.borrow().get(key).cloned() else {
            return Ok(None);
        };
        completed
            .callback_proof
            .validate(context, locals, location)?;
        Ok(Some(completed.values))
    }
}

impl crate::Resolver<'_> {
    pub(crate) fn execute_compile_time_results(
        &mut self,
        run: &CompileTimeRun,
        span: Span,
        used: &[bool],
    ) -> Result<Vec<ConstantValue>, Diagnostic> {
        self.check_source_execution(span)?;
        let context = self.compile_time.ok_or_else(|| {
            Diagnostic::new(span, "#run requires a checked procedure readiness context")
        })?;
        let location = SourceSpan {
            source: self.debug.source().unwrap_or(context.source),
            span,
        };
        let key = RunCacheKey {
            lexical: self.run_lexical_key(span)?,
            flags: run.flags,
            result_use: Some(used.to_vec()),
            specialization: self
                .meta
                .source_specialization_keys
                .get(&self.procedure)
                .cloned(),
            file: context.file,
            owner: context.owner,
            source: location.source,
            start: span.start,
            end: span.end,
            destination: None,
            cast: None,
        };
        if let Some(values) = context.cache.completed_results(
            &key,
            context,
            location,
            &self.meta.local_declarations,
        )? {
            return Ok(values);
        }
        let proof = context
            .cache
            .proof(&key, context, &self.meta.local_declarations);
        if let Some(publication) = self
            .resume_compile_time(&key, context, location)
            .transpose()?
        {
            return finish_results(context, key, location, publication, proof);
        }
        if let Some(pending) = context.cache.pending_execution() {
            context.record_pending(pending.dependencies);
            return Err(Diagnostic::at_source(
                location,
                "#run results are waiting for the suspended virtual state",
            ));
        }
        let CompileTimeBody::Expression(source) = &run.body else {
            return Err(Diagnostic::at_source(
                location,
                "constant result groups require a #run procedure call",
            ));
        };
        let (path, args) = match &source.kind {
            ExpressionKind::Call(name, args) => (
                NamePath {
                    root: *name,
                    members: vec![],
                },
                args,
            ),
            ExpressionKind::QualifiedCall(path, args) => (path.clone(), args),
            _ => {
                return Err(Diagnostic::at_source(
                    location,
                    "constant result groups require a #run procedure call",
                ));
            }
        };
        let result_types = self
            .describe_bound_call_results(&path, args, used, span)
            .map_err(|error| error.with_fallback_source(location.source))?;
        for &ty in &result_types {
            if let Err(error) = materializable_type(self.types, ty, context.limits.evaluation_depth)
            {
                return Err(self.result_publication_error(context, location, error));
            }
        }
        let (signature, call) = self
            .resolve_call_binding(&path, args, span)
            .map_err(|error| error.with_fallback_source(location.source))?;
        if signature.results.len() != used.len()
            || !signature
                .results
                .iter()
                .map(|result| result.ty)
                .eq(result_types.iter().copied())
        {
            return Err(Diagnostic::at_source(
                location,
                "constant result count or types changed during call binding",
            ));
        }
        crate::result_obligations::check_result_use(&signature.results, used, 0, span)?;
        let mut signatures = context.signatures.clone();
        signatures.extend(context.generics.borrow().signature_snapshot());
        signatures.extend(self.meta.local_declarations.signature_snapshot());
        let mut procedures = context.procedures.clone();
        procedures.extend(self.meta.local_declarations.ready_snapshot());
        let bindings = prototypes::snapshot(context, self.meta);
        let globals = self
            .meta
            .external_globals
            .snapshot(context.globals)
            .map_err(|error| error.with_fallback_source(location.source))?;
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
        .and_then(|provider| provider.with_process_abi(&bindings.process_abi))
        .map_err(|error| Diagnostic::at_source(location, error.to_string()))?;
        if self.graph_scope.is_some() {
            context.effects.set_source_origin(self.source_run_origin(
                context.workspace,
                self.procedure,
                location.source,
                span,
            )?);
        }
        let mut constants = vec![];
        let outcome = if run.flags.stallable {
            if self.graph_scope.is_none() {
                return Err(Diagnostic::at_source(
                    location,
                    "stallable #run requires a retained source graph",
                ));
            }
            let origin =
                self.source_run_origin(context.workspace, self.procedure, location.source, span)?;
            match suspension::evaluate(
                &provider,
                suspension::Request {
                    context,
                    key: &key,
                    origin,
                    location,
                    temporary: None,
                    proof: &proof,
                },
                suspension::Input::Call(&call),
                &result_types,
                &self.meta.local_declarations,
            ) {
                suspension::Publication::Complete(values) => {
                    constants = values;
                    Outcome::Complete(vec![])
                }
                suspension::Publication::Pending(dependencies) => Outcome::Pending(dependencies),
                suspension::Publication::Failed(error) => Outcome::Failed(error),
            }
        } else {
            let mut vm = new_vm(
                &provider,
                BorrowedEffects(context.effects),
                context.limits,
                context.target,
                Some(&context.cache.state),
            )
            .map_err(|error| Diagnostic::at_source(location, error.to_string()))?;
            let outcome = vm
                .evaluate_call_owned_validated(context.owner, &call, |vm, values| {
                    CallbackCheck {
                        proof: &proof,
                        context,
                        locals: &self.meta.local_declarations,
                    }
                    .validate()?;
                    if values.len() != result_types.len() {
                        return Err(jai_vm::Error::InvalidIr(
                            "#run result count does not match declaration count",
                        ));
                    }
                    constants = vm
                        .materialize_values(values)?
                        .into_iter()
                        .zip(&result_types)
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
                })
                .outcome;
            context.cache.state.replace(Some(vm.into_state()));
            outcome
        };
        proof.validate(context, &self.meta.local_declarations, location)?;
        let publication = match outcome {
            Outcome::Complete(_) => suspension::Publication::Complete(constants),
            Outcome::Pending(dependencies) => suspension::Publication::Pending(dependencies),
            Outcome::Failed(error) => suspension::Publication::Failed(error),
        };
        finish_results(context, key, location, publication, proof)
    }

    fn result_publication_error(
        &self,
        context: &Context<'_>,
        location: SourceSpan,
        error: jai_vm::Error,
    ) -> Diagnostic {
        if let jai_vm::Error::Type(jai_types::TypeError::Incomplete(ty)) = &error {
            context.record_pending(vec![Dependency::Type(*ty)]);
        }
        Diagnostic::at_source(location, error.to_string())
    }
}

fn finish_results(
    context: &Context<'_>,
    key: RunCacheKey,
    location: SourceSpan,
    publication: suspension::Publication,
    proof: Rc<CallbackProof>,
) -> Result<Vec<ConstantValue>, Diagnostic> {
    match publication {
        suspension::Publication::Complete(values) => {
            context.cache.proofs.borrow_mut().remove(&key);
            context.cache.results.borrow_mut().insert(
                key,
                CompletedResults {
                    values: values.clone(),
                    callback_proof: proof,
                },
            );
            Ok(values)
        }
        suspension::Publication::Pending(dependencies) => {
            if !key.flags.stallable
                && dependencies.iter().any(|dependency| {
                    matches!(dependency, Dependency::Effect(_) | Dependency::Host(_))
                })
            {
                return Err(Diagnostic::at_source(
                    location,
                    "compiler or host dependencies require #run,stallable",
                ));
            }
            context.record_pending(dependencies);
            Err(Diagnostic::at_source(
                location,
                "#run results are waiting for checked dependencies",
            ))
        }
        suspension::Publication::Failed(error) => {
            Err(Diagnostic::at_source(location, error.to_string()))
        }
    }
}
