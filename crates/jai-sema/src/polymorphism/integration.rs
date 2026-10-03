//! Connect pure overload matches to the shared typed procedure work queue.
use super::{BakedValue, ProcedureTemplate, SpecializationId, Specializations, Substitution};
use crate::overloads::{
    ArgumentInfo, ArgumentType, Candidate, CandidateVariadic, ConstantArgument, Match, Parameter,
    TypePattern,
};
use crate::{ParameterDefault, ParameterSignature, ResultSignature, Signature};
use jai_ir::{ConstantValue, ProcedureId, ProcedurePrototype};
use jai_modules::{Binding, FileInstanceId, ModuleGraph};
use jai_source::{DeclarationId, Diagnostic, Span};
use jai_syntax::{ExpressionKind, FileDeclarationKind, NamePath};
use jai_types::{
    CallingConvention, ContextMode, ProcedureType, TypeId, TypeRegistry, TypeView, Variadic,
};
use std::collections::{HashMap, HashSet};
mod callable_policies;
mod callbacks;
mod modifier_jobs;
mod modifiers;
pub(crate) use modifier_jobs::ModifierReadiness;
pub(crate) use modifier_jobs::{ModifierSource, build_modifier_source, procedure_modifier_source};
pub(crate) use modifiers::ModifierExecution;

pub(crate) struct TemplateDefinition {
    pub template: ProcedureTemplate,
    pub file: FileInstanceId,
    pub span: Span,
    pub convention: CallingConvention,
    pub context: ContextMode,
}

/// An owned request, so binding a body does not retain a RefCell borrow.
pub(crate) struct GenericBody {
    pub specialization: SpecializationId,
    pub declaration: DeclarationId,
    pub file: FileInstanceId,
    pub signature: Signature,
    pub substitution: Substitution,
}

