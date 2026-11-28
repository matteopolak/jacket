//! `rust-spacing [--check] [paths...]`: adds the blank lines rustfmt leaves out (see the library
//! docs for the rules). Paths are files or directories; without any, every `.rs` file under
//! `crates/` and `tools/rust-spacing/` of the checkout that holds the current directory.
//!
//! With `--check`, nothing is written: the files that would change are listed and the exit
//! status is 1. A file that is not valid Rust exits with 2.

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::{env, fs};

const DEFAULT_ROOTS: [&str; 2] = ["crates", "tools/rust-spacing"];
const SKIPPED_DIRS: [&str; 3] = ["target", "node_modules", ".git"];

fn main() -> ExitCode {
    let mut check = false;
    let mut paths = Vec::new();

    for arg in env::args_os().skip(1) {
        match arg.to_str() {
            Some("--check") => check = true,
            Some("-h" | "--help") => {
                println!("usage: rust-spacing [--check] [paths...]");

                return ExitCode::SUCCESS;
            }
            _ => paths.push(PathBuf::from(arg)),
        }
    }

    if paths.is_empty() {
        let Some(root) = checkout_root() else {
            eprintln!("rust-spacing: no rustfmt.toml above the current directory; pass paths");

            return ExitCode::from(2);
        };

        paths.extend(DEFAULT_ROOTS.map(|dir| root.join(dir)));
    }

    let mut files = Vec::new();

    for path in &paths {
        if let Err(error) = collect(path, &mut files) {
            eprintln!("rust-spacing: {}: {error}", path.display());

            return ExitCode::from(2);
        }
    }

    let mut changed = false;

    for file in &files {
        match respace(file, check) {
            Ok(true) => {
                changed = true;
                println!("{}", file.display());
            }
            Ok(false) => {}
            Err(message) => {
                eprintln!("rust-spacing: {}: {message}", file.display());

                return ExitCode::from(2);
            }
        }
    }

    ExitCode::from(u8::from(check && changed))
}

/// Spaces `file`, writing it unless `check`. Whether it changed.
fn respace(file: &Path, check: bool) -> Result<bool, String> {
    let source = fs::read_to_string(file).map_err(|error| error.to_string())?;
    let spaced = rust_spacing::space(&source).map_err(|error| {
        let start = error.span().start();

        format!("{}:{}: {error}", start.line, start.column + 1)
    })?;

    if spaced == source {
        return Ok(false);
    }

    if !check {
        fs::write(file, spaced).map_err(|error| error.to_string())?;
    }

    Ok(true)
}

fn checkout_root() -> Option<PathBuf> {
    let start = env::current_dir().ok()?;

    start
        .ancestors()
        .find(|dir| dir.join("rustfmt.toml").is_file())
        .map(Path::to_path_buf)
}

fn collect(path: &Path, files: &mut Vec<PathBuf>) -> std::io::Result<()> {
    if !path.is_dir() {
        files.push(path.to_path_buf());

        return Ok(());
    }

    let mut entries: Vec<PathBuf> = fs::read_dir(path)?
        .map(|entry| entry.map(|e| e.path()))
        .collect::<Result<_, _>>()?;

    entries.sort();

    for entry in entries {
        let name = entry
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default();

        if entry.is_dir() {
            if !SKIPPED_DIRS.contains(&name) {
                collect(&entry, files)?;
            }
        } else if entry.extension().is_some_and(|ext| ext == "rs") {
            files.push(entry);
        }
    }

    Ok(())
}
