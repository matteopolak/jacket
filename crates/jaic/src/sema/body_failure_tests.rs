//! Re-checking bodies that failed in an editor analysis.
use super::scope::EntityKind;
use super::value::ProcId;
use super::{Compiler, Options, Value, VirtualFs};
use crate::ast;
use crate::ide::{IdeFacts, IdeRoot};
use crate::intern::Sym;
use crate::source::Span;
use crate::types::TypeId;
use std::rc::Rc;
use std::sync::Arc;

fn failed() -> (Compiler, ProcId) {
    let mut fs = VirtualFs::default();
    fs.insert("/audit/main.jai", b"broken :: missing; failed :: () -> s64 { nested :: () {}; nested(); return absent; } replacement :: () -> s64 { return 7; } main :: () {}".as_slice());
    let mut options = Options::host();
    options.runtime_support = false;
    options.stack_trace = false;
    let mut compiler = Compiler::new(options, Rc::new(fs));
    compiler
        .ed
        .record(IdeFacts::new(vec![IdeRoot::under("/audit/")]));
    assert!(
        compiler
            .compile_program(std::path::Path::new("/audit/main.jai"))
            .is_err()
    );
    assert!(
        compiler
            .ide_check_all()
            .iter()
            .any(|e| e.message.contains("absent"))
    );
    let at = compiler
        .pr
        .procs
        .iter()
        .position(|p| p.name.as_str() == "failed")
        .unwrap();
    (compiler, ProcId(at as u32))
}

#[test]
fn replacing_body_retries_without_source_epoch_change() {
    let (mut compiler, id) = failed();
    assert!(compiler.lower_body(id).is_err());
    let replacement = compiler
        .pr
        .procs
        .iter()
        .find(|p| p.name.as_str() == "replacement")
        .unwrap()
        .lit
        .clone();
    let header = compiler.proc(id).lit.header.clone();
    compiler.pr.procs[id.0 as usize].lit = Arc::new(ast::ProcLit {
        header,
        body: replacement.body.clone(),
    });
    compiler.lower_body(id).unwrap();
    let Some(super::procs::ProcTarget::Func(func)) = compiler.proc(id).target else {
        panic!("compiled target");
    };
    assert!(compiler.out.program.funcs[func.0 as usize].is_some());
}

#[test]
fn replacing_a_parked_body_retries_during_the_next_audit() {
    let (mut compiler, id) = failed();
    compiler.ide_check_all();
    assert!(compiler.pr.parked_bodies.contains(&id));
    let replacement = compiler
        .pr
        .procs
        .iter()
        .find(|p| p.name.as_str() == "replacement")
        .unwrap()
        .lit
        .clone();
    let header = compiler.proc(id).lit.header.clone();
    compiler.pr.procs[id.0 as usize].lit = Arc::new(ast::ProcLit {
        header,
        body: replacement.body.clone(),
    });
    let errors = compiler.ide_check_all();
    assert!(
        !errors.iter().any(|e| e.message.contains("absent")),
        "{errors:?}"
    );
    let Some(super::procs::ProcTarget::Func(func)) = compiler.proc(id).target else {
        panic!("compiled target");
    };
    assert!(compiler.out.program.funcs[func.0 as usize].is_some());
}

#[test]
fn adding_visible_binding_retries_without_loading_a_file() {
    let (mut compiler, id) = failed();
    let scope = compiler.scope(compiler.proc(id).scope).parent.unwrap();
    let name = Sym::intern("absent");
    compiler.add_entity(
        scope,
        name,
        Span::NONE,
        EntityKind::Const {
            value: Value::Int(7),
            ty: TypeId::S64,
        },
        false,
    );
    compiler.lower_body(id).unwrap();
    let Some(super::procs::ProcTarget::Func(func)) = compiler.proc(id).target else {
        panic!("compiled target");
    };
    assert!(compiler.out.program.funcs[func.0 as usize].is_some());
}

#[test]
fn removing_ide_facts_keeps_strict_lowering_unmemoized() {
    let (mut compiler, id) = failed();
    let before = compiler
        .out
        .program
        .func_names
        .iter()
        .filter(|n| n.as_str() == "nested")
        .count();
    compiler.ed.stop();
    assert!(
        compiler
            .lower_body(id)
            .unwrap_err()
            .message
            .contains("absent")
    );
    let after = compiler
        .out
        .program
        .func_names
        .iter()
        .filter(|n| n.as_str() == "nested")
        .count();
    assert!(
        after > before,
        "strict lowering must really execute, not consult IDE failure facts"
    );
}

/// `bad<i>` traps in its `#insert`; `good<i>` lowers fine. Both are exports, so a lenient
/// lowering takes them up.
fn failing_and_good_bodies(n: usize) -> String {
    let mut source = String::new();
    for i in 0..n {
        source += &format!(
            "bad{i} :: () -> s64 #program_export {{ #insert -> string {{ a: [2] s64; k := {i} + 5; a[k] = 1; return \"inserted := {i};\"; }} return {i}; }}\n"
        );
    }
    for i in 0..n {
        source += &format!("good{i} :: () -> s64 #program_export {{ return {i}; }}\n");
    }
    source += "main :: () {";
    for i in 0..n {
        source += &format!(" bad{i}(); good{i}();");
    }
    source + " }"
}

/// The compile-time runs done by `rounds` calls of `lower_reachable`, the way a metaprogram
/// that keeps asking for more lowering makes them, over `n` bodies whose `#insert` traps.
fn runs_over_rounds(n: usize, rounds: usize) -> u64 {
    let mut options = Options::host();
    options.runtime_support = false;
    options.stack_trace = false;
    let mut compiler = Compiler::new(options, Rc::new(VirtualFs::default()));
    compiler
        .begin_sources(&[super::ProgramSource::String(
            failing_and_good_bodies(n).into(),
        )])
        .unwrap();
    for _ in 0..rounds {
        compiler.lower_reachable();
    }
    compiler.ct.executions
}

#[test]
fn a_body_whose_insert_trapped_is_not_run_again_while_nothing_changed() {
    let (one, many) = (runs_over_rounds(4, 1), runs_over_rounds(4, 6));
    assert_eq!(many, one, "later rounds must not redo the failing bodies");
}
