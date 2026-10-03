//! Object emission and guarded host linking; no textual IR compilation subprocess.
mod libraries;
use crate::Error;
use jai_codegen::{
    optimization::Optimization,
    target::{Cpu, CpuName, Features, NativeTarget, TargetOptions, TargetSelection, Triple},
};
pub use libraries::{LibraryKind, build_library, validate_exports, validate_library_target};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};

pub fn target_options(args: &[std::ffi::OsString]) -> Result<TargetOptions, Error> {
    let mut options = TargetOptions::default();
    if let Some(value) = env::var_os("JAI_RS_TARGET") {
        options.selection =
            TargetSelection::Triple(Triple::new(text(&value)?).map_err(target_error)?);
    }
    if let Some(value) = env::var_os("JAI_RS_CPU") {
        options.cpu = cpu(text(&value)?)?;
    }
    if let Some(value) = env::var_os("JAI_RS_FEATURES") {
        options.features = Features::new(text(&value)?).map_err(target_error)?;
    }
    if let Some(value) = env::var_os("JAI_RS_OPT") {
        options.optimization = optimization(text(&value)?)?;
    }
    if let Some(value) = env::var_os("JAI_RS_DEBUG") {
        options.debug = match text(&value)? {
            "off" | "0" => jai_codegen::debug::DebugInformation::Off,
            "line-tables" | "1" => jai_codegen::debug::DebugInformation::LineTables,
            "variables" | "2" => jai_codegen::debug::DebugInformation::Variables,
            _ => {
                return Err(Error::Arguments(
                    "JAI_RS_DEBUG must be off, 0, line-tables, 1, variables or 2",
                ));
            }
        };
    }
    let mut i = 0;
    while i < args.len() {
        let flag = text(&args[i])?;
        if matches!(flag, "-g" | "-gline-tables-only" | "-g0") {
            options.debug = if flag == "-g0" {
                jai_codegen::debug::DebugInformation::Off
            } else if flag == "-gline-tables-only" {
                jai_codegen::debug::DebugInformation::LineTables
            } else {
                jai_codegen::debug::DebugInformation::Variables
            };
            i += 1;
            continue;
        }
        if matches!(flag, "-O0" | "-O1" | "-O2" | "-O3" | "-Os" | "-Oz") {
            options.optimization = optimization(&flag[2..])?;
            i += 1;
            continue;
        }
        let value = args
            .get(i + 1)
            .ok_or(Error::Arguments("missing native option value"))?;
        match flag {
            "--target" => {
                options.selection =
                    TargetSelection::Triple(Triple::new(text(value)?).map_err(target_error)?)
            }
            "--cpu" => options.cpu = cpu(text(value)?)?,
            "--features" => options.features = Features::new(text(value)?).map_err(target_error)?,
            _ => {
                return Err(Error::Arguments(
                    "unknown native option; expected --target, --cpu, --features or -O0/-O1/-O2/-O3/-Os/-Oz",
                ));
            }
        }
        i += 2;
    }
    Ok(options)
}
fn text(value: &std::ffi::OsStr) -> Result<&str, Error> {
    value
        .to_str()
        .ok_or(Error::Arguments("native options must contain UTF-8 text"))
}
fn target_error(error: jai_codegen::target::Error) -> Error {
    Error::Source(error.to_string())
}
fn cpu(value: &str) -> Result<Cpu, Error> {
    Ok(match value {
        "generic" => Cpu::Generic,
        "native" => Cpu::Host,
        _ => Cpu::Named(CpuName::new(value).map_err(target_error)?),
    })
}
fn optimization(value: &str) -> Result<Optimization, Error> {
    use jai_codegen::optimization::BitcodeOptimization as Level;
    let bitcode = match value {
        "0" | "O0" => Level::O0,
        "1" | "O1" => Level::O1,
        "2" | "O2" => Level::O2,
        "3" | "O3" => Level::O3,
        "s" | "Os" => Level::Os,
        "z" | "Oz" => Level::Oz,
        _ => return Err(Error::Arguments("optimization must be 0, 1, 2, 3, s or z")),
    };
    Ok(Optimization {
        bitcode,
        ..Default::default()
    })
}

