//! Independent typed generic-struct argument inference, without application code.
use jaic::sema::{Compiler, Options, ProgramSource, VirtualFs};
use std::rc::Rc;

fn compile(source: &str) -> Result<(), String> {
    let mut options = Options::host();
    options.runtime_support = false;
    options.import_paths = vec!["/lib".into()];
    let mut fs = VirtualFs::default();
    fs.insert(
        "/lib/Hidden/module.jai",
        r#"
        Public :: struct(T: Type, count: u64) { value: T; }
        #scope_file
        Private :: struct(T: Type, count: u64) { value: T; }
    "#
        .as_bytes(),
    );
    let mut c = Compiler::new(options, Rc::new(fs));
    c.compile_sources(&[ProgramSource::String(source.into())])
        .map_err(|e| e.render(&c.sources))
}

#[test]
fn unsigned_struct_parameter_retains_type_during_procedure_inference() {
    compile(
        r#"
        Parcel :: struct(T: Type, capacity: u64) { value: T; }
        write :: (p: *Parcel($T, $count), value: T) { p.value = value; }
        check :: () -> bool { p: Parcel(int, 20); write(*p, 1); return p.value == 1; }
        #assert check(); main :: () {}
    "#,
    )
    .unwrap();
}

#[test]
fn declaration_order_and_inference_names_do_not_change_bindings() {
    for (parameters, arguments, pattern) in [
        ("T: Type, capacity: u64", "int, 20", "$Element, $Count"),
        ("capacity: u64, T: Type", "20, int", "$Count, $Element"),
    ] {
        compile(&format!(
            r#"
            Parcel :: struct({parameters}) {{ value: T; }}
            write :: (p: *Parcel({pattern}), value: Element) {{ p.value = value; }}
            check :: () -> bool {{ p: Parcel({arguments}); write(*p, 7); return p.value == 7; }}
            #assert check(); main :: () {{}}
        "#
        ))
        .unwrap();
    }
}

#[test]
fn identical_values_with_different_declared_types_stay_distinct() {
    compile(
        r#"
        Parcel :: struct(T: Type, unsigned: u64, signed: s32) { value: T; }
        unsigned_only :: (v: u64) -> bool { return v == 20; }
        signed_only :: (v: s32) -> bool { return v == 20; }
        write :: (p: *Parcel($T, $U, $S), value: T) -> bool {
            p.value = value; return unsigned_only(U) && signed_only(S);
        }
        check :: () -> bool { p: Parcel(int, 20, 20); return write(*p, 7) && p.value == 7; }
        #assert check(); main :: () {}
    "#,
    )
    .unwrap();
}

#[test]
fn inferred_struct_parameter_types_are_not_positional_arguments() {
    compile(
        r#"
        Parcel :: struct(seed: $Seed, capacity: u64) { value: Seed; }
        write :: (p: *Parcel($Value, $Count)) -> bool { return Value == 3 && Count == 20; }
        check :: () -> bool { p: Parcel(3, 20); return write(*p); }
        #assert check(); main :: () {}
    "#,
    )
    .unwrap();
}

#[test]
fn baked_struct_alias_uses_its_remaining_parameter_declaration() {
    compile(
        r#"
        Parcel :: struct(T: Type, capacity: u64) { value: T; }
        IntParcel :: #bake_arguments Parcel(T=int);
        write :: (p: *IntParcel($Count), value: int) { p.value = value; }
        check :: () -> bool { p: IntParcel(20); write(*p, 7); return p.value == 7; }
        #assert check(); main :: () {}
    "#,
    )
    .unwrap();
}

#[test]
fn repeated_instances_keep_value_and_type_bindings_isolated() {
    compile(
        r#"
        Parcel :: struct(T: Type, capacity: u64) { value: T; }
        write :: (p: *Parcel($T, $Count), value: T) -> u64 { p.value = value; return Count; }
        check :: () -> bool {
            a: Parcel(int, 20); b: Parcel(u64, 30); c: Parcel(int, 30);
            return write(*a, 1) == 20 && write(*b, cast(u64) 2) == 30
                && write(*c, 3) == 30 && a.value == 1 && b.value == 2 && c.value == 3;
        }
        #assert check(); main :: () {}
    "#,
    )
    .unwrap();
}

#[test]
fn typed_capacity_does_not_allow_wrong_payload_or_unrelated_family() {
    for call in ["write(*p, value);", "write(*other, 1);", "fixed(*p);"] {
        compile(&format!(
            r#"
            Parcel :: struct(T: Type, capacity: u64) {{ value: T; }}
            Other :: struct(T: Type, capacity: u64) {{ value: T; }}
            write :: (p: *Parcel($T, $Count), value: T) {{ p.value = value; }}
            fixed :: (p: *Parcel($T, 21)) {{}}
            main :: () {{ p: Parcel(int, 20); other: Other(int, 20); value: u64 = 1; {call} }}
        "#
        ))
        .expect_err(
            "type inference must not introduce payload conversions or merge struct families",
        );
    }
}

#[test]
fn qualified_baked_alias_matches_its_own_parameter_declaration() {
    compile(
        r#"
        Domain :: struct {
            Parcel :: struct(T: Type, capacity: u64) { value: T; }
            IntParcel :: #bake_arguments Parcel(T=int);
        }
        write :: (p: *Domain.IntParcel($Count), value: int) -> u64 { p.value=value; return Count; }
        check :: () -> bool { p: Domain.IntParcel(20); return write(*p, 7)==20 && p.value==7; }
        #assert check(); main :: () {}
    "#,
    )
    .unwrap();
}


#[test]
fn qualified_constructor_lookup_preserves_family_identity_and_privacy() {
    compile(
        r#"
        Domain :: struct {
            Parcel :: struct(T: Type, capacity: u64) { value: T; }
            Other :: struct(T: Type, capacity: u64) { value: T; }
        }
        write :: (p: *Domain.Parcel($T, $Count), value: T) { p.value=value; }
        main :: () { p: Domain.Other(int, 20); write(*p, 7); }
    "#,
    )
    .expect_err("qualified constructor identity cannot match an unrelated family");
    let error = compile(
        r#"
        M :: #import "Hidden";
        write :: (p: *M.Private($T, $Count), value: T) {}
        main :: () { p: M.Public(int, 20); write(*p, 7); }
    "#,
    )
    .expect_err("qualified pattern evaluation cannot expose private declarations");
    assert!(error.contains("Private"), "{error}");
}


#[test]
fn existing_compiler_named_pattern_extension_is_not_a_reference_contract() {
    compile(
        r#"
        Parcel :: struct(T: Type, capacity: u64) { value: T; }
        write :: (p: *Parcel(capacity=$Count, T=$Element), value: Element) { p.value=value; }
        check :: () -> bool { p: Parcel(capacity=20, T=int); write(*p, 7); return p.value==7; }
        #assert check(); main :: () {}
    "#,
    )
    .unwrap();
}
