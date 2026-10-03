//! Prefix and function-style casts share the same unresolved target and operand.
use super::*;
use jai_types::{CastModifier, CastModifiers, StorageBitcastStrength};

impl Parser<'_> {
    pub(super) fn postfix_cast(&mut self, value: Expression) -> Result<Expression, Diagnostic> {
        self.need(Punct::OpenParen)?;
        let ty = if self.allow_qualified {
            self.type_syntax()?
        } else {
            TypeSyntax::Builtin(BuiltinType::Scalar(self.scalar_type()?))
        };
        let mode = self.cast_modifiers()?.mode();
        self.need(Punct::CloseParen)?;
        let span = Span::new(value.span.start, self.tokens[self.at - 1].span.end);
        Ok(Expression {
            span,
            kind: if let Some(ty) = ty.as_scalar() {
                ExpressionKind::Cast(mode, ty, Box::new(value))
            } else {
                ExpressionKind::TypeCast {
                    mode,
                    ty,
                    value: Box::new(value),
                }
            },
        })
    }

    pub(super) fn cast_expression(&mut self, start: Span) -> Result<Expression, Diagnostic> {
        let mut modifiers = self.cast_modifiers()?;
        self.need(Punct::OpenParen)?;
        let ty = if self.allow_qualified {
            self.type_syntax()?
        } else {
            TypeSyntax::Builtin(BuiltinType::Scalar(self.scalar_type()?))
        };
        let (value, end, dereference) = if self.take(Punct::Comma) {
            let value = self.expression(0)?;
            while self.take(Punct::Comma) {
                self.cast_modifier(&mut modifiers)?;
            }
            self.need(Punct::CloseParen)?;
            (value, self.tokens[self.at - 1].span.end, false)
        } else {
            self.need(Punct::CloseParen)?;
            let dereference = self.take(Punct::PostfixDeref);
            let value = self.expression(21)?;
            let end = value.span.end;
            (value, end, dereference)
        };
        let mode = modifiers.mode();
        let cast = Expression {
            span: Span::new(start.start, end),
            kind: if let Some(ty) = ty.as_scalar() {
                ExpressionKind::Cast(mode, ty, Box::new(value))
            } else {
                ExpressionKind::TypeCast {
                    mode,
                    ty,
                    value: Box::new(value),
                }
            },
        };
        Ok(if dereference {
            Expression {
                span: cast.span,
                kind: ExpressionKind::Dereference(Box::new(cast)),
            }
        } else {
            cast
        })
    }

    pub(super) fn cast_modifiers(&mut self) -> Result<CastModifiers, Diagnostic> {
        let mut modifiers = CastModifiers::checked();
        while self.take(Punct::Comma) {
            self.cast_modifier(&mut modifiers)?;
        }
        Ok(modifiers)
    }

    fn cast_modifier(&mut self, modifiers: &mut CastModifiers) -> Result<(), Diagnostic> {
        let span = self.token().span;
        let modifier = match self.token().spelling(self.source).as_ref() {
            "no_check" if self.token().kind == Kind::Ident => CastModifier::NoBoundsCheck,
            "trunc" if self.token().kind == Kind::Ident => CastModifier::Truncate,
            "force" if self.token().kind == Kind::Ident => {
                CastModifier::Force(StorageBitcastStrength::EqualSize)
            }
            "FORCE" if self.token().kind == Kind::Ident => {
                CastModifier::Force(StorageBitcastStrength::Prefix)
            }
            _ => return Err(self.error("expected no_check, trunc, force, or FORCE cast modifier")),
        };
        self.at += 1;
        modifiers
            .insert(modifier)
            .map_err(|error| Diagnostic::new(span, error.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn force_case_and_all_cast_spellings_keep_storage_strength() {
        for (spelling, strength) in [
            ("force", StorageBitcastStrength::EqualSize),
            ("FORCE", StorageBitcastStrength::Prefix),
        ] {
            for expression in [
                format!("cast,{spelling}(u32)42"),
                format!("cast(u32,42,{spelling})"),
                format!("(42).(u32,{spelling})"),
            ] {
                let text = format!("main::()->int{{return {expression};}}");
                let module = parse(&text).unwrap();
                assert!(
                    matches!(
                        &module.procedures[0].body[0].kind,
                        StatementKind::Return(Some(Expression {
                            kind: ExpressionKind::Cast(CastMode::Force(actual), _, _), ..
                        })) if *actual == strength
                    ),
                    "{text}"
                );
            }
            let mut sources = jai_source::SourceMap::default();
            let text = format!("main::(){{x:u32=xx,{spelling} 42;}}");
            let id = sources.insert("contextual-force.jai".into(), text);
            let parsed = parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap();
            let FileItem::Declaration(declaration) = &parsed.items()[0] else {
                panic!()
            };
            let FileDeclarationKind::Procedure(procedure) = &declaration.kind else {
                panic!()
            };
            let StatementKind::Declare(
                Declaration::Explicit {
                    initializer: Some(initializer),
                    ..
                }
                | Declaration::UnresolvedExplicit {
                    initializer: Some(initializer),
                    ..
                },
            ) = &procedure.body[0].kind
            else {
                panic!()
            };
            assert!(
                matches!(&initializer.kind, ExpressionKind::InferredCast { mode: CastMode::Force(actual), .. } if *actual == strength)
            );
        }
        for text in [
            "main::(){x:=cast,force,force(u32)42;}",
            "main::(){x:=cast,force,FORCE(u32)42;}",
            "main::(){x:=cast,force,no_check(u32)42;}",
            "main::(){x:=cast,FORCE,trunc(u32)42;}",
            "main::(){x:=cast,Force(u32)42;}",
        ] {
            assert!(parse(text).is_err(), "{text}");
        }
    }

    #[test]
    fn nested_sgpu_comma_casts_keep_target_operand_and_group_boundaries() {
        let text = "main :: () { pointer := cast(*u32, cast(*u8, this) + 16); value := cast(u8, 257) + 1; }";
        let mut sources = jai_source::SourceMap::default();
        let id = sources.insert("comma-casts.jai".into(), text.into());
        let parsed = parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap();
        let FileItem::Declaration(declaration) = &parsed.items()[0] else {
            panic!()
        };
        let FileDeclarationKind::Procedure(procedure) = &declaration.kind else {
            panic!()
        };
        let StatementKind::Declare(Declaration::Inferred {
            initializer, ..
        }) = &procedure.body[0].kind
        else {
            panic!()
        };
        assert_eq!(
            initializer.span.text(text),
            "cast(*u32, cast(*u8, this) + 16)"
        );
        assert!(
            matches!(&initializer.kind, ExpressionKind::TypeCast {mode:CastMode::Checked,ty:TypeSyntax::Pointer(_),value} if matches!(&value.kind,ExpressionKind::Binary(BinaryOp::Add,left,_) if matches!(left.kind,ExpressionKind::TypeCast {..})))
        );
        let StatementKind::Declare(Declaration::Inferred {
            initializer, ..
        }) = &procedure.body[1].kind
        else {
            panic!()
        };
        assert!(
            matches!(&initializer.kind,ExpressionKind::Binary(BinaryOp::Add,left,_) if left.span.text(text)=="cast(u8, 257)")
        );
    }

    #[test]
    fn scalar_bridge_and_unchecked_modifier_use_the_same_cast_node() {
        let module = parse("main :: () -> int { return cast,no_check(u8, 257); }").unwrap();
        assert!(
            matches!(&module.procedures[0].body[0].kind, StatementKind::Return(Some(Expression {kind:ExpressionKind::Cast(CastMode::Unchecked,_,value),..})) if matches!(value.kind,ExpressionKind::Integer(257)))
        );
    }

    #[test]
    fn missing_function_cast_operands_and_extra_arguments_are_rejected() {
        for text in [
            "main::(){x:=cast(int,);}",
            "main::(){x:=cast(int,1,2);}",
            "main::(){x:=cast(int,1;}",
        ] {
            assert!(parse(text).is_err());
        }
    }

    #[test]
    fn trunc_prefix_function_and_contextual_forms_keep_the_distinct_policy() {
        let module = parse("main::()->int{return cast,trunc(u8)256;}").unwrap();
        assert!(matches!(
            &module.procedures[0].body[0].kind,
            StatementKind::Return(Some(Expression {
                kind: ExpressionKind::Cast(CastMode::Truncate, _, _),
                ..
            }))
        ));
        let module = parse("main::()->int{return cast(u8,256,trunc);}").unwrap();
        assert!(matches!(
            &module.procedures[0].body[0].kind,
            StatementKind::Return(Some(Expression {
                kind: ExpressionKind::Cast(CastMode::Truncate, _, _),
                ..
            }))
        ));
        let mut sources = jai_source::SourceMap::default();
        let id = sources.insert(
            "contextual-trunc.jai".into(),
            "main::(){x:u8=xx,trunc 256;}".into(),
        );
        assert!(parse_file(sources.get(id).unwrap(), &mut Symbols::default()).is_ok());
        let id = sources.insert(
            "cast-dereference.jai".into(),
            "main::(){x:=cast(*u32).* (p+16);}".into(),
        );
        let parsed = parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap();
        let FileItem::Declaration(declaration) = &parsed.items()[0] else {
            panic!()
        };
        let FileDeclarationKind::Procedure(procedure) = &declaration.kind else {
            panic!()
        };
        let StatementKind::Declare(Declaration::Inferred {
            initializer, ..
        }) = &procedure.body[0].kind
        else {
            panic!()
        };
        assert!(
            matches!(&initializer.kind, ExpressionKind::Dereference(value) if matches!(value.kind,ExpressionKind::TypeCast{..}))
        );
    }

    #[test]
    fn repeated_conflicting_and_unknown_modifiers_have_source_diagnostics() {
        for text in [
            "main::(){x:=cast,trunc,trunc(u8)256;}",
            "main::(){x:=cast,trunc,no_check(u8)256;}",
            "main::(){x:=cast,no_check(u8,256,trunc);}",
            "main::(){x:=cast(u8,256,trunc,trunc);}",
            "main::(){x:=cast,lossy(u8)256;}",
        ] {
            assert!(parse(text).is_err(), "{text}");
        }
    }

    #[test]
    fn postfix_casts_preserve_checked_targets_and_continuation_precedence() {
        let text = "Gpu_Queue::enum u8{ONE::1;} main::(){queue:=(it_index+1).(Gpu_Queue); value:=(pointer+16).(*u32,trunc).*+1;}";
        let mut sources = jai_source::SourceMap::default();
        let id = sources.insert("postfix-casts.jai".into(), text.into());
        let parsed = parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap();
        let FileItem::Declaration(declaration) = &parsed.items()[1] else {
            panic!()
        };
        let FileDeclarationKind::Procedure(procedure) = &declaration.kind else {
            panic!()
        };
        let StatementKind::Declare(Declaration::Inferred {
            initializer, ..
        }) = &procedure.body[0].kind
        else {
            panic!()
        };
        assert!(
            matches!(&initializer.kind,ExpressionKind::TypeCast{mode:CastMode::Checked,ty:TypeSyntax::Named(_),value} if matches!(value.kind,ExpressionKind::Binary(BinaryOp::Add,_,_)))
        );
        let StatementKind::Declare(Declaration::Inferred {
            initializer, ..
        }) = &procedure.body[1].kind
        else {
            panic!()
        };
        assert!(
            matches!(&initializer.kind,ExpressionKind::Binary(BinaryOp::Add,left,_) if matches!(&left.kind,ExpressionKind::Dereference(cast) if matches!(cast.kind,ExpressionKind::TypeCast{mode:CastMode::Truncate,..})))
        );
        for text in [
            "main::(){x:=(1).(u8,trunc,no_check);}",
            "main::(){x:=(1).(u8,lossy);}",
        ] {
            let id = sources.insert("invalid-postfix-casts.jai".into(), text.into());
            assert!(parse_file(sources.get(id).unwrap(), &mut Symbols::default()).is_err());
        }
    }
}