pub fn build(
    program: &jai_sema::Program,
    target: &NativeTarget,
    output: &Path,
) -> Result<(), Error> {
    if !target.is_host() {
        return Err(Error::Arguments(
            "cross-target executable linking is unsupported; use emit-object with a target triple",
        ));
    }
    let reachable =
        jai_codegen::native_reachability::Reachable::executable(program.library(), program.entry())
            .map_err(|error| Error::Source(error.to_string()))?;
    let tool = crate::native_tools::compiler()?;
    let scratch = Scratch::new()?;
    let object = scratch.0.join("program.o");
    emit_object(program, target, &object)?;
    let mut command = Command::new(tool);
    let linked = scratch.0.join("program");
    command.arg(&object).arg("-o").arg(&linked);
    if target
        .build_target()
        .map_err(|error| Error::Source(error.to_string()))?
        .operating_system
        == jai_types::OperatingSystem::MacOS
    {
        command.arg("-Wl,-no_fixup_chains");
    }
    crate::native_tools::scrub_environment(&mut command);
    let _dependencies = link_dependencies(&mut command, program.library(), target, &reachable)?;
    let status = command.status().map_err(Error::BackendIo)?;
    if !status.success() {
        return Err(Error::BackendFailed);
    }
    publish_artifact(output, |path| {
        fs::copy(&linked, path).map_err(|cause| Error::Io {
            path: path.into(),
            cause,
        })?;
        Ok(())
    })
}
fn link_dependencies(
    command: &mut Command,
    library: &jai_sema::Library,
    target: &NativeTarget,
    reachable: &jai_codegen::native_reachability::Reachable,
) -> Result<Vec<crate::native_dependencies::VerifiedDependency>, Error> {
    let dependencies = crate::native_dependencies::prepare(library, target, reachable)?;
    let mut referenced: std::collections::HashSet<_> = library
        .prototypes()
        .iter()
        .filter(|prototype| reachable.contains(prototype.id))
        .filter_map(|prototype| match &prototype.origin {
            jai_sema::PrototypeOrigin::Foreign {
                library: Some(library),
                ..
            } => Some(library.id),
            _ => None,
        })
        .collect();
    referenced.extend(
        library
            .globals()
            .iter()
            .filter(|global| reachable.contains_global(global.id()))
            .filter_map(|global| match global.initializer() {
                jai_sema::GlobalInitializer::External(data) => match data.source() {
                    jai_sema::ExternalDataSource::Library(owner) => Some(owner.id),
                    jai_sema::ExternalDataSource::Program => None,
                },
                _ => None,
            }),
    );
    let triple = target.triple.as_str().to_string_lossy();
    for library in library.foreign_libraries() {
        if library.options.link_always || referenced.contains(&library.id) {
            if !crate::native_dependencies::apply(command, &dependencies, library.id) {
                command.args(crate::foreign_libraries::arguments(library, &triple)?);
            }
        }
    }
    if target.triple.as_str().to_string_lossy().contains("linux") {
        command.arg("-lm");
    }
    Ok(dependencies)
}
pub fn emit_object(
    program: &jai_sema::Program,
    target: &NativeTarget,
    output: &Path,
) -> Result<(), Error> {
    let context = jai_codegen::Context::create();
    let module = jai_codegen::lower_program_object_for_target(&context, program, target)
        .map_err(Error::Codegen)?;
    publish_artifact(output, |path| {
        target.write_object(&module, path).map_err(target_error)
    })
}
pub enum NativeUnit {
    Application(jai_sema::Program),
    Library(jai_sema::Library),
}
pub fn emit_unit_object(
    unit: &NativeUnit,
    target: &NativeTarget,
    output: &Path,
) -> Result<(), Error> {
    let context = jai_codegen::Context::create();
    let module = match unit {
        NativeUnit::Application(program) => {
            jai_codegen::lower_program_object_for_target(&context, program, target)
        }
        NativeUnit::Library(library) => {
            jai_codegen::lower_library_for_target(&context, library, &publication(library), target)
        }
    }
    .map_err(Error::Codegen)?;
    publish_artifact(output, |path| {
        target.write_object(&module, path).map_err(target_error)
    })
}
pub fn emit_unit_llvm(unit: &NativeUnit, target: &NativeTarget) -> Result<String, Error> {
    let context = jai_codegen::Context::create();
    let module = match unit {
        NativeUnit::Application(program) => {
            jai_codegen::lower_program_object_for_target(&context, program, target)
        }
        NativeUnit::Library(library) => {
            jai_codegen::lower_library_for_target(&context, library, &publication(library), target)
        }
    }
    .map_err(Error::Codegen)?;
    target.prepare(&module).map_err(target_error)?;
    Ok(module.print_to_string().to_string())
}
fn publication(library: &jai_sema::Library) -> jai_codegen::native_reachability::Publication {
    if library.program_exports().is_empty() {
        jai_codegen::native_reachability::Publication::AllBodies
    } else {
        jai_codegen::native_reachability::Publication::Selected(vec![])
    }
}

