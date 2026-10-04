//! Select source promotions before resolving canonical physical paths.
use crate::local_declarations::RecordMetadata;
use crate::{Diagnostic, Span};
use jai_source::Symbol;
use jai_syntax::{Expression, ExpressionKind as E, UsingNames, UsingSelection};
use jai_types::{FieldId, TypeId, TypeView};

pub(crate) fn selected(
    selection: &UsingSelection,
    name: Symbol,
    span: Span,
) -> Result<bool, Diagnostic> {
    Ok(match selection {
        UsingSelection::All => true,
        UsingSelection::Only(UsingNames::Names(names)) => {
            names.iter().any(|item| item.name == name)
        }
        UsingSelection::Except(UsingNames::Names(names)) => {
            !names.iter().any(|item| item.name == name)
        }
        UsingSelection::Only(UsingNames::Expression(_))
        | UsingSelection::Except(UsingNames::Expression(_))
        | UsingSelection::Map(_) => {
            return Err(Diagnostic::new(
                span,
                "computed record using requires checked source publication",
            ));
        }
    })
}

pub(crate) fn target_names(source: &Expression) -> Result<Vec<Symbol>, Diagnostic> {
    let mut suffix = Vec::new();
    let mut value = source;
    loop {
        if suffix.len() >= crate::constant_limits::MAX_CONSTANT_DEPTH {
            return Err(Diagnostic::new(
                source.span,
                "record using target exceeds compiler depth budget",
            ));
        }
        match &value.kind {
            E::Name(name) => {
                suffix.push(*name);
                break;
            }
            E::QualifiedName(path) => {
                if suffix
                    .len()
                    .saturating_add(path.members.len())
                    .saturating_add(1)
                    > crate::constant_limits::MAX_CONSTANT_DEPTH
                {
                    return Err(Diagnostic::new(
                        source.span,
                        "record using target exceeds compiler depth budget",
                    ));
                }
                suffix.extend(path.members.iter().rev());
                suffix.push(path.root);
                break;
            }
            E::Member {
                base,
                member,
            } => {
                suffix.push(*member);
                value = base;
            }
            _ => {
                return Err(Diagnostic::new(
                    value.span,
                    "record using target requires an original field or type path",
                ));
            }
        }
    }
    suffix.reverse();
    Ok(suffix)
}

/// None denotes a namespace target, which the real namespace producer binds.
/// Physical paths only contain actual validated FieldIds; pointer promotion
/// cannot be represented by substituting a pointee's FieldId for a dereference.
pub(crate) fn target_path(
    ty: TypeId,
    source: &Expression,
    types: &dyn TypeView,
    metadata: &mut impl FnMut(TypeId) -> Result<RecordMetadata, Diagnostic>,
) -> Result<Option<(TypeId, Vec<FieldId>)>, Diagnostic> {
    let names = target_names(source)?;
    let mut current = ty;
    let mut path = Vec::new();
    for (index, name) in names.iter().enumerate() {
        let record = metadata(current)?;
        let Some(field) = record.fields.iter().find(|field| field.name == Some(*name)) else {
            if index == 0 {
                return Ok(None);
            }
            return Err(Diagnostic::new(
                source.span,
                "unknown record using target field",
            ));
        };
        let actual = types
            .validate_field(current, field.id)
            .map_err(|e| Diagnostic::new(source.span, e.to_string()))?;
        if actual != field.ty {
            return Err(Diagnostic::new(
                source.span,
                "record using target differs from its canonical field type",
            ));
        }
        path.push(field.id);
        current = field.ty;
    }
    if !matches!(types.kind(current), Ok(jai_types::TypeKind::Record(_))) {
        return Err(Diagnostic::new(
            source.span,
            "record using requires an owned record value; pointer promotion needs a checked dereference path",
        ));
    }
    Ok(Some((current, path)))
}

pub(crate) fn visible_names(
    root: TypeId,
    span: Span,
    types: &dyn TypeView,
    mut metadata: impl FnMut(TypeId) -> Result<RecordMetadata, Diagnostic>,
) -> Result<std::collections::HashSet<Symbol>, Diagnostic> {
    // Carry the complete ordered selection chain. A hidden name is never a
    // candidate for ambiguity checking in the enclosing record.
    let mut pending = vec![(
        root,
        Vec::<UsingSelection>::new(),
        std::collections::HashSet::new(),
    )];
    let mut names = std::collections::HashSet::new();
    let mut visited = 0usize;
    while let Some((ty, policies, mut ancestors)) = pending.pop() {
        visited += 1;
        if visited > 65_536 || ancestors.len() >= crate::constant_limits::MAX_CONSTANT_DEPTH {
            return Err(Diagnostic::new(
                span,
                "record using exceeds compiler declaration budget",
            ));
        }
        if !ancestors.insert(ty) {
            return Err(Diagnostic::new(span, "cyclic using field promotion"));
        }
        let record = metadata(ty)?;
        visited = visited
            .checked_add(record.fields.len())
            .and_then(|n| n.checked_add(record.using.len()))
            .filter(|n| *n <= 65_536)
            .ok_or_else(|| {
                Diagnostic::new(span, "record using exceeds compiler declaration budget")
            })?;
        for field in &record.fields {
            if let Some(name) = field.name {
                let mut allowed = true;
                for selection in &policies {
                    allowed &= selected(selection, name, span)?;
                }
                if allowed {
                    names.insert(name);
                }
            }
            if field.syntax.using() {
                // Also validate selectors with no visible child names.
                if matches!(
                    field.syntax.using_selection(),
                    UsingSelection::Map(_)
                        | UsingSelection::Only(UsingNames::Expression(_))
                        | UsingSelection::Except(UsingNames::Expression(_))
                ) {
                    return Err(Diagnostic::new(
                        field.syntax.span(),
                        "computed record using requires checked source publication",
                    ));
                }
                if !matches!(types.kind(field.ty), Ok(jai_types::TypeKind::Record(_))) {
                    return Err(Diagnostic::new(
                        field.syntax.span(),
                        "using field requires an owned record value",
                    ));
                }
                let mut next = policies.clone();
                next.push(field.syntax.using_selection().clone());
                if pending.len() >= 65_536 {
                    return Err(Diagnostic::new(
                        span,
                        "record using exceeds compiler declaration budget",
                    ));
                }
                pending.push((field.ty, next, ancestors.clone()));
            }
        }
        for directive in &record.using {
            if let Some((child, _)) = target_path(ty, &directive.target, types, &mut metadata)? {
                let mut next = policies.clone();
                next.push(directive.selection.clone());
                if pending.len() >= 65_536 {
                    return Err(Diagnostic::new(
                        span,
                        "record using exceeds compiler declaration budget",
                    ));
                }
                pending.push((child, next, ancestors.clone()));
            }
        }
    }
    Ok(names)
}
