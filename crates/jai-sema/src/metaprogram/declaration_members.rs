//! Original quoted statements become file items without parsing generated text.
use super::*;
use syntax::{FileDeclaration, FileDeclarationKind as D, FileItem, StatementKind as S, Visibility};

const MAX_ITEMS: usize = 65_536;
const MAX_DEPTH: usize = 128;

pub(crate) fn literal_file_items(
    body: &syntax::CodeBody,
    source: jai_source::SourceId,
    visibility: Visibility,
) -> Result<Vec<FileItem>, Diagnostic> {
    match body {
        syntax::CodeBody::Block(body) => convert(body, source, visibility, 0, &mut 0),
        syntax::CodeBody::Statement(statement) => convert(
            std::slice::from_ref(statement),
            source,
            visibility,
            0,
            &mut 0,
        ),
        syntax::CodeBody::Expression(expression) => Err(Diagnostic::new(
            expression.span,
            "file insertion requires quoted declaration statements or a declaration block",
        )),
        syntax::CodeBody::Null => Err(Diagnostic::new(
            Span::default(),
            "#code,null has no declarations to insert",
        )),
    }
}

fn declaration(
    statement: &syntax::Statement,
    source: jai_source::SourceId,
    visibility: Visibility,
) -> Result<FileDeclaration, Diagnostic> {
    let kind = match &statement.kind {
        S::Declare(declaration) => D::Global(syntax::GlobalDeclaration {
            declaration: declaration.clone(),
            span: statement.span,
        }),
        S::Constant(constant) => D::Constant(constant.clone()),
        S::Library(library) => D::Library(library.clone()),
        S::Procedure(procedure) => D::Procedure((**procedure).clone()),
        S::ProcedurePrototype(prototype) => D::ProcedurePrototype(prototype.clone()),
        S::Record(record) => D::Record(record.clone()),
        S::Enum(enumeration) => D::Enum(enumeration.clone()),
        S::TypeAlias(alias) => D::TypeAlias(alias.clone()),
        _ => {
            return Err(Diagnostic::new(
                statement.span,
                "file insertion requires an original declaration; runtime statements cannot become file declarations",
            ));
        }
    };
    Ok(FileDeclaration {
        program_export: None,
        visibility,
        kind,
        location: SourceSpan {
            source,
            span: statement.span,
        },
    })
}

