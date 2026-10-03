//! Captured source contracts follow real expression-binding identities and scopes.
use super::*;
#[path = "selected_call_producers.rs"]
mod selected_call_producers;
use jai_ir::ExpressionBindingId;

impl Resolver<'_> {
    pub(super) fn captured_record_contract(
        &mut self,
        ty: TypeId,
        initializers: &[(jai_types::FieldId, &ValueExpr)],
        span: Span,
        depth: usize,
    ) -> Result<Option<ValueContract>, Diagnostic> {
        let Some(scope) = self.graph_scope else {
            return Ok(None);
        };
        let file = self
            .meta
            .record_specializations
            .record(ty)
            .map_or(scope.code_origin().0, |record| record.file);
        let Some(mut contract) =
            self.record_shape_contract(ty, file, &HashMap::new(), span, depth + 1)?
        else {
            return Ok(None);
        };
        for (field, producer) in initializers {
            let declared = self.record_field_value_contract(&contract, *field, span, depth + 1)?;
            let captured = self.callback_value_contract_inner(producer, span, depth + 1)?;
            let value = self.retain_captured_contract(declared, captured, span, depth + 1)?;
            if let Some(value) = value {
                let ContractKind::Record(record) = &mut contract.kind else {
                    unreachable!()
                };
                record.fields.insert(*field, value);
            }
        }
        Ok(Some(contract))
    }

    fn retain_captured_contract(
        &self,
        declared: Option<ValueContract>,
        captured: Option<ValueContract>,
        span: Span,
        depth: usize,
    ) -> Result<Option<ValueContract>, Diagnostic> {
        if depth >= crate::constant_limits::MAX_CONSTANT_DEPTH {
            return Err(Diagnostic::new(
                span,
                "captured aggregate contract exceeds field depth",
            ));
        }
        let (mut declared, captured) = match (declared, captured) {
            (Some(declared), Some(captured)) => (declared, captured),
            (declared, captured) => return Ok(declared.or(captured)),
        };
        if declared.ty != captured.ty {
            return Err(Diagnostic::new(
                span,
                "captured aggregate contract has a different field type",
            ));
        }
        match captured.kind {
            ContractKind::Record(captured) => {
                for (field, value) in captured.fields {
                    let annotation =
                        self.record_field_value_contract(&declared, field, span, depth + 1)?;
                    let value =
                        self.retain_captured_contract(annotation, Some(value), span, depth + 1)?;
                    if let (ContractKind::Record(record), Some(value)) = (&mut declared.kind, value)
                    {
                        record.fields.insert(field, value);
                    }
                }
            }
            ContractKind::Pointer(captured) | ContractKind::Sequence(captured) => {
                if let ContractKind::Pointer(inner) | ContractKind::Sequence(inner) =
                    &mut declared.kind
                    && let Some(value) = self.retain_captured_contract(
                        Some((**inner).clone()),
                        Some(*captured),
                        span,
                        depth + 1,
                    )?
                {
                    **inner = value;
                }
            }
            ContractKind::Callable {
                ..
            } => {}
        }
        // Callable leaves retain the destination's checked source policy.
        Ok(Some(declared))
    }

    pub(crate) fn capture_expression_value_contract(
        &mut self,
        binding: ExpressionBindingId,
        source: &syntax::Expression,
        checked: &ValueExpr,
        span: Span,
    ) -> Result<(), Diagnostic> {
        self.validate_expression_contract_producer(binding, span)?;
        let contract = self.callback_expression_contract(source, checked, span)?;
        self.publish_expression_contract(binding, checked, contract, span)
    }

    pub(crate) fn capture_checked_expression_value_contract(
        &mut self,
        binding: ExpressionBindingId,
        checked: &ValueExpr,
        span: Span,
    ) -> Result<(), Diagnostic> {
        self.validate_expression_contract_producer(binding, span)?;
        let contract = self.callback_value_contract(checked, span)?;
        self.publish_expression_contract(binding, checked, contract, span)
    }

    pub(super) fn validate_expression_contract_producer(
        &self,
        binding: ExpressionBindingId,
        span: Span,
    ) -> Result<(), Diagnostic> {
        let owner = self
            .expression_owner
            .map(|owner| self.compile_time.map_or(owner, |context| context.owner));
        if owner != Some(binding.procedure()) {
            return Err(Diagnostic::new(
                span,
                "captured callback binding belongs to another source owner",
            ));
        }
        if self
            .meta
            .callbacks
            .expression_producers
            .contains_key(&binding)
        {
            return Err(Diagnostic::new(
                span,
                "expression callback binding already has a producer",
            ));
        }
        Ok(())
    }

    pub(super) fn publish_expression_contract(
        &mut self,
        binding: ExpressionBindingId,
        checked: &ValueExpr,
        contract: Option<ValueContract>,
        span: Span,
    ) -> Result<(), Diagnostic> {
        if contract
            .as_ref()
            .is_some_and(|contract| contract.ty != checked.type_id(self.types))
        {
            return Err(Diagnostic::new(
                span,
                "captured callback contract has a different producer type",
            ));
        }
        self.meta
            .callbacks
            .expression_producers
            .insert(binding, contract);
        Ok(())
    }

    pub(super) fn expression_binding_contract(
        &self,
        binding: ExpressionBindingId,
        ty: TypeId,
        span: Span,
    ) -> Result<Option<ValueContract>, Diagnostic> {
        let contract = self
            .meta
            .callbacks
            .expression_bindings
            .get(&binding)
            .or_else(|| self.meta.callbacks.expression_producers.get(&binding))
            .cloned()
            .flatten();
        if contract.as_ref().is_some_and(|contract| contract.ty != ty) {
            return Err(Diagnostic::new(
                span,
                "bound callback contract has a different captured type",
            ));
        }
        Ok(contract)
    }

    pub(super) fn bound_expression_contract(
        &mut self,
        bindings: &[(ExpressionBindingId, ValueExpr)],
        body: &ValueExpr,
        span: Span,
        depth: usize,
    ) -> Result<Option<ValueContract>, Diagnostic> {
        let mut previous = Vec::with_capacity(bindings.len());
        let result = (|| {
            for (binding, producer) in bindings {
                let contract = match self.meta.callbacks.expression_producers.get(binding) {
                    Some(contract) => contract.clone(),
                    None => self.callback_value_contract_inner(producer, span, depth + 1)?,
                };
                if contract
                    .as_ref()
                    .is_some_and(|contract| contract.ty != producer.type_id(self.types))
                {
                    return Err(Diagnostic::new(
                        span,
                        "binding callback contract has a different producer type",
                    ));
                }
                previous.push((
                    *binding,
                    self.meta
                        .callbacks
                        .expression_bindings
                        .insert(*binding, contract),
                ));
            }
            self.callback_value_contract_inner(body, span, depth + 1)
        })();
        for (binding, previous) in previous.into_iter().rev() {
            match previous {
                Some(previous) => {
                    self.meta
                        .callbacks
                        .expression_bindings
                        .insert(binding, previous);
                }
                None => {
                    self.meta.callbacks.expression_bindings.remove(&binding);
                }
            }
        }
        result
    }
}
