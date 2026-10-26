//! `jaic` command line: `jaic [options] file.jai [options] [+Plugin ...] [- metaprogram args] [-- program args]`.
#![forbid(unsafe_code)]
use self::cli::Cli;
use self::options::{Action, CliError, CliOption, OPTIONS, Request, Section};
use self::parse::{color_choice, parse, report_cli_error};
use self::run::run;
use self::usage::usage_text;
use std::process::ExitCode;

/// Every Rust allocation goes through the counter that enforces `JAIC_MEMORY_LIMIT`; unarmed it
/// costs one relaxed load per call. The `mimalloc` feature (on in every LLVM build) puts mimalloc under it.
#[cfg(not(feature = "mimalloc"))]
#[global_allocator]
static ALLOCATOR: jaic::memory_limit::CountingAllocator =
    jaic::memory_limit::CountingAllocator(std::alloc::System);

#[cfg(feature = "mimalloc")]
#[global_allocator]
static ALLOCATOR: jaic::memory_limit::CountingAllocator<mimalloc::MiMalloc> =
    jaic::memory_limit::CountingAllocator(mimalloc::MiMalloc);

// What jaic prints to stderr takes the build status line down first (`jaic::progress`).
macro_rules! eprintln {
    ($($arg:tt)*) => {{
        jaic::progress::before_output();
        ::std::eprintln!($($arg)*)
    }};
}

macro_rules! eprint {
    ($($arg:tt)*) => {{
        jaic::progress::before_output();
        ::std::eprint!($($arg)*)
    }};
}

mod crash_runtime;

/// `-timings`: wall time per compiler phase, printed to stderr when the command ends.
mod timings {
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::{Duration, Instant};

    use jaic::memory_limit::{Tally, tally};

    static ENABLED: AtomicBool = AtomicBool::new(false);

    /// Phases in first-seen order; a phase that runs more than once (one backend call per
    /// workspace) accumulates.
    static PHASES: Mutex<Vec<Phase>> = Mutex::new(Vec::new());

    struct Phase {
        name: &'static str,
        time: Duration,
        calls: u32,
        /// What Rust code allocated during the phase, workers' included.
        allocated: Tally,
    }

    pub fn enable() {
        ENABLED.store(true, Ordering::Relaxed);
        jaic::memory_limit::count_allocations(true);
    }

    /// What the build status line calls `phase`.
    fn progress_phase(phase: &str) -> Option<jaic::progress::Phase> {
        use jaic::progress::Phase;
        match phase {
            "front end" | "workspaces" => Some(Phase::Checking),
            "prepare output" | "codegen" => Some(Phase::Generating),
            "link" | "debug info" => Some(Phase::Linking),
            "run" => Some(Phase::Program),
            _ => None,
        }
    }

    /// Run `f`, adding its wall time to `phase`, and show `phase` on the build status line.
    pub fn time<T>(phase: &'static str, f: impl FnOnce() -> T) -> T {
        let _shown = progress_phase(phase).map(jaic::progress::enter);
        if !ENABLED.load(Ordering::Relaxed) {
            return f();
        }
        let (start, before) = (Instant::now(), tally());
        let result = f();
        let (elapsed, allocated) = (start.elapsed(), tally().since(before));
        let mut phases = PHASES.lock().unwrap();
        match phases.iter_mut().find(|p| p.name == phase) {
            Some(p) => {
                p.time += elapsed;
                p.calls += 1;
                p.allocated.allocations += allocated.allocations;
                p.allocated.bytes += allocated.bytes;
            }
            None => phases.push(Phase {
                name: phase,
                time: elapsed,
                calls: 1,
                allocated,
            }),
        }
        result
    }

    /// One `jaic-timing: <phase> <seconds> <calls>` line per phase (tools/bench parses them),
    /// then `interp`: the time compile-time code ran, which overlaps `front end`. Then one
    /// `jaic-alloc: <phase> <allocations> <bytes>` line per phase: the blocks Rust code
    /// requested during it (phases nest, so `total` holds the others).
    pub fn report() {
        if !ENABLED.load(Ordering::Relaxed) {
            return;
        }
        let phases = PHASES.lock().unwrap();
        for p in phases.iter() {
            eprintln!(
                "jaic-timing: {} {:.6} {}",
                p.name,
                p.time.as_secs_f64(),
                p.calls
            );
        }
        let (interp, calls) = jaic::interp::compile_time_spent();
        eprintln!("jaic-timing: interp {:.6} {calls}", interp.as_secs_f64());
        for p in phases.iter() {
            eprintln!(
                "jaic-alloc: {} {} {}",
                p.name, p.allocated.allocations, p.allocated.bytes
            );
        }
    }
}

