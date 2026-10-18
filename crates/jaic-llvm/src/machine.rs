//! Target machines: triples, CPUs and features, and coverage options.
use super::options::{OptLevel, Options};
use inkwell::targets::{
    CodeModel, InitializationConfig, RelocMode, Target, TargetMachine, TargetTriple,
};
use jaic::target::Target as BuildTarget;
use std::path::PathBuf;
use std::process::Command;

/// `sancov-module` takes its options from LLVM's command-line registry, not from pipeline
/// parameters. A `-coverage` build numbers every basic block (level 2) with a guard and a row in
/// the PC table; a `fuzzer` build uses edge coverage (level 3) in inline 8-bit counters, the PC
/// table and traced compares. The registry is process-wide, so it is set once, by the first
/// build that asks: one process builds with one of the two.
#[allow(unsafe_code)]
pub(super) fn set_coverage_options(fuzzer: bool) {
    static OPTIONS: std::sync::Once = std::sync::Once::new();
    OPTIONS.call_once(|| {
        let flags: &[&str] = if fuzzer {
            &[
                "-sanitizer-coverage-level=3",
                "-sanitizer-coverage-inline-8bit-counters",
                "-sanitizer-coverage-pc-table",
                "-sanitizer-coverage-trace-compares",
            ]
        } else {
            &[
                "-sanitizer-coverage-level=2",
                "-sanitizer-coverage-trace-pc-guard",
                "-sanitizer-coverage-pc-table",
            ]
        };
        let args: Vec<std::ffi::CString> = std::iter::once("jaic")
            .chain(flags.iter().copied())
            .map(|a| std::ffi::CString::new(a).expect("no NUL in a constant"))
            .collect();
        let pointers: Vec<*const std::ffi::c_char> = args.iter().map(|a| a.as_ptr()).collect();
        // SAFETY: the strings outlive the call, which copies what it keeps.
        unsafe {
            inkwell::llvm_sys::support::LLVMParseCommandLineOptions(
                pointers.len() as i32,
                pointers.as_ptr(),
                std::ptr::null(),
            );
        }
    });
}

/// The libFuzzer engine without its `main`, found through the sanitizer driver (`clang
/// --print-file-name`), or a message saying what to install.
pub(super) fn fuzzer_runtime(driver: &str) -> Result<PathBuf, String> {
    let arch = std::env::consts::ARCH;
    let name = if cfg!(target_os = "macos") {
        "libclang_rt.fuzzer_no_main_osx.a".to_string()
    } else {
        format!("libclang_rt.fuzzer_no_main-{arch}.a")
    };
    let out = Command::new(driver)
        .arg(format!("--print-file-name={name}"))
        .output()
        .map_err(|e| format!("could not run `{driver}` to find {name}: {e}"))?;
    let path = PathBuf::from(String::from_utf8_lossy(&out.stdout).trim());
    if path.is_absolute() && path.is_file() {
        Ok(path)
    } else {
        Err(format!(
            "`-sanitize fuzzer` needs {name} from the LLVM that built jaic (`{driver}` did not find it)\nhelp: install compiler-rt (Homebrew `llvm`, or `libclang-rt-23-dev` from apt.llvm.org) or set JAIC_SANITIZER_CC to its clang"
        ))
    }
}

/// The host triple. On macOS LLVM's default names the Darwin kernel version, which it maps to
/// a newer macOS than the SDK the linker targets (a warning per link); objects are built for a
/// deployment target instead (`MACOSX_DEPLOYMENT_TARGET`, default 11.0, the first arm64 macOS).
fn host_triple(macos_version: Option<&str>) -> TargetTriple {
    let default = TargetMachine::get_default_triple();
    let text = default.as_str().to_string_lossy().into_owned();
    match text.split_once("-apple-darwin") {
        Some((arch, _)) => {
            let version = macos_version
                .map(str::to_string)
                .or_else(|| std::env::var("MACOSX_DEPLOYMENT_TARGET").ok())
                .unwrap_or_else(|| "11.0".into());
            TargetTriple::create(&format!("{arch}-apple-macosx{version}"))
        }
        None => default,
    }
}

