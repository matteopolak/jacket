//! `jacket build`, `check` and `run`: `jaic` with its arguments untouched.
//!
//! The rule is that `jacket build <args>` is `jaic <args>` byte for byte. `jacket` adds only what
//! the project says and the arguments left open:
//!
//! - no `.jai` file among the arguments: the project's entry file goes first (`[project] entry`,
//!   else the first of `jacket_config::INFERRED_ENTRIES` that exists);
//! - `[project] import_path` becomes `-import_dir`;
//! - `[experimental]` becomes `-parse_threads`, `-serial_parse` and `-codegen_units`, each only
//!   when the arguments do not already carry that flag ([`project_flags`], which `jacket test`
//!   uses too).
//!
//! `check` is `jaic -check <args>`. `run` is `jaic <args>` followed by the program (arguments
//! after `--` go to it); with `-interp` the whole command line, `--` included, is jaic's.
use jacket_config::{Files, Loaded};
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

/// What the project contributes to a command line.
pub struct Context<'a> {
    pub loaded: Option<&'a Loaded>,
    pub cwd: &'a Path,
    pub fs: &'a dyn Files,
}

fn text(arg: &OsStr) -> &str {
    arg.to_str().unwrap_or("")
}

/// The arguments that are jaic's own flags and files: up to the metaprogram's `-` or the
/// program's `--`.
fn flag_region(args: &[OsString]) -> &[OsString] {
    let end = args
        .iter()
        .position(|a| a == "-" || a == "--")
        .unwrap_or(args.len());
    &args[..end]
}

fn has_flag(region: &[OsString], name: &str) -> bool {
    region.iter().any(|a| {
        let a = text(a);
        a == name
            || a.strip_prefix(name)
                .is_some_and(|rest| rest.starts_with('='))
    })
}

/// The `.jai` file the arguments name, if any.
pub fn named_input(args: &[OsString]) -> Option<&OsString> {
    flag_region(args)
        .iter()
        .find(|a| text(a).ends_with(".jai") && !text(a).starts_with('-'))
}

/// Whether the arguments only ask about jaic (`-help`, `-version`): nothing is added to them.
fn asks_about_jaic(args: &[OsString]) -> bool {
    flag_region(args).iter().any(|a| {
        matches!(
            text(a),
            "-help" | "--help" | "-h" | "-version" | "--version"
        )
    })
}

/// The entry file of the project at hand, absolute.
pub fn entry(ctx: &Context) -> Option<PathBuf> {
    let base = ctx.loaded.map_or(ctx.cwd, |l| l.dir());
    let project = ctx.loaded.map(|l| &l.config.project);
    jacket_config::first_entry(ctx.fs, base, project)
        // A configured entry that is not there yet is still what the project asks for; jaic
        // says it cannot read it.
        .or_else(|| {
            project
                .and_then(|p| p.entry.0.first())
                .map(|e| base.join(e))
        })
}

/// A flag the project supplies.
struct Supplied {
    flag: &'static str,
    value: Option<String>,
    /// The user's arguments override it: any of these (or the flag itself) among them drops it.
    /// `None` for a flag that is repeatable and always added.
    unless: Option<&'static [&'static str]>,
}

/// What `[project]` and `[experimental]` ask of the compiler, in command-line order.
fn supplied(loaded: &Loaded) -> Vec<Supplied> {
    let mut out: Vec<Supplied> = loaded
        .config
        .project
        .import_dirs(loaded.dir())
        .into_iter()
        .map(|dir| Supplied {
            flag: "-import_dir",
            value: Some(dir.to_string_lossy().into_owned()),
            unless: None,
        })
        .collect();
    let experimental = &loaded.config.experimental;
    if let Some(n) = experimental.parse_threads {
        out.push(Supplied {
            flag: "-parse_threads",
            value: Some(n.to_string()),
            unless: Some(&[]),
        });
    }
    if experimental.serial_parse == Some(true) {
        out.push(Supplied {
            flag: "-serial_parse",
            value: None,
            unless: Some(&[]),
        });
    }
    if let Some(n) = experimental.codegen_units {
        out.push(Supplied {
            flag: "-codegen_units",
            value: Some(n.to_string()),
            unless: Some(&["-no_split"]),
        });
    }
    out
}

/// The flags the project adds in front of `args` (jaic's arguments, user-written): its import
/// folders, and each `[experimental]` setting the arguments do not already carry. `build`,
/// `check`, `run` and `test` all take their compiler flags from here.
pub fn project_flags(args: &[OsString], loaded: Option<&Loaded>) -> Vec<OsString> {
    let Some(loaded) = loaded else {
        return Vec::new();
    };
    let region = flag_region(args);
    let mut out: Vec<OsString> = Vec::new();
    for supplied in supplied(loaded) {
        let given = supplied.unless.is_some_and(|others| {
            has_flag(region, supplied.flag) || others.iter().any(|o| has_flag(region, o))
        });
        if !given {
            out.push(supplied.flag.into());
            out.extend(supplied.value.map(OsString::from));
        }
    }
    out
}

