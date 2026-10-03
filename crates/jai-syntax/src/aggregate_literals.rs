//! Staged aggregate literals retain type syntax and relative source places.
//! Registration waits for the matching source AST and semantic consumers.
use super::*;

impl Parser<'_> {
    pub(super) fn literal_type_target(
        &self,
        expression: Expression,
    ) -> Result<TypeSyntax, Diagnostic> {
        let span = expression.span;
        let application = |base, arguments| {
            TypeSyntax::Application(TypeApplicationSyntax {
                base: Box::new(base),
                arguments,
                span,
            })
        };
        Ok(match expression.kind {
            ExpressionKind::Type(ty) => ty,
            ExpressionKind::Name(root) => self.literal_named_type(NamePath {
                root,
                members: Vec::new(),
            }),
            ExpressionKind::QualifiedName(path) => self.literal_named_type(path),
            ExpressionKind::Call(root, arguments) => application(
                self.literal_named_type(NamePath {
                    root,
                    members: Vec::new(),
                }),
                arguments,
            ),
            ExpressionKind::QualifiedCall(path, arguments) => {
                application(self.literal_named_type(path), arguments)
            }
            ExpressionKind::IndirectCall { callee, args } => {
                application(self.literal_type_target(*callee)?, args)
            }
            _ => {
                return Err(Diagnostic::new(
                    span,
                    "aggregate literal target requires type syntax",
                ))
            }
        })
    }

    fn literal_named_type(&self, path: NamePath) -> TypeSyntax {
        if path.members.is_empty()
            && let Some(builtin) = BuiltinType::from_spelling(self.symbols.name(path.root))
        {
            return TypeSyntax::Builtin(builtin);
        }
        TypeSyntax::Named(path)
    }

    pub(super) fn struct_literal(
        &mut self,
        ty: Option<TypeSyntax>,
        start: usize,
    ) -> Result<Expression, Diagnostic> {
        self.need(Punct::StructLiteral)?;
        if self.take(Punct::CloseBrace) {
            return Ok(Expression {
                kind: ExpressionKind::StructLiteral(StructLiteral {
                    ty,
                    fields: Vec::new(),
                }),
                span: Span::new(start, self.tokens[self.at - 1].span.end),
            });
        }
        let first = self.expression(0)?;
        if !self.is(Punct::Assign) {
            let mut values = vec![first];
            while !self.take(Punct::CloseBrace) {
                self.need(Punct::Comma)?;
                if self.take(Punct::CloseBrace) {
                    break;
                }
                values.push(self.expression(0)?);
                if self.is(Punct::Assign) {
                    return Err(
                        self.error("named and positional struct literal members cannot be mixed")
                    );
                }
            }
            return Ok(Expression {
                kind: ExpressionKind::PositionalStructLiteral(PositionalStructLiteral {
                    ty,
                    values,
                }),
                span: Span::new(start, self.tokens[self.at - 1].span.end),
            });
        }
        let mut fields = vec![self.struct_literal_field(first)?];
        while !self.take(Punct::CloseBrace) {
            self.need(Punct::Comma)?;
            if self.take(Punct::CloseBrace) {
                break;
            }
            let target = self.expression(0)?;
            if !self.is(Punct::Assign) {
                return Err(
                    self.error("named and positional struct literal members cannot be mixed")
                );
            }
            fields.push(self.struct_literal_field(target)?);
        }
        Ok(Expression {
            kind: ExpressionKind::StructLiteral(StructLiteral { ty, fields }),
            span: Span::new(start, self.tokens[self.at - 1].span.end),
        })
    }

    fn struct_literal_field(
        &mut self,
        expression: Expression,
    ) -> Result<StructLiteralField, Diagnostic> {
        let start = expression.span.start;
        let target = PlaceSyntax::try_from(expression).map_err(|error| {
            Diagnostic::new(
                error.span,
                "struct literal field requires a relative field place",
            )
        })?;
        validate_relative_field_place(&target)?;
        self.need(Punct::Assign)?;
        let value = self.expression(0)?;
        Ok(StructLiteralField {
            target,
            span: Span::new(start, value.span.end),
            value,
        })
    }
}

fn validate_relative_field_place(target: &PlaceSyntax) -> Result<(), Diagnostic> {
    let mut base = match &target.kind {
        PlaceKind::Name(_) | PlaceKind::Qualified(_) => return Ok(()),
        PlaceKind::Member { base, .. } | PlaceKind::Index { base, .. } => base.as_ref(),
        _ => {
            return Err(Diagnostic::new(
                target.span,
                "struct literal field requires a relative field place",
            ))
        }
    };
    loop {
        base = match &base.kind {
            ExpressionKind::Name(_) | ExpressionKind::QualifiedName(_) => return Ok(()),
            ExpressionKind::Member { base, .. } | ExpressionKind::Index { base, .. } => {
                base.as_ref()
            }
            _ => {
                return Err(Diagnostic::new(
                    base.span,
                    "struct literal field requires a relative field place",
                ))
            }
        };
    }
}
