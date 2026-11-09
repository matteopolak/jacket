//! The wall-clock limit of an editor's check also reaches the compiler's own work, not only
//! interpreted code: a check that spends its time elsewhere stops with a "checking stopped"
//! notice instead of running its course.
use jaic::ide::lint_compile::{CheckLimits, LintCompile, LintRoot, LintSetup};
use jaic::interp::SandboxHost;
use jaic::sema::{Options, VirtualFs};
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;
use std::time::{Duration, Instant};

const PRELUDE: &str = "Operating_System_Tag :: enum u32 { WINDOWS; LINUX; MACOS; WASM; } CPU_Tag :: enum u32 { X64; ARM64; CUSTOM; }";

/// Compile `source` with a wall-clock limit that has passed before the compiler starts: the
/// error it fails with, and whether the check says it stopped.
fn check(source: &str) -> (String, bool) {
    check_within(source, 1e-9)
}

/// `check` with a wall-clock limit of `seconds`, counted from the start of the compile.
fn check_within(source: &str, seconds: f64) -> (String, bool) {
    let (error, stopped, _) = run(source, seconds, |_| {});
    (error, stopped)
}

/// `check` that asks the host to stop after `polls` questions instead of by the clock, so where
/// the compiler asks decides the result and the machine's speed does not. Also how many
/// questions the compile asked.
fn check_polls(source: &str, polls: u64) -> (String, bool, u64) {
    run(source, 1e6, |host| host.stop_after_polls(polls))
}

fn run(
    source: &str,
    seconds: f64,
    configure: impl FnOnce(&mut SandboxHost),
) -> (String, bool, u64) {
    let mut fs = VirtualFs::default();
    fs.insert("/main.jai", source.as_bytes());
    fs.insert("/prelude/module.jai", PRELUDE.as_bytes());
    let fs = Rc::new(fs);
    let mut options = Options::host();
    options.runtime_support = false;
    options.preload = Some("/prelude/module.jai".into());
    let host = Rc::new(RefCell::new(SandboxHost::default()));
    configure(&mut host.borrow_mut());
    let mut lint = LintCompile::start(LintSetup {
        fs,
        options,
        host: host.clone(),
        observer: None,
        report: Box::new(|_, _| {}),
        limits: CheckLimits::new(CheckLimits::DEFAULT_EVAL_STEPS, seconds),
    });
    let result = lint.compile(LintRoot::Program(Path::new("/main.jai")), &[]);
    let error = result.err().map(|e| e.render(&lint.compiler.src.sources));
    let polls = host.borrow().polls();
    (error.unwrap_or_default(), lint.stopped(), polls)
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

#[test]
fn many_inserted_strings_stop_at_the_limit() {
    // One body, so no unit of work starts between the inserts: each insert must ask.
    let mut source = String::from("main :: () { x := 0;\n");
    for _ in 0..100_000 {
        source += "#insert \"x += 1;\";\n";
    }
    source += "}\n";
    let (error, stopped) = check_within(&source, 0.05);
    assert!(error.contains("checking stopped"), "{error}");
    assert!(stopped);
}

#[test]
fn a_long_inserted_string_stops_in_the_parser() {
    let program = |text: &str| format!("main :: () {{ x := 0; #insert \"{text}\"; }}\n");
    // The questions a whole compile asks when there is nothing to parse.
    let (error, stopped, base) = check_polls(&program(""), u64::MAX);
    assert!(!stopped, "{error}");
    // A few more than that are all the answer there is room for before the end of the check,
    // and only the parser, asking every 256 statements, gets to use them: it is told to stop
    // before the inserted statements are checked. The stop is reported at the `#insert`, not at
    // a statement of the inserted text.
    let (error, stopped, _) = check_polls(&program(&"x += 1;".repeat(100_000)), base + 5);
    assert!(error.contains("checking stopped"), "{error}");
    assert!(error.contains("main.jai"), "{error}");
    assert!(!error.contains("#insert at"), "{error}");
    assert!(stopped);
}

#[test]
fn a_very_wide_expression_stops_at_the_limit() {
    let program = |elements: usize| {
        let list = vec!["1"; elements].join(", ");
        format!("main :: () {{ a := int.[{list}]; }}\n")
    };
    let (error, stopped, base) = check_polls(&program(1), u64::MAX);
    assert!(!stopped, "{error}");
    // Only the elements grow: the compile ends before them unless checking them asks too.
    let (error, stopped, _) = check_polls(&program(5_000), base + 5);
    assert!(error.contains("checking stopped"), "{error}");
    assert!(stopped);
}

#[test]
fn a_blocking_foreign_call_does_not_hold_the_check() {
    // The editor's host answers libc itself on a virtual clock: `sleep` does not block, and a
    // program that sleeps forever is stopped by the limit like any other loop.
    let source = "
libc :: #system_library \"libc\";
sleep :: (seconds: u32) -> u32 #foreign libc;
#run sleep(3600);
#run { while true sleep(3600); }
main :: () {}
";
    let start = Instant::now();
    let (error, stopped) = check_within(source, 0.5);
    assert!(error.contains("checking stopped"), "{error}");
    assert!(stopped);
    assert!(
        start.elapsed() < Duration::from_secs(30),
        "{:?}",
        start.elapsed()
    );
}
