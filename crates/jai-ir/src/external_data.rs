//! Checked native data declarations preserve their source linkage identity.
pub(crate) mod verify;
use crate::{ForeignLibrary, ForeignLibraryError, GlobalId, IrError, ProcedureId, storage};
use jai_source::{DeclarationId, SourceSpan};
use jai_types::{IntegerType, ScalarType, TypeId, TypeKind, TypeView};
use std::fmt;

/// Source identity stays distinct from the program's dense `GlobalId` storage slot.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LocalExternalDataIndex(usize);
impl LocalExternalDataIndex {
    pub const fn new(index: usize) -> Self {
        Self(index)
    }
    pub const fn index(self) -> usize {
        self.0
    }
}

/// A local declaration uses its actual append-only lexical registration index.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExternalDataId {
    File(DeclarationId),
    Local {
        procedure: ProcedureId,
        index: LocalExternalDataIndex,
    },
}

/// Bare `#elsewhere` names a program symbol, not a fabricated library declaration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExternalDataSource {
    Program,
    Library(ForeignLibrary),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StorageKind {
    Integer(IntegerType),
    Bool,
    Value,
}

/// An external declaration has no initial value and grants no host read capability.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExternalData {
    id: ExternalDataId,
    ty: TypeId,
    source: ExternalDataSource,
    symbol: String,
    location: SourceSpan,
    storage: StorageKind,
}

#[derive(Debug)]
pub enum ExternalDataError {
    InvalidSymbol,
    InvalidLocation,
    Library(ForeignLibraryError),
    Type(IrError),
    UnknownProcedure(ProcedureId),
    LibraryIdentity(crate::ForeignLibraryId),
    DuplicateIdentity {
        identity: ExternalDataId,
        first: GlobalId,
        second: GlobalId,
    },
    ExportAlias {
        global: GlobalId,
        declared: String,
        exported: String,
    },
}
impl fmt::Display for ExternalDataError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSymbol => {
                formatter.write_str("external data symbol must be nonempty and contain no NUL")
            }
            Self::InvalidLocation => {
                formatter.write_str("external data source span has an invalid extent")
            }
            Self::Library(error) => error.fmt(formatter),
            Self::Type(error) => error.fmt(formatter),
            Self::UnknownProcedure(id) => write!(
                formatter,
                "external data has unknown local procedure owner {id:?}"
            ),
            Self::LibraryIdentity(id) => write!(
                formatter,
                "external data library {id:?} differs from the canonical library ledger"
            ),
            Self::DuplicateIdentity {
                identity,
                first,
                second,
            } => write!(
                formatter,
                "external data source identity {identity:?} has multiple storage slots ({first:?}, {second:?})"
            ),
            Self::ExportAlias {
                global,
                declared,
                exported,
            } => write!(
                formatter,
                "external global {global:?} names {declared:?}; exporting it as {exported:?} requires an unsupported native data alias"
            ),
        }
    }
}

impl std::error::Error for ExternalDataError {
}

impl ExternalData {
    pub fn new(
        id: ExternalDataId,
        ty: TypeId,
        source: ExternalDataSource,
        symbol: String,
        location: SourceSpan,
        types: &dyn TypeView,
    ) -> Result<Self, ExternalDataError> {
        if symbol.is_empty() || symbol.as_bytes().contains(&0) {
            return Err(ExternalDataError::InvalidSymbol);
        }
        if location.span.start > location.span.end {
            return Err(ExternalDataError::InvalidLocation);
        }
        if let ExternalDataSource::Library(library) = &source {
            library.validate().map_err(ExternalDataError::Library)?;
        }
        storage::runtime_type(types, ty).map_err(ExternalDataError::Type)?;
        let storage = match types
            .kind(ty)
            .map_err(IrError::from)
            .map_err(ExternalDataError::Type)?
        {
            TypeKind::Integer(integer) => StorageKind::Integer(*integer),
            TypeKind::Bool => StorageKind::Bool,
            _ => StorageKind::Value,
        };
        Ok(Self {
            id,
            ty,
            source,
            symbol,
            location,
            storage,
        })
    }
    pub fn id(&self) -> ExternalDataId {
        self.id
    }
    pub fn ty(&self) -> TypeId {
        self.ty
    }
    pub fn source(&self) -> &ExternalDataSource {
        &self.source
    }
    pub fn symbol(&self) -> &str {
        &self.symbol
    }
    pub fn location(&self) -> SourceSpan {
        self.location
    }

