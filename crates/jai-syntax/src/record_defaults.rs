//! A record-body assignment overrides that record's construction default.
use super::*;

impl Parser<'_> {
    pub(super) fn record_default_override_prefix(&self) -> bool {
        if self.token().kind != Kind::Ident {
            return false;
        }
        matches!(
            self.tokens.get(self.at + 1).map(|token| token.kind),
            Some(Kind::Punctuation(
                Punct::Assign | Punct::Dot | Punct::OpenBracket | Punct::OpenParen
            ))
        )
    }

    pub(super) fn record_default_override(&mut self) -> Result<RecordMember, Diagnostic> {
        let start = self.token().span.start;
        let target = PlaceSyntax::try_from(self.expression(0)?)?;
        self.need(Punct::Assign)?;
        let value = self.initializer()?;
        self.need(Punct::Semicolon)?;
        Ok(RecordMember::DefaultOverride {
            target,
            value,
            span: Span::new(start, self.tokens[self.at - 1].span.end),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compiler_inherited_defaults_preserve_the_ordered_target_and_value() {
        let text = "Declaration :: struct { #as using entry: Scope_Entry; base.kind = .DECLARATION; own: int = 4; #if ENABLED { base.details.flags = xx 3; } }";
        let mut sources = jai_source::SourceMap::default();
        let id = sources.insert("record-defaults.jai".into(), text.into());
        let mut symbols = Symbols::default();
        let parsed = parse_file(sources.get(id).unwrap(), &mut symbols).unwrap();
        let FileItem::Declaration(declaration) = &parsed.items()[0] else {
            panic!()
        };
        let FileDeclarationKind::Record(record) = &declaration.kind else {
            panic!()
        };
        assert!(matches!(
            record.members.as_slice(),
            [
                RecordMember::Field(_),
                RecordMember::DefaultOverride { .. },
                RecordMember::Field(_),
                RecordMember::Conditional { .. }
            ]
        ));
        let RecordMember::DefaultOverride {
            target,
            value,
            span,
        } = &record.members[1]
        else {
            panic!()
        };
        assert_eq!(target.span.text(text), "base.kind");
        assert_eq!(value.span.text(text), ".DECLARATION");
        assert_eq!(span.text(text), "base.kind = .DECLARATION;");
        assert!(
            matches!(&target.kind, PlaceKind::Qualified(path) if symbols.name(path.root) == "base" && symbols.name(path.members[0]) == "kind")
        );
        let RecordMember::Conditional {
            then_members, ..
        } = &record.members[3]
        else {
            panic!()
        };
        assert!(
            matches!(&then_members[0], RecordMember::DefaultOverride { target:PlaceSyntax { kind:PlaceKind::Qualified(path),.. }, value:Expression { kind:ExpressionKind::InferredCast {..},.. },.. } if path.members.len()==2)
        );
    }

    #[test]
    fn record_default_calls_and_incomplete_targets_are_rejected() {
        for text in [
            "Bad :: struct { call() = 3; }",
            "Bad :: struct { base. = 3; }",
            "Bad :: struct { base.kind = ; }",
        ] {
            let mut sources = jai_source::SourceMap::default();
            let id = sources.insert("bad-record-default.jai".into(), text.into());
            assert!(parse_file(sources.get(id).unwrap(), &mut Symbols::default()).is_err());
        }
    }
}
