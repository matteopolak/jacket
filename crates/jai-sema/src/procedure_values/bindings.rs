//! Source callable names, defaults and result contracts belong to bindings.
use super::*;
use jai_types::FieldId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum CallbackArgumentPolicy {
    Established,
    Ambiguous,
}
#[derive(Clone)]
pub(crate) struct CallbackSignature {
    pub(crate) argument_policy: CallbackArgumentPolicy,
    pub(crate) ty: TypeId,
    pub(crate) parameters: Vec<CallbackParameter>,
    pub(crate) source_variadic: crate::overloads::CandidateVariadic,
    pub(crate) results: Vec<ResultSignature>,
}

#[derive(Clone)]
pub(crate) struct CallbackParameter {
    pub(crate) name: Option<Symbol>,
    pub(crate) default: Option<ParameterDefault>,
    pub(crate) ty: TypeId,
    pub(crate) evaluation: syntax::ParameterEvaluation,
}

#[derive(Default)]
pub(crate) struct CallbackRegistry {
    pub(crate) expression_producers:
        HashMap<jai_ir::ExpressionBindingId, Option<super::contracts::ValueContract>>,
    pub(crate) expression_bindings:
        HashMap<jai_ir::ExpressionBindingId, Option<super::contracts::ValueContract>>,
    pub(crate) discarded_defaults: HashMap<(ProcedureId, Symbol), super::contracts::ValueContract>,
    pub(crate) source_annotations: super::source_annotations::SourceProcedureAnnotations,
    pub(crate) places: HashMap<Place, CallbackSignature>,
    pub(crate) value_contracts: HashMap<Place, super::contracts::ValueContract>,
    pub(crate) field_contracts: HashMap<FieldId, super::contracts::ValueContract>,
    pub(crate) returned_contracts:
        HashMap<ProcedureId, Vec<Option<super::contracts::ValueContract>>>,
    pub(crate) generic_parameters: HashMap<(ProcedureId, Symbol), super::contracts::ValueContract>,
    pub(crate) generic_type_arguments:
        HashMap<(ProcedureId, Symbol), super::contracts::ValueContract>,
}

impl CallbackSignature {
    pub(crate) fn source(signature: &Signature) -> Self {
        Self {
            argument_policy: CallbackArgumentPolicy::Established,
            ty: signature.ty,
            source_variadic: signature.source_variadic,
            results: signature.results.clone(),
            parameters: signature
                .parameters
                .iter()
                .map(|parameter| CallbackParameter {
                    name: Some(parameter.name),
                    default: parameter.default.clone(),
                    ty: parameter.ty,
                    evaluation: parameter.evaluation,
                })
                .collect(),
        }
    }
    pub(crate) fn annotation(
        ty: TypeId,
        syntax: &syntax::ProcedureTypeSyntax,
        types: &TypeRegistry,
        source_parameters: &[TypeId],
        defaults: &[Option<ParameterDefault>],
    ) -> Self {
        Self {
            argument_policy: CallbackArgumentPolicy::Established,
            ty,
            source_variadic: syntax
                .parameters
                .iter()
                .position(|parameter| parameter.variadic)
                .map_or(crate::overloads::CandidateVariadic::None, |parameter| {
                    if syntax.convention == CallingConvention::C {
                        crate::overloads::CandidateVariadic::C {
                            fixed_parameters: source_parameters.len(),
                        }
                    } else {
                        crate::overloads::CandidateVariadic::Jai {
                            parameter,
                        }
                    }
                }),
            results: types
                .procedure_definition(ty)
                .expect("checked callback type")
                .results
                .iter()
                .zip(&syntax.results)
                .map(|(ty, result)| ResultSignature {
                    name: result.name,
                    ty: *ty,
                    default: None,
                    usage: result.usage,
                })
                .collect(),
            parameters: syntax
                .parameters
                .iter()
                .filter(|parameter| {
                    !(parameter.variadic && syntax.convention == CallingConvention::C)
                })
                .zip(source_parameters)
                .enumerate()
                .map(|(index, (parameter, ty))| CallbackParameter {
                    name: parameter.name,
                    default: defaults.get(index).cloned().flatten(),
                    ty: *ty,
                    evaluation: parameter.evaluation,
                })
                .collect(),
        }
    }
}

