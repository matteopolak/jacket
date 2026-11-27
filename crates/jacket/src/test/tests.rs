use super::partition::{parse, partition, time_part, time_parts};
use super::*;
use jaic::fxhash::HashMap;

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
fn time_parts_split_the_recorded_time_evenly_and_cover_every_test_once() {
    let times: HashMap<String, f64> = [
        ("slow", 10.0),
        ("mid_a", 4.0),
        ("mid_b", 4.0),
        ("quick_a", 1.0),
        ("quick_b", 1.0),
    ]
    .into_iter()
    .map(|(name, seconds)| (name.to_string(), seconds))
    .collect();
    let names = ["quick_a", "slow", "mid_a", "quick_b", "mid_b", "unrecorded"];
    let parts = time_parts(&names, &times, 2);
    // Each test lands in one part, whatever order the names are listed in.
    assert_eq!(parts.len(), names.len());
    assert!(parts.iter().all(|part| *part < 2));
    let mut reversed = names;
    reversed.reverse();
    let again = time_parts(&reversed, &times, 2);
    for (name, part) in names.iter().zip(&parts) {
        let other = reversed.iter().position(|n| n == name).unwrap();
        assert_eq!(*part, again[other], "{name}");
    }
    // The slow test fills one part alone; the rest share the other (10 against 10 plus the median).
    let load = |part: usize| {
        names
            .iter()
            .zip(&parts)
            .filter(|(_, p)| **p == part)
            .map(|(name, _)| times.get(*name).copied().unwrap_or(4.0))
            .sum::<f64>()
    };
    assert!(
        (load(0) - load(1)).abs() <= 4.0,
        "{} against {}",
        load(0),
        load(1)
    );
    assert_eq!(time_part("time:1/2"), Ok((0, 2)));
    assert!(time_part("time:3/2").is_err());
    assert!(time_part("hash:1/2").is_err());
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

#[test]
fn a_times_file_goes_only_with_a_time_split() {
    let times = std::path::Path::new("times.json");
    let error = parse(Some("time:1/2"), None)
        .err()
        .expect("time needs the file");
    assert!(error.contains("--partition-times"), "{error}");
    let error = parse(Some("hash:1/2"), Some(times))
        .err()
        .expect("hash takes no file");
    assert!(error.contains("only with `time:K/N`"), "{error}");
    let error = parse(None, Some(times))
        .err()
        .expect("no split takes no file");
    assert!(error.contains("only with `time:K/N`"), "{error}");
    assert!(parse(Some("hash:1/2"), None).is_ok());
}

#[test]
fn a_board_skips_the_cases_that_run_a_host_program() {
    let skip = |kind| case("A::x", kind).board_skip(&[], false);
    assert!(skip(Kind::Golden).is_some());
    assert!(skip(Kind::Test).is_none());
    assert!(skip(Kind::Bench).is_none());
    // A compile-fail test needs no runner, so it is not skipped.
    assert!(skip(Kind::CompileFail).is_none());
    assert!(case("A::x", Kind::Golden).board_skip(&[], true).is_none());
}
