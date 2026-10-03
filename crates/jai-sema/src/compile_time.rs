//! Compile-time execution requests retain their defining namespace and source.
use jai_ir::{
    Call, ConstantKind, ConstantValue, Global, Places, Procedure, ProcedureId, ValueExpr,
};
use jai_modules::FileInstanceId;
use jai_source::{LocatedDiagnostic, SourceSpan};
use jai_syntax::{CompileTimeBody, CompileTimeRun};
use jai_types::{ScalarType, TypeId, TypeKind, TypeView};
use jai_vm::{
    CompilerEffects, Dependency, Limits, Outcome, ProcedureAvailability, ProcedureProvider, Value,
    Vm,
};
use std::cell::RefCell;
use std::collections::HashMap;
mod anonymous;
mod compiler_plan_binding;
mod compiler_values;
pub(crate) use compiler_values::materialize_compiler_captures;
mod compiler_code;
pub(crate) use compiler_code::{CompilerDestination, CompilerRunResult};
mod callback_readiness;
use callback_readiness::{CallbackCheck, CallbackProof};
use std::rc::Rc;
mod owners;
pub(crate) mod prototypes;
mod provider;
mod results;
mod suspension;
pub use provider::ReadyProcedures;

#[derive(Clone, Debug)]
pub struct RunRequest {
    pub file: FileInstanceId,
    pub body: CompileTimeBody,
    pub flags: jai_syntax::RunFlags,
    pub location: SourceSpan,
}

#[derive(Debug)]
pub enum RunOutcome {
    Complete(Option<ConstantValue>),
    Pending(Vec<Dependency>),
    Failed(LocatedDiagnostic),
}

