//! Named build targets: `--target NAME` picks a `[target.NAME]` table of `jacket.toml` or one of
//! the built-in boards, and `jacket targets` lists them. A name that is neither passes to jaic
//! as the LLVM triple it is, unless it cannot be one.
use jacket_config::Config;
use jacket_config::schema::Target as Table;
use std::ffi::OsString;
use std::path::Path;

/// Where the boards live in the standard library.
pub const BOARDS: &str = "Extensions/Embedded/Boards";

/// What jacket knows about a board of the standard library: the triple it builds for and the
/// command that runs its programs. A test keeps this in step with [`BOARDS`].
const BUILT_IN: &[(&str, &str, &str)] = &[
    (
        "Qemu_Virt_Aarch64",
        "aarch64-none-elf",
        "qemu-system-aarch64 -M virt -cpu cortex-a53 -nographic \
         -semihosting-config enable=on,target=native -kernel {exe} -append {args}",
    ),
    (
        "Mps2_An386",
        "thumbv7em-none-eabihf",
        "qemu-system-arm -M mps2-an386 -cpu cortex-m4 -nographic \
         -semihosting-config enable=on,target=native -kernel {exe} -append {args}",
    ),
    (
        "Mps2_An505",
        "thumbv8m.main-none-eabihf",
        "qemu-system-arm -M mps2-an505 -cpu cortex-m33 -nographic \
         -semihosting-config enable=on,target=native -kernel {exe} -append {args}",
    ),
];

/// A target `--target` can name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Target {
    pub name: String,
    /// Written in the project's `jacket.toml` rather than built in.
    pub project: bool,
    pub triple: Option<String>,
    pub board: Option<String>,
    pub runner: Option<String>,
}

impl Target {
    fn from_table(name: &str, table: &Table) -> Self {
        Target {
            name: name.to_string(),
            project: true,
            triple: table.triple.clone(),
            board: table.board.clone(),
            runner: table.runner.clone(),
        }
    }

    /// The jaic arguments that build for this target: its triple, then its board's plugin. A
    /// board sets its own triple on the workspace it builds, so a target with one passes none:
    /// `-target` would also make the metaprogram that runs the plugin a program for the board.
    fn flags(&self) -> (Vec<OsString>, Option<OsString>) {
        let mut flags = Vec::new();
        if let (Some(triple), None) = (&self.triple, &self.board) {
            flags.extend(["-target".into(), triple.into()]);
        }
        let plugin = self
            .board
            .as_ref()
            .map(|board| format!("+{BOARDS}/{board}").into());
        (flags, plugin)
    }

    /// The command that runs `exe`: the runner's words with `{exe}` filled in, or `exe` added
    /// last when the runner does not say where it goes, and `{args}` (arguments for the tests)
    /// empty.
    pub fn command(&self, exe: &Path) -> Option<Vec<OsString>> {
        let runner = self.runner.as_deref()?;
        let exe = exe.to_string_lossy();
        let mut words: Vec<OsString> = runner
            .split_whitespace()
            .map(|word| {
                if word == "{args}" {
                    String::new()
                } else {
                    word.replace("{exe}", &exe)
                }
                .into()
            })
            .collect();
        if !runner.contains("{exe}") {
            words.push(exe.as_ref().into());
        }
        (!words.is_empty()).then_some(words)
    }
}

/// Every target: the project's tables, then the built-in boards they do not replace.
pub fn all(config: Option<&Config>) -> Vec<Target> {
    let mut targets: Vec<Target> = config
        .into_iter()
        .flat_map(|config| &config.target)
        .map(|(name, table)| Target::from_table(name, table))
        .collect();
    for &(name, triple, runner) in BUILT_IN {
        if !targets.iter().any(|target| target.name == name) {
            targets.push(Target {
                name: name.to_string(),
                project: false,
                triple: Some(triple.to_string()),
                board: Some(name.to_string()),
                runner: Some(runner.to_string()),
            });
        }
    }
    targets
}

