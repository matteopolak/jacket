//! Parse #run modifiers after the directive token has been consumed.
use super::*;

impl Parser<'_> {
    pub(super) fn run_flags(&mut self) -> Result<RunFlags, Diagnostic> {
        let mut flags = RunFlags::default();
        while self.take(Punct::Comma) {
            let token = self.token();
            if token.kind != Kind::Ident {
                return Err(self.error("expected a #run flag identifier after ','"));
            }
            let spelling = token.spelling(self.source);
            if spelling != "stallable" {
                return Err(self.error(format!("unsupported #run flag `{spelling}`")));
            }
            if flags.stallable {
                return Err(self.error("duplicate #run flag `stallable`"));
            }
            if !self.allow_qualified {
                return Err(self.error("#run,stallable requires resumable compile-time execution"));
            }
            flags.stallable = true;
            self.at += 1;
        }
        Ok(flags)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parser(source: &str, allow_qualified: bool) -> Parser<'_> {
        let tokens = lex(source).unwrap();
        let at = tokens
            .iter()
            .position(|token| token.kind == Kind::Directive(Directive::Run))
            .unwrap()
            + 1;
        Parser {
            source,
            tokens,
            at,
            symbols: Symbols::default(),
            allow_qualified,
            record_conditional_depth: 0,
            file_conditional_depth: 0,
        }
    }

    #[test]
    fn flags_preserve_the_call_body_boundary_and_default_policy() {
        let mut parser = parser("#run,stallable build();", true);
        assert_eq!(parser.run_flags().unwrap(), RunFlags { stallable: true });
        assert_eq!(parser.text(), "build");
        let mut parser = self::parser("#run build();", true);
        assert_eq!(parser.run_flags().unwrap(), RunFlags::default());
        assert_eq!(parser.text(), "build");
    }

    #[test]
    fn flags_use_canonical_identifier_spelling() {
        let mut parser = parser("#run,sta\\   llable build();", true);
        let span = parser.tokens[parser.at + 1].span;
        assert_eq!(span.text(parser.source), "sta\\   llable");
        assert_eq!(parser.run_flags().unwrap(), RunFlags { stallable: true });
        assert_eq!(parser.text(), "build");
    }

    #[test]
    fn call_anonymous_and_block_forms_preserve_the_body_token_and_source_span() {
        for (source, expected_body) in [
            ("#run original(); #run,stallable build();", "build"),
            ("value :: #run,stallable -> int { return 42; };", "->"),
            ("#run,stallable { configure(); }", "{"),
        ] {
            let mut parser = parser(source, true);
            // Select the flagged run even when an earlier unflagged run exists.
            parser.at = parser
                .tokens
                .windows(4)
                .position(|tokens| {
                    tokens[0].kind == Kind::Directive(Directive::Run)
                        && tokens[1].kind == Kind::Punctuation(Punct::Comma)
                        && tokens[2].spelling(source) == "stallable"
                        && tokens[3].span.text(source) == expected_body
                })
                .unwrap()
                + 1;
            let flag_span = parser.tokens[parser.at + 1].span;
            assert_eq!(flag_span.text(source), "stallable");
            assert!(parser.run_flags().unwrap().stallable);
            assert_eq!(parser.text(), expected_body);
            assert!(parser.token().span.start >= flag_span.end);
        }
    }

    #[test]
    fn unknown_duplicate_and_missing_flags_are_located() {
        for (source, expected_span, expected_message) in [
            (
                "#run,other build();",
                "other",
                "unsupported #run flag `other`",
            ),
            (
                "#run,stallable,stallable build();",
                "stallable",
                "duplicate #run flag `stallable`",
            ),
            (
                "#run, { }",
                "{",
                "expected a #run flag identifier after ','",
            ),
            (
                "#run,stallable, -> int { return 1; }",
                "->",
                "expected a #run flag identifier after ','",
            ),
        ] {
            let mut parser = parser(source, true);
            let error = parser.run_flags().unwrap_err();
            assert_eq!(error.span.text(source), expected_span);
            assert_eq!(error.message, expected_message);
            if expected_message.starts_with("duplicate") {
                assert_eq!(error.span.start, source.rfind("stallable").unwrap());
            }
        }
        let mut parser = parser("#run,", true);
        let error = parser.run_flags().unwrap_err();
        assert_eq!(error.span, Span::new(5, 5));
    }

    #[test]
    fn legacy_execution_rejects_stallable_with_the_flag_span() {
        let source = "#run,stallable build();";
        let mut parser = parser(source, false);
        let error = parser.run_flags().unwrap_err();
        assert_eq!(error.span.text(source), "stallable");
        assert_eq!(
            error.message,
            "#run,stallable requires resumable compile-time execution"
        );
        let mut parser = self::parser("#run build();", false);
        assert_eq!(parser.run_flags().unwrap(), RunFlags::default());
    }
}
