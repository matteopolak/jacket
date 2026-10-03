//! Source settings retain canonical reflection flags without rewriting physical types.
use super::*;
use jai_types::RecordReflectionFlag;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RecordReflectionSettingSyntax {
    pub flag: RecordReflectionFlag,
    pub span: Span,
}

impl Parser<'_> {
    pub(super) fn record_reflection_setting(&mut self) -> Option<RecordReflectionSettingSyntax> {
        let token = self.token();
        let flag = match token.kind {
            Kind::Directive(Directive::TypeInfoNone) => RecordReflectionFlag::NoTypeInfo,
            // These exact spellings stay at the lexical boundary until the production
            // directive tags and checked record consumers activate together.
            Kind::UnknownDirective => match token.spelling(self.source).as_ref() {
                "#type_info_procedures_are_void_pointers" => {
                    RecordReflectionFlag::ProceduresAreVoidPointers
                }
                "#type_info_no_size_complaint" => RecordReflectionFlag::NoSizeComplaint,
                _ => return None,
            },
            _ => return None,
        };
        self.at += 1;
        Some(RecordReflectionSettingSyntax {
            flag,
            span: token.span,
        })
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
    fn source_settings_preserve_each_canonical_flag_and_exact_directive_span() {
        let text = "#type_info_none #type_info_procedures_are_void_pointers #type_info_no_size_complaint {";
        let mut parser = parser(text);
        let expected = [
            RecordReflectionFlag::NoTypeInfo,
            RecordReflectionFlag::ProceduresAreVoidPointers,
            RecordReflectionFlag::NoSizeComplaint,
        ];
        let mut settings = Vec::new();
        for flag in expected {
            let setting = parser.record_reflection_setting().unwrap();
            assert_eq!(setting.flag, flag);
            assert!(setting.span.text(text).starts_with("#type_info_"));
            settings.push(setting.flag);
        }
        assert!(parser.is(Punct::OpenBrace));
        assert_eq!(
            jai_types::RecordReflectionPolicy::from_flags(settings).bits(),
            7
        );
    }

    #[test]
    fn unrelated_or_misspelled_directives_are_not_consumed() {
        for text in [
            "#type_info_no_size_complaints",
            "#invented_file_directive",
            "#align 4",
            "field:int;",
        ] {
            let mut parser = parser(text);
            assert!(parser.record_reflection_setting().is_none());
            assert_eq!(parser.at, 0);
        }
    }
}
