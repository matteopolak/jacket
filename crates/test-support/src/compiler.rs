//! Compile helpers over `jaic`, for tests that inspect what the compiler decided.
//!
//! Needs the `compiler` feature.

use std::{path::Path, rc::Rc};

use jaic::sema::{Compiler, Options, ProgramSource, VirtualFs};
use jaic::target::TargetCpu;

/// A compiler with the runtime support module turned off, so tests see only their own source.
#[must_use]
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

/// Compile `source` as one string program with the runtime support module off, and render any
/// error against the sources.
///
/// # Errors
/// The rendered diagnostic when the program does not compile.
pub fn compile_rendered(source: &str) -> Result<(), String> {
    let (compiler, result) = compile_source(source);

    result.map_err(|error| error.render(&compiler.src.sources))
}

/// Compile `source` as `/main.jai`, with any error rendered against the sources.
///
/// # Errors
/// The rendered diagnostic when the program does not compile.
pub fn compile_main_file(source: &str) -> Result<(), String> {
    let mut fs = VirtualFs::default();

    fs.insert("/main.jai", source.as_bytes());
    let mut options = Options::host();

    options.runtime_support = false;
    let mut compiler = Compiler::new(options, Rc::new(fs));

    compiler
        .compile_program(Path::new("/main.jai"))
        .map_err(|error| error.render(&compiler.src.sources))
}

/// Compile the program `main` from an in-memory file system holding `files` (path, text), with
/// the runtime support module off. `tweak` sets any other option, such as `import_paths`; the
/// compiler is returned for inspection.
pub fn compile_virtual(
    files: &[(&str, &str)],
    main: &str,
    tweak: impl FnOnce(&mut Options),
) -> (Compiler, jaic::sema::Result<()>) {
    let mut fs = VirtualFs::default();

    for (path, text) in files {
        fs.insert(*path, text.as_bytes());
    }

    let mut options = Options::host();

    options.runtime_support = false;
    tweak(&mut options);
    let mut compiler = Compiler::new(options, Rc::new(fs));
    let result = compiler.compile_program(Path::new(main));

    (compiler, result)
}

/// [`compile_virtual`] with any error rendered against the sources.
///
/// # Errors
/// The rendered diagnostic when the program does not compile.
pub fn compile_virtual_rendered(
    files: &[(&str, &str)],
    main: &str,
    tweak: impl FnOnce(&mut Options),
) -> Result<(), String> {
    let (compiler, result) = compile_virtual(files, main, tweak);

    result.map_err(|error| error.render(&compiler.src.sources))
}

/// Compile `/main.jai` against library modules `modules` (name, text), each at
/// `/modules/<name>/module.jai`, which is also the import folder and the standard library
/// folder; the runtime support module is off.
///
/// # Errors
/// The diagnostic when the program does not compile.
pub fn compile_with_modules(
    main: &str,
    modules: &[(&str, &str)],
) -> Result<(), Box<jaic::diagnostics::Diagnostic>> {
    let paths: Vec<String> = modules
        .iter()
        .map(|(name, _)| format!("/modules/{name}/module.jai"))
        .collect();
    let mut files: Vec<(&str, &str)> = paths
        .iter()
        .zip(modules)
        .map(|(path, (_, source))| (path.as_str(), *source))
        .collect();

    files.push(("/main.jai", main));
    compile_virtual(&files, "/main.jai", |options| {
        options.import_paths = vec!["/modules".into()];
        options.preload_stdlib = Some("/modules".into());
    })
    .1
}

/// Compile `source` as one string program against the repository's standard library, with its
/// import folder and standard library folder set; the compiler is returned for inspection.
pub fn compile_with_stdlib(source: &str) -> (Compiler, jaic::sema::Result<()>) {
    let stdlib = crate::repo::stdlib();
    let mut options = Options::host();

    options.import_paths = vec![stdlib.clone()];
    options.set_stdlib(&jaic::sema::NativeFs, &stdlib);
    let mut compiler = Compiler::new(options, Rc::new(jaic::sema::NativeFs));
    let result = compiler.compile_sources(&[ProgramSource::String(source.into())]);

    (compiler, result)
}

/// Compile `source` as one string program against the repository's standard library, preloading
/// its prelude module; any error is rendered against the sources.
///
/// # Errors
/// The rendered diagnostic when the program does not compile.
pub fn compile_with_prelude(source: &str) -> Result<(), String> {
    let root = crate::repo::stdlib();
    let mut options = Options::host();

    options.import_paths = vec![root.clone()];
    options.preload = Some(root.join("prelude/module.jai"));
    let mut compiler = Compiler::new(options, Rc::new(jaic::sema::NativeFs));

    compiler
        .compile_sources(&[ProgramSource::String(source.into())])
        .map_err(|error| error.render(&compiler.src.sources))
}
