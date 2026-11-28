//! A canonical type alias does not import its defining scope's operators.

use test_support::compiler::compile_virtual_rendered;

fn compile(facade: &str, body: &str) -> Result<(), String> {
    let main = format!(
        "using #import \"Facade\"; check :: () {{a:Value=.{{17}}; b:Value=.{{4}}; {body}}} main :: () {{}}"
    );

    compile_virtual_rendered(
        &[
            (
                "/modules/Definition/module.jai",
                "Value :: struct {n:int;} operator - :: (a:Value,b:Value)->Value {return .{a.n-b.n};} operator ! :: (a:Value)->bool {return a.n==0;}",
            ),
            ("/modules/Facade/module.jai", facade),
            ("/main.jai", &main),
        ],
        "/main.jai",
        |options| options.import_paths = vec!["/modules".into()],
    )
}

#[test]
fn canonical_type_only_facade_does_not_leak_symbolic_or_named_operators() {
    let facade = "Value :: Impl.Value; #scope_file Impl :: #import \"Definition\";";

    // Named, the overload is not in scope. As an operator it is found in the type's own module
    // (with a warning), so only the named form fails.
    compile(facade, "v := operator -(a,b);")
        .expect_err("a type alias cannot reexport hidden overloads");

    for body in ["v := a-b;", "v := !a;"] {
        compile(facade, body).unwrap();
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
