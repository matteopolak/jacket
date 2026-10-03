//! Validate workspace entry and output policy before writing artifacts.
use super::ArtifactCommand;
use crate::{Error, Output, backend};
use jai_codegen::target::{NativeTarget, TargetOptions};
use jai_driver::{BuildSetting, CompilerSession, ScheduledBuild, WorkspaceOutput};
use std::{collections::HashSet, path::Path};

#[derive(Clone, Copy)]
pub(super) enum ArtifactFormat {
    Llvm,
    Object,
    Executable,
    DynamicLibrary,
    StaticLibrary,
}
pub(super) struct Artifact {
    pub(super) workspace: jai_vm::WorkspaceId,
    pub(super) unit: backend::NativeUnit,
    pub(super) format: ArtifactFormat,
    pub(super) target: NativeTarget,
    pub(super) output: Output,
}

pub(super) fn plan(
    source: &Path,
    scheduled: ScheduledBuild,
    options: &TargetOptions,
    command: &ArtifactCommand,
    session: &CompilerSession,
) -> Result<Vec<Artifact>, Error> {
    // The scheduler omits retired workspaces. An empty completed session is a
    // successful recipe and must not select or overwrite a native artifact.
    if scheduled.workspaces.is_empty() {
        return Ok(vec![]);
    }
    let mut artifacts = vec![];
    let mut disabled = 0;
    let mut inputs = HashSet::new();
    for workspace in scheduled.workspaces {
        let WorkspaceOutput::Checked {
            unit,
            library,
            settings,
        } = workspace.output
        else {
            return Err(Error::Source(format!(
                "workspace {} ('{}') is awaiting source inputs; no artifact was emitted",
                workspace.id.get(),
                workspace.name
            )));
        };
        inputs.extend(unit.sources().iter().map(|source| source.path().to_owned()));
        let (unit, format) = match settings.output_kind {
            jai_types::BuildOutputKind::None => {
                disabled += 1;
                continue;
            }
            jai_types::BuildOutputKind::DynamicLibrary
            | jai_types::BuildOutputKind::StaticLibrary => {
                backend::validate_exports(&library).map_err(|error| {
                    setting_error(session, workspace.id, BuildSetting::OutputKind, error)
                })?;
                let format = match command {
                    ArtifactCommand::Llvm(_) => ArtifactFormat::Llvm,
                    ArtifactCommand::Object(_) => ArtifactFormat::Object,
                    ArtifactCommand::Executable(_)
                        if settings.output_kind == jai_types::BuildOutputKind::DynamicLibrary =>
                    {
                        ArtifactFormat::DynamicLibrary
                    }
                    ArtifactCommand::Executable(_) => ArtifactFormat::StaticLibrary,
                };
                (backend::NativeUnit::Library(*library), format)
            }
            jai_types::BuildOutputKind::Object => (
                backend::NativeUnit::Library(*library),
                if matches!(command, ArtifactCommand::Llvm(_)) {
                    ArtifactFormat::Llvm
                } else {
                    ArtifactFormat::Object
                },
            ),
            jai_types::BuildOutputKind::Executable => {
                let entry = jai_sema::select_entry(unit.graph(), &library)
                    .map_err(|error| Error::Source(error.render(unit.graph().sources())))?;
                match entry {
                    Some(entry) => {
                        let program = (*library)
                            .into_program(entry)
                            .map_err(|error| Error::Source(error.to_string()))?;
                        (
                            backend::NativeUnit::Application(program),
                            match command {
                                ArtifactCommand::Llvm(_) => ArtifactFormat::Llvm,
                                ArtifactCommand::Object(_) => ArtifactFormat::Object,
                                ArtifactCommand::Executable(_) => ArtifactFormat::Executable,
                            },
                        )
                    }
                    None if !library.program_exports().is_empty()
                        && !matches!(command, ArtifactCommand::Executable(_)) =>
                    {
                        backend::validate_exports(&library).map_err(|error| {
                            setting_error(session, workspace.id, BuildSetting::OutputKind, error)
                        })?;
                        (
                            backend::NativeUnit::Library(*library),
                            if matches!(command, ArtifactCommand::Llvm(_)) {
                                ArtifactFormat::Llvm
                            } else {
                                ArtifactFormat::Object
                            },
                        )
                    }
                    None if workspace.id == scheduled.root => continue,
                    None => {
                        return Err(Error::Source(format!(
                            "workspace {} ('{}') has no application main procedure; no artifact was emitted",
                            workspace.id.get(),
                            workspace.name
                        )));
                    }
                }
            }
        };
        let mut workspace_options = options.clone();
        if settings.bitcode != jai_types::BitcodeOptimization::Unset {
            workspace_options.optimization.bitcode = settings.bitcode;
        }
        if settings.machine != jai_types::MachineOptimization::Unset {
            workspace_options.optimization.machine = settings.machine;
        }
        let target = NativeTarget::select(&workspace_options)
            .map_err(|error| Error::Source(error.to_string()))?;
        if matches!(format, ArtifactFormat::Executable) && !target.is_host() {
            return Err(Error::Arguments(
                "cross-target executable linking is unsupported; use emit-object with a target triple",
            ));
        }
        match format {
            ArtifactFormat::DynamicLibrary => {
                backend::validate_library_target(&target, backend::LibraryKind::Dynamic).map_err(
                    |error| setting_error(session, workspace.id, BuildSetting::OutputKind, error),
                )?;
                crate::native_tools::compiler()?;
            }
            ArtifactFormat::StaticLibrary => {
                crate::native_tools::archiver()?;
            }
            ArtifactFormat::Executable => {
                crate::native_tools::compiler()?;
            }
            ArtifactFormat::Llvm | ArtifactFormat::Object => {}
        }
        let output = match settings.output_path {
            Some(path) => Output::File(if path.is_absolute() {
                path
            } else {
                source.parent().expect("canonical source parent").join(path)
            }),
            None if workspace.id == scheduled.root => command
                .output()
                .map_or(Output::Stdout, |path| Output::File(path.into())),
            None => Output::File(command.child_output(source, &workspace.name, workspace.id.get())),
        };
        artifacts.push(Artifact {
            workspace: workspace.id,
            unit,
            format,
            target,
            output,
        });
    }
    if artifacts.is_empty() && disabled > 0 {
        eprintln!("checked {disabled} workspace(s); native output disabled");
        return Ok(artifacts);
    }
    if artifacts.is_empty() {
        return Err(Error::Source(
            "no application main procedure in any workspace; no artifact was emitted".into(),
        ));
    }
    let mut destinations = HashSet::new();
    for artifact in &artifacts {
        if let Output::File(path) = &artifact.output {
            let parent = path
                .parent()
                .filter(|parent| !parent.as_os_str().is_empty())
                .unwrap_or(Path::new("."));
            let parent = parent.canonicalize().map_err(|cause| Error::Io {
                path: parent.into(),
                cause,
            })?;
            let resolved = parent.join(
                path.file_name()
                    .ok_or(Error::Arguments("artifact output requires a filename"))?,
            );
            crate::native_tools::validate_output(&resolved).map_err(|error| {
                setting_error(session, artifact.workspace, BuildSetting::OutputPath, error)
            })?;
            if inputs.contains(&resolved) || resolved == source {
                return Err(setting_error(
                    session,
                    artifact.workspace,
                    BuildSetting::OutputPath,
                    Error::Source(
                        "artifact output would overwrite its source input; no artifact was emitted"
                            .into(),
                    ),
                ));
            }
            if !destinations.insert(resolved.clone()) {
                return Err(setting_error(
                    session,
                    artifact.workspace,
                    BuildSetting::OutputPath,
                    Error::Source(format!(
                        "multiple workspaces request output {}; no artifact was emitted",
                        resolved.display()
                    )),
                ));
            }
        }
    }
    Ok(artifacts)
}

fn setting_error(
    session: &CompilerSession,
    workspace: jai_vm::WorkspaceId,
    setting: BuildSetting,
    error: Error,
) -> Error {
    if let Some(location) = session
        .workspace(workspace)
        .and_then(|workspace| workspace.option_origin(setting))
    {
        Error::Source(format!(
            "{}:{}:{}: error: {error}",
            location.path.display(),
            location.line,
            location.column
        ))
    } else {
        error
    }
}