/// One binding attempt records readiness dependencies separately from diagnostics.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum EffectsMode {
    Compiler,
    Isolated,
    CompilerPlanBinding,
}
pub(crate) struct Context<'a> {
    pub foreign: &'a std::collections::HashSet<ProcedureId>,
    pub workspace: jai_vm::WorkspaceId,
    pub owner: ProcedureId,
    pub generics: &'a RefCell<crate::polymorphism::integration::GenericContext>,
    pub context: Option<&'a jai_ir::ContextDefinition>,
    pub procedures: &'a HashMap<ProcedureId, Procedure>,
    pub signatures: &'a HashMap<ProcedureId, TypeId>,
    pub globals: &'a [Global],
    pub pending_global_alignments: &'a std::collections::HashSet<jai_ir::GlobalId>,
    pub places: &'a Places,
    pub source: jai_source::SourceId,
    pub file: FileInstanceId,
    pub target: Option<jai_vm::ByteTarget>,
    pub limits: Limits,
    pub pending: RefCell<Vec<Dependency>>,
    pub cache: &'a Cache,
    pub effects: &'a dyn EffectService,
    pub effect_mode: EffectsMode,
    pub compiler: &'a HashMap<ProcedureId, jai_vm::CompilerProcedure>,
    pub runtime: &'a HashMap<ProcedureId, jai_vm::RuntimeProcedure>,
    pub file_abi: &'a HashMap<ProcedureId, jai_vm::file_abi::FileAbiProcedure>,
    pub process_abi: &'a HashMap<ProcedureId, jai_vm::process_abi::ProcessAbiProcedure>,
    pub heap_abi: &'a HashMap<ProcedureId, jai_vm::heap_abi::HeapAbiProcedure>,
    pub deferred: &'a std::collections::HashSet<jai_source::DeclarationId>,
    pub pending_constants: RefCell<Vec<jai_source::DeclarationId>>,
    pub pending_field_defaults: RefCell<Vec<jai_types::FieldId>>,
}
#[derive(Default)]
pub(crate) struct Cache {
    compiler_values: RefCell<HashMap<RunCacheKey, compiler_code::CompletedCompilerRun>>,
    compiler_continuations: RefCell<HashMap<RunCacheKey, compiler_code::SuspendedCompilerRun>>,
    values: RefCell<HashMap<RunCacheKey, CompletedRun>>,
    results: RefCell<HashMap<RunCacheKey, CompletedResults>>,
    continuations: RefCell<HashMap<RunCacheKey, suspension::SuspendedRun>>,
    anonymous_owners: RefCell<HashMap<RunCacheKey, ProcedureId>>,
    proofs: RefCell<HashMap<RunCacheKey, Rc<CallbackProof>>>,
    callback_failure: RefCell<Option<jai_source::Diagnostic>>,
    state: RefCell<Option<jai_vm::VmState>>,
}
#[derive(Clone)]
struct CompletedRun {
    value: Option<ConstantValue>,
    callback_proof: Rc<CallbackProof>,
}
#[derive(Clone)]
struct CompletedResults {
    values: Vec<ConstantValue>,
    callback_proof: Rc<CallbackProof>,
}
impl Cache {
    pub(crate) fn take_callback_failure(&self) -> Option<jai_source::Diagnostic> {
        self.callback_failure.borrow_mut().take()
    }
    fn completed(
        &self,
        key: &RunCacheKey,
        context: &Context<'_>,
        location: SourceSpan,
        locals: &crate::local_declarations::LocalDeclarationRegistry,
    ) -> Result<Option<CompletedRun>, jai_source::Diagnostic> {
        let Some(completed) = self.values.borrow().get(key).cloned() else {
            return Ok(None);
        };
        completed
            .callback_proof
            .validate(context, locals, location)?;
        Ok(Some(completed))
    }
    fn proof(
        &self,
        key: &RunCacheKey,
        context: &Context<'_>,
        locals: &crate::local_declarations::LocalDeclarationRegistry,
    ) -> Rc<CallbackProof> {
        Rc::clone(
            self.proofs
                .borrow_mut()
                .entry(key.clone())
                .or_insert_with(|| Rc::new(CallbackProof::new(context, locals))),
        )
    }
}
#[derive(Clone, PartialEq, Eq, Hash)]
struct RunCacheKey {
    lexical: crate::metaprogram::RunLexicalKey,
    flags: jai_syntax::RunFlags,
    result_use: Option<Vec<bool>>,
    specialization: Option<jai_modules::SourceSpecializationKey>,
    file: FileInstanceId,
    owner: ProcedureId,
    source: jai_source::SourceId,
    start: usize,
    end: usize,
    destination: Option<TypeId>,
    cast: Option<jai_types::CastMode>,
}
#[derive(Clone, Copy, Default)]
pub(crate) struct EvaluationScope {
    pub owner: Option<ProcedureId>,
    pub target: Option<jai_vm::ByteTarget>,
}
#[derive(Default)]
struct ExecutionState<'a> {
    scope: EvaluationScope,
    state: Option<&'a RefCell<Option<jai_vm::VmState>>>,
    callbacks: Option<CallbackCheck<'a>>,
}
pub(crate) trait EffectService {
    fn set_source_origin(&self, origin: jai_vm::SourceOrigin);
    fn begin(&self);
    fn request(&self, request: jai_vm::CompilerRequest) -> jai_vm::EffectOutcome;
    fn finish(&self, commit: bool) -> Result<(), jai_vm::Error>;
    fn suspend(&self) -> Result<(), jai_vm::Error>;
    fn resume(&self) -> Result<(), jai_vm::Error>;
    fn poll_request(
        &self,
        request: &jai_vm::CompilerRequest,
        key: jai_vm::EffectKey,
    ) -> jai_vm::EffectOutcome;
    fn service_pending(&self, dependencies: &[Dependency]) -> Result<bool, jai_vm::Error>;
    fn host_request(
        &self,
        request: jai_vm::host_effects::HostRequest,
    ) -> jai_vm::host_effects::HostOutcome;
    fn host_file_scope(&self) -> Option<jai_vm::host_effects::FilePathScope>;
    fn poll_host_request(
        &self,
        key: jai_vm::host_effects::HostRequestKey,
    ) -> jai_vm::host_effects::HostOutcome;
}
pub(crate) struct SharedEffects<'a>(RefCell<&'a mut dyn CompilerEffects>);
impl<'a> SharedEffects<'a> {
    pub fn new(effects: &'a mut dyn CompilerEffects) -> Self {
        Self(RefCell::new(effects))
    }
}
impl EffectService for SharedEffects<'_> {
    fn host_request(
        &self,
        request: jai_vm::host_effects::HostRequest,
    ) -> jai_vm::host_effects::HostOutcome {
        self.0.borrow_mut().host_request(request)
    }
    fn host_file_scope(&self) -> Option<jai_vm::host_effects::FilePathScope> {
        self.0.borrow().host_file_scope()
    }
    fn poll_host_request(
        &self,
        key: jai_vm::host_effects::HostRequestKey,
    ) -> jai_vm::host_effects::HostOutcome {
        self.0.borrow_mut().poll_host_request(key)
    }
    fn set_source_origin(&self, origin: jai_vm::SourceOrigin) {
        self.0.borrow_mut().set_source_origin(origin);
    }
    fn begin(&self) {
        self.0.borrow_mut().begin();
    }
    fn request(&self, request: jai_vm::CompilerRequest) -> jai_vm::EffectOutcome {
        self.0.borrow_mut().request(request)
    }
    fn finish(&self, commit: bool) -> Result<(), jai_vm::Error> {
        self.0.borrow_mut().finish(commit)
    }
    fn suspend(&self) -> Result<(), jai_vm::Error> {
        self.0.borrow_mut().suspend()
    }
    fn resume(&self) -> Result<(), jai_vm::Error> {
        self.0.borrow_mut().resume()
    }
    fn poll_request(
        &self,
        request: &jai_vm::CompilerRequest,
        key: jai_vm::EffectKey,
    ) -> jai_vm::EffectOutcome {
        self.0.borrow_mut().poll_request(request, key)
    }
    fn service_pending(&self, dependencies: &[Dependency]) -> Result<bool, jai_vm::Error> {
        self.0.borrow_mut().service_pending(dependencies)
    }
}
struct BorrowedEffects<'a>(&'a dyn EffectService);
impl CompilerEffects for BorrowedEffects<'_> {
    fn host_request(
        &mut self,
        request: jai_vm::host_effects::HostRequest,
    ) -> jai_vm::host_effects::HostOutcome {
        self.0.host_request(request)
    }
    fn host_file_scope(&self) -> Option<jai_vm::host_effects::FilePathScope> {
        self.0.host_file_scope()
    }
    fn poll_host_request(
        &mut self,
        key: jai_vm::host_effects::HostRequestKey,
    ) -> jai_vm::host_effects::HostOutcome {
        self.0.poll_host_request(key)
    }
    fn set_source_origin(&mut self, origin: jai_vm::SourceOrigin) {
        self.0.set_source_origin(origin);
    }
    fn begin(&mut self) {
        self.0.begin();
    }
    fn request(&mut self, request: jai_vm::CompilerRequest) -> jai_vm::EffectOutcome {
        self.0.request(request)
    }
    fn finish(&mut self, commit: bool) -> Result<(), jai_vm::Error> {
        self.0.finish(commit)
    }
    fn suspend(&mut self) -> Result<(), jai_vm::Error> {
        self.0.suspend()
    }
    fn resume(&mut self) -> Result<(), jai_vm::Error> {
        self.0.resume()
    }
    fn poll_request(
        &mut self,
        request: &jai_vm::CompilerRequest,
        key: jai_vm::EffectKey,
    ) -> jai_vm::EffectOutcome {
        self.0.poll_request(request, key)
    }
    fn service_pending(&mut self, dependencies: &[Dependency]) -> Result<bool, jai_vm::Error> {
        self.0.service_pending(dependencies)
    }
}
impl<'a> Context<'a> {
    pub(crate) fn isolated_for_source<'b>(
        &'b self,
        owner: ProcedureId,
        file: FileInstanceId,
        source: jai_source::SourceId,
        cache: &'b Cache,
        effects: &'b dyn EffectService,
    ) -> Context<'b> {
        let mut context: Context<'b> = self.for_source(owner, file, source);
        context.cache = cache;
        context.effects = effects;
        context.effect_mode = EffectsMode::Isolated;
        context
    }
    pub(crate) fn for_source(
        &self,
        owner: ProcedureId,
        file: FileInstanceId,
        source: jai_source::SourceId,
    ) -> Self {
        Self {
            foreign: self.foreign,
            workspace: self.workspace,
            owner,
            generics: self.generics,
            context: self.context,
            procedures: self.procedures,
            signatures: self.signatures,
            globals: self.globals,
            pending_global_alignments: self.pending_global_alignments,
            places: self.places,
            source,
            file,
            target: self.target,
            limits: self.limits,
            pending: RefCell::new(vec![]),
            cache: self.cache,
            effects: self.effects,
            effect_mode: self.effect_mode,
            compiler: self.compiler,
            runtime: self.runtime,
            file_abi: self.file_abi,
            process_abi: self.process_abi,
            heap_abi: self.heap_abi,
            deferred: self.deferred,
            pending_constants: RefCell::new(vec![]),
            pending_field_defaults: RefCell::new(vec![]),
        }
    }

    pub(crate) fn merge_pending_from(&self, child: &Self) {
        let dependencies = child.pending.borrow_mut().drain(..).collect();
        self.record_pending(dependencies);
        let fields: Vec<_> = child
            .pending_field_defaults
            .borrow_mut()
            .drain(..)
            .collect();
        let mut pending_fields = self.pending_field_defaults.borrow_mut();
        for field in fields {
            if !pending_fields.contains(&field) {
                pending_fields.push(field);
            }
        }
        let dependencies: Vec<_> = child.pending_constants.borrow_mut().drain(..).collect();
        let mut pending = self.pending_constants.borrow_mut();
        for dependency in dependencies {
            if !pending.contains(&dependency) {
                pending.push(dependency);
            }
        }
    }

    pub fn record_pending(&self, dependencies: Vec<Dependency>) {
        let mut pending = self.pending.borrow_mut();
        for dependency in dependencies {
            if !pending.contains(&dependency) {
                pending.push(dependency);
            }
        }
    }
}

