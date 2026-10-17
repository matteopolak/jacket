//! Reductions of default generic pointee identity and constraint binding.
use jaic::sema::{Compiler, Options, ProgramSource, VirtualFs};
use std::rc::Rc;

#[path = "support/reflection_bootstrap.rs"]
mod bootstrap;

fn compile(source: &str) -> (Compiler, jaic::sema::Result<()>) {
    let mut options = Options::host();
    options.runtime_support = false;
    options.preload = Some("/lib/prelude/module.jai".into());
    let mut fs = VirtualFs::default();
    fs.insert("/lib/prelude/module.jai", bootstrap::BOOTSTRAP.as_bytes());
    let mut c = Compiler::new(options, Rc::new(fs));
    let result = c.compile_sources(&[ProgramSource::String(source.into())]);
    (c, result)
}

#[test]
fn implicit_default_pointees_bind_the_same_constraint_as_explicit_instances() {
    for pointee in ["Parcel", "Parcel(void)"] {
        let (c, result) = compile(&format!(
            r#"
            Parcel :: struct(T: Type = void) {{ data: *T; }}
            accept :: (pointer: *$P/Parcel) -> bool {{ return pointer == null; }}
            check :: () -> bool {{ pointer: *{pointee}; return accept(pointer); }}
            #assert check(); main :: () {{}}
        "#
        ));
        if let Err(e) = result {
            panic!("{}", e.render(&c.sources));
        }
    }
}

#[test]
fn default_pointer_types_preserve_canonical_type_and_alias_identity() {
    let (c, result) = compile(
        r#"
        Parcel :: struct(T: Type = void) { data: *T; }
        Alias :: Parcel;
        #assert *Parcel == *Parcel(void);
        #assert **Alias == **Parcel(void);
        accepts :: (p: *Parcel(void)) -> bool { return p == null; }
        check :: () -> bool { p: *Alias; return accepts(p); }
        #assert check(); main :: () {}
    "#,
    );
    if let Err(e) = result {
        panic!("{}", e.render(&c.sources));
    }
}

#[test]
fn invalid_type_arguments_are_rejected_even_when_the_parameter_is_unused() {
    for declaration in [
        "Parcel :: struct(T: Type = void) {} value:Parcel(7);",
        "Parcel :: struct(T: Type = 7) {} value:*Parcel;",
    ] {
        let (c, result) = compile(&format!("{declaration} main :: () {{}}"));
        let e = result.expect_err("generic type binding must reject an integer");
        assert!(
            e.message.contains("expected a type"),
            "{}",
            e.render(&c.sources)
        );
    }
}

#[test]
fn pointer_constraints_retain_nominal_and_pointer_diagnostics() {
    for (argument, expected) in [
        ("pointer", "not an instance"),
        ("value", "expected a pointer"),
    ] {
        let (c, result) = compile(&format!(
            r#"
            Parcel :: struct(T: Type = void) {{ data:*T; }}
            Other :: struct {{ data:*void; }}
            accept :: (pointer: *$P/Parcel) {{}}
            main :: () {{ value:Other; pointer:=*value; accept({argument}); }}
        "#
        ));
        let e = result.expect_err("constraint mismatch must still reject");
        assert!(e.message.contains(expected), "{}", e.render(&c.sources));
    }
}

#[test]
fn explicit_specializations_do_not_accept_different_type_arguments() {
    let (c, result) = compile(
        r#"
        Parcel :: struct(T: Type = void) { data:*T; }
        accept :: (pointer:*Parcel(void)) {}
        main :: () { value:Parcel(u32); accept(*value); }
    "#,
    );
    let e = result.expect_err("explicit specialization must not become a wildcard constraint");
    assert!(e.message.contains("Parcel"), "{}", e.render(&c.sources));
}

#[test]
fn defaults_dependent_arguments_and_nondefault_instances_execute_through_constraints() {
    let (c, result) = compile(&format!(
        "{} #assert check()==0;",
        include_str!("../../../tests/compiler/semantics/default-generic-pointers.jai")
    ));
    if let Err(e) = result {
        panic!("{}", e.render(&c.sources));
    }
}

#[test]
fn reflection_keeps_default_instance_identity_and_separate_generic_source() {
    let (c, result) = compile(
        r#"
        Parcel :: struct(T: Type = void) { data:*T; }
        Alias :: Parcel;
        #assert type_info(*Alias)==type_info(*Parcel(void));
        #assert type_info(**Alias)==type_info(**Parcel(void));
        #assert type_info(*Alias).pointer_to==type_info(Parcel(void));
        #assert type_info(Parcel(void)).polymorph_source_struct==type_info(Parcel(u32)).polymorph_source_struct;
        #assert type_info(Parcel(void))!=type_info(Parcel(void)).polymorph_source_struct;
        main :: () {}
    "#,
    );
    if let Err(e) = result {
        panic!("{}", e.render(&c.sources));
    }
}
