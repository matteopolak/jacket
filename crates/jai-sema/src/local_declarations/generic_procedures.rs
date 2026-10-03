//! Lexical templates specialize their real declarations in captured environments.
use super::*;
use crate::modules::aggregates::parameterized::LexicalTypeArguments;
use crate::overloads::{Argument, ArgumentInfo, Candidate, Match};
use crate::polymorphism::{CallablePolicyBinding, ProcedureTemplate, Substitution};
use std::cell::RefCell;
mod baked_sources;
mod contracts;
mod materialization;
mod modifiers;
mod result_contracts;

#[derive(Clone)]
struct Definition {
    source: syntax::Procedure,
    environment: sources::SourceEnvironment,
    template: ProcedureTemplate<LocalDeclarationId>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Key {
    declaration: LocalDeclarationId,
    substitution: Substitution,
}

#[derive(Default)]
pub(super) struct LocalGenericProcedures {
    definitions: HashMap<LocalDeclarationId, Definition>,
    reservations: HashMap<Key, ProcedureId>,
    signatures: HashMap<Key, Signature>,
    active: HashSet<Key>,
    failures: HashMap<Key, Diagnostic>,
    modifiers: modifiers::LocalModifiers,
}

impl LocalGenericProcedures {
    pub(super) fn callback_failure(&self, procedure: ProcedureId) -> Option<&Diagnostic> {
        self.failures.iter().find_map(|(key, error)| {
            let identity = self
                .signatures
                .get(key)
                .map(|signature| signature.id)
                .or_else(|| self.reservations.get(key).copied());
            (identity == Some(procedure)).then_some(error)
        })
    }
}

impl Resolver<'_> {
    pub(super) fn local_generic_operator_candidate(
        &mut self,
        depth: usize,
        declaration: &Declaration,
    ) -> Result<Candidate<LocalDeclarationId>, Diagnostic> {
        if let Some(definition) = self
            .meta
            .local_declarations
            .generic_procedures
            .definitions
            .get(&declaration.id)
        {
            return Ok(definition.template.candidate.clone());
        }
        let DeclarationSyntax::Procedure(source) = &declaration.syntax else {
            unreachable!("local operator registration retains a procedure");
        };
        let environment = self.local_procedure_source_environment(depth, declaration);
        let template =
            self.with_local_source_environment(&environment, source.span, |definition| {
                let lexical = definition.local_operator_lexical_arguments(source)?;
                let patterns = match definition.graph_scope {
                    Some(scope) => scope.operator_template_patterns(
                        source,
                        definition.types,
                        &mut definition.meta.record_specializations,
                        &lexical,
                    )?,
                    None => HashMap::new(),
                };
                // Each callback is invoked serially by the pure header builder.
                let resolver = RefCell::new(definition);
                crate::polymorphism::from_procedure_with_patterns(
                    declaration.id,
                    source,
                    |ty, span| resolver.borrow_mut().lexical_annotation(ty, span),
                    |expression| {
                        let mut definition = resolver.borrow_mut();
                        if matches!(expression.kind, syntax::ExpressionKind::CallerLocation) {
                            let ty = definition.caller_location_type(expression.span)?;
                            Ok(ArgumentInfo::caller_location(ty))
                        } else if let Some(read) =
                            definition.ready_runtime_parameter_default(expression, None)?
                        {
                            Ok(ArgumentInfo::runtime_read(read))
                        } else {
                            definition.describe_argument(expression)
                        }
                    },
                    |expression| resolver.borrow_mut().local_integer_count(expression),
                    &patterns,
                )
            })?;
        let candidate = template.candidate.clone();
        self.meta
            .local_declarations
            .generic_procedures
            .definitions
            .insert(
                declaration.id,
                Definition {
                    source: source.clone(),
                    environment,
                    template,
                },
            );
        Ok(candidate)
    }

    fn local_operator_lexical_arguments(
        &mut self,
        source: &syntax::Procedure,
    ) -> Result<LexicalTypeArguments, Diagnostic> {
        let mut lexical = LexicalTypeArguments::default();
        for parameter in &source.parameters {
            let ty = match &parameter.binding {
                syntax::ParameterBinding::RequiredType(ty)
                | syntax::ParameterBinding::DefaultedType {
                    ty: Some(ty), ..
                } => Some(ty),
                _ => None,
            };
            if let Some(ty) = ty {
                self.local_operator_application_origins(ty, &mut lexical)?;
            }
        }
        for result in &source.results {
            if let syntax::ResultBinding::Typed {
                ty, ..
            } = &result.binding
            {
                self.local_operator_application_origins(ty, &mut lexical)?;
            }
        }
        for frame in &self.scopes {
            for (&name, binding) in frame {
                if let Some(binding) = self.record_argument_binding(binding) {
                    lexical.roots.insert(name, binding);
                } else {
                    lexical.roots.remove(&name);
                }
            }
        }
        for (&ty, members) in &self.meta.local_declarations.record_namespaces {
            lexical.namespaces.insert(
                ty,
                members
                    .iter()
                    .filter_map(|(&name, binding)| {
                        self.record_argument_binding(binding)
                            .map(|binding| (name, binding))
                    })
                    .collect(),
            );
        }
        Ok(lexical)
    }

