//! Selected optional bakes materialize their original source recipe once.
use crate::overloads::{Argument, ArgumentInfo, ArgumentType, ConstantArgument, Match};
use crate::polymorphism::BakedValue;
use crate::procedure_values::contracts::ValueContract;
use crate::{Diagnostic, Resolver, Span, TypeId};
use jai_ir::ProcedureId;
use jai_source::SourceSpan;
use jai_syntax as syntax;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq)]
struct SelectedConstantKey {
    owner: ProcedureId,
    source: SourceSpan,
    scope: Vec<crate::local_declarations::LexicalScopeId>,
    expected: TypeId,
    checks: crate::safety_checks::ActiveChecks,
    target: Option<jai_types::LayoutPolicy>,
}

impl std::hash::Hash for SelectedConstantKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        use std::hash::Hash;
        self.owner.hash(state);
        self.source.source.hash(state);
        self.source.span.start.hash(state);
        self.source.span.end.hash(state);
        self.scope.hash(state);
        self.expected.hash(state);
        self.checks.hash(state);
        self.target.hash(state);
    }
}

#[derive(Clone)]
struct SelectedConstant {
    info: ArgumentInfo,
    contract: Option<ValueContract>,
    producer: jai_ir::ValueExpr,
    revision: (ProcedureId, Option<usize>, Option<usize>),
}

#[derive(Default)]
pub(crate) struct SelectedConstants {
    values: HashMap<SelectedConstantKey, SelectedConstant>,
}

