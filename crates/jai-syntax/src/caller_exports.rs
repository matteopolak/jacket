//! Caller exports retain declarations, cleanup bodies and caller returns structurally.
use super::*;

impl Parser<'_> {
    pub(super) fn caller_export_statement(&mut self) -> Result<StatementKind, Diagnostic> {
        self.need(Punct::Backtick)?;
        let statement = self.statement()?;
        if !matches!(
            statement.kind,
            StatementKind::Declare(_)
                | StatementKind::Constant(_)
                | StatementKind::Defer(_)
                | StatementKind::Return(_)
                | StatementKind::ReturnValues(_)
        ) {
            return Err(Diagnostic::new(
                statement.span,
                "a caller export requires a declaration, defer, or return",
            ));
        }
        Ok(StatementKind::CallerExport(Box::new(statement)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parser(text: &str) -> Parser<'_> {
        Parser {
            source: text,
            tokens: lex(text).unwrap(),
            at: 0,
            symbols: Symbols::default(),
            allow_qualified: true,
            record_conditional_depth: 0,
            file_conditional_depth: 0,
        }
    }

    #[test]
    fn allocator_restore_and_profile_exit_exports_retain_deferred_bodies() {
        for text in [
            "`defer context.allocator = old_allocator;",
            "`defer { note_exit(42); restore(saved); }",
        ] {
            let mut parser = parser(text);
            let StatementKind::CallerExport(export) = parser.caller_export_statement().unwrap()
            else {
                panic!()
            };
            assert_eq!(export.span.text(text), &text[1..]);
            let StatementKind::Defer(body) = &export.kind else {
                panic!()
            };
            assert!(!body.is_empty());
            assert!(
                body.iter()
                    .all(|statement| statement.span.start > export.span.start)
            );
            assert_eq!(parser.token().kind, Kind::Eof);
        }
    }

    #[test]
    fn caller_returns_keep_void_single_and_multiple_result_syntax() {
        for text in [
            "`return;",
            "`return 42;",
            "`return 1, true;",
            "`return answer = 42;",
        ] {
            let mut parser = parser(text);
            let StatementKind::CallerExport(statement) = parser.caller_export_statement().unwrap()
            else {
                panic!();
            };
            assert!(matches!(
                statement.kind,
                StatementKind::Return(_) | StatementKind::ReturnValues(_)
            ));
            assert_eq!(statement.span.text(text), &text[1..]);
            assert_eq!(parser.token().kind, Kind::Eof);
        }
    }

    #[test]
    fn caller_exports_keep_declarations_and_reject_other_statement_categories() {
        assert!(matches!(
            parser("`value := 42;").caller_export_statement().unwrap(),
            StatementKind::CallerExport(statement)
                if matches!(statement.kind, StatementKind::Declare(_))
        ));
        for text in [
            "`consume();",
            "`break;",
            "`continue;",
            "`context.allocator = previous;",
        ] {
            let error = parser(text).caller_export_statement().unwrap_err();
            assert_eq!(error.span.text(text), &text[1..]);
            assert_eq!(
                error.message,
                "a caller export requires a declaration, defer, or return"
            );
        }
    }
}
