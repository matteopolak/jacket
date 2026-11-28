//! Roots linted in one run share parsed files; the findings must be the ones each root gives alone.
use std::path::PathBuf;

use jailint::{
    Lint,
    config::Config,
    driver::{Options, run},
};
use test_support::TempDir;

fn options(root: &std::path::Path, path: PathBuf, jobs: usize) -> Options {
    Options {
        compiler_options: jaic::sema::Options::host(),
        source: None,
        paths: vec![path],
        import_dirs: Vec::new(),
        add_sources: Vec::new(),
        limits: jaic::ide::lint_compile::CheckLimits::default(),
        overlay: std::collections::BTreeMap::new(),
        stdlib: root.join("stdlib"),
        config: Config::default(),
        jobs,
        verbose: false,
    }
}

fn key(lint: &Lint) -> (String, u32, String, String) {
    (
        lint.path.clone(),
        lint.start as u32,
        lint.rule.to_string(),
        lint.message.clone(),
    )
}

#[test]
fn roots_sharing_parsed_files_report_what_each_reports_alone() {
    let root = TempDir::new("shared-parses");

    for dir in ["stdlib/prelude", "stdlib/Runtime_Support", "programs"] {
        std::fs::create_dir_all(root.join(dir)).unwrap();
    }

    std::fs::write(root.join("stdlib/prelude/module.jai"), "").unwrap();
    std::fs::write(
        root.join("stdlib/Runtime_Support/module.jai"),
        "#module_parameters(DEFINE_SYSTEM_ENTRY_POINT:bool, DEFINE_INITIALIZATION:bool, ENABLE_BACKTRACE_ON_CRASH:bool);",
    )
    .unwrap();
    let programs = root.join("programs");

    std::fs::write(programs.join("shared.jai"), "helper :: () { lost := 1; }").unwrap();

    for name in ["a", "b", "c", "d"] {
        std::fs::write(
            programs.join(format!("{name}.jai")),
            format!("#load \"shared.jai\"; main :: () {{ unused_{name} := 2; helper(); }}"),
        )
        .unwrap();
    }

    let mut alone = Vec::new();

    for name in ["a", "b", "c", "d"] {
        let path = programs.join(format!("{name}.jai"));
        let found = run(&options(&root, path.clone(), 1));

        assert!(found.incomplete.is_empty(), "{:?}", found.incomplete);
        alone.extend(
            found
                .lints
                .iter()
                .filter(|l| l.path.ends_with(&format!("/{name}.jai")))
                .map(key),
        );
    }

    alone.sort();
    assert!(!alone.is_empty(), "the unused locals must be reported");

    for jobs in [1, 4] {
        let together = run(&options(&root, programs.clone(), jobs));

        assert!(together.incomplete.is_empty(), "{:?}", together.incomplete);
        let mut own: Vec<_> = together
            .lints
            .iter()
            .filter(|l| !l.path.ends_with("shared.jai"))
            .map(key)
            .collect();

        own.sort();
        assert_eq!(own, alone, "jobs={jobs}");
        assert!(
            together
                .lints
                .iter()
                .any(|l| l.path.ends_with("shared.jai")),
            "the shared file is reported once"
        );
    }
}
