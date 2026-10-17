//! Module configuration controls.
use jaic::sema::{Compiler, Options, VirtualFs};
use std::{path::Path, rc::Rc};

fn compile(module: &str, main: &str) -> (Compiler, jaic::sema::Result<()>) {
    compile_with_dependency(module, main, None)
}

fn compile_with_dependency(
    module: &str,
    main: &str,
    dependency: Option<&str>,
) -> (Compiler, jaic::sema::Result<()>) {
    let mut fs = VirtualFs::default();
    fs.insert("/lib/Choice/module.jai", module.as_bytes());
    fs.insert("/app/main.jai", main.as_bytes());
    if let Some(source) = dependency {
        fs.insert("/lib/Dep/module.jai", source.as_bytes());
    }
    let mut options = Options::host();
    options.runtime_support = false;
    options.import_paths = vec!["/lib".into()];
    let mut compiler = Compiler::new(options, Rc::new(fs));
    let result = compiler.compile_program(Path::new("/app/main.jai"));
    (compiler, result)
}

const INTERFACE: &str = r#"#module_parameters ()(Implementation: $I/interface Protocol = Fallback) {
    Protocol :: struct {
        run :: (value: s64) -> s64 {
            return value;
        }
    }

    Fallback :: struct {
        run :: (value: s64) -> s64 {
            return value;
        }
    }
}

Number :: s64;
selected_type :: Implementation;

read :: () -> s64 {
    return Implementation.run(7);
}
"#;

const PLAIN_IMPORT: &str = r#"Public :: #import "Choice";
Custom :: #import "Choice"()(Implementation = Hooks);

Hooks :: struct {
    run :: (value: Public.Number) -> s64 {
        return value;
    }
}

#assert Custom.selected_type == Hooks;

main :: () {
    value := Custom.read();
}
"#;

#[test]
fn interface_program_setting_can_use_a_plain_imports_independent_public_type() {
    let (compiler, result) = compile(INTERFACE, PLAIN_IMPORT);
    if let Err(error) = result {
        panic!("{}", error.render(&compiler.sources));
    }
    assert_eq!(
        compiler
            .modules
            .iter()
            .filter(|m| m.name == "Choice")
            .count(),
        1
    );
}

#[test]
fn polymorphic_program_setting_uses_the_configured_value_before_validation() {
    let (compiler, result) = compile(
        &INTERFACE.replace("$I/interface Protocol", "$I"),
        PLAIN_IMPORT,
    );
    if let Err(error) = result {
        panic!("{}", error.render(&compiler.sources));
    }
}

#[test]
fn an_early_public_member_condition_does_not_consume_the_default() {
    let main = format!("#if Public.Number == s64 {{ early :: true; }} {PLAIN_IMPORT}");
    let (compiler, result) = compile(INTERFACE, &main);
    if let Err(error) = result {
        panic!("{}", error.render(&compiler.sources));
    }
}

#[test]
fn ordinary_interface_settings_keep_distinct_nominal_module_instances() {
    let module = format!(
        "{} Token :: struct {{ value:s64; }}",
        INTERFACE.replace("#module_parameters ()(", "#module_parameters (")
    );
    let main = format!(
        "{} #assert Public.Token != Custom.Token;",
        PLAIN_IMPORT.replace("\"Choice\"()(", "\"Choice\"(")
    );
    let (compiler, result) = compile(&module, &main);
    if let Err(error) = result {
        panic!("{}", error.render(&compiler.sources));
    }
    assert_eq!(
        compiler
            .modules
            .iter()
            .filter(|m| m.name == "Choice")
            .count(),
        2
    );
}

#[test]
fn unused_invalid_program_interface_is_still_rejected() {
    let (compiler, result) = compile(
        INTERFACE,
        "M :: #import \"Choice\"()(Implementation=Broken); Broken :: struct {} main :: () {}",
    );
    let error = result.expect_err("unused interface contracts must be checked");
    assert!(
        error.message.contains("run"),
        "{}",
        error.render(&compiler.sources)
    );
}

