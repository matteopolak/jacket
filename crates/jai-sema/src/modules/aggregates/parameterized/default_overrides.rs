//! Record construction assignments retain source order and canonical field identities.
use super::*;
use jai_types::{FieldId, RecordKind};

#[derive(Clone)]
pub(crate) struct RecordDefaultOverride {
    pub(crate) path: Box<[FieldId]>,
    pub(crate) value: syntax::Expression,
    pub(crate) span: Span,
}

pub(super) struct OverrideSource<'a> {
    pub(super) file: FileInstanceId,
    pub(super) owner: TypeId,
    pub(super) shape: &'a RecordMetadata,
    pub(super) members: &'a [syntax::RecordMember],
}

pub(super) fn collect(
    graph: &ModuleGraph,
    source: OverrideSource<'_>,
    types: &TypeRegistry,
    nominals: &Nominals<'_>,
    records: &RecordSpecializations,
) -> Result<Vec<RecordDefaultOverride>, LocatedDiagnostic> {
    let OverrideSource {
        file,
        owner,
        shape,
        members,
    } = source;
    let mut overrides = Vec::new();
    for member in members {
        let syntax::RecordMember::DefaultOverride {
            target,
            value,
            span,
        } = member
        else {
            continue;
        };
        let names = match &target.kind {
            syntax::PlaceKind::Name(name) => vec![*name],
            syntax::PlaceKind::Qualified(path) => std::iter::once(path.root)
                .chain(path.members.iter().copied())
                .collect(),
            _ => {
                return Err(located(
                    graph,
                    file,
                    Diagnostic::new(
                        target.span,
                        "record default overrides currently require a named field path",
                    ),
                ));
            }
        };
        if names.len() > crate::constant_limits::MAX_CONSTANT_DEPTH {
            return Err(located(
                graph,
                file,
                Diagnostic::new(
                    target.span,
                    "record default path exceeds compiler depth budget",
                ),
            ));
        }
        let mut ty = owner;
        let mut path = Vec::new();
        for name in names {
            let metadata = |ty| {
                if ty == owner {
                    Some(shape.clone())
                } else {
                    records
                        .record(ty)
                        .map(|record| record.shape.clone())
                        .or_else(|| {
                            nominals.records.get(&ty).map(|record| RecordMetadata {
                                using: Vec::new(),
                                name: None,
                                kind: record.kind,
                                fields: record
                                    .fields
                                    .iter()
                                    .map(|field| FieldMetadata {
                                        name: Some(field.name),
                                        id: field.id,
                                        ty: field.ty,
                                        syntax: field.syntax.clone().into(),
                                    })
                                    .collect(),
                            })
                        })
                }
                .ok_or_else(|| {
                    Diagnostic::new(
                        target.span,
                        "record default path requires a ready record field schema",
                    )
                })
            };
            if metadata(ty)
                .map_err(|error| located(graph, file, error))?
                .kind
                != RecordKind::Struct
            {
                return Err(located(
                    graph,
                    file,
                    Diagnostic::new(
                        target.span,
                        "record default overrides require a struct field path",
                    ),
                ));
            }
            let projected = crate::record_default_overrides::find_field_path(
                ty,
                name,
                target.span,
                types,
                metadata,
            )
            .map_err(|error| located(graph, file, error))?;
            for field in projected {
                if types
                    .record_definition(ty)
                    .map_err(|error| {
                        located(graph, file, Diagnostic::new(target.span, error.to_string()))
                    })?
                    .kind
                    != RecordKind::Struct
                {
                    return Err(located(
                        graph,
                        file,
                        Diagnostic::new(
                            target.span,
                            "record default overrides require a struct field path",
                        ),
                    ));
                }
                ty = types.validate_field(ty, field).map_err(|error| {
                    located(graph, file, Diagnostic::new(target.span, error.to_string()))
                })?;
                path.push(field);
                if path.len() > crate::constant_limits::MAX_CONSTANT_DEPTH {
                    return Err(located(
                        graph,
                        file,
                        Diagnostic::new(
                            target.span,
                            "record default path exceeds compiler depth budget",
                        ),
                    ));
                }
            }
        }
        overrides.push(RecordDefaultOverride {
            path: path.into_boxed_slice(),
            value: value.clone(),
            span: *span,
        });
    }
    Ok(overrides)
}
