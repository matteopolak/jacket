//! A constant, procedure or type of a body is visible only below the line that declares it, as in
//! the official compiler (`docs/language/scoping.md`, rule scope.19).
use jaic::diagnostics::{Code, Diagnostic};
use jaic::sema::{Compiler, Options, VirtualFs};
use std::{path::Path, rc::Rc};

fn compile(main: &str, modules: &[(&str, &str)]) -> Result<(), Box<Diagnostic>> {
    let mut fs = VirtualFs::default();
    for (name, source) in modules {
        fs.insert(format!("/modules/{name}/module.jai"), source.as_bytes());
    }
    fs.insert("/main.jai", main.as_bytes());
    let mut options = Options::host();
    options.runtime_support = false;
    options.import_paths = vec![Path::new("/modules").to_path_buf()];
    options.preload_stdlib = Some(Path::new("/modules").to_path_buf());
    let mut compiler = Compiler::new(options, Rc::new(fs));
    compiler.compile_program(Path::new("/main.jai"))
}

fn rejected(main: &str) -> Diagnostic {
    let d = compile(main, &[]).expect_err("must be rejected");
    assert_eq!(d.code, Code::UseBeforeDeclaration, "{d:?}");
    *d
}

// rules: scope.19
#[test]
fn a_procedure_called_above_its_declaration_is_an_error() {
    let d = rejected("main :: () { foo(); foo :: () { } }");
    assert_eq!(d.message, "`foo` is used above its declaration");
    assert_eq!(d.code.name(), "use-before-declaration");
    assert!(d.notes.iter().any(|(_, n)| n == "declared here"));
}

#[test]
fn mutually_recursive_local_procedures_are_an_error() {
    rejected(
        "main :: () {
            even :: (n: int) -> bool { if n == 0 return true; return odd(n - 1); }
            odd :: (n: int) -> bool { if n == 0 return false; return even(n - 1); }
        }",
    );
}

#[test]
fn a_type_and_a_constant_used_above_their_declarations_are_errors() {
    rejected("main :: () { x: T; T :: struct { a: int = 3; } y := x.a; }");
    rejected("main :: () { A :: B; B :: 1; y := A; }");
}

#[test]
fn compile_time_uses_above_the_declaration_are_errors() {
    rejected("main :: () { v :: #run g(); g :: () -> int { return 5; } y := v; }");
    rejected("main :: () { #assert K == 3; K :: 3; y := K; }");
    rejected("main :: () { #if K == 3 { } K :: 3; y := K; }");
    rejected("main :: () { #insert \"x := Q;\"; Q :: 9; y := Q; }");
}

#[test]
fn a_use_in_a_body_two_local_procedures_deep_is_not_checked() {
    compile(
        "main :: () {
            outer :: () -> int {
                inner :: () -> int { return later(); }
                return inner();
            }
            later :: () -> int { return 7; }
            x := outer();
        }",
        &[],
    )
    .expect("accepted");
}

#[test]
fn a_declaration_above_its_use_is_fine() {
    compile(
        "main :: () { limit :: 3; bar :: () { } foo :: () { bar(); } x := limit; foo(); }",
        &[],
    )
    .expect("accepted");
}

#[test]
fn the_same_error_is_reported_in_an_imported_module() {
    let d = compile(
        "#import \"Lib\";\nmain :: () { go(); }",
        &[("Lib", "go :: () { foo(); foo :: () { } }")],
    )
    .expect_err("must be rejected");
    assert_eq!(d.code, Code::UseBeforeDeclaration, "{d:?}");
}

#[test]
fn struct_bodies_are_declarative_even_inside_ordered_procedures() {
    for parameter in ["", "(N:int)"] {
        let instance = if parameter.is_empty() {
            "Box"
        } else {
            "Box(1)"
        };
        for before in [true, false] {
            let declaration = "later :: 3; make :: ()->string {return \"value:int = 42;\";}";
            let object = format!("box:{instance};");
            let tail = if before {
                format!("{object} {declaration}")
            } else {
                format!("{declaration} {object}")
            };
            compile(&format!("main :: () {{ Box :: struct{parameter} {{ cells:[later]int; #insert #run make(); }} {tail} }}"), &[]).unwrap();
        }
    }
}

#[test]
fn generic_headers_observe_the_instantiation_site() {
    compile(
        "main :: () { Box :: struct(N:Later) { x:int; } Later :: int; box:Box(1); }",
        &[],
    )
    .unwrap();
    rejected("main :: () { Box :: struct(N:Later) { x:int; } box:Box(1); Later :: int; }");
}

#[test]
fn declarative_scopes_do_not_relax_order_inside_methods_or_enums() {
    rejected("main :: () { R :: struct { method :: () { f(); f :: () {} } } R.method(); }");
    rejected("main :: () { E :: enum { ONE :: later; } later :: 17; e:E; }");
    rejected("main :: () { E :: enum Later { ONE; } Later :: u8; e:E; }");
    let error = compile(
        "main :: () { R :: struct { items:[n]int; } n:=3; r:R; }",
        &[],
    )
    .unwrap_err();
    assert_eq!(error.code, Code::LocalNotAccessible, "{error:?}");
}
