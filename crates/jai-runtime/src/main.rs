//! Minimal runtime CLI: its dependency graph contains no native backend.
use jai_runtime::{Options, Script};
use std::{env, path::PathBuf, process::ExitCode};

fn main() -> ExitCode {
    match run() {
        Ok(status) => ExitCode::from(status),
        Err(error) => { eprintln!("{error}"); ExitCode::FAILURE }
    }
}
fn run() -> Result<u8, Box<dyn std::error::Error>> {
    let mut args = env::args_os().skip(1);
    if args.next().as_deref() != Some(std::ffi::OsStr::new("run")) {
        return Err("usage: jai-script run <file.jai> [--fuel <steps>] [-- <arguments>...]".into());
    }
    let path = PathBuf::from(args.next().ok_or("missing script source file")?);
    let mut options = Options::default();
    let mut arguments = Vec::new();
    while let Some(value) = args.next() {
        if value == "--" {
            for value in args { arguments.push(value.into_string().map_err(|_| "script arguments must be UTF-8")?); }
            break;
        } else if value == "--fuel" {
            options.limits.fuel = args.next().ok_or("--fuel requires a count")?
                .to_str().ok_or("fuel must be an integer")?.parse()?;
        } else { return Err("script flags must be --fuel <steps> or -- <arguments>...".into()); }
    }
    let script = Script::prepare(&path, &jai_modules::Filesystem, options)?;
    Ok(script.run(&arguments)?.process_status())
}
