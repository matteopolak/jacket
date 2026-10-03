//! Package newly emitted objects using the checked source export table.
use super::{
    NativeUnit, Scratch, emit_unit_object, link_dependencies, publication, publish_artifact,
};
use crate::Error;
use jai_codegen::{native_reachability::Reachable, target::NativeTarget};
use jai_sema::{ExportTarget, Library};
use jai_types::{CallingConvention, ContextMode, OperatingSystem};
use std::{ffi::OsString, fs, path::Path, process::Command};

#[derive(Clone, Copy)]
pub enum LibraryKind {
    Dynamic,
    Static,
}

pub fn validate_exports(library: &Library) -> Result<(), Error> {
    if library.program_exports().is_empty() {
        return Err(Error::Source("native library output requires explicit #program_export declarations; no artifact was emitted".into()));
    }
    for export in library.program_exports() {
        if let ExportTarget::Procedure(id) = export.target {
            let procedure = library.procedure_by_id(id).expect("checked export target");
            let signature = library
                .types()
                .procedure_definition(procedure.signature)
                .map_err(|error| Error::Source(error.to_string()))?;
            if signature.convention != CallingConvention::C
                || signature.context != ContextMode::None
            {
                return Err(Error::Source(format!(
                    "native library export {:?} must use #c_call without implicit context; no artifact was emitted",
                    export.symbol.as_str()
                )));
            }
        }
    }
    Ok(())
}

pub fn validate_library_target(target: &NativeTarget, kind: LibraryKind) -> Result<(), Error> {
    if matches!(kind, LibraryKind::Dynamic) {
        if !target.is_host() {
            return Err(Error::Arguments(
                "cross-target dynamic library linking is unsupported; use emit-object with a target triple",
            ));
        }
        let operating_system = target
            .build_target()
            .map_err(|error| Error::Source(error.to_string()))?
            .operating_system;
        if !matches!(
            operating_system,
            OperatingSystem::MacOS | OperatingSystem::Linux
        ) {
            return Err(Error::Arguments(
                "dynamic library linking is supported on macOS and Linux hosts",
            ));
        }
    }
    Ok(())
}

pub fn build_library(
    unit: &NativeUnit,
    target: &NativeTarget,
    output: &Path,
    kind: LibraryKind,
) -> Result<(), Error> {
    let NativeUnit::Library(library) = unit else {
        unreachable!("library packaging has a checked library")
    };
    validate_exports(library)?;
    validate_library_target(target, kind)?;
    let tool = match kind {
        LibraryKind::Dynamic => crate::native_tools::compiler()?,
        LibraryKind::Static => crate::native_tools::archiver()?,
    };
    let scratch = Scratch::new()?;
    let object = scratch.0.join("library.o");
    emit_unit_object(unit, target, &object)?;
    let staged = scratch.0.join("library");
    let mut command = Command::new(tool);
    let mut _dependencies = Vec::new();
    match kind {
        LibraryKind::Static => {
            command.arg("crs").arg(&staged).arg(&object);
        }
        LibraryKind::Dynamic => {
            let filename = output
                .file_name()
                .ok_or(Error::Arguments("library output requires a filename"))?;
            let operating_system = target
                .build_target()
                .map_err(|error| Error::Source(error.to_string()))?
                .operating_system;
            if operating_system == OperatingSystem::MacOS {
                let mut identity = OsString::from("@rpath/");
                identity.push(filename);
                command
                    .arg("-dynamiclib")
                    .arg("-Wl,-no_fixup_chains")
                    .arg("-Xlinker")
                    .arg("-install_name")
                    .arg("-Xlinker")
                    .arg(identity);
            } else {
                command
                    .arg("-shared")
                    .arg("-Xlinker")
                    .arg("-soname")
                    .arg("-Xlinker")
                    .arg(filename);
            }
            command.arg(&object).arg("-o").arg(&staged);
            let reachable = Reachable::library(library, &publication(library))
                .map_err(|error| Error::Source(error.to_string()))?;
            _dependencies = link_dependencies(&mut command, library, target, &reachable)?;
        }
    }
    crate::native_tools::scrub_environment(&mut command);
    if !command.status().map_err(Error::BackendIo)?.success() {
        return Err(Error::BackendFailed);
    }
    publish_artifact(output, |path| {
        fs::copy(&staged, path).map_err(|cause| Error::Io {
            path: path.into(),
            cause,
        })?;
        Ok(())
    })
}
