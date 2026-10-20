//! Building workspaces and the LLVM output backend.
use super::cli::{Cli, Flavor};
use super::parse::shown;
use super::timings;
#[cfg(feature = "llvm")]
use super::tuning;
#[cfg(feature = "llvm")]
use jaic::build::OutputType;
use jaic::build::{BuildSettings, OutputBackend};
use jaic::builtin_keys::MachineCodeOptimization;
#[cfg(feature = "llvm")]
use jaic::builtin_keys::{BitcodeOptimization, WorkspaceFlag};
use jaic::sema::Compiler;
#[cfg(feature = "llvm")]
use jaic::sema::{Options, TargetCpu, TargetOs};
#[cfg(feature = "llvm")]
use jaic::target::Target;
use std::path::{Path, PathBuf};

/// Write the top-level program. `-exe`, `-output_path` and the optimization options override
/// what the metaprogram set.
pub(super) fn build(
    compiler: &mut Compiler,
    cli: &Cli,
    source: &Path,
    mut settings: BuildSettings,
) -> Result<(), String> {
    let name = cli.exe.clone().unwrap_or_else(|| {
        if settings.output_executable_name.is_empty() {
            source
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned()
        } else {
            settings.output_executable_name.clone()
        }
    });
    let dir = cli
        .output_path
        .clone()
        .unwrap_or_else(|| PathBuf::from(&settings.output_path));
    let output = dir.join(name);
    if let Some(level) = cli.opt_level.or(cli.flavor.map(Flavor::level)) {
        // `-O` and the flavors stand for a whole build: the metaprogram's code generation
        // switches go too.
        settings.optimization = level;
        settings.machine_code_optimization = MachineCodeOptimization::Unset;
        settings.llvm_switches.clear();
        settings.frame_pointers = None;
    }
    if settings.use_custom_link_command {
        // Nothing waits for the top-level program's messages, so nothing could link it.
        return Err(
            "`use_custom_link_command` is set for the top-level program, which no \
             metaprogram links\nhelp: set it on a workspace the metaprogram creates and link \
             on READY_FOR_CUSTOM_LINK_COMMAND"
                .into(),
        );
    }
    timings::time("prepare output", || compiler.prepare_compiled_output())
        .map_err(|error| compiler.render(&error))?;
    native_backend(cli).write_output(&compiler.out.program, &settings, &output)?;
    if cli.verbose {
        eprintln!("jaic: wrote {}", shown(&output));
    }
    Ok(())
}

#[cfg(feature = "llvm")]
pub(super) fn native_backend(cli: &Cli) -> LlvmBackend {
    LlvmBackend {
        emit_ir: cli.emit_ir.clone(),
        debug_info: !cli.no_debug_info,
        target: cli.build_target().ok().flatten(),
        explicit_target: cli.target.is_some() || cli.os.is_some() || cli.cpu.is_some(),
        sanitize: cli
            .sanitize
            .iter()
            .map(String::as_str)
            .chain(cli.coverage.then_some("coverage"))
            .collect::<Vec<_>>()
            .join(","),
        no_inline: cli.no_inline,
        no_split: cli.no_split,
        switches: cli.tuning.switches(),
    }
}

/// Without the `llvm` feature (an interpreter-only build, e.g. for a host without LLVM
/// libraries) a build reports that it cannot write native output.
#[cfg(not(feature = "llvm"))]
pub(super) fn native_backend(_cli: &Cli) -> NoBackend {
    NoBackend
}

#[cfg(not(feature = "llvm"))]
pub(super) struct NoBackend;

#[cfg(not(feature = "llvm"))]
impl OutputBackend for NoBackend {
    fn write_output(
        &mut self,
        _program: &jaic::ir::Program,
        _settings: &BuildSettings,
        _output: &Path,
    ) -> Result<(), String> {
        Err("this jaic was built without the `llvm` feature and cannot write native output".into())
    }
}

/// Native output through `jaic-llvm` and the system linker.
#[cfg(feature = "llvm")]
pub(super) struct LlvmBackend {
    emit_ir: Option<PathBuf>,
    debug_info: bool,
    /// What to build for; `None` for the host.
    pub(super) target: Option<Target>,
    /// An explicit host OS/CPU still overrides workspace output settings.
    explicit_target: bool,
    /// The `-sanitize` lists, comma-joined (empty: no sanitizers).
    sanitize: String,
    /// `-no_inline` and `-no_split`: they hold for every workspace this backend writes.
    no_inline: bool,
    no_split: bool,
    switches: tuning::CodegenSwitches,
}

