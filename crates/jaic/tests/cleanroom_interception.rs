//! Canonical body interception for called procedures in system and user modules.
use jaic::{
    records::{FieldRef, Item, Records},
    sema::{Compiler, Options, VirtualFs},
};
use std::{
    collections::{HashMap, HashSet},
    path::Path,
    rc::Rc,
};

fn refs(records: &Records, id: i64, field: &str) -> Vec<i64> {
    match records.field(id, field) {
        Some(FieldRef::List(list)) => list
            .iter()
            .map(|item| match item {
                Item::Ref(id) => id,
                _ => panic!("expected reference"),
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn reference(records: &Records, id: i64, field: &str) -> i64 {
    match records.field(id, field) {
        Some(FieldRef::Ref(id)) => id,
        _ => panic!("missing {field}"),
    }
}

#[test]
fn retained_member_and_anonymous_bodies_have_canonical_ownership() {
    use jaic::sema::code_export::ModifiedStmt;

    let mut fs = VirtualFs::default();
    fs.insert("/lib/prelude/module.jai", "".as_bytes());
    fs.insert(
        "/app/main.jai",
        r#"
R :: struct { touch :: (x:int)->int {return x+1;} }
main :: () -> int {
    callback := (x:int)->int {return x+3;};
    return R.touch(callback(4));
}
"#
        .as_bytes(),
    );
    let mut options = Options::host();
    options.runtime_support = false;
    options.preload = Some("/lib/prelude/module.jai".into());
    let fs = Rc::new(fs);
    let mut compiler = Compiler::new(options.clone(), fs.clone());
    compiler
        .compile_program(Path::new("/app/main.jai"))
        .unwrap();
    let mut other = Compiler::new(options, fs);
    let mut records = Records::default();
    other.compile_program(Path::new("/app/main.jai")).unwrap();
    while other.export_typechecked(&mut records).is_some() {}
    let mut bodies = HashSet::new();
    let mut headers = HashSet::new();
    for _ in 0..10 {
        let Some(message) = compiler.export_typechecked(&mut records) else {
            break;
        };
        for typed in refs(&records, message, "procedure_bodies") {
            let body = reference(&records, typed, "expression");
            let header = reference(&records, body, "header");
            assert!(bodies.insert(body));
            assert!(headers.insert(header));
            assert_eq!(reference(&records, header, "body_or_null"), body);
            let block = reference(&records, body, "block");
            let statements: Vec<_> = refs(&records, block, "statements")
                .into_iter()
                .map(ModifiedStmt::Record)
                .collect();
            assert!(
                other.modify_procedure(&records, body, &statements).is_err(),
                "foreign workspace body must be rejected"
            );
            let copy = records.add(records.get(body).unwrap().clone());
            assert!(
                compiler
                    .modify_procedure(&records, copy, &statements)
                    .is_err(),
                "copying fields cannot transfer canonical ownership"
            );
            compiler
                .modify_procedure(&records, body, &statements)
                .expect("retained body belongs to the actual canonical procedure");
        }
    }
    assert_eq!(
        bodies.len(),
        3,
        "main, member and callback each reported once"
    );
    compiler.relower_modified().unwrap();
    assert!(compiler.modify_procedure(&records, 999999, &[]).is_err());
}

#[test]
fn called_library_bodies_have_canonical_headers_and_are_reported_once() {
    let mut fs = VirtualFs::default();
    fs.insert("/lib/prelude/module.jai", "".as_bytes());
    fs.insert(
        "/lib/Probe/module.jai",
        r#"
ordinary :: (x:int) -> int { return hidden(x); }
alias :: ordinary;
generic :: (x:$T) -> T { return x; }
baked :: ($bias:int, x:$T) -> T { return x+bias; }
#scope_file
hidden :: (x:int) -> int { return x+1; }
unused :: () { unavailable_unused_body(); }
"#
        .as_bytes(),
    );
    fs.insert("/app/main.jai", r#"#import "Probe"; main :: () -> int { return ordinary(1)+alias(2)+generic(3)+generic(4)+baked(2,4)+baked(2,5); }"#.as_bytes());
    let mut options = Options::host();
    options.runtime_support = false;
    options.preload = Some("/lib/prelude/module.jai".into());
    options.import_paths.push("/lib".into());
    let mut compiler = Compiler::new(options, Rc::new(fs));
    compiler
        .compile_program(Path::new("/app/main.jai"))
        .unwrap();
    let mut records = Records::default();
    let mut bodies: HashMap<String, Vec<i64>> = HashMap::new();
    let mut headers: HashMap<i64, usize> = HashMap::new();
    let mut calls = Vec::new();
    let mut body_headers = HashSet::new();
    for iteration in 0..20 {
        let Some(message) = compiler.export_typechecked(&mut records) else {
            break;
        };
        assert!(iteration < 19, "interception must reach quiescence");
        for tc in refs(&records, message, "procedure_headers") {
            *headers
                .entry(reference(&records, tc, "expression"))
                .or_default() += 1;
        }
        for tc in refs(&records, message, "procedure_bodies") {
            let body = reference(&records, tc, "expression");
            let header = reference(&records, body, "header");
            assert!(
                body_headers.insert(header),
                "duplicate body for canonical header {header}"
            );
            assert_eq!(reference(&records, header, "body_or_null"), body);
            // A generic source tree and its concrete instance are distinct records.
            if matches!(records.field(header, "procedure_flags"), Some(FieldRef::Int(flags)) if flags & 4 != 0)
            {
                continue;
            }
            let Some(FieldRef::Str(name)) = records.field(header, "name") else {
                panic!("header name");
            };
            bodies
                .entry(String::from_utf8_lossy(name).into_owned())
                .or_default()
                .push(header);
            for node in refs(&records, tc, "subexpressions") {
                if let Some(FieldRef::Ref(header)) =
                    records.field(node, "resolved_procedure_expression")
                {
                    calls.push(header);
                }
            }
        }
    }
    for name in ["main", "ordinary", "generic", "baked", "hidden"] {
        assert_eq!(
            bodies.get(name).map(Vec::len),
            Some(1),
            "{name}: {bodies:?}"
        );
        assert_eq!(headers.get(&bodies[name][0]), Some(&1));
    }
    assert!(!bodies.contains_key("unused"));
    let baked_args = refs(&records, bodies["baked"][0], "arguments");
    assert_eq!(baked_args.len(), 2);
    for arg in baked_args {
        assert_ne!(
            reference(&records, arg, "type"),
            0,
            "baked and runtime arguments retain their concrete types"
        );
    }
    assert!(
        calls
            .iter()
            .filter(|&&h| h == bodies["ordinary"][0])
            .count()
            >= 2
    );
    assert!(calls.iter().filter(|&&h| h == bodies["generic"][0]).count() >= 2);
    assert!(compiler.export_typechecked(&mut records).is_none());
    compiler
        .add_source(&jaic::sema::ProgramSource::String(
            "P :: #import \"Probe\"; #assert P.hidden(1)==2;".into(),
        ))
        .unwrap();
    let error = compiler
        .settle()
        .expect_err("interception must not change private member access");
    assert!(error.message.contains("hidden"), "{}", error.message);
}
