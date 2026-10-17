//! Enum dependency, ordering and invalid-program regressions.
use jaic::sema::{Compiler, Options, ProgramSource, VirtualFs};
use std::rc::Rc;

#[path = "support/reflection_bootstrap.rs"]
mod bootstrap;

fn compile(source: &str) -> (Compiler, jaic::sema::Result<()>) {
    let mut options = Options::host();
    options.runtime_support = false;
    options.preload = Some("/lib/prelude/module.jai".into());
    let mut fs = VirtualFs::default();
    fs.insert(
        "/lib/prelude/module.jai",
        format!("{} {}", bootstrap::BOOTSTRAP, ENUM_RECORD).as_bytes(),
    );
    let mut c = Compiler::new(options, Rc::new(fs));
    let result = c.compile_sources(&[ProgramSource::String(source.into())]);
    (c, result)
}

fn check(source: &str) {
    let (c, r) = compile(source);
    if let Err(e) = r {
        panic!("{}", e.render(&c.sources));
    }
}

const ENUM_RECORD: &str = "Type_Info_Enum :: struct {using #as info:Type_Info; name:string; internal_type:*Type_Info_Integer; names:[]string; values:[]s64; enum_type_flags:u32;}";

#[test]
fn enum_values_execute_in_the_interpreter() {
    check(&format!(
        "{} #assert enum_dependency_check()==0;",
        include_str!("../../../tests/compiler/semantics/enum-dependencies.jai")
    ));
}

#[test]
fn ide_forward_member_navigation_preserves_declaration_locations() {
    let source = "E :: enum {MASK::FIRST|E.LAST;FIRST::1;LAST::4;} main :: () { value:E=.MASK; }";
    let mut options = Options::host();
    options.runtime_support = false;
    let mut fs = VirtualFs::default();
    fs.insert("/app/main.jai", source.as_bytes());
    let mut c = Compiler::new(options, Rc::new(fs));
    c.ide = Some(Box::new(jaic::sema::ide::IdeFacts::new(vec![
        "/app/".into(),
    ])));
    if let Err(e) = c.compile_program(std::path::Path::new("/app/main.jai")) {
        panic!("{}", e.render(&c.sources));
    }
    assert!(c.ide_check_all().is_empty());
    for (usage, declaration) in [
        ("FIRST|", "FIRST::"),
        ("LAST;", "LAST::"),
        ("MASK;", "MASK::"),
    ] {
        let spans = c.ide_definition(jaic::source::FileId(0), source.find(usage).unwrap() as u32);
        assert_eq!(spans.len(), 1, "{usage}");
        assert_eq!(spans[0].start, source.find(declaration).unwrap() as u32);
    }
}

#[test]
fn forward_values_qualified_members_and_implicit_numbering_keep_source_order() {
    check(
        r#"
        E :: enum u32 {FIRST; MASK::LAST|16; LAST::4; AFTER;}
        Q :: enum {A::Q.C+1;B;C::8;D;}
        F :: enum_flags u32 {FIRST;MASK::FIRST|LAST;LAST::4;NEXT;ZERO::0;RESTART;}
        #assert E.FIRST==xx 0 && E.MASK==xx 20 && E.AFTER==xx 5;
        #assert Q.A==xx 9 && Q.B==xx 10 && Q.D==xx 9;
        #assert F.MASK==xx 5 && F.NEXT==xx 8 && F.RESTART==xx 1;
        main :: () {}
    "#,
    );
}

#[test]
fn selected_conditionals_and_inserts_participate_in_the_same_dependency_graph() {
    check(
        r#"
        E :: enum {A::B+C; #if true {B::7;} else {B::unknown_inactive;} #insert "C :: 9; D;"; LAST;}
        F :: enum_flags {MASK::A|B; #if true {A::1;} #insert "B :: 4; NEXT;"; AFTER;}
        #assert E.A==xx 16 && E.D==xx 10 && E.LAST==xx 11;
        #assert F.MASK==xx 5 && F.NEXT==xx 8 && F.AFTER==xx 16;
        G :: enum {FIRST::1; #if FIRST==xx 1 {SECOND;} #insert "THIRD;"; LAST;}
        #assert G.LAST==xx 4;
        H :: enum { #if LAST==xx 4 {FIRST::NEXT;NEXT::7;} #insert ifx LAST==xx 4 then "INJECTED :: FIRST;" else ""; LAST::4; }
        #assert H.FIRST==xx 7 && H.INJECTED==xx 7;
        Empty :: enum {FIRST; #if LAST==xx 1 {#if false {}} LAST;}
        #assert Empty.LAST==xx 1;
        main :: () {}
    "#,
    );
}

#[test]
fn reflection_retains_declared_names_values_flags_and_active_insertion_order() {
    check(
        r#"
        E :: enum_flags u32 #specified {FIRST::1; MASK::FIRST|LAST; #if true {LAST::4;} #insert "NEXT; ALIAS :: FIRST;"; AFTER;}
        info :: type_info(E);
        #assert info.names.count==6 && info.values.count==6;
        #assert info.names[0]=="FIRST" && info.values[0]==1;
        #assert info.names[1]=="MASK" && info.values[1]==5;
        #assert info.names[2]=="LAST" && info.values[2]==4;
        #assert info.names[3]=="NEXT" && info.values[3]==8;
        #assert info.names[4]=="ALIAS" && info.values[4]==1;
        #assert info.names[5]=="AFTER" && info.values[5]==2;
        #assert info.enum_type_flags==5;
        main :: () {}
    "#,
    );
}

#[test]
fn invalid_enums_retain_duplicate_cycle_unknown_and_range_diagnostics() {
    for (source, expected) in [
        ("E :: enum {A::B;B::A;}", "circular"),
        ("E :: enum {A::B;B;}", "circular"),
        ("E :: enum {A::E.B;B::E.A;}", "circular"),
        ("E :: enum {A::E.E;}", "no member"),
        (
            "E :: enum {#if LAST==xx 1 {FIRST;} LAST;}",
            "declaration-order dependency",
        ),
        ("E :: enum {A::7;A::8;}", "declared more than once"),
        (
            "E :: enum {A::7;#if true {A::8;}}",
            "declared more than once",
        ),
        (
            r#"E :: enum {A::7;#insert "A :: 8;";}"#,
            "declared more than once",
        ),
        ("E :: enum u8 {A::LAST;LAST::256;}", "out of range"),
        ("E :: enum u8 {A::255;B;}", "out of range"),
        ("E :: enum_flags u8 {A::128;B;}", "out of range"),
        ("E :: enum {A::missing;}", "unknown identifier"),
    ] {
        let (c, r) = compile(&format!("{source} main :: () {{}}"));
        let e = r.expect_err("invalid enum must fail");
        assert!(e.message.contains(expected), "{}", e.render(&c.sources));
    }
}