/// The LLVM CPU name and feature string for a build for `target`, from
/// `llvm_options.target_system_cpu` / `target_system_features`.
///
/// Without a CPU a build targets the oldest CPU its triple runs on, never the build machine's:
/// a program built on a runner with AVX-512 must still run on one without it (release jaifmt
/// is built on one CI machine and run on others). `"native"` opts into the build machine's CPU
/// and features.
pub(super) fn cpu_and_features(
    cpu: Option<&str>,
    features: Option<&str>,
    target: BuildTarget,
) -> (String, String) {
    let mut features = features.unwrap_or_default().to_string();
    let cpu = match cpu.filter(|c| !c.is_empty()) {
        Some("native") => {
            let host = TargetMachine::get_host_cpu_features().to_string();
            features = if features.is_empty() {
                host
            } else {
                format!("{host},{features}")
            };
            TargetMachine::get_host_cpu_name().to_string()
        }
        Some(cpu) => cpu.to_string(),
        None => baseline_cpu(target).to_string(),
    };
    // wasm: `memory.copy`/`memory.fill` for memcpy and memset. Without them LLVM calls
    // `memmove`, which Wasi_Runtime implements with that same intrinsic. Only an explicit
    // `-bulk-memory` turns them off.
    if target.is_wasm() && !features.contains("bulk-memory") {
        if !features.is_empty() {
            features.push(',');
        }
        features.push_str("+bulk-memory");
    }
    (cpu, features)
}

/// The oldest CPU every machine running `target` has: x86-64 (SSE2) on x86-64, the M1 on arm64
/// macOS (macOS 11, the deployment target, runs only on Apple silicon there), and LLVM's
/// `generic` (armv8-a, the wasm MVP) elsewhere.
pub(super) fn baseline_cpu(target: BuildTarget) -> &'static str {
    use jaic::sema::TargetCpu;
    match (target.cpu, target.is_apple()) {
        // Every Intel Mac (and Rosetta) has SSSE3.
        (TargetCpu::X64, true) => "core2",
        (TargetCpu::X64, false) => "x86-64",
        (TargetCpu::Arm64, true) => "apple-m1",
        _ => "generic",
    }
}

/// The target machine for `options`, and what it targets.
pub(super) fn target_machine(
    options: &Options,
) -> Result<(TargetMachine, TargetTriple, BuildTarget, jaic::abi::Arch), String> {
    // Target registration writes process-wide tables: once, not from every codegen thread.
    static TARGETS: std::sync::Once = std::sync::Once::new();
    TARGETS.call_once(|| {
        // Only the backends `BuildTarget::arch` accepts: initialising all of them would make
        // the static link keep every LLVM backend (see docs/tools/llvm-setup.md). No
        // disassembler: nothing reads machine code back. The asm parser is needed for inline asm.
        let config = InitializationConfig {
            disassembler: false,
            ..InitializationConfig::default()
        };
        Target::initialize_x86(&config);
        Target::initialize_aarch64(&config);
        Target::initialize_webassembly(&config);
    });
    let (triple, build_target) = match options.target {
        Some(t) => (TargetTriple::create(&t.to_string()), t),
        None => (
            host_triple(options.codegen.macos_version.as_deref()),
            BuildTarget::host(),
        ),
    };
    let arch = build_target
        .arch()
        .ok_or_else(|| format!("unsupported target architecture in '{build_target}'"))?;
    let target = Target::from_triple(&triple).map_err(|e| e.to_string())?;
    let (cpu, features) = cpu_and_features(
        options.cpu.as_deref(),
        options.features.as_deref(),
        build_target,
    );
    // A wasm module is linked statically; PIC would ask for Emscripten-style dynamic linking.
    let reloc = if arch.is_wasm() {
        RelocMode::Static
    } else {
        RelocMode::PIC
    };
    let machine = target
        .create_target_machine(
            &triple,
            &cpu,
            &features,
            options
                .codegen
                .machine_level
                .unwrap_or(options.opt_level)
                .llvm(),
            reloc,
            CodeModel::Default,
        )
        .ok_or("could not create a target machine")?;
    if options.codegen.machine_level.unwrap_or(options.opt_level) == OptLevel::O0 {
        use_fast_isel(&machine);
    }
    Ok((machine, triple, build_target, arch))
}

/// Select instructions at -O0 with `FastISel` instead of `GlobalISel`.
///
/// LLVM turns `GlobalISel` on by default for unoptimized `AArch64` code, and it is the slowest
/// part of an unoptimized build there (about a third of codegen time, about 20% of the whole
/// build); `FastISel` is the default on x86-64 and produces code of the same quality at -O0.
/// `FastISel` hands any instruction it cannot select to `SelectionDAG`, so nothing is rejected.
#[allow(unsafe_code)]
fn use_fast_isel(machine: &TargetMachine) {
    use inkwell::llvm_sys::target_machine as tm;
    // SAFETY: `machine` owns a live target machine for the whole call.
    unsafe {
        tm::LLVMSetTargetMachineGlobalISel(machine.as_mut_ptr(), 0);
        tm::LLVMSetTargetMachineFastISel(machine.as_mut_ptr(), 1);
    }
}

/// Whether to run the LLVM IR verifier on each module before code generation. It costs about
/// 6% of an unoptimized build, so it runs in debug builds of jaic (and so under `cargo test`)
/// and with `-dev_verify_ir`; a release jaic skips it.
pub(super) fn verify_ir(options: &Options) -> bool {
    options.codegen.verify_ir.unwrap_or(cfg!(debug_assertions))
}
