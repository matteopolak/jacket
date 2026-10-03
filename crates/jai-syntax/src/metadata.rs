//! Source-level declaration notes and record attributes remain unresolved metadata.
use super::*;

#[derive(Clone, Debug)]
pub struct NoteSyntax {
    pub name: Symbol,
    pub arguments: Vec<NoteArgument>,
    pub span: Span,
}
#[derive(Clone, Debug)]
pub struct NoteArgument {
    pub name: Option<Symbol>,
    pub value: NoteValue,
}
#[derive(Clone, Debug)]
pub enum NoteValue {
    Expression(Expression),
    Word(Symbol),
}
#[derive(Clone, Debug)]
pub enum RecordAttribute {
    Alignment(Expression),
    NoPadding,
    TypeInfoNone,
    Reflection(RecordReflectionSettingSyntax),
}
#[derive(Clone, Debug)]
pub enum FieldAttribute {
    Alignment(Expression),
    Placement(FieldPlacementSyntax),
}

pub(super) fn merge_record_attributes(
    prefix: &mut Vec<RecordAttribute>,
    suffix: Vec<RecordAttribute>,
    span: Span,
) -> Result<(), Diagnostic> {
    for attribute in &suffix {
        if prefix
            .iter()
            .any(|existing| same_record_attribute(existing, attribute))
        {
            return Err(Diagnostic::new(span, "duplicate record attribute"));
        }
    }
    prefix.extend(suffix);
    Ok(())
}

fn same_record_attribute(left: &RecordAttribute, right: &RecordAttribute) -> bool {
    match (left, right) {
        (RecordAttribute::Reflection(left), RecordAttribute::Reflection(right)) => {
            left.flag == right.flag
        }
        _ => std::mem::discriminant(left) == std::mem::discriminant(right),
    }
}