impl crate::Resolver<'_> {
    pub(crate) fn bind_run_constant(
        &mut self,
        constant: &jai_syntax::ConstantDeclaration,
    ) -> Result<(), jai_source::Diagnostic> {
        let expected = constant
            .ty
            .as_ref()
            .map(|annotation| self.lexical_annotation(annotation, constant.span))
            .transpose()?;
        if let Some(CompilerRunResult::Code(id)) = self.execute_compiler_code_initializer(
            &constant.initializer,
            CompilerDestination::Code(expected),
        )? {
            return self.bind_name(constant.name, crate::Binding::Code(id));
        }
        let run = match &constant.initializer.kind {
            jai_syntax::ExpressionKind::CompileTime(run) => run.clone(),
            _ => CompileTimeRun {
                flags: Default::default(),
                body: CompileTimeBody::Expression(Box::new(constant.initializer.clone())),
            },
        };
        let value = self
            .execute_compile_time(&run, constant.initializer.span, expected)?
            .ok_or_else(|| {
                jai_source::Diagnostic::new(constant.span, "void #run cannot initialize a constant")
            })?;
        let binding = materialized_binding(value, None, constant.span, self.meta)?;
        self.bind_name(constant.name, binding)
    }
    pub(crate) fn resolve_compile_time(
        &mut self,
        run: &CompileTimeRun,
        span: jai_source::Span,
    ) -> Result<crate::Expr, jai_source::Diagnostic> {
        if let Some(CompilerRunResult::Code(id)) =
            self.execute_compiler_code_run(run, span, CompilerDestination::Code(None))?
        {
            return Ok(crate::Expr::Code(id));
        }
        let value = self
            .execute_compile_time(run, span, None)?
            .ok_or_else(|| jai_source::Diagnostic::new(span, "void #run cannot supply a value"))?;
        let ty = value.ty;
        self.typed_value(value.into_expression(), ty, span)
    }
    pub(crate) fn resolve_compile_time_expected(
        &mut self,
        run: &CompileTimeRun,
        span: jai_source::Span,
        expected: TypeId,
    ) -> Result<crate::Expr, jai_source::Diagnostic> {
        if let Some(CompilerRunResult::Code(id)) =
            self.execute_compiler_code_run(run, span, CompilerDestination::Code(Some(expected)))?
        {
            return Ok(crate::Expr::Code(id));
        }
        let value = self
            .execute_compile_time(run, span, Some(expected))?
            .ok_or_else(|| jai_source::Diagnostic::new(span, "void #run cannot supply a value"))?;
        self.typed_value(value.into_expression(), expected, span)
    }
    pub(crate) fn resolve_compile_time_cast(
        &mut self,
        run: &CompileTimeRun,
        span: jai_source::Span,
        target: TypeId,
        mode: jai_types::CastMode,
    ) -> Result<crate::Expr, jai_source::Diagnostic> {
        let value = self
            .execute_compile_time_with_cast(run, span, Some(target), Some(mode))?
            .ok_or_else(|| jai_source::Diagnostic::new(span, "void #run cannot supply a value"))?;
        self.typed_value(value.into_expression(), target, span)
    }
    pub(crate) fn resolve_compile_time_statement(
        &mut self,
        run: &CompileTimeRun,
        span: jai_source::Span,
    ) -> Result<jai_ir::Statement, jai_source::Diagnostic> {
        self.execute_compile_time(run, span, None)?;
        Ok(jai_ir::Statement::Block(jai_ir::Block {
            statements: vec![],
            flow: jai_ir::Flow::FallsThrough,
        }))
    }
    pub(crate) fn execute_compile_time(
        &mut self,
        run: &CompileTimeRun,
        span: jai_source::Span,
        destination: Option<TypeId>,
    ) -> Result<Option<ConstantValue>, jai_source::Diagnostic> {
        self.execute_compile_time_with_cast(run, span, destination, None)
    }
    fn execute_compile_time_with_cast(
        &mut self,
        run: &CompileTimeRun,
        span: jai_source::Span,
        destination: Option<TypeId>,
        cast: Option<jai_types::CastMode>,
    ) -> Result<Option<ConstantValue>, jai_source::Diagnostic> {
        self.check_source_execution(span)?;
        let context = self.compile_time.ok_or_else(|| {
            jai_source::Diagnostic::new(span, "#run requires a checked procedure readiness context")
        })?;
        let source_id = self.debug.source().unwrap_or(context.source);
        let key = RunCacheKey {
            lexical: self.run_lexical_key(span)?,
            flags: run.flags,
            result_use: None,
            specialization: self
                .meta
                .source_specialization_keys
                .get(&self.procedure)
                .cloned(),
            file: context.file,
            owner: context.owner,
            source: source_id,
            start: span.start,
            end: span.end,
            destination,
            cast,
        };
        let location = SourceSpan {
            source: source_id,
            span,
        };
        if let Some(completed) =
            context
                .cache
                .completed(&key, context, location, &self.meta.local_declarations)?
        {
            return Ok(completed.value);
        }
        let proof = context
            .cache
            .proof(&key, context, &self.meta.local_declarations);
        if let Some(publication) = self
            .resume_compile_time(&key, context, location)
            .transpose()?
        {
            return finish_source_run(
                context,
                key,
                location,
                suspension::scalar(publication, location),
                proof,
            );
        }
        // A graph-owned run retains reached source prerequisites even without
        // the stallable flag. Restarting after a late procedure/type read would
        // reissue effects which precede that read. External waits still require
        // the original stallable policy at the suspension boundary.
        let origin = if self.graph_scope.is_some() {
            Some(self.source_run_origin(context.workspace, self.procedure, source_id, span)?)
        } else if run.flags.stallable {
            return Err(jai_source::Diagnostic::at_source(
                location,
                "stallable #run requires a retained source graph",
            ));
        } else {
            None
        };
        let outcome = if let CompileTimeBody::Expression(source) = &run.body {
            if cast.is_none()
                && let Some(expected) = destination
                && let Ok(Some(actual)) = self.describe_argument_type(source)
            {
                let value = self.typed_value(ValueExpr::Zero(actual), actual, span)?;
                self.coerce_value(value, expected, span)
                    .map_err(|error| error.with_fallback_source(source_id))?;
            }
            let expression = match destination.filter(|_| cast.is_none()) {
                Some(ty) => self.expr_expected(source, ty),
                None => self.expr(source),
            }
            .map_err(|error| error.with_fallback_source(source_id))?;
            if let Some(scope) = self.graph_scope
                && let Some(procedure) =
                    scope.initializer_call_dependency(&expression, context, self.meta)
            {
                context.record_pending(vec![Dependency::Procedure(procedure)]);
                return Err(jai_source::Diagnostic::at_source(
                    location,
                    "initializer call is waiting for its checked source body",
                ));
            }
            // Deferred constants may wrap the same source directive while binding it.
            if let Some(completed) =
                context
                    .cache
                    .completed(&key, context, location, &self.meta.local_declarations)?
            {
                return Ok(completed.value);
            }
            if self.graph_scope.is_some() {
                context.effects.set_source_origin(self.source_run_origin(
                    context.workspace,
                    self.procedure,
                    source_id,
                    span,
                )?);
            }
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
            match expression {
                crate::Expr::Void(call) => {
                    if let Some(expected) = destination {
                        return Err(jai_source::Diagnostic::at_source(
                            location,
                            jai_vm::Error::TypeMismatch { expected }.to_string(),
                        ));
                    }
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
                    .map_err(|error| {
                        jai_source::Diagnostic::at_source(location, error.to_string())
                    })?;
                    if let Some(pending) = context.cache.pending_execution()
                        && !run.flags.stallable
                    {
                        context.record_pending(pending.dependencies);
                        return Err(jai_source::Diagnostic::at_source(
                            location,
                            "#run is waiting for the suspended virtual state",
                        ));
                    } else if let Some(origin) = origin.clone() {
                        suspension::scalar(
                            suspension::evaluate(
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
                                &[],
                                &self.meta.local_declarations,
                            ),
                            location,
                        )
                    } else {
                        evaluate_void_state(
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
                                    proof: &proof,
                                    context,
                                    locals: &self.meta.local_declarations,
                                }),
                            },
                        )
                    }
                }
                expression => {
                    let expected = match destination {
                        Some(ty) => ty,
                        None => self
                            .expression_type(&expression, span)
                            .map_err(|error| error.with_fallback_source(source_id))?,
                    };
                    let expression = match cast {
                        Some(mode) => self
                            .cast_operand(expression, expected, mode, span)
                            .map_err(|error| error.with_fallback_source(source_id))?,
                        None => expression,
                    };
                    let expression = self
                        .coerce_value(expression, expected, span)
                        .map_err(|error| error.with_fallback_source(source_id))?;
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
                    .map_err(|error| {
                        jai_source::Diagnostic::at_source(location, error.to_string())
                    })?;
                    if let Some(pending) = context.cache.pending_execution()
                        && !run.flags.stallable
                    {
                        context.record_pending(pending.dependencies);
                        return Err(jai_source::Diagnostic::at_source(
                            location,
                            "#run is waiting for the suspended virtual state",
                        ));
                    } else if let Some(origin) = origin.clone() {
                        suspension::scalar(
                            suspension::evaluate(
                                &provider,
                                suspension::Request {
                                    context,
                                    key: &key,
                                    origin,
                                    location,
                                    temporary: None,
                                    proof: &proof,
                                },
                                suspension::Input::Expression(&expression),
                                &[expected],
                                &self.meta.local_declarations,
                            ),
                            location,
                        )
                    } else {
                        evaluate_state(
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
                                    proof: &proof,
                                    context,
                                    locals: &self.meta.local_declarations,
                                }),
                            },
                        )
                    }
                }
            }
        } else if let Some(pending) = context.cache.pending_execution() {
            context.record_pending(pending.dependencies);
            return Err(jai_source::Diagnostic::at_source(
                location,
                "#run is waiting for the suspended virtual state",
            ));
        } else {
            let existing_owner = context.cache.anonymous_owners.borrow().get(&key).copied();
            let owner = match existing_owner {
                Some(owner) => owner,
                None => {
                    let owner = context.generics.borrow_mut().reserve_local_procedure()?;
                    context
                        .cache
                        .anonymous_owners
                        .borrow_mut()
                        .insert(key.clone(), owner);
                    owner
                }
            };
            let suspension = origin.map(|origin| suspension::Request {
                context,
                key: &key,
                origin,
                location,
                temporary: None,
                proof: &proof,
            });
            self.execute_anonymous_run(
                &run.body,
                location,
                destination,
                cast,
                suspension,
                anonymous::Execution {
                    owner,
                    proof: &proof,
                },
            )
            .map_err(|error| error.with_fallback_source(source_id))?
        };
        proof.validate(context, &self.meta.local_declarations, location)?;
        finish_source_run(context, key, location, outcome, proof)
    }
}

