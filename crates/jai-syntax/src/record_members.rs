//! Ordered record bodies preserve fields and declarations in one source sequence.
use super::*;

#[derive(Clone, Debug)]
pub enum RecordMember {
    Import(ScopedImportDeclaration),
    Placement(RecordPlacementSyntax),
    Using(UsingDirective),
    AnonymousRecord(Box<RecordTypeSyntax>),
    DefaultOverride {
        target: PlaceSyntax,
        value: Expression,
        span: Span,
    },
    Assert {
        condition: Expression,
        message: Option<Expression>,
        span: Span,
    },
    CompileTimeCases {
        cases: CompileTimeCases<RecordMember>,
        span: Span,
    },
    Conditional {
        condition: Expression,
        then_members: Vec<RecordMember>,
        else_members: Vec<RecordMember>,
        span: Span,
    },
    Field(FieldDeclaration),
    Constant(ConstantDeclaration),
    TypeAlias(TypeAliasDeclaration),
    Procedure(Box<Procedure>),
    ProcedurePrototype(ProcedurePrototype),
    Record(Box<RecordDeclaration>),
    Enum(EnumDeclaration),
    Insert(InsertDirective),
}

impl Parser<'_> {
    pub(super) fn record_members(&mut self) -> Result<Vec<RecordMember>, Diagnostic> {
        self.need(Punct::OpenBrace)?;
        let mut members = Vec::new();
        while !self.take(Punct::CloseBrace) {
            members.extend(self.record_member_group()?);
        }
        Ok(members)
    }

    pub(super) fn record_member_group(&mut self) -> Result<Vec<RecordMember>, Diagnostic> {
        if self.token().kind == Kind::Keyword(Keyword::Using)
            && let Some(members) = self.record_using_group()?
        {
            return Ok(members);
        }
        if self.token().kind == Kind::Eof {
            return Err(self.error("unterminated record declaration"));
        }
        if self.import_prefix() {
            return Ok(vec![RecordMember::Import(
                self.scoped_import_declaration()?,
            )]);
        }
        if self.token().kind == Kind::Directive(Directive::Place) {
            return Ok(vec![RecordMember::Placement(self.record_placement()?)]);
        }
        if self.token().kind == Kind::UnknownDirective && self.text() != "#overlay" {
            return Err(self.error(format!("unknown directive '{}'", self.text())));
        }
        if self.token().kind == Kind::Directive(Directive::Assert) {
            return Ok(vec![self.record_assertion()?]);
        }
        if self.token().kind == Kind::Directive(Directive::If) {
            return Ok(vec![self.record_conditional()?]);
        }
        if self.token().kind == Kind::Directive(Directive::Insert) {
            let directive = self.insert_directive(0)?;
            self.insert_terminator(&directive)?;
            return Ok(vec![RecordMember::Insert(directive)]);
        }
        if matches!(
            self.token().kind,
            Kind::Keyword(Keyword::Struct | Keyword::Union)
        ) {
            return Ok(vec![self.anonymous_record()?]);
        }
        match self.nominal_prefix() {
            Some(Keyword::Struct | Keyword::Union) => {
                return Ok(vec![RecordMember::Record(Box::new(
                    self.record_declaration()?,
                ))]);
            }
            Some(Keyword::Enum | Keyword::EnumFlags) => {
                return Ok(vec![RecordMember::Enum(self.enum_declaration()?)]);
            }
            _ => {}
        }
        if self.type_alias_prefix() {
            return Ok(vec![RecordMember::TypeAlias(
                self.type_alias_declaration()?,
            )]);
        }
        if self.starts_procedure() {
            return Ok(vec![match self.procedure_declaration()? {
                FileDeclarationKind::Procedure(procedure) => {
                    RecordMember::Procedure(Box::new(procedure))
                }
                FileDeclarationKind::ProcedurePrototype(prototype) => {
                    RecordMember::ProcedurePrototype(prototype)
                }
                FileDeclarationKind::OperatorAlias(alias) => {
                    return Err(Diagnostic::new(
                        alias.span,
                        "operator aliases require a file source namespace",
                    ));
                }
                _ => unreachable!("procedure parser produces definition or prototype"),
            }]);
        }
        if self.named_prefix(Punct::Constant) {
            let span = self.token().span;
            let name = self.name()?;
            let StatementKind::Constant(constant) = self.data_declaration(name, span)?.kind else {
                unreachable!("constant prefix produces a constant declaration")
            };
            return Ok(vec![RecordMember::Constant(constant)]);
        }
        if self.named_prefix(Punct::Colon) {
            let checkpoint = self.at;
            let span = self.token().span;
            let name = self.name()?;
            self.need(Punct::Colon)?;
            let ty = self.type_syntax()?;
            if self.take(Punct::Colon) {
                let initializer = self.expression(0)?;
                self.need(Punct::Semicolon)?;
                return Ok(vec![RecordMember::Constant(ConstantDeclaration {
                    name,
                    span,
                    ty: Some(ty),
                    initializer,
                })]);
            }
            self.at = checkpoint;
        }
        if self.record_default_override_prefix() {
            return Ok(vec![self.record_default_override()?]);
        }
        Ok(self
            .record_field_group()?
            .into_iter()
            .map(RecordMember::Field)
            .collect())
    }

    pub(super) fn record_field_group(&mut self) -> Result<Vec<FieldDeclaration>, Diagnostic> {
        let field_start = self.token().span.start;
        let prefix = self.placed_field_prefix()?;
        let using = prefix.qualifiers.using;
        let first_span = self.token().span;
        let mut names = vec![(self.name()?, first_span)];
        while self.take(Punct::Comma) {
            let span = self.token().span;
            names.push((self.name()?, span));
        }
        if using && names.len() != 1 {
            return Err(self.error("grouped using record fields are not implemented"));
        }
        let mut attributes = Vec::new();
        let binding = if self.take(Punct::Infer) {
            if names.len() != 1 {
                return Err(self.error("grouped inferred record fields are not implemented"));
            }
            FieldBinding::Inferred(self.expression(0)?)
        } else {
            self.need(Punct::Colon)?;
            let ty = self.type_syntax()?;
            attributes = self.field_attributes()?;
            let initializer = if self.take(Punct::Assign) {
                Some(self.initializer()?)
            } else {
                None
            };
            FieldBinding::Explicit {
                ty,
                initializer,
            }
        };
        let suffix_attributes = self.field_attributes()?;
        if !attributes.is_empty() && !suffix_attributes.is_empty() {
            return Err(self.error("duplicate field alignment attribute"));
        }
        attributes.extend(suffix_attributes);
        if matches!(
            &binding,
            FieldBinding::Explicit {
                ty: TypeSyntax::InlineRecord(_) | TypeSyntax::InlineEnum(_),
                initializer: None
            }
        ) {
            self.take(Punct::Semicolon);
        } else {
            self.need(Punct::Semicolon)?;
        }
        if let Some(placement) = prefix.placement {
            attributes.push(FieldAttribute::Placement(placement));
        }
        let notes = self.notes()?;
        let span = Span::new(field_start, self.tokens[self.at - 1].span.end);
        Ok(names
            .into_iter()
            .map(|(name, name_span)| FieldDeclaration {
                name,
                binding: binding.clone(),
                using,
                using_selection: prefix.using_selection.clone(),
                conversion: prefix.qualifiers.conversion,
                span: Span::new(name_span.start, span.end),
                attributes: attributes.clone(),
                notes: notes.clone(),
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_source::SourceMap;

    #[test]
    fn canonical_record_members_preserve_declarations_between_fields() {
        let text = "Container :: struct { before: int; LIMIT :: 4; Count :: u32; State :: enum { READY; } Inner :: struct { value: int; } get :: (value: int) -> int { return value; } external :: () #foreign Lib; #insert generated; after: Inner; }";
        let mut sources = SourceMap::default();
        let id = sources.insert("members.jai".into(), text.into());
        let file = parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap();
        let FileItem::Declaration(FileDeclaration {
            kind: FileDeclarationKind::Record(record),
            ..
        }) = &file.items()[0]
        else {
            panic!("expected record")
        };
        assert!(matches!(
            record.members.as_slice(),
            [
                RecordMember::Field(_),
                RecordMember::Constant(_),
                RecordMember::TypeAlias(_),
                RecordMember::Enum(_),
                RecordMember::Record(_),
                RecordMember::Procedure(_),
                RecordMember::ProcedurePrototype(_),
                RecordMember::Insert(_),
                RecordMember::Field(_)
            ]
        ));
        assert_eq!(record.fields().count(), 2);
    }

    #[test]
    fn conversion_fields_preserve_as_separately_from_using() {
        let text = "Type_Info_Integer :: struct { using #as info: Type_Info; signed: bool; } Wrapper :: struct { #as value: Base; }";
        let mut sources = SourceMap::default();
        let id = sources.insert("conversion.jai".into(), text.into());
        let file = parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap();
        let FileItem::Declaration(FileDeclaration {
            kind: FileDeclarationKind::Record(record),
            ..
        }) = &file.items()[0]
        else {
            panic!("expected record")
        };
        let base = record.fields().next().unwrap();
        assert!(base.using);
        assert_eq!(base.conversion, FieldConversion::Implicit);
        let FileItem::Declaration(FileDeclaration {
            kind: FileDeclarationKind::Record(wrapper),
            ..
        }) = &file.items()[1]
        else {
            panic!("expected record")
        };
        let base = wrapper.fields().next().unwrap();
        assert!(!base.using);
        assert_eq!(base.conversion, FieldConversion::Implicit);
    }

    #[test]
    fn record_modifiers_are_separate_from_parameters_and_runtime_members() {
        let text = "Holder :: struct (T: Type) #modify { return true; } { value: T; }";
        let mut sources = SourceMap::default();
        let id = sources.insert("modify-record.jai".into(), text.into());
        let file = parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap();
        let FileItem::Declaration(FileDeclaration {
            kind: FileDeclarationKind::Record(record),
            ..
        }) = &file.items()[0]
        else {
            panic!("expected record")
        };
        assert_eq!(record.parameters.len(), 1);
        assert_eq!(record.members.len(), 1);
        assert!(matches!(
            record.modify.as_ref().unwrap().body.as_slice(),
            [Statement {
                kind: StatementKind::Return(Some(Expression {
                    kind: ExpressionKind::Bool(true),
                    ..
                })),
                ..
            }]
        ));
    }
}

#[cfg(test)]
mod placement_integration_tests {
    use super::*;
    use jai_source::SourceMap;

    fn file(text: &str) -> Result<ParsedFile, jai_source::LocatedDiagnostic> {
        let mut sources = SourceMap::default();
        let id = sources.insert("placed-record.jai".into(), text.into());
        parse_file(sources.get(id).unwrap(), &mut Symbols::default())
    }

    #[test]
    fn complete_record_ast_preserves_cursor_overlay_and_reflection_policy() {
        let text = "Storage :: struct #type_info_procedures_are_void_pointers #type_info_no_size_complaint { anchor:u64; LIMIT::8; #place anchor; view:u32=---; #overlay(anchor) using #as alias:u64=---; } after::()->int{return 42;}";
        let parsed = file(text).unwrap();
        assert_eq!(parsed.items().len(), 2);
        let FileItem::Declaration(declaration) = &parsed.items()[0] else {
            panic!()
        };
        let FileDeclarationKind::Record(record) = &declaration.kind else {
            panic!()
        };
        assert_eq!(record.members.len(), 5);
        let RecordMember::Placement(placement) = &record.members[2] else {
            panic!()
        };
        assert_eq!(placement.target.span.text(text), "anchor");
        assert_eq!(placement.span.text(text), "#place anchor;");
        let RecordMember::Field(field) = &record.members[4] else {
            panic!()
        };
        assert!(field.using);
        assert_eq!(field.conversion, FieldConversion::Implicit);
        let FieldAttribute::Placement(FieldPlacementSyntax::Overlay {
            target,
            span,
        }) = &field.attributes[0]
        else {
            panic!()
        };
        assert_eq!(target.span.text(text), "anchor");
        assert_eq!(span.text(text), "#overlay(anchor)");
        let RecordAttribute::Reflection(first) = &record.attributes[0] else {
            panic!()
        };
        assert_eq!(
            first.flag,
            jai_types::RecordReflectionFlag::ProceduresAreVoidPointers
        );
        assert_eq!(
            first.span.text(text),
            "#type_info_procedures_are_void_pointers"
        );
        let RecordAttribute::Reflection(second) = &record.attributes[1] else {
            panic!()
        };
        assert_eq!(
            second.flag,
            jai_types::RecordReflectionFlag::NoSizeComplaint
        );
        assert_eq!(second.span.text(text), "#type_info_no_size_complaint");
    }

    #[test]
    fn placement_and_overlay_reject_values_calls_and_missing_delimiters() {
        for body in [
            "#place 1;",
            "#place anchor();",
            "#place anchor view:u64;",
            "#overlay(anchor()) alias:u64;",
            "#overlay anchor alias:u64;",
            "#overlay(anchor) #overlay(anchor) alias:u64;",
        ] {
            let text = format!("Storage::struct{{anchor:u64; {body}}}");
            let error = file(&text).unwrap_err();
            assert!(error.location.span.end <= text.len(), "{text}");
        }
    }

    #[test]
    fn reflection_duplicates_are_per_flag_and_unknown_spellings_stay_errors() {
        for attributes in [
            "#type_info_no_size_complaint #type_info_no_size_complaint",
            "#type_info_procedures_are_void_pointers #type_info_procedures_are_void_pointers",
            "#type_info_no_size_complaints",
        ] {
            let text = format!("Storage::struct {attributes} {{ value:u64; }}");
            assert!(file(&text).is_err(), "{text}");
        }
        // Header prefix and suffix merge also checks the actual flag, not just
        // the common Reflection variant discriminant.
        assert!(file("Storage::struct #type_info_procedures_are_void_pointers {value:u64;} #type_info_no_size_complaint").is_ok());
        assert!(file("Storage::struct #type_info_no_size_complaint {value:u64;} #type_info_no_size_complaint").is_err());
    }
}
