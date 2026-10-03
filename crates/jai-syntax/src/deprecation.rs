//! Declaration diagnostics retain the original optional message bytes.
use super::*;

#[derive(Clone, Debug)]
pub struct Deprecation {
    pub message: Option<Vec<u8>>,
    pub span: Span,
}

impl Parser<'_> {
    pub(super) fn deprecation_suffix(
        &mut self,
        deprecation: &mut Option<Deprecation>,
    ) -> Result<(), Diagnostic> {
        while self.token().kind == Kind::Directive(Directive::Deprecated) {
            if deprecation.is_some() {
                return Err(self.error("duplicate #deprecated procedure attribute"));
            }
            if !self.allow_qualified {
                return Err(self.error("#deprecated requires declaration reference diagnostics"));
            }
            let start = self.token().span.start;
            self.at += 1;
            let message = if self.token().kind == Kind::String {
                let token = self.token();
                self.at += 1;
                Some(literals::string(token.span.text(self.source), token.span)?)
            } else {
                None
            };
            *deprecation = Some(Deprecation {
                message,
                span: Span::new(start, self.tokens[self.at - 1].span.end),
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source(text: &str) -> Result<ParsedFile, jai_source::LocatedDiagnostic> {
        let mut sources = jai_source::SourceMap::default();
        let id = sources.insert("deprecated.jai".into(), text.into());
        parse_file(sources.get(id).unwrap(), &mut Symbols::default())
    }

    #[test]
    fn deprecated_bodies_retain_message_bytes_independently_of_other_policies() {
        let text = "array_swap :: (a:*[..]$T,b:*[..]T) #deprecated \"Use Swap() instead.\\x00\" #no_context #no_debug {} @Reason bare::() #deprecated {}";
        let file = source(text).unwrap();
        let FileItem::Declaration(declaration) = &file.items()[0] else {
            panic!()
        };
        let FileDeclarationKind::Procedure(procedure) = &declaration.kind else {
            panic!()
        };
        let deprecation = procedure.deprecation.as_ref().unwrap();
        assert_eq!(
            deprecation.message.as_deref(),
            Some(b"Use Swap() instead.\0".as_slice())
        );
        assert_eq!(
            deprecation.span.text(text),
            "#deprecated \"Use Swap() instead.\\x00\""
        );
        assert!(!procedure.debug.emits());
        assert_eq!(procedure.context, jai_types::ContextMode::None);
        assert_eq!(procedure.notes.len(), 1);
        let FileItem::Declaration(declaration) = &file.items()[1] else {
            panic!()
        };
        let FileDeclarationKind::Procedure(procedure) = &declaration.kind else {
            panic!()
        };
        assert!(procedure.deprecation.as_ref().unwrap().message.is_none());
    }

    #[test]
    fn original_foreign_prototype_suffixes_retain_declaration_metadata() {
        let file = source("NSAddressOfSymbol::(symbol:*void)->*void #foreign libc #deprecated \"use dlysym()\"; old::() #deprecated #foreign;").unwrap();
        for item in file.items() {
            let FileItem::Declaration(declaration) = item else {
                panic!()
            };
            let FileDeclarationKind::ProcedurePrototype(prototype) = &declaration.kind else {
                panic!()
            };
            assert!(prototype.deprecation.is_some());
            assert!(matches!(prototype.binding, PrototypeBinding::Foreign(_)));
        }
    }

    #[test]
    fn duplicate_and_type_abi_annotations_do_not_lose_deprecation() {
        for text in [
            "old::() #deprecated #deprecated {}",
            "old::() #deprecated #foreign #deprecated;",
            "Callback::#type () #deprecated;",
        ] {
            let error = source(text).unwrap_err();
            assert!(error.message.contains("deprecated"));
            assert_eq!(error.location.span.text(text), "#deprecated");
        }
        assert!(
            parse("old::() #deprecated {}")
                .unwrap_err()
                .message
                .contains("diagnostics")
        );
    }
}