impl Resolver<'_> {
    fn selected_constant_key(
        &self,
        source: &syntax::Expression,
        expected: TypeId,
    ) -> Result<SelectedConstantKey, Diagnostic> {
        let location = self.ast_source_location(source.span)?;
        Ok(SelectedConstantKey {
            owner: self
                .expression_owner
                .map(|owner| self.compile_time.map_or(owner, |context| context.owner))
                .ok_or_else(|| {
                    Diagnostic::at_source(
                        location,
                        "selected constant needs its actual source owner",
                    )
                })?,
            source: location,
            scope: self.local_scopes.capture_identity(),
            expected,
            checks: self.checks,
            target: self.target_layout,
        })
    }

    pub(crate) fn selected_source_constant_contract(
        &self,
        source: &syntax::Expression,
        expected: TypeId,
    ) -> Result<Option<Option<ValueContract>>, Diagnostic> {
        if self.expression_owner.is_none() {
            return Ok(None);
        }
        let key = self.selected_constant_key(source, expected)?;
        if let Some(failure) = self.baked_source_policy_failure(key.owner) {
            return Err(failure);
        }
        Ok(self
            .meta
            .selected_constants
            .values
            .get(&key)
            .filter(|value| value.revision == self.baked_source_policy_revision(key.owner))
            .map(|value| value.contract.clone()))
    }

    pub(crate) fn materialize_selected_source_constant(
        &mut self,
        source: &syntax::Expression,
        expected: TypeId,
    ) -> Result<ArgumentInfo, Diagnostic> {
        let key = self.selected_constant_key(source, expected)?;
        if let Some(failure) = self.baked_source_policy_failure(key.owner) {
            return Err(failure);
        }
        if let Some(previous) = self.meta.selected_constants.values.get(&key).cloned() {
            let revision = self.baked_source_policy_revision(key.owner);
            if previous.revision != revision {
                // The checked value and explicit #run effects stay cached. Only
                // policy derivation is refreshed under this actual source owner.
                let producer = if matches!(source.kind, syntax::ExpressionKind::BakeArguments(_)) {
                    let expression = self.expr_expected(source, expected)?;
                    self.coerce_value(expression, expected, source.span)?
                } else {
                    previous.producer.clone()
                };
                if matches!(source.kind, syntax::ExpressionKind::BakeArguments(_)) {
                    let constant = self.literal_constant(producer.clone(), source.span)?;
                    let current = BakedValue::runtime(constant, self.types)
                        .map_err(|error| Diagnostic::at_source(key.source, error.to_string()))?;
                    if !matches!(&previous.info.constant, Some(ConstantArgument::Value(value)) if value == &current)
                    {
                        return Err(Diagnostic::at_source(
                            key.source,
                            "selected constant value changed during its original body recheck",
                        ));
                    }
                }
                let contract = self.callback_expression_contract(source, &producer, source.span)?;
                if let Some(failure) = self.baked_source_policy_failure(key.owner) {
                    return Err(failure);
                }
                let revision = self.baked_source_policy_revision(key.owner);
                if let Some(value) = self.meta.selected_constants.values.get_mut(&key) {
                    value.contract = contract;
                    value.producer = producer;
                    value.revision = revision;
                }
            }
            return Ok(previous.info);
        }
        let expression = self.expr_expected(source, expected).map_err(|error| {
            Diagnostic::at_source(
                key.source,
                format!(
                    "selected constant materialization failed: {}",
                    error.message
                ),
            )
        })?;
        let value = self
            .coerce_value(expression, expected, source.span)
            .map_err(|error| {
                Diagnostic::at_source(
                    key.source,
                    format!(
                        "selected constant materialization failed: {}",
                        error.message
                    ),
                )
            })?;
        let contract = self.callback_expression_contract(source, &value, source.span)?;
        let producer = value.clone();
        let constant = self
            .literal_constant(value.clone(), source.span)
            .or_else(|_| self.evaluate_pure_constant(value, source.span))
            .map_err(|error| {
                Diagnostic::at_source(
                    key.source,
                    format!(
                        "selected constant materialization failed: {}",
                        error.message
                    ),
                )
            })?;
        let value = BakedValue::runtime(constant, self.types)
            .map_err(|error| Diagnostic::at_source(key.source, error.to_string()))?;
        if let Some(failure) = self.baked_source_policy_failure(key.owner) {
            return Err(failure);
        }
        let revision = self.baked_source_policy_revision(key.owner);
        let info = ArgumentInfo::constant(value, expected);
        self.meta.selected_constants.values.insert(
            key,
            SelectedConstant {
                info: info.clone(),
                contract,
                producer,
                revision,
            },
        );
        Ok(info)
    }

    pub(crate) fn materialize_selected_optional_match(
        &mut self,
        matched: Match,
        source: &[syntax::CallArgument],
        span: Span,
    ) -> Result<Match, Diagnostic> {
        let scope = self.graph_scope.ok_or_else(|| {
            Diagnostic::new(span, "selected bake requires its defining module scope")
        })?;
        let candidate = scope.candidate(matched.declaration, self.types, span)?;
        let mut selected = HashMap::new();
        for binding in &matched.bindings {
            let parameter = &candidate.parameters[binding.parameter];
            if parameter.baking != syntax::ParameterBaking::Optional
                || parameter.evaluation == syntax::ParameterEvaluation::Discard
            {
                continue;
            }
            let argument = source.get(binding.argument).ok_or_else(|| {
                Diagnostic::new(span, "selected bake lacks its original source argument")
            })?;
            if self.optional_baking_needs_materialization(&argument.value)? {
                let expected = scope.materialize_pattern(
                    &parameter.ty,
                    &matched.substitution,
                    self.types,
                    &mut self.meta.record_specializations,
                    argument.value.span,
                )?;
                let info = self.materialize_selected_source_constant(&argument.value, expected)?;
                selected.insert(binding.argument, info);
            }
        }
        if selected.is_empty() {
            return Ok(matched);
        }
        let arguments = source
            .iter()
            .enumerate()
            .map(|(ordinal, argument)| {
                let info = match selected.remove(&ordinal) {
                    Some(info) => info,
                    None if matches!(
                        argument.value.kind,
                        syntax::ExpressionKind::ShortLambda(_)
                    ) =>
                    {
                        ArgumentInfo {
                            ty: ArgumentType::ContextualProcedure {
                                compatible_signatures: Box::new([]),
                            },
                            constant: None,
                        }
                    }
                    None => self.describe_argument(&argument.value)?,
                };
                Ok(Argument {
                    name: argument.name,
                    spread: argument.spread,
                    info,
                    span: argument.value.span,
                })
            })
            .collect::<Result<Vec<_>, Diagnostic>>()?;
        let matched = crate::overloads::match_candidate_with_nominals(
            self.types, self, &candidate, &arguments, span,
        )?;
        self.refine_source_declaration_match(&candidate, source, &arguments, matched, span)
    }
}
