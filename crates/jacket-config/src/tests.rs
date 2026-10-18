use super::*;
use crate::migrate::{Outcome, plan, run};

/// A directory tree in memory.
struct Mem(Vec<(&'static str, &'static str)>);

impl Files for Mem {
    fn is_file(&self, path: &Path) -> bool {
        self.0.iter().any(|(p, _)| Path::new(p) == path)
    }

    fn read(&self, path: &Path) -> Option<String> {
        self.0
            .iter()
            .find(|(p, _)| Path::new(p) == path)
            .map(|(_, t)| t.to_string())
    }
}

const JAI: &str =
    "# the project\nbuild_files = [\"first.jai\"] # entry\nimport_path = [\"vendor\"]\n";
const LINT: &str = "# lint settings\nexclude = [\n  \"tests/**\", # fixtures\n]\nmax_nesting = 4\n\n[rules]\n# strict here\nfloat_equality = \"deny\"\n";
const FMT: &str =
    "# fmt\nindent_width = 2\nbrace_style = \"same_line\" # always\nignore = [\"a/**\"]\n";

fn migrated(files: Vec<(&'static str, &'static str)>) -> (String, Config, Vec<String>) {
    let fs = Mem(files);
    let p = plan(&fs, Path::new("/p")).unwrap();
    let config = parse(&p.text).unwrap_or_else(|e| panic!("{e}\n{}", p.text));
    (p.text, config, p.warnings)
}

#[test]
fn project_file_alone() {
    let (text, config, warnings) = migrated(vec![("/p/jai.toml", JAI)]);
    assert_eq!(config.project.entry.0, ["first.jai"]);
    assert_eq!(config.project.import_path.0, ["vendor"]);
    assert!(warnings.is_empty());
    assert!(
        text.contains("# the project") && text.contains("# entry"),
        "{text}"
    );
}

#[test]
fn lint_file_alone() {
    let (text, config, _) = migrated(vec![("/p/jailint.toml", LINT)]);
    assert_eq!(config.lint.exclude, ["tests/**"]);
    assert_eq!(config.lint.max_nesting, Some(4));
    assert_eq!(config.lint.rules["float_equality"], "deny");
    assert!(text.contains("[lint.rules]"), "{text}");
    assert!(
        text.contains("# strict here") && text.contains("# fixtures"),
        "{text}"
    );
}

#[test]
fn format_file_alone() {
    let (text, config, _) = migrated(vec![("/p/jaifmt.toml", FMT)]);
    assert_eq!(config.format.indent_width, Some(2));
    assert_eq!(config.format.ignore, ["a/**"]);
    assert!(text.contains("# always"), "{text}");
}

#[test]
fn all_three_merge() {
    let (text, config, _) = migrated(vec![
        ("/p/jai.toml", JAI),
        ("/p/jailint.toml", LINT),
        ("/p/jaifmt.toml", FMT),
    ]);
    assert_eq!(config.project.entry.0, ["first.jai"]);
    assert_eq!(config.lint.max_nesting, Some(4));
    assert_eq!(config.format.brace_style.as_deref(), Some("same_line"));
    assert!(
        config.unknown_keys().is_empty(),
        "{:?}",
        config.unknown_keys()
    );
    assert!(text.starts_with(
        "# Written by `jacket config migrate` from jai.toml, jailint.toml, jaifmt.toml."
    ));
}

#[test]
fn unknown_keys_are_kept_with_a_warning() {
    let (text, config, warnings) = migrated(vec![
        (
            "/p/jai.toml",
            "build_files = \"x.jai\"\nflavour = \"mint\"\n",
        ),
        ("/p/jailint.toml", "colour = 1\n"),
    ]);
    assert!(
        text.contains("flavour") && text.contains("colour"),
        "{text}"
    );
    assert_eq!(warnings.len(), 2, "{warnings:?}");
    assert_eq!(config.unknown_keys(), ["project.flavour", "lint.colour"]);
    assert_eq!(config.project.entry.0, ["x.jai"]);
}

#[test]
fn an_invalid_legacy_file_is_an_error_naming_it() {
    let fs = Mem(vec![("/p/jai.toml", "build_files = [")]);
    let error = plan(&fs, Path::new("/p")).unwrap_err();
    assert_eq!(error.path, Path::new("/p/jai.toml"));
}

