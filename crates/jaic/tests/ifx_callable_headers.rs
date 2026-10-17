//! Runtime conditional callables retain lexical else headers, not TypeId-global metadata.
use jaic::sema::{Compiler, Options, ProgramSource, VirtualFs};
use std::rc::Rc;

fn compile(source: &str) -> Result<(), String> {
    let mut options = Options::host();
    options.runtime_support = false;
    let mut c = Compiler::new(options, Rc::new(VirtualFs::default()));
    c.compile_sources(&[ProgramSource::String(source.into())])
        .map_err(|error| error.render(&c.sources))
}

const PAIR: &str = "f :: (x:=3)->int {return 100+x;} g :: (x:=4)->int {return 200+x;}";

#[test]
fn runtime_dispatch_uses_else_default_for_both_flags() {
    compile(&format!("{PAIR} choose :: (flag:bool)->int {{h:=ifx flag then f else g; return h();}} #assert choose(true)==104; #assert choose(false)==204;")).unwrap();
}

#[test]
fn reversing_arms_changes_header_without_changing_dispatch() {
    compile(&format!("{PAIR} choose :: (flag:bool)->int {{h:=ifx flag then g else f; return h();}} #assert choose(true)==203; #assert choose(false)==103;")).unwrap();
}

#[test]
fn mutation_after_assignment_changes_neither_header_nor_callee() {
    compile(&format!("{PAIR} choose :: (flag:bool)->int {{h:=ifx flag then f else g; flag=!flag; return h();}} #assert choose(true)==104; #assert choose(false)==204;")).unwrap();
}

#[test]
fn named_arguments_and_explicit_values_work_after_join() {
    compile(&format!("{PAIR} choose :: (flag:bool)->int {{h:=ifx flag then f else g; return h(x=9);}} #assert choose(true)==109; #assert choose(false)==209;")).unwrap();
}

#[test]
fn else_parameter_name_is_retained_even_when_then_callee_is_selected() {
    compile("f::(left:=3)->int{return left;} g::(right:=4)->int{return right;} choose::(flag:bool)->int{h:=ifx flag then f else g;return h(right=9);} #assert choose(true)==9; #assert choose(false)==9;").unwrap();
}

#[test]
fn then_only_parameter_name_is_not_merged_into_else_header() {
    let error = compile("f::(left:=3)->int{return left;} g::(right:=4)->int{return right;} choose::(flag:bool)->int{h:=ifx flag then f else g;return h(left=9);}")
        .expect_err("official then-only named argument rejection");
    assert!(error.contains("no parameter named `left`"), "{error}");
}

#[test]
fn nested_else_headers_and_ordinary_aliases_remain_independent() {
    compile(&format!("{PAIR} third :: (x:=8)->int {{return 300+x;}} choose :: (a:bool,b:bool)->int {{direct:=f; h:=ifx a then f else (ifx b then g else third); return direct()+h();}} #assert choose(true,true)==103+108; #assert choose(false,true)==103+208; #assert choose(false,false)==103+308;")).unwrap();
}

#[test]
fn declared_callback_header_overrides_inferred_expression_header() {
    compile(&format!("{PAIR} Callback :: (value:=11)->int; choose :: (flag:bool)->int {{h:Callback=ifx flag then f else g; return h();}} #assert choose(true)==111; #assert choose(false)==211;")).unwrap();
}

#[test]
fn explicit_callback_does_not_accept_the_inferred_header_parameter_name() {
    let error = compile(&format!("{PAIR} Callback :: (value:=11)->int; choose :: (flag:bool)->int {{h:Callback=ifx flag then f else g; return h(x=7);}}"))
        .expect_err("the written callback header remains authoritative");
    assert!(error.contains("no parameter named `x`"), "{error}");
}

#[test]
fn else_default_keeps_its_private_loaded_file_scope() {
    let mut fs = VirtualFs::default();
    fs.insert(
        "/lib/Pair/module.jai",
        b"#load \"first.jai\"; #load \"last.jai\";".as_slice(),
    );
    fs.insert(
        "/lib/Pair/first.jai",
        b"#scope_file secret::3; #scope_export f::(x:=secret)->int{return 100+x;}".as_slice(),
    );
    fs.insert(
        "/lib/Pair/last.jai",
        b"#scope_file secret::19; #scope_export g::(x:=secret)->int{return 200+x;}".as_slice(),
    );
    fs.insert("/main.jai", b"P::#import \"Pair\"; choose::(flag:bool)->int{h:=ifx flag then P.f else P.g;return h();} #assert choose(true)==119; #assert choose(false)==219; #assert !#exists(P.secret); main::(){}".as_slice());
    let mut options = Options::host();
    options.runtime_support = false;
    options.import_paths = vec!["/lib".into()];
    let mut c = Compiler::new(options, Rc::new(fs));
    if let Err(error) = c.compile_program(std::path::Path::new("/main.jai")) {
        panic!("{}", error.render(&c.sources));
    }
}

#[test]
fn required_else_argument_does_not_inherit_then_default() {
    let error=compile("f :: (x:=3)->int {return x;} g :: (x:int)->int {return x;} choose :: (flag:bool)->int {h:=ifx flag then f else g; return h();}").expect_err("else header requires an argument");
    assert!(error.contains("missing argument 1"), "{error}");
}

#[test]
fn incompatible_branch_types_are_not_erased_by_header_annotation() {
    let error=compile("f :: (x:int)->int {return x;} g :: (x:string)->int {return x.count;} choose :: (flag:bool)->int {h:=ifx flag then f else g; return h(3);}").expect_err("branch signatures differ");
    assert!(
        error.contains("no overload matches procedure type"),
        "{error}"
    );
}
