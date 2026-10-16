//! Program settings belong to workspace main inputs, not imported modules.
use jaic::sema::{Compiler, Options, ProgramSource, VirtualFs};
use std::{path::Path, rc::Rc};

fn compiler(files: &[(&str, &str)]) -> Compiler {
    let mut fs = VirtualFs::default();
    fs.insert("/lib/Settings/module.jai", b"#module_parameters (local:=3)(enabled:=true); public_enabled::enabled; public_local::local;".as_slice());
    for &(path, source) in files {
        fs.insert(path, source.as_bytes());
    }
    let mut options = Options::host();
    options.runtime_support = false;
    options.import_paths = vec!["/lib".into()];
    Compiler::new(options, Rc::new(fs))
}

fn success(mut compiler: Compiler) {
    if let Err(error) = compiler.compile_program(Path::new("/app/main.jai")) {
        panic!("{}", error.render(&compiler.sources));
    }
}

#[test]
fn main_and_loaded_main_files_may_supply_program_parameters() {
    for root in [
        "#load \"settings.jai\";",
        "#import \"Settings\"()(enabled=false);",
        "#import \"Settings\"(local=9)(enabled=false); #assert public_local==9 && !public_enabled;",
    ] {
        success(compiler(&[
            ("/app/main.jai", root),
            (
                "/app/settings.jai",
                "#scope_file #import \"Settings\"()(enabled=false); #assert !public_enabled;",
            ),
        ]));
    }
}

#[test]
fn nested_loads_and_anonymous_string_imports_keep_module_ownership() {
    for provider in [
        "#load \"settings.jai\"; ready::true;",
        "#import \"Settings\"()(enabled=unknown_argument); ready::true;",
    ] {
        let mut compiler = compiler(&[
            ("/app/main.jai", "#import \"Provider\"; #assert ready;"),
            ("/lib/Provider/module.jai", provider),
            (
                "/lib/Provider/settings.jai",
                "#import \"Settings\"()(enabled=false);",
            ),
        ]);
        let error = compiler
            .compile_program(Path::new("/app/main.jai"))
            .unwrap_err();
        assert!(
            error.message.contains("main program"),
            "{}",
            error.render(&compiler.sources)
        );
    }
    let mut compiler = compiler(&[(
        "/app/main.jai",
        "#import,string \"#import \\\"Settings\\\"()(enabled=false);\";",
    )]);
    let error = compiler
        .compile_program(Path::new("/app/main.jai"))
        .unwrap_err();
    assert!(
        error.message.contains("main program"),
        "{}",
        error.render(&compiler.sources)
    );
}

#[test]
fn imported_files_and_modules_may_only_supply_module_parameters() {
    for imported in [
        "#import \"Provider\";",
        "P::#import \"Provider\"; #assert P.ready;",
        "#import,file \"provider.jai\";",
    ] {
        let mut compiler = compiler(&[
            ("/app/main.jai", imported),
            (
                "/lib/Provider/module.jai",
                "#import \"Settings\"()(enabled=false); ready::true;",
            ),
            (
                "/app/provider.jai",
                "#scope_file #import \"Settings\"()(enabled=false);",
            ),
        ]);
        let error = compiler
            .compile_program(Path::new("/app/main.jai"))
            .expect_err("non-main PROGRAM settings must fail before binding");
        assert!(
            error.message.contains("main program"),
            "{}",
            error.render(&compiler.sources)
        );
    }
}

#[test]
fn ordinary_module_arguments_remain_valid_inside_imported_modules() {
    success(compiler(&[
        ("/app/main.jai", "#import \"Provider\"; #assert ready;"),
        (
            "/lib/Provider/module.jai",
            "S::#import \"Settings\"(local=9); ready::S.public_local==9;",
        ),
    ]));
}

#[test]
fn generated_main_inputs_and_each_workspace_root_have_their_own_origin() {
    for enabled in [false, true] {
        let mut compiler = compiler(&[]);
        let source = format!(
            "#import \"Settings\"()(enabled={enabled}); #assert public_enabled=={enabled};"
        );
        compiler
            .compile_sources(&[ProgramSource::String(source)])
            .unwrap_or_else(|error| panic!("{}", error.render(&compiler.sources)));
    }
}

#[test]
fn generated_module_inputs_do_not_acquire_main_program_privileges() {
    let mut compiler = compiler(&[]);
    compiler
        .prepare_sources(&[ProgramSource::String(String::new())])
        .unwrap();
    let module = compiler.new_module("independent", None, Vec::new());
    let error = compiler
        .add_source(&ProgramSource::ModuleString(
            "#import \"Settings\"()(enabled=false);".into(),
            module.0,
        ))
        .and_then(|()| compiler.settle())
        .expect_err("generated non-main inputs remain non-main");
    assert!(
        error.message.contains("main program"),
        "{}",
        error.render(&compiler.sources)
    );
}

#[test]
fn duplicate_and_mixed_list_bindings_are_still_rejected() {
    for source in [
        "#import \"Settings\"()(enabled=false,enabled=true);",
        "#import \"Settings\"(local=4)(local=5);",
    ] {
        let mut compiler = compiler(&[("/app/main.jai", source)]);
        assert!(
            compiler
                .compile_program(Path::new("/app/main.jai"))
                .is_err()
        );
    }
}
