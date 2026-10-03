//! Per-use source type arguments describe fields without changing nominal identity.
use super::*;

impl Resolver<'_> {
    pub(super) fn record_application_contract(
        &self,
        ty: TypeId,
        application: &syntax::TypeApplicationSyntax,
        retained: &ContractSyntax,
        outer: &HashMap<Symbol, Option<ValueContract>>,
        span: Span,
        depth: usize,
    ) -> Result<Option<ValueContract>, Diagnostic> {
        if depth >= crate::constant_limits::MAX_CONSTANT_DEPTH {
            return Err(Diagnostic::new(
                span,
                "record callback contract exceeds source type depth",
            ));
        }
        let Some(record) = self.meta.record_specializations.record(ty) else {
            return Ok(None);
        };
        let Some(origin) = record.origin else {
            return Ok(None);
        };
        let Some(scope) = self.graph_scope else {
            return Ok(None);
        };
        let Some(parameters) = scope.callback_record_parameters(origin.0) else {
            return Ok(None);
        };
        let bound = crate::modules::aggregates::parameterized::binder::bind_arguments(
            &parameters,
            &application.arguments,
            span,
        )?;
        let mut bindings = HashMap::new();
        for argument in bound {
            let Some(actual) = record.substitution.ty(argument.parameter.name) else {
                continue;
            };
            let Some(source) = super::generics::source_type(argument.expression) else {
                continue;
            };
            let name = match &source {
                syntax::TypeSyntax::Variable(name) => Some(*name),
                syntax::TypeSyntax::Named(path) if path.members.is_empty() => Some(path.root),
                _ => None,
            };
            let forwarded = name
                .and_then(|name| {
                    if argument.defaulted {
                        bindings.get(&name)
                    } else {
                        outer.get(&name)
                    }
                })
                .cloned();
            let contract = match forwarded {
                Some(contract) => contract,
                None if argument.defaulted => {
                    let source = scope.callback_contract_syntax_in_specialization(
                        record.file,
                        Some(&record.substitution),
                        &source,
                        span,
                    )?;
                    self.normalized_value_contract(actual, &source, span, depth + 1)?
                }
                None => {
                    let index = application
                        .arguments
                        .iter()
                        .position(|candidate| std::ptr::eq(&candidate.value, argument.expression))
                        .expect("bound record argument source");
                    self.bound_syntax_contract(
                        actual,
                        retained.child(ContractStep::Argument(index)),
                        outer,
                        span,
                        depth + 1,
                    )?
                }
            };
            bindings.insert(argument.parameter.name, contract);
        }
        self.record_shape_contract(ty, record.file, &bindings, span, depth + 1)
    }
    pub(super) fn record_shape_contract(
        &self,
        ty: TypeId,
        file: jai_modules::FileInstanceId,
        bindings: &HashMap<Symbol, Option<ValueContract>>,
        span: Span,
        depth: usize,
    ) -> Result<Option<ValueContract>, Diagnostic> {
        if depth >= crate::constant_limits::MAX_CONSTANT_DEPTH {
            return Err(Diagnostic::new(
                span,
                "record callback contract exceeds field depth",
            ));
        }
        if !self.callback_contract_type(ty, depth) {
            return Ok(None);
        }
        Ok(Some(ValueContract {
            ty,
            kind: ContractKind::Record(RecordContract {
                file,
                bindings: bindings.clone(),
                fields: HashMap::new(),
            }),
        }))
    }
    pub(super) fn record_field_value_contract(
        &self,
        contract: &ValueContract,
        field: jai_types::FieldId,
        span: Span,
        depth: usize,
    ) -> Result<Option<ValueContract>, Diagnostic> {
        let ContractKind::Record(source) = &contract.kind else {
            return Ok(None);
        };
        if let Some(value) = source.fields.get(&field) {
            return Ok(Some(value.clone()));
        }
        let metadata = self.record_metadata(contract.ty, span)?;
        let Some(field) = metadata
            .fields
            .iter()
            .find(|candidate| candidate.id == field)
        else {
            return Ok(None);
        };
        let syntax = match field.syntax.as_ref() {
            crate::local_declarations::FieldSourceRef::Named(field) => match &field.binding {
                syntax::FieldBinding::Explicit { ty, .. } => ty.clone(),
                syntax::FieldBinding::Inferred(expression) => match &expression.kind {
                    syntax::ExpressionKind::TypeCast { ty, .. } => ty.clone(),
                    _ => return Ok(None),
                },
            },
            crate::local_declarations::FieldSourceRef::AnonymousRecord(record) => {
                syntax::TypeSyntax::InlineRecord(Box::new(record.clone()))
            }
        };
        if let syntax::TypeSyntax::Named(path) = &syntax
            && path.members.is_empty()
            && let Some(alias) = self.record_callback_alias(contract.ty, path.root)
        {
            return self.bound_syntax_contract(field.ty, &alias, &source.bindings, span, depth + 1);
        }
        let retained = if let Some(scope) = self.graph_scope {
            let substitution = self
                .meta
                .record_specializations
                .record(contract.ty)
                .map(|record| &record.substitution);
            let alias = match &syntax {
                syntax::TypeSyntax::Named(path) if path.members.is_empty() => self
                    .meta
                    .record_specializations
                    .record(contract.ty)
                    .and_then(|record| record.origin)
                    .and_then(|origin| scope.callback_record_alias(origin.0, path.root)),
                _ => None,
            };
            scope.callback_contract_syntax_in_specialization(
                source.file,
                substitution,
                alias.as_ref().unwrap_or(&syntax),
                span,
            )?
        } else {
            self.retained_callback_syntax(&syntax, span)?
        };
        self.bound_syntax_contract(field.ty, &retained, &source.bindings, span, depth + 1)
    }
}
