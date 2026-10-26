//! Each top-level compile-time evaluation has a block budget of its own, and a host can stop
//! compile-time code altogether (an editor's wall-clock limit).
use jaic::interp::{SandboxHost, SharedHost};
use jaic::sema::{Compiler, Options, VirtualFs};
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;
use std::time::Duration;

const PRELUDE: &str = "Operating_System_Tag :: enum u32 { WINDOWS; LINUX; MACOS; WASM; } CPU_Tag :: enum u32 { X64; ARM64; CUSTOM; }";

fn compiler(source: &str, limit: Option<u64>) -> Compiler {
    let mut fs = VirtualFs::default();
    fs.insert("/main.jai", source.as_bytes());
    fs.insert("/prelude/module.jai", PRELUDE.as_bytes());
    let mut options = Options::host();
    options.runtime_support = false;
    options.preload = Some("/prelude/module.jai".into());
    options.eval_block_limit = limit;
    Compiler::new(options, Rc::new(fs))
}

fn run(compiler: &mut Compiler) -> Result<(), String> {
    compiler
        .compile_program(Path::new("/main.jai"))
        .map_err(|error| error.render(&compiler.src.sources))
}

/// Eight runs of a thousand blocks each.
const EIGHT_RUNS: &str = "
count :: (n: int) -> int { total := 0; for 0..n { total += it; } return total; }
#run count(1000); #run count(1000); #run count(1000); #run count(1000);
#run count(1000); #run count(1000); #run count(1000); #run count(1000);
main :: () {}
";

#[test]
fn every_evaluation_starts_with_its_own_budget() {
    // Together the runs spend more than the limit, but none of them does alone.
    let mut each = compiler(EIGHT_RUNS, Some(2000));
    run(&mut each).unwrap();
    // With one budget for the whole compile, as before, the later runs trap.
    let mut shared = compiler(EIGHT_RUNS, None);
    shared.ct.interp.block_budget = Some(2000);
    let error = run(&mut shared).unwrap_err();
    assert!(error.contains("execution budget exhausted"), "{error}");
}

#[test]
fn a_runaway_run_fails_alone_and_says_where_it_was() {
    let source = "
spin :: () { while true {} }
outer :: () { spin(); }
#run outer();
main :: () {}
";
    let mut c = compiler(source, Some(1000));
    let error = run(&mut c).unwrap_err();
    assert!(
        error.contains("execution budget exhausted (1000 basic blocks)"),
        "{error}"
    );
    assert!(error.contains("still running `spin` at"), "{error}");
    assert!(error.contains("called from `outer` at"), "{error}");
    assert!(
        error.contains("raise the compile-time evaluation budget"),
        "{error}"
    );
    // The error is at the directive that started the run, not inside the loop.
    assert!(error.contains("#run outer()"), "{error}");
    assert_eq!(c.ct.interp.budget_traps, 1);
}

#[test]
fn runs_after_a_runaway_one_still_run() {
    // The runaway is first in the program; the constant after it is evaluated by its own budget.
    let source = "
spin :: () { while true {} }
Bad :: #run spin();
Good :: #run 1 + 2;
main :: () {}
";
    let mut c = compiler(source, Some(1000));
    let error = run(&mut c).unwrap_err();
    assert!(error.contains("still running `spin`"), "{error}");
    let diagnostics = c.ide_check_all();
    let messages: Vec<_> = diagnostics.iter().map(|d| d.message.as_str()).collect();
    let exhausted = messages
        .iter()
        .filter(|m| m.contains("execution budget exhausted"))
        .count();
    assert!(exhausted <= 1, "{messages:#?}");
}

#[test]
fn a_stopped_host_runs_no_compile_time_code() {
    let host = Rc::new(RefCell::new(SandboxHost::default()));
    host.borrow_mut().stop_after(Duration::ZERO);
    let mut c = compiler(
        "count :: (n: int) -> int { total := 0; for 0..n { total += it; } return total; } #run count(3); main :: () {}",
        Some(1000),
    );
    c.ct.interp.host = Box::new(SharedHost(host));
    let error = run(&mut c).unwrap_err();
    assert!(error.contains("checking stopped"), "{error}");
}

#[test]
fn the_wall_clock_limit_interrupts_a_running_loop() {
    let host = Rc::new(RefCell::new(SandboxHost::default()));
    host.borrow_mut().stop_after(Duration::from_millis(100));
    let mut c = compiler("#run { while true {} } main :: () {}", None);
    c.ct.interp.block_budget = Some(u64::MAX);
    c.ct.interp.host = Box::new(SharedHost(host.clone()));
    let error = run(&mut c).unwrap_err();
    assert!(error.contains("checking stopped"), "{error}");
    assert!(!error.contains("budget"), "{error}");
    assert!(host.borrow_mut().has_stopped());
}
