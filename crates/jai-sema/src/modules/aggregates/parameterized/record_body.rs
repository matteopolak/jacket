//! Borrow source record bodies without inventing a name for anonymous schemas.
use crate::local_declarations::FieldSourceRef;
use crate::{Span, Symbol, syntax};

#[derive(Clone, Copy)]
pub(super) struct RecordBody<'a> {
    pub(super) name: Option<Symbol>,
    pub(super) kind: jai_types::RecordKind,
    pub(super) members: &'a [syntax::RecordMember],
    pub(super) attributes: &'a [syntax::RecordAttribute],
    pub(super) notes: &'a [syntax::NoteSyntax],
    pub(super) modify: Option<&'a syntax::ModifyDirective>,
    pub(super) span: Span,
}
impl<'a> RecordBody<'a> {
    pub(super) fn physical_fields(self) -> impl Iterator<Item = FieldSourceRef<'a>> {
        self.members.iter().filter_map(|member| match member {
            syntax::RecordMember::Field(field) => Some(FieldSourceRef::Named(field)),
            syntax::RecordMember::AnonymousRecord(record) => {
                Some(FieldSourceRef::AnonymousRecord(record))
            }
            _ => None,
        })
    }
}
impl<'a> From<&'a syntax::RecordDeclaration> for RecordBody<'a> {
    fn from(record: &'a syntax::RecordDeclaration) -> Self {
        Self {
            name: Some(record.name),
            kind: record.kind,
            members: &record.members,
            attributes: &record.attributes,
            notes: &record.notes,
            modify: record.modify.as_ref(),
            span: record.span,
        }
    }
}
impl<'a> From<&'a syntax::RecordTypeSyntax> for RecordBody<'a> {
    fn from(record: &'a syntax::RecordTypeSyntax) -> Self {
        Self {
            name: None,
            kind: record.kind,
            members: &record.members,
            attributes: &record.attributes,
            notes: &record.notes,
            modify: record.modify.as_ref(),
            span: record.span,
        }
    }
}
