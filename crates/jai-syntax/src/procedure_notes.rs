//! Ordered source notes belong to their declaration, including procedure suffixes.
use super::*;

impl Parser<'_> {
    pub(super) fn procedure_notes(&mut self) -> Result<Vec<NoteSyntax>, Diagnostic> {
        let notes = self.notes()?;
        self.take(Punct::Semicolon);
        Ok(notes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_source::{SourceMap, Symbols};

    #[test]
    fn notes_follow_source_bodies_and_keep_order_and_source_text() {
        let mut sources = SourceMap::default();
        let source = "print::(format:string,args:..Any){} @PrintLike @Reason(\"bytes\\x00\",context) main::(){}";
        let id = sources.insert("notes.jai".into(), source.to_owned());
        let mut symbols = Symbols::default();
        let file = parse_file(sources.get(id).unwrap(), &mut symbols).unwrap();
        let [FileItem::Declaration(first), FileItem::Declaration(second)] = file.items() else {
            panic!("two procedure declarations");
        };
        let FileDeclarationKind::Procedure(first) = &first.kind else {
            panic!("first procedure");
        };
        assert_eq!(first.notes.len(), 2);
        assert_eq!(symbols.name(first.notes[0].name), "PrintLike");
        assert_eq!(first.notes[0].span.text(source), "@PrintLike");
        assert_eq!(
            first.notes[1].span.text(source),
            "@Reason(\"bytes\\x00\",context)"
        );
        assert!(
            matches!(&first.notes[1].arguments[0].value, NoteValue::Expression(Expression {kind:ExpressionKind::String(bytes),..}) if bytes == b"bytes\0")
        );
        assert!(
            matches!(&second.kind,FileDeclarationKind::Procedure(procedure) if procedure.notes.is_empty())
        );
    }

    #[test]
    fn prototype_notes_and_body_policies_remain_independent() {
        let mut sources = SourceMap::default();
        let source = "host::() #foreign; @Host; quiet::() #no_debug {} @PrintLike; compile::() #compile_time {} @Compile;";
        let id = sources.insert("policies.jai".into(), source.to_owned());
        let mut symbols = Symbols::default();
        let file = parse_file(sources.get(id).unwrap(), &mut symbols).unwrap();
        let [
            FileItem::Declaration(host),
            FileItem::Declaration(quiet),
            FileItem::Declaration(compile),
        ] = file.items()
        else {
            panic!("three declarations");
        };
        assert!(
            matches!(&host.kind,FileDeclarationKind::ProcedurePrototype(prototype) if prototype.notes.len()==1)
        );
        assert!(
            matches!(&quiet.kind,FileDeclarationKind::Procedure(procedure) if !procedure.debug.emits() && procedure.notes.len()==1)
        );
        assert!(
            matches!(&compile.kind,FileDeclarationKind::Procedure(procedure) if procedure.execution==jai_types::ProcedureExecution::CompileTimeOnly && procedure.notes.len()==1)
        );
    }
}
