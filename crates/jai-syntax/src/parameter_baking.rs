//! A formal can require a constant argument or specialize only when one exists.
use super::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ParameterBaking {
    #[default]
    None,
    Required,
    Optional,
}

impl Parser<'_> {
    pub(super) fn parameter_baking(&mut self) -> Result<ParameterBaking, Diagnostic> {
        let baking = if self.take(Punct::DoubleDollar) {
            ParameterBaking::Optional
        } else if self.take(Punct::Dollar) {
            ParameterBaking::Required
        } else {
            return Ok(ParameterBaking::None);
        };
        if !self.allow_qualified {
            return Err(self.error("baked parameters require specialization"));
        }
        if self.is(Punct::Dollar) || self.is(Punct::DoubleDollar) {
            return Err(self.error("a parameter baking marker must be '$' or '$$'"));
        }
        Ok(baking)
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
    fn optional_and_required_markers_stop_before_the_original_formal_name() {
        for (source, expected) in [
            ("x: int", ParameterBaking::None),
            ("$x: int", ParameterBaking::Required),
            ("$$x: int", ParameterBaking::Optional),
            ("$$ x: int", ParameterBaking::Optional),
        ] {
            let mut parser = parser(source);
            assert_eq!(parser.parameter_baking().unwrap(), expected);
            assert_eq!(parser.text(), "x");
        }
    }

    #[test]
    fn repeated_markers_are_located_and_scalar_execution_cannot_erase_baking() {
        let text = "$$$x: int";
        let error = parser(text).parameter_baking().unwrap_err();
        assert_eq!(error.span.text(text), "$");
        assert_eq!(error.span.start, 2);
        assert_eq!(
            error.message,
            "a parameter baking marker must be '$' or '$$'"
        );
        let mut parser = parser("$$x: int");
        parser.allow_qualified = false;
        assert_eq!(
            parser.parameter_baking().unwrap_err().message,
            "baked parameters require specialization"
        );
    }
}