impl Parser<'_> {
    pub(super) fn notes(&mut self) -> Result<Vec<NoteSyntax>, Diagnostic> {
        let mut notes = Vec::new();
        while self.token().kind == Kind::Note {
            let span = self.token().span;
            let canonical = self.token().spelling(self.source);
            let spelling = canonical.strip_prefix('@').unwrap_or_default();
            if spelling.is_empty() {
                return Err(self.error("expected a note name after '@'"));
            }
            let name = self.symbols.intern(spelling);
            self.at += 1;
            let mut arguments = Vec::new();
            if self.take(Punct::OpenParen) && !self.take(Punct::CloseParen) {
                loop {
                    let current = self.token();
                    let next = self.tokens.get(self.at + 1).map(|token| token.kind);
                    let word = matches!(current.kind, Kind::Ident | Kind::Keyword(_));
                    if word && next == Some(Kind::Punctuation(Punct::Assign)) {
                        return Err(self.error("named note arguments are not implemented"));
                    }
                    let value = if word
                        && matches!(
                            next,
                            Some(Kind::Punctuation(Punct::Comma | Punct::CloseParen))
                        ) {
                        // Notes such as @JsonName(context) contain words, not keyword expressions.
                        let symbol = self.symbols.intern(&current.spelling(self.source));
                        self.at += 1;
                        NoteValue::Word(symbol)
                    } else {
                        NoteValue::Expression(self.expression(0)?)
                    };
                    arguments.push(NoteArgument {
                        name: None,
                        value,
                    });
                    if self.take(Punct::CloseParen) {
                        break;
                    }
                    self.need(Punct::Comma)?;
                    if self.is(Punct::CloseParen) {
                        return Err(self.error("expected a note argument after ','"));
                    }
                }
            }
            notes.push(NoteSyntax {
                name,
                arguments,
                span: Span::new(span.start, self.tokens[self.at - 1].span.end),
            });
        }
        Ok(notes)
    }

    pub(super) fn record_attributes(&mut self) -> Result<Vec<RecordAttribute>, Diagnostic> {
        let mut attributes = Vec::new();
        let mut seen = [false; 3];
        loop {
            if matches!(self.token().kind, Kind::UnknownDirective) {
                let span = self.token().span;
                if let Some(setting) = self.record_reflection_setting() {
                    let attribute = RecordAttribute::Reflection(setting);
                    if attributes
                        .iter()
                        .any(|existing| same_record_attribute(existing, &attribute))
                    {
                        return Err(Diagnostic::new(span, "duplicate record attribute"));
                    }
                    attributes.push(attribute);
                    continue;
                }
            }
            let (index, directive) = match self.token().kind {
                Kind::Directive(Directive::Align) => (0, Directive::Align),
                Kind::Directive(Directive::NoPadding) => (1, Directive::NoPadding),
                Kind::Directive(Directive::TypeInfoNone) => (2, Directive::TypeInfoNone),
                _ => break,
            };
            if seen[index] {
                return Err(self.error("duplicate record attribute"));
            }
            seen[index] = true;
            self.at += 1;
            let attribute = match directive {
                Directive::Align => RecordAttribute::Alignment(self.expression(0)?),
                Directive::NoPadding => RecordAttribute::NoPadding,
                Directive::TypeInfoNone => RecordAttribute::TypeInfoNone,
                _ => unreachable!("only record attributes reach this branch"),
            };
            attributes.push(attribute);
        }
        Ok(attributes)
    }

    pub(super) fn field_attributes(&mut self) -> Result<Vec<FieldAttribute>, Diagnostic> {
        let mut attributes = Vec::new();
        while self.token().kind == Kind::Directive(Directive::Align) {
            if !attributes.is_empty() {
                return Err(self.error("duplicate field alignment attribute"));
            }
            self.at += 1;
            attributes.push(FieldAttribute::Alignment(self.expression(0)?));
        }
        Ok(attributes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parser(source: &str) -> Parser<'_> {
        Parser {
            source,
            tokens: lex(source).unwrap(),
            at: 0,
            symbols: Symbols::default(),
            allow_qualified: true,
            record_conditional_depth: 0,
            file_conditional_depth: 0,
        }
    }

    #[test]
    fn notes_preserve_names_reserved_words_expressions_and_spans() {
        let source = "@JsonIgnore @JsonName(context) @Serialize(1) @Label(\"value\") next";
        let mut parser = parser(source);
        let notes = parser.notes().unwrap();
        assert_eq!(notes.len(), 4);
        assert_eq!(notes[0].span.text(source), "@JsonIgnore");
        assert!(notes[0].arguments.is_empty());
        assert_eq!(parser.symbols.name(notes[1].name), "JsonName");
        assert_eq!(notes[1].span.text(source), "@JsonName(context)");
        let NoteValue::Word(word) = notes[1].arguments[0].value else {
            panic!()
        };
        assert_eq!(parser.symbols.name(word), "context");
        assert!(notes[1].arguments[0].name.is_none());
        assert!(matches!(
            notes[2].arguments[0].value,
            NoteValue::Expression(Expression {
                kind: ExpressionKind::Integer(1),
                ..
            })
        ));
        assert!(
            matches!(&notes[3].arguments[0].value, NoteValue::Expression(Expression { kind: ExpressionKind::String(bytes), .. }) if bytes == b"value")
        );
        assert_eq!(parser.text(), "next");
    }

    #[test]
    fn identifier_note_words_remain_distinct_from_expressions() {
        let mut parser = parser("@JsonName(plaintext) @Tag(context, 1 + 2)");
        let notes = parser.notes().unwrap();
        let NoteValue::Word(word) = notes[0].arguments[0].value else {
            panic!()
        };
        assert_eq!(parser.symbols.name(word), "plaintext");
        assert!(matches!(
            notes[1].arguments[1].value,
            NoteValue::Expression(Expression {
                kind: ExpressionKind::Binary(BinaryOp::Add, ..),
                ..
            })
        ));
    }

    #[test]
    fn attributes_consume_bare_alignment_expressions_and_stop_at_delimiters() {
        let mut parser = parser("#type_info_none #align 4 #no_padding {");
        let attributes = parser.record_attributes().unwrap();
        assert!(matches!(
            attributes.as_slice(),
            [
                RecordAttribute::TypeInfoNone,
                RecordAttribute::Alignment(Expression {
                    kind: ExpressionKind::Integer(4),
                    ..
                }),
                RecordAttribute::NoPadding
            ]
        ));
        assert!(parser.is(Punct::OpenBrace));
        let mut parser = self::parser("#align 32 + 32;");
        let attributes = parser.record_attributes().unwrap();
        assert!(matches!(
            attributes.as_slice(),
            [RecordAttribute::Alignment(Expression {
                kind: ExpressionKind::Binary(BinaryOp::Add, ..),
                ..
            })]
        ));
        assert!(parser.is(Punct::Semicolon));
    }

    #[test]
    fn malformed_metadata_and_duplicates_have_located_diagnostics() {
        for source in [
            "@",
            "@Tag(",
            "@Tag(,)",
            "@Tag(a,)",
            "@Tag(a b)",
            "@Range(min=0)",
        ] {
            let mut parser = parser(source);
            let error = parser.notes().unwrap_err();
            assert!(error.span.end <= source.len(), "{source}");
        }
        for source in [
            "#align",
            "#align;",
            "#align #no_padding",
            "#align 4 #align 8",
            "#no_padding #no_padding",
            "#type_info_none #type_info_none",
        ] {
            let mut parser = parser(source);
            let error = parser.record_attributes().unwrap_err();
            assert!(error.span.end <= source.len(), "{source}");
        }
        let mut parser = parser("#no_padding #no_padding");
        assert_eq!(
            parser
                .record_attributes()
                .unwrap_err()
                .span
                .text(parser.source),
            "#no_padding"
        );
    }

    #[test]
    fn recent_jaison_notes_are_decoded_from_real_source_tokens() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../corpus/upstream/rluba--jaison/examples/example.jai");
        let source = match std::fs::read_to_string(&path) {
            Ok(source) => source,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                eprintln!(
                    "skipping optional source fixture absent at {}",
                    path.display()
                );
                return;
            }
            Err(error) => panic!("could not read {}: {error}", path.display()),
        };
        let mut parser = parser(&source);
        let starts: Vec<_> = parser
            .tokens
            .iter()
            .enumerate()
            .filter_map(|(at, token)| {
                (token.kind == Kind::Note
                    && matches!(token.span.text(&source), "@JsonName" | "@JsonIgnore"))
                .then_some(at)
            })
            .collect();
        assert!(!starts.is_empty());
        for at in starts {
            parser.at = at;
            let notes = parser.notes().unwrap();
            assert!(!notes.is_empty());
        }
    }

    #[test]
    fn field_alignment_is_distinct_from_record_attributes() {
        let mut parser = parser("#align 4;");
        let attributes = parser.field_attributes().unwrap();
        assert!(matches!(
            attributes.as_slice(),
            [FieldAttribute::Alignment(Expression {
                kind: ExpressionKind::Integer(4),
                ..
            })]
        ));
        assert!(parser.is(Punct::Semicolon));
        let mut parser = self::parser("#align 4 #align 8");
        let error = parser.field_attributes().unwrap_err();
        assert_eq!(error.message, "duplicate field alignment attribute");
    }

    #[test]
    fn record_prefix_and_postfix_duplicates_do_not_partially_merge() {
        let mut prefix = vec![RecordAttribute::TypeInfoNone];
        let span = Span::new(25, 39);
        let error = merge_record_attributes(
            &mut prefix,
            vec![RecordAttribute::NoPadding, RecordAttribute::TypeInfoNone],
            span,
        )
        .unwrap_err();
        assert_eq!(error.span, span);
        assert_eq!(prefix.len(), 1);
        merge_record_attributes(&mut prefix, vec![RecordAttribute::NoPadding], span).unwrap();
        assert!(matches!(
            prefix.as_slice(),
            [RecordAttribute::TypeInfoNone, RecordAttribute::NoPadding]
        ));
    }
}
