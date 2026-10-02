//! Unresolved type expressions and ordered nominal declarations.
use super::*;
use jai_types::{CallingConvention, ContextMode, FloatType, RecordKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuiltinType {
    Scalar(ScalarType),
    Float(FloatType),
    String,
    Void,
    Type,
}

impl BuiltinType {
    pub(super) fn from_spelling(text: &str) -> Option<Self> {
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
            _ => return None,
        })
    }
}

#[derive(Clone, Debug)]
pub enum TypeSyntax {
    Builtin(BuiltinType),
    Named(NamePath),
    Pointer(Box<TypeSyntax>),
    FixedArray {
        count: Box<Expression>,
        element: Box<TypeSyntax>,
    },
    Slice(Box<TypeSyntax>),
    DynamicArray(Box<TypeSyntax>),
    Procedure(ProcedureTypeSyntax),
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
pub struct ProcedureTypeParameter {
    pub name: Option<Symbol>,
    pub ty: TypeSyntax,
    pub using: bool,
    pub span: Span,
}
#[derive(Clone, Debug)]
pub struct ProcedureTypeSyntax {
    pub parameters: Vec<ProcedureTypeParameter>,
    pub results: Vec<ProcedureTypeParameter>,
    pub convention: CallingConvention,
    pub context: ContextMode,
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
    pub span: Span,
}
#[derive(Clone, Debug)]
pub struct RecordDeclaration {
    pub name: Symbol,
    pub kind: RecordKind,
    pub fields: Vec<FieldDeclaration>,
    pub span: Span,
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
}
#[derive(Clone, Debug)]
pub struct EnumDeclaration {
    pub name: Symbol,
    pub representation: Option<TypeSyntax>,
    pub kind: EnumKind,
    pub specified: bool,
    pub members: Vec<EnumMember>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct StructLiteralField {
    pub name: Symbol,
    pub value: Expression,
    pub span: Span,
}
#[derive(Clone, Debug)]
pub struct StructLiteral {
    pub ty: Option<NamePath>,
    pub fields: Vec<StructLiteralField>,
}

impl Parser<'_> {
    pub(super) fn type_syntax(&mut self) -> Result<TypeSyntax, Diagnostic> {
        if self.take(Punct::Mul) {
            return Ok(TypeSyntax::Pointer(Box::new(self.type_syntax()?)));
        }
        if self.take(Punct::OpenBracket) {
            if self.take(Punct::CloseBracket) {
                return Ok(TypeSyntax::Slice(Box::new(self.type_syntax()?)));
            }
            if self.take(Punct::Range) {
                self.need(Punct::CloseBracket)?;
                return Ok(TypeSyntax::DynamicArray(Box::new(self.type_syntax()?)));
            }
            let count = Box::new(self.expression(0)?);
            self.need(Punct::CloseBracket)?;
            return Ok(TypeSyntax::FixedArray {
                count,
                element: Box::new(self.type_syntax()?),
            });
        }
        let explicit_procedure = self.token().kind == Kind::Directive(Directive::Type);
        if explicit_procedure {
            self.at += 1;
        }
        if self.is(Punct::OpenParen) {
            return Ok(TypeSyntax::Procedure(self.procedure_type_syntax()?));
        }
        if explicit_procedure {
            return Err(self.error("expected procedure signature after #type"));
        }
        if self.token().kind != Kind::Ident {
            return Err(self.error("expected type expression"));
        }
        let builtin = BuiltinType::from_spelling(self.text());
        let root = self.name()?;
        let mut members = Vec::new();
        while self.take(Punct::Dot) {
            members.push(self.name()?);
        }
        Ok(match (builtin, members.is_empty()) {
            (Some(builtin), true) => TypeSyntax::Builtin(builtin),
            _ => TypeSyntax::Named(NamePath { root, members }),
        })
    }

