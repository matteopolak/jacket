//! Baked-formal contracts retain source policy outside runtime ABI slots.
use super::*;

impl Resolver<'_> {
    pub(crate) fn baked_callback_call_source(
        &self,
        path: &syntax::NamePath,
        span: Span,
    ) -> Result<Option<syntax::Expression>, Diagnostic> {
        if self.baked_callable_type(path).is_none() {
            return Ok(None);
        }
        self.checked_baked_callback_binding_contract(path, span)?
            .ok_or_else(|| {
                Diagnostic::new(span, "baked callback has no checked source contract")
            })?;
        Ok(Some(syntax::Expression {
            kind: if path.members.is_empty() {
                syntax::ExpressionKind::Name(path.root)
            } else {
                syntax::ExpressionKind::QualifiedName(path.clone())
            },
            span,
        }))
    }

    pub(crate) fn baked_callback_binding_contract(
        &self,
        path: &syntax::NamePath,
    ) -> Option<ValueContract> {
        let ty = self.baked_callback_formal_type(path)?;
        self.meta
            .callbacks
            .generic_parameters
            .get(&(self.procedure, path.root))
            .filter(|contract| contract.ty == ty)
            .cloned()
    }

    pub(crate) fn checked_baked_callback_binding_contract(
        &self,
        path: &syntax::NamePath,
        span: Span,
    ) -> Result<Option<ValueContract>, Diagnostic> {
        let Some(ty) = self.baked_callback_formal_type(path) else {
            return Ok(None);
        };
        let contract = self.baked_callback_binding_contract(path);
        if contract.is_none() && self.callback_contract_type(ty, 0) {
            return Err(Diagnostic::new(
                span,
                "baked callback has no checked source contract",
            ));
        }
        Ok(contract)
    }

    fn baked_callback_formal_type(&self, path: &syntax::NamePath) -> Option<TypeId> {
        if !path.members.is_empty() || self.local_name_present(path.root) {
            return None;
        }
        let scope = self.graph_scope?;
        let (_, parameters, _) = scope.callback_contract_header(self.procedure)?;
        if !parameters.iter().any(|parameter| {
            parameter.name == path.root && parameter.baking != syntax::ParameterBaking::None
        }) {
            return None;
        }
        let substitution = scope.callback_contract_substitution(self.procedure)?;
        let crate::polymorphism::BakedValue::Value(value) = substitution.constant(path.root)?
        else {
            return None;
        };
        Some(value.ty)
    }

    pub(crate) fn capture_baked_callback_variables(
        &mut self,
        procedure: ProcedureId,
        source: &syntax::Parameter,
        argument: Option<&syntax::Expression>,
        bindings: &mut HashMap<Symbol, Option<ValueContract>>,
        inferred: &mut HashMap<Symbol, ValueContract>,
        span: Span,
    ) -> Result<bool, Diagnostic> {
        if source.baking == syntax::ParameterBaking::None {
            return Ok(false);
        }
        let Some(scope) = self.graph_scope else {
            return Ok(false);
        };
        let Some(substitution) = scope.callback_contract_substitution(procedure) else {
            return Ok(false);
        };
        let Some(crate::polymorphism::BakedValue::Value(value)) =
            substitution.constant(source.name)
        else {
            return Ok(false);
        };
        let annotation = match &source.binding {
            syntax::ParameterBinding::RequiredType(ty)
            | syntax::ParameterBinding::DefaultedType { ty: Some(ty), .. } => Some(ty),
            syntax::ParameterBinding::DefaultedType { ty: None, .. }
            | syntax::ParameterBinding::Defaulted { ty: None, .. } => None,
            _ => return Ok(true),
        };
        let Some(annotation) = annotation else {
            if !self.callback_contract_type(value.ty, 0) {
                return Ok(true);
            }
            let contract = match argument {
                Some(argument) => self
                    .preview_expected_callback_contract(argument, value.ty, argument.span)?
                    .ok_or_else(|| {
                        Diagnostic::new(
                            argument.span,
                            "inferred baked callback requires its checked argument contract",
                        )
                    })?,
                None => {
                    return Err(Diagnostic::new(
                        source.span,
                        "inferred baked callback default requires its checked source contract",
                    ));
                }
            };
            inferred.insert(source.name, contract);
            return Ok(true);
        };
        if let Some(argument) = argument {
            let contract =
                self.preview_expected_callback_contract(argument, value.ty, argument.span)?;
            self.capture_generic_callback_variables(annotation, contract.as_ref(), bindings, span)?;
        } else if self.callback_contract_type(value.ty, 0) {
            // A type annotation can provide its own policy; an inferred variable
            // default needs a separate checked proof from its defining scope.
            let (file, _, _) = scope.callback_contract_header(procedure).ok_or_else(|| {
                Diagnostic::new(span, "baked callback has no original source header")
            })?;
            let retained = scope.callback_contract_syntax_in_specialization(
                file,
                Some(&substitution),
                annotation,
                span,
            )?;
            let contract = self
                .bound_generic_callback_contract(value.ty, &retained, bindings, span)?
                .ok_or_else(|| {
                    Diagnostic::new(
                        span,
                        "baked callback default requires its checked source contract",
                    )
                })?;
            self.capture_generic_callback_variables(annotation, Some(&contract), bindings, span)?;
        }
        Ok(true)
    }

    pub(crate) fn bind_baked_callback_parameter(
        &mut self,
        procedure: ProcedureId,
        source: &syntax::Parameter,
        bindings: &HashMap<Symbol, Option<ValueContract>>,
        inferred: &HashMap<Symbol, ValueContract>,
        span: Span,
    ) -> Result<bool, Diagnostic> {
        if source.baking == syntax::ParameterBaking::None {
            return Ok(false);
        }
        let Some(scope) = self.graph_scope else {
            return Ok(false);
        };
        let Some(substitution) = scope.callback_contract_substitution(procedure) else {
            return Ok(false);
        };
        let Some(crate::polymorphism::BakedValue::Value(value)) =
            substitution.constant(source.name)
        else {
            return Ok(false);
        };
        let annotation = match &source.binding {
            syntax::ParameterBinding::RequiredType(ty)
            | syntax::ParameterBinding::DefaultedType { ty: Some(ty), .. } => ty,
            syntax::ParameterBinding::DefaultedType { ty: None, .. }
            | syntax::ParameterBinding::Defaulted { ty: None, .. } => {
                if !self.callback_contract_type(value.ty, 0) {
                    return Ok(false);
                }
                let contract = inferred.get(&source.name).ok_or_else(|| {
                    Diagnostic::new(
                        source.span,
                        "inferred baked callback has no checked source contract",
                    )
                })?;
                if contract.ty != value.ty {
                    return Err(Diagnostic::new(
                        source.span,
                        "inferred baked callback contract does not match its checked value",
                    ));
                }
                return Ok(self.merge_generic_callback_parameter(
                    procedure,
                    source.name,
                    contract.clone(),
                ));
            }
            _ => return Ok(false),
        };
        let (file, _, _) = scope
            .callback_contract_header(procedure)
            .ok_or_else(|| Diagnostic::new(span, "baked callback has no original source header"))?;
        let retained = scope.callback_contract_syntax_in_specialization(
            file,
            Some(&substitution),
            annotation,
            span,
        )?;
        match self.bound_generic_callback_contract(value.ty, &retained, bindings, span)? {
            Some(contract) => {
                Ok(self.merge_generic_callback_parameter(procedure, source.name, contract))
            }
            None => Ok(false),
        }
    }
}
