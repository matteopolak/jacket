//! Whether a documented module also builds for wasm, found by checking a program that imports it.

use super::model::Module;
use std::path::Path;
use std::process::Command;

/// `flags` without the ones that choose a target (`-os`, `-cpu`, `-target`) or a mode; the error
/// is a flag the compiler would refuse.
fn without_target(flags: &[std::ffi::OsString]) -> Result<Vec<std::ffi::OsString>, String> {
    let text: Vec<String> = flags
        .iter()
        .map(|flag| flag.to_string_lossy().into_owned())
        .collect();
    let found = jaic_cli_args::arguments::target_flags(&text)?;
    Ok(found.host.into_iter().map(Into::into).collect())
}

/// The command that checks `file` for wasm: the user's flags, except the ones that choose a
/// target, since this check chooses its own.
fn wasm_check_command(
    jaic: &Path,
    stdlib: &Path,
    imports: &[std::ffi::OsString],
    file: &Path,
) -> Result<Command, String> {
    let mut command = Command::new(jaic);
    command
        .env(jacket_config::env::STDLIB.name, stdlib)
        .args(["-check", "-no_dce", "-os", "wasm"])
        .args(without_target(imports)?)
        .arg(file);
    Ok(command)
}

/// Whether `module` can be imported when the target is wasm, and most of its items exist there.
/// Found by checking a program that imports it, so a module that asserts another system, or
/// leaves everything to other systems, is out. `n` keeps the scratch files of threads apart.
pub(super) fn builds_for_wasm(
    jaic: &Path,
    stdlib: &Path,
    imports: &[std::ffi::OsString],
    module: &Module,
    n: usize,
) -> Result<bool, String> {
    let asserted_elsewhere =
        !module.platforms.is_empty() && !module.platforms.iter().any(|p| p == "WASM");
    let there = module
        .items
        .iter()
        .filter(|i| i.platforms.is_empty() || i.platforms.iter().any(|p| p == "WASM"))
        .count();
    let somewhere = there > 0 && there * 2 >= module.items.len();
    if asserted_elsewhere || !somewhere {
        return Ok(false);
    }
    let dir = std::env::temp_dir().join(format!("jacket-doc-wasm-{}", std::process::id()));
    let file = dir.join(format!("{n}.jai"));
    let program = format!("#import \"{}\";\nmain :: () {{}}\n", module.import_name());
    std::fs::create_dir_all(&dir)
        .and_then(|()| std::fs::write(&file, program))
        .map_err(|error| {
            format!(
                "cannot write the wasm check of {} to `{}`: {}",
                module.id,
                file.display(),
                jaic::io_reason(&error)
            )
        })?;
    let mut command = wasm_check_command(jaic, stdlib, imports, &file)?;
    let checked = command.output();
    let _ = std::fs::remove_file(&file);
    // A check that fails is the answer (not for wasm); one that cannot run is an error.
    match checked {
        Ok(output) => Ok(output.status.success()),
        Err(error) => Err(format!(
            "could not start `{}` to check {} for wasm: {}",
            jaic.display(),
            module.id,
            jaic::io_reason(&error)
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::model;

    #[test]
    fn the_wasm_check_keeps_import_folders_and_drops_the_target() {
        let flags: Vec<std::ffi::OsString> = [
            "-import_dir",
            "/lib",
            "-os",
            "windows",
            "-cpu=arm64",
            "-no_color",
        ]
        .into_iter()
        .map(Into::into)
        .collect();
        assert_eq!(
            without_target(&flags).unwrap(),
            ["-import_dir", "/lib", "-no_color"]
        );
        // The check names its own target and keeps the user's other flags.
        let command = wasm_check_command(
            Path::new("jaic"),
            Path::new("/s"),
            &flags,
            Path::new("m.jai"),
        )
        .unwrap();
        let args: Vec<_> = command.get_args().collect();
        assert_eq!(
            args,
            [
                "-check",
                "-no_dce",
                "-os",
                "wasm",
                "-import_dir",
                "/lib",
                "-no_color",
                "m.jai"
            ]
        );
        // A flag the compiler would refuse is reported, not passed on.
        let bad: Vec<std::ffi::OsString> = vec!["-os".into()];
        assert!(without_target(&bad).is_err());
        assert!(
            wasm_check_command(Path::new("jaic"), Path::new("/s"), &bad, Path::new("m.jai"))
                .is_err()
        );
    }

    #[test]
    fn a_wasm_check_that_cannot_start_is_an_error_not_a_no() {
        let module = Module {
            id: "Sample".into(),
            items: vec![model::Item::default()],
            ..Module::default()
        };
        let error = builds_for_wasm(Path::new("/nowhere/jaic"), Path::new("/s"), &[], &module, 0)
            .unwrap_err();
        assert!(
            error.starts_with("could not start `/nowhere/jaic` to check Sample for wasm"),
            "{error}"
        );
        // A module that asserts another system needs no check.
        let elsewhere = Module {
            platforms: vec!["LINUX".into()],
            ..module
        };
        assert!(
            !builds_for_wasm(
                Path::new("/nowhere/jaic"),
                Path::new("/s"),
                &[],
                &elsewhere,
                0
            )
            .unwrap()
        );
    }
}
