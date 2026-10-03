//! Checked source header components exist before a callable identity is reserved.
use super::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum SourceHeaderPhase {
    Preview,
    Definition {
        procedure: ProcedureId,
        readiness: HeaderReadiness,
    },
}
impl SourceHeaderPhase {
    fn readiness(self) -> HeaderReadiness {
        match self {
            Self::Preview => HeaderReadiness::TypesOnly,
            Self::Definition {
                readiness, ..
            } => readiness,
        }
    }
    fn complete_owner(self) -> Option<ProcedureId> {
        match self {
            Self::Definition {
                procedure,
                readiness: HeaderReadiness::Complete,
            } => Some(procedure),
            _ => None,
        }
    }
}
#[derive(Clone, Copy)]
enum SourceResultShape {
    Ordered,
    NormalizedVoid,
}
pub(crate) struct CheckedSourceHeader {
    source_results: SourceResultShape,
    pub(crate) ty: TypeId,
    pub(crate) parameters: Vec<ParameterSignature>,
    pub(crate) source_variadic: crate::overloads::CandidateVariadic,
    pub(crate) results: Vec<ResultSignature>,
}
impl CheckedSourceHeader {
    pub(crate) fn result_sources<'a>(
        &self,
        source: &'a [syntax::ProcedureResult],
        span: Span,
    ) -> Result<&'a [syntax::ProcedureResult], Diagnostic> {
        match self.source_results {
            SourceResultShape::NormalizedVoid if source.len() == 1 && self.results.is_empty() => {
                Ok(&source[..0])
            }
            SourceResultShape::Ordered if source.len() == self.results.len() => Ok(source),
            _ => Err(Diagnostic::new(
                span,
                "source result metadata differs from checked anonymous header",
            )),
        }
    }

    pub(super) fn with_identity(self, id: ProcedureId) -> Signature {
        Signature {
            id,
            ty: self.ty,
            parameters: self.parameters,
            source_variadic: self.source_variadic,
            results: self.results,
        }
    }
    pub(crate) fn callback_metadata(&self) -> crate::procedure_values::bindings::CallbackSignature {
        use crate::procedure_values::bindings::{
            CallbackArgumentPolicy, CallbackParameter, CallbackSignature,
        };
        CallbackSignature {
            argument_policy: CallbackArgumentPolicy::Established,
            ty: self.ty,
            source_variadic: self.source_variadic,
            parameters: self
                .parameters
                .iter()
                .map(|parameter| CallbackParameter {
                    name: Some(parameter.name),
                    ty: parameter.ty,
                    default: parameter.default.clone(),
                    evaluation: parameter.evaluation,
                })
                .collect(),
            results: self.results.clone(),
        }
    }
}
impl Resolver<'_> {
    pub(super) fn source_header_components(
        &mut self,
        source: CallableSource<'_>,
        phase: SourceHeaderPhase,
    ) -> Result<CheckedSourceHeader, Diagnostic> {
        let readiness = phase.readiness();
        let mut names = HashSet::new();
        let mut parameters = Vec::new();
        for parameter in source.parameters {
            if !names.insert(parameter.name) {
                return Err(Diagnostic::new(parameter.span, "duplicate parameter name"));
            }
            if parameter.baking != syntax::ParameterBaking::None {
                return Err(Diagnostic::new(
                    parameter.span,
                    "local polymorphic procedures require a lexical specialization origin",
                ));
            }
            if parameter.variadic && source.convention == CallingConvention::C {
                continue;
            }
            let (ty, expression) = match &parameter.binding {
                syntax::ParameterBinding::Required(ty) => (self.types.scalar(*ty), None),
                syntax::ParameterBinding::RequiredType(ty) => (
                    self.source_header_annotation(ty, parameter.span, phase)?,
                    None,
                ),
                syntax::ParameterBinding::Defaulted {
                    ty,
                    expression,
                } => (
                    match ty {
                        Some(ty) => self.types.scalar(*ty),
                        None => self.source_header_default_type(expression, phase)?,
                    },
                    Some(expression),
                ),
                syntax::ParameterBinding::DefaultedType {
                    ty,
                    expression,
                } => (
                    match ty {
                        Some(ty) => self.source_header_annotation(ty, parameter.span, phase)?,
                        None => self.source_header_default_type(expression, phase)?,
                    },
                    Some(expression),
                ),
            };
            if parameter.evaluation == syntax::ParameterEvaluation::Discard {
                if let Some(procedure) = phase.complete_owner()
                    && let Some(expression) = expression
                {
                    self.check_discarded_argument(expression, ty)?;
                    if let Some(contract) = self.preview_callback_expression_contract(expression)?
                        && contract.ty == ty
                    {
                        self.meta
                            .callbacks
                            .discarded_defaults
                            .insert((procedure, parameter.name), contract);
                    }
                }
                parameters.push(ParameterSignature {
                    name: parameter.name,
                    ty,
                    default: expression.map(|_| ParameterDefault::Discarded),
                    evaluation: parameter.evaluation,
                });
                continue;
            }
            let default = match expression {
                Some(expression) if phase == SourceHeaderPhase::Preview => {
                    Some(self.preview_source_parameter_default(expression, ty)?)
                }
                Some(expression)
                    if matches!(expression.kind, syntax::ExpressionKind::CallerLocation)
                        && readiness == HeaderReadiness::TypesOnly =>
                {
                    Some(ParameterDefault::CallerLocation)
                }
                Some(expression)
                    if matches!(
                        expression.kind,
                        syntax::ExpressionKind::Code(syntax::CodeBody::Null)
                    ) =>
                {
                    Some(self.parameter_default(expression, ty)?)
                }
                Some(_) if readiness == HeaderReadiness::TypesOnly => None,
                Some(expression) => Some(self.parameter_default(expression, ty)?),
                None => None,
            };
            parameters.push(ParameterSignature {
                name: parameter.name,
                ty,
                default,
                evaluation: parameter.evaluation,
            });
        }
        let mut results = Vec::new();
        let mut names = HashSet::new();
        for result in source.results {
            if result.name.is_some_and(|name| !names.insert(name)) {
                return Err(Diagnostic::new(result.span, "duplicate result name"));
            }
            let (ty, expression) = match &result.binding {
                syntax::ResultBinding::Typed {
                    ty,
                    default,
                } => (
                    self.source_header_annotation(ty, result.span, phase)?,
                    default.as_ref(),
                ),
                syntax::ResultBinding::InferredDefault(expression) => (
                    self.source_header_default_type(expression, phase)?,
                    Some(expression),
                ),
            };
            let default = match expression {
                Some(_) if readiness == HeaderReadiness::TypesOnly => None,
                Some(expression) => Some(self.local_typed_constant(expression, ty)?),
                None => None,
            };
            results.push(ResultSignature {
                name: result.name,
                ty,
                default,
                usage: result.usage,
            });
        }
        let source_results = if results.len() == 1 && results[0].ty == self.types.void() {
            SourceResultShape::NormalizedVoid
        } else {
            SourceResultShape::Ordered
        };
        crate::procedure_values::signatures::normalize_results(
            &mut results,
            self.types,
            |result| result.ty,
        );
        let variadic = crate::procedure_values::signatures::normalize_variadic(
            source.parameters,
            &mut parameters,
            source.convention,
            self.types,
            source.span,
        )?;
        let source_variadic = crate::procedure_values::signatures::source_variadic(
            source.parameters,
            &parameters,
            source.convention,
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
                variadic,
            })
            .map_err(|error| Diagnostic::new(source.span, error.to_string()))?;

        if let Some(procedure) = phase.complete_owner() {
            self.register_result_contracts(procedure, source.results, &results)?;
        }
        Ok(CheckedSourceHeader {
            source_results,
            ty,
            parameters,
            source_variadic,
            results,
        })
    }
    pub(crate) fn preview_source_parameter_default(
        &mut self,
        expression: &syntax::Expression,
        ty: TypeId,
    ) -> Result<ParameterDefault, Diagnostic> {
        if matches!(expression.kind, syntax::ExpressionKind::CallerLocation) {
            return Ok(ParameterDefault::CallerLocation);
        }
        if matches!(
            expression.kind,
            syntax::ExpressionKind::Code(syntax::CodeBody::Null)
        ) {
            if ty != self.types.code_type() {
                return Err(Diagnostic::new(
                    expression.span,
                    "#code,null requires a compile-time Code parameter",
                ));
            }
            return Ok(ParameterDefault::CodeNull {
                ty,
            });
        }
        if let Some(read) = self.ready_runtime_parameter_default(expression, Some(ty))? {
            return Ok(ParameterDefault::RuntimeRead(read));
        }
        let info = self.describe_argument(expression)?;
        let value = crate::overloads::bake(
            self.types,
            &crate::overloads::TypePattern::Concrete(ty),
            &info,
            &crate::polymorphism::Substitution::default(),
            expression.span,
        )?;
        value
            .into_runtime(ty, self.types)
            .map(ParameterDefault::Constant)
            .map_err(|error| Diagnostic::new(expression.span, error.to_string()))
    }

    fn source_header_annotation(
        &mut self,
        syntax: &syntax::TypeSyntax,
        span: Span,
        phase: SourceHeaderPhase,
    ) -> Result<TypeId, Diagnostic> {
        match phase {
            SourceHeaderPhase::Preview => self.preview_annotation(syntax, span),
            SourceHeaderPhase::Definition {
                ..
            } => self.lexical_annotation(syntax, span),
        }
    }
    fn source_header_default_type(
        &mut self,
        expression: &syntax::Expression,
        phase: SourceHeaderPhase,
    ) -> Result<TypeId, Diagnostic> {
        match phase {
            SourceHeaderPhase::Preview => {
                let description = self.describe_argument(expression)?;
                self.argument_type(&description, expression.span)
            }
            SourceHeaderPhase::Definition {
                ..
            } => self.local_default_type(expression),
        }
    }
}
