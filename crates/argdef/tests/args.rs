//! A struct shaped like jaic's: single-dash flags, values, repeats, plugins, `-` and `--`.
#![allow(clippy::duplicated_attributes)]
use argdef::{Args, Error, ErrorKind, Group, Shell};

#[derive(Default, Debug, PartialEq)]
struct Plugin {
    name: String,
    args: Vec<String>,
}

impl Group for Plugin {
    fn start(name: &str) -> Self {
        Plugin {
            name: name.to_string(),
            args: Vec::new(),
        }
    }

    fn push(&mut self, arg: &str) {
        self.args.push(arg.to_string());
    }
}

#[derive(Default, Debug, PartialEq, Clone, Copy)]
enum Level {
    #[default]
    Warn,
    Allow,
    Deny,
}

#[derive(Args, Default, Debug)]
struct Tuning {
    /// threads to use
    #[arg(name = "-threads", value = "n", section = "Output")]
    threads: Option<usize>,
    #[arg(name = "-dev_probe", hidden)]
    probe: bool,
}

/// a compiler
///
/// Options may come before or after the file.
#[derive(Args, Default, Debug)]
#[args(name = "demo", version = "1.0", usage = "demo [options] file [+Plugin ...]", sections = ["Modes", "Output", "Plugins"])]
struct Demo {
    /// only check
    #[arg(name = "-check", section = "Modes", tag = "mode")]
    check: bool,
    /// the executable's name
    /// (`-exe build/app`)
    #[arg(
        name = "-exe",
        value = "name",
        needs = "an executable name",
        section = "Output",
        hint = "file"
    )]
    exe: Option<String>,
    /// a module folder (repeatable)
    #[arg(
        name = "-import_dir",
        name = "-I",
        value = "dir",
        needs = "a directory",
        section = "Output",
        hint = "dir",
        tag = "import"
    )]
    imports: Vec<String>,
    /// optimization level
    #[arg(name = "-O0", set = 0, synopsis = "-O0, -O1", section = "Output")]
    #[arg(name = "-O1", set = 1)]
    opt: Option<u8>,
    /// warning levels
    #[arg(name = "-A", with = level_allow, value = "code", attached, section = "Output")]
    #[arg(name = "-W", with = level_warn, value = "code", attached)]
    levels: Vec<(String, Level)>,
    /// print the help
    #[arg(name = "-help", alias = "-h", exit, section = "Modes")]
    help: bool,
    #[arg(flatten)]
    tuning: Tuning,
    /// `+Name`: a plugin
    #[arg(group = "+", synopsis = "+Name", section = "Plugins")]
    plugins: Vec<Plugin>,
    /// the rest is for the metaprogram
    #[arg(rest = "-", until = "--", synopsis = "-", section = "Plugins")]
    command_line: Option<Vec<String>>,
    /// the rest is for the program
    #[arg(rest = "--", synopsis = "--", section = "Plugins")]
    program_args: Option<Vec<String>>,
    #[arg(positional)]
    files: Vec<String>,
}

fn level_allow(demo: &mut Demo, _: &str, value: &str) -> Result<(), Error> {
    demo.levels.push((value.to_string(), Level::Allow));
    Ok(())
}

fn level_warn(demo: &mut Demo, _: &str, value: &str) -> Result<(), Error> {
    demo.levels.push((value.to_string(), Level::Warn));
    Ok(())
}

fn parse(args: &[&str]) -> Result<Demo, Error> {
    Demo::parse(&args.iter().map(ToString::to_string).collect::<Vec<_>>())
}

#[test]
fn flags_values_and_positionals_in_any_order() {
    let demo = parse(&["-check", "a.jai", "-exe", "out", "-I=x", "-import_dir", "y"]).unwrap();
    assert!(demo.check);
    assert_eq!(demo.exe.as_deref(), Some("out"));
    assert_eq!(demo.imports, ["x", "y"]);
    assert_eq!(demo.files, ["a.jai"]);
}

#[test]
fn a_switch_can_store_a_constant_and_share_a_field() {
    assert_eq!(parse(&["-O1"]).unwrap().opt, Some(1));
    assert_eq!(parse(&["-O0", "-O1"]).unwrap().opt, Some(1));
}

#[test]
fn a_short_option_takes_a_glued_value() {
    let demo = parse(&["-Wunused", "-W", "dead"]).unwrap();
    assert_eq!(
        demo.levels,
        [("unused".into(), Level::Warn), ("dead".into(), Level::Warn)]
    );
    assert_eq!(
        parse(&["-Aold"]).unwrap().levels,
        [("old".into(), Level::Allow)]
    );
    let _ = Level::Deny;
}

