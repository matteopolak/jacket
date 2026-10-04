//! Unresolved type expressions and ordered nominal declarations.
use super::*;
#[path = "anonymous_pointer_aliases.rs"]
mod anonymous_pointer_aliases;
#[path = "callback_aliases.rs"]
mod callback_aliases;
use jai_types::{FloatType, RecordKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuiltinType {
    Scalar(ScalarType),
    Float(FloatType),
    String,
    Void,
    Type,
    Context,
    Any,
}

impl BuiltinType {
    pub fn from_spelling(text: &str) -> Option<Self> {
        Some(match text {
            "int" | "s64" => Self::Scalar(ScalarType::Int(IntegerType::S64)),
            "s8" => Self::Scalar(ScalarType::Int(IntegerType::S8)),
            "s16" => Self::Scalar(ScalarType::Int(IntegerType::S16)),
            "s32" => Self::Scalar(ScalarType::Int(IntegerType::S32)),
            "u8" => Self::Scalar(ScalarType::Int(IntegerType::U8)),
            "u16" => Self::Scalar(ScalarType::Int(IntegerType::U16)),
            "u32" => Self::Scalar(ScalarType::Int(IntegerType::U32)),
            "u64" => Self::Scalar(ScalarType::Int(IntegerType::U64)),
            "bool" => Self::Scalar(ScalarType::Bool),
            "float" | "float32" => Self::Float(FloatType::F32),
            "float64" => Self::Float(FloatType::F64),
            "string" => Self::String,
            "void" => Self::Void,
            "Type" => Self::Type,
            "Any" => Self::Any,
            _ => return None,
        })
    }
}

#[derive(Clone, Debug)]
pub enum TypeSyntax {
    This,
    TypeOf(Box<Expression>),
    Builtin(BuiltinType),
    Named(NamePath),
    Variable(Symbol),
    Restricted {
        variable: Symbol,
        restriction: TypeRestrictionSyntax,
        span: Span,
    },
    InlineRecord(Box<RecordTypeSyntax>),
    InlineEnum(Box<EnumTypeSyntax>),
    Variant {
        kind: TypeVariantKind,
        base: Box<TypeSyntax>,
    },
    Pointer(Box<TypeSyntax>),
    FixedArray {
        count: Box<Expression>,
        element: Box<TypeSyntax>,
    },
    Slice(Box<TypeSyntax>),
    DynamicArray(Box<TypeSyntax>),
    Procedure(ProcedureTypeSyntax),
    Application(TypeApplicationSyntax),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TypeVariantKind {
    Distinct,
    IsA,
}
#[derive(Clone, Debug)]
pub struct TypeAliasDeclaration {
    pub name: Symbol,
    pub ty: TypeSyntax,
    pub span: Span,
}
impl TypeSyntax {
    /// The executable scalar AST is an explicit bridge, not a name lookup.
    pub fn as_scalar(&self) -> Option<ScalarType> {
        match self {
            Self::Builtin(BuiltinType::Scalar(ty)) => Some(*ty),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub enum FieldBinding {
    Explicit {
        ty: TypeSyntax,
        initializer: Option<Expression>,
    },
    Inferred(Expression),
}
#[derive(Clone, Debug)]
pub struct FieldDeclaration {
    pub name: Symbol,
    pub binding: FieldBinding,
    pub using: bool,
    pub using_selection: UsingSelection,
    pub conversion: FieldConversion,
    pub span: Span,
    pub attributes: Vec<FieldAttribute>,
    pub notes: Vec<NoteSyntax>,
}
#[derive(Clone, Debug)]
pub struct RecordDeclaration {
    pub name: Symbol,
    pub kind: RecordKind,
    pub members: Vec<RecordMember>,
    pub parameters: Vec<RecordParameter>,
    pub span: Span,
    pub attributes: Vec<RecordAttribute>,
    pub notes: Vec<NoteSyntax>,
    pub modify: Option<ModifyDirective>,
}
#[derive(Clone, Debug)]
pub struct RecordTypeSyntax {
    pub kind: jai_types::RecordKind,
    pub members: Vec<RecordMember>,
    pub parameters: Vec<RecordParameter>,
    pub attributes: Vec<RecordAttribute>,
    pub notes: Vec<NoteSyntax>,
    pub span: Span,
    pub modify: Option<ModifyDirective>,
}
impl RecordDeclaration {
    pub fn fields(&self) -> impl Iterator<Item = &FieldDeclaration> {
        self.members.iter().filter_map(|member| match member {
            RecordMember::Field(field) => Some(field),
            _ => None,
        })
    }
}
impl RecordTypeSyntax {
    pub fn fields(&self) -> impl Iterator<Item = &FieldDeclaration> {
        self.members.iter().filter_map(|member| match member {
            RecordMember::Field(field) => Some(field),
            _ => None,
        })
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnumKind {
    Values,
    Flags,
}
#[derive(Clone, Debug)]
pub struct EnumMember {
    pub name: Symbol,
    pub initializer: Option<Expression>,
    pub span: Span,
    pub notes: Vec<NoteSyntax>,
}
#[derive(Clone, Debug)]
pub struct EnumDeclaration {
    pub name: Symbol,
    pub representation: Option<TypeSyntax>,
    pub kind: EnumKind,
    pub specified: bool,
    pub members: Vec<EnumBodyItem>,
    pub span: Span,
    pub notes: Vec<NoteSyntax>,
}
#[derive(Clone, Debug)]
pub struct EnumTypeSyntax {
    pub representation: Option<TypeSyntax>,
    pub kind: EnumKind,
    pub specified: bool,
    pub members: Vec<EnumBodyItem>,
    pub span: Span,
    pub notes: Vec<NoteSyntax>,
}

#[derive(Clone, Debug)]
pub struct StructLiteralField {
    pub target: PlaceSyntax,
    pub value: Expression,
    pub span: Span,
}
#[derive(Clone, Debug)]
pub struct StructLiteral {
    pub ty: Option<TypeSyntax>,
    pub fields: Vec<StructLiteralField>,
}
#[derive(Clone, Debug)]
pub struct PositionalStructLiteral {
    pub ty: Option<TypeSyntax>,
    pub values: Vec<Expression>,
}

impl Parser<'_> {
    pub(super) fn type_alias_prefix(&self) -> bool {
        if !self.named_prefix(Punct::Constant) {
            return false;
        }
        let token = self.tokens[self.at + 2];
        match token.kind {
            Kind::Punctuation(Punct::Mul) => {
                let mut leaf = self.at + 2;
                while self
                    .tokens
                    .get(leaf)
                    .is_some_and(|token| token.kind == Kind::Punctuation(Punct::Mul))
                {
                    leaf += 1;
                }
                match self.tokens.get(leaf).map(|token| token.kind) {
                    Some(Kind::Keyword(
                        Keyword::Struct | Keyword::Union | Keyword::Enum | Keyword::EnumFlags,
                    )) => true,
                    Some(Kind::Ident) => {
                        BuiltinType::from_spelling(&self.tokens[leaf].spelling(self.source))
                            .is_some()
                            && self.tokens.get(leaf + 1).is_some_and(|token| {
                                token.kind == Kind::Punctuation(Punct::Semicolon)
                            })
                    }
                    _ => false,
                }
            }
            Kind::Punctuation(Punct::OpenParen) => self.bodyless_procedure_type_prefix(),
            Kind::Directive(Directive::Type | Directive::Context) => true,
            Kind::Punctuation(Punct::OpenBracket) => !self.tokens[self.at + 2..]
                .iter()
                .take_while(|token| {
                    !matches!(token.kind, Kind::Punctuation(Punct::Semicolon) | Kind::Eof)
                })
                .any(|token| {
                    matches!(
                        token.kind,
                        Kind::Punctuation(Punct::ArrayLiteral | Punct::StructLiteral)
                    )
                }),
            Kind::Ident => {
                BuiltinType::from_spelling(&token.spelling(self.source)).is_some()
                    && self
                        .tokens
                        .get(self.at + 3)
                        .is_some_and(|token| token.kind == Kind::Punctuation(Punct::Semicolon))
            }
            _ => false,
        }
    }
    pub(super) fn type_alias_declaration(&mut self) -> Result<TypeAliasDeclaration, Diagnostic> {
        let start = self.token().span.start;
        let name = self.name()?;
        self.need(Punct::Constant)?;
        let ty = self.type_syntax()?;
        self.need(Punct::Semicolon)?;
        Ok(TypeAliasDeclaration {
            name,
            ty,
            span: Span::new(start, self.tokens[self.at - 1].span.end),
        })
    }
    pub(super) fn type_syntax(&mut self) -> Result<TypeSyntax, Diagnostic> {
        self.type_syntax_with_results(true)
    }
    pub(super) fn parameter_type_syntax(&mut self) -> Result<TypeSyntax, Diagnostic> {
        self.type_syntax_with_results(false)
    }
    fn type_syntax_with_results(&mut self, result_list: bool) -> Result<TypeSyntax, Diagnostic> {
        let start = self.token().span.start;
        if self.token().kind == Kind::Directive(Directive::This) {
            self.at += 1;
            return Ok(TypeSyntax::This);
        }
        if self.token().kind == Kind::Keyword(Keyword::TypeOf) {
            return self.type_of_annotation();
        }
        if self.token().kind == Kind::Directive(Directive::Context) {
            self.at += 1;
            return Ok(TypeSyntax::Builtin(BuiltinType::Context));
        }
        if matches!(
            self.token().kind,
            Kind::Keyword(Keyword::Struct | Keyword::Union)
        ) {
            return Ok(TypeSyntax::InlineRecord(Box::new(self.record_type()?)));
        }
        if matches!(
            self.token().kind,
            Kind::Keyword(Keyword::Enum | Keyword::EnumFlags)
        ) {
            return Ok(TypeSyntax::InlineEnum(Box::new(self.enum_type()?)));
        }
        if self.take(Punct::Dollar) {
            let variable = self.name()?;
            return Ok(match self.type_restriction()? {
                Some(restriction) => TypeSyntax::Restricted {
                    variable,
                    restriction,
                    span: Span::new(start, self.tokens[self.at - 1].span.end),
                },
                None => TypeSyntax::Variable(variable),
            });
        }
        if self.take(Punct::Mul) {
            return Ok(TypeSyntax::Pointer(Box::new(
                self.type_syntax_with_results(result_list)?,
            )));
        }
        if self.take(Punct::OpenBracket) {
            if self.take(Punct::CloseBracket) {
                return Ok(TypeSyntax::Slice(Box::new(
                    self.type_syntax_with_results(result_list)?,
                )));
            }
            if self.take(Punct::Range) {
                self.need(Punct::CloseBracket)?;
                return Ok(TypeSyntax::DynamicArray(Box::new(
                    self.type_syntax_with_results(result_list)?,
                )));
            }
            let count = Box::new(self.expression(0)?);
            self.need(Punct::CloseBracket)?;
            return Ok(TypeSyntax::FixedArray {
                count,
                element: Box::new(self.type_syntax_with_results(result_list)?),
            });
        }
        let explicit_procedure = self.token().kind == Kind::Directive(Directive::Type);
        if explicit_procedure {
            self.at += 1;
            if self.take(Punct::Comma) {
                let kind = match self.text() {
                    "distinct" if self.token().kind == Kind::Ident => TypeVariantKind::Distinct,
                    "isa" if self.token().kind == Kind::Ident => TypeVariantKind::IsA,
                    _ => return Err(self.error("expected distinct or isa type modifier")),
                };
                self.at += 1;
                return Ok(TypeSyntax::Variant {
                    kind,
                    base: Box::new(self.type_syntax_with_results(result_list)?),
                });
            }
            if let Some(tag) = self.compiler_type_tag() {
                return Ok(TypeSyntax::Builtin(tag));
            }
        }
        if self.is(Punct::OpenParen) {
            return Ok(TypeSyntax::Procedure(
                self.procedure_type_syntax(result_list)?,
            ));
        }
        if explicit_procedure {
            return self.type_syntax_with_results(result_list);
        }
        if self.token().kind != Kind::Ident {
            return Err(self.error("expected type expression"));
        }
        let builtin = BuiltinType::from_spelling(&self.token().spelling(self.source));
        let root = self.name()?;
        let mut members = Vec::new();
        while self.take(Punct::Dot) {
            members.push(self.name()?);
        }
        let base = match (builtin, members.is_empty()) {
            (Some(builtin), true) => TypeSyntax::Builtin(builtin),
            _ => TypeSyntax::Named(NamePath {
                root,
                members,
            }),
        };
        if self.is(Punct::OpenParen) {
            Ok(TypeSyntax::Application(self.type_application(base, start)?))
        } else {
            Ok(base)
        }
    }

    pub(super) fn nominal_prefix(&self) -> Option<Keyword> {
        if !self.named_prefix(Punct::Constant) {
            return None;
        }
        match self.tokens.get(self.at + 2)?.kind {
            Kind::Keyword(
                keyword @ (Keyword::Struct | Keyword::Union | Keyword::Enum | Keyword::EnumFlags),
            ) => Some(keyword),
            _ => None,
        }
    }
    pub(super) fn record_declaration(&mut self) -> Result<RecordDeclaration, Diagnostic> {
        let start = self.token().span.start;
        let name = self.name()?;
        self.need(Punct::Constant)?;
        let mut record = self.record_type()?;
        record.notes.extend(self.notes()?);
        self.take(Punct::Semicolon);
        Ok(RecordDeclaration {
            name,
            kind: record.kind,
            members: record.members,
            parameters: record.parameters,
            attributes: record.attributes,
            notes: record.notes,
            modify: record.modify,
            span: Span::new(start, self.tokens[self.at - 1].span.end),
        })
    }
    pub(super) fn record_type(&mut self) -> Result<RecordTypeSyntax, Diagnostic> {
        let start = self.token().span.start;
        let kind = if self.keyword(Keyword::Struct) {
            RecordKind::Struct
        } else if self.keyword(Keyword::Union) {
            RecordKind::Union
        } else {
            return Err(self.error("expected struct or union"));
        };
        let parameters = self.record_parameters()?;
        let mut notes = Vec::new();
        let mut attributes = Vec::new();
        loop {
            let before = self.at;
            notes.extend(self.notes()?);
            let next = self.record_attributes()?;
            metadata::merge_record_attributes(
                &mut attributes,
                next,
                Span::new(start, self.tokens[self.at - 1].span.end),
            )?;
            if before == self.at {
                break;
            }
        }
        let modify = self.modify_directive()?;
        let members = self.record_members()?;
        let suffix = self.record_attributes()?;
        metadata::merge_record_attributes(
            &mut attributes,
            suffix,
            Span::new(start, self.tokens[self.at - 1].span.end),
        )?;
        notes.extend(self.notes()?);
        Ok(RecordTypeSyntax {
            kind,
            members,
            parameters,
            attributes,
            notes,
            modify,
            span: Span::new(start, self.tokens[self.at - 1].span.end),
        })
    }
    pub(super) fn enum_declaration(&mut self) -> Result<EnumDeclaration, Diagnostic> {
        let start = self.token().span.start;
        let name = self.name()?;
        self.need(Punct::Constant)?;
        let enumeration = self.enum_type()?;
        self.take(Punct::Semicolon);
        Ok(EnumDeclaration {
            name,
            representation: enumeration.representation,
            kind: enumeration.kind,
            specified: enumeration.specified,
            members: enumeration.members,
            notes: enumeration.notes,
            span: Span::new(start, self.tokens[self.at - 1].span.end),
        })
    }
    /// Anonymous annotations leave the surrounding declaration terminator unread.
    fn enum_type(&mut self) -> Result<EnumTypeSyntax, Diagnostic> {
        let start = self.token().span.start;
        let kind = if self.keyword(Keyword::Enum) {
            EnumKind::Values
        } else if self.keyword(Keyword::EnumFlags) {
            EnumKind::Flags
        } else {
            return Err(self.error("expected enum or enum_flags"));
        };
        let mut notes = self.notes()?;
        let representation = if self.is(Punct::OpenBrace)
            || self.token().kind == Kind::Directive(Directive::Specified)
        {
            None
        } else {
            Some(self.type_syntax()?)
        };
        notes.extend(self.notes()?);
        let specified = self.token().kind == Kind::Directive(Directive::Specified);
        if specified {
            self.at += 1;
        }
        notes.extend(self.notes()?);
        self.need(Punct::OpenBrace)?;
        let members = self.enum_body_items(0)?;
        notes.extend(self.notes()?);
        Ok(EnumTypeSyntax {
            notes,
            representation,
            kind,
            specified,
            members,
            span: Span::new(start, self.tokens[self.at - 1].span.end),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_source::{LocatedDiagnostic, SourceMap};
    use jai_types::{CallingConvention, ContextMode};

    fn file(source: &str, symbols: &mut Symbols) -> Result<ParsedFile, LocatedDiagnostic> {
        let mut sources = SourceMap::default();
        let id = sources.insert("types.jai".into(), source.into());
        parse_file(sources.get(id).unwrap(), symbols)
    }
    fn declaration(file: &ParsedFile, index: usize) -> &FileDeclarationKind {
        let FileItem::Declaration(declaration) = &file.items()[index] else {
            panic!("expected declaration")
        };
        &declaration.kind
    }
    fn explicit(field: &FieldDeclaration) -> &TypeSyntax {
        let FieldBinding::Explicit {
            ty, ..
        } = &field.binding
        else {
            panic!("expected explicit field")
        };
        ty
    }

    #[test]
    fn ordered_fields_grouping_defaults_and_using_keep_source_identity() {
        let source = "Vector :: struct { x, y: int; count: u32 = 9; active := true; using position: Geometry.Point; }";
        let mut symbols = Symbols::default();
        let parsed = file(source, &mut symbols).unwrap();
        let FileDeclarationKind::Record(record) = declaration(&parsed, 0) else {
            panic!("expected record")
        };
        assert_eq!(record.kind, RecordKind::Struct);
        assert_eq!(symbols.name(record.name), "Vector");
        let names: Vec<_> = record
            .fields()
            .map(|field| symbols.name(field.name))
            .collect();
        assert_eq!(names, ["x", "y", "count", "active", "position"]);
        assert_eq!(
            explicit(record.fields().next().unwrap()).as_scalar(),
            Some(ScalarType::Int(IntegerType::S64))
        );
        assert!(matches!(
            record.fields().nth(2).unwrap().binding,
            FieldBinding::Explicit {
                initializer: Some(Expression {
                    kind: ExpressionKind::Integer(9),
                    ..
                }),
                ..
            }
        ));
        assert!(matches!(
            record.fields().nth(3).unwrap().binding,
            FieldBinding::Inferred(Expression {
                kind: ExpressionKind::Bool(true),
                ..
            })
        ));
        assert!(record.fields().nth(4).unwrap().using);
        let TypeSyntax::Named(path) = explicit(record.fields().nth(4).unwrap()) else {
            panic!("expected named type")
        };
        assert_eq!(symbols.name(path.root), "Geometry");
        assert_eq!(symbols.name(path.members[0]), "Point");
        assert_eq!(
            record.fields().next().unwrap().span.text(source),
            "x, y: int;"
        );
        assert_eq!(record.span.text(source), source);
        assert_eq!(parsed.location(record.span).source, parsed.source());
    }

    #[test]
    fn nested_arrays_pointers_builtins_and_procedure_signatures_are_unresolved() {
        let mut symbols = Symbols::default();
        let parsed = file("Buffer :: struct { fixed: [Count + 1] *Geometry.Point; view: [] u8; growing: [..] *[] s16; callback: #type (using self: *Buffer, int) -> (ok: bool, amount: u32) #c_call #no_context; scalar_callback: () -> int, bool; name: string; weight: float; precise: float64; raw: *void; info: Type; }", &mut symbols).unwrap();
        let FileDeclarationKind::Record(record) = declaration(&parsed, 0) else {
            panic!()
        };
        let TypeSyntax::FixedArray {
            count,
            element,
        } = explicit(record.fields().next().unwrap())
        else {
            panic!()
        };
        assert!(matches!(
            count.kind,
            ExpressionKind::Binary(BinaryOp::Add, _, _)
        ));
        assert!(
            matches!(element.as_ref(), TypeSyntax::Pointer(inner) if matches!(inner.as_ref(), TypeSyntax::Named(_)))
        );
        assert!(
            matches!(explicit(record.fields().nth(1).unwrap()), TypeSyntax::Slice(element) if element.as_scalar() == Some(ScalarType::Int(IntegerType::U8)))
        );
        assert!(
            matches!(explicit(record.fields().nth(2).unwrap()), TypeSyntax::DynamicArray(element) if matches!(element.as_ref(), TypeSyntax::Pointer(inner) if matches!(inner.as_ref(), TypeSyntax::Slice(_))))
        );
        let TypeSyntax::Procedure(signature) = explicit(record.fields().nth(3).unwrap()) else {
            panic!()
        };
        assert_eq!(signature.parameters.len(), 2);
        assert!(signature.parameters[0].using);
        assert!(signature.parameters[1].name.is_none());
        assert_eq!(signature.results.len(), 2);
        assert_eq!(symbols.name(signature.results[0].name.unwrap()), "ok");
        assert_eq!(signature.convention, CallingConvention::C);
        assert_eq!(signature.context, ContextMode::None);
        let TypeSyntax::Procedure(signature) = explicit(record.fields().nth(4).unwrap()) else {
            panic!()
        };
        assert_eq!(signature.results.len(), 2);
        assert!(matches!(
            explicit(record.fields().nth(5).unwrap()),
            TypeSyntax::Builtin(BuiltinType::String)
        ));
        assert!(matches!(
            explicit(record.fields().nth(6).unwrap()),
            TypeSyntax::Builtin(BuiltinType::Float(FloatType::F32))
        ));
        assert!(matches!(
            explicit(record.fields().nth(7).unwrap()),
            TypeSyntax::Builtin(BuiltinType::Float(FloatType::F64))
        ));
        assert!(
            matches!(explicit(record.fields().nth(8).unwrap()), TypeSyntax::Pointer(inner) if matches!(inner.as_ref(), TypeSyntax::Builtin(BuiltinType::Void)))
        );
        assert!(matches!(
            explicit(record.fields().nth(9).unwrap()),
            TypeSyntax::Builtin(BuiltinType::Type)
        ));
    }

    #[test]
    fn enumeration_order_representation_and_initializers_are_preserved() {
        let mut symbols = Symbols::default();
        let parsed = file("Fruit :: enum u32 { BANANA :: 5; APPLE; APRICOT :: 12; } Bits :: enum_flags u8 #specified { A :: 1; B :: 2; BOTH :: A | B; }; Default :: enum { FIRST; SECOND; }", &mut symbols).unwrap();
        let FileDeclarationKind::Enum(fruit) = declaration(&parsed, 0) else {
            panic!()
        };
        assert_eq!(fruit.kind, EnumKind::Values);
        assert!(!fruit.specified);
        assert_eq!(
            fruit.representation.as_ref().unwrap().as_scalar(),
            Some(ScalarType::Int(IntegerType::U32))
        );
        assert_eq!(
            EnumMemberSyntax::new(&fruit.members)
                .map(|member| symbols.name(member.name))
                .collect::<Vec<_>>(),
            ["BANANA", "APPLE", "APRICOT"]
        );
        assert!(
            EnumMemberSyntax::new(&fruit.members)
                .nth(1)
                .unwrap()
                .initializer
                .is_none()
        );
        let FileDeclarationKind::Enum(bits) = declaration(&parsed, 1) else {
            panic!()
        };
        assert_eq!(bits.kind, EnumKind::Flags);
        assert!(bits.specified);
        assert!(matches!(
            EnumMemberSyntax::new(&bits.members)
                .nth(2)
                .unwrap()
                .initializer
                .as_ref()
                .unwrap()
                .kind,
            ExpressionKind::Binary(BinaryOp::BitOr, _, _)
        ));
        let FileDeclarationKind::Enum(default) = declaration(&parsed, 2) else {
            panic!()
        };
        assert!(default.representation.is_none());
    }

    #[test]
    fn literal_types_context_fields_and_member_precedence_are_distinct() {
        let mut symbols = Symbols::default();
        let source = "main :: () -> int { a := Geometry.Point.{x=1, child=.{enabled=true},}; b: Geometry.Point = .{}; return make(a).child.x + Geometry.Point.{x=2}.x * 3; }";
        let parsed = file(source, &mut symbols).unwrap();
        let FileDeclarationKind::Procedure(procedure) = declaration(&parsed, 0) else {
            panic!()
        };
        let StatementKind::Declare(Declaration::Inferred {
            initializer, ..
        }) = &procedure.body[0].kind
        else {
            panic!()
        };
        let ExpressionKind::StructLiteral(literal) = &initializer.kind else {
            panic!()
        };
        let TypeSyntax::Named(path) = literal.ty.as_ref().unwrap() else {
            panic!("named target")
        };
        assert_eq!(symbols.name(path.root), "Geometry");
        assert_eq!(symbols.name(path.members[0]), "Point");
        assert_eq!(literal.fields.len(), 2);
        assert_eq!(literal.fields[0].span.text(source), "x=1");
        assert!(matches!(
            &literal.fields[1].value.kind,
            ExpressionKind::StructLiteral(StructLiteral {
                ty: None,
                ..
            })
        ));
        let StatementKind::Declare(Declaration::UnresolvedExplicit {
            ty,
            initializer: Some(initializer),
            ..
        }) = &procedure.body[1].kind
        else {
            panic!()
        };
        assert!(matches!(ty, TypeSyntax::Named(_)));
        assert!(
            matches!(&initializer.kind, ExpressionKind::StructLiteral(StructLiteral { ty: None, fields }) if fields.is_empty())
        );
        let StatementKind::Return(Some(expression)) = &procedure.body[2].kind else {
            panic!()
        };
        let ExpressionKind::Binary(BinaryOp::Add, left, right) = &expression.kind else {
            panic!()
        };
        assert!(
            matches!(&left.kind, ExpressionKind::Member { base, member } if symbols.name(*member) == "x" && matches!(base.kind, ExpressionKind::Member { .. }))
        );
        assert!(
            matches!(&right.kind, ExpressionKind::Binary(BinaryOp::Multiply, left, _) if matches!(left.kind, ExpressionKind::Member { .. }))
        );
    }

    #[test]
    fn scalar_annotations_keep_the_explicit_bridge_and_nominal_spelling_is_shared() {
        let mut symbols = Symbols::default();
        let one = file("Point :: struct { x: int; }", &mut symbols).unwrap();
        let two = file("p: Point; n: int;", &mut symbols).unwrap();
        let FileDeclarationKind::Record(record) = declaration(&one, 0) else {
            panic!()
        };
        let FileDeclarationKind::Global(global) = declaration(&two, 0) else {
            panic!()
        };
        let Declaration::UnresolvedExplicit {
            name,
            ty: TypeSyntax::Named(path),
            initializer,
            ..
        } = &global.declaration
        else {
            panic!()
        };
        assert_eq!(global.declaration.name(), *name);
        assert_eq!(path.root, record.name);
        assert!(initializer.is_none());
        let FileDeclarationKind::Global(global) = declaration(&two, 1) else {
            panic!()
        };
        assert!(matches!(
            global.declaration,
            Declaration::Explicit {
                ty: ScalarType::Int(IntegerType::S64),
                ..
            }
        ));
    }

    #[test]
    fn reference_and_recent_upstream_shapes_have_regressions() {
        let mut symbols = Symbols::default();
        // Member_Offset and Special_Float_Handling from the pinned jaison source.
        assert!(file("Member_Offset :: struct { member: *Type_Info_Struct_Member; offset_in_bytes: s64; } Special_Float_Handling :: enum_flags { SUPPORT_NULL :: 0x01; SUPPORT_STRINGS :: 0x02; SUPPORT_ALL :: SUPPORT_NULL | SUPPORT_STRINGS; }", &mut symbols).is_ok());
        // Grouped fields and using embedding from how_to/006 and how_to/042.
        assert!(file("Vector3 :: struct { x, y, z: float; } Entity :: struct { using position: Vector3; id: s64; }", &mut symbols).is_ok());
    }

    #[test]
    fn any_annotations_are_builtin_tags_instead_of_named_spelling_checks() {
        let parsed = file(
            "values:[]Any; take::(args:..Any){}",
            &mut Symbols::default(),
        )
        .unwrap();
        let FileDeclarationKind::Global(GlobalDeclaration {
            declaration:
                Declaration::UnresolvedExplicit {
                    ty: TypeSyntax::Slice(element),
                    ..
                },
            ..
        }) = declaration(&parsed, 0)
        else {
            panic!("expected slice annotation")
        };
        assert!(matches!(
            element.as_ref(),
            TypeSyntax::Builtin(BuiltinType::Any)
        ));
        let FileDeclarationKind::Procedure(procedure) = declaration(&parsed, 1) else {
            panic!("expected procedure")
        };
        assert!(procedure.parameters[0].variadic);
        assert!(matches!(
            procedure.parameters[0].binding,
            ParameterBinding::RequiredType(TypeSyntax::Builtin(BuiltinType::Any))
        ));
    }

    #[test]
    fn grouped_field_defaults_preserve_the_shared_initializer_span() {
        let text = "Texture :: struct { width,height,depth:s32 = 1; }";
        let parsed = file(text, &mut Symbols::default()).unwrap();
        let FileDeclarationKind::Record(record) = declaration(&parsed, 0) else {
            panic!("expected record")
        };
        assert_eq!(record.fields().count(), 3);
        let mut initializer_spans = Vec::new();
        for field in record.fields() {
            let FieldBinding::Explicit {
                initializer: Some(initializer),
                ..
            } = &field.binding
            else {
                panic!("expected initializer")
            };
            assert!(matches!(initializer.kind, ExpressionKind::Integer(1)));
            initializer_spans.push(initializer.span);
        }
        assert_eq!(initializer_spans, vec![initializer_spans[0]; 3]);
        assert_eq!(initializer_spans[0].text(text), "1");
    }

    #[test]
    fn record_layout_inline_fields_and_variant_aliases_preserve_source_structure() {
        let source = "Handle :: #type,distinct u32; Filename :: #type,isa string; Packed :: struct #type_info_none { pointer: *int #align 4; _context: struct { value: int; } @JsonName(context) } #no_padding @Serializable";
        let parsed = file(source, &mut Symbols::default()).unwrap();
        let FileDeclarationKind::TypeAlias(handle) = declaration(&parsed, 0) else {
            panic!("expected alias")
        };
        assert!(matches!(
            handle.ty,
            TypeSyntax::Variant {
                kind: TypeVariantKind::Distinct,
                ..
            }
        ));
        let FileDeclarationKind::TypeAlias(filename) = declaration(&parsed, 1) else {
            panic!("expected alias")
        };
        assert!(matches!(
            filename.ty,
            TypeSyntax::Variant {
                kind: TypeVariantKind::IsA,
                ..
            }
        ));
        let FileDeclarationKind::Record(record) = declaration(&parsed, 2) else {
            panic!("expected record")
        };
        assert!(matches!(
            record.attributes.as_slice(),
            [RecordAttribute::TypeInfoNone, RecordAttribute::NoPadding]
        ));
        assert!(matches!(
            record.fields().next().unwrap().attributes.as_slice(),
            [FieldAttribute::Alignment(Expression {
                kind: ExpressionKind::Integer(4),
                ..
            })]
        ));
        let TypeSyntax::InlineRecord(inline) = explicit(record.fields().nth(1).unwrap()) else {
            panic!("expected inline record")
        };
        assert_eq!(inline.fields().count(), 1);
        assert!(inline.notes.is_empty());
        assert_eq!(record.fields().nth(1).unwrap().notes.len(), 1);
        assert_eq!(record.notes.len(), 1);
    }

    #[test]
    fn local_nominal_declarations_and_positional_literals_are_ordered() {
        let parsed = file("main :: () { Local :: struct { x,y:int; } Status :: enum { READY; } Count :: int; callback :: () -> int { return 3; } external :: () #foreign Lib; point := Local.{1,2,}; }", &mut Symbols::default()).unwrap();
        let FileDeclarationKind::Procedure(procedure) = declaration(&parsed, 0) else {
            panic!("expected procedure")
        };
        assert!(matches!(procedure.body[0].kind, StatementKind::Record(_)));
        assert!(matches!(procedure.body[1].kind, StatementKind::Enum(_)));
        assert!(matches!(
            procedure.body[2].kind,
            StatementKind::TypeAlias(_)
        ));
        assert!(matches!(
            procedure.body[3].kind,
            StatementKind::Procedure(_)
        ));
        assert!(matches!(
            procedure.body[4].kind,
            StatementKind::ProcedurePrototype(_)
        ));
        let StatementKind::Declare(Declaration::Inferred {
            initializer, ..
        }) = &procedure.body[5].kind
        else {
            panic!("expected local")
        };
        let ExpressionKind::PositionalStructLiteral(literal) = &initializer.kind else {
            panic!("expected positional literal")
        };
        assert!(literal.ty.is_some());
        assert!(matches!(
            literal.values.as_slice(),
            [
                Expression {
                    kind: ExpressionKind::Integer(1),
                    ..
                },
                Expression {
                    kind: ExpressionKind::Integer(2),
                    ..
                }
            ]
        ));
    }

    #[test]
    fn malformed_and_unimplemented_aggregate_syntax_has_located_errors() {
        for source in [
            "Point :: struct { x: int;",
            "Point :: struct { x: ; }",
            "Point :: struct { x: int }",
            "Point :: struct { x: [.. 2] int; }",
            "Point :: struct { x: [] ; }",
            "Point :: struct { x: [3 int; }",
            "Point :: struct { x: *; }",
            "Point :: struct { x: #type,unknown int; }",
            "Point :: struct { x: () ->; }",
            "Point :: struct { x: (int; }",
            "Point :: struct { x, y := 1; }",
            "Point :: struct { x, y: int = ; }",
            "Point :: struct { using x, y: int; }",
            "Point :: struct #align { x: int; }",
            "Fruit :: enum { APPLE, BANANA }",
            "Fruit :: enum { APPLE = 1; }",
            "Fruit :: enum { APPLE;",
            "x := Point.{x=};",
            "x := Point.{1,x=2};",
            "x := .{x=1 y=2};",
            "x := 1.{x=2};",
            "x := Point.{x=1;",
            "main :: () { Local :: struct { x: int; } value := Local.{1,; }",
        ] {
            let error = file(source, &mut Symbols::default()).unwrap_err();
            assert!(error.location.span.start <= source.len(), "{source}");
            assert!(!error.message.is_empty(), "{source}");
        }
    }

    #[test]
    fn executable_parser_does_not_claim_aggregate_lowering() {
        for source in [
            "Point :: struct { x: int; }",
            "Fruit :: enum { APPLE; }",
            "p: Point;",
            "bytes: [] u8;",
            "main :: () { p := Point.{x=1}; }",
            "main :: () { p: Point = .{x=1}; }",
            "main :: () -> int { return make().x; }",
            "main :: () -> int { return p.x; }",
        ] {
            assert!(parse(source).is_err(), "{source}");
        }
    }
}
