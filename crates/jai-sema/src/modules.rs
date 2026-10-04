//! Resolve declaration identities through the defining file's namespace.
use super::ScalarConstant as ConstantValue;
use super::*;
use jai_modules::{FileInstanceId, ModuleGraph};
use jai_source::{DeclarationId, LocatedDiagnostic, SourceSpan};
use jai_types::{FieldId, TypeKind};
use syntax::{FileDeclarationKind, NamePath};
pub(crate) mod aggregates;
mod baked_constants;
mod baked_sources;
mod compile_time;
pub(crate) mod compiler_intrinsics;
mod constants;
mod debug_sources;
mod deprecation;
mod discovery_conditions;
mod insertion_capture;
mod parameter_discovery;
mod parameter_interfaces;
mod parameter_nominals;
pub(crate) mod placeholder_demands;
pub(crate) mod runtime_intrinsics;
pub(crate) mod source_specializations;
mod storage_alignment;
mod using_discovery;
pub use parameter_discovery::{
    ParameterDiscoveryOutcome as DiscoveryParameterOutcome,
    ParameterDiscoveryPending as DiscoveryParameterPending,
};
mod entry;
pub use entry::select_entry;
mod allocator_schema;
mod callback_bindings;
mod context;
mod context_bindings;
mod context_registration;
mod deferred_constants;
mod enum_constants;
mod external_data;
pub(crate) mod field_default_jobs;
mod file_abi_bindings;
pub(crate) mod foreign_libraries;
mod native_library_binding;
pub use native_library_binding::NativeSourceLibraryBinding;
mod global_initializers;
mod polymorphic_defaults;
mod polymorphic_headers;
mod procedure_default_jobs;
mod procedure_headers;
mod procedure_signatures;
mod process_abi_bindings;
mod record_method_headers;
mod reflection;
mod reflection_catalog;
mod run_origin;
pub(crate) mod runtime_defaults;
mod short_lambdas;
pub use file_abi_bindings::FileAbiBindingContext;
pub use process_abi_bindings::ProcessAbiBindingContext;
mod discovery_session;
mod source_lookup_demands;
mod source_preparation;
pub use source_preparation::SourcePreparationPending;
mod insertion_jobs;
mod insertion_queries;
mod prepared_session;
pub use discovery_session::{
    DiscoveryReadiness, PreparedDiscoveryOutcome, PreparedDiscoveryRequests,
    PreparedDiscoverySession,
};
mod program_exports;
pub use prepared_session::{
    LibraryPending, LibraryReadiness, PreparedLibrarySession, SourcePrefixReadiness,
};
mod scope;
mod sequence_constants;
mod target_values;
use aggregates::Nominals;
use constants::Constants;
pub(super) use scope::FileScope;
use scope::{declaration_id, located, path};
#[cfg(test)]
mod tests;

