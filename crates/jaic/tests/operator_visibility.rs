//! A canonical type alias does not import its defining scope's operators.
use jaic::sema::{Compiler, Options, VirtualFs};
use std::{
    path::{Path, PathBuf},
    rc::Rc,
};

fn compile(facade: &str, body: &str) -> Result<(), String> {
    let mut fs = VirtualFs::default();
    fs.insert("/modules/Definition/module.jai", "Value :: struct {n:int;} operator - :: (a:Value,b:Value)->Value {return .{a.n-b.n};} operator ! :: (a:Value)->bool {return a.n==0;}".as_bytes());
    fs.insert("/modules/Facade/module.jai", facade.as_bytes());
    fs.insert("/main.jai", format!("using #import \"Facade\"; check :: () {{a:Value=.{{17}}; b:Value=.{{4}}; {body}}} main :: () {{}}").as_bytes());
    let mut options = Options::host();
    options.runtime_support = false;
    options.import_paths = vec![PathBuf::from("/modules")];
    let mut compiler = Compiler::new(options, Rc::new(fs));
    compiler
        .compile_program(Path::new("/main.jai"))
        .map_err(|error| error.render(&compiler.sources))
}

#[test]
fn canonical_type_only_facade_does_not_leak_symbolic_or_named_operators() {
    let facade = "Value :: Impl.Value; #scope_file Impl :: #import \"Definition\";";
    for body in ["v := a-b;", "v := !a;", "v := operator -(a,b);"] {
        compile(facade, body).expect_err("a type alias cannot reexport hidden overloads");
    }
}

#[test]
fn visible_direct_imports_and_filtered_reexports_keep_both_call_forms() {
    for facade in [
        "using #import \"Definition\";",
        "using,except(absent) #import \"Definition\";",
    ] {
        compile(facade, "v:=a-b; w:=operator -(a,b); zero:=!a;").unwrap();
    }
}