fn finish_source_run(
    context: &Context<'_>,
    key: RunCacheKey,
    location: SourceSpan,
    outcome: RunOutcome,
    proof: Rc<CallbackProof>,
) -> Result<Option<ConstantValue>, jai_source::Diagnostic> {
    match outcome {
        RunOutcome::Complete(value) => {
            context.cache.proofs.borrow_mut().remove(&key);
            context.cache.values.borrow_mut().insert(
                key,
                CompletedRun {
                    value: value.clone(),
                    callback_proof: proof,
                },
            );
            Ok(value)
        }
        RunOutcome::Pending(dependencies) => {
            if !key.flags.stallable
                && dependencies.iter().any(|dependency| {
                    matches!(dependency, Dependency::Effect(_) | Dependency::Host(_))
                })
            {
                return Err(jai_source::Diagnostic::at_source(
                    location,
                    "compiler or host dependencies require #run,stallable",
                ));
            }
            context.record_pending(dependencies);
            Err(jai_source::Diagnostic::at_source(
                location,
                "#run is waiting for checked procedure or type dependencies",
            ))
        }
        RunOutcome::Failed(error) => Err(jai_source::Diagnostic::at_source(
            error.location,
            error.message,
        )),
    }
}

pub(crate) fn materialized_binding(
    value: ConstantValue,
    constraint: Option<ScalarType>,
    span: jai_source::Span,
    meta: &mut crate::reflection::MetaContext,
) -> Result<crate::Binding, jai_source::Diagnostic> {
    let binding = match value.kind {
        ConstantKind::Int(value) => crate::Binding::Constant(crate::ScalarConstant::Int(value)),
        ConstantKind::Float(value) => crate::Binding::Constant(crate::ScalarConstant::Float(value)),
        ConstantKind::Bool(value) => crate::Binding::Constant(crate::ScalarConstant::Bool(value)),
        ConstantKind::Enum(integer) => {
            crate::Binding::Enum(crate::modules::aggregates::EnumConstant {
                ty: value.ty,
                value: integer,
            })
        }
        ConstantKind::RuntimeType(value) => crate::Binding::Type(value.identity().ty()),
        _ => crate::Binding::TypedConstant(meta.intern_constant(value)),
    };
    if let Some(ty) = constraint {
        match binding {
            crate::Binding::Constant(value) => {
                Ok(crate::Binding::Constant(value.coerce(ty, span)?))
            }
            _ => Err(jai_source::Diagnostic::new(
                span,
                "nominal constant cannot implicitly convert to scalar type",
            )),
        }
    } else {
        Ok(binding)
    }
}

