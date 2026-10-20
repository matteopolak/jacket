//! Placeholder binding and deferred-body dependency regressions.
mod common;
use common::compile_main_file as compile;
use jaic::sema::{Compiler, Options, ProgramSource, VirtualFs};
use std::rc::Rc;


#[test]
fn placeholder_accepts_one_definition_and_keeps_normal_duplicate_errors() {
    compile("#placeholder Value; Value :: 41; #assert Value == 41; main :: () {}").unwrap();
    for source in [
        "#placeholder Value; Value :: 41; Value :: 42; main :: () {}",
        "Value :: 41; Value :: 42; main :: () {}",
    ] {
        let error = compile(source).expect_err("duplicate definitions must fail");
        assert!(error.contains("already declared"), "{error}");
    }
}

#[test]
fn placeholder_definition_does_not_disable_procedure_overload_rules() {
    compile("#placeholder f; f :: (x: int) -> int { return x; } f :: (x: bool) -> bool { return x; } #assert f(41) == 41; #assert f(true); main :: () {}").unwrap();
}

#[test]
fn undeclared_names_and_unfulfilled_placeholders_keep_diagnostics() {
    for (source, expected) in [
        ("main :: () { absent(); }", "unknown identifier"),
        (
            "#placeholder Value; main :: () { x := Value; }",
            "#placeholder",
        ),
    ] {
        let error = compile(source).expect_err("unavailable declarations must fail");
        assert!(error.contains(expected), "{error}");
    }
}

#[test]
fn typed_procedure_value_is_available_while_its_body_waits() {
    let mut options = Options::host();
    options.runtime_support = false;
    let mut compiler = Compiler::new(options, Rc::new(VirtualFs::default()));
    let source = "#placeholder Body; Callback :: #type () -> int; delayed :: () -> int { #insert Body; } available :: #run () -> bool { callback: Callback = delayed; return callback != null; }(); #assert available; main :: () {}";
    if let Err(error) = compiler.begin_sources(&[ProgramSource::String(source.into())]) {
        panic!("{}", error.render(&compiler.sources));
    }
    let error = compiler
        .finish_program()
        .expect_err("the body still needs its definition");
    assert!(
        error.message.contains("#placeholder"),
        "{}",
        error.render(&compiler.sources)
    );
}

#[test]
fn editor_diagnostics_from_deferred_attempts_do_not_survive_successful_binding() {
    let mut options = Options::host();
    options.runtime_support = false;
    let mut compiler = Compiler::new(options, Rc::new(VirtualFs::default()));
    compiler
        .ed
        .record(jaic::ide::IdeFacts::new(vec![jaic::ide::IdeRoot::Virtual]));
    compiler.begin_sources(&[ProgramSource::String(
        "#placeholder Body; delayed :: () -> int { #insert Body; } main :: () -> int { return delayed(); }".into()
    )]).unwrap();
    let entity = compiler
        .entities
        .iter()
        .position(|entity| entity.name.as_str() == "delayed")
        .unwrap();
    let jaic::sema::scope::Resolved::Proc(procedure) = compiler
        .resolve_entity(jaic::sema::EntityId(entity as u32))
        .unwrap()
    else {
        panic!("declared procedure");
    };
    compiler
        .proc_func(procedure, jaic::source::Span::NONE)
        .unwrap();
    assert!(compiler.drain_bodies_lenient().is_some());
    compiler
        .add_source(&ProgramSource::String("Body :: \"return 42;\";".into()))
        .unwrap();
    compiler.finish_program().unwrap();
    let diagnostics = compiler.ide_check_all();
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
}

#[test]
fn scoped_placeholder_definitions_preserve_overload_sets_for_other_files() {
    let mut fs = VirtualFs::default();
    fs.insert(
        "/main.jai",
        b"#placeholder f; anchor :: #code f; #load \"other.jai\"; main :: () {}".as_slice(),
    );
    fs.insert(
        "/other.jai",
        b"#assert f(41) == 41; #assert f(true);".as_slice(),
    );
    let mut options = Options::host();
    options.runtime_support = false;
    let mut compiler = Compiler::new(options, Rc::new(fs));
    compiler
        .begin_sources(&[ProgramSource::File("/main.jai".into())])
        .unwrap();
    let entity = compiler
        .entities
        .iter()
        .position(|entity| entity.name.as_str() == "anchor")
        .unwrap();
    let jaic::sema::scope::Resolved::Const {
        value: jaic::sema::Value::Code(code),
        ..
    } = compiler
        .resolve_entity(jaic::sema::EntityId(entity as u32))
        .unwrap()
    else {
        panic!("scope Code");
    };
    compiler
        .add_source(&ProgramSource::CodeString(
            "f :: (x: int) -> int { return x; } f :: (x: bool) -> bool { return x; }".into(),
            code,
        ))
        .unwrap();
    if let Err(error) = compiler.finish_program() {
        panic!("{}", error.render(&compiler.sources));
    }
}

