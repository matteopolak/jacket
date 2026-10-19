//! Overload clashes that a call must not hide.
use jaic::sema::{Compiler, Options, ProgramSource, VirtualFs};
use std::rc::Rc;

fn compile(source: &str) -> Result<(), String> {
    let mut options = Options::host();
    options.runtime_support = false;
    let mut compiler = Compiler::new(options, Rc::new(VirtualFs::default()));
    compiler
        .compile_sources(&[ProgramSource::String(source.into())])
        .map_err(|error| error.render(&compiler.sources))
}

#[test]
fn identical_overloads_are_rejected_even_when_a_call_names_them() {
    let error = compile("f :: (a: int) {}\nf :: (a: int) {}\nmain :: () { f(1); }\n").unwrap_err();
    assert!(error.contains("identical parameter types"), "{error}");
}

#[test]
fn aliases_of_one_procedure_are_a_single_overload() {
    let source = "Impl :: struct { read_it :: () {} }\n\
                  read_it :: Impl.read_it;\n\
                  #if true { read_it :: Impl.read_it; }\n\
                  main :: () { read_it(); }\n";
    compile(source).unwrap();
}

#[test]
fn different_procedures_with_identical_parameters_are_still_rejected() {
    let source = "A :: struct { f :: (a: int) {} }\n\
                  B :: struct { f :: (a: int) {} }\n\
                  f :: A.f;\n\
                  f :: B.f;\n\
                  main :: () { f(1); }\n";
    let error = compile(source).unwrap_err();
    assert!(error.contains("identical parameter types"), "{error}");
}

#[test]
fn module_parameters_are_rejected_in_a_loaded_program_file() {
    let mut options = Options::host();
    options.runtime_support = false;
    let mut fs = VirtualFs::default();
    fs.insert("/p/params.jai", b"#module_parameters(N := 3);\n".as_slice());
    let mut compiler = Compiler::new(options, Rc::new(fs));
    let error = compiler
        .compile_sources(&[ProgramSource::String(
            "#load \"/p/params.jai\";\nmain :: () {}\n".into(),
        )])
        .map(|_| ())
        .map_err(|error| error.render(&compiler.sources))
        .unwrap_err();
    assert!(error.contains("top level of an imported module"), "{error}");
}

#[test]
fn module_parameters_are_rejected_in_the_main_program_file() {
    let error = compile("#module_parameters(N := 3);\nmain :: () {}\n").unwrap_err();
    assert!(error.contains("top level of an imported module"), "{error}");
}