/// The command line `jaic` is started with.
pub fn jaic_argv(check: bool, args: &[OsString], ctx: &Context) -> Vec<OsString> {
    let mut argv: Vec<OsString> = Vec::new();
    if check {
        argv.push("-check".into());
    }
    if asks_about_jaic(args) {
        argv.extend(args.iter().cloned());
        return argv;
    }
    if named_input(args).is_none()
        && let Some(entry) = entry(ctx)
    {
        argv.push(entry.into_os_string());
    }
    argv.extend(project_flags(args, ctx.loaded));
    argv.extend(args.iter().cloned());
    argv
}

/// Where `jaic argv` writes the executable (`docs/tools/cli.md`, "Where the executable goes").
pub fn executable(argv: &[OsString], cwd: &Path) -> Option<PathBuf> {
    let source = Path::new(named_input(argv)?);
    let region = flag_region(argv);
    let value = |name: &str| {
        region
            .iter()
            .position(|a| a == name)
            .and_then(|i| region.get(i + 1))
            .map(PathBuf::from)
    };
    let absolute = |p: PathBuf| {
        if p.is_absolute() {
            p
        } else {
            cwd.join(p)
        }
    };
    let dir = match value("-output_path") {
        Some(dir) => absolute(dir),
        None => absolute(source.parent().unwrap_or(Path::new("")).to_path_buf()),
    };
    let mut path = match value("-exe") {
        Some(exe) if exe.components().count() > 1 || exe.is_absolute() => {
            if value("-output_path").is_some() && exe.is_relative() {
                dir.join(exe)
            } else {
                absolute(exe)
            }
        }
        Some(name) => dir.join(name),
        None => dir.join(source.file_stem()?),
    };
    let suffix = std::env::consts::EXE_SUFFIX;
    if !suffix.is_empty() && path.extension().is_none_or(|e| e != &suffix[1..]) {
        let mut name = path.into_os_string();
        name.push(suffix);
        path = PathBuf::from(name);
    }
    Some(path)
}

/// `jacket run`'s arguments split into jaic's and the program's.
pub fn split_run(args: &[OsString]) -> (&[OsString], &[OsString]) {
    if flag_region_has(args, "-interp") {
        return (args, &[]);
    }
    match args.iter().position(|a| a == "--") {
        Some(i) => (&args[..i], &args[i + 1..]),
        None => (args, &[]),
    }
}

