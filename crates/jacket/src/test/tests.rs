use super::*;

#[test]
fn a_run_without_a_seed_draws_a_short_one() {
    assert!(random_seed() < 1_000_000_000);
}

pub(super) fn case(name: &str, kind: Kind) -> Case {
    Case {
        name: name.into(),
        local: name.rsplit("::").next().unwrap_or(name).into(),
        kind,
        root: 0,
        file: PathBuf::from("/t.jai"),
        line: 1,
        notes: Vec::new(),
        options: TestOptions::default(),
        run_as: String::new(),
        doc: None,
        cases: Vec::new(),
    }
}

#[test]
fn filters_select_tests_and_subtests() {
    let table = case("A::table", Kind::Test);
    let none: [String; 0] = [];
    assert_eq!(select(&table, &none, false).as_deref(), Some("table"));
    assert_eq!(
        select(&table, &["tab".into()], false).as_deref(),
        Some("table")
    );
    assert_eq!(select(&table, &["tab".into()], true), None);
    assert_eq!(
        select(&table, &["A::table/two".into()], true).as_deref(),
        Some("table/two")
    );
    assert_eq!(select(&table, &["other".into()], false), None);
    let golden = case("A::compile_fail/bad", Kind::CompileFail);
    assert!(select(&golden, &["compile_fail/bad".into()], false).is_some());
}

#[test]
fn paths_among_the_flags_are_resolved_from_the_current_directory() {
    let args = TestArgs {
        path: vec![PathBuf::from("lib")],
        junit: Some(PathBuf::from("out.xml")),
        jaic_args: [
            "-O2",
            "-import_dir",
            "vendor",
            "-emit_ir=x.ll",
            "-add",
            "-output_path",
            "-output_path=out",
            "-os",
            "wasm",
        ]
        .map(String::from)
        .into(),
        ..<TestArgs as clap::Args>::augment_args(clap::Command::new("t"))
            .try_get_matches_from(["t"])
            .map(|m| <TestArgs as clap::FromArgMatches>::from_arg_matches(&m).unwrap())
            .unwrap()
    };
    let resolved = resolve_paths(&args, Path::new("/w"));
    assert_eq!(resolved.path, [PathBuf::from("/w/lib")]);
    assert_eq!(resolved.junit, Some(PathBuf::from("/w/out.xml")));
    assert_eq!(
        resolved.jaic_args,
        [
            "-O2",
            "-import_dir",
            "/w/vendor",
            "-emit_ir=/w/x.ll",
            "-add",
            "-output_path",
            "-output_path=/w/out",
            "-os",
            "wasm"
        ]
    );
    for boundary in ["-", "--", "+Plugin"] {
        let mut bounded = args.clone();
        bounded.jaic_args = [boundary, "-import_dir", "keep-relative"]
            .map(String::from)
            .into();
        assert_eq!(
            resolve_paths(&bounded, Path::new("/w")).jaic_args,
            bounded.jaic_args
        );
    }
}

#[test]
fn partitions_cover_every_test_once() {
    let names = ["a", "b", "c", "d", "e"];
    for how in ["hash", "count"] {
        let parts: Vec<_> = (1..=3)
            .map(|k| partition(&format!("{how}:{k}/3")).unwrap())
            .collect();
        for (i, name) in names.iter().enumerate() {
            assert_eq!(
                parts.iter().filter(|p| p(i, name)).count(),
                1,
                "{how} {name}"
            );
        }
    }
    assert!(partition("hash:0/3").is_err());
    assert!(partition("count:4/3").is_err());
    assert!(partition("mod:1/2").is_err());
}