#[test]
fn a_failed_header_contract_remains_an_error_on_repeated_expansion() {
    let (mut compiler, result) = compile(
        INTERFACE,
        "M :: #import \"Choice\"()(Implementation=Broken); Broken :: struct {} main :: () {}",
    );
    let original = result.expect_err("invalid interface");
    let repeated = compiler
        .expand_all()
        .expect_err("the error cannot be consumed");
    assert_eq!(original.message, repeated.message);
    assert_eq!(original.span, repeated.span);
}

#[test]
fn a_real_program_value_dependency_cycle_is_still_rejected() {
    let main = "Public :: #import \"Choice\"; Custom :: #import \"Choice\"()(Implementation=Hooks); Hooks :: Public.selected_type; main::(){}";
    let (compiler, result) = compile(INTERFACE, main);
    let error = result.expect_err("the selected value cannot depend on itself");
    assert!(
        error.message.contains("circular"),
        "{}",
        error.render(&compiler.sources)
    );
}

#[test]
fn conflicting_program_settings_are_still_rejected() {
    let main = "A :: #import \"Choice\"()(Implementation=One); B :: #import \"Choice\"()(Implementation=Two); One :: struct {run::(value:s64)->s64{return value;}} Two :: struct {run::(value:s64)->s64{return value;}} main::(){}";
    let (compiler, result) = compile(INTERFACE, main);
    let error = result.expect_err("conflicting settings cannot be overwritten");
    assert!(
        error.message.contains("another value"),
        "{}",
        error.render(&compiler.sources)
    );
}

#[test]
fn a_valid_unused_default_contract_is_checked_without_configuration() {
    let (compiler, result) = compile(INTERFACE, "M :: #import \"Choice\"; main::(){}");
    if let Err(error) = result {
        panic!("{}", error.render(&compiler.sources));
    }
}

#[test]
fn interfaces_loaded_only_while_lowering_a_body_are_still_checked() {
    for configured in [false, true] {
        let module = if configured {
            INTERFACE.to_owned()
        } else {
            INTERFACE.replace("= Fallback)", "= Broken)") + " Broken :: struct {}"
        };
        let arguments = if configured {
            "()(Implementation=Broken)"
        } else {
            ""
        };
        let main = format!(
            "Broken :: struct {{}} main :: () {{ M :: #import \"Choice\"{arguments}; value: M.Number; }}"
        );
        let (compiler, result) = compile(&module, &main);
        let error = result.expect_err("body-only modules also retain interface contracts");
        assert!(
            error.message.contains("run"),
            "{}",
            error.render(&compiler.sources)
        );
    }
}

#[test]
fn modules_discovered_by_a_late_contract_keep_their_top_level_errors() {
    let module = INTERFACE.replace("= Fallback)", "= Dep.Hooks)") + " Dep :: #import \"Dep\";";
    let dependency = "Hooks :: struct { run :: (value:s64)->s64 {return value;} } #assert false;";
    let (compiler, result) = compile_with_dependency(
        &module,
        "main :: () { M :: #import \"Choice\"; value: M.Number; }",
        Some(dependency),
    );
    let error = result.expect_err("newly discovered module work must settle");
    assert!(
        error.message.contains("assert"),
        "{}",
        error.render(&compiler.sources)
    );
}

#[test]
fn late_module_runs_execute_once_and_repeated_settlement_is_idempotent() {
    let module = INTERFACE.replace("= Fallback)", "= Dep.Hooks)") + " Dep :: #import \"Dep\";";
    let dependency = "counter:s64; #run {counter+=1;} #assert counter==1; Hooks :: struct { run :: (value:s64)->s64 {return value;} }";
    let (mut compiler, result) = compile_with_dependency(
        &module,
        "main :: () { M :: #import \"Choice\"; value: M.Number; }",
        Some(dependency),
    );
    if let Err(error) = result {
        panic!("{}", error.render(&compiler.sources));
    }
    compiler.settle().unwrap();
    compiler
        .add_source(&jaic::sema::ProgramSource::String(
            "Observe :: #import \"Dep\"; #assert Observe.counter==1;".into(),
        ))
        .unwrap();
    if let Err(error) = compiler.settle() {
        panic!("{}", error.render(&compiler.sources));
    }
}
