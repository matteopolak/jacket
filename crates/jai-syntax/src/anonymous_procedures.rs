//! Full procedure expressions retain their actual source header and body.
use super::*;

impl Parser<'_> {
    pub(super) fn starts_anonymous_procedure(&self) -> bool {
        let mut start = self.at;
        if matches!(
            self.token().kind,
            Kind::Keyword(Keyword::Inline | Keyword::NoInline)
        ) {
            start += 1;
        }
        if self
            .tokens
            .get(start)
            .is_none_or(|token| token.kind != Kind::Punctuation(Punct::OpenParen))
        {
            return false;
        }
        let mut depth = 0;
        let mut formal = false;
        for (offset, token) in self.tokens[start..].iter().enumerate() {
            match token.kind {
                Kind::Punctuation(Punct::OpenParen) => depth += 1,
                Kind::Punctuation(Punct::CloseParen) => {
                    depth -= 1;
                    if depth == 0 {
                        if offset != 1 && !formal {
                            return false;
                        }
                        return self.tokens.get(start + offset + 1).is_some_and(|next| {
                            matches!(
                                next.kind,
                                Kind::Punctuation(Punct::Arrow | Punct::OpenBrace)
                            ) || safety_checks::is_check_directive(next.kind)
                                || matches!(
                                    next.kind,
                                    Kind::Directive(
                                        Directive::CCall
                                            | Directive::CppMethod
                                            | Directive::NoContext
                                            | Directive::NoDebug
                                            | Directive::CompileTime
                                            | Directive::Expand
                                            | Directive::Deprecated
                                            | Directive::Modify
                                    )
                                )
                        });
                    }
                }
                Kind::Punctuation(Punct::Colon | Punct::Infer) if depth == 1 => formal = true,
                Kind::Eof => return false,
                _ => {}
            }
        }
        false
    }

    pub(super) fn anonymous_procedure(&mut self) -> Result<Expression, Diagnostic> {
        let start = self.token().span.start;
        let (mut header, _, _) = self.source_procedure_header(false)?;
        if header.convention == jai_types::CallingConvention::C {
            self.validate_c_variadic(&header.parameters)?;
        }
        header.modify = self.modify_directive()?;
        let body = self.block()?;
        header.notes = self.notes()?;
        let span = Span::new(start, self.tokens[self.at - 1].span.end);
        Ok(Expression {
            span,
            kind: ExpressionKind::AnonymousProcedure(Box::new(SourceProcedureSyntax {
                header,
                body,
                span,
            })),
        })
    }
}

#[cfg(test)]
mod condition_regressions {
    use super::*;

    fn source_procedure(source: &str) -> Procedure {
        let mut sources = jai_source::SourceMap::default();
        let id = sources.insert("condition.jai".into(), source.into());
        let parsed = parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap();
        let FileItem::Declaration(declaration) = parsed.items()[0].clone() else {
            panic!("expected declaration")
        };
        let FileDeclarationKind::Procedure(procedure) = declaration.kind else {
            panic!("expected procedure")
        };
        procedure
    }

    #[test]
    fn original_allocator_condition_retains_full_expression_and_block() {
        let condition = "mode == .ALLOCATE || (mode == .RESIZE && !old_memory)";
        let source = format!(
            "probe :: () {{ if {condition} {{ pointer := c_malloc(cast(u64) requested_size); }} else {{ return; }} }}"
        );
        let procedure = source_procedure(&source);
        let StatementKind::If(value, then_body, else_body) = &procedure.body[0].kind else {
            panic!("expected ordinary conditional")
        };
        assert_eq!(value.span.text(&source), condition);
        assert!(matches!(value.kind, ExpressionKind::Binary(..)));
        assert_eq!(then_body.len(), 1);
        assert_eq!(else_body.len(), 1);
        assert!(matches!(else_body[0].kind, StatementKind::Return(None)));
    }

    #[test]
    fn original_focus_range_parentheses_retain_bounds_and_body() {
        let source = "probe :: () { for 0..(1 << loop_count) { continue; } }";
        let procedure = source_procedure(source);
        let StatementKind::Range(range) = &procedure.body[0].kind else {
            panic!("expected range")
        };
        assert_eq!(range.start.span.text(source), "0");
        assert_eq!(range.end.span.text(source), "(1 << loop_count)");
        assert_eq!(range.body.len(), 1);
    }

    #[test]
    fn genuine_empty_typed_and_inferred_callable_headers_remain_callable() {
        for source in [
            "probe :: () { f := () { return; }; }",
            "probe :: () { f := (value: int) -> int { return value; }; }",
            "probe :: () { f := (value := 1) -> int { return value; }; }",
        ] {
            let procedure = source_procedure(source);
            let StatementKind::Declare(Declaration::Inferred { initializer, .. }) =
                &procedure.body[0].kind
            else {
                panic!("expected inferred local")
            };
            assert!(matches!(
                initializer.kind,
                ExpressionKind::AnonymousProcedure(_)
            ));
        }
    }

    #[test]
    fn sgpu_pointer_array_target_is_actual_pointer_type_syntax() {
        let source = "probe :: () { names := (*u8).[\"VK_LAYER_KHRONOS_validation\"].data; }";
        let procedure = source_procedure(source);
        let StatementKind::Declare(Declaration::Inferred { initializer, .. }) =
            &procedure.body[0].kind
        else {
            panic!("expected local")
        };
        let ExpressionKind::Member { base, .. } = &initializer.kind else {
            panic!("expected original data projection")
        };
        let ExpressionKind::ArrayLiteral(ArrayLiteral {
            element_type: Some(TypeSyntax::Pointer(element)),
            elements,
        }) = &base.kind
        else {
            panic!("expected pointer array target")
        };
        assert!(matches!(
            element.as_ref(),
            TypeSyntax::Builtin(BuiltinType::Scalar(ScalarType::Int(IntegerType::U8)))
        ));
        assert_eq!(
            base.span.text(source),
            "(*u8).[\"VK_LAYER_KHRONOS_validation\"]"
        );
        assert_eq!(elements.len(), 1);
    }

    #[test]
    fn array_type_queries_and_applications_keep_original_target_children() {
        for (source, query) in [
            ("probe :: () { values := type_of(original).[item]; }", true),
            ("probe :: () { values := Box(element_type).[item]; }", false),
        ] {
            let procedure = source_procedure(source);
            let StatementKind::Declare(Declaration::Inferred { initializer, .. }) =
                &procedure.body[0].kind
            else {
                panic!("expected local")
            };
            let ExpressionKind::ArrayLiteral(ArrayLiteral {
                element_type: Some(target),
                elements,
            }) = &initializer.kind
            else {
                panic!("expected typed array")
            };
            if query {
                let TypeSyntax::TypeOf(value) = target else {
                    panic!("expected original type query")
                };
                assert_eq!(value.span.text(source), "original");
            } else {
                let TypeSyntax::Application(application) = target else {
                    panic!("expected application target")
                };
                assert_eq!(application.span.text(source), "Box(element_type)");
                assert_eq!(application.arguments.len(), 1);
            }
            assert_eq!(elements.len(), 1);
        }
    }
}
