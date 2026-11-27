//! Named build targets: `--target NAME` picks a `[target.NAME]` table of `jacket.toml` or one of
//! the built-in boards, and `jacket toolchain` lists them. A name that is neither passes to jaic
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
        "Arduino_Uno",
        "avr-none-elf",
        "qemu-system-avr -M uno -bios {exe} -nographic -serial stdio -monitor none",
    ),
    (
        "Qemu_Virt_Aarch64",
        "aarch64-none-elf",
        "qemu-system-aarch64 -M virt -cpu cortex-a53 -nographic -nic none \
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
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Target {
    pub name: String,
    /// Written in the project's `jacket.toml` rather than built in.
    pub project: bool,
    pub triple: Option<String>,
    pub board: Option<String>,
    pub runner: Option<String>,
    /// Link settings written by meaning; `jaic` renders them for the linker it uses.
    pub stack_size: Option<u64>,
    pub entry: Option<String>,
    pub gc_sections: Option<bool>,
}

impl Target {
    fn from_table(name: &str, table: &Table) -> Self {
        Target {
            name: name.to_string(),
            project: true,
            triple: table.triple.clone(),
            board: table.board.clone(),
            runner: table.runner.clone(),
            stack_size: table.stack_size,
            entry: table.entry.clone(),
            gc_sections: table.gc_sections,
        }
    }

    /// The triple this target builds for: its own, else its board's.
    pub fn effective_triple(&self) -> Option<&str> {
        self.triple.as_deref().or_else(|| {
            let board = self.board.as_deref()?;
            BUILT_IN
                .iter()
                .find(|(name, _, _)| *name == board)
                .map(|(_, triple, _)| *triple)
        })
    }

    /// The jaic arguments that build for this target: its triple, then its board's plugin. A
    /// board sets its own triple on the workspace it builds, so a target with one passes none:
    /// `-target` would also make the metaprogram that runs the plugin a program for the board.
    fn flags(&self) -> (Vec<OsString>, Option<OsString>) {
        let mut flags = Vec::new();
        if let (Some(triple), None) = (&self.triple, &self.board) {
            flags.extend(["-target".into(), triple.into()]);
        }
        // `ld.lld`'s spellings: jaic translates them for the linker it finds.
        let mut link: Vec<String> = Vec::new();
        match self.gc_sections {
            Some(true) => link.push("--gc-sections".into()),
            Some(false) => link.push("--no-gc-sections".into()),
            None => {}
        }
        if let Some(entry) = &self.entry {
            link.push(format!("--entry={entry}"));
        }
        if let Some(size) = self.stack_size {
            link.extend(["-z".into(), format!("stack-size={size}")]);
        }
        for arg in link {
            flags.extend(["-link_arg".into(), arg.into()]);
        }
        let plugin = self
            .board
            .as_ref()
            .map(|board| format!("+{BOARDS}/{board}").into());
        (flags, plugin)
    }

    /// This target with `{sysroot}` in its runner filled in. Only running needs the sysroot, so
    /// a build never looks for one.
    pub fn with_sysroot(mut self) -> Result<Self, String> {
        if let Some(runner) = self.runner.take() {
            self.runner = Some(crate::toolchain::fill_sysroot(&runner, &self)?);
        }
        Ok(self)
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
                ..Target::default()
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

/// Resolves the `--target`/`-target` of jaic's `args`. A name `jacket toolchain` lists is
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
            // A triple: jaic reads it. One for another machine is a target with no runner, so
            // `jacket run` does not start what it built here.
            let foreign = jaic::target::Target::parse(&value)
                .is_ok_and(|triple| triple != jaic::target::Target::host());
            let target = foreign.then(|| Target {
                name: value.clone(),
                triple: Some(value.clone()),
                ..Target::default()
            });
            out.splice(at..at, ["-target".into(), value.into()]);
            out.extend(args[region_end..].iter().cloned());
            return Ok(Expansion {
                args: out,
                target,
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
            "`jacket toolchain` lists the targets; an LLVM triple such as aarch64-none-elf also works"
                .to_string()
        }
    };
    format!("unknown target `{name}`\nhelp: {help}")
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
    fn a_triple_for_another_machine_is_a_target_with_no_runner() {
        let wasm = expand(&os(&["--target", "wasm32-wasi"]), None).unwrap();
        let target = wasm.target.unwrap();
        assert_eq!(target.triple.as_deref(), Some("wasm32-wasi"));
        assert_eq!(target.runner, None);
        let host = jaic::target::Target::host().to_string();
        assert!(
            expand(&os(&["--target", &host]), None)
                .unwrap()
                .target
                .is_none()
        );
    }

    #[test]
    fn an_unknown_name_suggests_the_nearest_target() {
        let error = expand(&os(&["--target", "pic"]), Some(&config(PROJECT))).unwrap_err();
        assert!(error.contains("unknown target `pic`"), "{error}");
        assert!(error.contains("did you mean `pico`?"), "{error}");
        let error = expand(&os(&["--target", "zzzzzz"]), None).unwrap_err();
        assert!(error.contains("`jacket toolchain`"), "{error}");
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
    fn link_settings_become_link_arguments_in_ld_lld_spelling() {
        let table = "[target.fw]\ntriple = \"aarch64-none-elf\"\nstack_size = 65536\nentry = \"start\"\ngc_sections = true\n";
        let got = expand(&os(&["m.jai", "--target", "fw"]), Some(&config(table))).unwrap();
        assert_eq!(
            got.args,
            os(&[
                "m.jai",
                "-target",
                "aarch64-none-elf",
                "-link_arg",
                "--gc-sections",
                "-link_arg",
                "--entry=start",
                "-link_arg",
                "-z",
                "-link_arg",
                "stack-size=65536",
            ])
        );
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
    fn a_board_builds_for_its_own_triple_unless_the_table_names_one() {
        let targets = all(Some(&config(PROJECT)));
        let find = |name: &str| targets.iter().find(|t| t.name == name).unwrap().clone();
        assert_eq!(find("pico").effective_triple(), Some("thumbv6m-none-eabi"));
        assert_eq!(find("dev").effective_triple(), Some("aarch64-none-elf"));
        assert_eq!(
            find("Mps2_An386").effective_triple(),
            Some("thumbv7em-none-eabihf")
        );
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