#[test]
fn the_nearest_directory_wins_and_jacket_toml_beats_legacy() {
    let fs = Mem(vec![
        ("/p/jacket.toml", "[project]\nentry = [\"a.jai\"]\n"),
        ("/p/jai.toml", "build_files = [\"b.jai\"]\n"),
        ("/p/sub/jailint.toml", "max_nesting = 3\n"),
    ]);
    let here = load(&fs, Path::new("/p/deep")).unwrap().unwrap();
    assert_eq!(here.config.project.entry.0, ["a.jai"]);
    assert!(
        here.notes.iter().any(|n| n.contains("ignoring jai.toml")),
        "{:?}",
        here.notes
    );
    let sub = load(&fs, Path::new("/p/sub/x")).unwrap().unwrap();
    assert_eq!(sub.location.dir, Path::new("/p/sub"));
    assert_eq!(sub.config.lint.max_nesting, Some(3));
    assert!(
        sub.notes
            .iter()
            .any(|n| n.contains("jacket config migrate"))
    );
    assert!(load(&fs, Path::new("/q")).unwrap().is_none());
}

#[test]
fn migrating_is_idempotent_and_dry_runs_write_nothing() {
    let dir = std::env::temp_dir().join(format!("jacket-config-migrate-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("jai.toml"), JAI).unwrap();
    std::fs::write(dir.join("jailint.toml"), LINT).unwrap();
    let location = locate(&Disk, &dir).unwrap();
    let (outcome, _) = run(&Disk, &location, true).unwrap();
    assert!(matches!(
        outcome,
        Outcome::Wrote {
            written: false,
            ..
        }
    ));
    assert!(!dir.join("jacket.toml").exists());
    let (outcome, _) = run(&Disk, &location, false).unwrap();
    assert!(matches!(
        outcome,
        Outcome::Wrote {
            written: true,
            ..
        }
    ));
    let first = std::fs::read_to_string(dir.join("jacket.toml")).unwrap();
    // The old files stay.
    assert!(dir.join("jai.toml").exists() && dir.join("jailint.toml").exists());
    // A second run finds the new file and leaves it alone.
    let location = locate(&Disk, &dir).unwrap();
    let (outcome, _) = run(&Disk, &location, false).unwrap();
    assert_eq!(outcome, Outcome::AlreadyMigrated(dir.join("jacket.toml")));
    assert_eq!(
        std::fs::read_to_string(dir.join("jacket.toml")).unwrap(),
        first
    );
    // Loading the written file gives what loading the legacy files did.
    let from_file = load(&Disk, &dir).unwrap().unwrap().config;
    let from_legacy = parse(&plan(&Disk, &dir).unwrap().text).unwrap();
    assert_eq!(from_file, from_legacy);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn entries_follow_the_configured_then_the_inferred_order() {
    let project = Project {
        entry: schema::Strings(vec!["x.jai".into()]),
        ..Project::default()
    };
    let configured = entry_files(Path::new("/p"), Some(&project));
    assert!(configured.declared);
    assert_eq!(configured.paths, [PathBuf::from("/p/x.jai")]);
    let inferred = entry_files(Path::new("/p"), None);
    assert!(!inferred.declared);
    let names: Vec<_> = inferred
        .paths
        .iter()
        .map(|p| p.strip_prefix("/p").unwrap().to_str().unwrap())
        .collect();
    assert_eq!(
        names,
        [
            "build.jai",
            "first.jai",
            "main.jai",
            "src/main.jai",
            "module.jai"
        ]
    );
    let fs = Mem(vec![("/p/module.jai", ""), ("/p/main.jai", "")]);
    assert_eq!(
        first_entry(&fs, Path::new("/p"), None),
        Some(PathBuf::from("/p/main.jai"))
    );
}

#[test]
fn experimental_and_toolchain_parse() {
    let config = parse("[toolchain]\nversion = \"0.20.0\"\n[experimental]\nparse_threads = 3\nserial_parse = true\ncodegen_units = 8\n").unwrap();
    assert_eq!(config.toolchain.version.as_deref(), Some("0.20.0"));
    assert_eq!(config.experimental.parse_threads, Some(3));
    assert_eq!(config.experimental.serial_parse, Some(true));
    assert_eq!(config.experimental.codegen_units, Some(8));
    assert!(parse("[project]\nentry = 3\n").is_err());
    assert_eq!(line_of("a = 1\n[rules]\n  b = 2", "b"), Some(3));
}
