//! Ordered record bodies preserve fields and declarations in one source sequence.
use super::*;

#[derive(Clone, Debug)]
pub enum RecordMember {
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
        if self.token().kind == Kind::Eof {
            return Err(self.error("unterminated record declaration"));
        }
        if self.token().kind == Kind::UnknownDirective {
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
        let prefix = field_prefix(&self.tokens, &mut self.at)?;
        let using = prefix.using;
        let mut names = vec![self.name()?];
        while self.take(Punct::Comma) {
            names.push(self.name()?);
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
            FieldBinding::Explicit { ty, initializer }
        };
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
        let notes = self.notes()?;
        let span = Span::new(field_start, self.tokens[self.at - 1].span.end);
        Ok(names
            .into_iter()
            .map(|name| FieldDeclaration {
                name,
                binding: binding.clone(),
                using,
                conversion: prefix.conversion,
                span,
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