    fn procedure_type_parameter(&mut self) -> Result<ProcedureTypeParameter, Diagnostic> {
        let start = self.token().span.start;
        let using = self.keyword(Keyword::Using);
        let name = if self.named_prefix(Punct::Colon) {
            let name = self.name()?;
            self.need(Punct::Colon)?;
            Some(name)
        } else {
            None
        };
        if using && name.is_none() {
            return Err(self.error("using procedure parameters require a name"));
        }
        let ty = self.type_syntax()?;
        Ok(ProcedureTypeParameter {
            name,
            ty,
            using,
            span: Span::new(start, self.tokens[self.at - 1].span.end),
        })
    }
    fn procedure_type_parameters(&mut self) -> Result<Vec<ProcedureTypeParameter>, Diagnostic> {
        self.need(Punct::OpenParen)?;
        let mut parameters = Vec::new();
        if !self.take(Punct::CloseParen) {
            loop {
                parameters.push(self.procedure_type_parameter()?);
                if self.take(Punct::CloseParen) {
                    break;
                }
                self.need(Punct::Comma)?;
            }
        }
        Ok(parameters)
    }
    fn procedure_type_syntax(&mut self) -> Result<ProcedureTypeSyntax, Diagnostic> {
        let parameters = self.procedure_type_parameters()?;
        let mut results = Vec::new();
        if self.take(Punct::Arrow) {
            if self.is(Punct::OpenParen) {
                results = self.procedure_type_parameters()?;
            } else {
                loop {
                    results.push(self.procedure_type_parameter()?);
                    if !self.take(Punct::Comma) {
                        break;
                    }
                }
            }
        }
        let mut convention = CallingConvention::Jai;
        let mut context = ContextMode::Implicit;
        loop {
            match self.token().kind {
                Kind::Directive(Directive::NoContext) if context == ContextMode::Implicit => {
                    context = ContextMode::None;
                }
                Kind::Directive(Directive::CCall) if convention == CallingConvention::Jai => {
                    convention = CallingConvention::C;
                }
                _ => break,
            }
            self.at += 1;
        }
        Ok(ProcedureTypeSyntax {
            parameters,
            results,
            convention,
            context,
        })
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
        let kind = if self.keyword(Keyword::Struct) {
            RecordKind::Struct
        } else if self.keyword(Keyword::Union) {
            RecordKind::Union
        } else {
            return Err(self.error("expected struct or union"));
        };
        self.need(Punct::OpenBrace)?;
        let mut fields = Vec::new();
        while !self.take(Punct::CloseBrace) {
            if self.token().kind == Kind::Eof {
                return Err(self.error("unterminated record declaration"));
            }
            let field_start = self.token().span.start;
            let using = self.keyword(Keyword::Using);
            let mut names = vec![self.name()?];
            while self.take(Punct::Comma) {
                names.push(self.name()?);
            }
            if using && names.len() != 1 {
                return Err(self.error("grouped using record fields are not implemented"));
            }
            let binding = if self.take(Punct::Infer) {
                if names.len() != 1 {
                    return Err(self.error("grouped inferred record fields are not implemented"));
                }
                FieldBinding::Inferred(self.expression(0)?)
            } else {
                self.need(Punct::Colon)?;
                let ty = self.type_syntax()?;
                let initializer = if self.take(Punct::Assign) {
                    Some(self.expression(0)?)
                } else {
                    None
                };
                if names.len() != 1 && initializer.is_some() {
                    return Err(self.error("grouped record field defaults are not implemented"));
                }
                FieldBinding::Explicit { ty, initializer }
            };
            self.need(Punct::Semicolon)?;
            let span = Span::new(field_start, self.tokens[self.at - 1].span.end);
            for name in names {
                fields.push(FieldDeclaration {
                    name,
                    binding: binding.clone(),
                    using,
                    span,
                });
            }
        }
        self.take(Punct::Semicolon);
        Ok(RecordDeclaration {
            name,
            kind,
            fields,
            span: Span::new(start, self.tokens[self.at - 1].span.end),
        })
    }
    pub(super) fn enum_declaration(&mut self) -> Result<EnumDeclaration, Diagnostic> {
        let start = self.token().span.start;
        let name = self.name()?;
        self.need(Punct::Constant)?;
        let kind = if self.keyword(Keyword::Enum) {
            EnumKind::Values
        } else if self.keyword(Keyword::EnumFlags) {
            EnumKind::Flags
        } else {
            return Err(self.error("expected enum or enum_flags"));
        };
        let representation = if self.is(Punct::OpenBrace)
            || self.token().kind == Kind::Directive(Directive::Specified)
        {
            None
        } else {
            Some(self.type_syntax()?)
        };
        let specified = self.token().kind == Kind::Directive(Directive::Specified);
        if specified {
            self.at += 1;
        }
        self.need(Punct::OpenBrace)?;
        let mut members = Vec::new();
        while !self.take(Punct::CloseBrace) {
            if self.token().kind == Kind::Eof {
                return Err(self.error("unterminated enum declaration"));
            }
            let start = self.token().span.start;
            let name = self.name()?;
            let initializer = if self.take(Punct::Constant) {
                Some(self.expression(0)?)
            } else {
                None
            };
            self.need(Punct::Semicolon)?;
            members.push(EnumMember {
                name,
                initializer,
                span: Span::new(start, self.tokens[self.at - 1].span.end),
            });
        }
        self.take(Punct::Semicolon);
        Ok(EnumDeclaration {
            name,
            representation,
            kind,
            specified,
            members,
            span: Span::new(start, self.tokens[self.at - 1].span.end),
        })
    }
    pub(super) fn struct_literal(
        &mut self,
        ty: Option<NamePath>,
        start: usize,
    ) -> Result<Expression, Diagnostic> {
        self.need(Punct::StructLiteral)?;
        let mut fields = Vec::new();
        if !self.take(Punct::CloseBrace) {
            loop {
                let field_start = self.token().span.start;
                if !self.named_prefix(Punct::Assign) {
                    return Err(self.error("only named struct literal fields are implemented"));
                }
                let name = self.name()?;
                self.need(Punct::Assign)?;
                let value = self.expression(0)?;
                let span = Span::new(field_start, value.span.end);
                fields.push(StructLiteralField { name, value, span });
                if self.take(Punct::CloseBrace) {
                    break;
                }
                self.need(Punct::Comma)?;
                if self.take(Punct::CloseBrace) {
                    break;
                }
            }
        }
        Ok(Expression {
            kind: ExpressionKind::StructLiteral(StructLiteral { ty, fields }),
            span: Span::new(start, self.tokens[self.at - 1].span.end),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_source::{LocatedDiagnostic, SourceMap};

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
        let FieldBinding::Explicit { ty, .. } = &field.binding else {
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
            .fields
            .iter()
            .map(|field| symbols.name(field.name))
            .collect();
        assert_eq!(names, ["x", "y", "count", "active", "position"]);
        assert_eq!(
            explicit(&record.fields[0]).as_scalar(),
            Some(ScalarType::Int(IntegerType::S64))
        );
        assert!(matches!(
            record.fields[2].binding,
            FieldBinding::Explicit {
                initializer: Some(Expression {
                    kind: ExpressionKind::Integer(9),
                    ..
                }),
                ..
            }
        ));
        assert!(matches!(
            record.fields[3].binding,
            FieldBinding::Inferred(Expression {
                kind: ExpressionKind::Bool(true),
                ..
            })
        ));
        assert!(record.fields[4].using);
        let TypeSyntax::Named(path) = explicit(&record.fields[4]) else {
            panic!("expected named type")
        };
        assert_eq!(symbols.name(path.root), "Geometry");
        assert_eq!(symbols.name(path.members[0]), "Point");
        assert_eq!(record.fields[0].span.text(source), "x, y: int;");
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
        let TypeSyntax::FixedArray { count, element } = explicit(&record.fields[0]) else {
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
            matches!(explicit(&record.fields[1]), TypeSyntax::Slice(element) if element.as_scalar() == Some(ScalarType::Int(IntegerType::U8)))
        );
        assert!(
            matches!(explicit(&record.fields[2]), TypeSyntax::DynamicArray(element) if matches!(element.as_ref(), TypeSyntax::Pointer(inner) if matches!(inner.as_ref(), TypeSyntax::Slice(_))))
        );
        let TypeSyntax::Procedure(signature) = explicit(&record.fields[3]) else {
            panic!()
        };
        assert_eq!(signature.parameters.len(), 2);
        assert!(signature.parameters[0].using);
        assert!(signature.parameters[1].name.is_none());
        assert_eq!(signature.results.len(), 2);
        assert_eq!(symbols.name(signature.results[0].name.unwrap()), "ok");
        assert_eq!(signature.convention, CallingConvention::C);
        assert_eq!(signature.context, ContextMode::None);
        let TypeSyntax::Procedure(signature) = explicit(&record.fields[4]) else {
            panic!()
        };
        assert_eq!(signature.results.len(), 2);
        assert!(matches!(
            explicit(&record.fields[5]),
            TypeSyntax::Builtin(BuiltinType::String)
        ));
        assert!(matches!(
            explicit(&record.fields[6]),
            TypeSyntax::Builtin(BuiltinType::Float(FloatType::F32))
        ));
        assert!(matches!(
            explicit(&record.fields[7]),
            TypeSyntax::Builtin(BuiltinType::Float(FloatType::F64))
        ));
        assert!(
            matches!(explicit(&record.fields[8]), TypeSyntax::Pointer(inner) if matches!(inner.as_ref(), TypeSyntax::Builtin(BuiltinType::Void)))
        );
        assert!(matches!(
            explicit(&record.fields[9]),
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
            fruit
                .members
                .iter()
                .map(|member| symbols.name(member.name))
                .collect::<Vec<_>>(),
            ["BANANA", "APPLE", "APRICOT"]
        );
        assert!(fruit.members[1].initializer.is_none());
        let FileDeclarationKind::Enum(bits) = declaration(&parsed, 1) else {
            panic!()
        };
        assert_eq!(bits.kind, EnumKind::Flags);
        assert!(bits.specified);
        assert!(matches!(
            bits.members[2].initializer.as_ref().unwrap().kind,
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
        let Statement::Declare(Declaration::Inferred { initializer, .. }) = &procedure.body[0]
        else {
            panic!()
        };
        let ExpressionKind::StructLiteral(literal) = &initializer.kind else {
            panic!()
        };
        assert_eq!(symbols.name(literal.ty.as_ref().unwrap().root), "Geometry");
        assert_eq!(
            symbols.name(literal.ty.as_ref().unwrap().members[0]),
            "Point"
        );
        assert_eq!(literal.fields.len(), 2);
        assert_eq!(literal.fields[0].span.text(source), "x=1");
        assert!(matches!(
            &literal.fields[1].value.kind,
            ExpressionKind::StructLiteral(StructLiteral { ty: None, .. })
        ));
        let Statement::Declare(Declaration::UnresolvedExplicit {
            ty,
            initializer: Some(initializer),
            ..
        }) = &procedure.body[1]
        else {
            panic!()
        };
        assert!(matches!(ty, TypeSyntax::Named(_)));
        assert!(
            matches!(&initializer.kind, ExpressionKind::StructLiteral(StructLiteral { ty: None, fields }) if fields.is_empty())
        );
        let Statement::Return(Some(expression)) = &procedure.body[2] else {
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
    fn malformed_and_unimplemented_aggregate_syntax_has_located_errors() {
        for source in [
            "Point :: struct { x: int;",
            "Point :: struct { x: ; }",
            "Point :: struct { x: int }",
            "Point :: struct { x: [.. 2] int; }",
            "Point :: struct { x: [] ; }",
            "Point :: struct { x: [3 int; }",
            "Point :: struct { x: *; }",
            "Point :: struct { x: #type int; }",
            "Point :: struct { x: () ->; }",
            "Point :: struct { x: (int; }",
            "Point :: struct { x: int = ---; }",
            "Point :: struct { x, y := 1; }",
            "Point :: struct { x, y: int = 1; }",
            "Point :: struct { using x, y: int; }",
            "Point :: struct { x :: 1; }",
            "Point :: struct #align 16 { x: int; }",
            "Fruit :: enum { APPLE, BANANA }",
            "Fruit :: enum { APPLE = 1; }",
            "Fruit :: enum { APPLE;",
            "x := Point.{x=};",
            "x := Point.{1,2};",
            "x := .{x=1 y=2};",
            "x := 1.{x=2};",
            "x := Point.{x=1;",
            "main :: () { p.x = 1; }",
            "main :: () { Local :: struct { x: int; } }",
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