pub(crate) fn is_run_constant(expression: &jai_syntax::Expression) -> bool {
    let mut work = vec![expression];
    while let Some(expression) = work.pop() {
        use jai_syntax::ExpressionKind as E;
        match &expression.kind {
            E::CompileTime(_) => return true,
            E::Unary(_, inner) | E::Cast(_, _, inner) | E::TypeCast { value: inner, .. } => {
                work.push(inner)
            }
            E::Binary(_, lhs, rhs) => {
                work.push(lhs);
                work.push(rhs);
            }
            E::Conditional(value) => {
                work.push(&value.condition);
                work.push(&value.then_value);
                if let Some(value) = &value.else_value {
                    work.push(value);
                }
            }
            E::Call(_, args) | E::QualifiedCall(_, args) => {
                work.extend(args.iter().map(|argument| &argument.value))
            }
            _ => {}
        }
    }
    false
}

pub fn evaluate<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
    provider: &P,
    effects: E,
    expression: &ValueExpr,
    expected: TypeId,
    location: SourceSpan,
    limits: Limits,
) -> RunOutcome {
    evaluate_state(
        provider,
        effects,
        expression,
        expected,
        location,
        limits,
        ExecutionState::default(),
    )
}
pub(crate) fn evaluate_for_target<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
    provider: &P,
    effects: E,
    expression: &ValueExpr,
    expected: TypeId,
    location: SourceSpan,
    limits: Limits,
    scope: EvaluationScope,
) -> RunOutcome {
    evaluate_state(
        provider,
        effects,
        expression,
        expected,
        location,
        limits,
        ExecutionState {
            scope,
            state: None,
            callbacks: None,
        },
    )
}
fn evaluate_state<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
    provider: &P,
    effects: E,
    expression: &ValueExpr,
    expected: TypeId,
    location: SourceSpan,
    limits: Limits,
    execution: ExecutionState<'_>,
) -> RunOutcome {
    let ExecutionState {
        scope,
        state,
        callbacks,
    } = execution;
    if expression.type_id(provider.types()) != expected {
        return failure(location, jai_vm::Error::TypeMismatch { expected });
    }
    if let Err(error) = materializable_type(provider.types(), expected, limits.evaluation_depth) {
        return readiness_error(location, error);
    }
    let mut vm = match new_vm(provider, effects, limits, scope.target, state) {
        Ok(vm) => vm,
        Err(error) => return failure(location, error),
    };
    let mut constant = None;
    let validate = |vm: &Vm<'_, P, E>, values: &[Value]| {
        if let Some(check) = callbacks {
            check.validate()?;
        }
        if values.len() != 1 {
            return Err(jai_vm::Error::InvalidIr(
                "#run expression must return exactly one value",
            ));
        }
        let value = vm.materialize_value(&values[0])?;
        constant = Some(materialize_with_runtime_types(
            provider.types(),
            value,
            expected,
            limits.evaluation_depth,
            &mut |value| vm.runtime_type_constant_value(value),
        )?);
        Ok(())
    };
    let outcome = match scope.owner {
        Some(owner) => vm.evaluate_owned_validated(owner, expression, validate),
        None => vm.evaluate_validated(expression, validate),
    }
    .outcome;
    if let Some(state) = state {
        state.replace(Some(vm.into_state()));
    }
    match outcome {
        Outcome::Complete(_) => RunOutcome::Complete(constant),
        Outcome::Pending(dependencies) => RunOutcome::Pending(dependencies),
        Outcome::Failed(error) => failure(location, error),
    }
}

