//! Formal evaluation policy is separate from parameter type and runtime ABI.
use super::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ParameterEvaluation {
    #[default]
    Evaluate,
    Discard,
}

impl Parser<'_> {
    pub(super) fn parameter_evaluation(
        &mut self,
        result: bool,
    ) -> Result<ParameterEvaluation, Diagnostic> {
        if self.token().kind != Kind::Directive(Directive::Discard) {
            return Ok(ParameterEvaluation::Evaluate);
        }
        if result {
            return Err(self.error("#discard applies only to procedure parameters"));
        }
        if !self.allow_qualified {
            return Err(self.error("#discard requires checked argument evaluation policy"));
        }
        self.at += 1;
        if self.token().kind == Kind::Directive(Directive::Discard) {
            return Err(self.error("duplicate #discard parameter policy"));
        }
        Ok(ParameterEvaluation::Discard)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source(text: &str) -> Result<ParsedFile, jai_source::LocatedDiagnostic> {
        let mut sources = jai_source::SourceMap::default();
        let id = sources.insert("discard.jai".into(), text.into());
        parse_file(sources.get(id).unwrap(), &mut Symbols::default())
    }

    #[test]
    fn disabled_basic_assert_preserves_each_discarded_formal() {
        let text = "assert :: (#discard arg: bool, #discard message := \"\", #discard args: ..Any, #discard loc := #caller_location) #expand {}";
        let file = source(text).unwrap();
        let FileItem::Declaration(declaration) = &file.items()[0] else {
            panic!()
        };
        let FileDeclarationKind::Procedure(procedure) = &declaration.kind else {
            panic!()
        };
        assert!(procedure.expands);
        assert_eq!(procedure.parameters.len(), 4);
        assert!(
            procedure
                .parameters
                .iter()
                .all(|parameter| parameter.evaluation == ParameterEvaluation::Discard)
        );
        assert_eq!(
            procedure.parameters[0].span.text(text),
            "#discard arg: bool"
        );
        assert!(procedure.parameters[2].variadic);
        assert!(matches!(
            &procedure.parameters[3].binding,
            ParameterBinding::Defaulted {
                expression: Expression {
                    kind: ExpressionKind::CallerLocation,
                    ..
                },
                ..
            }
        ));
    }

    #[test]
    fn callback_source_signatures_and_prototypes_retain_evaluation_policy() {
        let file = source("Callback :: #type (#discard value:int, live:int)->int; host::(#discard value:int,live:int) #foreign;").unwrap();
        let FileItem::Declaration(declaration) = &file.items()[0] else {
            panic!()
        };
        let FileDeclarationKind::TypeAlias(alias) = &declaration.kind else {
            panic!()
        };
        let TypeSyntax::Procedure(signature) = &alias.ty else {
            panic!()
        };
        assert_eq!(
            signature.parameters[0].evaluation,
            ParameterEvaluation::Discard
        );
        assert_eq!(
            signature.parameters[1].evaluation,
            ParameterEvaluation::Evaluate
        );
        assert_eq!(
            signature.results[0].evaluation,
            ParameterEvaluation::Evaluate
        );
        let FileItem::Declaration(declaration) = &file.items()[1] else {
            panic!()
        };
        let FileDeclarationKind::ProcedurePrototype(prototype) = &declaration.kind else {
            panic!()
        };
        assert_eq!(
            prototype.parameters[0].evaluation,
            ParameterEvaluation::Discard
        );
    }

    #[test]
    fn duplicate_result_and_legacy_discard_forms_are_diagnostics() {
        for text in [
            "f::(#discard #discard value:int){}",
            "Callback::#type ()->#discard int;",
            "f::(value:#discard int){}",
        ] {
            let error = source(text).unwrap_err();
            assert_eq!(error.location.span.text(text), "#discard");
        }
        assert!(
            parse("f::(#discard value:int){}")
                .unwrap_err()
                .message
                .contains("evaluation policy")
        );
    }
}
