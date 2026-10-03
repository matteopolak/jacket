//! Source spellings attach to actual nominal type and opaque field identities.
use super::{DebugSourceLocation, DebugSources};
use jai_types::{FieldId, TypeId, TypeKind, TypeView};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeSource {
    /// Anonymous source records have no invented type name.
    pub name: Option<String>,
    pub location: DebugSourceLocation,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldSource {
    pub name: String,
    pub location: DebugSourceLocation,
}

#[derive(Clone, Debug, Default)]
pub(super) struct TypeProvenance {
    types: HashMap<TypeId, TypeSource>,
    fields: HashMap<FieldId, FieldSource>,
}

impl DebugSources {
    pub fn insert_type_source(&mut self, ty: TypeId, source: TypeSource) -> Option<TypeSource> {
        self.type_provenance.types.insert(ty, source)
    }
    pub fn type_source(&self, ty: TypeId) -> Option<&TypeSource> {
        self.type_provenance.types.get(&ty)
    }
    pub fn type_sources(&self) -> impl Iterator<Item = (TypeId, &TypeSource)> {
        self.type_provenance
            .types
            .iter()
            .map(|(ty, source)| (*ty, source))
    }
    pub fn insert_field_source(&mut self, id: FieldId, source: FieldSource) -> Option<FieldSource> {
        self.type_provenance.fields.insert(id, source)
    }
    pub fn field_source(&self, id: FieldId) -> Option<&FieldSource> {
        self.type_provenance.fields.get(&id)
    }
    pub fn field_sources(&self) -> impl Iterator<Item = (FieldId, &FieldSource)> {
        self.type_provenance
            .fields
            .iter()
            .map(|(id, source)| (*id, source))
    }
    pub(crate) fn validate_type_sources(&self, types: &dyn TypeView) -> Result<(), crate::IrError> {
        for (ty, source) in self.type_sources() {
            // Only distinct nominal identities carry declaration names. An
            // alias of a primitive must not rename that shared primitive ID.
            match types.kind(ty)? {
                TypeKind::Record(id) => {
                    types.record(*id)?;
                }
                TypeKind::Enum(id) => {
                    types.enumeration(*id)?;
                }
                TypeKind::Distinct(id) => {
                    types.distinct(*id)?;
                }
                _ => return Err(jai_types::TypeError::WrongKind(ty).into()),
            }
            self.validate_location(&source.location)?;
            if source.name.as_ref().is_some_and(|name| invalid_name(name)) {
                return Err(crate::IrError::UnknownIdentity {
                    kind: "debug type name",
                    index: ty.index(),
                });
            }
        }
        for (id, source) in self.field_sources() {
            // FieldId contains its nominal record arena and ordinal; the
            // registry checks both before returning its actual storage type.
            types.field_type(id)?;
            self.validate_location(&source.location)?;
            if invalid_name(&source.name) {
                return Err(crate::IrError::UnknownIdentity {
                    kind: "debug field name",
                    index: id.index(),
                });
            }
        }
        Ok(())
    }
}

fn invalid_name(name: &str) -> bool {
    name.is_empty() || name.as_bytes().contains(&0)
}
