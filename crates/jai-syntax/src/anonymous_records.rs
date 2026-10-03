//! Unnamed record members preserve physical structure without a generated name.
use super::*;

impl Parser<'_> {
    pub(super) fn anonymous_record(&mut self) -> Result<RecordMember, Diagnostic> {
        let mut record = self.record_type()?;
        record.notes = self.notes()?;
        self.take(Punct::Semicolon);
        record.span.end = self.tokens[self.at - 1].span.end;
        Ok(RecordMember::AnonymousRecord(Box::new(record)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_types::RecordKind;

    #[test]
    fn compiler_and_focus_anonymous_unions_preserve_nested_storage_and_terminators() {
        let text = "Pointer_Info :: struct { union { global_symbol:*Declaration; string_literal:*Literal; }; data_pointer:*u8; union { keyword:int; punctuation:u8; } }";
        let mut sources = jai_source::SourceMap::default();
        let id = sources.insert("anonymous-unions.jai".into(), text.into());
        let mut symbols = Symbols::default();
        let parsed = parse_file(sources.get(id).unwrap(), &mut symbols).unwrap();
        let FileItem::Declaration(declaration) = &parsed.items()[0] else {
            panic!()
        };
        let FileDeclarationKind::Record(record) = &declaration.kind else {
            panic!()
        };
        let [
            RecordMember::AnonymousRecord(first),
            RecordMember::Field(_),
            RecordMember::AnonymousRecord(second),
        ] = record.members.as_slice()
        else {
            panic!()
        };
        assert_eq!(first.kind, RecordKind::Union);
        assert_eq!(second.kind, RecordKind::Union);
        assert!(first.span.text(text).ends_with("};"));
        assert!(second.span.text(text).ends_with('}'));
        assert_eq!(first.fields().count(), 2);
        assert_eq!(
            symbols.name(first.fields().next().unwrap().name),
            "global_symbol"
        );
        assert!(symbols.find("anonymous").is_none());
    }

    #[test]
    fn nested_anonymous_struct_and_union_members_keep_definition_order() {
        let text = "Value :: struct { before:int; union { struct { x:int; y:int; }; packed:u64; } after:int; }";
        let mut sources = jai_source::SourceMap::default();
        let id = sources.insert("anonymous-struct.jai".into(), text.into());
        let parsed = parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap();
        let FileItem::Declaration(declaration) = &parsed.items()[0] else {
            panic!()
        };
        let FileDeclarationKind::Record(record) = &declaration.kind else {
            panic!()
        };
        let [
            RecordMember::Field(_),
            RecordMember::AnonymousRecord(union),
            RecordMember::Field(_),
        ] = record.members.as_slice()
        else {
            panic!()
        };
        let [
            RecordMember::AnonymousRecord(nested),
            RecordMember::Field(_),
        ] = union.members.as_slice()
        else {
            panic!()
        };
        assert_eq!(nested.kind, RecordKind::Struct);
        assert_eq!(nested.fields().count(), 2);
    }

    #[test]
    fn incomplete_anonymous_record_bodies_are_located_errors() {
        for text in [
            "Bad::struct{union{a:int;",
            "Bad::struct{union a:int;}",
            "Bad::struct{struct{a:;} }",
        ] {
            let mut sources = jai_source::SourceMap::default();
            let id = sources.insert("invalid-anonymous-record.jai".into(), text.into());
            assert!(parse_file(sources.get(id).unwrap(), &mut Symbols::default()).is_err());
        }
    }
}
