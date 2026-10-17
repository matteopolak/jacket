//! Compilation helpers shared by the integration tests.
#![allow(dead_code)]
use jaic::sema::{Compiler, Options, ProgramSource, TargetCpu, VirtualFs};
use std::{path::Path, rc::Rc};

/// A compiler with the runtime support module turned off, so tests see only their own source.
pub fn bare_compiler() -> Compiler {
    bare_compiler_for(None)
}

fn bare_compiler_for(cpu: Option<TargetCpu>) -> Compiler {
    let mut options = Options::host();
    if let Some(cpu) = cpu {
        options.cpu = cpu;
    }
    options.runtime_support = false;
    Compiler::new(options, Rc::new(VirtualFs::default()))
}

/// Compile `source` as one string program; the compiler is returned for inspection.
pub fn compile_source(source: &str) -> (Compiler, jaic::sema::Result<()>) {
    let mut c = bare_compiler();
    let result = c.compile_sources(&[ProgramSource::String(source.into())]);
    (c, result)
}

/// Compile `source` for `cpu`.
pub fn compile_for_cpu(cpu: TargetCpu, source: &str) -> (Compiler, jaic::sema::Result<()>) {
    let mut c = bare_compiler_for(Some(cpu));
    let result = c.compile_sources(&[ProgramSource::String(source.into())]);
    (c, result)
}

/// Compile `source` as `/main.jai`, with any error rendered against the sources.
pub fn compile_main_file(source: &str) -> Result<(), String> {
    let mut fs = VirtualFs::default();
    fs.insert("/main.jai", source.as_bytes());
    let mut options = Options::host();
    options.runtime_support = false;
    let mut compiler = Compiler::new(options, Rc::new(fs));
    compiler
        .compile_program(Path::new("/main.jai"))
        .map_err(|error| error.render(&compiler.sources))
}