/// All templates belong to the graph; only selected calls allocate procedure IDs.
/// No source or file-scope reference is stored in this context.
pub(crate) struct GenericContext {
    isolated_procedures: HashSet<ProcedureId>,
    modifiers: modifier_jobs::ModifierJobs,
    templates: HashMap<DeclarationId, TemplateDefinition>,
    work: Specializations,
    signatures: HashMap<ProcedureId, Signature>,
    prototype_templates: HashSet<DeclarationId>,
    prototypes: HashMap<ProcedureId, ProcedurePrototype>,
}
impl GenericContext {
    pub(crate) fn new(first_procedure: usize) -> Self {
        Self {
            isolated_procedures: HashSet::new(),
            modifiers: modifier_jobs::ModifierJobs::default(),
            templates: HashMap::new(),
            work: Specializations::new(first_procedure),
            signatures: HashMap::new(),
            prototype_templates: HashSet::new(),
            prototypes: HashMap::new(),
        }
    }
    pub(crate) fn register(&mut self, definition: TemplateDefinition) {
        self.templates
            .insert(definition.template.candidate.declaration, definition);
    }
    pub(crate) fn register_prototype(&mut self, definition: TemplateDefinition) {
        self.prototype_templates
            .insert(definition.template.candidate.declaration);
        self.register(definition);
    }
    pub(crate) fn candidate(&self, declaration: DeclarationId) -> Option<Candidate> {
        self.templates
            .get(&declaration)
            .map(|definition| definition.template.candidate.clone())
    }
    pub(crate) fn result_usages(
        &self,
        declaration: DeclarationId,
    ) -> Option<Vec<jai_syntax::ResultUsage>> {
        Some(
            self.templates
                .get(&declaration)?
                .template
                .results
                .iter()
                .map(|result| result.usage)
                .collect(),
        )
    }
    /// Argument descriptions need a result type without reserving body work.
    pub(crate) fn preview_results(
        &self,
        matched: &Match,
        types: &mut TypeRegistry,
        span: Span,
        nominal: &mut impl FnMut(
            &mut TypeRegistry,
            DeclarationId,
            Substitution,
        ) -> Result<TypeId, super::SubstitutionError>,
    ) -> Result<Vec<TypeId>, Diagnostic> {
        let definition = self.templates.get(&matched.declaration).ok_or_else(|| {
            Diagnostic::new(span, "selected declaration is not a procedure template")
        })?;
        let mut results = definition
            .template
            .results
            .iter()
            .map(|result| {
                super::materialize_with_nominals(types, &result.ty, &matched.substitution, nominal)
                    .map_err(|error| {
                        Diagnostic::new(
                            definition.span,
                            format!("invalid specialized result: {error:?}"),
                        )
                    })
            })
            .collect::<Result<Vec<_>, Diagnostic>>()?;
        if results.len() == 1 && matches!(types.kind(results[0]), Ok(jai_types::TypeKind::Void)) {
            results.clear();
        }
        Ok(results)
    }
    pub(crate) fn is_template(&self, declaration: DeclarationId) -> bool {
        self.templates.contains_key(&declaration)
    }
    pub(crate) fn callback_source_origin(
        &self,
        id: ProcedureId,
    ) -> Option<(DeclarationId, FileInstanceId)> {
        self.work
            .entries()
            .iter()
            .find(|entry| entry.procedure == id)
            .map(|entry| (entry.key.declaration, entry.defining_file))
    }
    pub(crate) fn callback_signature(&self, id: ProcedureId) -> Option<Signature> {
        self.signatures.get(&id).cloned()
    }
    pub(crate) fn callback_substitution(&self, id: ProcedureId) -> Option<Substitution> {
        self.work
            .entries()
            .iter()
            .find(|entry| entry.procedure == id)
            .map(|entry| entry.key.substitution.clone())
    }
    pub(crate) fn recheck_callback_body(&mut self, id: ProcedureId) -> Result<bool, Diagnostic> {
        if self
            .callback_source_origin(id)
            .is_some_and(|(declaration, _)| self.prototype_templates.contains(&declaration))
        {
            return Ok(false);
        }
        self.work.recheck_body(id)
    }
    pub(crate) fn callback_body_ready(&self, id: ProcedureId) -> Option<bool> {
        self.work.callback_body_ready(id)
    }
    pub(crate) fn callback_body_failure(&self, id: ProcedureId) -> Option<&Diagnostic> {
        self.work.callback_body_failure(id)
    }
    pub(crate) fn callback_readiness_revision(&self) -> usize {
        self.work.callback_readiness_revision()
    }
    pub(crate) fn pending_callback_rechecks(&self) -> Vec<ProcedureId> {
        self.work.pending_callback_rechecks()
    }
    pub(crate) fn signature_snapshot(&self) -> HashMap<ProcedureId, TypeId> {
        self.signatures
            .iter()
            .map(|(&id, signature)| (id, signature.ty))
            .chain(self.modifiers.signatures())
            .collect()
    }
    pub(crate) fn source_declaration(&self, procedure: ProcedureId) -> Option<DeclarationId> {
        self.work
            .entries()
            .iter()
            .find(|entry| entry.procedure == procedure)
            .map(|entry| entry.key.declaration)
    }
    pub(crate) fn prototype_snapshot(&self) -> Vec<ProcedurePrototype> {
        let mut prototypes = self.prototypes.values().cloned().collect::<Vec<_>>();
        prototypes.sort_by_key(|prototype| prototype.id.index());
        prototypes
    }
    pub(crate) fn specialization_count(&self) -> usize {
        self.work.entries().len()
    }
    pub(crate) fn reserve_local_procedure(&mut self) -> Result<ProcedureId, Diagnostic> {
        self.work.reserve_procedure_identity()
    }
    pub(crate) fn mark_isolated_procedure(&mut self, id: ProcedureId) {
        self.isolated_procedures.insert(id);
    }
    pub(crate) fn is_isolated_procedure(&self, id: ProcedureId) -> bool {
        self.is_modifier_procedure(id) || self.isolated_procedures.contains(&id)
    }
    pub(crate) fn calling_mode(
        &self,
        declaration: DeclarationId,
    ) -> Option<(CallingConvention, ContextMode)> {
        self.templates
            .get(&declaration)
            .map(|definition| (definition.convention, definition.context))
    }
    pub(crate) fn has_pending_bodies(&self) -> bool {
        self.work
            .entries()
            .iter()
            .any(|entry| matches!(entry.state(), super::SpecializationState::BodyPending))
    }
    pub(crate) fn next_body(&mut self) -> Option<GenericBody> {
        let specialization = self.work.next_body()?;
        let entry = self
            .work
            .get(specialization)
            .expect("queued specialization exists");
        Some(GenericBody {
            specialization,
            declaration: entry.key.declaration,
            file: entry.defining_file,
            signature: self.signatures[&entry.procedure].clone(),
            substitution: entry.key.substitution.clone(),
        })
    }
    pub(crate) fn complete(&mut self, id: SpecializationId) -> Result<(), Diagnostic> {
        self.work.complete(id)
    }
    pub(crate) fn retry_body(&mut self, id: SpecializationId) -> Result<(), Diagnostic> {
        self.work.retry_body(id)
    }
    pub(crate) fn fail(
        &mut self,
        id: SpecializationId,
        error: Diagnostic,
    ) -> Result<(), Diagnostic> {
        self.work.fail(id, error)
    }
    pub(crate) fn specialize_with_origins(
        &mut self,
        matched: &Match,
        types: &mut TypeRegistry,
        span: Span,
        nominal: &mut impl FnMut(
            &mut TypeRegistry,
            DeclarationId,
            Substitution,
        ) -> Result<TypeId, super::SubstitutionError>,
        prototype: &mut impl FnMut(
            DeclarationId,
            &Signature,
            &TypeRegistry,
        ) -> Result<ProcedurePrototype, Diagnostic>,
    ) -> Result<Signature, Diagnostic> {
        let definition = self.templates.get(&matched.declaration).ok_or_else(|| {
            Diagnostic::new(span, "selected declaration is not a procedure template")
        })?;
        let reservation = self.work.reserve(
            matched.substitution.key(matched.declaration),
            definition.file,
        )?;
        if !reservation.newly_reserved {
            if let Some(super::Readiness::Failed(error)) =
                self.work.query(reservation.specialization)
            {
                return Err(error.clone());
            }
            return self
                .signatures
                .get(&reservation.procedure)
                .cloned()
                .ok_or_else(|| {
                    Diagnostic::new(span, "recursive specialization signature is not ready")
                });
        }
        let signature = (|| {
            let mut parameters = Vec::new();
            let mut variadic = Variadic::None;
            let mut source_variadic = CandidateVariadic::None;
            let mut runtime_count = 0;
            for (index, parameter) in definition.template.candidate.parameters.iter().enumerate() {
                let mut ty = super::materialize_with_nominals(
                    types,
                    &parameter.ty,
                    &matched.substitution,
                    nominal,
                )
                .map_err(|error| {
                    Diagnostic::new(
                        definition.span,
                        format!("invalid specialized parameter: {error:?}"),
                    )
                })?;
                if parameter.is_baked(&matched.substitution) {
                    continue;
                }
                if matches!(definition.template.candidate.variadic, CandidateVariadic::Jai { parameter } if parameter == index)
                {
                    source_variadic = CandidateVariadic::Jai {
                        parameter: parameters.len(),
                    };
                    if parameter.evaluation == jai_syntax::ParameterEvaluation::Evaluate {
                        variadic = Variadic::Jai {
                            parameter: runtime_count,
                            element: ty,
                        };
                    }
                    ty = types
                        .slice(ty)
                        .map_err(|error| Diagnostic::new(definition.span, error.to_string()))?;
                }
                let default = parameter.default.as_ref().map(|value| {
                    if parameter.evaluation == jai_syntax::ParameterEvaluation::Discard {
                        Ok(ParameterDefault::Discarded)
                    } else if let Some(ConstantArgument::RuntimeRead(read)) = &value.constant {
                        if read.ty() != ty || value.ty != ArgumentType::Known(ty) {
                            return Err(Diagnostic::new(definition.span, "runtime storage default differs from its specialized parameter type"));
                        }
                        Ok(ParameterDefault::RuntimeRead(read.clone()))
                    } else if matches!(value.constant, Some(ConstantArgument::CodeNull)) {
                        if value.ty != ArgumentType::Known(ty) || !matches!(types.kind(ty), Ok(jai_types::TypeKind::Code)) {
                            return Err(Diagnostic::new(definition.span, "#code, null default differs from its specialized Code parameter type"));
                        }
                        Ok(ParameterDefault::CodeNull { ty })
                    } else if matches!(value.constant, Some(ConstantArgument::CallerLocation)) {
                        if value.ty != ArgumentType::Known(ty) {
                            return Err(Diagnostic::new(definition.span, "#caller_location default differs from its specialized parameter type"));
                        }
                        Ok(ParameterDefault::CallerLocation)
                    } else {
                        constant_for_target(value, ty, types, definition.span).map(ParameterDefault::Constant)
                    }
                }).transpose()?;
                parameters.push(ParameterSignature {
                    evaluation: parameter.evaluation,
                    name: parameter.name,
                    ty,
                    default,
                });
                if parameter.evaluation == jai_syntax::ParameterEvaluation::Evaluate {
                    runtime_count += 1;
                }
            }
            let mut results = Vec::new();
            for result in &definition.template.results {
                let ty = super::materialize_with_nominals(
                    types,
                    &result.ty,
                    &matched.substitution,
                    nominal,
                )
                .map_err(|error| {
                    Diagnostic::new(
                        definition.span,
                        format!("invalid specialized result: {error:?}"),
                    )
                })?;
                let default = result
                    .default
                    .as_ref()
                    .map(|value| constant_for_target(value, ty, types, definition.span))
                    .transpose()?;
                results.push(ResultSignature {
                    name: result.name,
                    ty,
                    default,
                    usage: result.usage,
                });
            }
            if results.len() == 1
                && matches!(types.kind(results[0].ty), Ok(jai_types::TypeKind::Void))
            {
                results.clear();
            }
            if matches!(
                definition.template.candidate.variadic,
                CandidateVariadic::C { .. }
            ) {
                source_variadic = CandidateVariadic::C {
                    fixed_parameters: parameters.len(),
                };
            }
            let ty = types
                .procedure(ProcedureType {
                    parameters: parameters
                        .iter()
                        .filter(|parameter| {
                            parameter.evaluation == jai_syntax::ParameterEvaluation::Evaluate
                        })
                        .map(|parameter| parameter.ty)
                        .collect(),
                    results: results.iter().map(|result| result.ty).collect(),
                    convention: definition.convention,
                    context: definition.context,
                    variadic: match definition.template.candidate.variadic {
                        CandidateVariadic::C { .. } => Variadic::C {
                            fixed_parameters: runtime_count,
                        },
                        _ => variadic,
                    },
                })
                .map_err(|error| Diagnostic::new(definition.span, error.to_string()))?;
            Ok::<_, Diagnostic>(Signature {
                id: reservation.procedure,
                ty,
                parameters,
                source_variadic,
                results,
            })
        })();
        match signature {
            Ok(signature) => {
                if self.prototype_templates.contains(&matched.declaration) {
                    let metadata = match prototype(matched.declaration, &signature, types) {
                        Ok(metadata)
                            if metadata.id == signature.id
                                && metadata.signature == signature.ty =>
                        {
                            metadata
                        }
                        Ok(_) => {
                            let error = Diagnostic::new(
                                span,
                                "generic prototype binder changed its reserved signature identity",
                            );
                            self.work.fail(reservation.specialization, error.clone())?;
                            return Err(error);
                        }
                        Err(error) => {
                            self.work.fail(reservation.specialization, error.clone())?;
                            return Err(error);
                        }
                    };
                    self.work
                        .publish_prototype(reservation.specialization, signature.ty)?;
                    self.prototypes.insert(metadata.id, metadata);
                } else {
                    self.work
                        .publish_signature(reservation.specialization, signature.ty)?;
                }
                self.signatures.insert(signature.id, signature.clone());
                Ok(signature)
            }
            Err(error) => {
                self.work.fail(reservation.specialization, error.clone())?;
                Err(error)
            }
        }
    }
}

