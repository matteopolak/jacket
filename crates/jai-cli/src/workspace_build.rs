//! Resolve compiler recipes and emit planned native artifacts.
mod planning;
mod settings;
use crate::{Error, Output, backend};
use jai_codegen::target::{NativeTarget, TargetOptions};
use jai_driver::{CompilerSession, SchedulerLimits, SchedulerOptions, WorkspaceScheduler};
use jai_vm::{Limits, MessageLevel, TargetTriple};
use planning::{ArtifactFormat, plan};
use settings::seed_runtime_settings;
use std::{
    io::Write,
    path::{Path, PathBuf},
};

pub enum ArtifactCommand {
    Llvm(Output),
    Object(PathBuf),
    Executable(PathBuf),
}
impl ArtifactCommand {
    fn output(&self) -> Option<&Path> {
        match self {
            Self::Llvm(Output::Stdout) => None,
            Self::Llvm(Output::File(path)) | Self::Object(path) | Self::Executable(path) => {
                Some(path)
            }
        }
    }
    fn child_output(&self, source: &Path, name: &str, id: u64) -> PathBuf {
        let suffix: String = name
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
            .collect();
        let suffix = if suffix.is_empty() {
            format!("workspace-{id}")
        } else {
            suffix
        };
        let mut path = self
            .output()
            .map(Path::to_owned)
            .unwrap_or_else(|| source.with_extension("ll"));
        let mut filename = path.file_name().unwrap_or_default().to_os_string();
        filename.push(".");
        filename.push(suffix);
        path.set_file_name(filename);
        path
    }
}

pub fn run(
    source: &Path,
    graph_options: jai_driver::modules::GraphOptions,
    bootstrap: jai_driver::modules::BootstrapOptions,
    compile_time_limits: Limits,
    options: &TargetOptions,
    selected: &NativeTarget,
    command: ArtifactCommand,
) -> Result<(), Error> {
    let source = source.canonicalize().map_err(|cause| Error::Io {
        path: source.into(),
        cause,
    })?;
    let triple = selected.triple.as_str().to_string_lossy();
    let scheduler_options = SchedulerOptions {
        target: selected
            .build_target()
            .map_err(|error| Error::Source(error.to_string()))?,
        target_triple: TargetTriple::parse(&triple)
            .map_err(|error| Error::Source(error.to_string()))?,
        compile_time_limits,
        limits: SchedulerLimits::default(),
        replay_limits: jai_driver::ReplayLimits::default(),
    };
    let mut session = CompilerSession::new();
    seed_runtime_settings(&mut session, &bootstrap)?;
    let mut scheduler = WorkspaceScheduler::new_with_bootstrap(
        &source,
        graph_options,
        bootstrap,
        scheduler_options,
    )
    .map_err(|error| Error::Source(error.to_string()))?;
    let scheduled = scheduler.resolve(&mut session);
    for output in session.take_outputs() {
        match output.stream {
            jai_vm::CompilerOutputStream::StandardOutput => {
                let mut stream = std::io::stdout().lock();
                stream.write_all(&output.bytes).map_err(Error::OutputIo)?;
                stream.flush()
            }
            jai_vm::CompilerOutputStream::StandardError => {
                let mut stream = std::io::stderr().lock();
                stream.write_all(&output.bytes).map_err(Error::OutputIo)?;
                stream.flush()
            }
        }
        .map_err(Error::OutputIo)?;
    }
    for message in session.take_messages() {
        let level = match message.level {
            MessageLevel::Info => "info",
            MessageLevel::Warning => "warning",
            MessageLevel::Error => continue,
        };
        if let Some(location) = message.location {
            eprintln!(
                "{}:{}:{}: {level}: {}",
                location.path.display(),
                location.line,
                location.column,
                message.text
            );
        } else {
            eprintln!("{level}: {}", message.text);
        }
    }
    let scheduled = scheduled.map_err(|error| Error::Source(error.to_string()))?;
    for workspace in &scheduled.workspaces {
        if let jai_driver::WorkspaceOutput::Checked { library, .. } = &workspace.output {
            crate::source_warnings::emit(library);
        }
    }
    let artifacts = plan(&source, scheduled, options, &command, &session)?;
    for artifact in artifacts {
        match artifact.format {
            ArtifactFormat::Llvm => {
                let ir = backend::emit_unit_llvm(&artifact.unit, &artifact.target)?;
                match artifact.output {
                    Output::Stdout => print!("{ir}"),
                    Output::File(path) => {
                        backend::write_llvm(&path, &ir)?;
                    }
                }
            }
            ArtifactFormat::Object | ArtifactFormat::Executable => {
                let Output::File(path) = artifact.output else {
                    unreachable!("native artifacts have paths")
                };
                if matches!(artifact.format, ArtifactFormat::Object) {
                    backend::emit_unit_object(&artifact.unit, &artifact.target, &path)?;
                    println!(
                        "wrote object {} (workspace {})",
                        path.display(),
                        artifact.workspace.get()
                    );
                } else {
                    let backend::NativeUnit::Application(program) = &artifact.unit else {
                        unreachable!("executable has checked entry")
                    };
                    backend::build(program, &artifact.target, &path)?;
                    println!(
                        "built {} (workspace {})",
                        path.display(),
                        artifact.workspace.get()
                    );
                }
            }
            ArtifactFormat::DynamicLibrary | ArtifactFormat::StaticLibrary => {
                let Output::File(path) = artifact.output else {
                    unreachable!("native libraries have paths")
                };
                let kind = match artifact.format {
                    ArtifactFormat::DynamicLibrary => backend::LibraryKind::Dynamic,
                    ArtifactFormat::StaticLibrary => backend::LibraryKind::Static,
                    _ => unreachable!(),
                };
                backend::build_library(&artifact.unit, &artifact.target, &path, kind)?;
                println!(
                    "built library {} (workspace {})",
                    path.display(),
                    artifact.workspace.get()
                );
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unnamed_child_paths_and_names_cannot_escape_output_directory() {
        let command = ArtifactCommand::Executable("out/program".into());
        assert_eq!(
            command.child_output(Path::new("main.jai"), "../child", 2),
            PathBuf::from("out/program.child")
        );
        assert_eq!(
            command.child_output(Path::new("main.jai"), "", 2),
            PathBuf::from("out/program.workspace-2")
        );
    }
}
