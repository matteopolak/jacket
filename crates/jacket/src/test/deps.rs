//! `jacket test --list --deps`: the source files the programs the tests build read.
use super::build::{PLUGIN, Toolchain, host_flags};
use super::{Case, Kind, discover};
use std::path::Path;
use std::process::Command;

/// The source files `jaic -deps` says compiling `program` reads, with the plugin that finds the
/// tests when `plugin` is set (a root's generated program). A program that does not compile has
/// none.
fn files(
    program: &Path,
    root_flags: &[String],
    toolchain: &Toolchain,
    plugin: bool,
) -> Vec<String> {
    let mut command = Command::new(&toolchain.jaic);
    command
        .current_dir(program.parent().unwrap_or(Path::new(".")))
        .arg(program)
        .args(["-deps", "-quiet"])
        .args(host_flags(&toolchain.flags))
        .args(root_flags);
    if plugin {
        command.arg(PLUGIN);
    }
    let Ok(output) = command.output() else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }
    // The plugin prints its records on standard output too: a file is a line that names one.
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| Path::new(line).is_absolute() && Path::new(line).is_file())
        .map(str::to_string)
        .collect()
}

/// The files every program the selected tests build reads, one per line, sorted: each root's
/// generated unit-test program (written by `build::unit` in `work/build-N`) and each golden
/// program.
pub(super) fn print(
    selected: &[Case],
    roots: &[discover::Root],
    root_flags: &[Vec<String>],
    work: &Path,
    toolchain: &Toolchain,
    bench: bool,
) {
    let mut found = std::collections::BTreeSet::new();
    for (i, root) in roots.iter().enumerate() {
        let units = if bench {
            &root.bench
        } else {
            &root.unit
        };
        let wanted = selected
            .iter()
            .any(|c| c.root == i && matches!(c.kind, Kind::Test | Kind::Bench));
        if wanted && !units.is_empty() {
            let program = work.join(format!("build-{i}")).join("tests.jai");
            found.extend(files(&program, &root_flags[i], toolchain, true));
        }
    }
    for case in selected {
        if matches!(case.kind, Kind::Golden | Kind::CompileFail) {
            found.extend(files(&case.file, &root_flags[case.root], toolchain, false));
        }
    }
    for file in found {
        println!("{file}");
    }
}