mod backend;
mod cli;
mod leak;
mod options;
mod parse;
mod run;
mod tuning;
mod usage;

fn main() -> ExitCode {
    #[cfg(all(feature = "llvm", feature = "mimalloc"))]
    jaic_llvm::fix_malloc_zone_for_fork();
    install_panic_hook();
    let args: Vec<String> = std::env::args().skip(1).collect();
    // Colour and layout first, so even command-line mistakes use them.
    let color = color_choice(&args);
    jaic::render::set_style(jaic::render::detect(color));
    if let Err(message) = jaic::memory_limit::arm_from_env() {
        eprint!(
            "{}",
            jaic::render::Report::new(jaic::render::Severity::Error, message).render()
        );
        return ExitCode::from(2);
    }
    let cli = match parse(&args) {
        Ok(Request::Compile(cli)) => *cli,
        Ok(Request::Help) => {
            println!("{}", usage_text());
            return ExitCode::SUCCESS;
        }
        Ok(Request::Version) => {
            println!("jaic {}", env!("CARGO_PKG_VERSION"));
            return ExitCode::SUCCESS;
        }
        Ok(Request::Stdlib) => {
            println!("{}", jaic::paths::default_stdlib_dir().display());
            return ExitCode::SUCCESS;
        }
        Ok(Request::DocComments(dir)) => {
            let docs = jaic::doc_comments::library_docs(
                std::path::Path::new(&dir),
                &jaic::paths::default_stdlib_dir(),
            );
            print!("{}", jaic::doc_comments::records(&docs));
            return ExitCode::SUCCESS;
        }
        Err(error) => return report_cli_error(error),
    };
    if cli.msvc_format {
        jaic::render::set_style(jaic::render::Style {
            layout: jaic::render::Layout::Msvc,
            color: false,
        });
    }

    // Deeply recursive programs and checking need a large stack.
    // On macOS the main thread stays free to run the program's foreign calls (AppKit only
    // works there); see `jaic::interp::main_thread`.
    #[cfg(target_os = "macos")]
    {
        jaic::interp::main_thread::serve(1 << 30, move || run(cli)).unwrap_or(ExitCode::from(101))
    }
    #[cfg(not(target_os = "macos"))]
    {
        let worker = std::thread::Builder::new()
            .stack_size(1 << 30)
            .spawn(move || run(cli));
        match worker.map(std::thread::JoinHandle::join) {
            Ok(Ok(code)) => code,
            _ => ExitCode::from(101),
        }
    }
}

