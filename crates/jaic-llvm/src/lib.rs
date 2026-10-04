//! LLVM backend for the jaic compiler core.
//!
//! [`emit_object`] lowers an `ir::Program` to LLVM IR and writes a native
//! object file; [`link`] turns object files into an executable with the
//! system C compiler driver.
mod abi;
mod lower;

use inkwell::OptimizationLevel;
use inkwell::context::Context;
use inkwell::passes::PassBuilderOptions;
use inkwell::targets::{
    CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine, TargetTriple,
};
use jaic::ir::{Library, Program};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Optimization pipeline selection.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum OptLevel {
    #[default]
    O0,
    O1,
    O2,
    O3,
}

impl OptLevel {
    fn llvm(self) -> OptimizationLevel {
        match self {
            OptLevel::O0 => OptimizationLevel::None,
            OptLevel::O1 => OptimizationLevel::Less,
            OptLevel::O2 => OptimizationLevel::Default,
            OptLevel::O3 => OptimizationLevel::Aggressive,
        }
    }
    /// New-pass-manager pipeline string, `None` when no optimization runs.
    fn pipeline(self) -> Option<&'static str> {
        match self {
            OptLevel::O0 => None,
            OptLevel::O1 => Some("default<O1>"),
            OptLevel::O2 => Some("default<O2>"),
            OptLevel::O3 => Some("default<O3>"),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Options {
    pub opt_level: OptLevel,
    /// Target triple; `None` selects the host.
    pub target: Option<String>,
    /// Also write the textual LLVM IR (before optimization) to this path.
    pub emit_ir: Option<PathBuf>,
}

/// Translate `program` to a native object file at `path`.
pub fn emit_object(program: &Program, options: &Options, path: &Path) -> Result<(), String> {
    Target::initialize_all(&InitializationConfig::default());
    let host = options.target.is_none();
    let triple = match &options.target {
        Some(t) => TargetTriple::create(t),
        None => TargetMachine::get_default_triple(),
    };
    let triple_str = triple.as_str().to_string_lossy().into_owned();
    let arch = abi::Arch::from_triple(&triple_str)
        .ok_or_else(|| format!("unsupported target architecture in '{triple_str}'"))?;
    let target = Target::from_triple(&triple).map_err(|e| e.to_string())?;
    let (cpu, features) = if host {
        (
            TargetMachine::get_host_cpu_name().to_string(),
            TargetMachine::get_host_cpu_features().to_string(),
        )
    } else {
        ("generic".to_string(), String::new())
    };
    let machine = target
        .create_target_machine(
            &triple,
            &cpu,
            &features,
            options.opt_level.llvm(),
            RelocMode::PIC,
            CodeModel::Default,
        )
        .ok_or("could not create a target machine")?;

    let context = Context::create();
    let module = context.create_module("jai");
    module.set_triple(&triple);
    module.set_data_layout(&machine.get_target_data().get_data_layout());
    lower::lower_program(&context, &module, program, arch)?;
    if let Some(ir_path) = &options.emit_ir {
        module.print_to_file(ir_path).map_err(|e| e.to_string())?;
    }
    module
        .verify()
        .map_err(|e| format!("invalid LLVM IR: {e}"))?;
    if let Some(pipeline) = options.opt_level.pipeline() {
        module
            .run_passes(pipeline, &machine, PassBuilderOptions::create())
            .map_err(|e| e.to_string())?;
    }
    machine
        .write_to_file(&module, FileType::Object, path)
        .map_err(|e| e.to_string())
}

/// The libraries from `program.libraries` that some foreign symbol uses.
pub fn used_libraries(program: &Program) -> Vec<Library> {
    let mut used = vec![false; program.libraries.len()];
    for foreign in &program.foreigns {
        if let Some(i) = foreign.library.filter(|&i| i < used.len()) {
            used[i] = true;
        }
    }
    program
        .libraries
        .iter()
        .zip(used)
        .filter(|&(_, u)| u)
        .map(|(l, _)| l.clone())
        .collect()
}

/// Link object files into an executable with the system `cc`.
pub fn link(
    objects: &[PathBuf],
    libraries: &[Library],
    output: &Path,
    dynamic_library: bool,
    extra_args: &[String],
) -> Result<(), String> {
    let mut cmd = Command::new("cc");
    if dynamic_library {
        cmd.arg("-shared");
    }
    cmd.args(objects).arg("-o").arg(output);
    let mut seen = Vec::new();
    for lib in libraries {
        for arg in library_args(lib) {
            if !seen.contains(&arg) {
                seen.push(arg);
            }
        }
    }
    cmd.args(&seen).args(extra_args);
    let out = cmd
        .output()
        .map_err(|e| format!("could not run the system linker 'cc': {e}"))?;
    if out.status.success() {
        Ok(())
    } else {
        Err(format!(
            "linking failed ({}):\n{}",
            out.status,
            String::from_utf8_lossy(&out.stderr)
        ))
    }
}

/// Linker arguments for one Jai library reference.
fn library_args(lib: &Library) -> Vec<String> {
    let name = lib.name.as_str();
    // libc and friends are always linked implicitly.
    if matches!(name, "c" | "libc") {
        return Vec::new();
    }
    if !lib.system {
        // A library shipped next to the source: link it by path when found.
        let path = Path::new(name);
        let file = path
            .file_name()
            .map(|f| f.to_string_lossy().into_owned())
            .unwrap_or_default();
        let dir = Path::new(&lib.base_dir).join(path.parent().unwrap_or(Path::new("")));
        let ext = if cfg!(target_os = "macos") {
            "dylib"
        } else {
            "so"
        };
        for candidate in [format!("{file}.{ext}"), format!("lib{file}.{ext}")] {
            let full = dir.join(candidate);
            if full.exists() {
                let parent = full.parent().unwrap_or(Path::new(".")).display();
                return vec![full.display().to_string(), format!("-Wl,-rpath,{parent}")];
            }
        }
    }
    let mut args = Vec::new();
    if cfg!(target_os = "macos") && Path::new("/opt/homebrew/lib").exists() {
        args.push("-L/opt/homebrew/lib".to_string());
    }
    let name = match name {
        "libm" => "m",
        "libpthread" => "pthread",
        "libdl" => "dl",
        other => other,
    };
    args.push(format!("-l{name}"));
    args
}
