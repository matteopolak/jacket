//! Compiler hooks called from a thread a compile-time run started see the same symbols and record
//! names as the compiling thread.
#![cfg(unix)]
use jaic::build::{BuildEnv, Workspaces};
use jaic::sema::{Compiler, NativeFs, Options, ProgramSource};
use std::{path::Path, rc::Rc};

const PROGRAM: &str = r#"
#import "Basic";
#import "Compiler";

libc :: #system_library "libc";
pthread_create :: (thread: *u64, attributes: *void, entry: *void, data: *void) -> s32 #foreign libc;
pthread_join :: (thread: u64, result: **void) -> s32 #foreign libc;

source: Code;
made: Code;

reparse :: () {
    root := compiler_get_nodes(source);
    made = compiler_get_code(root);
}

worker :: (data: *void) -> *void #c_call {
    ctx: #Context;
    push_context ctx { reparse(); }
    return null;
}

ident_name :: (code: Code) -> string {
    root := compiler_get_nodes(code);
    binary := cast(*Code_Binary_Operator) root;
    ident := cast(*Code_Ident) binary.left;
    return copy_string(ident.name);
}

#run {
    source = #code jacket_threaded_name + 1;
    // Everything the thread calls is compiled first, on the compiling thread.
    reparse();
    assert(ident_name(made) == "jacket_threaded_name");
    made = source;
    thread: u64;
    assert(pthread_create(*thread, null, cast(*void) worker, null) == 0);
    assert(pthread_join(thread, null) == 0);
    assert(ident_name(made) == "jacket_threaded_name");
}

main :: () {}
"#;

#[test]
#[ignore = "record names are per thread until their table is global"]
fn code_parsed_on_a_started_thread_keeps_its_names() {
    let library = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../stdlib")
        .canonicalize()
        .unwrap();
    let mut options = Options::host();
    options.preload = Some(library.join(jaic::PRELOAD_ENTRY));
    options.import_paths = vec![library];
    let fs = Rc::new(NativeFs);
    let workspaces = Workspaces::new(BuildEnv {
        fs: fs.clone(),
        options: options.clone(),
        backend: None,
        unwritten_output_hint: None,
        command_line: Vec::new(),
        make_host: Box::new(|_| Box::new(jaic::interp::NativeHost)),
        report: Box::new(|_, message| panic!("unexpected diagnostic: {message}")),
        observer: None,
    });
    let mut compiler = Compiler::new(options, fs);
    compiler.attach_workspaces(workspaces);
    if let Err(error) = compiler.compile_sources(&[ProgramSource::String(PROGRAM.into())]) {
        panic!("{}", error.render(&compiler.sources));
    }
}