/// What one write of a workspace's output needs, worked out from its settings.
#[cfg(feature = "llvm")]
struct Prepared {
    output: PathBuf,
    /// The first object file (`emit_objects` numbers the others after it).
    object: PathBuf,
    /// The objects stay after linking, for the debugger.
    keep_objects: bool,
    target: Option<Target>,
    options: jaic_llvm::Options,
    /// Linker arguments beyond the metaprogram's (`-mmacosx-version-min`).
    extra_link_args: Vec<String>,
}

#[cfg(feature = "llvm")]
impl LlvmBackend {
    pub(super) fn workspace_target(
        &self,
        settings: &BuildSettings,
    ) -> Result<Option<Target>, String> {
        if self.explicit_target {
            return Ok(self.target);
        }
        if settings.llvm_target.is_some() {
            return Ok(settings.llvm_target);
        }
        let host = Options::host();
        let os = settings.os.unwrap_or(host.os);
        let cpu = settings.cpu.unwrap_or(host.cpu);
        if os == TargetOs::Wasm {
            return Ok(Some(Target::wasm_module()));
        }
        if cpu == TargetCpu::Wasm {
            return Err("a native workspace OS requires an X64 or ARM64 CPU".into());
        }
        if os == host.os && cpu == host.cpu {
            return Ok(None);
        }
        Ok(Some(match os {
            TargetOs::Windows => Target::windows(cpu, cfg!(target_os = "windows")),
            _ => Target::new(os, cpu),
        }))
    }

