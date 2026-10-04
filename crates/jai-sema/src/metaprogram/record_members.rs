//! Reify direct declaration quotations before a record's field identities exist.
use super::*;

const MAX_INSERT_DEPTH: usize = 128;
const MAX_MEMBERS: usize = 65_536;

enum Completion<'a> {
    Root,
    Using(syntax::UsingDirective),
    Then {
        condition: &'a syntax::Expression,
        else_body: &'a [syntax::Statement],
        span: Span,
        depth: usize,
    },
    Else {
        condition: &'a syntax::Expression,
        then_members: Vec<syntax::RecordMember>,
        span: Span,
    },
    CaseArm {
        cases: &'a syntax::CompileTimeCases<syntax::Statement>,
        index: usize,
        arms: Vec<syntax::CompileTimeCaseArm<syntax::RecordMember>>,
        depth: usize,
    },
    CaseDefault {
        cases: &'a syntax::CompileTimeCases<syntax::Statement>,
        arms: Vec<syntax::CompileTimeCaseArm<syntax::RecordMember>>,
    },
}
struct MemberFrame<'a> {
    pending: Vec<(&'a syntax::Statement, usize)>,
    result: Vec<syntax::RecordMember>,
    completion: Completion<'a>,
}

/// This adapter has no runtime evaluator or source-text path. Its caller retains
/// the enclosing record's source file and specialization environment.
pub(crate) fn literal_record_members(
    body: &syntax::CodeBody,
) -> Result<Vec<syntax::RecordMember>, Diagnostic> {
    let mut pending = Vec::new();
    enqueue(body, 0, &mut pending)?;
    let mut frames = vec![MemberFrame {
        pending,
        result: Vec::new(),
        completion: Completion::Root,
    }];
    let mut visited = 0usize;
    loop {
        let queued = frames.iter().fold(0usize, |count, frame| {
            count.saturating_add(frame.pending.len())
        });
        if visited.saturating_add(queued) > MAX_MEMBERS {
            let span = frames
                .iter()
                .rev()
                .find_map(|frame| frame.pending.last().map(|(statement, _)| statement.span))
                .unwrap_or_default();
            return Err(Diagnostic::new(
                span,
                "record insertion exceeds declaration budget",
            ));
        }
        let Some((statement, depth)) = frames.last_mut().unwrap().pending.pop() else {
            let frame = frames.pop().unwrap();
            match frame.completion {
                Completion::Root => return Ok(frame.result),
                Completion::Using(directive) => {
                    let mut members = frame.result;
                    if members.len() != 1 {
                        return Err(Diagnostic::new(
                            directive.span,
                            "record using declaration requires one original child",
                        ));
                    }
                    if let syntax::RecordMember::Field(field) = &mut members[0] {
                        field.using = true;
                        field.using_selection = directive.selection;
                        field.span = directive.span;
                    } else {
                        members.push(syntax::RecordMember::Using(directive));
                    }
                    frames.last_mut().unwrap().result.extend(members);
                }
                Completion::Then {
                    condition,
                    else_body,
                    span,
                    depth,
                } => {
                    let mut pending = Vec::new();
                    enqueue_statements(else_body, depth, span, &mut pending)?;
                    frames.push(MemberFrame {
                        pending,
                        result: Vec::new(),
                        completion: Completion::Else {
                            condition,
                            then_members: frame.result,
                            span,
                        },
                    });
                }
                Completion::Else {
                    condition,
                    then_members,
                    span,
                } => frames
                    .last_mut()
                    .unwrap()
                    .result
                    .push(syntax::RecordMember::Conditional {
                        condition: condition.clone(),
                        then_members,
                        else_members: frame.result,
                        span,
                    }),
                Completion::CaseArm {
                    cases,
                    index,
                    mut arms,
                    depth,
                } => {
                    let original = &cases.arms[index];
                    arms.push(syntax::CompileTimeCaseArm {
                        label: original.label.clone(),
                        body: frame.result,
                        falls_through: original.falls_through,
                        span: original.span,
                    });
                    if let Some(next) = cases.arms.get(index + 1) {
                        let mut pending = Vec::new();
                        enqueue_statements(&next.body, depth, next.span, &mut pending)?;
                        frames.push(MemberFrame {
                            pending,
                            result: Vec::new(),
                            completion: Completion::CaseArm {
                                cases,
                                index: index + 1,
                                arms,
                                depth,
                            },
                        });
                    } else if let Some(default) = &cases.default {
                        let mut pending = Vec::new();
                        enqueue_statements(&default.body, depth, default.span, &mut pending)?;
                        frames.push(MemberFrame {
                            pending,
                            result: Vec::new(),
                            completion: Completion::CaseDefault {
                                cases,
                                arms,
                            },
                        });
                    } else {
                        frames
                            .last_mut()
                            .unwrap()
                            .result
                            .push(record_cases(cases, arms, None));
                    }
                }
                Completion::CaseDefault {
                    cases,
                    arms,
                } => {
                    let default = cases
                        .default
                        .as_ref()
                        .expect("default completion has source arm");
                    frames.last_mut().unwrap().result.push(record_cases(
                        cases,
                        arms,
                        Some(syntax::CompileTimeCaseDefault {
                            body: frame.result,
                            span: default.span,
                        }),
                    ));
                }
            }
            continue;
        };
        visited += 1;
        if visited > MAX_MEMBERS {
            return Err(Diagnostic::new(
                statement.span,
                "record insertion exceeds declaration budget",
            ));
        }
        let member = match &statement.kind {
            syntax::StatementKind::UsingDeclaration {
                declaration, ..
            } => {
                let directive = statement.using_declaration_directive().ok_or_else(|| {
                    Diagnostic::new(
                        statement.span,
                        "record using requires one named original declaration",
                    )
                })?;
                frames.push(MemberFrame {
                    pending: vec![(declaration, depth + 1)],
                    result: Vec::new(),
                    completion: Completion::Using(directive),
                });
                continue;
            }
            syntax::StatementKind::CompileTimeCases(cases) => {
                visited = visited
                    .saturating_add(cases.arms.len())
                    .saturating_add(usize::from(cases.default.is_some()));
                if visited > MAX_MEMBERS {
                    return Err(Diagnostic::new(
                        statement.span,
                        "record insertion exceeds declaration budget",
                    ));
                }
                if let Some(first) = cases.arms.first() {
                    let mut pending = Vec::new();
                    enqueue_statements(&first.body, depth + 1, first.span, &mut pending)?;
                    frames.push(MemberFrame {
                        pending,
                        result: Vec::new(),
                        completion: Completion::CaseArm {
                            cases,
                            index: 0,
                            arms: Vec::new(),
                            depth: depth + 1,
                        },
                    });
                    continue;
                }
                if let Some(default) = &cases.default {
                    let mut pending = Vec::new();
                    enqueue_statements(&default.body, depth + 1, default.span, &mut pending)?;
                    frames.push(MemberFrame {
                        pending,
                        result: Vec::new(),
                        completion: Completion::CaseDefault {
                            cases,
                            arms: Vec::new(),
                        },
                    });
                    continue;
                }
                record_cases(cases, Vec::new(), None)
            }
            syntax::StatementKind::CompileTimeAssert {
                condition,
                message,
            } => syntax::RecordMember::Assert {
                condition: condition.clone(),
                message: message.clone(),
                span: statement.span,
            },
            syntax::StatementKind::CompileTimeIf {
                condition,
                then_body,
                else_body,
            } => {
                let mut pending = Vec::new();
                enqueue_statements(then_body, depth + 1, statement.span, &mut pending)?;
                frames.push(MemberFrame {
                    pending,
                    result: Vec::new(),
                    completion: Completion::Then {
                        condition,
                        else_body,
                        span: statement.span,
                        depth: depth + 1,
                    },
                });
                continue;
            }
            syntax::StatementKind::Declare(declaration) => {
                let binding = match declaration {
                    syntax::Declaration::GroupMember {
                        ..
                    } => {
                        return Err(Diagnostic::new(
                            statement.span,
                            "file declaration group requires its source publication owner",
                        ));
                    }
                    syntax::Declaration::External {
                        ..
                    } => {
                        return Err(Diagnostic::new(
                            statement.span,
                            "external storage cannot become a record insertion field",
                        ));
                    }
                    syntax::Declaration::Inferred {
                        initializer, ..
                    } => syntax::FieldBinding::Inferred(initializer.clone()),
                    syntax::Declaration::Explicit {
                        ty,
                        initializer,
                        ..
                    } => syntax::FieldBinding::Explicit {
                        ty: syntax::TypeSyntax::Builtin(syntax::BuiltinType::Scalar(*ty)),
                        initializer: initializer.clone(),
                    },
                    syntax::Declaration::UnresolvedExplicit {
                        ty,
                        initializer,
                        ..
                    } => syntax::FieldBinding::Explicit {
                        ty: ty.clone(),
                        initializer: initializer.clone(),
                    },
                };
                let attributes = declaration
                    .attributes()
                    .iter()
                    .map(|attribute| match attribute {
                        syntax::DeclarationAttribute::Alignment(expression) => {
                            syntax::FieldAttribute::Alignment(expression.clone())
                        }
                    })
                    .collect();
                syntax::RecordMember::Field(syntax::FieldDeclaration {
                    name: declaration.name(),
                    binding,
                    using: false,
                    using_selection: syntax::UsingSelection::All,
                    conversion: syntax::FieldConversion::None,
                    span: statement.span,
                    attributes,
                    notes: Vec::new(),
                })
            }
            syntax::StatementKind::Using(value) => syntax::RecordMember::Using(value.clone()),
            syntax::StatementKind::Assign(name, value) => {
                syntax::RecordMember::DefaultOverride {
                    target: syntax::PlaceSyntax {
                        kind: syntax::PlaceKind::Name(*name),
                        // The scalar statement retains its enclosing range,
                        // rather than a separate identifier token range.
                        span: statement.span,
                    },
                    value: value.clone(),
                    span: statement.span,
                }
            }
            syntax::StatementKind::AssignPlace {
                target,
                value,
            } => syntax::RecordMember::DefaultOverride {
                target: target.clone(),
                value: value.clone(),
                span: statement.span,
            },
            syntax::StatementKind::Constant(value) => {
                let mut value = value.clone();
                // The statement wrapper retains the full declaration range;
                // preserve that range when its wrapper becomes a record member.
                value.span = statement.span;
                syntax::RecordMember::Constant(value)
            }
            syntax::StatementKind::TypeAlias(value) => {
                syntax::RecordMember::TypeAlias(value.clone())
            }
            syntax::StatementKind::Procedure(value) => {
                syntax::RecordMember::Procedure(value.clone())
            }
            syntax::StatementKind::ProcedurePrototype(value) => {
                syntax::RecordMember::ProcedurePrototype(value.clone())
            }
            syntax::StatementKind::Record(value) => {
                syntax::RecordMember::Record(Box::new(value.clone()))
            }
            syntax::StatementKind::Enum(value) => syntax::RecordMember::Enum(value.clone()),
            syntax::StatementKind::Insert(directive) => {
                if !directive.replacements.is_empty() {
                    return Err(Diagnostic::new(
                        directive.span,
                        "record insertion jump replacements require statement expansion binding",
                    ));
                }
                let syntax::ExpressionKind::Code(body) = &directive.value.kind else {
                    return Err(Diagnostic::new(
                        directive.value.span,
                        "record insertion currently requires direct quoted declaration syntax",
                    ));
                };
                enqueue(body, depth + 1, &mut frames.last_mut().unwrap().pending)?;
                continue;
            }
            _ => {
                return Err(Diagnostic::new(
                    statement.span,
                    "record insertion requires declarations; runtime statements cannot become record members",
                ));
            }
        };
        frames.last_mut().unwrap().result.push(member);
    }
}