pub fn write_llvm(output: &Path, ir: &str) -> Result<(), Error> {
    publish_artifact(output, |path| {
        fs::write(path, ir).map_err(|cause| Error::Io {
            path: path.into(),
            cause,
        })
    })
}

fn publish_artifact(
    output: &Path,
    write: impl FnOnce(&Path) -> Result<(), Error>,
) -> Result<(), Error> {
    let parent = output
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let scratch = Scratch::in_directory(parent)?;
    let staged = scratch.0.join("artifact");
    write(&staged)?;
    fs::rename(&staged, output).map_err(|cause| Error::Io {
        path: output.into(),
        cause,
    })
}

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Result<Self, Error> {
        Self::in_directory(&env::temp_dir())
    }
    fn in_directory(parent: &Path) -> Result<Self, Error> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        for _ in 0..32 {
            let path = parent.join(format!(
                ".jai-rs-native-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Ok(Self(path)),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(cause) => {
                    return Err(Error::Io {
                        path,
                        cause,
                    });
                }
            }
        }
        Err(Error::Arguments(
            "could not reserve native object scratch directory",
        ))
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn args(values: &[&str]) -> Vec<std::ffi::OsString> {
        values.iter().map(std::ffi::OsString::from).collect()
    }
    #[test]
    fn explicit_target_cpu_features_and_optimization_are_typed() {
        let options = target_options(&args(&[
            "--target",
            "x86_64-unknown-linux-gnu",
            "--cpu",
            "generic",
            "--features",
            "+sse2,-avx",
            "-Oz",
        ]))
        .unwrap();
        assert_eq!(
            options.selection,
            TargetSelection::Triple(Triple::new("x86_64-unknown-linux-gnu").unwrap())
        );
        assert_eq!(options.cpu, Cpu::Generic);
        assert_eq!(options.features.as_str(), "+sse2,-avx");
        assert_eq!(
            options.optimization.bitcode,
            jai_codegen::optimization::BitcodeOptimization::Oz
        );
    }
    #[test]
    fn malformed_cli_native_configuration_fails_before_compilation() {
        for values in [
            &["--target"][..],
            &["--target", "macos"],
            &["--cpu", "contains space"],
            &["--features", "sse2"],
            &["-O9"],
            &["--other", "value"],
        ] {
            assert!(target_options(&args(values)).is_err(), "{values:?}");
        }
    }
    #[test]
    fn debug_flags_select_explicit_typed_information_modes() {
        let options = target_options(&args(&["-g", "-O0"])).unwrap();
        assert_eq!(
            options.debug,
            jai_codegen::debug::DebugInformation::Variables
        );
        let options = target_options(&args(&["-g", "-gline-tables-only"])).unwrap();
        assert_eq!(
            options.debug,
            jai_codegen::debug::DebugInformation::LineTables
        );
        let options = target_options(&args(&["-gline-tables-only", "-g0"])).unwrap();
        assert_eq!(options.debug, jai_codegen::debug::DebugInformation::Off);
    }
}