pub fn evaluate_void<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
    provider: &P,
    effects: E,
    call: &Call,
    location: SourceSpan,
    limits: Limits,
) -> RunOutcome {
    evaluate_void_state(
        provider,
        effects,
        call,
        location,
        limits,
        ExecutionState::default(),
    )
}
fn evaluate_void_state<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
    provider: &P,
    effects: E,
    call: &Call,
    location: SourceSpan,
    limits: Limits,
    execution: ExecutionState<'_>,
) -> RunOutcome {
    let ExecutionState {
        scope,
        state,
        callbacks,
    } = execution;
    let signature = match provider.procedure(call.procedure) {
        ProcedureAvailability::Ready(procedure) => procedure.procedure().signature,
        ProcedureAvailability::Compiler(procedure) => procedure.signature,
        ProcedureAvailability::Runtime(procedure) => procedure.signature,
        ProcedureAvailability::FileAbi(procedure) => procedure.signature,
        ProcedureAvailability::HeapAbi(procedure) => procedure.signature,
        ProcedureAvailability::ProcessAbi(procedure) => procedure.signature(),
        ProcedureAvailability::Pending(dependency) => return RunOutcome::Pending(vec![dependency]),
        ProcedureAvailability::Foreign => {
            return failure(
                location,
                jai_vm::Error::UnsupportedForeignProcedure(call.procedure),
            );
        }
        ProcedureAvailability::Missing => {
            return failure(location, jai_vm::Error::MissingProcedure(call.procedure));
        }
        ProcedureAvailability::Failed(error) => return readiness_error(location, error),
    };
    let results = match provider.types().kind(signature) {
        Ok(TypeKind::Procedure(id)) => match provider.types().procedure_type(*id) {
            Ok(signature) => &signature.results,
            Err(error) => return readiness_error(location, error.into()),
        },
        Ok(_) => {
            return failure(
                location,
                jai_vm::Error::InvalidIr("procedure signature is not a procedure type"),
            );
        }
        Err(error) => return readiness_error(location, error.into()),
    };
    if !results.is_empty() {
        return failure(location, "#run statement call must return void");
    }
    let mut vm = match new_vm(provider, effects, limits, scope.target, state) {
        Ok(vm) => vm,
        Err(error) => return failure(location, error),
    };
    let validate = |_: &Vm<'_, P, E>, values: &[Value]| {
        if let Some(check) = callbacks {
            check.validate()?;
        }
        if values.is_empty() {
            Ok(())
        } else {
            Err(jai_vm::Error::InvalidIr(
                "#run statement call must return void",
            ))
        }
    };
    let outcome = match scope.owner {
        Some(owner) => vm.evaluate_call_owned_validated(owner, call, validate),
        None => vm.evaluate_call_validated(call, validate),
    }
    .outcome;
    if let Some(state) = state {
        state.replace(Some(vm.into_state()));
    }
    match outcome {
        Outcome::Complete(values) if values.is_empty() => RunOutcome::Complete(None),
        Outcome::Complete(_) => failure(location, "#run statement call must return void"),
        Outcome::Pending(dependencies) => RunOutcome::Pending(dependencies),
        Outcome::Failed(error) => failure(location, error),
    }
}

