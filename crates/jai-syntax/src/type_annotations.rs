//! Type-query annotations retain their operand for read-only type resolution.
use super::*;

impl Parser<'_> {
    pub(super) fn type_of_annotation(&mut self) -> Result<TypeSyntax, Diagnostic> {
        if !self.keyword(Keyword::TypeOf) {
            return Err(self.error("expected type_of annotation"));
        }
        self.need(Punct::OpenParen)?;
        let operand = self.expression(0)?;
        self.need(Punct::CloseParen)?;
        Ok(TypeSyntax::TypeOf(Box::new(operand)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_async_parameter_annotation_retains_the_declared_field_query() {
        let text = "error :: (code: type_of(Error.code), platform_code: $T = 0) -> Error { return .{code=code, platform_code=platform_code}; }";
        let mut sources = jai_source::SourceMap::default();
        let id = sources.insert("file-async-annotation.jai".into(), text.into());
        let mut symbols = Symbols::default();
        let parsed = parse_file(sources.get(id).unwrap(), &mut symbols).unwrap();
        let FileItem::Declaration(declaration) = &parsed.items()[0] else {
            panic!()
        };
        let FileDeclarationKind::Procedure(procedure) = &declaration.kind else {
            panic!()
        };
        let ParameterBinding::RequiredType(TypeSyntax::TypeOf(operand)) =
            &procedure.parameters[0].binding
        else {
            panic!()
        };
        assert_eq!(operand.span.text(text), "Error.code");
        assert!(
            matches!(&operand.kind, ExpressionKind::QualifiedName(path) if symbols.name(path.root)=="Error" && symbols.name(path.members[0])=="code")
        );
        assert_eq!(
            procedure.parameters[0].span.text(text),
            "code: type_of(Error.code)"
        );
    }

    #[test]
    fn query_annotations_nest_under_pointer_and_array_types() {
        let text =
            "Container :: struct { values: []type_of(source.value); pointer: *type_of(source); }";
        let mut sources = jai_source::SourceMap::default();
        let id = sources.insert("nested-type-query.jai".into(), text.into());
        let parsed = parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap();
        let FileItem::Declaration(declaration) = &parsed.items()[0] else {
            panic!()
        };
        let FileDeclarationKind::Record(record) = &declaration.kind else {
            panic!()
        };
        let fields: Vec<_> = record.fields().collect();
        assert!(
            matches!(&fields[0].binding, FieldBinding::Explicit {ty:TypeSyntax::Slice(element),..} if matches!(element.as_ref(),TypeSyntax::TypeOf(_)))
        );
        assert!(
            matches!(&fields[1].binding, FieldBinding::Explicit {ty:TypeSyntax::Pointer(element),..} if matches!(element.as_ref(),TypeSyntax::TypeOf(_)))
        );
    }

    #[test]
    fn incomplete_type_queries_report_the_original_token() {
        let text = "Error :: struct { value: type_of(); }";
        let mut sources = jai_source::SourceMap::default();
        let id = sources.insert("invalid-query.jai".into(), text.into());
        let error = parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap_err();
        assert_eq!(error.location.span.text(text), ")");
    }
}
