//! Variable storage attributes retain expressions for typed layout evaluation.
use super::*;

/// Controls which contents of a file global are used in emitted native data.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GlobalResetPolicy {
    #[default]
    Reset,
    Preserve,
}

#[derive(Clone, Debug)]
pub enum DeclarationAttribute {
    Alignment(Expression),
}

impl Parser<'_> {
    pub(super) fn declaration_attributes(
        &mut self,
    ) -> Result<Vec<DeclarationAttribute>, Diagnostic> {
        let mut attributes = Vec::new();
        while self.token().kind == Kind::Directive(Directive::Align) {
            if !self.allow_qualified {
                return Err(self.error("storage alignment requires declaration layout resolution"));
            }
            if !attributes.is_empty() {
                return Err(self.error("duplicate declaration alignment attribute"));
            }
            self.at += 1;
            attributes.push(DeclarationAttribute::Alignment(self.expression(0)?));
        }
        Ok(attributes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_source::SourceMap;

    #[test]
    fn runtime_support_global_and_posix_stack_alignment_retain_source_expressions() {
        let text = "first_thread_temporary_storage_data: [TEMPORARY_STORAGE_SIZE] u8 #align 64; main :: () { temporary_storage_data: [1024] u8 #align 32 + 32 = ---; value := 1; }";
        let mut sources = SourceMap::default();
        let id = sources.insert("storage-layout.jai".into(), text.into());
        let parsed = parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap();
        let FileItem::Declaration(FileDeclaration {
            kind: FileDeclarationKind::Global(global),
            ..
        }) = &parsed.items()[0]
        else {
            panic!("expected global")
        };
        let [DeclarationAttribute::Alignment(alignment)] = global.declaration.attributes() else {
            panic!("expected alignment")
        };
        assert_eq!(alignment.span.text(text), "64");
        let FileItem::Declaration(FileDeclaration {
            kind: FileDeclarationKind::Procedure(procedure),
            ..
        }) = &parsed.items()[1]
        else {
            panic!("expected procedure")
        };
        let StatementKind::Declare(local) = &procedure.body[0].kind else {
            panic!("expected local")
        };
        let [DeclarationAttribute::Alignment(alignment)] = local.attributes() else {
            panic!("expected local alignment")
        };
        assert_eq!(alignment.span.text(text), "32 + 32");
        let StatementKind::Declare(inferred) = &procedure.body[1].kind else {
            panic!("expected inferred")
        };
        assert!(inferred.attributes().is_empty());
    }

    #[test]
    fn duplicate_and_legacy_alignment_do_not_disappear() {
        let text = "value: u32 #align 16 #align 32;";
        let mut sources = SourceMap::default();
        let id = sources.insert("duplicate-layout.jai".into(), text.into());
        let error = parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap_err();
        assert_eq!(error.message, "duplicate declaration alignment attribute");
        assert_eq!(error.location.span.text(text), "#align");
        assert_eq!(
            parse("main :: () { value: u32 #align 16; }")
                .unwrap_err()
                .message,
            "storage alignment requires declaration layout resolution"
        );
    }

    #[test]
    fn no_reset_is_a_typed_file_global_policy_for_single_and_grouped_declarations() {
        let text = "Point :: struct { x: s64; } #no_reset point: Point; #no_reset first, second: u8 = 1, 2;";
        let mut sources = SourceMap::default();
        let id = sources.insert("no-reset-globals.jai".into(), text.into());
        let parsed = parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap();
        let FileItem::Declaration(FileDeclaration {
            kind: FileDeclarationKind::Global(record),
            ..
        }) = &parsed.items()[1]
        else {
            panic!("expected record-typed global")
        };
        assert_eq!(record.reset_policy, GlobalResetPolicy::Preserve);
        assert_eq!(record.reset_policy_span.unwrap().text(text), "#no_reset");
        let FileItem::Declaration(FileDeclaration {
            kind: FileDeclarationKind::Global(first),
            ..
        }) = &parsed.items()[2]
        else {
            panic!("expected first grouped global")
        };
        let FileItem::Declaration(FileDeclaration {
            kind: FileDeclarationKind::Global(second),
            ..
        }) = &parsed.items()[3]
        else {
            panic!("expected second grouped global")
        };
        assert_eq!(first.reset_policy, GlobalResetPolicy::Preserve);
        assert_eq!(second.reset_policy, GlobalResetPolicy::Preserve);
        assert_eq!(first.reset_policy_span, second.reset_policy_span);
    }

    #[test]
    fn no_reset_rejects_non_global_declarations_at_the_attribute() {
        let error = parse("#no_reset initialize :: () {}").unwrap_err();
        assert_eq!(
            error.message,
            "#no_reset requires a file global declaration"
        );
        assert_eq!(
            error.span.text("#no_reset initialize :: () {}"),
            "#no_reset"
        );
    }
}