    /// Recheck the declaration against the canonical frozen type view.
    pub fn validate(&self, types: &dyn TypeView) -> Result<(), ExternalDataError> {
        let checked = Self::new(
            self.id,
            self.ty,
            self.source.clone(),
            self.symbol.clone(),
            self.location,
            types,
        )?;
        if checked.storage != self.storage {
            return Err(ExternalDataError::Type(IrError::InvalidValue(self.ty)));
        }
        Ok(())
    }

    /// Canonical scalar storage shape; aggregate declarations return `None`.
    pub fn scalar(&self) -> Option<ScalarType> {
        match self.storage {
            StorageKind::Integer(integer) => Some(ScalarType::Int(integer)),
            StorageKind::Bool => Some(ScalarType::Bool),
            StorageKind::Value => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_source::{Identities, SourceMap, Span};
    use jai_types::{RecordKind, TypeRegistry};

    fn origin() -> (ExternalDataId, SourceSpan) {
        let mut identities = Identities::default();
        let mut sources = SourceMap::default();
        let source = sources.insert("owned.jai".into(), "counter:s64 #elsewhere;".into());
        (
            ExternalDataId::File(identities.declaration()),
            SourceSpan {
                source,
                span: Span {
                    start: 0,
                    end: 22,
                },
            },
        )
    }

    #[test]
    fn program_binding_preserves_actual_origin_without_library_identity() {
        let types = TypeRegistry::new();
        let (id, location) = origin();
        let ty = types.scalar(ScalarType::Int(IntegerType::S64));
        let data = ExternalData::new(
            id,
            ty,
            ExternalDataSource::Program,
            "counter".into(),
            location,
            &types,
        )
        .unwrap();
        assert_eq!(data.id(), id);
        assert_eq!(data.location(), location);
        assert_eq!(data.source(), &ExternalDataSource::Program);
        assert_eq!(data.scalar(), Some(ScalarType::Int(IntegerType::S64)));
        assert!(data.validate(&TypeRegistry::new()).is_err());
    }

    #[test]
    fn malformed_native_symbols_and_source_extents_are_rejected() {
        let types = TypeRegistry::new();
        let (id, location) = origin();
        let ty = types.scalar(ScalarType::Int(IntegerType::S64));
        for symbol in ["", "counter\0alias"] {
            assert!(matches!(
                ExternalData::new(
                    id,
                    ty,
                    ExternalDataSource::Program,
                    symbol.into(),
                    location,
                    &types
                ),
                Err(ExternalDataError::InvalidSymbol)
            ));
        }
        let invalid = SourceSpan {
            span: Span {
                start: 22,
                end: 0,
            },
            ..location
        };
        assert!(matches!(
            ExternalData::new(
                id,
                ty,
                ExternalDataSource::Program,
                "counter".into(),
                invalid,
                &types
            ),
            Err(ExternalDataError::InvalidLocation)
        ));
    }

    #[test]
    fn aggregate_storage_requires_its_complete_canonical_record() {
        let mut types = TypeRegistry::new();
        let (id, location) = origin();
        let record = types.reserve_record(RecordKind::Struct);
        assert!(
            ExternalData::new(
                id,
                record,
                ExternalDataSource::Program,
                "state".into(),
                location,
                &types
            )
            .is_err()
        );
        let total = types.scalar(ScalarType::Int(IntegerType::S64));
        let status = types.scalar(ScalarType::Int(IntegerType::S32));
        types.define_record(record, [total, status]).unwrap();
        let data = ExternalData::new(
            id,
            record,
            ExternalDataSource::Program,
            "state".into(),
            location,
            &types,
        )
        .unwrap();
        assert_eq!(data.ty(), record);
        assert_eq!(data.scalar(), None);
        data.validate(&types.freeze().unwrap()).unwrap();
    }
}
