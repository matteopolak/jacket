//! Late specializations preserve the same source descriptor metadata as static records.
use super::*;
use jai_types::{
    ReflectedEnumMember, ReflectedEnumMetadata, ReflectedFieldMetadata, ReflectedRecordMetadata,
    ReflectionMetadata,
};

impl RecordSpecializations {
    pub(crate) fn append_reflection_metadata(
        &self,
        metadata: &mut ReflectionMetadata,
        symbols: &jai_source::Symbols,
    ) {
        for (ty, record) in self.records() {
            if let Some(name) = record.shape.name {
                metadata.name(ty, symbols.name(name).as_bytes());
            }
            if let Some(source) = self.reflected_record(ty) {
                metadata.record(ty, source.clone());
            }
            for field in &record.shape.fields {
                metadata.field(
                    field.id,
                    ReflectedFieldMetadata {
                        name: field.name.map(|name| symbols.name(name).as_bytes().into()),
                        using: field.syntax.using(),
                    },
                );
                if let Some(notes) = self.reflected_field_notes(field.id) {
                    metadata.field_notes(field.id, notes);
                }
            }
        }
        for (ty, enumeration) in self.member_enums() {
            if let Some(name) = enumeration.name {
                metadata.name(ty, symbols.name(name).as_bytes());
            }
            metadata.enumeration(
                ty,
                ReflectedEnumMetadata {
                    flags: enumeration.flags,
                    members: enumeration
                        .values
                        .iter()
                        .map(|(name, value)| ReflectedEnumMember {
                            name: Some(symbols.name(*name).as_bytes().into()),
                            value: *value,
                        })
                        .collect(),
                },
            );
        }
        // Only complete checked Type namespaces can replace the unsupported
        // member marker. A pending/method/value member retains that diagnostic.
        for (owner, names) in self.source_namespaces() {
            let Some(scope) = self.member_bindings(owner) else {
                continue;
            };
            let constants = names
                .iter()
                .map(|&name| match scope.constant(name) {
                    Some(BakedValue::Type(ty)) => Some(jai_types::ReflectedTypeConstant {
                        name: symbols.name(name).as_bytes().into(),
                        represented_type: *ty,
                    }),
                    _ => None,
                })
                .collect::<Option<Vec<_>>>();
            if let Some(constants) = constants {
                metadata.record_type_constants(owner, constants);
            }
        }
    }
}

pub(super) fn record_body_metadata(
    record: RecordBody<'_>,
    source: &str,
) -> ReflectedRecordMetadata {
    let union = record.kind == jai_types::RecordKind::Union;
    ReflectedRecordMetadata {
        notes: notes(record.notes, source),
        textual_flags: record.attributes.iter().fold(
            if union {
                2
            } else {
                0
            },
            |flags, attribute| {
                flags
                    | match attribute {
                        syntax::RecordAttribute::NoPadding => 4,
                        syntax::RecordAttribute::TypeInfoNone => 8,
                        syntax::RecordAttribute::Alignment(_)
                        | syntax::RecordAttribute::Reflection(_) => 0,
                    }
            },
        ),
        status_flags: 0,
        nontextual_flags: if union {
            64
        } else {
            0
        },
        unsupported_members: record.members.iter().any(|member| {
            !matches!(
                member,
                syntax::RecordMember::Using(_)
                    | syntax::RecordMember::Field(_)
                    | syntax::RecordMember::Placement(_)
                    | syntax::RecordMember::AnonymousRecord(_)
                    | syntax::RecordMember::DefaultOverride { .. }
            )
        }),
    }
}
pub(super) fn notes(notes: &[syntax::NoteSyntax], source: &str) -> Box<[Box<[u8]>]> {
    notes
        .iter()
        .map(|note| {
            note.span
                .text(source)
                .strip_prefix('@')
                .unwrap_or(note.span.text(source))
                .as_bytes()
                .into()
        })
        .collect()
}
