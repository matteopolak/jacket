//! `jacket`'s own commands: `new`, `init`, `config`, `clean` and `version`.
use crate::cli::ConfigAction;
use crate::{project, tools};
use jacket_config::{Disk, FILE_NAME, migrate};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};

fn fail(message: impl std::fmt::Display) -> ExitCode {
    eprintln!("jacket: {message}");
    ExitCode::from(2)
}

// ---- new / init ------------------------------------------------------------------------------

const MANIFEST_BIN: &str = "[project]\nentry = [\"build.jai\"]\n";
const MANIFEST_LIB: &str = "[project]\nentry = [\"module.jai\"]\n";

fn build_jai(name: &str) -> String {
    format!(
        "\
// The build metaprogram: `jacket build` compiles this file, and its #run builds the program.
#import \"Compiler\";

build :: () {{
    // This workspace only runs the build; the program gets one of its own.
    set_build_options_dc(.{{ do_output = false }});

    w := compiler_create_workspace(\"{name}\");
    options := get_build_options(w);
    options.output_executable_name = \"{name}\";
    set_build_options(options, w);

    add_build_file(\"main.jai\", w);
}}

#run build();
"
    )
}

fn main_jai(name: &str) -> String {
    format!("#import \"Basic\";\n\nmain :: () {{\n    print(\"Hello from {name}!\\n\");\n}}\n")
}

fn module_jai(name: &str) -> String {
    format!(
        "// {name}: a library module. Import it with `#import \"{name}\";`.\n\ngreeting :: () -> string {{\n    return \"Hello from {name}!\";\n}}\n"
    )
}

fn write(path: &Path, text: &str) -> Result<(), String> {
    std::fs::write(path, text).map_err(|e| format!("could not write {}: {e}", path.display()))
}

fn scaffold(dir: &Path, name: &str, lib: bool) -> Result<Vec<&'static str>, String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("could not create {}: {e}", dir.display()))?;
    let manifest = if lib {
        MANIFEST_LIB
    } else {
        MANIFEST_BIN
    };
    let toolchain = format!(
        "\n[toolchain]\nversion = \"{}\"\n",
        env!("CARGO_PKG_VERSION")
    );
    write(&dir.join(FILE_NAME), &format!("{manifest}{toolchain}"))?;
    let mut made = vec![FILE_NAME];
    if lib {
        write(&dir.join("module.jai"), &module_jai(name))?;
        made.push("module.jai");
    } else {
        write(&dir.join("build.jai"), &build_jai(name))?;
        write(&dir.join("main.jai"), &main_jai(name))?;
        made.extend(["build.jai", "main.jai"]);
    }
    Ok(made)
}

fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        && !name.starts_with('-')
}

pub fn new(name: &str, lib: bool) -> ExitCode {
    let dir = PathBuf::from(name);
    let name = dir
        .file_name()
        .map_or(String::new(), |n| n.to_string_lossy().into_owned());
    if !valid_name(&name) {
        return fail(format!(
            "`{name}` is not a project name\nhelp: use letters, digits, `_` and `-`"
        ));
    }
    if dir.exists() {
        return fail(format!("`{}` already exists", dir.display()));
    }
    finish_scaffold(&dir, &name, lib)
}

pub fn init(lib: bool) -> ExitCode {
    let Ok(dir) = std::env::current_dir() else {
        return fail("cannot read the current directory");
    };
    let name = dir
        .file_name()
        .map_or(String::new(), |n| n.to_string_lossy().into_owned());
    if !valid_name(&name) {
        return fail(format!(
            "`{name}` is not a project name\nhelp: use letters, digits, `_` and `-`; `jacket new <name>` makes a directory"
        ));
    }
    for file in [FILE_NAME, "build.jai", "main.jai", "module.jai"] {
        if dir.join(file).exists() {
            return fail(format!(
                "`{file}` already exists here; `jacket init` does not overwrite"
            ));
        }
    }
    finish_scaffold(&dir, &name, lib)
}

fn finish_scaffold(dir: &Path, name: &str, lib: bool) -> ExitCode {
    match scaffold(dir, name, lib) {
        Ok(made) => {
            println!(
                "created {} {name} in {}: {}",
                if lib {
                    "library"
                } else {
                    "program"
                },
                dir.display(),
                made.join(", ")
            );
            ExitCode::SUCCESS
        }
        Err(message) => fail(message),
    }
}

// ---- config ----------------------------------------------------------------------------------

pub fn config(action: ConfigAction, cwd: &Path) -> ExitCode {
    match action {
        ConfigAction::Migrate {
            dry_run,
        } => config_migrate(cwd, dry_run),
        ConfigAction::Show => config_show(cwd),
    }
}

