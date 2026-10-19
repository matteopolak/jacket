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

fn failed() -> (Compiler, ProcId) {
    let mut fs = VirtualFs::default();
    fs.insert("/audit/main.jai", b"broken :: missing; failed :: () -> s64 { nested :: () {}; nested(); return absent; } replacement :: () -> s64 { return 7; } main :: () {}".as_slice());
    let mut options = Options::host();
    options.runtime_support = false;
    options.stack_trace = false;
    let mut compiler = Compiler::new(options, Rc::new(fs));
    compiler.ide = Some(Box::new(IdeFacts::new(vec![IdeRoot::under("/audit/")])));
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
        .procs
        .iter()
        .find(|p| p.name.as_str() == "replacement")
        .unwrap()
        .lit
        .clone();
    let header = compiler.proc(id).lit.header.clone();
    compiler.procs[id.0 as usize].lit = Rc::new(ast::ProcLit {
        header,
        body: replacement.body.clone(),
    });
    compiler.lower_body(id).unwrap();
    let Some(super::procs::ProcTarget::Func(func)) = compiler.proc(id).target else {
        panic!("compiled target");
    };
    assert!(compiler.program.funcs[func.0 as usize].is_some());
}

#[test]
fn replacing_a_parked_body_retries_during_the_next_audit() {
    let (mut compiler, id) = failed();
    compiler.ide_check_all();
    assert!(compiler.parked_bodies.contains(&id));
    let replacement = compiler
        .procs
        .iter()
        .find(|p| p.name.as_str() == "replacement")
        .unwrap()
        .lit
        .clone();
    let header = compiler.proc(id).lit.header.clone();
    compiler.procs[id.0 as usize].lit = Rc::new(ast::ProcLit {
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
    assert!(compiler.program.funcs[func.0 as usize].is_some());
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
    assert!(compiler.program.funcs[func.0 as usize].is_some());
}

#[test]
fn removing_ide_facts_keeps_strict_lowering_unmemoized() {
    let (mut compiler, id) = failed();
    let before = compiler
        .program
        .func_names
        .iter()
        .filter(|n| n.as_str() == "nested")
        .count();
    compiler.ide = None;
    assert!(
        compiler
            .lower_body(id)
            .unwrap_err()
            .message
            .contains("absent")
    );
    let after = compiler
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
