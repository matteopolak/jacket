//! Check unchanged source libraries through the normal graph and semantic APIs.
use jai_modules::{Filesystem, GraphOptions, ModuleGraph};
use jai_sema::{ResolveOptions, resolve_library_with_options};
use jai_types::{Architecture, BuildTarget, ByteOrder, LayoutPolicy, OperatingSystem};
use std::{env, path::PathBuf, process::ExitCode};

const USAGE: &str =
    "usage: source-library-check <linux-x64-lp64|macos-arm64-lp64> <module-root> <entry.jai>...";

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
    let (operating_system, architecture) =
        match arguments.next().as_deref().and_then(|value| value.to_str()) {
            Some("linux-x64-lp64") => (OperatingSystem::Linux, Architecture::X86_64),
            Some("macos-arm64-lp64") => (OperatingSystem::MacOS, Architecture::Arm64),
            _ => return Err(USAGE.into()),
        };
    let module_root = PathBuf::from(arguments.next().ok_or(USAGE)?);
    let entries = arguments.map(PathBuf::from).collect::<Vec<_>>();
    if entries.is_empty() {
        return Err(USAGE.into());
    }
    let total = entries.len();
    let target = BuildTarget {
        operating_system,
        architecture,
        layout: LayoutPolicy::lp64(),
        byte_order: ByteOrder::Little,
    };
    let options = ResolveOptions {
        target: Some(target.clone()),
        layout: Some(target.layout),
        ..ResolveOptions::default()
    };
    let mut failed = 0usize;
    for entry in entries {
        let result = ModuleGraph::load_with_target(
            &entry,
            GraphOptions {
                import_dirs: vec![module_root.clone()],
            },
            &Filesystem,
            target.clone(),
        )
        .map_err(|error| error.to_string())
        .and_then(|graph| {
            resolve_library_with_options(&graph, &options, &mut jai_vm::NoEffects)
                .map(|library| {
                    (
                        graph.modules().len(),
                        library.procedures().len(),
                        library.prototypes().len(),
                    )
                })
                .map_err(|error| error.render(graph.sources()))
        });
        match result {
            Ok((modules, bodies, prototypes)) => println!(
                "{}: checked semantic library ({modules} modules, {bodies} bodies, {prototypes} prototypes)",
                entry.display()
            ),
            Err(error) => {
                failed += 1;
                eprintln!("{}: failed source library\n{error}", entry.display());
            }
        }
    }
    println!("source libraries checked: {total}, failed: {failed}");
    if failed == 0 {
        Ok(())
    } else {
        Err(format!(
            "{failed} source libraries failed semantic checking"
        ))
    }
}
