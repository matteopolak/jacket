//! Original named results own procedure locals, without changing runtime formals.
use crate::{Block, Diagnostic, Local, Resolver, Statement, ValueExpr, syntax};

impl Resolver<'_> {
    pub(crate) fn named_result_body(
        &mut self,
        source: &[syntax::ProcedureResult],
        statements: &[syntax::Statement],
    ) -> Result<Block, Diagnostic> {
        // Header checking canonicalizes a sole void annotation to no runtime
        // results. Resolve that original annotation in this defining environment
        // before comparing result ordinals, including aliases and substitutions.
        let source = if self.results.is_empty()
            && let [result] = source
            && let syntax::ResultBinding::Typed {
                ty, ..
            } = &result.binding
            && self.lexical_annotation(ty, result.span)? == self.types.void()
        {
            &[][..]
        } else {
            source
        };
        if source.len() != self.results.len()
            || !source
                .iter()
                .zip(self.results)
                .all(|(source, result)| source.name == result.name)
        {
            return Err(
                self.error("named result storage does not match the actual source signature")
            );
        }
        let mut slots = vec![None; self.results.len()];
        let mut initializers = Vec::new();
        let mut declarations = Vec::new();
        for (ordinal, source) in source.iter().enumerate() {
            let Some(name) = source.name else {
                continue;
            };
            let result = &self.results[ordinal];
            let ty = result.ty;
            let initial = match &result.default {
                Some(value) => Some(value.clone().into_expression()),
                None if self
                    .types
                    .record_definition(ty)
                    .is_ok_and(|record| record.kind == jai_types::RecordKind::Union) =>
                {
                    None
                }
                None => Some(self.default_value(ty, source.span)?.into_expression()),
            };
            let previous_span = std::mem::replace(&mut self.span, source.span);
            let local = self.declare_typed(name, ty);
            self.span = previous_span;
            let local = local?;
            match &source.binding {
                syntax::ResultBinding::Typed {
                    ty, ..
                } => {
                    self.bind_callback_annotation(local.place(), ty, source.span)?;
                }
                syntax::ResultBinding::InferredDefault(expression) => {
                    let contract = match &initial {
                        Some(value) => {
                            self.callback_expression_contract(expression, value, source.span)?
                        }
                        None => None,
                    };
                    self.bind_value_contract(local.place(), contract, source.span)?;
                }
            }
            slots[ordinal] = Some(local);
            declarations.push((local, name, source.span, initializers.len()));
            initializers.push(match initial {
                Some(value) => Statement::Store(local.place(), value),
                // Like an ordinary union local, the body must establish an active
                // alternative before reading an explicitly uninitialized result.
                None => Statement::Block(Block {
                    statements: Vec::new(),
                    flow: crate::Flow::FallsThrough,
                }),
            });
        }
        self.local_scopes.set_named_results(self.procedure, slots);
        let mut body = self.block(statements, false)?;
        self.debug.prepend_block(initializers.len());
        for (local, name, span, index) in declarations {
            if let Some(location) = self.debug_location(span)? {
                self.debug.named_result_local(
                    local.id(),
                    self.symbols.name(name).to_owned(),
                    location,
                    index,
                );
            }
        }
        initializers.append(&mut body.statements);
        body.statements = initializers;
        Ok(body)
    }

    fn checked_named_result_local(&self, ordinal: usize) -> Result<Option<Local>, Diagnostic> {
        let Some(local) = self.local_scopes.named_result(self.procedure, ordinal) else {
            return Ok(None);
        };
        let result = self
            .results
            .get(ordinal)
            .ok_or_else(|| self.error("named result ordinal is unavailable"))?;
        if result.name.is_none()
            || local.id().procedure() != self.procedure
            || local.ty() != result.ty
        {
            return Err(
                self.error("named result storage belongs to a different procedure or result type")
            );
        }
        Ok(Some(local))
    }

    pub(crate) fn capture_named_return(
        &mut self,
        ordinal: usize,
        statements: &mut Vec<Statement>,
    ) -> Result<Option<ValueExpr>, Diagnostic> {
        let Some(local) = self.checked_named_result_local(ordinal)? else {
            return Ok(None);
        };
        let value = ValueExpr::Load(local.place());
        self.reject_returned_pack_alias(&value, self.span)?;
        crate::sequences::reject_returned_temporary(&value, self.types, self.span)?;
        let snapshot = self.allocate_typed(local.ty())?;
        statements.push(Statement::Store(snapshot.place(), value));
        Ok(Some(ValueExpr::Load(snapshot.place())))
    }
}
