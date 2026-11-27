//! Compile-time code built for another C library than the host's must not call it natively.
use jaic::sema::{Compiler, NativeFs, Options, ProgramSource};
use jaic::target::TargetOs;
use std::{path::PathBuf, rc::Rc};

/// A POSIX target whose C library is not the host's.
fn other_c_library() -> TargetOs {
    if cfg!(target_os = "linux") {
        TargetOs::MacOS
    } else {
        TargetOs::Linux
    }
}

fn compile(os: TargetOs, body: &str) -> Result<(), String> {
    let stdlib = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    let mut options = Options::host();
    options.os = os;
    options.import_paths = vec![stdlib.clone()];
    options.set_stdlib(&NativeFs, &stdlib);
    let mut compiler = Compiler::new(options, Rc::new(NativeFs));
    let source = format!(
        r#"
        #import "Basic";
        #import "File_Utilities";
        // A `#no_reset` global keeps every run on the target compiler.
        #no_reset ran := false;
        #run {{
            {body}
            ran = true;
        }}
        main :: () {{}}
    "#
    );
    compiler
        .compile_sources(&[ProgramSource::String(source)])
        .map_err(|error| error.render(&compiler.src.sources))
}

#[test]
fn a_target_c_library_call_on_another_host_is_refused() {
    let file = env!("CARGO_MANIFEST_DIR");
    let error = compile(
        other_c_library(),
        &format!(r#"assert(file_exists("{file}"));"#),
    )
    .unwrap_err();
    assert!(
        error.contains("which the C library of this") && error.contains("does not provide"),
        "{error}"
    );
}

#[test]
fn calls_every_c_library_makes_the_same_way_still_run() {
    // Allocation, strings and output reach the C library as on the host.
    compile(
        other_c_library(),
        r#"text := tprint("% %", 1, "two"); assert(text == "1 two"); write_string("");"#,
    )
    .unwrap_or_else(|error| panic!("{error}"));
}
