//! Context additions share their declaration parser with quoted source statements.
use super::*;

impl Parser<'_> {
    pub(super) fn context_field_declaration(
        &mut self,
    ) -> Result<ContextFieldDeclaration, Diagnostic> {
        debug_assert_eq!(self.token().kind, Kind::Directive(Directive::AddContext));
        self.at += 1;
        if matches!(
            self.token().kind,
            Kind::Keyword(Keyword::Using) | Kind::Directive(Directive::As)
        ) {
            let mut fields = self.record_field_group()?;
            if fields.len() != 1 {
                return Err(self.error("#add_context requires one field declaration"));
            }
            return Ok(ContextFieldDeclaration::Field(fields.remove(0)));
        }
        let span = self.token().span;
        let name = self.name()?;
        Ok(match self.data_declaration(name, span)?.kind {
            StatementKind::Declare(declaration) => {
                ContextFieldDeclaration::Variable(GlobalDeclaration {
                    declaration,
                    span,
                    reset_policy: GlobalResetPolicy::Reset,
                    reset_policy_span: None,
                })
            }
            StatementKind::Constant(declaration) => ContextFieldDeclaration::Constant(declaration),
            _ => unreachable!("data declaration produces declaration"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_source::SourceMap;

    #[test]
    fn preload_context_quote_preserves_conversion_and_using() {
        let text = "FIRST_ADD_CONTEXT :: #code #add_context #as using base: Context_Base;";
        let mut sources = SourceMap::default();
        let id = sources.insert("context-quote.jai".into(), text.into());
        let mut symbols = Symbols::default();
        let file = parse_file(sources.get(id).unwrap(), &mut symbols).unwrap();
        let FileItem::Declaration(FileDeclaration {
            kind: FileDeclarationKind::Constant(constant),
            ..
        }) = &file.items()[0]
        else {
            panic!("expected constant")
        };
        let ExpressionKind::Code(CodeBody::Statement(statement)) = &constant.initializer.kind
        else {
            panic!("expected statement quotation")
        };
        let StatementKind::ContextField(ContextFieldDeclaration::Field(field)) = &statement.kind
        else {
            panic!("expected context field")
        };
        assert!(field.using);
        assert_eq!(field.conversion, FieldConversion::Implicit);
        assert_eq!(symbols.name(field.name), "base");
        assert!(matches!(
            field.binding,
            FieldBinding::Explicit {
                ty: TypeSyntax::Named(_),
                initializer: None
            }
        ));
        assert_eq!(constant.initializer.span.end, text.len());
    }

    #[test]
    fn prefixed_context_fields_preserve_alignment_defaults_and_notes() {
        let text = "#add_context using base: Base #align 4 = make_base(); @context_base";
        let mut sources = SourceMap::default();
        let id = sources.insert("context-field.jai".into(), text.into());
        let file = parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap();
        let FileItem::ContextField {
            declaration: ContextFieldDeclaration::Field(field),
            ..
        } = &file.items()[0]
        else {
            panic!("expected field")
        };
        assert!(field.using);
        assert_eq!(field.conversion, FieldConversion::None);
        assert_eq!(field.attributes.len(), 1);
        assert_eq!(field.notes.len(), 1);
        assert!(matches!(
            field.binding,
            FieldBinding::Explicit {
                initializer: Some(_),
                ..
            }
        ));
    }
}