fn convert(
    statements: &[syntax::Statement],
    source: jai_source::SourceId,
    visibility: Visibility,
    depth: usize,
    visited: &mut usize,
) -> Result<Vec<FileItem>, Diagnostic> {
    if depth > MAX_DEPTH {
        return Err(Diagnostic::new(
            statements.first().map_or(Span::default(), |s| s.span),
            "declaration insertion exceeds its syntax depth limit",
        ));
    }
    let mut result = Vec::new();
    for statement in statements {
        *visited = visited.saturating_add(1);
        if *visited > MAX_ITEMS {
            return Err(Diagnostic::new(
                statement.span,
                "declaration insertion exceeds its item limit",
            ));
        }
        let location = SourceSpan {
            source,
            span: statement.span,
        };
        let item = match &statement.kind {
            S::UsingDeclaration {
                declaration: child,
                selection,
                target_span,
            } => FileItem::UsingDeclaration {
                declaration: declaration(child, source, visibility)?,
                selection: selection.clone(),
                target_span: *target_span,
                location,
            },
            S::Using(directive) => FileItem::Using {
                directive: directive.clone(),
                visibility,
                location,
            },
            S::Import(import) => FileItem::Import(syntax::ImportDeclaration {
                namespace: import.namespace,
                using: import.using,
                mode: import.mode,
                target: import.target.clone(),
                arguments: import.arguments.clone(),
                visibility,
                location,
            }),
            S::ContextField(field) => FileItem::ContextField {
                declaration: field.clone(),
                location,
            },
            S::Insert(directive) => FileItem::Insert {
                directive: directive.clone(),
                location,
            },
            S::CompileTimeAssert { condition, message } => FileItem::Assert {
                condition: condition.clone(),
                message: message.clone(),
                location,
            },
            S::CompileTimeIf {
                condition,
                then_body,
                else_body,
            } => FileItem::Conditional {
                condition: condition.clone(),
                then_items: convert(then_body, source, visibility, depth + 1, visited)?,
                else_items: convert(else_body, source, visibility, depth + 1, visited)?,
                location,
            },
            S::CompileTimeCases(cases) => {
                let mut arms = Vec::new();
                for arm in &cases.arms {
                    arms.push(syntax::CompileTimeCaseArm {
                        label: arm.label.clone(),
                        falls_through: arm.falls_through,
                        span: arm.span,
                        body: convert(&arm.body, source, visibility, depth + 1, visited)?,
                    });
                }
                let default = cases
                    .default
                    .as_ref()
                    .map(|default| {
                        Ok::<_, Diagnostic>(syntax::CompileTimeCaseDefault {
                            span: default.span,
                            body: convert(&default.body, source, visibility, depth + 1, visited)?,
                        })
                    })
                    .transpose()?;
                FileItem::CompileTimeCases {
                    cases: syntax::CompileTimeCases {
                        value: cases.value.clone(),
                        operator: cases.operator,
                        arms,
                        default,
                        complete: cases.complete,
                        span: cases.span,
                    },
                    location,
                }
            }
            S::Expression(syntax::Expression {
                kind: syntax::ExpressionKind::CompileTime(run),
                ..
            }) => FileItem::Run(syntax::RunDirective {
                flags: run.flags,
                body: run.body.clone(),
                location,
            }),
            _ => FileItem::Declaration(declaration(statement, source, visibility)?),
        };
        result.push(item);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn quote(text: &str) -> (syntax::CodeBody, jai_source::SourceId) {
        let mut sources = jai_source::SourceMap::default();
        let source = sources.insert("insert.jai".into(), text.into());
        let parsed = syntax::parse_file(
            sources.get(source).unwrap(),
            &mut jai_source::Symbols::default(),
        )
        .unwrap();
        let FileItem::Declaration(FileDeclaration {
            kind: D::Constant(constant),
            ..
        }) = &parsed.items()[0]
        else {
            panic!("constant")
        };
        let syntax::ExpressionKind::Code(body) = &constant.initializer.kind else {
            panic!("quote")
        };
        (body.clone(), source)
    }
    #[test]
    fn preserves_original_order_source_and_declaration_kinds() {
        let text = "Body::#code { value:int=40; answer::42; Pair::struct{ x:int; } worker::()->int{return answer;} }";
        let (body, source) = quote(text);
        let items = literal_file_items(&body, source, Visibility::File).unwrap();
        assert_eq!(items.len(), 4);
        for item in &items {
            let FileItem::Declaration(declaration) = item else {
                panic!("declaration")
            };
            assert_eq!(declaration.location.source, source);
            assert_eq!(declaration.visibility, Visibility::File);
            assert!(!declaration.location.span.text(text).is_empty());
        }
        assert!(matches!(
            items[0],
            FileItem::Declaration(FileDeclaration {
                kind: D::Global(_),
                ..
            })
        ));
        assert!(matches!(
            items[1],
            FileItem::Declaration(FileDeclaration {
                kind: D::Constant(_),
                ..
            })
        ));
        assert!(matches!(
            items[2],
            FileItem::Declaration(FileDeclaration {
                kind: D::Record(_),
                ..
            })
        ));
        assert!(matches!(
            items[3],
            FileItem::Declaration(FileDeclaration {
                kind: D::Procedure(_),
                ..
            })
        ));
    }
    #[test]
    fn preserves_conditions_without_selecting_or_executing_them() {
        let (body, source) = quote("Body::#code { #if ready { left::41; } else { right::42; } }");
        let items = literal_file_items(&body, source, Visibility::Export).unwrap();
        let FileItem::Conditional {
            then_items,
            else_items,
            ..
        } = &items[0]
        else {
            panic!("conditional")
        };
        assert_eq!(then_items.len(), 1);
        assert_eq!(else_items.len(), 1);
    }
    #[test]
    fn runtime_statement_rejection_retains_its_actual_span() {
        let text = "Body::#code { result:=40; result+=2; }";
        let (body, source) = quote(text);
        let error = literal_file_items(&body, source, Visibility::Export).unwrap_err();
        assert_eq!(error.span.text(text), "result+=2;");
        assert!(error.message.contains("runtime statements"));
    }
}
