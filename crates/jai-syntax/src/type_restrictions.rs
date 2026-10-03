//! Restrictions preserve unresolved type syntax rather than canonical identities.
use super::*;

#[derive(Clone, Debug)]
pub enum TypeRestrictionSyntax {
    Nominal(Box<TypeSyntax>),
    Interface(Box<TypeSyntax>),
}

impl Parser<'_> {
    /// Called after an introducing variable's name, leaving ordinary `$T` untouched.
    pub(super) fn type_restriction(&mut self) -> Result<Option<TypeRestrictionSyntax>, Diagnostic> {
        if !self.take(Punct::Div) {
            return Ok(None);
        }
        let interface = self.keyword(Keyword::Interface);
        if matches!(
            self.token().kind,
            Kind::Directive(Directive::Run | Directive::Insert)
        ) {
            return Err(self.error("compile-time execution is not permitted in a type restriction"));
        }
        let ty = Box::new(self.parameter_type_syntax()?);
        Ok(Some(if interface {
            TypeRestrictionSyntax::Interface(ty)
        } else {
            TypeRestrictionSyntax::Nominal(ty)
        }))
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
    fn nominal_and_interface_restrictions_preserve_paths_and_the_formal_boundary() {
        for (text, interface) in [
            ("/Types.Blentity, next: int", false),
            ("/interface Matchable, next: int", true),
        ] {
            let mut parser = parser(text);
            let restriction = parser.type_restriction().unwrap().unwrap();
            let ty = match restriction {
                TypeRestrictionSyntax::Nominal(ty) => {
                    assert!(!interface);
                    ty
                }
                TypeRestrictionSyntax::Interface(ty) => {
                    assert!(interface);
                    ty
                }
            };
            assert!(matches!(*ty, TypeSyntax::Named(_)));
            assert_eq!(parser.token().kind, Kind::Punctuation(Punct::Comma));
        }
        let mut parser = parser(", next: int");
        assert!(parser.type_restriction().unwrap().is_none());
        assert_eq!(parser.at, 0);
    }

    #[test]
    fn incomplete_and_executing_restrictions_have_located_diagnostics() {
        for text in [
            "/#run build()",
            "/interface #run build()",
            "/#insert generated",
        ] {
            let error = parser(text).type_restriction().unwrap_err();
            assert!(error.span.text(text).starts_with('#'));
            assert_eq!(
                error.message,
                "compile-time execution is not permitted in a type restriction"
            );
        }
        let text = "/interface , next: int";
        let error = parser(text).type_restriction().unwrap_err();
        assert_eq!(error.span.text(text), ",");
        assert_eq!(error.message, "expected type expression");
    }
}