struct ScopedDeclarations<'a> {
    context: Option<crate::context::Schema>,
    graph: &'a ModuleGraph,
    values: HashMap<DeclarationId, Binding>,
    signatures: HashMap<DeclarationId, Signature>,
    source_procedures: procedure_headers::identities::SourceProcedures,
    source_lookup: source_lookup_demands::Requests,
    callable_aliases: HashMap<DeclarationId, Vec<DeclarationId>>,
    generics: std::cell::RefCell<crate::polymorphism::integration::GenericContext>,
    nominals: Nominals<'a>,
    defaults: HashMap<FieldId, jai_ir::ConstantValue>,
}
fn infer_constant_type(
    graph: &ModuleGraph,
    file: FileInstanceId,
    expression: &syntax::Expression,
    types: &mut TypeRegistry,
    nominals: &Nominals<'_>,
    constants: &Constants<'_>,
) -> Result<TypeId, LocatedDiagnostic> {
    match &expression.kind {
        syntax::ExpressionKind::String(_) | syntax::ExpressionKind::HereString(_) => {
            Ok(types.string())
        }
        syntax::ExpressionKind::StructLiteral(literal) => {
            let annotation = literal.ty.as_ref().ok_or_else(|| {
                located(
                    graph,
                    file,
                    Diagnostic::new(expression.span, "record literal requires a contextual type"),
                )
            })?;
            nominals.resolve_type(
                graph,
                file,
                annotation,
                types,
                expression.span,
                &mut |file, source| constants.evaluate(file, source),
            )
        }

        syntax::ExpressionKind::Name(name) => infer_name_constant(
            graph,
            file,
            &path(*name),
            expression,
            types,
            nominals,
            constants,
        ),
        syntax::ExpressionKind::QualifiedName(path) => {
            infer_name_constant(graph, file, path, expression, types, nominals, constants)
        }
        _ => Ok(constants.evaluate(file, expression)?.type_id(types)),
    }
}
fn infer_name_constant(
    graph: &ModuleGraph,
    file: FileInstanceId,
    path: &NamePath,
    expression: &syntax::Expression,
    types: &TypeRegistry,
    nominals: &Nominals<'_>,
    constants: &Constants<'_>,
) -> Result<TypeId, LocatedDiagnostic> {
    if let Ok(jai_modules::Binding::Parameter(id)) = graph.lookup(file, path)
        && let jai_modules::ParameterValue::Enumeration(value) = &graph.parameter(id).unwrap().value
    {
        return nominals
            .declarations
            .get(&value.declaration)
            .copied()
            .ok_or_else(|| {
                located(
                    graph,
                    file,
                    Diagnostic::new(
                        expression.span,
                        "module enum parameter has no resolved nominal declaration",
                    ),
                )
            });
    }
    if let Ok(jai_modules::Binding::Declaration(id)) = graph.lookup(file, path)
        && let Some(ty) = constants.nominal_type(id)
    {
        return Ok(ty);
    }
    if let Ok(jai_modules::Binding::Declaration(id)) = graph.lookup(file, path)
        && let Some(&ty) = nominals.value_types.get(&id)
    {
        return Ok(ty);
    }
    if let Some(ty) = nominals
        .source_enum_member_type(graph, file, path, expression.span)
        .map_err(|error| located(graph, file, error))?
    {
        return Ok(ty);
    }
    if let Some(member) = nominals
        .enum_member(graph, file, path, expression.span)
        .map_err(|error| located(graph, file, error))?
    {
        return Ok(member.ty);
    }
    Ok(constants.evaluate(file, expression)?.type_id(types))
}
/// Type-check all reachable declarations without inventing a library entry point.
pub fn resolve_library(graph: &ModuleGraph) -> Result<Library, LocatedDiagnostic> {
    resolve_library_with_options(
        graph,
        &crate::ResolveOptions::default(),
        &mut jai_vm::NoEffects,
    )
}
/// Resolve with an explicit target and transactional compiler effects.
pub fn resolve_library_with_options(
    graph: &ModuleGraph,
    options: &crate::ResolveOptions,
    effects: &mut dyn jai_vm::CompilerEffects,
) -> Result<Library, LocatedDiagnostic> {
    match resolve_prepared(graph, options, effects, None)? {
        PreparedResolution::Library(library) => Ok(*library),
        PreparedResolution::Conditions(_) => {
            unreachable!("library resolution has no discovery requests")
        }
        PreparedResolution::Parameters(_) => {
            unreachable!("library resolution has no parameter requests")
        }
        PreparedResolution::Using(_) => unreachable!("resolution has no using requests"),
        PreparedResolution::Cases(_) => unreachable!("library resolution has no case requests"),
    }
}

