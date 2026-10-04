//! Common file declaration syntax has one owner and distinct source names.
use super::*;
use std::sync::Arc;

impl Parser<'_> {
    pub(super) fn file_data_declarations(&mut self) -> Result<Vec<GlobalOrConstant>, Diagnostic> {
        self.file_data_declarations_with_reset_policy(GlobalResetPolicy::Reset, None)
    }

    pub(super) fn file_data_declarations_with_reset_policy(
        &mut self,
        reset_policy: GlobalResetPolicy,
        reset_policy_span: Option<Span>,
    ) -> Result<Vec<GlobalOrConstant>, Diagnostic> {
        let mut names = vec![(self.name()?, self.tokens[self.at - 1].span)];
        while self.take(Punct::Comma) {
            let span = self.token().span;
            names.push((self.name()?, span));
        }
        let first = names[0];
        if names.len() == 1 {
            let declaration = self.data_declaration(first.0, first.1)?;
            return Ok(vec![match declaration.kind {
                StatementKind::Declare(source) => GlobalOrConstant::Global(GlobalDeclaration {
                    declaration: source,
                    span: declaration.span,
                    reset_policy,
                    reset_policy_span,
                }),
                StatementKind::Constant(source) => GlobalOrConstant::Constant(source),
                _ => unreachable!("data declaration has one typed source result"),
            }]);
        }
        let source = self.data_declaration_kind(first.0, first.1)?;
        let StatementKind::Declare(source) = source else {
            return Err(
                self.error("file constant result lists require compile-time result publication")
            );
        };
        let first_initializer = match &source {
            Declaration::Inferred {
                initializer, ..
            } => Some(initializer),
            Declaration::Explicit {
                initializer, ..
            }
            | Declaration::UnresolvedExplicit {
                initializer, ..
            } => initializer.as_ref(),
            Declaration::External {
                ..
            }
            | Declaration::GroupMember {
                ..
            } => None,
        };
        if self.is(Punct::Comma) && first_initializer.is_none() {
            return Err(self.error("declaration initializer list requires '='"));
        }
        let mut extra_initializers = Vec::new();
        while self.take(Punct::Comma) {
            extra_initializers.push(self.initializer()?);
        }
        if !extra_initializers.is_empty() && extra_initializers.len() + 1 != names.len() {
            return Err(self.error("file declaration initializer count does not match name count"));
        }
        if !extra_initializers.is_empty()
            && first_initializer
                .into_iter()
                .chain(&extra_initializers)
                .any(|value| matches!(value.kind, ExpressionKind::Uninitialized))
        {
            return Err(self.error("--- must be the entire declaration initializer"));
        }
        self.need(Punct::Semicolon)?;
        let declaration_span = Span::new(first.1.start, self.tokens[self.at - 1].span.end);
        if matches!(source, Declaration::External { .. }) {
            return Err(Diagnostic::new(
                declaration_span,
                "external data bindings require an individual symbol declaration",
            ));
        }
        let group = Arc::new(DeclarationGroup {
            names,
            source,
            extra_initializers,
        });
        Ok(group
            .names
            .iter()
            .enumerate()
            .map(|(ordinal, &(name, span))| {
                GlobalOrConstant::Global(GlobalDeclaration {
                    declaration: Declaration::GroupMember {
                        name,
                        ordinal,
                        group: Arc::clone(&group),
                    },
                    span: Span::new(span.start, declaration_span.end),
                    reset_policy,
                    reset_policy_span,
                })
            })
            .collect())
    }

    pub(super) fn declaration_initializers(&mut self) -> Result<Vec<Expression>, Diagnostic> {
        let mut values = vec![self.initializer()?];
        while self.take(Punct::Comma) {
            values.push(self.initializer()?);
        }
        if values
            .iter()
            .any(|value| matches!(value.kind, ExpressionKind::Uninitialized))
            && (values.len() != 1 || !matches!(values[0].kind, ExpressionKind::Uninitialized))
        {
            return Err(Diagnostic::new(
                values[0].span,
                "--- must be the entire declaration initializer",
            ));
        }
        Ok(values)
    }
}
pub(super) enum GlobalOrConstant {
    Global(GlobalDeclaration),
    Constant(ConstantDeclaration),
}