#[test]
fn a_flattened_struct_adds_its_options() {
    assert_eq!(parse(&["-threads", "4"]).unwrap().tuning.threads, Some(4));
    assert!(parse(&["-dev_probe"]).unwrap().tuning.probe);
}

#[test]
fn a_bad_value_says_which_option_and_what_it_got() {
    let error = parse(&["-threads", "many"]).unwrap_err();
    assert_eq!(error.message, "`-threads` takes a whole number, not `many`");
    assert_eq!(error.help, ["for example `-threads 4`"]);
}

#[test]
fn mistakes_are_typed() {
    let error = parse(&["-bogus"]).unwrap_err();
    assert_eq!(
        (error.kind, error.subject.as_str()),
        (ErrorKind::Unknown, "-bogus")
    );
    let error = parse(&["-exe"]).unwrap_err();
    assert_eq!(error.kind, ErrorKind::Missing);
    assert_eq!(error.message, "`-exe` needs an executable name");
    assert_eq!(
        parse(&["-I"]).unwrap_err().message,
        "`-I` needs a directory"
    );
}

#[test]
fn a_value_may_look_like_an_option() {
    assert_eq!(
        parse(&["-exe", "-check"]).unwrap().exe.as_deref(),
        Some("-check")
    );
}

#[test]
fn a_plugin_takes_everything_up_to_a_marker() {
    let demo = parse(&[
        "a.jai",
        "+One",
        "-bogus",
        "x",
        "+Two(A=1)",
        "-",
        "m1",
        "--",
        "p1",
    ])
    .unwrap();
    assert_eq!(demo.files, ["a.jai"]);
    assert_eq!(
        demo.plugins,
        [
            Plugin {
                name: "One".into(),
                args: vec!["-bogus".into(), "x".into()]
            },
            Plugin {
                name: "Two(A=1)".into(),
                args: vec![]
            },
        ]
    );
    assert_eq!(demo.command_line, Some(vec!["m1".into()]));
    assert_eq!(demo.program_args, Some(vec!["p1".into()]));
}

#[test]
fn a_marker_with_nothing_after_it_is_still_seen() {
    let demo = parse(&["--"]).unwrap();
    assert_eq!(demo.program_args, Some(vec![]));
    assert_eq!(demo.command_line, None);
    // After `--` even `-` and `--` are the program's.
    let demo = parse(&["--", "-", "--", "+x"]).unwrap();
    assert_eq!(
        demo.program_args,
        Some(vec!["-".into(), "--".into(), "+x".into()])
    );
}

#[test]
fn an_exit_option_stops_parsing() {
    let demo = parse(&["-h", "-bogus"]).unwrap();
    assert!(demo.help);
    assert!(parse(&["-bogus", "-h"]).is_err());
}

#[test]
fn scanning_reports_options_up_to_a_marker() {
    let spec = Demo::spec();
    let args: Vec<String> = ["-check", "-exe", "x", "-I=d", "-bogus", "+P", "-I", "e"]
        .map(String::from)
        .to_vec();
    let scan = spec.scan(&args);
    assert_eq!(scan.boundary, Some(5));
    assert!(scan.error.is_none());
    let seen: Vec<(&str, Option<&str>, &str)> = scan
        .tokens
        .iter()
        .map(|t| (t.name, t.value.as_deref(), t.opt.map_or("", |o| o.tag)))
        .collect();
    assert_eq!(
        seen,
        [
            ("-check", None, "mode"),
            ("-exe", Some("x"), ""),
            ("-I", Some("d"), "import"),
            ("-bogus", None, ""),
        ]
    );
    let args = ["-I"].map(String::from);
    assert_eq!(spec.scan(&args).error.unwrap().kind, ErrorKind::Missing);
}

#[test]
fn help_lists_sections_in_order_with_doc_comments() {
    let text = Demo::spec().help().render(false);
    assert_eq!(
        text,
        "\
demo 1.0: a compiler

Usage: demo [options] file [+Plugin ...]

Options may come before or after the file.

Modes:
  -check                    only check
  -help                     print the help

Output:
  -exe <name>               the executable's name
                            (`-exe build/app`)
  -import_dir, -I <dir>     a module folder (repeatable)
  -O0, -O1                  optimization level
  -A <code>                 warning levels
  -threads <n>              threads to use

Plugins:
  +Name                     `+Name`: a plugin
  -                         the rest is for the metaprogram
  --                        the rest is for the program"
    );
}

#[test]
fn help_is_bold_only_in_colour() {
    let spec = Demo::spec();
    let plain = spec.help().render(false);
    let colored = spec.help().render(true);
    assert!(!plain.contains('\x1b'));
    assert!(colored.contains("\x1b[1;4mModes:\x1b[0m"));
    assert!(colored.contains("\x1b[1m-check\x1b[0m"));
}