/// Evaluate pending source conditions in their retained defining source scopes.
pub fn resolve_discovery_conditions(
    graph: &ModuleGraph,
    requests: &[jai_modules::DeferredCondition],
    options: &crate::ResolveOptions,
    effects: &mut dyn jai_vm::CompilerEffects,
) -> Result<DiscoveryConditionOutcome, LocatedDiagnostic> {
    match resolve_prepared(
        graph,
        options,
        effects,
        Some(PreparedDiscovery::Conditions(requests)),
    )? {
        PreparedResolution::Conditions(outcome) => Ok(outcome),
        PreparedResolution::Library(_) => {
            unreachable!("condition resolution does not freeze a library")
        }
        PreparedResolution::Parameters(_) => {
            unreachable!("condition resolution has no parameter requests")
        }
        PreparedResolution::Using(_) => unreachable!("resolution has no using requests"),
        PreparedResolution::Cases(_) => unreachable!("condition resolution has no case requests"),
    }
}

/// Resolve original case selectors once in their retained, typed source contexts.
pub fn resolve_discovery_cases(
    graph: &ModuleGraph,
    requests: &[jai_modules::DeferredCase],
    options: &crate::ResolveOptions,
    effects: &mut dyn jai_vm::CompilerEffects,
) -> Result<DiscoveryCaseOutcome, LocatedDiagnostic> {
    match resolve_prepared(
        graph,
        options,
        effects,
        Some(PreparedDiscovery::Cases(requests)),
    )? {
        PreparedResolution::Cases(outcome) => Ok(outcome),
        _ => unreachable!("case resolution returns canonical case choices"),
    }
}
pub type InsertionAdmissionCallback<'a> = dyn Fn(
        jai_modules::InsertionRequestId,
        &jai_modules::DeclarationInsertionCode,
    ) -> Result<jai_modules::InsertionAdmission, jai_modules::InsertionPublicationError>
    + 'a;

#[derive(Debug)]
pub struct DiscoveryInsertionOutcome {
    pub specializations: Vec<jai_modules::SourceSpecializationKey>,
    pub decisions: Vec<DiscoveryInsertionDecision>,
    pub pending: Vec<DiscoveryInsertionPending>,
}
#[derive(Clone, Debug)]
pub struct DiscoveryInsertionDecision {
    pub request: jai_modules::InsertionRequestId,
    pub code: jai_modules::DeclarationInsertionCode,
    pub admission: jai_modules::InsertionAdmission,
}
#[derive(Clone, Debug)]
pub struct DiscoveryInsertionPending {
    pub request: jai_modules::InsertionRequestId,
    pub diagnostic: LocatedDiagnostic,
}

#[derive(Debug)]
pub struct DiscoveryCaseOutcome {
    pub decisions: Vec<(jai_modules::CaseRequestId, syntax::CompileTimeCaseChoice)>,
    pub pending: Vec<DiscoveryCasePending>,
    pub specializations: Vec<jai_modules::SourceSpecializationKey>,
}
#[derive(Debug)]
pub struct DiscoveryCasePending {
    pub request: jai_modules::CaseRequestId,
    pub diagnostic: LocatedDiagnostic,
    pub procedures: Vec<jai_vm::Dependency>,
    pub constants: Vec<DeclarationId>,
}

pub fn resolve_discovery_parameters(
    graph: &ModuleGraph,
    requests: &[jai_modules::DeferredParameter],
    options: &crate::ResolveOptions,
    effects: &mut dyn jai_vm::CompilerEffects,
) -> Result<DiscoveryParameterOutcome, LocatedDiagnostic> {
    match resolve_prepared(
        graph,
        options,
        effects,
        Some(PreparedDiscovery::Parameters(requests)),
    )? {
        PreparedResolution::Parameters(outcome) => Ok(outcome),
        _ => unreachable!("parameter resolution returns source parameter decisions"),
    }
}

#[derive(Debug)]
pub struct DiscoveryConditionOutcome {
    pub specializations: Vec<jai_modules::SourceSpecializationKey>,
    pub decisions: Vec<(jai_modules::ConditionRequestId, bool)>,
    pub pending: Vec<DiscoveryConditionPending>,
}