fn record_cases(
    source: &syntax::CompileTimeCases<syntax::Statement>,
    arms: Vec<syntax::CompileTimeCaseArm<syntax::RecordMember>>,
    default: Option<syntax::CompileTimeCaseDefault<syntax::RecordMember>>,
) -> syntax::RecordMember {
    syntax::RecordMember::CompileTimeCases {
        cases: syntax::CompileTimeCases {
            default_position: source.default_position,
            default_through: source.default_through,
            value: source.value.clone(),
            operator: source.operator,
            arms,
            default,
            complete: source.complete,
            span: source.span,
        },
        span: source.span,
    }
}

fn enqueue<'a>(
    body: &'a syntax::CodeBody,
    depth: usize,
    pending: &mut Vec<(&'a syntax::Statement, usize)>,
) -> Result<(), Diagnostic> {
    let (statements, span) = match body {
        syntax::CodeBody::Null => {
            return Err(Diagnostic::new(
                Span::default(),
                "record insertion requires captured declarations; #code,null has no syntax",
            ));
        }
        syntax::CodeBody::Block(statements) => (
            statements.as_slice(),
            statements
                .first()
                .map_or(Span::default(), |statement| statement.span),
        ),
        syntax::CodeBody::Statement(statement) => {
            (std::slice::from_ref(statement.as_ref()), statement.span)
        }
        syntax::CodeBody::Expression(expression) => {
            return Err(Diagnostic::new(
                expression.span,
                "record insertion requires quoted declaration syntax, not an expression",
            ));
        }
    };
    enqueue_statements(statements, depth, span, pending)
}

