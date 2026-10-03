//! Physical record fields retain their actual named or unnamed source node.
use super::*;

#[derive(Clone)]
pub(crate) enum FieldSource {
    Named(syntax::FieldDeclaration),
    AnonymousRecord(syntax::RecordTypeSyntax),
}

#[derive(Clone, Copy)]
pub(crate) enum FieldSourceRef<'a> {
    Named(&'a syntax::FieldDeclaration),
    AnonymousRecord(&'a syntax::RecordTypeSyntax),
}

impl FieldSource {
    pub(crate) fn as_ref(&self) -> FieldSourceRef<'_> {
        match self {
            Self::Named(field) => FieldSourceRef::Named(field),
            Self::AnonymousRecord(record) => FieldSourceRef::AnonymousRecord(record),
        }
    }
    pub(crate) fn span(&self) -> Span {
        self.as_ref().span()
    }
    pub(crate) fn using(&self) -> bool {
        self.as_ref().using()
    }
    pub(crate) fn conversion(&self) -> syntax::FieldConversion {
        self.as_ref().conversion()
    }
    pub(crate) fn initializer(&self) -> Option<&syntax::Expression> {
        self.as_ref().initializer()
    }
    pub(crate) fn named_binding(&self) -> Option<&syntax::FieldBinding> {
        match self {
            Self::Named(field) => Some(&field.binding),
            Self::AnonymousRecord(_) => None,
        }
    }
    pub(crate) fn is_anonymous(&self) -> bool {
        matches!(self, Self::AnonymousRecord(_))
    }
    pub(crate) fn notes(&self) -> &[syntax::NoteSyntax] {
        self.as_ref().notes()
    }
}

impl<'a> FieldSourceRef<'a> {
    pub(crate) fn name(self) -> Option<Symbol> {
        match self {
            Self::Named(field) => Some(field.name),
            Self::AnonymousRecord(_) => None,
        }
    }
    pub(crate) fn span(self) -> Span {
        match self {
            Self::Named(field) => field.span,
            Self::AnonymousRecord(record) => record.span,
        }
    }
    pub(crate) fn using(self) -> bool {
        match self {
            Self::Named(field) => field.using,
            Self::AnonymousRecord(_) => true,
        }
    }
    pub(crate) fn conversion(self) -> syntax::FieldConversion {
        match self {
            Self::Named(field) => field.conversion,
            Self::AnonymousRecord(_) => syntax::FieldConversion::None,
        }
    }
    pub(crate) fn initializer(self) -> Option<&'a syntax::Expression> {
        match self {
            Self::Named(field) => match &field.binding {
                syntax::FieldBinding::Explicit { initializer, .. } => initializer.as_ref(),
                syntax::FieldBinding::Inferred(expression) => Some(expression),
            },
            Self::AnonymousRecord(_) => None,
        }
    }
    pub(crate) fn attributes(self) -> &'a [syntax::FieldAttribute] {
        match self {
            Self::Named(field) => &field.attributes,
            Self::AnonymousRecord(_) => &[],
        }
    }
    pub(crate) fn notes(self) -> &'a [syntax::NoteSyntax] {
        match self {
            Self::Named(field) => &field.notes,
            Self::AnonymousRecord(_) => &[],
        }
    }
    pub(crate) fn to_owned(self) -> FieldSource {
        match self {
            Self::Named(field) => FieldSource::Named(field.clone()),
            Self::AnonymousRecord(record) => FieldSource::AnonymousRecord(record.clone()),
        }
    }
}

impl From<syntax::FieldDeclaration> for FieldSource {
    fn from(field: syntax::FieldDeclaration) -> Self {
        Self::Named(field)
    }
}
