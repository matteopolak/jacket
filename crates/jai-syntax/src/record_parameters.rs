//! Source parameters and applications for nominal type specialization.
use super::*;

#[derive(Clone, Debug)]
pub struct RecordParameter {
    pub name: Symbol,
    pub binding: RecordParameterBinding,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum RecordParameterBinding {
    Typed {
        ty: TypeSyntax,
        default: Option<Expression>,
    },
    InferredDefault(Expression),
}
#[derive(Clone, Debug)]
pub struct TypeApplicationSyntax {
    pub base: Box<TypeSyntax>,
    pub arguments: Vec<CallArgument>,
    pub span: Span,
}

impl Parser<'_> {
    pub(super) fn record_parameters(&mut self) -> Result<Vec<RecordParameter>, Diagnostic> {
        if !self.take(Punct::OpenParen) {
            return Ok(Vec::new());
        }
        let mut parameters = Vec::new();
        if self.take(Punct::CloseParen) {
            return Ok(parameters);
        }
        loop {
            let start = self.token().span.start;
            // Record parameters are baked with or without the optional dollar marker.
            self.take(Punct::Dollar);
            let name = self.name()?;
            let binding = if self.take(Punct::Infer) {
                RecordParameterBinding::InferredDefault(self.expression(0)?)
            } else {
                self.need(Punct::Colon)?;
                let ty = self.parameter_type_syntax()?;
                let default = if self.take(Punct::Assign) {
                    Some(self.expression(0)?)
                } else {
                    None
                };
                RecordParameterBinding::Typed { ty, default }
            };
            parameters.push(RecordParameter {
                name,
                binding,
                span: Span::new(start, self.tokens[self.at - 1].span.end),
            });
            if self.take(Punct::CloseParen) {
                break;
            }
            self.need(Punct::Comma)?;
            if self.take(Punct::CloseParen) {
                break;
            }
        }
        Ok(parameters)
    }

    pub(super) fn type_application(
        &mut self,
        base: TypeSyntax,
        start: usize,
    ) -> Result<TypeApplicationSyntax, Diagnostic> {
        self.need(Punct::OpenParen)?;
        let mut arguments = Vec::new();
        if !self.take(Punct::CloseParen) {
            loop {
                let name = if self.named_prefix(Punct::Assign) {
                    let name = self.name()?;
                    self.need(Punct::Assign)?;
                    Some(name)
                } else {
                    None
                };
                arguments.push(CallArgument {
                    name,
                    value: self.expression(0)?,
                    spread: false,
                });
                if self.take(Punct::CloseParen) {
                    break;
                }
                self.need(Punct::Comma)?;
                if self.take(Punct::CloseParen) {
                    break;
                }
            }
        }
        Ok(TypeApplicationSyntax {
            base: Box::new(base),
            arguments,
            span: Span::new(start, self.tokens[self.at - 1].span.end),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_source::SourceMap;

    fn source(text: &str, symbols: &mut Symbols) -> ParsedFile {
        let mut sources = SourceMap::default();
        let id = sources.insert("record-parameters.jai".into(), text.into());
        parse_file(sources.get(id).unwrap(), symbols).unwrap()
    }

    #[test]
    fn record_parameters_and_type_applications_preserve_order_and_bindings() {
        let mut symbols = Symbols::default();
        let file = source(
            "Holder :: struct ($T: Type, N: int = 4, Enabled := true) { values: [N] T; } value: Holder(u8, N=8, Enabled=false);",
            &mut symbols,
        );
        let FileItem::Declaration(FileDeclaration {
            kind: FileDeclarationKind::Record(record),
            ..
        }) = &file.items()[0]
        else {
            panic!("expected record")
        };
        assert_eq!(
            record
                .parameters
                .iter()
                .map(|parameter| symbols.name(parameter.name))
                .collect::<Vec<_>>(),
            ["T", "N", "Enabled"]
        );
        assert!(matches!(
            record.parameters[0].binding,
            RecordParameterBinding::Typed {
                ty: TypeSyntax::Builtin(BuiltinType::Type),
                default: None
            }
        ));
        assert!(matches!(
            record.parameters[1].binding,
            RecordParameterBinding::Typed {
                default: Some(Expression {
                    kind: ExpressionKind::Integer(4),
                    ..
                }),
                ..
            }
        ));
        assert!(matches!(
            record.parameters[2].binding,
            RecordParameterBinding::InferredDefault(Expression {
                kind: ExpressionKind::Bool(true),
                ..
            })
        ));
        let FileItem::Declaration(FileDeclaration {
            kind:
                FileDeclarationKind::Global(GlobalDeclaration {
                    declaration:
                        Declaration::UnresolvedExplicit {
                            ty: TypeSyntax::Application(application),
                            ..
                        },
                    ..
                }),
            ..
        }) = &file.items()[1]
        else {
            panic!("expected application")
        };
        assert_eq!(application.arguments.len(), 3);
        assert!(application.arguments[0].name.is_none());
        assert_eq!(symbols.name(application.arguments[1].name.unwrap()), "N");
        assert_eq!(
            symbols.name(application.arguments[2].name.unwrap()),
            "Enabled"
        );
    }

    #[test]
    fn pinned_queue_header_and_nested_application_signature_parse() {
        let file = source(
            "Queue :: struct(User_Data: Type) { completion_port: HANDLE; } open_file :: (queue: *Queue($T)) -> Queue(T), result := Error.{} { return queue.*, result; }",
            &mut Symbols::default(),
        );
        let FileItem::Declaration(FileDeclaration {
            kind: FileDeclarationKind::Procedure(procedure),
            ..
        }) = &file.items()[1]
        else {
            panic!("expected procedure")
        };
        let ParameterBinding::RequiredType(TypeSyntax::Pointer(inner)) =
            &procedure.parameters[0].binding
        else {
            panic!("expected pointer")
        };
        let TypeSyntax::Application(application) = inner.as_ref() else {
            panic!("expected application")
        };
        assert!(matches!(
            application.arguments[0].value.kind,
            ExpressionKind::CompileVariable(_)
        ));
        assert_eq!(procedure.results.len(), 2);
        assert!(matches!(
            procedure.results[0].binding,
            ResultBinding::Typed {
                ty: TypeSyntax::Application(_),
                ..
            }
        ));
    }
}
