//! Staged finalizer adapter; register with the external global storage variant.
//! Joint registration also requires the sealed checked-source-owner ledger lookup.
use super::{ExternalDataError, ExternalDataId, ExternalDataSource};
use crate::{IrError, Library};
use std::collections::HashMap;

/// Run after shared ledger validation proves canonical libraries and source owners.
pub(crate) fn verify_library(library: &Library) -> Result<(), ExternalDataError> {
    let mut identities = HashMap::new();
    let canonical_libraries: HashMap<_, _> = library
        .foreign_libraries()
        .iter()
        .map(|binding| (binding.id, binding))
        .collect();
    let exported_globals: HashMap<_, _> = library
        .program_exports()
        .iter()
        .filter_map(|export| {
            if let crate::ExportTarget::Global(global) = export.target {
                Some((global, export.symbol.as_str()))
            } else {
                None
            }
        })
        .collect();
    for global in library.globals() {
        let crate::GlobalInitializer::External(data) = global.initializer() else {
            continue;
        };
        data.validate(library.types())?;
        if data.ty() != global.ty() {
            return Err(ExternalDataError::Type(IrError::TypeMismatch {
                expected: global.ty(),
                actual: data.ty(),
            }));
        }
        if let ExternalDataId::Local { procedure, .. } = data.id()
            && library.procedure_by_id(procedure).is_none()
            && library.source_procedure_owners().get(procedure).is_none()
        {
            return Err(ExternalDataError::UnknownProcedure(procedure));
        }
        if let ExternalDataSource::Library(binding) = data.source()
            && canonical_libraries.get(&binding.id).copied() != Some(binding)
        {
            return Err(ExternalDataError::LibraryIdentity(binding.id));
        }
        if let Some(first) = identities.insert(data.id(), global.id()) {
            return Err(ExternalDataError::DuplicateIdentity {
                identity: data.id(),
                first,
                second: global.id(),
            });
        }
        if let Some(exported) = exported_globals.get(&global.id())
            && *exported != data.symbol()
        {
            return Err(ExternalDataError::ExportAlias {
                global: global.id(),
                declared: data.symbol().into(),
                exported: (*exported).into(),
            });
        }
    }
    Ok(())
}
