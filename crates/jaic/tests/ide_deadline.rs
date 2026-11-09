//! The wall-clock limit of an editor's check also reaches the compiler's own work, not only
//! interpreted code: a check that spends its time elsewhere stops with a "checking stopped"
//! notice instead of running its course.
use jaic::ide::lint_compile::{CheckLimits, LintCompile, LintRoot, LintSetup};
use jaic::interp::SandboxHost;
use jaic::sema::{Options, VirtualFs};
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

const PRELUDE: &str = "Operating_System_Tag :: enum u32 { WINDOWS; LINUX; MACOS; WASM; } CPU_Tag :: enum u32 { X64; ARM64; CUSTOM; }";

/// Compile `source` with a wall-clock limit that has passed before the compiler starts: the
/// error it fails with, and whether the check says it stopped.
fn check(source: &str) -> (String, bool) {
    let mut fs = VirtualFs::default();
    fs.insert("/main.jai", source.as_bytes());
    fs.insert("/prelude/module.jai", PRELUDE.as_bytes());
    let fs = Rc::new(fs);
    let mut options = Options::host();
    options.runtime_support = false;
    options.preload = Some("/prelude/module.jai".into());
    let mut lint = LintCompile::start(LintSetup {
        fs,
        options,
        host: Rc::new(RefCell::new(SandboxHost::default())),
        observer: None,
        report: Box::new(|_, _| {}),
        limits: CheckLimits::new(CheckLimits::DEFAULT_EVAL_STEPS, 1e-9),
    });
    let result = lint.compile(LintRoot::Program(Path::new("/main.jai")), &[]);
    let error = result.err().map(|e| e.render(&lint.compiler.src.sources));
    (error.unwrap_or_default(), lint.stopped())
}

#[test]
fn polymorph_recursion_stops_at_the_limit() {
    // Without the limit this runs to the instance cap and fails with its own error.
    let source = "
nest :: (x: $T, n: int) { if n > 0 nest(*x, n - 1); }
main :: () { v := 1; nest(v, 3); }
";
    let (error, stopped) = check(source);
    assert!(error.contains("checking stopped"), "{error}");
    assert!(stopped);
}

#[test]
fn queued_bodies_stop_at_the_limit() {
    let mut source = String::new();
    for i in 0..50 {
        source += &format!("f{i} :: () {{ x := {i}; }}\n");
    }
    source += "main :: () {";
    for i in 0..50 {
        source += &format!(" f{i}();");
    }
    source += " }\n";
    let (error, stopped) = check(&source);
    assert!(error.contains("checking stopped"), "{error}");
    assert!(stopped);
}

#[test]
fn macro_expansion_and_struct_instances_stop_at_the_limit() {
    let source = "
twice :: (code: Code) #expand { `#insert code; `#insert code; }
Box :: struct ($T: Type) { v: T; }
main :: () { x := 0; twice(#code { x += 1; }); b: Box(int); }
";
    let (error, stopped) = check(source);
    assert!(error.contains("checking stopped"), "{error}");
    assert!(stopped);
}