#[test]
fn an_unstarted_insertion_generator_waits_for_later_source_without_execution() {
    let mut options = Options::host();
    options.runtime_support = false;
    let mut compiler = Compiler::new(options, Rc::new(VirtualFs::default()));
    let source =
        "#placeholder Body; #insert -> string { return Body; } main :: () { #assert Ready; }";
    if let Err(error) = compiler.begin_sources(&[ProgramSource::String(source.into())]) {
        panic!("{}", error.render(&compiler.sources));
    }
    assert!(
        compiler.meta.perf.runs.is_empty(),
        "semantic waiting must precede thunk execution"
    );
    compiler
        .add_source(&ProgramSource::String("Body :: \"Ready :: true;\";".into()))
        .unwrap();
    if let Err(error) = compiler.finish_program() {
        panic!("{}", error.render(&compiler.sources));
    }
}

#[test]
fn module_insertion_generator_waits_in_its_canonical_module_scope() {
    let mut fs = VirtualFs::default();
    fs.insert(
        "/lib/Deferred/module.jai",
        b"#placeholder Body; #insert -> string { return Body; }".as_slice(),
    );
    fs.insert(
        "/main.jai",
        b"M :: #import \"Deferred\"; main :: () { value:=M.answer(); }".as_slice(),
    );
    let mut options = Options::host();
    options.runtime_support = false;
    options.import_paths = vec!["/lib".into()];
    let mut compiler = Compiler::new(options, Rc::new(fs));
    if let Err(error) = compiler.begin_sources(&[ProgramSource::File("/main.jai".into())]) {
        panic!("{}", error.render(&compiler.sources));
    }
    assert!(compiler.meta.perf.runs.is_empty());
    let module = compiler
        .modules
        .iter()
        .position(|m| m.name.as_str() == "Deferred")
        .unwrap();
    compiler
        .add_source(&ProgramSource::ModuleString(
            "Body :: \"answer :: () -> int {return 42;}\";".into(),
            module as u32,
        ))
        .unwrap();
    if let Err(error) = compiler.finish_program() {
        panic!("{}", error.render(&compiler.sources));
    }
}

#[test]
fn a_parameter_naming_an_undefined_placeholder_is_decided_again_once_it_is_defined() {
    let mut options = Options::host();
    options.runtime_support = false;
    let mut compiler = Compiler::new(options, Rc::new(VirtualFs::default()));
    compiler
        .begin_sources(&[ProgramSource::String(
            "#placeholder Vec; takes :: (v: *Vec) {} main :: () {}".into(),
        )])
        .unwrap();
    let procedure = |compiler: &mut Compiler| {
        let entity = compiler
            .entities
            .iter()
            .position(|entity| entity.name.as_str() == "takes")
            .unwrap();
        let jaic::sema::scope::Resolved::Proc(procedure) = compiler
            .resolve_entity(jaic::sema::EntityId(entity as u32))
            .unwrap()
        else {
            panic!("declared procedure");
        };
        procedure
    };
    let takes = procedure(&mut compiler);
    compiler.refresh_implicit_poly(takes).unwrap();
    assert!(
        !compiler.proc(takes).is_poly,
        "nothing says it is polymorphic yet"
    );
    compiler
        .add_source(&ProgramSource::String(
            "Vec :: struct(T: Type) { item: T; }".into(),
        ))
        .unwrap();
    compiler.refresh_implicit_poly(takes).unwrap();
    assert!(
        compiler.proc(takes).is_poly,
        "the first answer, given before `Vec` existed, must not be final"
    );
}

#[test]
fn an_overload_set_with_an_unresolved_member_is_compared_again_once_it_resolves() {
    let mut options = Options::host();
    options.runtime_support = false;
    let mut compiler = Compiler::new(options, Rc::new(VirtualFs::default()));
    compiler
        .begin_sources(&[ProgramSource::String(
            "#placeholder Alias; f :: (a: Alias) {} f :: (a: s64) {} main :: () {}".into(),
        )])
        .unwrap();
    let set: Vec<_> = compiler
        .entities
        .iter()
        .enumerate()
        .filter(|(_, entity)| entity.name.as_str() == "f")
        .map(|(index, _)| jaic::sema::EntityId(index as u32))
        .collect();
    assert_eq!(set.len(), 2);
    assert!(
        compiler.check_identical_overloads(&set).is_ok(),
        "`Alias` is not defined yet, so the two cannot be told to be identical"
    );
    compiler
        .add_source(&ProgramSource::String("Alias :: s64;".into()))
        .unwrap();
    let error = compiler
        .check_identical_overloads(&set)
        .expect_err("the set was not fully compared the first time");
    assert!(
        error.message.contains("identical parameter types"),
        "{}",
        error.message
    );
}