#[test]
fn a_tool_can_add_sections_and_paragraphs() {
    let help = Demo::spec()
        .help()
        .section_before(
            "Plugins",
            "Environment",
            vec![argdef::Row {
                label: "DEMO_MEMORY_LIMIT=<bytes|nK|nM>".into(),
                text: "where it lives".into(),
            }],
        )
        .footer("exit status: 0 success");
    let text = help.render(false);
    assert!(text.find("Environment:").unwrap() < text.find("Plugins:").unwrap());
    // A label wider than the column puts its text on the next line.
    assert!(text.contains(
        "  DEMO_MEMORY_LIMIT=<bytes|nK|nM>\n                            where it lives\n"
    ));
    assert!(text.ends_with("exit status: 0 success"));
}

#[test]
fn completions_cover_names_values_and_hints_but_not_hidden_options() {
    let spec = Demo::spec();
    let bash = spec.completions(Shell::Bash);
    assert!(
        bash.contains("-check -exe -import_dir -I -O0 -O1"),
        "{bash}"
    );
    assert!(
        bash.contains("-exe)\n            _demo_reply compgen -f"),
        "{bash}"
    );
    assert!(
        bash.contains("-import_dir|-I)\n            _demo_reply compgen -d"),
        "{bash}"
    );
    assert!(bash.contains("complete -o filenames -F _demo demo"));
    assert!(!bash.contains("-dev_probe"));
    let zsh = spec.completions(Shell::Zsh);
    assert!(zsh.starts_with("#compdef demo\n"));
    assert!(
        zsh.contains("'-exe[the executable's name]:name:_files'")
            || zsh.contains("'-exe[the executable'\\''s name]:name:_files'"),
        "{zsh}"
    );
    assert!(
        zsh.contains("'-import_dir[a module folder (repeatable)]:dir:_files -/'"),
        "{zsh}"
    );
    assert!(!zsh.contains("-dev_probe"));
    // `-name=value`, and a value glued to an `attached` option.
    assert!(zsh.contains("'-exe=[the executable"), "{zsh}");
    assert!(zsh.contains("'-A-[warning levels]:code: '"), "{zsh}");
    let fish = spec.completions(Shell::Fish);
    assert!(
        fish.contains("complete -c demo -o check -d 'only check'"),
        "{fish}"
    );
    assert!(
        fish.contains("complete -c demo -s I -r -f -a '(__fish_complete_directories)'"),
        "{fish}"
    );
    assert!(fish.contains("complete -c demo -o exe -r -F -d"), "{fish}");
}

#[test]
fn shells_are_read_by_name() {
    use argdef::FromArg;
    assert_eq!(Shell::from_arg("-completions", "zsh").unwrap(), Shell::Zsh);
    let error = Shell::from_arg("-completions", "csh").unwrap_err();
    assert_eq!(error.message, "unknown shell `csh` for `-completions`");
    assert_eq!(error.help, ["use bash, zsh or fish"]);
}

#[derive(Args, Default)]
struct Flavors {
    /// optimized
    #[arg(name = "-o", set = 1, conflict = "pick one flavor", section = "Build")]
    #[arg(
        name = "-od",
        set = 2,
        conflict = "pick one flavor",
        help = "lightly optimized"
    )]
    flavor: Option<u8>,
    /// quiet
    #[arg(name = "-quiet")]
    quiet: bool,
    /// not an option
    #[arg(skip)]
    derived: String,
}

#[test]
fn conflicting_constants_are_refused_and_repeats_are_fine() {
    let flavors =
        |args: &[&str]| Flavors::parse(&args.iter().map(ToString::to_string).collect::<Vec<_>>());
    assert!(flavors(&["-od", "-od"]).is_ok());
    assert_eq!(
        flavors(&["-o", "-od"]).err().unwrap().message,
        "pick one flavor"
    );
}

#[test]
fn a_field_without_a_section_joins_the_one_before_and_skipped_fields_are_not_options() {
    let spec = Flavors::spec();
    assert_eq!(
        spec.opts.iter().map(|o| o.section).collect::<Vec<_>>(),
        ["Build", "Build", "Build"]
    );
    assert_eq!(spec.opts.len(), 3);
    assert!(Flavors::default().derived.is_empty());
}

