//! Namespace lookup keeps nested nominal constants typed through default evaluation.
use super::*;

pub(crate) fn member_value(
    graph: &ModuleGraph,
    file: FileInstanceId,
    nominals: &Nominals<'_>,
    records: &RecordSpecializations,
    scope: Option<&Substitution>,
    path: &syntax::NamePath,
    span: Span,
) -> Option<BakedValue> {
    if path.members.is_empty() {
        return scope
            .and_then(|scope| {
                scope
                    .constant(path.root)
                    .cloned()
                    .or_else(|| scope.ty(path.root).map(BakedValue::Type))
            })
            .or_else(|| {
                declaration_id(graph, file, path, span)
                    .ok()
                    .and_then(|id| nominals.value_constants.get(&id))
                    .cloned()
                    .map(BakedValue::Value)
            });
    }
    for count in (0..path.members.len()).rev() {
        let prefix = syntax::NamePath {
            root: path.root,
            members: path.members[..count].to_vec(),
        };
        let root = if count == 0 {
            scope.and_then(|scope| scope.ty(path.root))
        } else {
            None
        }
        .or_else(|| {
            declaration_id(graph, file, &prefix, span)
                .ok()
                .and_then(|id| nominals.declarations.get(&id).copied())
        });
        let Some(root) = root else {
            continue;
        };
        let mut value = BakedValue::Type(root);
        let mut complete = true;
        for member in &path.members[count..] {
            let BakedValue::Type(ty) = value else {
                complete = false;
                break;
            };
            if let Some(enumeration) = records.member_enum(ty)
                && let Some((_, member)) =
                    enumeration.values.iter().find(|(name, _)| name == member)
            {
                value = BakedValue::Value(jai_ir::ConstantValue {
                    ty,
                    kind: jai_ir::ConstantKind::Enum(*member),
                });
            } else if let Some(member) = nominals
                .enums
                .get(&ty)
                .and_then(|enumeration| enumeration.members.get(member))
            {
                value = BakedValue::Value(jai_ir::ConstantValue {
                    ty,
                    kind: jai_ir::ConstantKind::Enum(*member),
                });
            } else if let Some(next) = records.member_bindings(ty).and_then(|scope| {
                scope
                    .constant(*member)
                    .cloned()
                    .or_else(|| scope.ty(*member).map(BakedValue::Type))
            }) {
                value = next;
            } else {
                complete = false;
                break;
            }
        }
        if complete {
            return Some(value);
        }
    }
    None
}

pub(crate) fn expression_path(expression: &syntax::Expression) -> Option<syntax::NamePath> {
    match &expression.kind {
        syntax::ExpressionKind::Name(name) | syntax::ExpressionKind::CompileVariable(name) => {
            Some(path(*name))
        }
        syntax::ExpressionKind::QualifiedName(path) => Some(path.clone()),
        _ => None,
    }
}
