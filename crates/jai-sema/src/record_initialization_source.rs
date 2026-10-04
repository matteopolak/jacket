//! Canonical construction actions retain the selected body chronology.
//! This helper resolves paths only; initializer evaluation belongs to preparation.
use jai_source::{Diagnostic, Span};
use jai_syntax::{Expression, ExpressionKind, FieldBinding, PlaceSyntax, RecordMember};
use jai_types::{FieldId, RecordKind, TypeId, TypeView};
#[derive(Clone, Debug)]
pub(crate) enum ActionSource {
    Explicit,
    ImplicitDefault(TypeId),
    NoWrite,
}
#[derive(Clone, Debug)]
pub(crate) struct Action {
    pub(crate) member_ordinal: usize,
    pub(crate) path: Box<[FieldId]>,
    pub(crate) span: Span,
    pub(crate) source: ActionSource,
}
pub(crate) fn selected_journal(
    types: &dyn TypeView,
    root: TypeId,
    members: &[RecordMember],
    mut override_path: impl FnMut(&PlaceSyntax) -> Result<Vec<FieldId>, Diagnostic>,
) -> Result<Vec<Action>, Diagnostic> {
    if members.len() > 65_536 {
        return Err(Diagnostic::new(
            Span::default(),
            "record construction exceeds compiler declaration budget",
        ));
    }
    let record = types
        .record_storage_definition(root)
        .map_err(|e| Diagnostic::new(Span::default(), e.to_string()))?;
    if record.kind != RecordKind::Struct {
        return Err(Diagnostic::new(
            Span::default(),
            "ordered record journals require a struct",
        ));
    }
    let mut physical = 0usize;
    let mut journal = Vec::new();
    let mut steps = 0usize;
    for (member_ordinal, member) in members.iter().enumerate() {
        let action = match member {
            RecordMember::Field(field) => {
                let canonical = types
                    .field(root, physical)
                    .map_err(|e| Diagnostic::new(field.span, e.to_string()))?;
                physical += 1;
                let expression = match &field.binding {
                    FieldBinding::Explicit {
                        initializer, ..
                    } => initializer.as_ref(),
                    FieldBinding::Inferred(value) => Some(value),
                };
                let source = match expression {
                    Some(expression)
                        if matches!(expression.kind, ExpressionKind::Uninitialized) =>
                    {
                        ActionSource::NoWrite
                    }
                    Some(_) => ActionSource::Explicit,
                    None => ActionSource::ImplicitDefault(canonical.ty),
                };
                Action {
                    member_ordinal,
                    path: Box::new([canonical.id]),
                    span: expression.map_or(field.span, |e| e.span),
                    source,
                }
            }
            RecordMember::AnonymousRecord(record) => {
                let canonical = types
                    .field(root, physical)
                    .map_err(|e| Diagnostic::new(record.span, e.to_string()))?;
                physical += 1;
                Action {
                    member_ordinal,
                    path: Box::new([canonical.id]),
                    span: record.span,
                    source: ActionSource::ImplicitDefault(canonical.ty),
                }
            }
            RecordMember::DefaultOverride {
                target,
                value,
                span,
            } => {
                let path = override_path(target)?;
                if path.is_empty() {
                    return Err(Diagnostic::new(
                        target.span,
                        "record override requires a nonempty canonical field path",
                    ));
                }
                if matches!(value.kind, ExpressionKind::Uninitialized) {
                    return Err(Diagnostic::new(
                        value.span,
                        "skipped record override semantics are not established",
                    ));
                }
                let source = ActionSource::Explicit;
                Action {
                    member_ordinal,
                    path: path.into(),
                    span: *span,
                    source,
                }
            }
            RecordMember::Conditional {
                span, ..
            }
            | RecordMember::CompileTimeCases {
                span, ..
            } => {
                return Err(Diagnostic::new(
                    *span,
                    "record construction requires selected source members",
                ));
            }
            RecordMember::Import(import) => {
                return Err(Diagnostic::new(
                    import.span,
                    "record import requires a checked record-source namespace producer",
                ));
            }
            RecordMember::Insert(directive) => {
                return Err(Diagnostic::new(
                    directive.span,
                    "record construction requires expanded source members",
                ));
            }
            // These namespace declarations/assertions do not create physical writes.
            RecordMember::Using(_)
            | RecordMember::Placement(_)
            | RecordMember::Assert {
                ..
            }
            | RecordMember::Constant(_)
            | RecordMember::TypeAlias(_)
            | RecordMember::Procedure(_)
            | RecordMember::ProcedurePrototype(_)
            | RecordMember::Record(_)
            | RecordMember::Enum(_) => continue,
        };
        if action.path.len() > 128 {
            return Err(Diagnostic::new(
                action.span,
                "record construction path exceeds compiler depth budget",
            ));
        }
        steps = steps
            .checked_add(action.path.len())
            .filter(|n| *n <= 1_048_576)
            .ok_or_else(|| {
                Diagnostic::new(
                    action.span,
                    "record construction exceeds compiler path budget",
                )
            })?;
        if journal.len() >= 65_536 {
            return Err(Diagnostic::new(
                action.span,
                "record construction exceeds compiler action budget",
            ));
        }
        let mut owner = root;
        for &field in &action.path {
            let record = types
                .record_storage_definition(owner)
                .map_err(|e| Diagnostic::new(action.span, e.to_string()))?;
            if record.kind != RecordKind::Struct {
                return Err(Diagnostic::new(
                    action.span,
                    "record construction paths require struct intermediates",
                ));
            }
            owner = types
                .validate_field(owner, field)
                .map_err(|e| Diagnostic::new(action.span, e.to_string()))?;
        }
        journal.push(action);
    }
    if physical != record.fields.len() {
        return Err(Diagnostic::new(
            Span::default(),
            "selected source fields differ from the canonical record shape",
        ));
    }
    Ok(journal)
}
/// Borrow the exact original expression; no clone or final default map intervenes.
pub(crate) fn explicit_expression<'a>(
    members: &'a [RecordMember],
    action: &Action,
) -> Option<&'a Expression> {
    if !matches!(action.source, ActionSource::Explicit) {
        return None;
    }
    match members.get(action.member_ordinal)? {
        RecordMember::Field(field) => match &field.binding {
            FieldBinding::Explicit {
                initializer, ..
            } => initializer.as_ref(),
            FieldBinding::Inferred(value) => Some(value),
        },
        RecordMember::DefaultOverride {
            value, ..
        } => Some(value),
        _ => None,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use jai_source::{SourceMap, Symbols};
    use jai_syntax::{FileDeclarationKind, FileItem, PlaceKind};
    use jai_types::{IntegerType, ScalarType, TypeRegistry};
    fn parsed(text: &str) -> (Vec<RecordMember>, Symbols) {
        let mut sources = SourceMap::default();
        let id = sources.insert("ordered-source.jai".into(), text.into());
        let mut symbols = Symbols::default();
        let parsed = jai_syntax::parse_file(sources.get(id).unwrap(), &mut symbols).unwrap();
        let FileItem::Declaration(declaration) = &parsed.items()[0] else {
            panic!()
        };
        let FileDeclarationKind::Record(record) = &declaration.kind else {
            panic!()
        };
        (record.members.clone(), symbols)
    }
    fn types(count: usize) -> (TypeRegistry, TypeId) {
        let mut types = TypeRegistry::new();
        let int = types.scalar(ScalarType::Int(IntegerType::S64));
        let root = types.reserve_record(RecordKind::Struct);
        types.define_record(root, vec![int; count]).unwrap();
        (types, root)
    }
    #[test]
    fn actual_ast_journal_keeps_defaults_and_body_overrides_interleaved() {
        let (members, symbols) = parsed("R :: struct { a:int=1; a=4; b:int=2; a=42; }");
        let (types, root) = types(2);
        let a = types.field(root, 0).unwrap().id;
        let journal = selected_journal(&types, root, &members, |target| {
            assert!(matches!(target.kind,PlaceKind::Name(n) if n==symbols.find("a").unwrap()));
            Ok(vec![a])
        })
        .unwrap();
        assert_eq!(
            journal.iter().map(|a| a.member_ordinal).collect::<Vec<_>>(),
            [0, 1, 2, 3]
        );
        assert_eq!(
            journal
                .iter()
                .map(|a| a.path[0].index())
                .collect::<Vec<_>>(),
            [0, 0, 1, 0]
        );
        assert_eq!(
            journal
                .iter()
                .map(|a| match explicit_expression(&members, a).unwrap().kind {
                    ExpressionKind::Integer(n) => n,
                    _ => panic!(),
                })
                .collect::<Vec<_>>(),
            [1, 4, 2, 42]
        );
    }
    #[test]
    fn no_write_is_retained_as_a_terminal_original_action_not_zero() {
        let (members, _) = parsed("R :: struct { a:int=---; b:int; }");
        let (types, root) = types(2);
        let journal = selected_journal(&types, root, &members, |_| panic!()).unwrap();
        assert!(matches!(journal[0].source, ActionSource::NoWrite));
        assert!(
            matches!(journal[1].source,ActionSource::ImplicitDefault(ty) if ty==types.field(root,1).unwrap().ty)
        );
        assert_eq!(
            journal[0].span,
            match &members[0] {
                RecordMember::Field(field) => match &field.binding {
                    FieldBinding::Explicit {
                        initializer: Some(value),
                        ..
                    } => value.span,
                    _ => panic!(),
                },
                _ => panic!(),
            }
        );
    }
    #[test]
    fn reordered_source_spans_never_sort_quoted_or_inserted_actions() {
        let (mut members, _) = parsed("R :: struct { a:int=1; b:int=2; }");
        let (types, root) = types(2);
        for (member, span) in members
            .iter_mut()
            .zip([Span::new(900, 901), Span::new(1, 2)])
        {
            let RecordMember::Field(field) = member else {
                panic!()
            };
            let FieldBinding::Explicit {
                initializer: Some(value),
                ..
            } = &mut field.binding
            else {
                panic!()
            };
            value.span = span;
        }
        let journal = selected_journal(&types, root, &members, |_| panic!()).unwrap();
        assert_eq!(
            journal.iter().map(|a| a.span.start).collect::<Vec<_>>(),
            [900, 1]
        );
        assert_eq!(
            journal
                .iter()
                .map(|a| a.path[0].index())
                .collect::<Vec<_>>(),
            [0, 1]
        );
    }
    #[test]
    fn wrong_override_owner_and_unselected_body_are_precise_boundaries() {
        let (members, _) = parsed("R :: struct { a:int; a=42; }");
        let (mut types, root) = types(1);
        let ty = types.field(root, 0).unwrap().ty;
        let foreign = types.reserve_record(RecordKind::Struct);
        types.define_record(foreign, [ty]).unwrap();
        let field = types.field(foreign, 0).unwrap().id;
        assert!(
            selected_journal(&types, root, &members, |_| Ok(vec![field]))
                .unwrap_err()
                .message
                .contains("belongs")
        );
        let (skipped_override, _) = parsed("R :: struct { a:int=1; a=---; }");
        let actual = types.field(root, 0).unwrap().id;
        let error =
            selected_journal(&types, root, &skipped_override, |_| Ok(vec![actual])).unwrap_err();
        assert!(error.message.contains("skipped record override"));
        let RecordMember::DefaultOverride {
            value, ..
        } = &skipped_override[1]
        else {
            panic!()
        };
        assert_eq!(error.span, value.span);
        let (unselected, _) = parsed("R :: struct { #if true { a:int; } }");
        assert!(
            selected_journal(&types, root, &unselected, |_| panic!())
                .unwrap_err()
                .message
                .contains("selected source members")
        );
    }
}
