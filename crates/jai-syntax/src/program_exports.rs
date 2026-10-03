//! Program exports annotate the following definition independently of module visibility.
use super::*;

#[derive(Clone, Debug)]
pub struct ProgramExport {
    /// An absent spelling exports the source declaration's own name.
    pub symbol: Option<String>,
    pub span: Span,
}

impl Parser<'_> {
    pub(super) fn program_export(&mut self) -> Result<ProgramExport, Diagnostic> {
        let start = self.token().span.start;
        self.at += 1;
        let symbol = if self.token().kind == Kind::String {
            Some(self.module_string()?)
        } else {
            None
        };
        let span = Span::new(start, self.tokens[self.at - 1].span.end);
        self.take(Punct::Semicolon);
        if self.token().kind == Kind::Directive(Directive::ProgramExport) {
            return Err(self.error("duplicate #program_export annotation"));
        }
        if self.token().kind != Kind::Ident {
            return Err(Diagnostic::new(
                span,
                "#program_export must precede a procedure or global definition",
            ));
        }
        Ok(ProgramExport { symbol, span })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_source::SourceMap;
    fn parse(source: &str) -> Result<ParsedFile, String> {
        let mut sources = SourceMap::default();
        let source = sources.insert("own-export.jai".into(), source.into());
        parse_file(sources.get(source).unwrap(), &mut Symbols::default())
            .map_err(|error| error.to_string())
    }
    #[test]
    fn exports_following_exact_definition_without_changing_visibility() {
        let source = "#scope_module #program_export \"native_answer\" answer :: () -> s32 #c_call { return 42; } after :: () {}";
        let file = parse(source).unwrap();
        let FileItem::Declaration(declaration) = &file.items()[1] else {
            panic!()
        };
        assert_eq!(declaration.visibility, Visibility::Module);
        assert_eq!(
            declaration
                .program_export
                .as_ref()
                .unwrap()
                .symbol
                .as_deref(),
            Some("native_answer")
        );
        let FileItem::Declaration(after) = &file.items()[2] else {
            panic!()
        };
        assert!(after.program_export.is_none());
    }
    #[test]
    fn global_and_unnamed_exports_preserve_annotation() {
        let file = parse("#program_export count : s32 = 42;").unwrap();
        let FileItem::Declaration(declaration) = &file.items()[0] else {
            panic!()
        };
        assert!(
            declaration
                .program_export
                .as_ref()
                .unwrap()
                .symbol
                .is_none()
        );
    }
    #[test]
    fn invalid_export_attachment_is_not_silently_discarded() {
        for source in [
            "#program_export",
            "#program_export #program_export f :: () {}",
            "#program_export Lib :: #import \"Lib\";",
            "#program_export T :: struct {};",
            "#program_export f :: () #foreign;",
        ] {
            assert!(parse(source).is_err(), "{source}");
        }
    }
    #[test]
    fn runtime_entry_alias_and_exports_remain_typed_syntax() {
        let file = parse("#program_export __jai_runtime_init :: () {} #program_export __jai_runtime_fini :: () {} #program_export \"main\" trampoline :: (argc:s32,argv:**u8)->s32 #c_call { __program_main :: () #entry_point; __program_main(); return 0; }").unwrap();
        assert_eq!(file.items().len(), 3);
        let FileItem::Declaration(trampoline) = &file.items()[2] else {
            panic!()
        };
        let FileDeclarationKind::Procedure(procedure) = &trampoline.kind else {
            panic!()
        };
        let StatementKind::ProcedurePrototype(alias) = &procedure.body[0].kind else {
            panic!()
        };
        assert!(matches!(alias.binding, PrototypeBinding::EntryPoint));
        assert_eq!(
            trampoline
                .program_export
                .as_ref()
                .unwrap()
                .symbol
                .as_deref(),
            Some("main")
        );
    }
}
