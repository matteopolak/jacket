//! Binding-time interface validation independent of particular library modules.
use jaic::sema::{Compiler, Options, VirtualFs};
use std::{path::Path, rc::Rc};

const MODULE: &str = r#"
#module_parameters(HOOKS: $I/interface Required = Required) {
    Required :: struct { run :: inline (value: int) {} }
}
touch :: () -> int { return 7; }
"#;

fn compile(body: &str) -> (Compiler, jaic::sema::Result<()>) {
    let mut fs = VirtualFs::default();
    fs.insert("/lib/Contract/module.jai", MODULE.as_bytes());
    fs.insert("/app/main.jai", body.as_bytes());
    let mut options = Options::host();
    options.runtime_support = false;
    options.import_paths = vec!["/lib".into()];
    let mut c = Compiler::new(options, Rc::new(fs));
    let result = c.compile_program(Path::new("/app/main.jai"));
    (c, result)
}

#[test]
fn unused_interface_replacement_is_validated_when_module_binds() {
    for source in [
        "Broken :: struct {}",
        "Broken :: struct { run :: (value: bool) {} }",
    ] {
        let (c, result) = compile(&format!(
            "{source} M :: #import \"Contract\"(HOOKS=Broken); main :: () {{ result:=M.touch(); }}"
        ));
        let error = result.expect_err("an unused invalid interface must be rejected");
        assert!(
            error.message.contains("interface") && error.message.contains("run"),
            "{}",
            error.render(&c.sources)
        );
    }
}

#[test]
fn valid_replacements_and_the_declared_sentinel_bind() {
    for source in [
        "M :: #import \"Contract\";",
        "Good :: struct { run :: inline (value: int) {} run :: (value: bool) {} } M :: #import \"Contract\"(HOOKS=Good);",
    ] {
        let (c, result) = compile(&format!("{source} #assert(M.touch()==7); main :: () {{}}"));
        if let Err(e) = result {
            panic!("{}", e.render(&c.sources));
        }
    }
}

#[test]
fn interfaces_apply_to_procedure_and_struct_type_arguments() {
    for replacement in ["Broken", "Good"] {
        for call in [
            "accept(value)",
            "accept_type(REPLACEMENT)",
            "Holder(REPLACEMENT)",
        ] {
            let source = format!(
                r#"
                Required :: struct {{ run :: inline (value:int) {{}} }}
                Good :: struct {{ run :: inline (value:int) {{}} }}
                Broken :: struct {{}}
                accept :: (value: $T/interface Required) {{}}
                accept_type :: ($value: $T/interface Required) {{}}
                Holder :: struct ($value: $T/interface Required) {{}}
                main :: () {{ value: {replacement}; {call}; }}
            "#,
                call = call.replace("REPLACEMENT", replacement)
            );
            let (c, result) = compile(&source);
            if replacement == "Broken" {
                let error = result.expect_err("all generic binding paths must enforce interfaces");
                assert!(
                    error.message.contains("interface") && error.message.contains("run"),
                    "{}",
                    error.render(&c.sources)
                );
            } else if let Err(e) = result {
                panic!("{}", e.render(&c.sources));
            }
        }
    }
}

#[test]
fn later_variadic_arguments_cannot_bypass_interface_validation() {
    let (c, result) = compile(
        r#"
        Required :: struct { run :: inline (value:int) {} }
        Good :: struct { run :: inline (value:int) {} }
        Broken :: struct {}
        accept :: (values: ..$T/interface Required) {}
        main :: () { accept(Good,Broken); }
    "#,
    );
    let error = result.expect_err("later elements must satisfy the contract too");
    assert!(
        error.message.contains("interface") && error.message.contains("run"),
        "{}",
        error.render(&c.sources)
    );
}

#[test]
fn unknown_type_values_do_not_silently_bypass_binding_time_validation() {
    let (c, result) = compile(
        r#"
        Required :: struct { run :: inline (value:int) {} }
        Good :: struct { run :: inline (value:int) {} }
        accept :: (values: ..$T/interface Required) {}
        #program_export "interface_runtime_type"
        check :: (unknown:Type) { accept(Good,unknown); }
        main :: () {}
    "#,
    );
    let error = result.expect_err("runtime type values cannot prove a binding-time interface");
    assert_eq!(
        error.kind,
        jaic::source::DiagnosticKind::InterfaceConstraint,
        "{}",
        error.render(&c.sources)
    );
    assert!(error.message.contains("concrete type binding"));
}
