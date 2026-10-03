//! Semicolon context pushes retain their actual enclosing-block lifetime.
use super::*;

impl Parser<'_> {
    /// The caller has consumed `push_context`; keep the suffix separate so
    /// declaration discovery sees the original containing lexical scope.
    pub(super) fn deferred_context(&mut self) -> Result<StatementKind, Diagnostic> {
        self.need(Punct::Comma)?;
        let modifier = self.token().span;
        let name = self.name()?;
        if self.symbols.name(name) != "defer_pop" {
            return Err(Diagnostic::new(modifier, "unknown push_context modifier"));
        }
        let value = if self.is(Punct::Semicolon) {
            None
        } else {
            Some(self.expression(0)?)
        };
        self.need(Punct::Semicolon)?;
        Ok(StatementKind::PushContextDeferred { value })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_source::SourceMap;

    #[test]
    fn deferred_push_keeps_following_statements_in_the_original_lexical_block() {
        let text =
            "main::(){push_context,defer_pop; x:=40; push_context, defer_pop context; x+=2;}";
        let mut sources = SourceMap::default();
        let id = sources.insert("deferred-context.jai".into(), text.into());
        let file = parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap();
        let FileItem::Declaration(FileDeclaration {
            kind: FileDeclarationKind::Procedure(procedure),
            ..
        }) = &file.items()[0]
        else {
            panic!("expected procedure")
        };
        assert_eq!(procedure.body.len(), 4);
        assert!(matches!(
            procedure.body[0].kind,
            StatementKind::PushContextDeferred { value: None }
        ));
        assert!(matches!(procedure.body[1].kind, StatementKind::Declare(_)));
        assert!(matches!(
            procedure.body[2].kind,
            StatementKind::PushContextDeferred { value: Some(_) }
        ));
        assert_eq!(procedure.body[0].span.text(text), "push_context,defer_pop;");
    }
}