/// `args` with `--target NAME` replaced by what the target stands for, and the target.
#[derive(Debug)]
pub struct Expansion {
    pub args: Vec<OsString>,
    pub target: Option<Target>,
}

/// Resolves the `--target`/`-target` of jaic's `args`. A name `jacket targets` lists is
/// expanded; a triple is left for jaic; anything else is an error naming the nearest target.
pub fn expand(args: &[OsString], config: Option<&Config>) -> Result<Expansion, String> {
    let region_end = args
        .iter()
        .position(|a| a == "-" || a == "--")
        .unwrap_or(args.len());
    let mut out: Vec<OsString> = Vec::new();
    let mut found: Option<(usize, String)> = None;
    let mut index = 0;
    while index < region_end {
        let Some(arg) = args[index].to_str() else {
            out.push(args[index].clone());
            index += 1;
            continue;
        };
        let (name, inline) = match arg.split_once('=') {
            Some((name, value)) => (name, Some(value.to_string())),
            None => (arg, None),
        };
        if name != "-target" && name != "--target" {
            out.push(args[index].clone());
            index += 1;
            continue;
        }
        let value = if let Some(value) = inline {
            value
        } else {
            index += 1;
            args.get(index)
                .filter(|_| index < region_end)
                .and_then(|v| v.to_str())
                .ok_or_else(|| format!("`{name}` needs a target name or an LLVM target triple"))?
                .to_string()
        };
        index += 1;
        found = Some((out.len(), value));
    }
    let Some((at, value)) = found else {
        return Ok(Expansion {
            args: args.to_vec(),
            target: None,
        });
    };
    let targets = all(config);
    let Some(target) = targets.iter().find(|t| t.name == value).cloned() else {
        if value.contains('-') {
            // A triple: jaic reads it.
            out.splice(at..at, ["-target".into(), value.into()]);
            out.extend(args[region_end..].iter().cloned());
            return Ok(Expansion {
                args: out,
                target: None,
            });
        }
        return Err(unknown(&value, &targets));
    };
    let (flags, plugin) = target.flags();
    out.splice(at..at, flags);
    out.extend(plugin);
    out.extend(args[region_end..].iter().cloned());
    Ok(Expansion {
        args: out,
        target: Some(target),
    })
}

/// What `jacket test --target NAME` builds with and runs with: the jaic flags, and the runner
/// (`{exe}` and `{args}` stand for the program and the test arguments, see
/// `test::build::program_and_args`). A board's plugin gets the option `tests`, which exports the
/// reset function that runs the tests.
pub fn for_tests(
    name: &str,
    config: Option<&Config>,
) -> Result<(Vec<String>, Option<String>), String> {
    let expansion = expand(&os_args(["--target", name]), config)?;
    let mut flags: Vec<String> = Vec::new();
    for arg in &expansion.args {
        let arg = arg.to_string_lossy().into_owned();
        let board = arg.starts_with('+');
        flags.push(arg);
        if board {
            flags.push("tests".to_string());
        }
    }
    let runner = expansion.target.and_then(|target| target.runner);
    Ok((flags, runner))
}

fn os_args<const N: usize>(words: [&str; N]) -> Vec<OsString> {
    words.into_iter().map(OsString::from).collect()
}

/// The error for a `--target` that names nothing.
fn unknown(name: &str, targets: &[Target]) -> String {
    let near = jaic::suggest::closest(name, targets.iter().map(|t| t.name.as_str()));
    let help = match near {
        Some(near) => format!("did you mean `{near}`?"),
        None => {
            "`jacket targets` lists the targets; an LLVM triple such as aarch64-none-elf also works"
                .to_string()
        }
    };
    format!("unknown target `{name}`\nhelp: {help}")
}

/// Whether `program` is an executable file in a `PATH` folder (or a path to one).
pub fn installed(program: &str) -> bool {
    let is_exe = |p: &Path| p.is_file();
    if program.contains(std::path::MAIN_SEPARATOR) {
        return is_exe(Path::new(program));
    }
    jacket_config::tools::on_path(Path::new(program)).is_some()
}

