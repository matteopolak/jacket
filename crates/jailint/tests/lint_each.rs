//! `lint_each` hands over each file's lints as the file finishes, so a stopped pass keeps what
//! it completed. The interrupt state is process-wide: one test, run in order.
use jaic::ide::{IdeFacts, IdeRoot};
use jaic::sema::{Compiler, NativeFs, Options, ProgramSource};
use std::rc::Rc;

const SOURCE: &str = "\
helper :: (a: int) -> int { unused_a := 1; return a + 1; }
main :: () { unused_b := 2; helper(3); }
";

fn compile() -> (Compiler, jaic::source::FileId) {
    let stdlib = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    let mut options = Options::host();

    options.import_paths = vec![stdlib.clone()];
    options.set_stdlib(&NativeFs, &stdlib);
    let mut c = Compiler::new(options, Rc::new(NativeFs));
    let mut facts = IdeFacts::new(vec![IdeRoot::Virtual]);

    facts.lint = true;
    c.ed.record(facts);
    c.begin_sources(&[ProgramSource::String(SOURCE.into())])
        .unwrap();
    c.finish_program().unwrap();
    let _ = c.ide_check_all();
    let file = c
        .src
        .files
        .iter()
        .map(|f| f.id)
        .find(|&id| c.src.sources.get(id).text.as_ref() == SOURCE)
        .expect("the program's own file");

    (c, file)
}

#[test]
fn each_file_is_handed_over_when_done_and_a_stop_hands_over_nothing_partial() {
    let (mut c, file) = compile();
    let configs = jailint::config::Configs::new(&jailint::config::Config::default());
    let all = jailint::lint_files(&mut c, &[file], &configs, true);

    assert!(
        all.iter().any(|l| l.rule == "unused_variable"),
        "the program has findings: {all:?}"
    );

    // Everything pending: the file's lints arrive once, equal to the whole pass's.
    let mut seen = Vec::new();
    let done = jailint::lint_each(
        &mut c,
        &[file],
        &configs,
        true,
        None,
        &|_| true,
        &mut |f, found| seen.push((f, found)),
    );

    assert!(done);
    assert_eq!(seen, [(file, all.clone())]);

    // Nothing pending: nothing is linted.
    let mut seen = Vec::new();
    let done = jailint::lint_each(
        &mut c,
        &[file],
        &configs,
        true,
        None,
        &|_| false,
        &mut |f, found| seen.push((f, found)),
    );

    assert!(done && seen.is_empty());

    // A stop in the middle of the file reports it unfinished and hands over none of its lints.
    for polls in [1, 2, 5, 20, 80] {
        let mut seen = Vec::new();

        jaic::interrupt::request(false);
        jaic::interrupt::arm();
        jaic::interrupt::stop_after(polls);
        let done = jailint::lint_each(
            &mut c,
            &[file],
            &configs,
            true,
            None,
            &|_| true,
            &mut |f, found| seen.push((f, found)),
        );
        let interrupted = jaic::interrupt::disarm();

        jaic::interrupt::stop_after(0);
        assert_eq!(done, !interrupted, "{polls} polls");

        if done {
            assert_eq!(seen, [(file, all.clone())], "{polls} polls");
        } else {
            assert!(seen.is_empty(), "{polls} polls: {seen:?}");
        }
    }
}