/// A panic is a bug in jaic, not in the program: say so, and how to report it, instead of
/// Rust's bare panic message. The exit status stays 101.
fn install_panic_hook() {
    let default = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let message = info
            .payload()
            .downcast_ref::<&str>()
            .map(std::string::ToString::to_string)
            .or_else(|| info.payload().downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "unknown panic".into());
        let at = info
            .location()
            .map(|l| format!(" (at {}:{})", l.file(), l.line()))
            .unwrap_or_default();
        let report = jaic::render::Report::new(
            jaic::render::Severity::Error,
            format!("internal compiler error: {message}"),
        )
        .note(format!("this is a bug in jaic{at}, not in your program"))
        .help(format!(
            "please report it at {}/issues with the program that triggers it; set RUST_BACKTRACE=1 for a backtrace",
            env!("CARGO_PKG_REPOSITORY")
        ));
        eprint!("{}", report.render());
        if std::env::var_os("RUST_BACKTRACE").is_some() {
            default(info);
        }
    }));
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "llvm")]
    use crate::backend::native_backend;
    use crate::cli::{Added, Flavor, Mode, Plugin};
    use crate::run::disable_check_bindings;
    #[cfg(feature = "llvm")]
    use jaic::build::BuildSettings;
    #[cfg(feature = "llvm")]
    use jaic::sema::Options;
    use jaic::sema::{TargetCpu, TargetOs};
    #[cfg(feature = "llvm")]
    use jaic::target::Target;
    use std::path::{Path, PathBuf};

    fn triple(args: &[&str], host_os: TargetOs, host_cpu: TargetCpu) -> Option<String> {
        let mut argv = vec!["main.jai".to_string()];
        argv.extend(args.iter().map(std::string::ToString::to_string));
        let Ok(Request::Compile(cli)) = parse(&argv) else {
            panic!("invalid command line {argv:?}");
        };
        cli.build_target_from(host_os, host_cpu)
            .expect("a target")
            .map(|target| target.to_string())
    }

    fn args(words: &[&str]) -> Vec<String> {
        words.iter().map(std::string::ToString::to_string).collect()
    }

    fn cli(words: &[&str]) -> Cli {
        match parse(&args(words)) {
            Ok(Request::Compile(cli)) => *cli,
            Ok(_) => panic!("{words:?} asked for help or the version"),
            Err(e) => panic!("{words:?}: {} {:?}", e.message, e.help),
        }
    }

    fn error(words: &[&str]) -> CliError {
        match parse(&args(words)) {
            Err(e) => e,
            Ok(_) => panic!("{words:?} should be a command-line mistake"),
        }
    }

    #[test]
    fn the_mode_is_an_option_and_the_file_may_come_anywhere() {
        assert!(cli(&["a.jai"]).mode == Mode::Build);
        assert!(cli(&["a.jai", "-check"]).mode == Mode::Check);
        let interp = cli(&["-interp", "a.jai", "-no_dce"]);
        assert!(interp.mode == Mode::Interp);
        assert_eq!(interp.file, "a.jai");
        assert!(parse(&args(&["a.jai", "-check", "-interp"])).is_err());
        assert!(error(&["a.jai", "b.jai"]).message.contains("one file"));
    }

    #[test]
    fn dashes_split_metaprogram_and_program_arguments() {
        let c = cli(&["a.jai", "-interp", "-", "m1", "m2", "--", "p1"]);
        assert_eq!(c.command_line, ["m1", "m2"]);
        assert_eq!(c.program_args, ["a.jai", "p1"]);
        let c = cli(&["a.jai", "-", "-release", "-x"]);
        assert_eq!(c.command_line, ["-release", "-x"]);
        assert!(
            error(&["a.jai", "--", "p"])
                .message
                .contains("only `-interp`")
        );
    }

    #[test]
    fn plugins_take_the_arguments_up_to_the_next_plugin() {
        let c = cli(&[
            "a.jai",
            "-I",
            "d",
            "+One",
            "-x",
            "1",
            "+Two(A=1)",
            "-",
            "meta",
        ]);
        assert_eq!(c.imports, [PathBuf::from("d")]);
        assert_eq!(c.plugins.len(), 2);
        assert_eq!(c.plugins[0].module, "One");
        assert_eq!(c.plugins[0].options, ["-x", "1"]);
        assert_eq!(c.plugins[1].module, "Two");
        assert_eq!(c.plugins[1].import_parameters(), "(A=1)");
        assert!(c.plugins[1].options.is_empty());
        assert_eq!(c.command_line, ["meta"]);
        assert!(
            error(&["a.jai", "-interp", "+One"])
                .message
                .contains("not `-interp`")
        );
        // Before the first plugin an unknown option is jaic's mistake.
        assert!(
            error(&["a.jai", "-bogus", "+One"])
                .message
                .contains("unknown option")
        );
    }

    #[test]
    fn modules_folder_then_import_dirs_then_stdlib() {
        let c = cli(&["a.jai", "-import_dir", "/p/vendor", "-I", "/p/more"]);
        let options =
            crate::run::compiler_options(&c, Path::new("/std"), Path::new("/p/app/a.jai"))
                .unwrap_or_else(|_| panic!("options"));
        assert_eq!(
            options.import_paths,
            ["/p/app/modules", "/p/vendor", "/p/more", "/std"].map(PathBuf::from)
        );
    }

    #[test]
    fn no_check_bindings_reaches_only_the_standard_librarys_check() {
        let stdlib = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
        let own = std::env::temp_dir().join(format!("jaic-check-{}", std::process::id()));
        std::fs::create_dir_all(own.join("modules/Check")).unwrap();
        std::fs::write(own.join("modules/Check/module.jai"), "").unwrap();
        let mut options = jaic::sema::Options::host();
        options.import_paths = vec![stdlib.clone()];
        let mut c = cli(&["a.jai", "+Check"]);
        disable_check_bindings(&mut c, &options, &stdlib);
        assert_eq!(c.plugins[0].import_parameters(), "(CHECK_BINDINGS=false)");
        // A module of that name found first elsewhere is somebody else's, not the library's.
        options.import_paths = vec![own.join("modules"), stdlib.clone()];
        let mut c = cli(&["a.jai", "+Check"]);
        disable_check_bindings(&mut c, &options, &stdlib);
        assert_eq!(c.plugins[0].import_parameters(), "");
        std::fs::remove_dir_all(own).unwrap();
    }

    #[test]
    fn default_plugin_parameters_do_not_override_explicit_ones() {
        let mut bare = Plugin::parse("Check");
        bare.default_parameter("CHECK_BINDINGS", "false");
        assert_eq!(bare.import_parameters(), "(CHECK_BINDINGS=false)");
        let mut other = Plugin::parse("Check(A=1)");
        other.default_parameter("CHECK_BINDINGS", "false");
        assert_eq!(other.import_parameters(), "(A=1,CHECK_BINDINGS=false)");
        let mut set = Plugin::parse("Check(CHECK_BINDINGS=true)");
        set.default_parameter("CHECK_BINDINGS", "false");
        assert_eq!(set.import_parameters(), "(CHECK_BINDINGS=true)");
    }

    #[test]
    fn optimized_is_a_flavor_that_takes_no_value() {
        assert!(cli(&["a.jai", "-o"]).flavor == Some(Flavor::Optimized));
        assert!(cli(&["-od", "a.jai"]).flavor == Some(Flavor::OptimizedDebug));
        assert!(cli(&["-o", "a.jai"]).flavor == Some(Flavor::Optimized));
        assert!(
            error(&["a.jai", "-o", "-very_debug"])
                .message
                .contains("flavor")
        );
        let c = cli(&["a.jai", "-exe", "build/app.exe", "-output_path", "out"]);
        assert_eq!(c.exe.as_deref(), Some("build/app.exe"));
        assert_eq!(c.output_path, Some(PathBuf::from("out")));
    }

    #[test]
    fn add_and_run_become_source_text() {
        let c = cli(&["a.jai", "-add", "x :: 1;", "-run", "f()", "-run", "g();"]);
        let text: Vec<String> = c.added.iter().map(Added::source).collect();
        assert_eq!(text[0], "x :: 1;");
        assert_eq!(text[1], "#run {\nf();\n};\n");
        assert_eq!(text[2], "#run {\ng();\n};\n");
    }

    #[test]
    fn windows_cross_builds_default_to_x64_mingw() {
        let mac = (TargetOs::MacOS, TargetCpu::Arm64);
        assert_eq!(
            triple(&["-os", "windows"], mac.0, mac.1).as_deref(),
            Some("x86_64-pc-windows-gnu")
        );
        assert_eq!(
            triple(&["-os", "windows", "-cpu", "arm64"], mac.0, mac.1).as_deref(),
            Some("aarch64-pc-windows-gnu")
        );
        assert_eq!(triple(&[], mac.0, mac.1), None);
    }

    #[cfg(feature = "llvm")]
    #[test]
    fn workspace_backend_target_preserves_explicit_overrides_and_rejects_invalid_cpu() {
        let backend = native_backend(&cli(&["main.jai"]));
        let mut settings = BuildSettings {
            os: Some(TargetOs::Linux),
            cpu: Some(TargetCpu::X64),
            llvm_target: Some("x86_64-unknown-linux-musl".parse().unwrap()),
            ..BuildSettings::default()
        };
        assert_eq!(
            backend.workspace_target(&settings).unwrap(),
            Some("x86_64-unknown-linux-musl".parse().unwrap())
        );
        let explicit = native_backend(&cli(&["main.jai", "-target", "aarch64-unknown-linux-gnu"]));
        assert_eq!(
            explicit.workspace_target(&settings).unwrap(),
            Some("aarch64-unknown-linux-gnu".parse().unwrap())
        );

        let host = Options::host();
        let os = jaic::target::os_name(host.os);
        let cpu = jaic::target::cpu_name(host.cpu);
        let explicit_host = native_backend(&cli(&["main.jai", "-os", os, "-cpu", cpu]));
        assert_eq!(
            explicit_host.workspace_target(&settings).unwrap(),
            explicit_host.target
        );

        settings.llvm_target = None;
        settings.cpu = Some(TargetCpu::Wasm);
        assert!(
            backend
                .workspace_target(&settings)
                .unwrap_err()
                .contains("native workspace OS")
        );
        settings.os = Some(TargetOs::Wasm);
        assert_eq!(
            backend.workspace_target(&settings).unwrap(),
            Some(Target::wasm_module())
        );
    }

    #[test]
    fn windows_hosts_build_for_themselves_or_the_other_cpu_with_msvc() {
        let arm = (TargetOs::Windows, TargetCpu::Arm64);
        assert_eq!(triple(&["-os", "windows"], arm.0, arm.1), None);
        assert_eq!(
            triple(&["-cpu", "x64"], arm.0, arm.1).as_deref(),
            Some("x86_64-pc-windows-msvc")
        );
        let x64 = (TargetOs::Windows, TargetCpu::X64);
        assert_eq!(
            triple(&["-cpu", "arm64"], x64.0, x64.1).as_deref(),
            Some("aarch64-pc-windows-msvc")
        );
        assert_eq!(
            triple(&["-target", "aarch64-w64-mingw32"], x64.0, x64.1).as_deref(),
            Some("aarch64-pc-windows-gnu")
        );
    }
}