/// The table `jacket targets` prints.
pub fn listing(targets: &[Target], installed: impl Fn(&str) -> bool) -> String {
    let rows: Vec<[String; 5]> = targets
        .iter()
        .map(|t| {
            let runner = match t
                .runner
                .as_deref()
                .and_then(|r| r.split_whitespace().next())
            {
                Some(program) if installed(program) => format!("{program} (installed)"),
                Some(program) => format!("{program} (not installed)"),
                None => "-".to_string(),
            };
            [
                t.name.clone(),
                if t.project {
                    "project"
                } else {
                    "built in"
                }
                .to_string(),
                t.triple.clone().unwrap_or_else(|| "-".to_string()),
                t.board.clone().unwrap_or_else(|| "-".to_string()),
                runner,
            ]
        })
        .collect();
    let header = ["TARGET", "FROM", "TRIPLE", "BOARD", "RUNNER"].map(String::from);
    let widths: Vec<usize> = (0..5)
        .map(|c| {
            std::iter::once(&header)
                .chain(&rows)
                .map(|row| row[c].len())
                .max()
                .unwrap_or(0)
        })
        .collect();
    let mut text = String::new();
    for row in std::iter::once(&header).chain(&rows) {
        let line: Vec<String> = row
            .iter()
            .zip(&widths)
            .map(|(cell, width)| format!("{cell:<width$}"))
            .collect();
        text.push_str(line.join("  ").trim_end());
        text.push('\n');
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(text: &str) -> Config {
        jacket_config::parse(text).unwrap()
    }

    fn os(words: &[&str]) -> Vec<OsString> {
        words.iter().map(OsString::from).collect()
    }

    const PROJECT: &str = "[target.pico]\ntriple = \"thumbv6m-none-eabi\"\nrunner = \"flash {exe}\"\n\
                           [target.dev]\nboard = \"Qemu_Virt_Aarch64\"\nrunner = \"emu\"\n";

    #[test]
    fn a_project_target_becomes_its_triple() {
        let got = expand(
            &os(&["m.jai", "--target", "pico", "-O2"]),
            Some(&config(PROJECT)),
        )
        .unwrap();
        assert_eq!(
            got.args,
            os(&["m.jai", "-target", "thumbv6m-none-eabi", "-O2"])
        );
        assert_eq!(got.target.unwrap().name, "pico");
    }

    #[test]
    fn a_board_adds_its_plugin_after_the_flags() {
        let got = expand(
            &os(&["m.jai", "-target=dev", "--", "a"]),
            Some(&config(PROJECT)),
        )
        .unwrap();
        assert_eq!(
            got.args,
            os(&[
                "m.jai",
                "+Extensions/Embedded/Boards/Qemu_Virt_Aarch64",
                "--",
                "a"
            ])
        );
    }

    #[test]
    fn a_built_in_board_is_a_target_without_a_table() {
        let got = expand(&os(&["m.jai", "--target", "Qemu_Virt_Aarch64"]), None).unwrap();
        assert_eq!(
            got.args,
            os(&["m.jai", "+Extensions/Embedded/Boards/Qemu_Virt_Aarch64"])
        );
        assert!(got.target.unwrap().runner.unwrap().contains("{exe}"));
    }

    #[test]
    fn a_board_sets_its_own_triple_so_none_is_passed() {
        let got = expand(&os(&["m.jai", "--target", "Mps2_An386"]), None).unwrap();
        assert_eq!(
            got.args,
            os(&["m.jai", "+Extensions/Embedded/Boards/Mps2_An386"])
        );
        let table = "[target.m4]\nboard = \"Mps2_An386\"\ntriple = \"thumbv7em-none-eabihf\"\n";
        let got = expand(&os(&["--target", "m4"]), Some(&config(table))).unwrap();
        assert_eq!(got.args, os(&["+Extensions/Embedded/Boards/Mps2_An386"]));
    }

    #[test]
    fn a_triple_and_no_target_pass_through() {
        let args = os(&["m.jai", "-target", "x86_64-pc-windows-gnu"]);
        assert_eq!(expand(&args, None).unwrap().args, args);
        let args = os(&["m.jai", "-O2"]);
        assert!(expand(&args, None).unwrap().target.is_none());
    }

    #[test]
    fn an_unknown_name_suggests_the_nearest_target() {
        let error = expand(&os(&["--target", "pic"]), Some(&config(PROJECT))).unwrap_err();
        assert!(error.contains("unknown target `pic`"), "{error}");
        assert!(error.contains("did you mean `pico`?"), "{error}");
        let error = expand(&os(&["--target", "zzzzzz"]), None).unwrap_err();
        assert!(error.contains("`jacket targets`"), "{error}");
    }

    #[test]
    fn a_target_flag_without_a_value_is_an_error() {
        assert!(expand(&os(&["--target"]), None).is_err());
        assert!(expand(&os(&["--target", "--", "x"]), None).is_err());
    }

    #[test]
    fn tests_take_a_target_s_flags_and_runner() {
        let (flags, runner) = for_tests("pico", Some(&config(PROJECT))).unwrap();
        assert_eq!(flags, ["-target", "thumbv6m-none-eabi"]);
        assert_eq!(runner.as_deref(), Some("flash {exe}"));
        let (flags, runner) = for_tests("x86_64-pc-windows-gnu", None).unwrap();
        assert_eq!(flags, ["-target", "x86_64-pc-windows-gnu"]);
        assert_eq!(runner, None);
        let (flags, runner) = for_tests("Qemu_Virt_Aarch64", None).unwrap();
        assert_eq!(
            flags,
            ["+Extensions/Embedded/Boards/Qemu_Virt_Aarch64", "tests"]
        );
        assert!(runner.unwrap().contains("-append {args}"));
    }

    #[test]
    fn the_runner_gets_the_executable() {
        let targets = all(Some(&config(PROJECT)));
        let pico = targets.iter().find(|t| t.name == "pico").unwrap();
        assert_eq!(
            pico.command(Path::new("/o/app")).unwrap(),
            os(&["flash", "/o/app"])
        );
        let dev = targets.iter().find(|t| t.name == "dev").unwrap();
        assert_eq!(
            dev.command(Path::new("/o/app")).unwrap(),
            os(&["emu", "/o/app"])
        );
        let none = Target {
            runner: None,
            ..pico.clone()
        };
        assert!(none.command(Path::new("/o/app")).is_none());
    }

    #[test]
    fn the_listing_shows_every_target_and_whether_its_runner_is_installed() {
        let text = listing(&all(Some(&config(PROJECT))), |program| program == "flash");
        let line = |name: &str| text.lines().find(|l| l.starts_with(name)).unwrap_or("");
        assert!(text.starts_with("TARGET"), "{text}");
        assert!(
            line("pico").contains("thumbv6m-none-eabi")
                && line("pico").contains("flash (installed)")
        );
        assert!(line("dev").contains("emu (not installed)") && line("dev").contains("project"));
        assert!(line("Qemu_Virt_Aarch64").contains("built in"));
        // The project's `dev` is listed once, and the built-in board it does not replace stays.
        assert_eq!(text.lines().count(), 3 + BUILT_IN.len());
    }

    #[test]
    fn the_built_in_boards_are_the_ones_in_the_library() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../stdlib")
            .join(BOARDS);
        let mut library: Vec<String> = std::fs::read_dir(dir)
            .unwrap()
            .filter_map(Result::ok)
            .filter(|e| e.path().join("module.jai").is_file())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        library.sort();
        let mut known: Vec<&str> = BUILT_IN.iter().map(|b| b.0).collect();
        known.sort_unstable();
        assert_eq!(library, known);
    }
}