fn new_vm<'a, P: ProcedureProvider + ?Sized, E: CompilerEffects>(
    provider: &'a P,
    effects: E,
    limits: Limits,
    target: Option<jai_vm::ByteTarget>,
    state: Option<&RefCell<Option<jai_vm::VmState>>>,
) -> Result<Vm<'a, P, E>, jai_vm::Error> {
    match state.and_then(|state| state.take()) {
        Some(state) => Vm::with_state(provider, effects, limits, state),
        None => Vm::new_with_target(provider, effects, limits, target.unwrap_or_default()),
    }
}

fn failure(location: SourceSpan, message: impl std::fmt::Display) -> RunOutcome {
    RunOutcome::Failed(LocatedDiagnostic {
        location,
        message: message.to_string(),
    })
}
fn readiness_error(location: SourceSpan, error: jai_vm::Error) -> RunOutcome {
    match error {
        jai_vm::Error::Type(jai_types::TypeError::Incomplete(ty)) => {
            RunOutcome::Pending(vec![Dependency::Type(ty)])
        }
        error => failure(location, error),
    }
}
fn materializable_type(
    types: &dyn TypeView,
    ty: TypeId,
    maximum_depth: usize,
) -> Result<(), jai_vm::Error> {
    let mut work = vec![(ty, 0)];
    let mut visited = HashMap::new();
    while let Some((ty, depth)) = work.pop() {
        if depth > maximum_depth.min(256) {
            return Err(jai_vm::Error::Limit(jai_vm::LimitKind::EvaluationDepth));
        }
        if visited.get(&ty).is_some_and(|previous| *previous >= depth) {
            continue;
        }
        visited.insert(ty, depth);
        match types.kind(ty)? {
            TypeKind::Integer(_) | TypeKind::Float(_) | TypeKind::Bool | TypeKind::String => {}
            TypeKind::Type => {
                jai_types::RuntimeTypeSchema::from_view(types)?;
            }
            TypeKind::Procedure(_) => {
                types.procedure_definition(ty)?;
            }
            // Pointer publication is value-dependent: only null/numeric leaves
            // pass conversion; no pointee owner is traversed or manufactured.
            TypeKind::Pointer(pointee) => {
                types.kind(*pointee)?;
            }
            TypeKind::Enum(id) => {
                types.enumeration(*id)?;
            }
            TypeKind::Record(id) => {
                let record = types.record(*id)?;
                work.extend(record.fields.iter().map(|&field| (field, depth + 1)));
            }
            TypeKind::FixedArray { element, .. } => work.push((*element, depth + 1)),
            TypeKind::Distinct(id) => work.push((types.distinct(*id)?.representation, depth + 1)),
            _ => return Err(jai_vm::Error::UnsupportedType(ty)),
        }
    }
    Ok(())
}

