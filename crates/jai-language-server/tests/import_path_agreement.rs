//! The language server, the linter and the command line search the same module folders, in the
//! same order, for a project with a `jacket.toml` and a `modules` folder.
use jai_language_server::{Limits, Session};
use jailint::config::Config;
use jailint::driver::{Options, root_import_paths};
use std::path::{Path, PathBuf};
mod common;
use common::{modules_environment, project, stdlib, uri_of};

fn existing(paths: impl IntoIterator<Item = PathBuf>) -> Vec<PathBuf> {
    paths
        .into_iter()
        .filter_map(|p| p.canonicalize().ok())
        .collect()
}

fn files() -> Vec<(&'static str, &'static str)> {
    vec![
        ("jacket.toml", "[project]\nimport_path = [\"vendor\"]\n"),
        ("main.jai", "main :: () {}\n"),
        ("modules/Local/module.jai", "local :: 1;\n"),
        ("vendor/Vendored.jai", "vendored :: 2;\n"),
    ]
}

fn linter_options(dir: &Path) -> Options {
    Options {
        paths: vec![dir.join("main.jai")],
        import_dirs: Vec::new(),
        stdlib: stdlib(),
        config: Config {
            discover: true,
            ..Config::default()
        },
        jobs: 1,
        verbose: false,
        compiler_options: jaic::sema::Options::host(),
        source: None,
        add_sources: Vec::new(),
        limits: jaic::ide::lint_compile::CheckLimits::default(),
    }
}

#[test]
fn language_server_linter_and_command_line_search_the_same_folders() {
    let dir = project("import-agreement", &files());
    let expected = existing([dir.join("modules"), dir.join("vendor"), stdlib()]);
    assert_eq!(expected.len(), 3);

    // The command line: `jacket` passes `import_path` as `-import_dir`, jaic puts its own
    // `modules` folder first and the standard library last.
    let config =
        jacket_config::parse(&std::fs::read_to_string(dir.join("jacket.toml")).unwrap()).unwrap();
    let cli = jaic::project::import_paths(&dir, &[], &config.project.import_dirs(&dir), &stdlib());
    assert_eq!(existing(cli), expected);

    // The linter.
    let linted = root_import_paths(&linter_options(&dir), &dir.join("main.jai"));
    assert_eq!(existing(linted), expected);

    // The language server, over the live text of the open file.
    let mut session = Session::with_environment(Limits::default(), modules_environment(None));
    session.set_workspace_folders(vec![dir.clone()]);
    let uri = uri_of(&dir.join("main.jai"));
    session
        .open(uri.clone(), 1, "main :: () {}\n".into())
        .unwrap();
    let coverage = session.semantic_coverage(&uri).unwrap().unwrap();
    let served = coverage
        .analysis
        .import_dirs
        .unwrap()
        .into_iter()
        .map(|p| p.path);
    assert_eq!(existing(served), expected);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn import_dir_flags_come_after_the_project_import_path() {
    let dir = project("import-agreement-flags", &files());
    std::fs::create_dir_all(dir.join("extra")).unwrap();
    let mut options = linter_options(&dir);
    options.import_dirs = vec![dir.join("extra")];
    let linted = existing(root_import_paths(&options, &dir.join("main.jai")));
    assert_eq!(
        linted,
        existing([
            dir.join("modules"),
            dir.join("vendor"),
            dir.join("extra"),
            stdlib()
        ])
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn changing_the_workspace_folders_changes_the_module_folders_served() {
    let dir = project(
        "import-folders-change",
        &[
            ("modules/Up/module.jai", "up :: 1;\n"),
            ("sub/main.jai", "main :: () {}\n"),
        ],
    );
    let mut session = Session::with_environment(Limits::default(), modules_environment(None));
    session.set_workspace_folders(vec![dir.clone()]);
    let uri = uri_of(&dir.join("sub/main.jai"));
    session
        .open(uri.clone(), 1, "main :: () {}\n".into())
        .unwrap();
    let served = |session: &mut Session| -> Vec<PathBuf> {
        let coverage = session.semantic_coverage(&uri).unwrap().unwrap();
        existing(
            coverage
                .analysis
                .import_dirs
                .unwrap()
                .into_iter()
                .map(|p| p.path),
        )
    };
    let up = dir.join("modules").canonicalize().unwrap();
    assert!(served(&mut session).contains(&up));
    session.set_workspace_folders(vec![dir.join("sub")]);
    assert!(!served(&mut session).contains(&up));
    session.set_workspace_folders(vec![dir.clone()]);
    assert!(served(&mut session).contains(&up));
    let _ = std::fs::remove_dir_all(&dir);
}