fn flag_region_has(args: &[OsString], flag: &str) -> bool {
    has_flag(flag_region(args), flag)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Mem(Vec<&'static str>);

    impl Files for Mem {
        fn is_file(&self, path: &Path) -> bool {
            self.0.iter().any(|p| Path::new(p) == path)
        }

        fn read(&self, _: &Path) -> std::io::Result<String> {
            Err(std::io::ErrorKind::NotFound.into())
        }
    }

    fn os(args: &[&str]) -> Vec<OsString> {
        args.iter().map(OsString::from).collect()
    }

    fn argv(
        check: bool,
        args: &[&str],
        loaded: Option<&Loaded>,
        files: Vec<&'static str>,
    ) -> Vec<String> {
        let fs = Mem(files);
        let ctx = Context {
            loaded,
            cwd: Path::new("/w"),
            fs: &fs,
        };
        jaic_argv(check, &os(args), &ctx)
            .iter()
            .map(|a| a.to_string_lossy().into_owned())
            .collect()
    }

    fn loaded(toml: &str) -> Loaded {
        Loaded {
            config: jacket_config::parse(toml).unwrap(),
            location: jacket_config::Location {
                dir: "/w".into(),
            },
            notes: Vec::new(),
            problems: Vec::new(),
        }
    }

    /// Typical jaic command lines reach jaic exactly as typed.
    #[test]
    fn jaic_arguments_are_forwarded_verbatim() {
        let cases: &[&[&str]] = &[
            &["main.jai"],
            &["main.jai", "-o"],
            &["-o", "main.jai", "-exe", "build/app"],
            &["main.jai", "-x64", "-release", "-import_dir", "vendor"],
            &["first.jai", "+Check", "-", "release", "--", "a"],
            &[
                "build.jai",
                "-plug",
                "Check",
                "-os",
                "windows",
                "-cpu",
                "arm64",
            ],
            &["a.jai", "-add", "x :: 1;", "-run", "print(\"hi\");"],
            &["m.jai", "-interp", "--", "one", "two"],
            &[
                "m.jai",
                "-sanitize",
                "address,undefined",
                "-emit_ir",
                "out.ll",
            ],
            &["-help"],
            &["-version"],
        ];
        for case in cases {
            assert_eq!(argv(false, case, None, vec![]), *case, "build {case:?}");
            let with = loaded("[project]\nentry = [\"x.jai\"]\n");
            let mut want = vec!["-check"];
            want.extend(case.iter());
            assert_eq!(
                argv(true, case, Some(&with), vec!["/w/x.jai"]),
                want,
                "check {case:?}"
            );
        }
    }

    #[test]
    fn the_entry_goes_first_when_no_file_is_named() {
        let with = loaded("[project]\nentry = [\"src/app.jai\", \"other.jai\"]\n");
        assert_eq!(
            argv(false, &["-o"], Some(&with), vec!["/w/src/app.jai"]),
            ["/w/src/app.jai", "-o"]
        );
        // Nothing configured: the first inferred entry that exists.
        assert_eq!(
            argv(false, &[], None, vec!["/w/main.jai", "/w/module.jai"]),
            ["/w/main.jai"]
        );
        assert_eq!(
            argv(false, &["-o"], None, vec!["/w/build.jai", "/w/first.jai"]),
            ["/w/build.jai", "-o"]
        );
        // Nothing to infer: jaic reports the missing file itself.
        assert_eq!(argv(false, &["-o"], None, vec![]), ["-o"]);
        // A file after `-` belongs to the metaprogram, not to jaic.
        assert_eq!(
            argv(false, &["-", "x.jai"], None, vec!["/w/main.jai"]),
            ["/w/main.jai", "-", "x.jai"]
        );
    }

    #[test]
    fn project_settings_become_jaic_flags_unless_given() {
        let with = loaded(
            "[project]\nentry = [\"m.jai\"]\nimport_path = [\"vendor\"]\n[experimental]\nparse_threads = 2\nserial_parse = true\ncodegen_units = 4\n",
        );
        assert_eq!(
            argv(false, &[], Some(&with), vec!["/w/m.jai"]),
            [
                "/w/m.jai",
                "-import_dir",
                "/w/vendor",
                "-parse_threads",
                "2",
                "-serial_parse",
                "-codegen_units",
                "4"
            ]
        );
        assert_eq!(
            argv(
                false,
                &["m.jai", "-parse_threads", "8", "-no_split"],
                Some(&with),
                vec![]
            ),
            [
                "-import_dir",
                "/w/vendor",
                "-serial_parse",
                "m.jai",
                "-parse_threads",
                "8",
                "-no_split"
            ]
        );
    }

    #[test]
    fn test_runs_take_the_same_project_flags() {
        let with = loaded(
            "[project]\nimport_path = [\"vendor\"]\n[experimental]\nparse_threads = 2\ncodegen_units = 4\n",
        );
        let flags = |args: &[&str]| -> Vec<String> {
            project_flags(&os(args), Some(&with))
                .iter()
                .map(|a| a.to_string_lossy().into_owned())
                .collect()
        };
        assert_eq!(
            flags(&["-O2"]),
            [
                "-import_dir",
                "/w/vendor",
                "-parse_threads",
                "2",
                "-codegen_units",
                "4"
            ]
        );
        assert_eq!(
            flags(&["-parse_threads=8", "-no_split"]),
            ["-import_dir", "/w/vendor"]
        );
        assert!(project_flags(&os(&["-O2"]), None).is_empty());
    }

    #[test]
    fn run_splits_program_arguments_unless_interpreting() {
        let args = os(&["m.jai", "-o", "--", "a", "b"]);
        let (jaic, program) = split_run(&args);
        assert_eq!(jaic, &os(&["m.jai", "-o"])[..]);
        assert_eq!(program, &os(&["a", "b"])[..]);
        let args = os(&["m.jai", "-interp", "--", "a"]);
        let (jaic, program) = split_run(&args);
        assert_eq!(jaic.len(), 4);
        assert!(program.is_empty());
    }

    #[test]
    fn executable_follows_the_cli_rules() {
        let at = |args: &[&str]| {
            executable(&os(args), Path::new("/w")).map(|p| p.to_string_lossy().into_owned())
        };
        let suffix = std::env::consts::EXE_SUFFIX;
        assert_eq!(at(&["src/m.jai"]), Some(format!("/w/src/m{suffix}")));
        assert_eq!(
            at(&["m.jai", "-exe", "app"]),
            Some(format!("/w/app{suffix}"))
        );
        assert_eq!(
            at(&["m.jai", "-exe", "build/app"]),
            Some(format!("/w/build/app{suffix}"))
        );
        assert_eq!(
            at(&["m.jai", "-output_path", "out", "-exe", "app"]),
            Some(format!("/w/out/app{suffix}"))
        );
        assert_eq!(at(&["-o"]), None);
    }
}
