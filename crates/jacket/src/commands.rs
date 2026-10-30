//! `jacket`'s own commands: `new`, `init`, `config`, `clean`, `explain` and `version`.
use crate::cli::ConfigAction;
use crate::{project, tools};
use jacket_config::{Disk, FILE_NAME};
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
        ConfigAction::Show => config_show(cwd),
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
    println!("# effective configuration from {}", loaded.file().display());
    println!("# CLI flags override it; settings not listed have their defaults\n");
    match toml::to_string(&loaded.config) {
        Ok(text) => {
            print!("{text}");
            ExitCode::SUCCESS
        }
        Err(error) => fail(error),
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
    let Ok(program) = tools::locate(tool) else {
        return "not found".into();
    };
    let flag = if tool == tools::JAIC {
        "-version"
    } else {
        "--version"
    };
    let output = Command::new(program)
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

// ---- explain ---------------------------------------------------------------------------------

/// `jacket explain [code]`: what a diagnostic code means, or the list of all of them.
pub fn explain(code: Option<&str>) -> ExitCode {
    use jaic::diagnostics::{Code, RETIRED};
    let Some(name) = code else {
        print!("{}", explain_list());
        return ExitCode::SUCCESS;
    };
    // `error[type-mismatch]` pasted whole is fine too.
    let name = name
        .trim()
        .trim_start_matches("error[")
        .trim_start_matches("warning[")
        .trim_end_matches(']');
    if let Some(code) = Code::parse(name) {
        print!("{}", explain_one(code));
        return ExitCode::SUCCESS;
    }
    if RETIRED.contains(&name) {
        println!("`{name}` is retired: no diagnostic uses it any more.");
        return ExitCode::SUCCESS;
    }
    let near = jaic::suggest::closest(
        name,
        Code::ALL
            .iter()
            .map(|code| code.name())
            .chain(RETIRED.iter().copied()),
    );
    match near {
        Some(near) => fail(format!(
            "no diagnostic code `{name}`\nhelp: did you mean `{near}`?"
        )),
        None => fail(format!(
            "no diagnostic code `{name}`\nhelp: `jacket explain` lists them all"
        )),
    }
}

fn explain_one(code: jaic::diagnostics::Code) -> String {
    let mut text = format!(
        "{} ({}, {})\n\n{}\n",
        code.name(),
        code.severity().as_str(),
        code.area(),
        wrap(code.summary())
    );
    if code.is_leniency() {
        text.push_str(&format!("\n{}\n", wrap(jaic::diagnostics::LENIENCY_NOTE)));
    }
    if let Some(explanation) = code.explanation() {
        text.push('\n');
        text.push_str(&wrap(explanation));
        text.push('\n');
    }
    if let Some(example) = code.example() {
        text.push_str("\nExample:\n\n");
        for line in example.lines() {
            text.push_str(&format!("    {line}\n"));
        }
    }
    text
}

fn explain_list() -> String {
    use jaic::diagnostics::Code;
    let mut areas: Vec<&str> = Vec::new();
    for code in Code::ALL {
        if !areas.contains(&code.area()) {
            areas.push(code.area());
        }
    }
    areas.sort_unstable();
    let width = Code::ALL
        .iter()
        .map(|code| code.name().len())
        .max()
        .unwrap_or(0);
    let mut text = String::new();
    for area in areas {
        text.push_str(&format!("{area}\n"));
        let mut codes: Vec<Code> = Code::ALL
            .iter()
            .copied()
            .filter(|code| code.area() == area)
            .collect();
        codes.sort_by_key(|code| code.name());
        for code in codes {
            text.push_str(&format!("  {:width$}  {}\n", code.name(), code.summary()));
        }
        text.push('\n');
    }
    text.push_str("`jacket explain <code>` says more about one.\n");
    text
}

/// Breaks `text` at 90 columns, keeping its own line breaks.
fn wrap(text: &str) -> String {
    let mut out = Vec::new();
    for paragraph in text.split('\n') {
        let mut line = String::new();
        for word in paragraph.split_whitespace() {
            if !line.is_empty() && line.len() + 1 + word.len() > 90 {
                out.push(std::mem::take(&mut line));
            }
            if !line.is_empty() {
                line.push(' ');
            }
            line.push_str(word);
        }
        out.push(line);
    }
    out.join("\n")
}

pub fn version() -> ExitCode {
    println!("jacket {}", env!("CARGO_PKG_VERSION"));
    for tool in tools::ALL {
        println!("{tool}: {}", tool_version(tool));
    }
    ExitCode::SUCCESS
}
