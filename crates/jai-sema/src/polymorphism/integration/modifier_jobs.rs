//! Auxiliary modifier bodies share procedure identity and checked-body scheduling.
use super::*;
use crate::modifiers::{ModifierPlan, ModifierSlot};
use crate::polymorphism::SpecializationKey;
use jai_source::Symbol;
use jai_syntax::{self as syntax, BuiltinType, StatementKind, TypeSyntax};
use jai_types::{InlineHint, ScalarType};
use std::collections::VecDeque;

#[derive(Clone)]
pub(crate) struct ModifierBody {
    pub key: SpecializationKey,
    pub declaration: DeclarationId,
    pub file: FileInstanceId,
    pub signature: Signature,
    pub procedure: syntax::Procedure,
    pub plan: ModifierPlan,
    pub initial: Substitution,
}

pub(crate) enum ModifierReadiness {
    Pending(ProcedureId),
    BodyReady(Box<ModifierBody>),
    Finished(Result<Substitution, Diagnostic>),
}
enum State {
    Queued,
    Binding,
    BodyReady,
    Finished(Result<Substitution, Diagnostic>),
}
struct Entry {
    procedure: ProcedureId,
    body: Option<ModifierBody>,
    state: State,
}
#[derive(Default)]
pub(super) struct ModifierJobs {
    entries: HashMap<SpecializationKey, Entry>,
    queue: VecDeque<SpecializationKey>,
}
impl ModifierJobs {
    pub(super) fn signatures(&self) -> impl Iterator<Item = (ProcedureId, TypeId)> + '_ {
        self.entries.values().filter_map(|entry| {
            entry
                .body
                .as_ref()
                .map(|body| (body.signature.id, body.signature.ty))
        })
    }
    pub(super) fn contains_procedure(&self, procedure: ProcedureId) -> bool {
        self.entries
            .values()
            .any(|entry| entry.procedure == procedure)
    }
    pub(super) fn len(&self) -> usize {
        self.entries.len()
    }
    fn readiness(&self, key: &SpecializationKey) -> Option<ModifierReadiness> {
        self.entries.get(key).map(|entry| match &entry.state {
            State::Queued | State::Binding => ModifierReadiness::Pending(entry.procedure),
            State::BodyReady => ModifierReadiness::BodyReady(Box::new(
                entry
                    .body
                    .as_ref()
                    .expect("ready modifier has a body")
                    .clone(),
            )),
            State::Finished(result) => ModifierReadiness::Finished(result.clone()),
        })
    }
}

impl GenericContext {
    pub(crate) fn request_modifier(
        &mut self,
        source: &syntax::Procedure,
        matched: &Match,
        types: &mut TypeRegistry,
        span: Span,
    ) -> Result<ModifierReadiness, Diagnostic> {
        let key = matched.substitution.key(matched.declaration);
        if let Some(ready) = self.modifiers.readiness(&key) {
            return Ok(ready);
        }
        let file = self
            .templates
            .get(&matched.declaration)
            .ok_or_else(|| {
                Diagnostic::new(span, "modifier declaration is not a registered template")
            })?
            .file;
        self.request_modifier_body(
            matched.declaration,
            file,
            &matched.substitution,
            |procedure| build(source, matched, procedure, types),
        )
    }