    fn prepare(&self, settings: &BuildSettings, output: &Path) -> Result<Prepared, String> {
        // Explicit CLI target wins; otherwise use the workspace's explicit LLVM
        // triple or its OS/CPU, including non-WASM child workspaces.
        let target = self.workspace_target(settings)?;
        let non_empty = |s: &String| (!s.is_empty()).then(|| s.clone());
        // Windows wants `.exe`/`.dll`/`.lib`; a name without an extension gets the platform's,
        // unless the metaprogram turned `append_executable_filename_extension` off.
        use jaic_llvm::OutputKind;
        let kind = match settings.output_type {
            OutputType::Executable => Some(OutputKind::Executable),
            OutputType::DynamicLibrary => Some(OutputKind::DynamicLibrary),
            OutputType::StaticLibrary => Some(OutputKind::StaticLibrary),
            OutputType::ObjectFile | OutputType::NoOutput => None,
        };
        let mut output = output.to_path_buf();
        if settings.append_extension
            && output.extension().is_none()
            && let Some(ext) = kind.and_then(|k| jaic_llvm::output_extension(target, k))
        {
            output.set_extension(ext);
        }
        if output.is_dir() {
            let name = output.join("program");
            return Err(format!(
                "the output path `{}` is a directory\nhelp: name the file to write inside it, as in `-exe {}`",
                shown(&output),
                shown(&name)
            ));
        }
        let create_dir = |dir: &Path, what: &str| {
            std::fs::create_dir_all(dir).map_err(|e| {
                format!(
                    "could not create the {what} directory `{}`: {}\nhelp: choose another place with `-output_path`",
                    shown(dir),
                    jaic::io_reason(&e)
                )
            })
        };
        if let Some(dir) = output.parent().filter(|d| !d.as_os_str().is_empty()) {
            create_dir(dir, "output")?;
        }
        // Objects, IR and bitcode go to `intermediate_path` when it is set, else next to the
        // output.
        let file_name = output.file_name().unwrap_or_default().to_os_string();
        let intermediate = if settings.intermediate_path.is_empty() {
            output.parent().map(Path::to_path_buf).unwrap_or_default()
        } else {
            let dir = PathBuf::from(&settings.intermediate_path);
            create_dir(&dir, "intermediate")?;
            dir
        };
        let beside = |suffix: &str| {
            let mut name = file_name.clone();
            name.push(suffix);
            intermediate.join(name)
        };
        use jaic_llvm::OptLevel;
        let opt_level = match settings.optimization {
            BitcodeOptimization::O1 => OptLevel::O1,
            BitcodeOptimization::O2 => OptLevel::O2,
            BitcodeOptimization::Os => OptLevel::Os,
            BitcodeOptimization::Oz => OptLevel::Oz,
            BitcodeOptimization::O3 => OptLevel::O3,
            BitcodeOptimization::O0 | BitcodeOptimization::Unset => OptLevel::O0,
        };
        // `Build_Options.emit_debug_info = .NONE`, `llvm_options.preserve_debug_info = false`
        // (or `set_optimization(..., false)`) turn it off.
        let debug_info = self.debug_info
            && settings.emit_debug_info != Some(false)
            && settings.llvm_switch(WorkspaceFlag::LlvmPreserveDebugInfo) != Some(false);
        let sanitize = if self.sanitize.is_empty() {
            jaic_llvm::Sanitize::default()
        } else {
            jaic_llvm::Sanitize::parse(&self.sanitize)?
        };
        let macos = target.unwrap_or_else(Target::host).is_apple();
        let macos_version = (macos && !settings.minimum_os_version.is_empty())
            .then(|| settings.minimum_os_version.clone());
        let mut extra_link_args = Vec::new();
        if let Some(version) = &macos_version {
            extra_link_args.push(format!("-mmacosx-version-min={version}"));
        }
        let switch = |flag: WorkspaceFlag| settings.llvm_switch(flag);
        let wanted = |on: bool, suffix: &str| on.then(|| beside(suffix));
        let codegen = jaic_llvm::Codegen {
            machine_level: match settings.machine_code_optimization {
                MachineCodeOptimization::None => Some(OptLevel::O0),
                MachineCodeOptimization::Less => Some(OptLevel::O1),
                MachineCodeOptimization::Default => Some(OptLevel::O2),
                MachineCodeOptimization::Aggressive => Some(OptLevel::O3),
                MachineCodeOptimization::Unset => None,
            },
            loop_unrolling: switch(WorkspaceFlag::LlvmEnableLoopUnrolling),
            loop_vectorization: switch(WorkspaceFlag::LlvmEnableLoopVectorization),
            slp_vectorization: switch(WorkspaceFlag::LlvmEnableSlpVectorization),
            merge_functions: switch(WorkspaceFlag::LlvmMergeFunctions),
            disable_inlining: if self.no_inline {
                Some(true)
            } else {
                switch(WorkspaceFlag::LlvmDisableInlining)
            },
            tail_calls: switch(WorkspaceFlag::LlvmEnableTailCalls),
            split_modules: switch(WorkspaceFlag::LlvmEnableSplitModules),
            single_unit: self.no_split,
            frame_pointers: settings.frame_pointers,
            no_red_zone: settings.disable_redzone,
            ir_after: wanted(settings.output_llvm_ir, ".ll"),
            bitcode_after: wanted(settings.output_bitcode, ".bc"),
            bitcode_before: wanted(
                settings.output_bitcode_before_optimizations,
                ".unoptimized.bc",
            ),
            macos_version,
            ..self.switches.codegen()
        };
        // `-emit_ir` wins over `output_llvm_ir_before_optimizations`.
        let emit_ir = self.emit_ir.clone().or_else(|| {
            wanted(
                settings.output_llvm_ir_before_optimizations,
                ".unoptimized.ll",
            )
        });
        let options = jaic_llvm::Options {
            opt_level,
            target,
            emit_ir,
            debug_info,
            sanitize,
            cpu: non_empty(&settings.llvm_cpu),
            features: non_empty(&settings.llvm_features),
            codegen,
        };
        // An unoptimized macOS executable or library keeps its objects, in `.build/` beside the
        // output (or in the intermediate directory): the debugger reads the DWARF from them
        // (`docs/native/debug-info.md`). Every other build deletes its objects after linking. A
        // sanitized build still writes the dSYM: the sanitizer's symbolizer does not read the debug
        // map on every macOS, and its reports need file and line.
        let keep_objects = debug_info
            && macos
            && opt_level == OptLevel::O0
            && self.sanitize.is_empty()
            && matches!(
                settings.output_type,
                OutputType::Executable | OutputType::DynamicLibrary
            )
            && std::env::var_os("JAIC_DSYM").is_none();
        let kept_dir = if settings.intermediate_path.is_empty() {
            intermediate.join(".build")
        } else {
            intermediate.clone()
        };
        if settings.output_type != OutputType::ObjectFile {
            remove_stale_objects(&kept_dir, &file_name);
        }
        let object = if settings.output_type == OutputType::ObjectFile {
            output.clone()
        } else if keep_objects {
            kept_dir.join(beside(".o").file_name().unwrap_or_default())
        } else {
            beside(".o")
        };
        // Creating the object file now finds out, with the system's reason, when that place
        // cannot be written, rather than from LLVM or the linker after code generation.
        if keep_objects {
            std::fs::create_dir_all(&kept_dir).map_err(|e| {
                format!(
                    "could not create the object directory `{}`: {}",
                    shown(&kept_dir),
                    jaic::io_reason(&e)
                )
            })?;
        }
        std::fs::File::create(&object).map_err(|e| {
            format!(
                "cannot write `{}`: {}\nhelp: choose a directory you can write to with `-output_path`",
                shown(&object),
                jaic::io_reason(&e)
            )
        })?;
        Ok(Prepared {
            output,
            object,
            keep_objects,
            target,
            options,
            extra_link_args,
        })
    }
}