/// Runtime addresses cannot escape virtual memory into a native constant.
pub fn materialize(
    types: &dyn TypeView,
    value: Value,
    expected: TypeId,
    maximum_depth: usize,
) -> Result<ConstantValue, jai_vm::Error> {
    materialize_with_runtime_types(types, value, expected, maximum_depth, &mut |_| {
        Err(jai_vm::Error::UnsupportedType(types.meta_type()))
    })
}
fn materialize_with_runtime_types(
    types: &dyn TypeView,
    value: Value,
    expected: TypeId,
    maximum_depth: usize,
    runtime_type: &mut dyn FnMut(&Value) -> Result<jai_ir::RuntimeTypeConstant, jai_vm::Error>,
) -> Result<ConstantValue, jai_vm::Error> {
    value.validate(types, expected, maximum_depth)?;
    fn convert(
        types: &dyn TypeView,
        value: Value,
        ty: TypeId,
        depth: usize,
        maximum: usize,
        runtime_type: &mut dyn FnMut(&Value) -> Result<jai_ir::RuntimeTypeConstant, jai_vm::Error>,
    ) -> Result<ConstantValue, jai_vm::Error> {
        if depth > maximum.min(256) {
            return Err(jai_vm::Error::Limit(jai_vm::LimitKind::EvaluationDepth));
        }
        let kind = match value {
            Value::StoredAggregate(_) => {
                return Err(jai_vm::Error::UnsupportedPointerOperation(
                    "stored aggregate requires checked VM publication materialization",
                ));
            }
            Value::AddressInteger(_) => {
                return Err(jai_vm::Error::UnsupportedPointerOperation(
                    "address-derived integer cannot be published as a native constant",
                ));
            }
            Value::Pointer(pointer) if pointer.is_null() => ConstantKind::Zero,
            Value::Pointer(pointer) if pointer.is_opaque() => {
                let (bits, width) = pointer
                    .opaque_address_bits()
                    .expect("closed numeric pointer origin");
                let integer_type = match width {
                    32 => jai_types::IntegerType::U32,
                    64 => jai_types::IntegerType::U64,
                    _ => {
                        return Err(jai_vm::Error::InvalidIr(
                            "numeric pointer has unsupported target width",
                        ));
                    }
                };
                let recipe = jai_ir::NativePointerConstant::new(
                    ty,
                    jai_types::Integer::wrapping(integer_type, i128::from(bits)),
                    jai_types::CastMode::Unchecked,
                    types,
                )
                .map_err(|error| match error {
                    jai_ir::NativePointerConstantError::Type(error) => jai_vm::Error::Type(error),
                    jai_ir::NativePointerConstantError::InvalidType(expected) => {
                        jai_vm::Error::TypeMismatch { expected }
                    }
                    jai_ir::NativePointerConstantError::CheckedCast => jai_vm::Error::CheckedCast,
                    jai_ir::NativePointerConstantError::UnsupportedWidth(_) => {
                        jai_vm::Error::InvalidIr(
                            "numeric pointer publication has unsupported width",
                        )
                    }
                    jai_ir::NativePointerConstantError::UnsupportedMode(_) => {
                        jai_vm::Error::InvalidIr(
                            "numeric pointer publication has unsupported cast mode",
                        )
                    }
                })?;
                ConstantKind::NativePointer(recipe)
            }
            Value::Int(value) => ConstantKind::Int(value),
            Value::Float(value) => ConstantKind::Float(value),
            Value::Bool(value) => ConstantKind::Bool(value),
            Value::Type { descriptor: None } => ConstantKind::Zero,
            value @ Value::Type {
                descriptor: Some(_),
            } => ConstantKind::RuntimeType(runtime_type(&value)?),
            Value::Procedure {
                signature,
                procedure,
            } if signature == ty => match procedure {
                Some(procedure) => ConstantKind::Procedure(procedure),
                None => ConstantKind::Zero,
            },
            Value::String(value) => ConstantKind::StringBytes(value),
            Value::Enum { value, .. } => ConstantKind::Enum(value),
            Value::Record { fields, .. } => {
                let TypeKind::Record(id) = types.kind(ty)? else {
                    return Err(jai_vm::Error::TypeMismatch { expected: ty });
                };
                let record = types.record(*id)?;
                ConstantKind::Record(
                    fields
                        .into_iter()
                        .zip(record.fields.iter())
                        .map(|(value, &ty)| {
                            convert(types, value, ty, depth + 1, maximum, runtime_type)
                        })
                        .collect::<Result<_, _>>()?,
                )
            }
            Value::Union { field, value, .. } => {
                let field = types.field(ty, field)?;
                ConstantKind::Union {
                    field: field.id,
                    value: Box::new(convert(
                        types,
                        *value,
                        field.ty,
                        depth + 1,
                        maximum,
                        runtime_type,
                    )?),
                }
            }
            Value::Array { elements, .. } => {
                let TypeKind::FixedArray { element, .. } = types.kind(ty)? else {
                    return Err(jai_vm::Error::TypeMismatch { expected: ty });
                };
                ConstantKind::Array(
                    elements
                        .into_iter()
                        .map(|value| {
                            convert(types, value, *element, depth + 1, maximum, runtime_type)
                        })
                        .collect::<Result<_, _>>()?,
                )
            }
            Value::Distinct { value, .. } => {
                let representation = types.distinct_definition(ty)?.representation;
                ConstantKind::Distinct(Box::new(convert(
                    types,
                    *value,
                    representation,
                    depth + 1,
                    maximum,
                    runtime_type,
                )?))
            }
            _ => return Err(jai_vm::Error::UnsupportedType(ty)),
        };
        Ok(ConstantValue { ty, kind })
    }
    convert(types, value, expected, 0, maximum_depth, runtime_type)
}

pub fn scalar_type(types: &dyn TypeView, value: &Value) -> Option<TypeId> {
    match value {
        Value::Int(value) => Some(types.scalar(ScalarType::Int(value.ty()))),
        Value::Bool(_) => Some(types.scalar(ScalarType::Bool)),
        Value::Float(value) => Some(types.float(value.ty())),
        Value::Type { .. } => Some(types.meta_type()),
        Value::Record { ty, .. }
        | Value::Enum { ty, .. }
        | Value::Distinct { ty, .. }
        | Value::Union { ty, .. }
        | Value::Array { ty, .. } => Some(*ty),
        Value::Procedure { signature, .. } => Some(*signature),
        Value::Pointer(pointer) if pointer.is_null() || pointer.is_opaque() => {
            types.lookup(&TypeKind::Pointer(pointer.pointee()))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod opaque_numeric_pointer_publication;