impl Resolver<'_> {
    pub(crate) fn bind_callback_annotation(
        &mut self,
        place: Place,
        syntax: &syntax::TypeSyntax,
        span: Span,
    ) -> Result<(), Diagnostic> {
        let contract = self.annotation_value_contract(place.ty(), syntax, span)?;
        if let Some(metadata) = contract
            .as_ref()
            .and_then(|contract| contract.callback())
            .cloned()
        {
            self.meta.callbacks.places.insert(place, metadata);
        }
        self.bind_value_contract(place, contract, span)
    }

    pub(crate) fn bind_callback_parameter(
        &mut self,
        place: Place,
        parameter: &syntax::Parameter,
        default: Option<&ParameterDefault>,
    ) -> Result<(), Diagnostic> {
        let syntax = match &parameter.binding {
            syntax::ParameterBinding::RequiredType(ty)
            | syntax::ParameterBinding::DefaultedType {
                ty: Some(ty), ..
            } => Some(ty),
            syntax::ParameterBinding::DefaultedType {
                ty: None,
                expression,
            }
            | syntax::ParameterBinding::Defaulted {
                ty: None,
                expression,
            } => match &expression.kind {
                syntax::ExpressionKind::TypeCast {
                    ty, ..
                } => Some(ty),
                _ => None,
            },
            _ => None,
        };
        if let Some(syntax) = syntax {
            let explicit = self
                .annotation_value_contract(place.ty(), syntax, parameter.span)?
                .is_some();
            self.bind_callback_annotation(place, syntax, parameter.span)?;
            if !explicit
                && let Some(contract) = self
                    .meta
                    .callbacks
                    .generic_parameters
                    .get(&(self.procedure, parameter.name))
                    .cloned()
            {
                self.bind_value_contract(place, Some(contract), parameter.span)?;
            }
        } else if let Some(ParameterDefault::Constant(value)) =
            default.and_then(ParameterDefault::prepared)
        {
            let contract =
                self.callback_value_contract(&value.clone().into_expression(), parameter.span)?;
            if let Some(metadata) = contract
                .as_ref()
                .and_then(|contract| contract.callback())
                .cloned()
            {
                self.meta.callbacks.places.insert(place, metadata);
            }
            self.bind_value_contract(place, contract, parameter.span)?;
        }
        Ok(())
    }

    pub(crate) fn bind_callback_declaration(
        &mut self,
        place: Place,
        declaration: &syntax::Declaration,
        value: Option<&Expr>,
        span: Span,
    ) -> Result<(), Diagnostic> {
        if let syntax::Declaration::UnresolvedExplicit {
            ty, ..
        }
        | syntax::Declaration::External {
            ty, ..
        } = declaration
        {
            return self.bind_callback_annotation(place, ty, span);
        }
        if let Some(
            Expr::Typed {
                value, ..
            }
            | Expr::Pointer {
                value, ..
            },
        ) = value
        {
            let initializer = match declaration {
                syntax::Declaration::GroupMember {
                    ..
                } => {
                    return Err(Diagnostic::new(
                        self.span,
                        "file declaration group requires its source publication owner",
                    ));
                }
                syntax::Declaration::External {
                    ..
                } => None,
                syntax::Declaration::Inferred {
                    initializer, ..
                } => Some(initializer),
                syntax::Declaration::Explicit {
                    initializer, ..
                }
                | syntax::Declaration::UnresolvedExplicit {
                    initializer, ..
                } => initializer.as_ref(),
            };
            let contract = match initializer {
                Some(source) => self.callback_expression_contract(source, value, span)?,
                None => self.callback_value_contract(value, span)?,
            };
            if let Some(metadata) = contract
                .as_ref()
                .and_then(|contract| contract.callback())
                .cloned()
            {
                self.meta.callbacks.places.insert(place, metadata);
            }
            self.bind_value_contract(place, contract, span)?;
        }
        Ok(())
    }

    pub(crate) fn callback_value_metadata(
        &mut self,
        value: &ValueExpr,
        span: Span,
    ) -> Result<Option<CallbackSignature>, Diagnostic> {
        Ok(self
            .callback_value_contract(value, span)?
            .and_then(|contract| contract.callback().cloned()))
    }
}