    /// A record or procedure adapter supplies owned checked auxiliary source.
    /// Its real declaration identity remains the cache origin.
    pub(crate) fn request_modifier_body(
        &mut self,
        declaration: DeclarationId,
        file: FileInstanceId,
        initial: &Substitution,
        build: impl FnOnce(
            ProcedureId,
        ) -> Result<(Signature, syntax::Procedure, ModifierPlan), Diagnostic>,
    ) -> Result<ModifierReadiness, Diagnostic> {
        let key = initial.key(declaration);
        if let Some(ready) = self.modifiers.readiness(&key) {
            return Ok(ready);
        }
        let procedure = self.work.reserve_procedure_identity()?;
        let body = build(procedure).and_then(|(signature, source, plan)| {
            if signature.id != procedure
                || signature
                    .parameters
                    .iter()
                    .filter(|parameter| {
                        parameter.evaluation == syntax::ParameterEvaluation::Evaluate
                    })
                    .count()
                    != plan.slots().len()
                || signature.results.len() != plan.slots().len() + 2
            {
                return Err(Diagnostic::new(
                    source.span,
                    "auxiliary modifier header does not match its reserved identity and slot plan",
                ));
            }
            Ok(ModifierBody {
                key: key.clone(),
                declaration,
                file,
                signature,
                procedure: source,
                plan,
                initial: initial.clone(),
            })
        });
        let (body, state) = match body {
            Ok(body) => {
                self.modifiers.queue.push_back(key.clone());
                (Some(body), State::Queued)
            }
            Err(error) => (None, State::Finished(Err(error))),
        };
        self.modifiers.entries.insert(
            key.clone(),
            Entry {
                procedure,
                body,
                state,
            },
        );
        Ok(self.modifiers.readiness(&key).expect("inserted modifier"))
    }
    pub(crate) fn next_modifier_body(&mut self) -> Option<ModifierBody> {
        let key = self.modifiers.queue.pop_front()?;
        let entry = self
            .modifiers
            .entries
            .get_mut(&key)
            .expect("queued modifier exists");
        assert!(
            matches!(entry.state, State::Queued),
            "modifier body is queued once"
        );
        entry.state = State::Binding;
        entry.body.clone()
    }
    pub(crate) fn complete_modifier_body(
        &mut self,
        key: &SpecializationKey,
    ) -> Result<(), Diagnostic> {
        let entry = self
            .modifiers
            .entries
            .get_mut(key)
            .ok_or_else(|| Diagnostic::new(Span::default(), "unknown modifier job"))?;
        if !matches!(entry.state, State::Binding) {
            return Err(Diagnostic::new(
                Span::default(),
                "modifier body is not being bound",
            ));
        }
        entry.state = State::BodyReady;
        Ok(())
    }
    pub(crate) fn retry_modifier_body(
        &mut self,
        key: &SpecializationKey,
    ) -> Result<(), Diagnostic> {
        let entry = self
            .modifiers
            .entries
            .get_mut(key)
            .ok_or_else(|| Diagnostic::new(Span::default(), "unknown modifier job"))?;
        if !matches!(entry.state, State::Binding) {
            return Err(Diagnostic::new(
                Span::default(),
                "modifier body is not being bound",
            ));
        }
        entry.state = State::Queued;
        self.modifiers.queue.push_back(key.clone());
        Ok(())
    }
    pub(crate) fn finish_modifier(
        &mut self,
        key: &SpecializationKey,
        result: Result<Substitution, Diagnostic>,
    ) -> Result<(), Diagnostic> {
        let entry = self
            .modifiers
            .entries
            .get_mut(key)
            .ok_or_else(|| Diagnostic::new(Span::default(), "unknown modifier job"))?;
        entry.state = State::Finished(result);
        Ok(())
    }
    pub(crate) fn fail_modifier(
        &mut self,
        key: &SpecializationKey,
        error: Diagnostic,
    ) -> Result<(), Diagnostic> {
        self.finish_modifier(key, Err(error))
    }
    pub(crate) fn has_pending_modifier_bodies(&self) -> bool {
        self.modifiers
            .entries
            .values()
            .any(|entry| matches!(entry.state, State::Queued | State::Binding))
    }
    pub(crate) fn is_modifier_procedure(&self, procedure: ProcedureId) -> bool {
        self.modifiers.contains_procedure(procedure)
    }
    pub(crate) fn modifier_job_count(&self) -> usize {
        self.modifiers.len()
    }
}

fn build(
    source: &syntax::Procedure,
    matched: &Match,
    id: ProcedureId,
    types: &mut TypeRegistry,
) -> Result<(Signature, syntax::Procedure, ModifierPlan), Diagnostic> {
    build_modifier_source(
        procedure_modifier_source(source, &matched.substitution, types)?,
        id,
        types,
    )
}

/// Keep source formals and inferred bindings independent of the job origin.
pub(crate) fn procedure_modifier_source<'a>(
    source: &'a syntax::Procedure,
    initial: &Substitution,
    types: &dyn TypeView,
) -> Result<ModifierSource<'a>, Diagnostic> {
    let modifier = source
        .modify
        .as_ref()
        .ok_or_else(|| Diagnostic::new(source.span, "procedure has no specialization modifier"))?;
    let mut slots = Vec::new();
    let mut names = HashSet::new();
    for binding in &initial.types {
        names.insert(binding.name);
        slots.push(ModifierSlot::Type { name: binding.name });
    }
    for binding in &initial.constants {
        if !names.insert(binding.name) {
            continue;
        }
        slots.push(match &binding.value {
            BakedValue::Type(_) => ModifierSlot::Type { name: binding.name },
            BakedValue::Value(value) => ModifierSlot::Baked {
                name: binding.name,
                ty: value.ty,
            },
            BakedValue::Float(value) => ModifierSlot::Baked {
                name: binding.name,
                ty: types.float(value.ty()),
            },
            BakedValue::String(_) => ModifierSlot::Baked {
                name: binding.name,
                ty: types.lookup(&jai_types::TypeKind::String).ok_or_else(|| {
                    Diagnostic::new(
                        modifier.span,
                        "modifier type view has no canonical string type",
                    )
                })?,
            },
            BakedValue::Code(_) => {
                return Err(Diagnostic::new(
                    modifier.span,
                    "#modify cannot store compiler-only Code bindings",
                ));
            }
        });
    }
    let mut result_names = Vec::new();
    for result in &source.results {
        if let syntax::ResultBinding::Typed { ty, .. } = &result.binding {
            collect_result_names(ty, &mut result_names);
        }
    }
    for name in result_names {
        if names.insert(name) {
            slots.push(ModifierSlot::Type { name });
        }
    }
    Ok(ModifierSource {
        name: source.name,
        parameters: &source.parameters,
        modifier,
        slots,
        context: source.context,
        checks: source.checks,
    })
}

