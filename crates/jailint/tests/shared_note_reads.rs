//! `unused_note` across roots: a file several programs load is linted by one of them, but a
//! note in it is read when any of them read it.
use jailint::{
    Level,
    config::Config,
    driver::{Options, run},
};
use std::path::{Path, PathBuf};
use test_support::TempDir;

fn options(dir: &Path, config: Config) -> Options {
    Options {
        compiler_options: jaic::sema::Options::host(),
        source: None,
        paths: vec![dir.to_path_buf()],
        import_dirs: Vec::new(),
        add_sources: Vec::new(),
        limits: jaic::ide::lint_compile::CheckLimits::default(),
        overlay: std::collections::BTreeMap::new(),
        stdlib: Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib"),
        config,
        jobs: 1,
        verbose: false,
    }
}

#[test]
fn a_note_read_by_another_program_that_loads_its_file_is_used() {
    let root = TempDir::new("shared-note-reads");
    let dir: PathBuf = root.join("programs");

    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("shared.jai"),
        "Config :: struct {\n    port: int; @json\n    host: string; @toml\n}\n",
    )
    .unwrap();
    // `a` sorts first, so it lints `shared.jai`, but only `b` reads a note.
    std::fs::write(dir.join("a.jai"), "#load \"shared.jai\";\nmain :: () {}\n").unwrap();
    std::fs::write(
        dir.join("b.jai"),
        "#load \"shared.jai\";\n#run {\n    for member: type_info(Config).members for note: member.notes if note == \"json\" {}\n}\nmain :: () {}\n",
    )
    .unwrap();
    let mut config = Config {
        root: dir.clone(),
        ..Config::default()
    };

    config.levels.insert("unused_note".into(), Level::Warn);
    let found = run(&options(&dir, config));

    assert!(found.incomplete.is_empty(), "{:?}", found.incomplete);
    let messages: Vec<&str> = found
        .lints
        .iter()
        .filter(|l| l.rule == "unused_note")
        .map(|l| l.message.as_str())
        .collect();

    assert_eq!(messages, ["no compile-time code read the note `@toml`"]);
}