fn enqueue_statements<'a>(
    statements: &'a [syntax::Statement],
    depth: usize,
    span: Span,
    pending: &mut Vec<(&'a syntax::Statement, usize)>,
) -> Result<(), Diagnostic> {
    if depth >= MAX_INSERT_DEPTH {
        return Err(Diagnostic::new(
            span,
            "record insertion exceeds expansion depth limit",
        ));
    }
    if pending.len().saturating_add(statements.len()) > MAX_MEMBERS {
        return Err(Diagnostic::new(
            span,
            "record insertion exceeds declaration budget",
        ));
    }
    pending.extend(statements.iter().rev().map(|statement| (statement, depth)));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn quote(text: &str) -> syntax::CodeBody {
        let mut sources = jai_source::SourceMap::default();
        let source = sources.insert("quote.jai".into(), text.into());
        let parsed = syntax::parse_file(
            sources.get(source).unwrap(),
            &mut jai_source::Symbols::default(),
        )
        .unwrap();
        let syntax::FileItem::Declaration(syntax::FileDeclaration {
            kind: syntax::FileDeclarationKind::Constant(constant),
            ..
        }) = &parsed.items()[0]
        else {
            panic!("constant")
        };
        let syntax::ExpressionKind::Code(body) = constant.initializer.kind.clone() else {
            panic!("quote")
        };
        body
    }

    #[test]
    fn direct_nested_quotes_keep_declaration_order_defaults_and_source_ranges() {
        let text = "members :: #code { before:s32=3; #insert #code { LIMIT::7; after:int; }; } ;";
        let body = quote(text);
        let members = literal_record_members(&body).unwrap();
        let [
            syntax::RecordMember::Field(before),
            syntax::RecordMember::Constant(limit),
            syntax::RecordMember::Field(after),
        ] = members.as_slice()
        else {
            panic!("declaration sequence")
        };
        assert_eq!(before.span.text(text), "before:s32=3;");
        assert_eq!(limit.span.text(text), "LIMIT::7;");
        assert_eq!(after.span.text(text), "after:int;");
        assert!(matches!(
            &before.binding,
            syntax::FieldBinding::Explicit {
                initializer: Some(syntax::Expression {
                    kind: syntax::ExpressionKind::Integer(3),
                    ..
                }),
                ..
            }
        ));
    }

    #[test]
    fn using_declarations_retain_their_child_and_selection() {
        let text = "members::#code{using,only(value) base:Base;};";
        let body = quote(text);
        let members = literal_record_members(&body).unwrap();
        let [syntax::RecordMember::Field(field)] = members.as_slice() else {
            panic!("original physical child retained");
        };
        assert!(field.using);
        assert!(matches!(
            field.using_selection,
            syntax::UsingSelection::Only(_)
        ));
        assert_eq!(field.span.text(text), "using,only(value) base:Base;");
    }

    #[test]
    fn runtime_statements_and_dynamic_code_are_rejected_with_original_ranges() {
        for text in [
            "members::#code { target+=1; };",
            "members::#code { #insert other; };",
            "members::#code { #insert(remove={ target+=1; }) #code { value:int; }; };",
        ] {
            let body = quote(text);
            let diagnostic = literal_record_members(&body).unwrap_err();
            assert!(diagnostic.span.end <= text.len());
            assert!(diagnostic.message.contains("record insertion"));
        }
    }

    #[test]
    fn quoted_default_overrides_keep_targets_values_and_assignment_ranges() {
        let text = "members::#code { value=7; base.value=9; };";
        let members = literal_record_members(&quote(text)).unwrap();
        let [
            syntax::RecordMember::DefaultOverride {
                target: scalar,
                value: first,
                span: first_span,
            },
            syntax::RecordMember::DefaultOverride {
                target: nested,
                value: second,
                span: second_span,
            },
        ] = members.as_slice()
        else {
            panic!("default overrides")
        };
        assert!(matches!(scalar.kind, syntax::PlaceKind::Name(_)));
        assert_eq!(scalar.span, *first_span);
        assert_eq!(first_span.text(text), "value=7;");
        assert_eq!(nested.span.text(text), "base.value");
        assert_eq!(second_span.text(text), "base.value=9;");
        assert!(matches!(first.kind, syntax::ExpressionKind::Integer(7)));
        assert!(matches!(second.kind, syntax::ExpressionKind::Integer(9)));
        assert_eq!(first.span.text(text), "7");
        assert_eq!(second.span.text(text), "9");
    }

    #[test]
    fn quoted_conditions_keep_both_branches_for_typed_record_selection() {
        let text = "members::#code { #if enabled { first:s32; #insert #code { LIMIT::9; }; } else { #if alternate { second:u8; } else { third:int; } } tail:bool; };";
        let members = literal_record_members(&quote(text)).unwrap();
        let [
            syntax::RecordMember::Conditional {
                then_members,
                else_members,
                span,
                ..
            },
            syntax::RecordMember::Field(tail),
        ] = members.as_slice()
        else {
            panic!("conditional plus trailing field")
        };
        assert!(matches!(
            then_members.as_slice(),
            [
                syntax::RecordMember::Field(_),
                syntax::RecordMember::Constant(_)
            ]
        ));
        assert!(
            matches!(else_members.as_slice(), [syntax::RecordMember::Conditional { then_members, else_members, .. }] if then_members.len() == 1 && else_members.len() == 1)
        );
        assert!(span.text(text).starts_with("#if enabled"));
        assert_eq!(tail.span.text(text), "tail:bool;");
    }

    #[test]
    fn quoted_case_tables_keep_labels_through_defaults_and_source_ranges() {
        let text = "members::#code { #if #complete SELECT == { case 1; first:s32; #through; case 2; #if enabled { second:u8; } else { third:int; } case; fallback:bool; } tail:u64; };";
        let members = literal_record_members(&quote(text)).unwrap();
        let [
            syntax::RecordMember::CompileTimeCases {
                cases,
                span,
            },
            syntax::RecordMember::Field(tail),
        ] = members.as_slice()
        else {
            panic!("case table plus trailing field")
        };
        assert_eq!(cases.operator, syntax::CaseOperator::Equal);
        assert!(cases.complete);
        assert_eq!(cases.arms.len(), 2);
        assert!(cases.arms[0].falls_through);
        assert!(!cases.arms[1].falls_through);
        assert_eq!(cases.value.span.text(text), "SELECT");
        assert_eq!(cases.arms[0].label.span.text(text), "1");
        assert!(cases.arms[0].span.text(text).starts_with("case 1;"));
        assert!(matches!(
            cases.arms[1].body.as_slice(),
            [syntax::RecordMember::Conditional { .. }]
        ));
        let default = cases.default.as_ref().unwrap();
        assert!(default.span.text(text).starts_with("case;"));
        assert!(matches!(
            default.body.as_slice(),
            [syntax::RecordMember::Field(_)]
        ));
        assert_eq!(*span, cases.span);
        assert!(span.text(text).starts_with("#if #complete"));
        assert_eq!(tail.span.text(text), "tail:u64;");
    }
}
