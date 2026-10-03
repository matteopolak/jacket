//! Source-only automatic module loading, independent of sema and native tools.
use jai_modules::{
    BootstrapOptions, Filesystem, GraphOptions, ModuleGraph, PreludeSource, RuntimeSupportOptions,
    RuntimeSupportParameters, RuntimeSupportSource,
};
use jai_types::{Architecture, BuildTarget, ByteOrder, LayoutPolicy, OperatingSystem};
use std::{env, path::PathBuf, process::ExitCode};

const USAGE: &str = "usage: bootstrap-check <entry.jai> <linux-x64-lp64|macos-arm64-lp64> <entry-point:true|false> <initialization:true|false> <backtrace:true|false> <module-root>...";

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let mut arguments = env::args_os().skip(1);
    let entry = PathBuf::from(arguments.next().ok_or(USAGE)?);
    let profile = arguments.next().ok_or(USAGE)?;
    let (operating_system, architecture) = match profile.to_str() {
        Some("linux-x64-lp64") => (OperatingSystem::Linux, Architecture::X86_64),
        Some("macos-arm64-lp64") => (OperatingSystem::MacOS, Architecture::Arm64),
        _ => return Err(USAGE.into()),
    };
    let mut boolean = || match arguments.next().as_deref().and_then(|value| value.to_str()) {
        Some("true") => Ok(true),
        Some("false") => Ok(false),
        _ => Err(USAGE.to_owned()),
    };
    let parameters = RuntimeSupportParameters {
        define_system_entry_point: boolean()?,
        define_initialization: boolean()?,
        enable_backtrace_on_crash: boolean()?,
        temporary_storage_size: 32768,
    };
    let import_dirs = arguments.map(PathBuf::from).collect::<Vec<_>>();
    if import_dirs.is_empty() {
        return Err(USAGE.into());
    }
    let graph = ModuleGraph::load_with_bootstrap_options(
        &entry,
        GraphOptions {
            import_dirs,
        },
        BootstrapOptions {
            prelude: PreludeSource::Search,
            runtime_support: Some(RuntimeSupportOptions {
                source: RuntimeSupportSource::Search,
                parameters,
            }),
        },
        &Filesystem,
        Some(BuildTarget {
            operating_system,
            architecture,
            layout: LayoutPolicy::lp64(),
            byte_order: ByteOrder::Little,
        }),
    )
    .map_err(|error| error.to_string())?;
    println!(
        "{}: {} module instances, {} source records, {} declarations (source graph only)",
        entry.display(),
        graph.modules().len(),
        graph.sources().records().len(),
        graph.declarations().len()
    );
    Ok(())
}