#[derive(Debug)]
pub struct DiscoveryConditionPending {
    pub request: jai_modules::ConditionRequestId,
    pub diagnostic: LocatedDiagnostic,
    pub procedures: Vec<jai_vm::Dependency>,
    pub constants: Vec<DeclarationId>,
}

#[derive(Debug)]
pub struct DiscoveryUsingOutcome {
    pub decisions: Vec<(jai_modules::UsingRequestId, jai_modules::FileUsingDecision)>,
    pub pending: Vec<DiscoveryUsingPending>,
    pub specializations: Vec<jai_modules::SourceSpecializationKey>,
}
#[derive(Debug)]
pub struct DiscoveryUsingPending {
    pub request: jai_modules::UsingRequestId,
    pub diagnostic: LocatedDiagnostic,
    pub procedures: Vec<jai_vm::Dependency>,
    pub constants: Vec<DeclarationId>,
}
pub fn resolve_discovery_using(
    graph: &ModuleGraph,
    requests: &[jai_modules::FileUsingRequest],
    options: &crate::ResolveOptions,
    effects: &mut dyn jai_vm::CompilerEffects,
) -> Result<DiscoveryUsingOutcome, LocatedDiagnostic> {
    match resolve_prepared(
        graph,
        options,
        effects,
        Some(PreparedDiscovery::Using(requests)),
    )? {
        PreparedResolution::Using(outcome) => Ok(outcome),
        _ => unreachable!("using resolution returns source publication decisions"),
    }
}
enum PreparedResolution {
    Using(DiscoveryUsingOutcome),
    Library(Box<Library>),
    Conditions(DiscoveryConditionOutcome),
    Cases(DiscoveryCaseOutcome),
    Parameters(DiscoveryParameterOutcome),
}