pub(crate) fn concrete_candidate<Origin>(
    declaration: Origin,
    signature: &Signature,
    types: &dyn TypeView,
) -> Candidate<Origin> {
    let variadic = signature.source_variadic;
    Candidate {
        declaration,
        variadic,
        parameters: signature
            .parameters
            .iter()
            .enumerate()
            .map(|(index, parameter)| Parameter {
                evaluation: parameter.evaluation,
                name: parameter.name,
                ty: TypePattern::Concrete(match variadic {
                    CandidateVariadic::Jai { parameter: pack } if pack == index => {
                        let Ok(jai_types::TypeKind::Slice(element)) = types.kind(parameter.ty)
                        else {
                            unreachable!("source Jai pack parameter is a checked slice");
                        };
                        *element
                    }
                    _ => parameter.ty,
                }),
                default: parameter.default.as_ref().map(|value| match value {
                    ParameterDefault::Source(_) => ArgumentInfo::typed(parameter.ty),
                    ParameterDefault::RuntimeRead(read) => ArgumentInfo::runtime_read(read.clone()),
                    ParameterDefault::Discarded => ArgumentInfo::typed(parameter.ty),
                    ParameterDefault::Constant(value) => {
                        ArgumentInfo::constant(BakedValue::Value(value.clone()), value.ty)
                    }
                    ParameterDefault::CallerLocation => ArgumentInfo::caller_location(parameter.ty),
                    ParameterDefault::CodeNull { ty } => ArgumentInfo::code_null(*ty),
                }),
                baking: jai_syntax::ParameterBaking::None,
            })
            .collect(),
    }
}

