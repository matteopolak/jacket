//! Bind anonymous run bodies as checked, temporary procedures.
use super::*;
use crate::{Resolver, ResultSignature};
use jai_ir::Flow;
use jai_types::{CallingConvention, ContextMode, ProcedureType, TypeKind, Variadic};

pub(super) struct Execution<'a> {
    pub(super) owner: ProcedureId,
    pub(super) proof: &'a CallbackProof,
}

struct AnonymousProvider<'a> {
    ready: ReadyProcedures<'a>,
    procedure: &'a Procedure,
}
impl ProcedureProvider for AnonymousProvider<'_> {
    fn context(&self) -> Option<&jai_ir::ContextDefinition> {
        self.ready.context()
    }
    fn types(&self) -> &dyn TypeView {
        self.ready.types()
    }
    fn signatures(&self) -> &HashMap<ProcedureId, TypeId> {
        self.ready.signatures()
    }
    fn globals(&self) -> &[Global] {
        self.ready.globals()
    }
    fn global_alignment_pending(&self, id: jai_ir::GlobalId) -> bool {
        self.ready.global_alignment_pending(id)
    }
    fn storage_alignments(&self) -> Option<&jai_ir::StorageAlignments> {
        self.ready.storage_alignments()
    }
    fn places(&self) -> Option<&Places> {
        self.ready.places()
    }
    fn procedure(&self, id: ProcedureId) -> ProcedureAvailability<'_> {
        if id == self.procedure.id {
            match jai_ir::verify_procedure_with_context(
                self.ready.types,
                self.procedure,
                self.ready.signatures,
                self.ready.globals,
                self.ready.places,
                self.ready.context,
            ) {
                Ok(checked) => ProcedureAvailability::Ready(checked),
                Err(jai_ir::IrError::Type(jai_types::TypeError::Incomplete(ty))) => {
                    ProcedureAvailability::Pending(Dependency::Type(ty))
                }
                Err(_) => ProcedureAvailability::Failed(jai_vm::Error::InvalidIr(
                    "anonymous #run body failed verification",
                )),
            }
        } else {
            self.ready.procedure(id)
        }
    }
}

