//! A struct with a variadic `..Type` parameter may be named by one of its own arguments through a
//! pointer; holding itself by value is still rejected.
use jaic::sema::{Compiler, NativeFs, Options, ProgramSource};
use std::{path::PathBuf, rc::Rc};

fn check(source: &str) -> Result<(), String> {
    let stdlib = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    let mut options = Options::host();
    options.import_paths = vec![stdlib.clone()];
    options.set_stdlib(&NativeFs, &stdlib);
    let mut compiler = Compiler::new(options, Rc::new(NativeFs));
    compiler
        .compile_sources(&[ProgramSource::String(source.into())])
        .map_err(|error| error.render(&compiler.src.sources))
}

/// The payload is as large as the largest case, which the `#insert` reads from type information.
const BOX: &str = r#"
#import "Basic";

Box :: struct ($types: ..Type) {
    #insert -> string {
        size := 0;

        for types {
            if type_info(it).runtime_size > size size = type_info(it).runtime_size;
        }

        return tprint("payload: [%] u8;", size);
    };
}
"#;

#[test]
fn a_case_may_point_back_at_the_struct() {
    let source = format!(
        "{BOX}\n\
         Tree :: Box(Leaf, Branch);\n\
         Leaf :: struct {{ name: string; }}\n\
         Branch :: struct {{ children: [..] *Tree; }}\n\
         #assert size_of(Tree) == size_of(Branch);\n\
         #assert size_of(Branch) == 40;\n\
         main :: () {{ tree: Tree; branch: Branch; array_add(*branch.children, *tree); }}"
    );
    check(&source).unwrap_or_else(|error| panic!("{source}\n{error}"));
}

#[test]
fn the_case_order_does_not_matter() {
    let source = format!(
        "{BOX}\n\
         Branch :: struct {{ children: [..] *Tree; }}\n\
         Leaf :: struct {{ name: string; }}\n\
         Tree :: Box(Branch, Leaf);\n\
         #assert size_of(Tree) == 40;\n\
         main :: () {{ tree: Tree; }}"
    );
    check(&source).unwrap_or_else(|error| panic!("{source}\n{error}"));
}

#[test]
fn a_variant_may_hold_pointers_to_itself() {
    let source = "#import \"Basic\";\n\
         #import \"Extensions/Variant\";\n\
         Tree :: Variant(Leaf, Branch);\n\
         Leaf :: struct { value: s64; }\n\
         Branch :: struct { children: [..] *Tree; }\n\
         #assert size_of(Tree) == 48;\n\
         main :: () {\n\
             root: Tree;\n\
             branch: Branch;\n\
             child := New(Tree);\n\
             set(child, Leaf.{ 7 });\n\
             array_add(*branch.children, child);\n\
             set(*root, branch);\n\
         }";
    check(source).unwrap_or_else(|error| panic!("{source}\n{error}"));
}

#[test]
fn a_case_holding_the_struct_by_value_is_a_cycle() {
    let source = format!(
        "{BOX}\n\
         Tree :: Box(Leaf, Branch);\n\
         Leaf :: struct {{ name: string; }}\n\
         Branch :: struct {{ inner: Tree; }}\n\
         main :: () {{ tree: Tree; }}"
    );
    let error = check(&source).unwrap_err();
    assert!(
        error.contains("contains itself") || error.contains("circular dependency"),
        "{error}"
    );
}

#[test]
fn a_variant_holding_itself_by_value_is_a_cycle() {
    let source = "#import \"Extensions/Variant\";\n\
         Tree :: Variant(Leaf, Branch);\n\
         Leaf :: struct { value: s64; }\n\
         Branch :: struct { inner: Tree; }\n\
         main :: () { tree: Tree; }";
    let error = check(source).unwrap_err();
    assert!(
        error.contains("contains itself") || error.contains("circular dependency"),
        "{error}"
    );
}