/// What bash offers after `words`, with the cursor on the last one, in `dir`.
fn bash_offers(script: &str, dir: &std::path::Path, words: &[&str]) -> Vec<String> {
    let array: Vec<String> = words.iter().map(|w| format!("'{w}'")).collect();
    let program = format!(
        "source \"$1\"; COMP_WORDS=({}); COMP_CWORD={}; _demo; printf '%s\\n' \"${{COMPREPLY[@]}}\"",
        array.join(" "),
        words.len() - 1
    );
    let script_path = dir.join("demo.bash");
    std::fs::write(&script_path, script).unwrap();
    let output = std::process::Command::new("bash")
        .args(["-c", &program, "bash"])
        .arg(&script_path)
        .current_dir(dir)
        .output()
        .unwrap();
    let mut names: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|l| *l != "demo.bash")
        .map(String::from)
        .collect();
    names.sort();
    names
}

#[test]
fn bash_completes_file_names_with_spaces_whole_and_after_an_equals_sign() {
    if std::process::Command::new("bash")
        .arg("--version")
        .output()
        .is_err()
    {
        return;
    }
    let dir = std::env::temp_dir().join(format!("argdef-bash-{}", std::process::id()));
    std::fs::create_dir_all(dir.join("sub dir")).unwrap();
    std::fs::write(dir.join("a b.txt"), "").unwrap();
    std::fs::write(dir.join("a*c.txt"), "").unwrap();
    let script = Demo::spec().completions(Shell::Bash);
    let both = ["a b.txt".to_string(), "a*c.txt".to_string()];
    // After the option, as two words.
    assert_eq!(bash_offers(&script, &dir, &["demo", "-exe", "a"]), both);
    // Bash splits `-exe=a` into `-exe`, `=` and `a`.
    assert_eq!(
        bash_offers(&script, &dir, &["demo", "-exe", "=", "a"]),
        both
    );
    assert_eq!(bash_offers(&script, &dir, &["demo", "-exe", "="]).len(), 3);
    // A directory option offers only directories.
    assert_eq!(
        bash_offers(&script, &dir, &["demo", "-I", "s"]),
        ["sub dir"]
    );
    // Options by name, and files for a bare argument.
    assert_eq!(bash_offers(&script, &dir, &["demo", "-chec"]), ["-check"]);
    assert_eq!(bash_offers(&script, &dir, &["demo", "a"]), both);
    std::fs::remove_dir_all(&dir).unwrap();
}

#[derive(Default, Debug, Args)]
struct Bytes {
    #[arg(name = "-linker", value = "program", os)]
    linker: Option<std::ffi::OsString>,
    #[arg(name = "-sysroot", value = "dir", os)]
    sysroot: Option<std::path::PathBuf>,
    #[arg(name = "-link_arg", value = "argument", os)]
    link_args: Vec<std::ffi::OsString>,
    #[arg(name = "-name", value = "text")]
    name: Option<String>,
    #[arg(positional)]
    files: Vec<String>,
}

#[cfg(unix)]
mod os_values {
    use super::*;
    use std::ffi::OsString;
    use std::os::unix::ffi::{OsStrExt, OsStringExt};

    fn bytes(b: &[u8]) -> OsString {
        OsString::from_vec(b.to_vec())
    }

    #[test]
    fn an_os_option_keeps_bytes_that_are_not_utf8() {
        let argv = [
            "-linker".into(),
            bytes(b"/bin/\xfe"),
            "-link_arg".into(),
            bytes(b"-L\xff"),
            bytes(b"-link_arg=/x/\xfe.a"),
            "-sysroot=/ok".into(),
            "a.jai".into(),
        ];
        let parsed = Bytes::parse(&argv).unwrap();
        assert_eq!(parsed.linker.unwrap().as_bytes(), b"/bin/\xfe");
        assert_eq!(parsed.link_args[0].as_bytes(), b"-L\xff");
        assert_eq!(parsed.link_args[1].as_bytes(), b"/x/\xfe.a");
        assert_eq!(parsed.sysroot.unwrap(), std::path::PathBuf::from("/ok"));
        assert_eq!(parsed.files, ["a.jai"]);
    }

    #[test]
    fn any_other_argument_must_be_utf8() {
        for argv in [
            vec![bytes(b"\xff.jai")],
            vec!["-name".into(), bytes(b"\xff")],
            vec![bytes(b"-name=\xff")],
        ] {
            let error = Bytes::parse(&argv).unwrap_err();
            assert!(
                error.message.contains("not valid UTF-8"),
                "{}",
                error.message
            );
            assert_eq!(
                error.help,
                ["only the values of -linker, -sysroot and -link_arg may be any bytes"]
            );
        }
    }

    #[test]
    fn scanning_treats_such_an_argument_as_no_option() {
        let argv = [bytes(b"\xff"), "-name".into(), "x".into()];
        let scan = Bytes::spec().scan(&argv);
        assert_eq!(scan.tokens.len(), 2);
        assert!(scan.tokens[0].opt.is_none());
        assert_eq!(scan.tokens[1].value.as_deref(), Some("x"));
    }
}