fn constant_for_target(
    info: &ArgumentInfo,
    ty: TypeId,
    types: &dyn TypeView,
    span: Span,
) -> Result<ConstantValue, Diagnostic> {
    let value = crate::overloads::bake(
        types,
        &TypePattern::Concrete(ty),
        info,
        &Substitution::default(),
        span,
    )?;
    value
        .into_runtime(ty, types)
        .map_err(|error| Diagnostic::new(span, error.to_string()))
}

/// Follow procedure aliases by declaration identity. Ordinary scalar/type aliases
/// are left for their existing evaluators; their names never become procedure IDs.
pub(crate) fn callable_aliases(graph: &ModuleGraph) -> HashMap<DeclarationId, Vec<DeclarationId>> {
    fn binding(
        graph: &ModuleGraph,
        value: Binding,
        active: &mut HashSet<DeclarationId>,
        cache: &mut HashMap<DeclarationId, Option<Vec<DeclarationId>>>,
    ) -> Option<Vec<DeclarationId>> {
        match value {
            Binding::OverloadSet(id) => {
                let mut targets = Vec::new();
                for &member in graph.overload_set(id)?.declarations() {
                    targets.extend(resolve(graph, member, active, cache)?);
                }
                targets.sort_unstable_by_key(|id| id.index());
                targets.dedup();
                Some(targets)
            }
            Binding::Declaration(id) => resolve(graph, id, active, cache),
            _ => None,
        }
    }
    fn resolve(
        graph: &ModuleGraph,
        id: DeclarationId,
        active: &mut HashSet<DeclarationId>,
        cache: &mut HashMap<DeclarationId, Option<Vec<DeclarationId>>>,
    ) -> Option<Vec<DeclarationId>> {
        if let Some(value) = cache.get(&id) {
            return value.clone();
        }
        if !active.insert(id) {
            return None;
        }
        let declaration = graph.declaration(id)?;
        let value = match &declaration.syntax().kind {
            FileDeclarationKind::Procedure(_) | FileDeclarationKind::ProcedurePrototype(_) => {
                Some(vec![id])
            }
            FileDeclarationKind::Constant(constant) if constant.ty.is_none() => {
                let path = match &constant.initializer.kind {
                    ExpressionKind::Name(root) => Some(NamePath {
                        root: *root,
                        members: vec![],
                    }),
                    ExpressionKind::QualifiedName(path) => Some(path.clone()),
                    _ => None,
                };
                path.and_then(|path| graph.lookup(declaration.file(), &path).ok())
                    .and_then(|value| binding(graph, value, active, cache))
            }
            _ => None,
        };
        active.remove(&id);
        cache.insert(id, value.clone());
        value
    }
    let mut cache = HashMap::new();
    let mut aliases = HashMap::new();
    for declaration in graph.declarations() {
        if matches!(declaration.syntax().kind, FileDeclarationKind::Constant(_))
            && let Some(targets) = resolve(graph, declaration.id(), &mut HashSet::new(), &mut cache)
        {
            aliases.insert(declaration.id(), targets);
        }
    }
    aliases
}