/// Delete the objects an earlier build of `file_name` left in `dir` (`<name>.o`, `<name>.o.N.o`),
/// so a build with fewer codegen units leaves none behind. The directory itself stays: concurrent
/// builds into the same output directory share it, and removing it would race their writes.
#[cfg(feature = "llvm")]
fn remove_stale_objects(dir: &Path, file_name: &std::ffi::OsStr) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let prefix = file_name.to_string_lossy().into_owned();
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        // `<name>.o`, then `<name>.o.N.o` for the other codegen units.
        let stale = name.strip_prefix(&prefix).is_some_and(|rest| {
            rest == ".o"
                || rest
                    .strip_prefix(".o.")
                    .and_then(|r| r.strip_suffix(".o"))
                    .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
        });
        if stale {
            let _ = std::fs::remove_file(entry.path());
        }
    }
}

#[cfg(feature = "llvm")]
impl OutputBackend for LlvmBackend {
    fn write_output(
        &mut self,
        program: &jaic::ir::Program,
        settings: &BuildSettings,
        output: &Path,
    ) -> Result<(), String> {
        let Prepared {
            output,
            object,
            keep_objects,
            target,
            options,
            extra_link_args,
        } = self.prepare(settings, output)?;
        let output = output.as_path();
        if matches!(
            settings.output_type,
            OutputType::ObjectFile | OutputType::NoOutput
        ) {
            return timings::time("codegen", || {
                jaic_llvm::emit_object(program, &options, &object)
            });
        }
        let objects = timings::time("codegen", || {
            jaic_llvm::emit_objects(program, &options, &object)
        })?;
        let libraries = jaic_llvm::used_libraries(program);
        let wasm = target.is_some_and(jaic::target::Target::is_wasm);
        let mut link_args = settings.additional_linker_arguments.clone();
        link_args.extend(extra_link_args);
        let debug_info = options.debug_info;
        let linked = timings::time("link", || match settings.output_type {
            OutputType::ObjectFile | OutputType::NoOutput => unreachable!(),
            OutputType::Executable | OutputType::DynamicLibrary if wasm => {
                let start = jaic::ir::Linkage::Export("_start".into());
                jaic_llvm::link_wasm(&jaic_llvm::WasmLink {
                    objects: &objects,
                    libraries: &libraries,
                    output,
                    has_start: settings.output_type == OutputType::Executable
                        && program.funcs.iter().flatten().any(|f| f.linkage == start),
                    strip_debug: !debug_info,
                    extra_args: &settings.additional_linker_arguments,
                })
            }
            OutputType::Executable | OutputType::DynamicLibrary => {
                jaic_llvm::link(&jaic_llvm::LinkRequest {
                    objects: &objects,
                    libraries: &libraries,
                    output,
                    dynamic_library: settings.output_type == OutputType::DynamicLibrary,
                    extra_args: &link_args,
                    target,
                    sanitize: options.sanitize,
                    debug_info,
                })
            }
            OutputType::StaticLibrary => jaic_llvm::archive(&objects, output, target),
        });
        if linked.is_ok() {
            jaic::build::report_executable(settings, output);
        }
        // macOS linkers leave DWARF in the objects and record where they are. An unoptimized
        // build keeps its objects for the debugger to read (the `dsymutil` pass would be a
        // tenth of the build); any other collects the DWARF into `output.dSYM` first.
        let macho = debug_info
            && linked.is_ok()
            && target.unwrap_or_else(Target::host).is_apple()
            && settings.output_type != OutputType::StaticLibrary;
        if macho
            && !keep_objects
            && let Err(message) = timings::time("debug info", || jaic_llvm::write_dsym(output))
        {
            eprintln!("warning: {message}");
        }
        if !(keep_objects && linked.is_ok()) {
            for object in &objects {
                let _ = std::fs::remove_file(object);
            }
        }
        linked
    }

    fn write_objects(
        &mut self,
        program: &jaic::ir::Program,
        settings: &BuildSettings,
        output: &Path,
    ) -> Result<jaic::build::LinkObjects, String> {
        let prepared = self.prepare(settings, output)?;
        let objects = timings::time("codegen", || {
            jaic_llvm::emit_objects(program, &prepared.options, &prepared.object)
        })?;
        let libraries = jaic_llvm::used_libraries(program);
        let inputs = jaic_llvm::link_inputs(&libraries, prepared.target)?;
        jaic::build::report_executable(settings, &prepared.output);
        Ok(jaic::build::LinkObjects {
            objects: objects.iter().map(|o| o.display().to_string()).collect(),
            system_libraries: inputs.system_libraries,
            user_libraries: inputs.user_libraries,
        })
    }
}