/// Genuine record and procedure adapters share checked source lowering.
/// The job request retains the original declaration and file identities.
pub(crate) struct ModifierSource<'a> {
    pub name: Symbol,
    pub parameters: &'a [syntax::Parameter],
    pub modifier: &'a syntax::ModifyDirective,
    pub slots: Vec<ModifierSlot>,
    pub context: ContextMode,
    pub checks: syntax::SafetyChecks,
}

pub(crate) fn build_modifier_source(
    source: ModifierSource<'_>,
    id: ProcedureId,
    types: &mut TypeRegistry,
) -> Result<(Signature, syntax::Procedure, ModifierPlan), Diagnostic> {
    let modifier = source.modifier;
    if source
        .slots
        .iter()
        .any(|slot| matches!(slot, ModifierSlot::Type { .. }))
    {
        jai_types::RuntimeTypeSchema::from_view(types)
            .map_err(|error| Diagnostic::new(modifier.span, error.to_string()))?;
    }
    let evaluation = |name| {
        source
            .parameters
            .iter()
            .find(|parameter| parameter.name == name)
            .map_or(syntax::ParameterEvaluation::Evaluate, |parameter| {
                parameter.evaluation
            })
    };
    let plan = ModifierPlan::new(
        source
            .slots
            .iter()
            .copied()
            .filter(|slot| {
                let name = match slot {
                    ModifierSlot::Type { name } | ModifierSlot::Baked { name, .. } => *name,
                };
                evaluation(name) == syntax::ParameterEvaluation::Evaluate
            })
            .collect(),
    )
    .map_err(|error| Diagnostic::new(modifier.span, error.to_string()))?;
    let mut procedure = syntax::Procedure {
        name: source.name,
        operator: None,
        source: syntax::SourceProcedureSyntax {
            header: syntax::SourceProcedureHeader {
                callable: syntax::CallableHeaderSyntax {
                    deprecation: None,
                    notes: Vec::new(),
                    parameters: Vec::new(),
                    results: Vec::new(),
                    convention: CallingConvention::Jai,
                    context: source.context,
                },
                checks: source.checks,
                inline_hint: InlineHint::Automatic,
                execution: jai_types::ProcedureExecution::CompileTimeOnly,
                debug: jai_types::DebugPolicy::Suppress,
                compiler: None,
                expands: false,
                modify: None,
            },
            body: modifier.body.clone(),
            span: modifier.span,
        },
    };
    rewrite_returns(&mut procedure.body, &plan)?;
    procedure.parameters = source
        .slots
        .iter()
        .map(|slot| {
            let name = match slot {
                ModifierSlot::Type { name } | ModifierSlot::Baked { name, .. } => *name,
            };
            let binding = match slot {
                ModifierSlot::Type { .. } => {
                    syntax::ParameterBinding::RequiredType(TypeSyntax::Builtin(BuiltinType::Type))
                }
                ModifierSlot::Baked { .. } => source
                    .parameters
                    .iter()
                    .find(|parameter| parameter.name == name)
                    .map(|parameter| parameter.binding.clone())
                    .ok_or_else(|| {
                        Diagnostic::new(
                            modifier.span,
                            "baked modifier binding has no source parameter",
                        )
                    })?,
            };
            Ok(syntax::Parameter {
                evaluation: evaluation(name),
                name,
                binding,
                using: false,
                baking: syntax::ParameterBaking::None,
                variadic: false,
                span: modifier.span,
            })
        })
        .collect::<Result<_, Diagnostic>>()?;
    let mut result_types = vec![types.scalar(ScalarType::Bool), types.string()];
    let mut parameter_types = Vec::new();
    for slot in plan.slots() {
        let ty = match slot {
            ModifierSlot::Type { .. } => types.meta_type(),
            ModifierSlot::Baked { ty, .. } => *ty,
        };
        parameter_types.push(ty);
        result_types.push(ty);
    }
    let signature_type = types
        .procedure(ProcedureType {
            parameters: parameter_types.clone().into_boxed_slice(),
            results: result_types.clone().into_boxed_slice(),
            convention: CallingConvention::Jai,
            context: source.context,
            variadic: Variadic::None,
        })
        .map_err(|error| Diagnostic::new(modifier.span, error.to_string()))?;
    let parameters: Vec<_> = procedure
        .parameters
        .iter()
        .zip(&source.slots)
        .map(|(parameter, slot)| ParameterSignature {
            evaluation: parameter.evaluation,
            name: parameter.name,
            ty: match slot {
                ModifierSlot::Type { .. } => types.meta_type(),
                ModifierSlot::Baked { ty, .. } => *ty,
            },
            default: None,
        })
        .collect();
    let results = result_types
        .into_iter()
        .map(|ty| ResultSignature {
            name: None,
            ty,
            default: None,
            usage: syntax::ResultUsage::Optional,
        })
        .collect();
    // The scheduler uses the checked Signature for this synthetic header; the
    // original header's result syntax remains only source metadata.
    procedure.results.clear();
    Ok((
        Signature {
            source_variadic: CandidateVariadic::None,
            id,
            ty: signature_type,
            parameters,
            results,
        },
        procedure,
        plan,
    ))
}