impl Resolver<'_> {
    pub(super) fn execute_anonymous_run(
        &mut self,
        source: &CompileTimeBody,
        location: SourceSpan,
        destination: Option<TypeId>,
        cast: Option<jai_types::CastMode>,
        suspension: Option<suspension::Request<'_>>,
        execution: Execution<'_>,
    ) -> Result<RunOutcome, jai_source::Diagnostic> {
        let Execution {
            owner: id,
            proof,
        } = execution;
        let context = self
            .compile_time
            .expect("run context was checked before body binding");
        let (result, body) = match source {
            CompileTimeBody::Block(body) => (None, body),
            CompileTimeBody::Procedure {
                result,
                body,
            } => {
                let ty = self.reflected_type_syntax(result, location.span)?;
                (!matches!(self.types.kind(ty), Ok(TypeKind::Void)))
                    .then_some(ty)
                    .map_or((None, body), |ty| (Some(ty), body))
            }
            CompileTimeBody::Expression(_) => unreachable!(),
        };
        if let Some(expected) = destination {
            let ty = result.ok_or_else(|| {
                jai_source::Diagnostic::at_source(location, "void #run cannot supply a value")
            })?;
            let value = self.typed_value(ValueExpr::Zero(ty), ty, location.span)?;
            let value = match cast {
                Some(mode) => self.cast_operand(value, expected, mode, location.span)?,
                None => value,
            };
            self.coerce_value(value, expected, location.span)?;
        }
        if let Some(ty) = result
            && let Err(error) = materializable_type(self.types, ty, context.limits.evaluation_depth)
        {
            return Ok(readiness_error(location, error));
        }
        let mut signatures = context.signatures.clone();
        signatures.extend(context.generics.borrow().signature_snapshot());
        let signature = self
            .types
            .procedure(ProcedureType {
                parameters: Box::new([]),
                results: result.into_iter().collect(),
                return_abi: jai_types::ForeignReturnAbi::Natural,
                convention: CallingConvention::Jai,
                context: ContextMode::Implicit,
                variadic: Variadic::None,
            })
            .map_err(|error| jai_source::Diagnostic::at_source(location, error.to_string()))?;
        let results: Vec<_> = result
            .into_iter()
            .map(|ty| ResultSignature {
                usage: jai_syntax::ResultUsage::Optional,
                name: None,
                ty,
                default: None,
            })
            .collect();
        let mut scopes = self.scopes.clone();
        scopes.push(HashMap::new());
        let debug_policy = self.debug.policy();
        let body_context = context.for_source(id, context.file, location.source);
        let procedure = (|| {
            let mut resolver = Resolver {
                expression_owner: Some(id),
                debug: crate::debug_capture::Capture::new(self.debug.source()),
                checks: self.checks,
                local_scopes: self.local_scopes.for_procedure(id),
                context: self.context,
                context_available: true,
                meta: self.meta,
                graph_scope: self.graph_scope,
                compile_time: Some(&body_context),
                target_layout: self.target_layout,
                procedure: id,
                types: self.types,
                places: self.places,
                signatures: self.signatures,
                symbols: self.symbols,
                scopes,
                globals: self.globals,
                locals: vec![],
                span: location.span,
                results: &results,
                loops: vec![],
                next_loop: 0,
                active_push: None,
                next_push: 0,
                cleanups: vec![],
                deferred_scopes: vec![],
                cleanup_context: None,
            };
            resolver.debug.enter_policy(debug_policy);
            let body = resolver.block(body, false)?;
            if !results.is_empty() && body.flow != Flow::Terminates {
                return Err(jai_source::Diagnostic::at_source(
                    location,
                    "value-returning #run body may reach its end",
                ));
            }
            Ok(Procedure {
                id,
                signature,
                parameters: vec![],
                locals: resolver.locals,
                body,
                cleanups: resolver.cleanups,
            })
        })();
        context.merge_pending_from(&body_context);
        let alignments = self.meta.storage_alignments.clone();
        self.meta.storage_alignments.clear_procedure(id);
        let procedure = procedure?;
        let call = Call::new(id, vec![]);
        let result_expression = if let Some(ty) = result {
            let expected = destination.unwrap_or(ty);
            let value = self.typed_value(
                ValueExpr::Call {
                    call: call.clone(),
                    ty,
                },
                ty,
                location.span,
            )?;
            let value = match cast {
                Some(mode) => self.cast_operand(value, expected, mode, location.span)?,
                None => value,
            };
            Some((self.coerce_value(value, expected, location.span)?, expected))
        } else {
            None
        };
        signatures.extend(context.generics.borrow().signature_snapshot());
        signatures.extend(self.meta.local_declarations.signature_snapshot());
        signatures.insert(id, signature);
        let mut procedures = context.procedures.clone();
        procedures.extend(self.meta.local_declarations.ready_snapshot());
        let bindings = prototypes::snapshot(context, self.meta);
        let globals = self
            .meta
            .external_globals
            .snapshot(context.globals)
            .map_err(|error| error.with_fallback_source(location.source))?;
        let snapshot = jai_ir::SourceProcedurePlaces::new(self.places.snapshot());
        self.remember_anonymous_owner(&procedure, &signatures, &globals, &snapshot, location)?;
        let ready = ReadyProcedures::new_with_context(
            self.types,
            &procedures,
            &signatures,
            &globals,
            snapshot.places(),
            context.context,
        )
        .map(|provider| {
            provider
                .with_foreign(&bindings.foreign)
                .with_storage_alignments(&alignments)
                .with_pending_global_alignments(context.pending_global_alignments)
                .with_generic_readiness(context.generics)
                .with_local_readiness(&self.meta.local_declarations)
                .with_callback_proof(proof)
        })
        .and_then(|provider| provider.with_compiler(context.compiler))
        .and_then(|provider| provider.with_runtime(&bindings.runtime))
        .and_then(|provider| provider.with_file_abi(&bindings.file_abi))
        .and_then(|provider| provider.with_heap_abi(&bindings.heap_abi))
        .and_then(|provider| provider.with_process_abi(&bindings.process_abi))
        .map_err(|error| jai_source::Diagnostic::at_source(location, error.to_string()))?;
        let provider = AnonymousProvider {
            ready,
            procedure: &procedure,
        };
        if self.graph_scope.is_some() {
            context.effects.set_source_origin(self.source_run_origin(
                context.workspace,
                self.procedure,
                location.source,
                location.span,
            )?);
        }
        if let Some(mut request) = suspension {
            request.temporary = Some(&procedure);
            let location = request.location;
            let publication = match &result_expression {
                Some((expression, expected)) => suspension::evaluate(
                    &provider,
                    request,
                    suspension::Input::Expression(expression),
                    &[*expected],
                    &self.meta.local_declarations,
                ),
                None => suspension::evaluate(
                    &provider,
                    request,
                    suspension::Input::Call(&call),
                    &[],
                    &self.meta.local_declarations,
                ),
            };
            return Ok(suspension::scalar(publication, location));
        }
        Ok(match result_expression {
            Some((expression, expected)) => evaluate_state(
                &provider,
                BorrowedEffects(context.effects),
                &expression,
                expected,
                location,
                context.limits,
                ExecutionState {
                    scope: EvaluationScope {
                        owner: Some(context.owner),
                        target: context.target,
                    },
                    state: Some(&context.cache.state),
                    callbacks: Some(CallbackCheck {
                        proof,
                        context,
                        locals: &self.meta.local_declarations,
                    }),
                },
            ),
            None => evaluate_void_state(
                &provider,
                BorrowedEffects(context.effects),
                &call,
                location,
                context.limits,
                ExecutionState {
                    scope: EvaluationScope {
                        owner: Some(context.owner),
                        target: context.target,
                    },
                    state: Some(&context.cache.state),
                    callbacks: Some(CallbackCheck {
                        proof,
                        context,
                        locals: &self.meta.local_declarations,
                    }),
                },
            ),
        })
    }
}
