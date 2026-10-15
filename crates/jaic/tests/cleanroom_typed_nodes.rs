//! Checked expressions must retain the actual procedure-instance result type.
use jaic::{
    records::{FieldRef, Item, Records},
    sema::{Compiler, Options, ProgramSource, VirtualFs},
};
use std::rc::Rc;

fn binary_result_types(source: &str) -> (Vec<String>, usize) {
    let mut options = Options::host();
    options.runtime_support = false;
    let mut compiler = Compiler::new(options, Rc::new(VirtualFs::default()));
    compiler
        .compile_sources(&[ProgramSource::String(source.into())])
        .unwrap();
    let mut records = Records::default();
    let mut messages = Vec::new();
    while let Some(message) = compiler.export_typechecked(&mut records) {
        messages.push(message);
    }
    let mut results = Vec::new();
    let mut templates = 0;
    for message in messages {
        let Some(FieldRef::List(bodies)) = records.field(message, "procedure_bodies") else {
            panic!("missing bodies")
        };
        for i in 0..bodies.len() {
            let Some(Item::Ref(body)) = bodies.get(i) else {
                panic!("missing body")
            };
            let Some(FieldRef::Ref(root)) = records.field(body, "expression") else {
                panic!("missing body root")
            };
            let Some(FieldRef::Ref(header)) = records.field(root, "header") else {
                panic!("missing body header")
            };
            if matches!(records.field(header,"procedure_flags"),Some(FieldRef::Int(flags)) if flags & 4 != 0)
            {
                // A source template has no unique instantiated type. Count it explicitly;
                // every concrete body below must carry all checked binary result types.
                templates += 1;
                continue;
            }
            let Some(FieldRef::List(nodes)) = records.field(body, "subexpressions") else {
                panic!("missing nodes")
            };
            for j in 0..nodes.len() {
                let Some(Item::Ref(node)) = nodes.get(j) else {
                    panic!("missing node")
                };
                if records.get(node).unwrap().tag != "Code_Binary_Operator" {
                    continue;
                }
                let Some(FieldRef::Ref(ty)) = records.field(node, "type") else {
                    panic!(
                        "checked binary has no result descriptor: {:?}",
                        records.get(node)
                    )
                };
                let Some(FieldRef::Str(name)) = records.field(ty, "__builtin") else {
                    panic!("missing builtin name")
                };
                results.push(String::from_utf8_lossy(name).into_owned());
            }
        }
    }
    results.sort();
    (results, templates)
}

#[test]
fn checked_local_binary_results_have_canonical_types() {
    let source = r#"
a :: 1000.0;
#program_export "sum"
sum :: (a:u8,b:u8)->u8 #c_call { return a+b; }
#program_export "compare"
compare :: (a:s32,b:s32)->bool #c_call { return a<b; }
#program_export "index"
index :: (a:*u16,i:s64)->u16 #c_call { return a[i]; }
main :: () {}
"#;
    assert_eq!(binary_result_types(source).0, ["bool", "u16", "u8"]);
}

#[test]
fn generic_instances_share_syntax_but_keep_distinct_result_types() {
    let result = binary_result_types(
        r#"
twice :: (x:$T) -> T #no_context { return x+x; }
#program_export "entry"
main :: () { a:u8=3; b:u16=4; twice(a); twice(b); }
"#,
    );
    assert_eq!(result.0, ["u16", "u8"]);
    assert_eq!(
        result.1, 1,
        "the uninstantiated source template remains syntax"
    );
}

#[test]
fn lexical_shadowing_and_unary_cast_results_are_checked_in_the_body() {
    let result = binary_result_types(
        r#"
x :: 300.0;
#program_export "nested"
nested :: (input:s32) -> s32 #c_call {
    x:s32=input;
    { x:float64=cast(float64)input; y:=x+x; }
    return -x + cast(s32)x;
}
main :: () {}
"#,
    );
    assert_eq!(result.0, ["float64", "s32"]);
}

#[test]
fn invalid_local_expression_still_reports_original_type_error() {
    let mut options = Options::host();
    options.runtime_support = false;
    let mut compiler = Compiler::new(options, Rc::new(VirtualFs::default()));
    let error = compiler
        .compile_sources(&[ProgramSource::String(
            r#"main :: () { a:u8=2; b:string="x"; c:=a+b; }"#.into(),
        )])
        .expect_err("metadata capture cannot bless invalid expressions");
    assert!(
        error.message.contains("type") || error.message.contains("operator"),
        "{}",
        error.render(&compiler.sources)
    );
}