fn collect_result_names(ty: &TypeSyntax, names: &mut Vec<Symbol>) {
    match ty {
        TypeSyntax::Variable(name) => names.push(*name),
        TypeSyntax::Pointer(ty) | TypeSyntax::Slice(ty) | TypeSyntax::DynamicArray(ty) => {
            collect_result_names(ty, names)
        }
        TypeSyntax::FixedArray { element, .. } => collect_result_names(element, names),
        TypeSyntax::Procedure(procedure) => {
            for parameter in procedure.parameters.iter().chain(&procedure.results) {
                collect_result_names(&parameter.ty, names);
            }
        }
        _ => {}
    }
}
fn rewrite_returns(body: &mut [syntax::Statement], plan: &ModifierPlan) -> Result<(), Diagnostic> {
    for statement in body {
        let span = statement.span;
        match &mut statement.kind {
            StatementKind::Return(value) => {
                let accepted = value
                    .take()
                    .ok_or_else(|| Diagnostic::new(span, "#modify must return acceptance bool"))?;
                statement.kind = StatementKind::ReturnValues(outputs(
                    vec![syntax::ReturnValue {
                        name: None,
                        value: accepted,
                    }],
                    plan,
                    span,
                )?);
            }
            StatementKind::ReturnValues(values) => {
                *values = outputs(std::mem::take(values), plan, span)?
            }
            StatementKind::If(_, yes, no)
            | StatementKind::CompileTimeIf {
                then_body: yes,
                else_body: no,
                ..
            } => {
                rewrite_returns(yes, plan)?;
                rewrite_returns(no, plan)?;
            }
            StatementKind::Block(body)
            | StatementKind::While(_, body)
            | StatementKind::CheckScope { body, .. }
            | StatementKind::PushContext { body, .. } => rewrite_returns(body, plan)?,
            StatementKind::Range(loop_) => rewrite_returns(&mut loop_.body, plan)?,
            StatementKind::ArrayLoop(loop_) => rewrite_returns(&mut loop_.body, plan)?,
            StatementKind::Cases(cases) => {
                for (_, body, _) in &mut cases.arms {
                    rewrite_returns(body, plan)?;
                }
                if let Some(body) = &mut cases.default {
                    rewrite_returns(body, plan)?;
                }
            }
            StatementKind::CompileTimeCases(cases) => {
                for arm in &mut cases.arms {
                    rewrite_returns(&mut arm.body, plan)?;
                }
                if let Some(default) = &mut cases.default {
                    rewrite_returns(&mut default.body, plan)?;
                }
            }
            // A nested procedure owns its own return contract.
            _ => {}
        }
    }
    Ok(())
}
fn outputs(
    mut values: Vec<syntax::ReturnValue>,
    plan: &ModifierPlan,
    span: Span,
) -> Result<Vec<syntax::ReturnValue>, Diagnostic> {
    if !(1..=2).contains(&values.len()) || values.iter().any(|value| value.name.is_some()) {
        return Err(Diagnostic::new(
            span,
            "#modify returns acceptance bool and optional explanation string",
        ));
    }
    if values.len() == 1 {
        values.push(syntax::ReturnValue {
            name: None,
            value: syntax::Expression {
                span,
                kind: ExpressionKind::String(vec![]),
            },
        });
    }
    values.extend(plan.slots().iter().map(|slot| syntax::ReturnValue {
        name: None,
        value: syntax::Expression {
            span,
            kind: ExpressionKind::Name(match slot {
                ModifierSlot::Type { name } | ModifierSlot::Baked { name, .. } => *name,
            }),
        },
    }));
    Ok(values)
}