mod arguments;

#[cfg(test)]
mod tests {
    use super::*;
    use jai_modules::GraphOptions;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT: AtomicUsize = AtomicUsize::new(0);

    #[test]
    fn result_preview_does_not_reserve_signatures_or_queue_bodies() {
        let directory = std::env::temp_dir().join(format!(
            "jai-generic-preview-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("main.jai");
        std::fs::write(
            &path,
            "nested :: (value:$T) -> T { return value; } main :: () {}",
        )
        .unwrap();
        let graph = ModuleGraph::load(&path, GraphOptions::default()).unwrap();
        let declaration = &graph.declarations()[0];
        let file = declaration.file();
        let FileDeclarationKind::Procedure(source) = &declaration.syntax().kind else {
            panic!("fixture has a source procedure");
        };
        let mut types = TypeRegistry::new();
        let template = super::super::from_procedure(
            declaration.id(),
            source,
            |_, _| unreachable!("fixture has only a type variable"),
            |_| unreachable!("fixture has no defaults"),
            |_| unreachable!("fixture has no array count"),
        )
        .unwrap();
        let matched = crate::overloads::match_candidate(
            &types,
            &template.candidate,
            &[crate::overloads::Argument {
                name: None,
                spread: false,
                info: ArgumentInfo::integer_literal(7),
                span: Span::default(),
            }],
            Span::default(),
        )
        .unwrap();
        let mut generics = GenericContext::new(20);
        generics.register(TemplateDefinition {
            template,
            file,
            span: Span::default(),
            convention: CallingConvention::Jai,
            context: ContextMode::None,
        });
        let result = generics
            .preview_results(&matched, &mut types, Span::default(), &mut |_, _, _| {
                unreachable!("scalar preview has no nominal applications")
            })
            .unwrap();
        assert_eq!(
            result,
            [types.scalar(jai_types::ScalarType::Int(jai_types::IntegerType::S64))]
        );
        assert_eq!(generics.specialization_count(), 0);
        assert!(generics.signature_snapshot().is_empty());
        assert!(generics.next_body().is_none());
        std::fs::remove_dir_all(directory).unwrap();
    }
}
