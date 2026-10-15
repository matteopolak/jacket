//! Independent ordinary variadic header/call diagnostics; no native execution.
use jaic::sema::{Compiler, Options, ProgramSource, VirtualFs};
use std::rc::Rc;

#[path = "support/reflection_bootstrap.rs"]
mod bootstrap;

fn compiler() -> Compiler {
    let mut options = Options::host();
    options.runtime_support = false;
    Compiler::new(options, Rc::new(VirtualFs::default()))
}

#[test]
fn c_variadic_before_fixed_parameter_is_a_diagnostic_not_a_lowering_panic() {
    for declaration in [
        "broken :: (items: ..int, tail:int) #c_call {}",
        "broken :: (items: ..int, tail:int) #foreign;",
        "broken :: (items: ..$T, tail:int) #c_call {}",
        "broken :: (items: ..int, more: ..int) #c_call {}",
    ] {
        let source = format!("{declaration}\nmain :: () {{ broken(1, tail=2); }}");
        let mut c = compiler();
        let error = c
            .compile_sources(&[ProgramSource::String(source)])
            .expect_err("invalid C variadic header must diagnose");
        assert!(
            error.message.contains("variadic") && error.message.contains("last"),
            "{}",
            error.render(&c.sources)
        );
    }
}

#[test]
fn c_procedure_types_reject_nonfinal_variadics() {
    let mut c = compiler();
    let error = c
        .compile_sources(&[ProgramSource::String(
            "Bad :: #type (items: ..int, tail:int) #c_call; value:Bad; main :: () {}".into(),
        )])
        .expect_err("invalid C procedure type must diagnose");
    assert!(
        error.message.contains("variadic") && error.message.contains("last"),
        "{}",
        error.render(&c.sources)
    );
}

#[test]
fn module_type_arguments_do_not_bypass_c_variadic_header_validation() {
    let mut fs = VirtualFs::default();
    fs.insert(
        "/call-tests/modules/Calls/module.jai",
        "#module_parameters(Item:Type); broken :: (items: ..Item, tail:int) #c_call {}".as_bytes(),
    );
    fs.insert(
        "/call-tests/main.jai",
        "M :: #import \"Calls\"(Item=int); main :: () {M.broken(1,tail=2);}".as_bytes(),
    );
    let mut options = Options::host();
    options.runtime_support = false;
    options.import_paths = vec!["/call-tests/modules".into()];
    let mut c = Compiler::new(options, Rc::new(fs));
    let error = c
        .compile_program(std::path::Path::new("/call-tests/main.jai"))
        .expect_err("invalid imported C signature must diagnose");
    assert!(
        error.message.contains("variadic") && error.message.contains("last"),
        "{}",
        error.render(&c.sources)
    );
}

#[test]
fn valid_c_varargs_and_baked_type_packs_are_distinct_from_jai_runtime_packs() {
    let source = r#"
    foreign_call :: (tag:int, items: ..int) #foreign;
    identity :: ($types: ..Type, value:int) -> int #c_call {return value;}
    main :: () { foreign_call(1,2,3); value:=identity(int,value=4); }
    "#;
    let mut fs = VirtualFs::default();
    fs.insert(
        "/call-tests/prelude/module.jai",
        bootstrap::BOOTSTRAP.as_bytes(),
    );
    let mut options = Options::host();
    options.runtime_support = false;
    options.preload = Some("/call-tests/prelude/module.jai".into());
    let mut c = Compiler::new(options, Rc::new(fs));
    if let Err(error) = c.compile_sources(&[ProgramSource::String(source.into())]) {
        panic!("{}", error.render(&c.sources));
    }
}

#[test]
fn ordinary_variadic_with_named_tail_and_valid_overloads_still_typechecks() {
    let source = r#"
    sum :: (items: ..int, tail:int=4) -> int {
        result:=tail; for items result+=it; return result;
    }
    pick :: (value:int) -> int {return value+1;}
    pick :: (value:float) -> float {return value+0.5;}
    #assert(sum(1,2,tail=3)==6);
    #assert(sum(1,2)==7);
    #assert(pick(2)==3);
    #assert(pick(2.0)==2.5);
    main :: () {}
    "#;
    let mut c = compiler();
    if let Err(error) = c.compile_sources(&[ProgramSource::String(source.into())]) {
        panic!("{}", error.render(&c.sources));
    }
}