    fn local_operator_application_origins(
        &mut self,
        ty: &syntax::TypeSyntax,
        lexical: &mut LexicalTypeArguments,
    ) -> Result<(), Diagnostic> {
        use syntax::TypeSyntax as T;
        match ty {
            T::Application(application) => {
                if let T::Named(path) = application.base.as_ref()
                    && self
                        .resolve_local_name(path.root, application.span)?
                        .is_some()
                {
                    let scope = self.graph_scope.ok_or_else(|| {
                        Diagnostic::new(
                            application.span,
                            "lexical template requires its source graph",
                        )
                    })?;
                    let binding = self.lexical_graph_binding(path, application.span)?.ok_or_else(|| {
                        Diagnostic::new(application.span, "lexical declaration does not denote a parameterized record template")
                    })?;
                    let origin = scope.imported_record_template(binding, application.span)?;
                    lexical
                        .templates
                        .insert((application.span.start, application.span.end), origin);
                }
                for argument in &application.arguments {
                    if let Some(ty) =
                        crate::modules::aggregates::parameterized::type_expression(&argument.value)
                    {
                        self.local_operator_application_origins(&ty, lexical)?;
                    }
                }
            }
            T::Pointer(inner)
            | T::Slice(inner)
            | T::DynamicArray(inner)
            | T::Variant {
                base: inner, ..
            } => {
                self.local_operator_application_origins(inner, lexical)?;
            }
            T::FixedArray {
                element, ..
            } => self.local_operator_application_origins(element, lexical)?,
            T::Procedure(procedure) => {
                for parameter in procedure.parameters.iter().chain(&procedure.results) {
                    self.local_operator_application_origins(&parameter.ty, lexical)?;
                }
            }
            T::Restricted {
                restriction, ..
            } => match restriction {
                syntax::TypeRestrictionSyntax::Nominal(ty)
                | syntax::TypeRestrictionSyntax::Interface(ty) => {
                    self.local_operator_application_origins(ty, lexical)?;
                }
            },
            T::TypeOf(expression) => {
                if let Some(ty) =
                    crate::modules::aggregates::parameterized::type_expression(expression)
                {
                    self.local_operator_application_origins(&ty, lexical)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub(crate) fn refine_local_operator_match(
        &mut self,
        candidate: &Candidate<LocalDeclarationId>,
        source: &[syntax::CallArgument],
        arguments: &[Argument],
        mut matched: Match<LocalDeclarationId>,
        span: Span,
    ) -> Result<Match<LocalDeclarationId>, Diagnostic> {
        self.validate_matched_source_policies(candidate, source, &matched)?;
        self.collect_local_operator_policies(candidate, source, &mut matched, span)?;
        self.refine_local_operator_modifier(&mut matched, span)?;
        self.collect_local_operator_policies(candidate, source, &mut matched, span)?;
        crate::overloads::recheck_match_with_nominals(
            self.types, self, candidate, arguments, matched, span,
        )
    }

    fn collect_local_operator_policies(
        &mut self,
        candidate: &Candidate<LocalDeclarationId>,
        source: &[syntax::CallArgument],
        matched: &mut Match<LocalDeclarationId>,
        span: Span,
    ) -> Result<(), Diagnostic> {
        let mut policies = Vec::new();
        for (index, parameter) in candidate.parameters.iter().enumerate() {
            let target = self.local_operator_pattern(&parameter.ty, &matched.substitution, span)?;
            for (occurrence, binding) in matched
                .bindings
                .iter()
                .filter(|binding| binding.parameter == index)
                .enumerate()
            {
                let argument = source.get(binding.argument).ok_or_else(|| {
                    Diagnostic::new(span, "local operator policy lacks its original argument")
                })?;
                if let Some(policy) = self.specialization_callable_policy(
                    &argument.value,
                    target,
                    argument.value.span,
                )? {
                    policies.push(CallablePolicyBinding {
                        name: parameter.name,
                        occurrence,
                        policy,
                    });
                }
            }
        }
        policies.extend(self.result_type_callable_policies(candidate, source, matched, span)?);
        matched.substitution.callables = policies;
        Ok(())
    }

    fn local_operator_pattern(
        &mut self,
        pattern: &crate::overloads::TypePattern,
        substitution: &Substitution,
        span: Span,
    ) -> Result<TypeId, Diagnostic> {
        match self.graph_scope {
            Some(scope) => scope.materialize_pattern(
                pattern,
                substitution,
                self.types,
                &mut self.meta.record_specializations,
                span,
            ),
            None => crate::polymorphism::materialize(self.types, pattern, substitution).map_err(
                |error| Diagnostic::new(span, format!("invalid lexical operator type: {error:?}")),
            ),
        }
    }

    pub(crate) fn preview_local_operator_match(
        &mut self,
        matched: &Match<LocalDeclarationId>,
        span: Span,
    ) -> Result<ArgumentInfo, Diagnostic> {
        let definition = self.local_operator_definition(matched.declaration, span)?;
        self.with_local_procedure_substitution(
            &definition.environment,
            &matched.substitution,
            span,
            |resolver| {
                let [result] = definition.template.results.as_slice() else {
                    return Err(Diagnostic::new(span, "operator requires one result"));
                };
                let ty =
                    resolver.local_operator_pattern(&result.ty, &matched.substitution, span)?;
                Ok(ArgumentInfo::typed(ty))
            },
        )
    }

    fn local_operator_definition(
        &self,
        declaration: LocalDeclarationId,
        span: Span,
    ) -> Result<Definition, Diagnostic> {
        self.meta
            .local_declarations
            .generic_procedures
            .definitions
            .get(&declaration)
            .cloned()
            .ok_or_else(|| {
                Diagnostic::new(span, "selected lexical operator template is unavailable")
            })
    }

    fn local_operator_signature(
        &mut self,
        procedure: ProcedureId,
        definition: &Definition,
        substitution: &Substitution,
        readiness: HeaderReadiness,
    ) -> Result<Signature, Diagnostic> {
        let source = &definition.source;
        let mut parameters = Vec::new();
        for (parameter, source_parameter) in definition
            .template
            .candidate
            .parameters
            .iter()
            .zip(&source.parameters)
        {
            if parameter.is_baked(substitution) {
                return Err(Diagnostic::new(
                    source_parameter.span,
                    "captured operator formals require runtime evaluation",
                ));
            }
            let ty =
                self.local_operator_pattern(&parameter.ty, substitution, source_parameter.span)?;
            let expression = match &source_parameter.binding {
                syntax::ParameterBinding::Defaulted {
                    expression, ..
                }
                | syntax::ParameterBinding::DefaultedType {
                    expression, ..
                } => Some(expression),
                _ => None,
            };
            let default = expression
                .filter(|_| readiness == HeaderReadiness::Complete)
                .map(|expression| self.parameter_default(expression, ty))
                .transpose()?;
            parameters.push(ParameterSignature {
                name: parameter.name,
                ty,
                default,
                evaluation: parameter.evaluation,
            });
        }
        let mut results = Vec::new();
        for (result, source_result) in definition.template.results.iter().zip(&source.results) {
            let ty = self.local_operator_pattern(&result.ty, substitution, source_result.span)?;
            let expression = match &source_result.binding {
                syntax::ResultBinding::Typed {
                    default, ..
                } => default.as_ref(),
                syntax::ResultBinding::InferredDefault(expression) => Some(expression),
            };
            let default = expression
                .filter(|_| readiness == HeaderReadiness::Complete)
                .map(|expression| self.local_typed_constant(expression, ty))
                .transpose()?;
            results.push(ResultSignature {
                name: result.name,
                ty,
                default,
                usage: result.usage,
            });
        }
        crate::procedure_values::signatures::normalize_results(
            &mut results,
            self.types,
            |result| result.ty,
        );
        let ty = self
            .types
            .procedure(ProcedureType {
                parameters: parameters
                    .iter()
                    .filter(|parameter| {
                        parameter.evaluation == syntax::ParameterEvaluation::Evaluate
                    })
                    .map(|parameter| parameter.ty)
                    .collect(),
                results: results.iter().map(|result| result.ty).collect(),
                return_abi: source.return_abi,
                convention: source.convention,
                context: source.context,
                variadic: jai_types::Variadic::None,
            })
            .map_err(|error| Diagnostic::new(source.span, error.to_string()))?;
        let signature = Signature {
            id: procedure,
            ty,
            parameters,
            source_variadic: crate::overloads::CandidateVariadic::None,
            results,
        };
        self.validate_source_operator_signature(
            source.operator.expect("template is an operator").kind,
            &signature,
            &source.parameters,
            source.span,
        )?;
        Ok(signature)
    }
}
