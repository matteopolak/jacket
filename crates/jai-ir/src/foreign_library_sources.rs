//! Actual source allocation witnesses survive freezing, independently of optional debug data.
use crate::{ForeignLibrary, ForeignLibraryId, IrError, SourceProcedureIdentity};
use std::collections::HashMap;

/// Compiler provenance records alone confer no native file or link authority.
#[derive(Clone, Debug, Default)]
pub struct ForeignLibrarySources(HashMap<ForeignLibraryId, SourceLibraryOrigin>);
#[derive(Clone, Debug)]
struct SourceLibraryOrigin {
    source: SourceProcedureIdentity,
    environment: Vec<u8>,
}
impl ForeignLibrarySources {
    pub fn from_source_records(
        records: impl IntoIterator<Item = (ForeignLibraryId, SourceProcedureIdentity, Vec<u8>)>,
    ) -> Result<Self, IrError> {
        let mut result = Self::default();
        for (id, source, environment) in records {
            if result
                .0
                .insert(
                    id,
                    SourceLibraryOrigin {
                        source,
                        environment,
                    },
                )
                .is_some()
            {
                return Err(IrError::DuplicateIdentity {
                    kind: "foreign library source",
                    index: id.index(),
                });
            }
        }
        Ok(result)
    }
    pub fn get(&self, id: ForeignLibraryId) -> Option<&SourceProcedureIdentity> {
        self.0.get(&id).map(|origin| &origin.source)
    }
    pub fn environment(&self, id: ForeignLibraryId) -> Option<&[u8]> {
        self.0.get(&id).map(|origin| origin.environment.as_slice())
    }
    pub(crate) fn validate(&self, libraries: &[ForeignLibrary]) -> Result<(), IrError> {
        for id in self.0.keys() {
            if !libraries.iter().any(|library| library.id == *id) {
                return Err(IrError::UnknownIdentity {
                    kind: "foreign library source",
                    index: id.index(),
                });
            }
        }
        Ok(())
    }
}