fn config_migrate(cwd: &Path, dry_run: bool) -> ExitCode {
    let Some(location) = jacket_config::locate(&Disk, cwd) else {
        println!("no configuration found above {}", cwd.display());
        return ExitCode::SUCCESS;
    };
    match migrate::run(&Disk, &location, dry_run) {
        Ok((migrate::Outcome::AlreadyMigrated(path), _)) => {
            println!("{} already exists; nothing to migrate", path.display());
            ExitCode::SUCCESS
        }
        Ok((migrate::Outcome::NothingToMigrate, _)) => {
            println!("no legacy config files in {}", location.dir.display());
            ExitCode::SUCCESS
        }
        Ok((
            migrate::Outcome::Wrote {
                path,
                written,
            },
            Some(plan),
        )) => {
            for warning in &plan.warnings {
                eprintln!("jacket: {warning}");
            }
            if written {
                println!("{}", migrate::summary(&plan, &path));
            } else {
                println!(
                    "# would write {} (--dry-run)\n{}",
                    path.display(),
                    plan.text
                );
            }
            ExitCode::SUCCESS
        }
        Ok(_) => ExitCode::SUCCESS,
        Err(error) => fail(error),
    }
}

fn config_show(cwd: &Path) -> ExitCode {
    let loaded = match project::load(cwd) {
        Ok(Some(loaded)) => loaded,
        Ok(None) => {
            println!(
                "# no jacket.toml above {}; every setting has its default",
                cwd.display()
            );
            return ExitCode::SUCCESS;
        }
        Err(error) => return fail(error),
    };
    println!("# effective configuration for {}", loaded.dir().display());
    println!("# CLI flags override it; settings not listed have their defaults\n");
    let text = match toml::to_string(&loaded.config) {
        Ok(text) => text,
        Err(error) => return fail(error),
    };
    let Ok(mut doc) = text.parse::<toml_edit::DocumentMut>() else {
        print!("{text}");
        return ExitCode::SUCCESS;
    };
    let sections: Vec<String> = doc.as_table().iter().map(|(k, _)| k.to_string()).collect();
    for section in sections {
        let origin = loaded
            .source_of(&section)
            .file_name()
            .map_or(String::new(), |n| n.to_string_lossy().into_owned());
        if let Some(table) = doc.get_mut(&section).and_then(|i| i.as_table_mut()) {
            annotate(table, &origin);
        }
    }
    print!("{doc}");
    ExitCode::SUCCESS
}

fn annotate(table: &mut toml_edit::Table, origin: &str) {
    for (_, item) in table.iter_mut() {
        match item {
            toml_edit::Item::Value(value) => {
                value.decor_mut().set_suffix(format!("  # {origin}"));
            }
            toml_edit::Item::Table(inner) => annotate(inner, origin),
            _ => {}
        }
    }
}

// ---- clean -----------------------------------------------------------------------------------

pub fn clean(cwd: &Path) -> ExitCode {
    let loaded = match project::load(cwd) {
        Ok(loaded) => loaded,
        Err(error) => return fail(error),
    };
    let fs = Disk;
    let ctx = crate::compile::Context {
        loaded: loaded.as_ref(),
        cwd,
        fs: &fs,
    };
    let Some(entry) = crate::compile::entry(&ctx) else {
        println!("nothing to clean: no entry file found");
        return ExitCode::SUCCESS;
    };
    let argv = [entry.into_os_string()];
    let Some(exe) = crate::compile::executable(&argv, cwd) else {
        return ExitCode::SUCCESS;
    };
    match std::fs::remove_file(&exe) {
        Ok(()) => println!("removed {}", exe.display()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => println!("nothing to clean"),
        Err(e) => return fail(format!("could not remove {}: {e}", exe.display())),
    }
    ExitCode::SUCCESS
}

// ---- version ---------------------------------------------------------------------------------

/// The first line a tool prints for its version flag.
fn tool_version(tool: &str) -> String {
    if !tools::exists(tool) {
        return "not found".into();
    }
    let flag = if tool == tools::JAIC {
        "-version"
    } else {
        "--version"
    };
    let output = Command::new(tools::find(tool))
        .arg(flag)
        .stdin(Stdio::null())
        .output();
    match output {
        Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout)
            .lines()
            .next()
            .unwrap_or("")
            .trim()
            .to_string(),
        Ok(_) => "installed (no version reported)".into(),
        Err(e) => format!("cannot run: {e}"),
    }
}

pub fn version() -> ExitCode {
    println!("jacket {}", env!("CARGO_PKG_VERSION"));
    for tool in tools::ALL {
        println!("{tool}: {}", tool_version(tool));
    }
    ExitCode::SUCCESS
}
