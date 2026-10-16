//! Independent nominal inheritance controls; no Apollo/type-name dispatch exceptions.
use jaic::sema::{Compiler, Options, VirtualFs};
use std::{path::Path, rc::Rc};

fn compile(source: &str, private: bool) -> Result<(), String> {
    let mut fs = VirtualFs::default();
    let scope = if private {
        "#scope_file"
    } else {
        ""
    };
    let module = format!(
        r#"
        Base :: struct {{ n:int; }}
        Derived :: #type,isa Base;
        Nested :: #type,isa Derived;
        Distinct :: #type Base;
        Unrelated :: struct {{ n:int; }}
        {scope}
        operator < :: (a:Base,b:Base)->bool {{ return a.n<b.n; }}
        operator <= :: (a:Base,b:Base)->bool {{ return a.n<=b.n; }}
        operator > :: (a:Base,b:Base)->bool {{ return a.n>b.n; }}
        operator >= :: (a:Base,b:Base)->bool {{ return a.n>=b.n; }}
        operator == :: (a:Base,b:Base)->bool {{ return a.n%10==b.n%10; }}
        operator + :: (a:Base,b:Base)->Base {{ return .{{a.n+b.n}}; }}
        operator - :: (a:Base,b:Base)->Base {{ return .{{a.n-b.n}}; }}
        operator ! :: (a:Base)->bool {{ return a.n==0; }}
    "#
    );
    fs.insert("/lib/Definition/module.jai", module.as_bytes());
    fs.insert("/main.jai", source.as_bytes());
    let mut options = Options::host();
    options.runtime_support = false;
    options.import_paths = vec!["/lib".into()];
    let mut compiler = Compiler::new(options, Rc::new(fs));
    compiler
        .compile_program(Path::new("/main.jai"))
        .map_err(|e| e.render(&compiler.sources))
}

#[test]
fn visible_base_comparisons_dispatch_for_nominal_and_nested_isa_types() {
    for name in ["Base", "Derived", "Nested"] {
        let source = format!(
            r#"M :: #import "Definition";
            using,only(.["<","<=",">",">=","==","+","-"]) M;
            check :: ()->bool {{
                a:=M.{name}.{{12}}; b:=M.{name}.{{2}};
                if !(b<a && b<=a && a>b && a>=b && a>=a && a<=a) return false;
                if !(a==b) || a!=b return false;
                return (a+b).n==14 && (a-b).n==10;
            }}
            #assert check(); main :: () {{}}"#
        );
        compile(&source, false).unwrap();
    }
}

#[test]
fn unrelated_record_does_not_acquire_base_comparisons() {
    compile(
        r#"using #import "Definition";
        main :: () { a:=Unrelated.{1}; b:=Unrelated.{2}; rejected:=a<b; }"#,
        false,
    )
    .expect_err("structural field similarity does not import an unrelated overload");
}

#[test]
fn reviewed_logical_not_uses_a_public_symbol_filter() {
    compile(
        r#"M :: #import "Definition"; using,only(.["!"]) M;
        #assert !M.Base.{0}; #assert !!M.Base.{7}; main :: () {}"#,
        false,
    )
    .unwrap();
}

#[test]
fn visible_base_logical_not_dispatches_for_nested_nominal_records() {
    for name in ["Derived", "Nested"] {
        let source = format!(
            r#"M :: #import "Definition"; using,only(.["!"]) M;
            check :: ()->bool {{ zero:=M.{name}.{{0}}; nonzero:=M.{name}.{{7}};
                return !zero && !!nonzero; }} #assert check(); main :: () {{}}"#
        );
        compile(&source, false).unwrap();
    }
}

#[test]
fn nominal_unary_lookup_preserves_privacy_and_unrelated_type_rejection() {
    for name in ["Base", "Derived", "Nested"] {
        let source = format!(
            r#"M :: #import "Definition"; using,only(.["!"]) M;
            main :: () {{ value:=M.{name}.{{0}}; rejected:=!value; }}"#
        );
        compile(&source, true)
            .expect_err("unary overload privacy is not bypassed by representation lookup");
        let source = format!(
            r#"M :: #import "Definition";
            main :: () {{ value:=M.{name}.{{0}}; rejected:=!value; }}"#
        );
        compile(&source, false).expect_err("a qualified type does not import unary overloads");
    }
    compile(
        r#"using #import "Definition";
        main :: () { value:=Unrelated.{0}; rejected:=!value; }"#,
        false,
    )
    .expect_err("similar record fields do not enable an unrelated unary overload");
}

#[test]
fn private_and_unimported_base_comparisons_stay_private() {
    let source = r#"M :: #import "Definition"; using,only(.["<"]) M;
        main :: () { a:=M.Derived.{1}; b:=M.Derived.{2}; rejected:=a<b; }"#;
    compile(source, true).expect_err("filters cannot expose private operators");
    compile(
        r#"M :: #import "Definition";
        main :: () { a:=M.Derived.{1}; b:=M.Derived.{2}; rejected:=a<b; }"#,
        false,
    )
    .expect_err("type visibility does not import its operators");
}

#[test]
fn only_and_except_do_not_leak_unselected_comparison_operators() {
    for filter in [r#"only(.["=="])"#, r#"except(.["<"])"#] {
        let source = format!(
            r#"M :: #import "Definition"; using,{filter} M;
            main :: () {{ a:=M.Derived.{{1}}; b:=M.Derived.{{2}}; rejected:=a<b; }}"#
        );
        compile(&source, false).expect_err("comparison selection follows ordinary scope rules");
    }
}

#[test]
fn unary_symbol_filter_excludes_hidden_and_internal_spellings() {
    for filter in [
        r#"except(.["!"])"#,
        r#"only(.["=="])"#,
        r#"only(.["operator!"])"#,
    ] {
        let source = format!(
            r#"M :: #import "Definition"; using,{filter} M;
            main :: () {{ value:=M.Nested.{{0}}; rejected:=!value; }}"#
        );
        compile(&source, false)
            .expect_err("public unary filters preserve selection and hide internal spellings");
    }
}