#[derive(Clone, Copy)]
enum PreparedDiscovery<'a> {
    Insertions(&'a [jai_modules::DeclarationInsertionRequest]),
    Using(&'a [jai_modules::FileUsingRequest]),
    Conditions(&'a [jai_modules::DeferredCondition]),
    Cases(&'a [jai_modules::DeferredCase]),
    Parameters(&'a [jai_modules::DeferredParameter]),
}

fn resolve_prepared(
    graph: &ModuleGraph,
    options: &crate::ResolveOptions,
    effects: &mut dyn jai_vm::CompilerEffects,
    discovery: Option<PreparedDiscovery<'_>>,
) -> Result<PreparedResolution, LocatedDiagnostic> {
    let mut phase = match prepared_session::prepare(graph, options, discovery)? {
        prepared_session::PreparedStart::Phase(phase) => phase,
        prepared_session::PreparedStart::Parameters(outcome) => {
            return Ok(PreparedResolution::Parameters(outcome));
        }
    };
    let effects = crate::compile_time::SharedEffects::new(effects);
    let mut discovery_jobs = match discovery {
        Some(PreparedDiscovery::Insertions(requests)) => {
            Some(discovery_conditions::Jobs::new_insertions(requests))
        }
        Some(PreparedDiscovery::Using(requests)) => {
            Some(discovery_conditions::Jobs::new_using(requests))
        }
        Some(PreparedDiscovery::Conditions(requests)) => {
            Some(discovery_conditions::Jobs::new(requests))
        }
        Some(PreparedDiscovery::Cases(requests)) => {
            Some(discovery_conditions::Jobs::new_cases(requests))
        }
        _ => None,
    };
    let mut worklist = None;
    let progress = phase.drive_bindings(&effects, discovery_jobs.as_mut(), None, &mut worklist);
    let progress = match progress {
        Ok(progress) => progress,
        Err(mut diagnostic) => {
            if let Some(worklist) = worklist.take()
                && let Err(cause) = worklist.cancel(&effects)
            {
                diagnostic
                    .message
                    .push_str(&format!("; cancellation failed: {cause}"));
            }
            return Err(diagnostic);
        }
    };
    let procedures = match progress {
        compile_time::BindingProgress::Complete(procedures) => procedures,
        compile_time::BindingProgress::Pending(pending) => {
            if let Some(worklist) = worklist {
                worklist
                    .cancel(&effects)
                    .map_err(|error| LocatedDiagnostic {
                        location: pending.diagnostic.location,
                        message: error.to_string(),
                    })?;
            }
            return Err(pending.diagnostic);
        }
        compile_time::BindingProgress::SourceRunReady
        | compile_time::BindingProgress::SourceRunsReady
        | compile_time::BindingProgress::TypesReady
        | compile_time::BindingProgress::HeadersReady
        | compile_time::BindingProgress::InitializersReady => {
            unreachable!("phase drives header readiness internally")
        }
    };
    if let Some(jobs) = discovery_jobs {
        if matches!(discovery, Some(PreparedDiscovery::Using(_))) {
            return Ok(PreparedResolution::Using(DiscoveryUsingOutcome {
                decisions: jobs.using_decisions,
                pending: jobs.using_pending,
                specializations: phase.take_specializations(),
            }));
        }
        if matches!(discovery, Some(PreparedDiscovery::Cases(_))) {
            return Ok(PreparedResolution::Cases(DiscoveryCaseOutcome {
                decisions: jobs.case_decisions,
                pending: jobs.case_pending,
                specializations: phase.take_specializations(),
            }));
        }
        return Ok(PreparedResolution::Conditions(DiscoveryConditionOutcome {
            specializations: phase.take_specializations(),
            decisions: jobs.decisions,
            pending: jobs.pending,
        }));
    }
    phase
        .into_library(procedures)
        .map(|library| PreparedResolution::Library(Box::new(library)))
}
/// Resolve an executable whose main declaration belongs to the application module.
pub fn resolve_graph(graph: &ModuleGraph) -> Result<Program, LocatedDiagnostic> {
    resolve_graph_with_options(
        graph,
        &crate::ResolveOptions::default(),
        &mut jai_vm::NoEffects,
    )
}
/// Resolve an executable with the same target/effect policy as its library.
pub fn resolve_graph_with_options(
    graph: &ModuleGraph,
    options: &crate::ResolveOptions,
    effects: &mut dyn jai_vm::CompilerEffects,
) -> Result<Program, LocatedDiagnostic> {
    let library = resolve_library_with_options(graph, options, effects)?;
    let root_file = graph.module(graph.root()).unwrap().entry();
    let entry = select_entry(graph, &library)?.ok_or_else(|| {
        located(
            graph,
            root_file,
            Diagnostic::new(Span::default(), "no application main procedure"),
        )
    })?;
    library.into_program(entry).map_err(|error| {
        located(
            graph,
            root_file,
            Diagnostic::new(Span::default(), error.to_string()),
        )
    })
}

fn hydrate_constants(
    evaluator: &mut aggregates::Defaults<'_, '_>,
    declarations: &ScopedDeclarations<'_>,
    meta: &crate::reflection::MetaContext,
) {
    for (&declaration, signature) in &declarations.signatures {
        evaluator.named.insert(
            declaration,
            jai_ir::ConstantValue {
                ty: signature.ty,
                kind: jai_ir::ConstantKind::Procedure(signature.id),
            },
        );
    }
    for (&declaration, binding) in &declarations.values {
        if let Binding::Enum(value) = binding {
            evaluator.named.insert(
                declaration,
                jai_ir::ConstantValue {
                    ty: value.ty,
                    kind: jai_ir::ConstantKind::Enum(value.value),
                },
            );
        }
        if let Binding::TypedConstant(id) = binding
            && let Some(value) = meta.constant(*id)
        {
            evaluator.named.insert(declaration, value.clone());
        }
    }
}
