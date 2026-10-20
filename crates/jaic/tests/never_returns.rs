//! Procedures that never return end their callers' blocks: the code after a call to one is
//! recorded as dead for the editor and the linter.
use jaic::ide::{IdeFacts, IdeRoot};
use jaic::sema::{Compiler, Options, ProgramSource, VirtualFs};
use std::rc::Rc;

/// The text of every dead run of `source` (as the editor would fade it), in source order.
fn dead_runs(source: &str) -> Vec<String> {
    let mut options = Options::host();
    options.runtime_support = false;
    let mut compiler = Compiler::new(options, Rc::new(VirtualFs::default()));
    compiler.ed.record(IdeFacts::new(vec![IdeRoot::Virtual]));
    compiler
        .begin_sources(&[ProgramSource::String(source.into())])
        .unwrap();
    let _ = compiler.finish_program();
    let errors = compiler.ide_check_all();
    assert!(errors.is_empty(), "{errors:?}");
    let ide = compiler.ed.facts().unwrap();
    let mut runs: Vec<(u32, u32)> = ide
        .dead_runs
        .iter()
        .filter(|((body, _), n)| ide.body_evals.get(body) == Some(*n))
        .map(|((_, run), _)| (run.start, run.end))
        .collect();
    runs.sort_unstable();
    runs.dedup();
    runs.iter()
        .map(|&(start, end)| source[start as usize..end as usize].to_string())
        .collect()
}

const EXIT: &str = "exit :: (status: s32) #foreign;\n";

#[test]
fn a_call_to_an_exit_primitive_ends_the_block() {
    let source = format!("{EXIT}main :: () {{ exit(1); dead := 1; }}\n");
    assert_eq!(dead_runs(&source), ["dead := 1"]);
}

#[test]
fn procedures_that_end_in_such_a_call_are_found_through_their_callers() {
    let source = format!(
        "{EXIT}\
inner :: () {{ exit(1); }}
middle :: () {{ inner(); }}
outer :: () {{ middle(); after_middle := 1; }}
main :: () {{ outer(); }}
"
    );
    assert_eq!(dead_runs(&source), ["after_middle := 1"]);
}

#[test]
fn a_procedure_with_a_returning_path_is_not_a_dead_end() {
    let source = format!(
        "{EXIT}\
sometimes :: (flag: bool) {{ if flag {{ exit(1); }} }}
looping :: () {{ looping(); }}
main :: () {{ sometimes(false); a := 1; looping(); b := 2; }}
"
    );
    assert_eq!(dead_runs(&source), Vec::<String>::new());
}

#[test]
fn instances_of_a_polymorphic_procedure_that_disagree_prove_nothing() {
    let source = format!(
        "{EXIT}\
fail :: () {{ exit(1); }}
pick :: (x: $T) {{ #if T == s64 {{ fail(); }} }}
main :: () {{ pick(1); pick(true); live := 1; }}
"
    );
    assert_eq!(dead_runs(&source), Vec::<String>::new());
}
