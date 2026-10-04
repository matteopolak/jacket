//! Value expressions and precedence parsing.
use super::*;

#[derive(Clone, Debug)]
pub struct Expression {
    pub kind: ExpressionKind,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum ExpressionKind {
    Integer(i128),
    Float(FloatLiteral),
    String(Vec<u8>),
    HereString(HereStringLiteral),
    Character(u8),
    Null,
    Type(TypeSyntax),
    CompileTime(CompileTimeRun),
    CompileTimePredicate,
    CallerLocation,
    SourceLocation,
    SourceFile,
    SourceFilepath,
    SourceLine,
    ShortLambda(Box<ShortLambda>),
    AnonymousProcedure(Box<SourceProcedureSyntax>),
    BakeArguments(Box<BakedArgumentsSyntax>),
    Code(CodeBody),
    Insert(Box<InsertDirective>),
    Uninitialized,
    Bool(bool),
    Name(Symbol),
    QualifiedName(NamePath),
    Call(Symbol, Vec<CallArgument>),
    QualifiedCall(NamePath, Vec<CallArgument>),
    CallHint {
        hint: jai_types::InlineHint,
        call: Box<Expression>,
    },
    IndirectCall {
        callee: Box<Expression>,
        args: Vec<CallArgument>,
    },
    ContextCall {
        callee: Box<Expression>,
        args: Vec<CallArgument>,
        overrides: Vec<CallArgument>,
    },
    Context,
    InferredMember(Symbol),
    CompileVariable(Symbol),
    AddressOf(Box<Expression>),
    Dereference(Box<Expression>),
    Index {
        base: Box<Expression>,
        index: Box<Expression>,
    },
    ArrayLiteral(ArrayLiteral),
    StructLiteral(StructLiteral),
    PositionalStructLiteral(PositionalStructLiteral),
    Member {
        base: Box<Expression>,
        member: Symbol,
    },
    Unary(UnaryOp, Box<Expression>),
    Cast(CastMode, ScalarType, Box<Expression>),
    InferredCast {
        mode: CastMode,
        value: Box<Expression>,
    },
    TypeCast {
        mode: CastMode,
        ty: TypeSyntax,
        value: Box<Expression>,
    },
    TypeQuery {
        query: TypeQueryKind,
        value: Box<Expression>,
    },
    Binary(BinaryOp, Box<Expression>, Box<Expression>),
    Conditional(ConditionalExpression),
}
#[derive(Clone, Debug, PartialEq)]
pub enum FloatLiteral {
    Decimal(DecimalLiteral),
    Bits32(u32),
    Bits64(u64),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TypeQueryKind {
    SizeOf,
    TypeOf,
    InitializerOf,
    TypeInfo,
    CodeOf,
}
#[derive(Clone, Debug)]
pub struct ArrayLiteral {
    pub element_type: Option<TypeSyntax>,
    pub elements: Vec<Expression>,
}
#[derive(Clone, Debug)]
pub struct ConditionalExpression {
    pub condition: Box<Expression>,
    pub then_value: ConditionalThenValue,
    pub else_value: Option<Box<Expression>>,
}

#[derive(Clone, Debug)]
pub enum ConditionalThenValue {
    Expression(Box<Expression>),
    ImplicitSubject,
}

impl Parser<'_> {
    pub(super) fn expression(&mut self, minimum: u8) -> Result<Expression, Diagnostic> {
        self.expression_until_call(minimum, false)
    }
    pub(super) fn expression_until_call(
        &mut self,
        minimum: u8,
        stop_after_call: bool,
    ) -> Result<Expression, Diagnostic> {
        let token = self.token();
        let span = token.span;
        if token.kind == Kind::UnknownDirective {
            return Err(self.error(format!("unknown directive '{}'", self.text())));
        }
        let unary = match token.kind {
            Kind::Punctuation(Punct::Sub) => Some(UnaryOp::Negate),
            Kind::Punctuation(Punct::Add) => Some(UnaryOp::Positive),
            Kind::Punctuation(Punct::Not) => Some(UnaryOp::LogicalNot),
            Kind::Punctuation(Punct::Complement) => Some(UnaryOp::Complement),
            _ => None,
        };
        let mut lhs = if matches!(
            token.kind,
            Kind::Keyword(Keyword::Inline | Keyword::NoInline)
        ) {
            let hint = if token.kind == Kind::Keyword(Keyword::Inline) {
                jai_types::InlineHint::Always
            } else {
                jai_types::InlineHint::Never
            };
            self.at += 1;
            if matches!(
                self.token().kind,
                Kind::Keyword(Keyword::Inline | Keyword::NoInline)
            ) {
                return Err(self.error("duplicate or conflicting call inlining modifiers"));
            }
            let call = self.expression_until_call(23, true)?;
            if !matches!(
                call.kind,
                ExpressionKind::Call(..)
                    | ExpressionKind::QualifiedCall(..)
                    | ExpressionKind::IndirectCall { .. }
                    | ExpressionKind::ContextCall { .. }
            ) {
                return Err(Diagnostic::new(
                    span,
                    "call inlining modifier requires a procedure call",
                ));
            }
            Expression {
                span: Span::new(span.start, call.span.end),
                kind: ExpressionKind::CallHint {
                    hint,
                    call: Box::new(call),
                },
            }
        } else if self.starts_anonymous_procedure() {
            self.anonymous_procedure()?
        } else if self.starts_short_lambda() {
            self.short_lambda()?
        } else if self.starts_parenthesized_dereference() {
            self.parenthesized_dereference()?
        } else if self.take(Punct::OpenParen) {
            let mut e = self.expression(0)?;
            self.need(Punct::CloseParen)?;
            e.span = Span::new(span.start, self.tokens[self.at - 1].span.end);
            e
        } else if self.is(Punct::StructLiteral) || self.is(Punct::OpenBrace) {
            if !self.allow_qualified {
                return Err(self.error("struct literals require aggregate type resolution"));
            }
            self.struct_literal(None, span.start)?
        } else if self.is(Punct::ArrayLiteral) {
            if !self.allow_qualified {
                return Err(self.error("array literals require type resolution"));
            }
            self.array_literal(None, span.start)?
        } else if self.take(Punct::ShiftLeft) {
            if !self.allow_qualified {
                return Err(self.error("dereference expressions require storage resolution"));
            }
            let value = self.expression(21)?;
            Expression {
                span: Span::new(span.start, value.span.end),
                kind: ExpressionKind::Dereference(Box::new(value)),
            }
        } else if self.take(Punct::Mul) {
            if !self.allow_qualified {
                return Err(self.error("address expressions require storage resolution"));
            }
            let value = self.expression(21)?;
            Expression {
                span: Span::new(span.start, value.span.end),
                kind: ExpressionKind::AddressOf(Box::new(value)),
            }
        } else if self.take(Punct::Dollar) {
            if !self.allow_qualified {
                return Err(
                    self.error("compile-time variable introductions require specialization")
                );
            }
            let name = self.name()?;
            Expression {
                span: Span::new(span.start, self.tokens[self.at - 1].span.end),
                kind: ExpressionKind::CompileVariable(name),
            }
        } else if self.keyword(Keyword::Context) {
            if !self.allow_qualified {
                return Err(self.error("context values require context resolution"));
            }
            Expression {
                span,
                kind: ExpressionKind::Context,
            }
        } else if self.take(Punct::Dot) {
            if !self.allow_qualified {
                return Err(self.error("inferred members require type resolution"));
            }
            let name = self.name()?;
            Expression {
                span: Span::new(span.start, self.tokens[self.at - 1].span.end),
                kind: ExpressionKind::InferredMember(name),
            }
        } else if self.keyword(Keyword::Null) {
            if !self.allow_qualified {
                return Err(self.error("null requires pointer type resolution"));
            }
            Expression {
                span,
                kind: ExpressionKind::Null,
            }
        } else if token.kind == Kind::Directive(Directive::BakeArguments) {
            let value = self.baked_arguments()?;
            Expression {
                span: value.span,
                kind: ExpressionKind::BakeArguments(Box::new(value)),
            }
        } else if token.kind == Kind::Directive(Directive::CompileTime) {
            if !self.allow_qualified {
                return Err(self.error("#compile_time requires typed execution phase resolution"));
            }
            self.at += 1;
            Expression {
                span,
                kind: ExpressionKind::CompileTimePredicate,
            }
        } else if token.kind == Kind::Directive(Directive::CallerLocation) {
            if !self.allow_qualified {
                return Err(self.error("caller locations require source call binding"));
            }
            self.at += 1;
            Expression {
                span,
                kind: ExpressionKind::CallerLocation,
            }
        } else if matches!(
            token.kind,
            Kind::Directive(
                Directive::Location | Directive::File | Directive::Filepath | Directive::Line
            )
        ) {
            if !self.allow_qualified {
                return Err(self.error("source directives require retained source resolution"));
            }
            self.at += 1;
            let kind = match token.kind {
                Kind::Directive(Directive::Location) => {
                    self.need(Punct::OpenParen)?;
                    if !self.take(Punct::CloseParen) {
                        return Err(
                            self.error("#location currently requires an empty argument list")
                        );
                    }
                    ExpressionKind::SourceLocation
                }
                Kind::Directive(Directive::File) => ExpressionKind::SourceFile,
                Kind::Directive(Directive::Filepath) => ExpressionKind::SourceFilepath,
                Kind::Directive(Directive::Line) => ExpressionKind::SourceLine,
                _ => unreachable!(),
            };
            Expression {
                span: Span::new(span.start, self.tokens[self.at - 1].span.end),
                kind,
            }
        } else if token.kind == Kind::Directive(Directive::Code) {
            if !self.allow_qualified {
                return Err(self.error("#code requires syntax value resolution"));
            }
            self.code_syntax()?
        } else if token.kind == Kind::Directive(Directive::Insert) {
            if !self.allow_qualified {
                return Err(self.error("#insert requires source insertion resolution"));
            }
            let directive = self.insert_directive(21)?;
            Expression {
                span: directive.span,
                kind: ExpressionKind::Insert(Box::new(directive)),
            }
        } else if token.kind == Kind::Directive(Directive::Char) {
            if !self.allow_qualified {
                return Err(self.error("character literals require byte literal type resolution"));
            }
            self.at += 1;
            if self.token().kind != Kind::String {
                return Err(self.error("#char requires a quoted byte string"));
            }
            let value = literals::character(self.text(), self.token().span)?;
            self.at += 1;
            Expression {
                span: Span::new(span.start, self.tokens[self.at - 1].span.end),
                kind: ExpressionKind::Character(value),
            }
        } else if token.kind == Kind::HereString {
            if !self.allow_qualified {
                return Err(self.error("here-strings require byte string type resolution"));
            }
            let value = literals::here_string(self.text(), span)?;
            self.at += 1;
            Expression {
                span,
                kind: ExpressionKind::HereString(value),
            }
        } else if token.kind == Kind::String {
            if !self.allow_qualified {
                return Err(self.error("string literals require type resolution"));
            }
            let value = literals::string(self.text(), span)?;
            self.at += 1;
            Expression {
                span,
                kind: ExpressionKind::String(value),
            }
        } else if matches!(
            token.kind,
            Kind::Punctuation(Punct::OpenBracket)
                | Kind::Directive(Directive::Type | Directive::Context)
                | Kind::Keyword(
                    Keyword::Struct | Keyword::Union | Keyword::Enum | Keyword::EnumFlags
                )
        ) {
            if !self.allow_qualified {
                return Err(self.error("type expressions require type resolution"));
            }
            let ty = self.type_syntax()?;
            Expression {
                span: Span::new(span.start, self.tokens[self.at - 1].span.end),
                kind: ExpressionKind::Type(ty),
            }
        } else if let Some(query) = match token.kind {
            Kind::Keyword(Keyword::SizeOf) => Some(TypeQueryKind::SizeOf),
            Kind::Keyword(Keyword::TypeOf) => Some(TypeQueryKind::TypeOf),
            Kind::Keyword(Keyword::InitializerOf) => Some(TypeQueryKind::InitializerOf),
            Kind::Keyword(Keyword::TypeInfo) => Some(TypeQueryKind::TypeInfo),
            Kind::Keyword(Keyword::CodeOf) => Some(TypeQueryKind::CodeOf),
            _ => None,
        } {
            if !self.allow_qualified {
                return Err(self.error("type queries require type resolution"));
            }
            self.at += 1;
            self.need(Punct::OpenParen)?;
            let value = Box::new(self.expression(0)?);
            self.need(Punct::CloseParen)?;
            Expression {
                span: Span::new(span.start, self.tokens[self.at - 1].span.end),
                kind: ExpressionKind::TypeQuery {
                    query,
                    value,
                },
            }
        } else if token.kind == Kind::Directive(Directive::Run) {
            self.compile_time()?
        } else if self.keyword(Keyword::Ifx) {
            self.conditional_expression(span)?
        } else if self.keyword(Keyword::AutoCast) {
            if !self.allow_qualified {
                return Err(Diagnostic::new(
                    span,
                    "contextual casts require type resolution",
                ));
            }
            let mode = self.cast_modifiers()?.mode();
            let value = self.expression(21)?;
            Expression {
                span: Span::new(span.start, value.span.end),
                kind: ExpressionKind::InferredCast {
                    mode,
                    value: Box::new(value),
                },
            }
        } else if self.keyword(Keyword::Cast) {
            self.cast_expression(span)?
        } else if let Some(op) = unary {
            self.at += 1;
            let rhs = self.expression(21)?;
            Expression {
                span: Span::new(span.start, rhs.span.end),
                kind: ExpressionKind::Unary(op, Box::new(rhs)),
            }
        } else if token.kind == Kind::Number {
            let kind = if self.allow_qualified {
                literals::number(self.text(), span)?
            } else {
                ExpressionKind::Integer(i128::from(
                    numeric_literals::integer(self.text())
                        .map_err(|error| Diagnostic::new(span, error.to_string()))?,
                ))
            };
            self.at += 1;
            Expression {
                span,
                kind,
            }
        } else if self.keyword(Keyword::True) {
            Expression {
                span,
                kind: ExpressionKind::Bool(true),
            }
        } else if self.keyword(Keyword::False) {
            Expression {
                span,
                kind: ExpressionKind::Bool(false),
            }
        } else if token.kind == Kind::Keyword(Keyword::IsConstant) {
            self.at += 1;
            Expression {
                span,
                kind: ExpressionKind::Name(self.symbols.intern(&token.spelling(self.source))),
            }
        } else if token.kind == Kind::Ident {
            Expression {
                span,
                kind: ExpressionKind::Name(self.name()?),
            }
        } else {
            return Err(self.error("expected expression; this syntax is not implemented yet"));
        };
        loop {
            if self.is(Punct::PostfixDeref) && minimum <= 23 {
                if !self.allow_qualified {
                    return Err(self.error("dereference requires pointer and storage resolution"));
                }
                self.at += 1;
                lhs = Expression {
                    span: Span::new(lhs.span.start, self.tokens[self.at - 1].span.end),
                    kind: ExpressionKind::Dereference(Box::new(lhs)),
                };
                continue;
            }
            if self.is(Punct::OpenBracket) && minimum <= 23 {
                if !self.allow_qualified {
                    return Err(self.error("indexing requires array and storage resolution"));
                }
                self.at += 1;
                let index = Box::new(self.expression(0)?);
                self.need(Punct::CloseBracket)?;
                lhs = Expression {
                    span: Span::new(lhs.span.start, self.tokens[self.at - 1].span.end),
                    kind: ExpressionKind::Index {
                        base: Box::new(lhs),
                        index,
                    },
                };
                continue;
            }
            if self.is(Punct::ArrayLiteral) && minimum <= 23 {
                if !self.allow_qualified {
                    return Err(self.error("array literals require type resolution"));
                }
                let start = lhs.span.start;
                let element_type = self.array_literal_type_target(lhs)?;
                lhs = self.array_literal(Some(element_type), start)?;
                continue;
            }
            if self.is(Punct::StructLiteral) && minimum <= 23 {
                if !self.allow_qualified {
                    return Err(self.error("struct literals require aggregate type resolution"));
                }
                let start = lhs.span.start;
                let ty = self.literal_type_target(lhs)?;
                lhs = self.struct_literal(Some(ty), start)?;
                continue;
            }
            if self.is(Punct::Dot) && minimum <= 23 {
                if self
                    .tokens
                    .get(self.at + 1)
                    .is_some_and(|token| token.kind == Kind::Punctuation(Punct::OpenParen))
                {
                    self.at += 1;
                    lhs = self.postfix_cast(lhs)?;
                    continue;
                }
                if !self.allow_qualified {
                    return Err(
                        self.error("qualified module references require module scope resolution")
                    );
                }
                self.at += 1;
                let member = self.name()?;
                let start = lhs.span.start;
                let kind = match lhs.kind {
                    ExpressionKind::Name(root) => ExpressionKind::QualifiedName(NamePath {
                        root,
                        members: vec![member],
                    }),
                    ExpressionKind::QualifiedName(mut path) => {
                        path.members.push(member);
                        ExpressionKind::QualifiedName(path)
                    }
                    _ => ExpressionKind::Member {
                        base: Box::new(lhs),
                        member,
                    },
                };
                lhs = Expression {
                    span: Span::new(start, self.tokens[self.at - 1].span.end),
                    kind,
                };
                continue;
            }
            if self.is(Punct::OpenParen) && minimum <= 23 {
                let start = lhs.span.start;
                if !self.allow_qualified && !matches!(lhs.kind, ExpressionKind::Name(_)) {
                    return Err(
                        self.error("indirect procedure calls require procedure type resolution")
                    );
                }
                let arguments = self.parsed_call_arguments()?;
                let end = self.tokens[self.at - 1].span.end;
                let kind = if !arguments.overrides.is_empty() {
                    ExpressionKind::ContextCall {
                        callee: Box::new(lhs),
                        args: arguments.arguments,
                        overrides: arguments.overrides,
                    }
                } else {
                    match lhs {
                        Expression {
                            kind: ExpressionKind::Name(name),
                            ..
                        } => ExpressionKind::Call(name, arguments.arguments),
                        Expression {
                            kind: ExpressionKind::QualifiedName(path),
                            ..
                        } => ExpressionKind::QualifiedCall(path, arguments.arguments),
                        callee => ExpressionKind::IndirectCall {
                            callee: Box::new(callee),
                            args: arguments.arguments,
                        },
                    }
                };
                lhs = Expression {
                    span: Span::new(start, end),
                    kind,
                };
                if stop_after_call {
                    break;
                }
                continue;
            }
            let Kind::Punctuation(punctuation) = self.token().kind else {
                break;
            };
            let Some((op, precedence)) = BinaryOp::parse(punctuation) else {
                break;
            };
            if precedence < minimum {
                break;
            }
            self.at += 1;
            if self.is(Punct::OpenBrace) && matches!(op, BinaryOp::Equal | BinaryOp::NotEqual) {
                break;
            }
            let rhs = self.expression(precedence + 1)?;
            lhs = Expression {
                span: Span::new(lhs.span.start, rhs.span.end),
                kind: ExpressionKind::Binary(op, Box::new(lhs), Box::new(rhs)),
            };
        }
        Ok(lhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_source::SourceMap;

    #[test]
    fn contextual_casts_preserve_checked_default_unchecked_modifier_and_postfix_precedence() {
        let text = "Options :: struct { text_output_flags: enum_flags u32 { LINK :: 1; TIMING :: 2; } = xx 3; } main :: () { small: u8 = xx value + 1; bits: u8 = xx,no_check (value + 256); pointer: *void = xx *records[0]; }";
        let parsed = file(text);
        let FileItem::Declaration(FileDeclaration {
            kind: FileDeclarationKind::Record(record),
            ..
        }) = &parsed.items()[0]
        else {
            panic!()
        };
        let FieldBinding::Explicit {
            initializer: Some(initializer),
            ..
        } = &record.fields().next().unwrap().binding
        else {
            panic!()
        };
        assert!(matches!(
            initializer.kind,
            ExpressionKind::InferredCast {
                mode: CastMode::Checked,
                ..
            }
        ));
        let FileItem::Declaration(FileDeclaration {
            kind: FileDeclarationKind::Procedure(procedure),
            ..
        }) = &parsed.items()[1]
        else {
            panic!()
        };
        let StatementKind::Declare(Declaration::Explicit {
            initializer: Some(value),
            ..
        }) = &procedure.body[0].kind
        else {
            panic!()
        };
        assert!(
            matches!(&value.kind, ExpressionKind::Binary(BinaryOp::Add, left, _) if matches!(left.kind, ExpressionKind::InferredCast { mode: CastMode::Checked, .. }))
        );
        let StatementKind::Declare(Declaration::Explicit {
            initializer: Some(value),
            ..
        }) = &procedure.body[1].kind
        else {
            panic!()
        };
        assert!(
            matches!(&value.kind, ExpressionKind::InferredCast { mode: CastMode::Unchecked, value } if matches!(value.kind, ExpressionKind::Binary(BinaryOp::Add, _, _)))
        );
        let StatementKind::Declare(Declaration::UnresolvedExplicit {
            initializer: Some(value),
            ..
        }) = &procedure.body[2].kind
        else {
            panic!()
        };
        assert!(
            matches!(&value.kind, ExpressionKind::InferredCast { value, .. } if matches!(&value.kind, ExpressionKind::AddressOf(value) if matches!(value.kind, ExpressionKind::Index { .. })))
        );
        assert_eq!(
            parse("main :: () { value: u8 = xx 3; }")
                .unwrap_err()
                .message,
            "contextual casts require type resolution"
        );
    }

    #[test]
    fn call_hints_retain_keyword_and_inner_call_ranges_without_consuming_binary_operands() {
        let text = "main::()->int{return inline helpers.answer(20) + no_inline answer(22);}";
        let parsed = file(text);
        let FileItem::Declaration(FileDeclaration {
            kind: FileDeclarationKind::Procedure(procedure),
            ..
        }) = &parsed.items()[0]
        else {
            panic!()
        };
        let StatementKind::Return(Some(expression)) = &procedure.body[0].kind else {
            panic!()
        };
        let ExpressionKind::Binary(BinaryOp::Add, left, right) = &expression.kind else {
            panic!()
        };
        for (expression, hint, outer, inner) in [
            (
                left.as_ref(),
                jai_types::InlineHint::Always,
                "inline helpers.answer(20)",
                "helpers.answer(20)",
            ),
            (
                right.as_ref(),
                jai_types::InlineHint::Never,
                "no_inline answer(22)",
                "answer(22)",
            ),
        ] {
            let ExpressionKind::CallHint {
                hint: actual,
                call,
            } = &expression.kind
            else {
                panic!()
            };
            assert_eq!(*actual, hint);
            assert_eq!(expression.span.text(text), outer);
            assert_eq!(call.span.text(text), inner);
        }
        for text in [
            "main::(){inline 2;}",
            "main::(){inline no_inline answer();}",
        ] {
            let mut sources = SourceMap::default();
            let id = sources.insert("bad-hint.jai".into(), text.into());
            assert!(parse_file(sources.get(id).unwrap(), &mut Symbols::default()).is_err());
        }
    }
    #[test]
    fn hinted_call_result_postfixes_remain_outside_the_hinted_call() {
        let text = "main::()->int{return inline make().value;}";
        let parsed = file(text);
        let FileItem::Declaration(FileDeclaration {
            kind: FileDeclarationKind::Procedure(procedure),
            ..
        }) = &parsed.items()[0]
        else {
            panic!()
        };
        let StatementKind::Return(Some(expression)) = &procedure.body[0].kind else {
            panic!()
        };
        let ExpressionKind::Member {
            base, ..
        } = &expression.kind
        else {
            panic!()
        };
        let ExpressionKind::CallHint {
            call, ..
        } = &base.kind
        else {
            panic!()
        };
        assert_eq!(base.span.text(text), "inline make()");
        assert_eq!(call.span.text(text), "make()");
        assert_eq!(expression.span.text(text), "inline make().value");
    }

    #[test]
    fn caller_location_defaults_retain_the_deferred_marker_and_original_range() {
        let text = "log :: (location: Source_Code_Location = #caller_location) {}";
        let parsed = file(text);
        let FileItem::Declaration(FileDeclaration {
            kind: FileDeclarationKind::Procedure(procedure),
            ..
        }) = &parsed.items()[0]
        else {
            panic!("expected procedure")
        };
        let ParameterBinding::DefaultedType {
            expression, ..
        } = &procedure.parameters[0].binding
        else {
            panic!("expected typed default")
        };
        assert!(matches!(expression.kind, ExpressionKind::CallerLocation));
        assert_eq!(expression.span.text(text), "#caller_location");
        assert_eq!(
            parse("log :: (location := #caller_location) {}")
                .unwrap_err()
                .message,
            "caller locations require source call binding"
        );
    }
    #[test]
    fn source_directives_retain_exact_markers_and_ranges() {
        for (source, directive, matches_kind) in [
            ("main :: () { value := #location(); }", "#location()", 0),
            ("main :: () { value := #file; }", "#file", 1),
            ("main :: () { value := #line; }", "#line", 2),
            ("main :: () { value := #filepath; }", "#filepath", 3),
        ] {
            let parsed = file(source);
            let FileItem::Declaration(FileDeclaration {
                kind: FileDeclarationKind::Procedure(procedure),
                ..
            }) = &parsed.items()[0]
            else {
                panic!("expected procedure")
            };
            let StatementKind::Declare(Declaration::Inferred {
                initializer: init, ..
            }) = &procedure.body[0].kind
            else {
                panic!("expected declaration")
            };
            let expected = match matches_kind {
                0 => matches!(init.kind, ExpressionKind::SourceLocation),
                1 => matches!(init.kind, ExpressionKind::SourceFile),
                2 => matches!(init.kind, ExpressionKind::SourceLine),
                3 => matches!(init.kind, ExpressionKind::SourceFilepath),
                _ => unreachable!(),
            };
            assert!(expected);
            assert_eq!(init.span.text(source), directive);
            assert!(parse(source).is_err());
        }
        for source in [
            "main :: () { value := #location; }",
            "main :: () { value := #location(1); }",
        ] {
            let mut sources = SourceMap::default();
            let id = sources.insert("source-errors.jai".into(), source.into());
            assert!(parse_file(sources.get(id).unwrap(), &mut Symbols::default()).is_err());
        }
    }
    fn file(source: &str) -> ParsedFile {
        let mut sources = SourceMap::default();
        let id = sources.insert("expressions.jai".into(), source.into());
        parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap()
    }
    #[test]
    fn typed_array_constant_is_distinct_from_an_array_type_alias() {
        let file = file(
            "Scalar :: int; Values :: [3] int; Callback :: #type (value:int)->int; numbers :: int.[1,2,3]; main :: () { bytes: [] u8 = .[1,2,3]; text := \"A\\xFF\"; pointer: *void = null; state := .READY; }",
        );
        for item in &file.items()[..3] {
            assert!(matches!(
                item,
                FileItem::Declaration(FileDeclaration {
                    kind: FileDeclarationKind::TypeAlias(_),
                    ..
                })
            ));
        }
        let FileItem::Declaration(FileDeclaration {
            kind: FileDeclarationKind::Constant(constant),
            ..
        }) = &file.items()[3]
        else {
            panic!()
        };
        assert!(
            matches!(&constant.initializer.kind, ExpressionKind::ArrayLiteral(ArrayLiteral {element_type:Some(TypeSyntax::Builtin(BuiltinType::Scalar(ScalarType::Int(IntegerType::S64)))),elements}) if elements.len()==3)
        );
        let FileItem::Declaration(FileDeclaration {
            kind: FileDeclarationKind::Procedure(procedure),
            ..
        }) = &file.items()[4]
        else {
            panic!()
        };
        assert!(matches!(
            &procedure.body[0].kind,
            StatementKind::Declare(Declaration::UnresolvedExplicit {
                initializer: Some(Expression {
                    kind: ExpressionKind::ArrayLiteral(ArrayLiteral {
                        element_type: None,
                        ..
                    }),
                    ..
                }),
                ..
            })
        ));
        assert!(
            matches!(&procedure.body[1].kind,StatementKind::Declare(Declaration::Inferred {initializer:Expression {kind:ExpressionKind::String(bytes),..},..}) if bytes==&[b'A',255])
        );
        assert!(matches!(
            &procedure.body[2].kind,
            StatementKind::Declare(Declaration::UnresolvedExplicit {
                initializer: Some(Expression {
                    kind: ExpressionKind::Null,
                    ..
                }),
                ..
            })
        ));
        assert!(matches!(
            &procedure.body[3].kind,
            StatementKind::Declare(Declaration::Inferred {
                initializer: Expression {
                    kind: ExpressionKind::InferredMember(_),
                    ..
                },
                ..
            })
        ));
    }
    #[test]
    fn pointer_query_cast_and_procedure_value_calls_preserve_precedence() {
        let file = file(
            "main :: () { pointer := *array[0].x; value := pointer.* + 2; size := size_of([3]int); converted := cast(*void) pointer; answer := table[0](42); blank:int=---; }",
        );
        let FileItem::Declaration(FileDeclaration {
            kind: FileDeclarationKind::Procedure(procedure),
            ..
        }) = &file.items()[0]
        else {
            panic!()
        };
        let StatementKind::Declare(Declaration::Inferred {
            initializer, ..
        }) = &procedure.body[0].kind
        else {
            panic!()
        };
        assert!(
            matches!(&initializer.kind, ExpressionKind::AddressOf(value) if matches!(value.kind,ExpressionKind::Member {..}))
        );
        let StatementKind::Declare(Declaration::Inferred {
            initializer, ..
        }) = &procedure.body[1].kind
        else {
            panic!()
        };
        assert!(
            matches!(&initializer.kind,ExpressionKind::Binary(BinaryOp::Add,left,_) if matches!(left.kind,ExpressionKind::Dereference(_)))
        );
        let StatementKind::Declare(Declaration::Inferred {
            initializer, ..
        }) = &procedure.body[2].kind
        else {
            panic!()
        };
        assert!(
            matches!(&initializer.kind,ExpressionKind::TypeQuery {query:TypeQueryKind::SizeOf,value} if matches!(value.kind,ExpressionKind::Type(TypeSyntax::FixedArray{..})))
        );
        let StatementKind::Declare(Declaration::Inferred {
            initializer, ..
        }) = &procedure.body[3].kind
        else {
            panic!()
        };
        assert!(matches!(initializer.kind, ExpressionKind::TypeCast { .. }));
        let StatementKind::Declare(Declaration::Inferred {
            initializer, ..
        }) = &procedure.body[4].kind
        else {
            panic!()
        };
        assert!(
            matches!(&initializer.kind,ExpressionKind::IndirectCall{callee,..} if matches!(callee.kind,ExpressionKind::Index{..}))
        );
        assert!(matches!(
            &procedure.body[5].kind,
            StatementKind::Declare(Declaration::Explicit {
                initializer: Some(Expression {
                    kind: ExpressionKind::Uninitialized,
                    ..
                }),
                ..
            })
        ));
        let mut sources = SourceMap::default();
        let id = sources.insert("bad.jai".into(), "main :: () { return ---; }".into());
        assert!(parse_file(sources.get(id).unwrap(), &mut Symbols::default()).is_err());
    }
}
