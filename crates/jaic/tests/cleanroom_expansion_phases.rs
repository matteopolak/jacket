//! Independent declaration discovery and configuration phase controls.
use jaic::sema::{Compiler, Options, VirtualFs};
use std::{path::Path, rc::Rc};

fn check(files: &[(&str, &str)]) -> Result<Compiler, String> {
    let mut fs = VirtualFs::default();
    for (path, source) in files {
        fs.insert(*path, source.as_bytes());
    }
    let mut options = Options::host();
    options.runtime_support = false;
    options.preload = Some("/prelude.jai".into());
    fs.insert("/prelude.jai", b"Operating_System_Tag :: enum u32 { WINDOWS; LINUX; MACOS; WASM; } CPU_Tag :: enum u32 { X64; ARM64; CUSTOM; }".as_slice());
    options.import_paths = vec!["/lib".into()];
    let mut compiler = Compiler::new(options, Rc::new(fs));
    compiler
        .compile_program(Path::new("/main.jai"))
        .map_err(|error| error.render(&compiler.sources))?;
    Ok(compiler)
}

#[test]
fn static_switch_load_precedes_defaulted_global_using_context_layout() {
    check(&[
        ("/main.jai", "#import \"Backend\"; main :: () {context.marker=9;}"),
        ("/lib/Backend/module.jai", "#load \"settings.jai\"; #if OS == {case .WINDOWS; #load \"context.jai\"; case .LINUX; #load \"context.jai\"; case .MACOS; #load \"context.jai\"; case; #load \"wrong.jai\";}"),
        ("/lib/Backend/settings.jai", "Settings :: struct {label:=\"default\"; count:=3;} #scope_module using settings:Settings;"),
        ("/lib/Backend/context.jai", "#add_context marker:int;"),
        ("/lib/Backend/wrong.jai", "#assert false;"),
    ]).unwrap();
}
