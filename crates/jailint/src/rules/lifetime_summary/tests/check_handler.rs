//! Code the runtime's check handler reaches is runtime code, even when a `#run` also calls it.
use super::*;
use jaic::sema::VirtualFs;

#[test]
fn procedures_the_check_handler_calls_are_not_compile_time_only() {
    let mut fs = VirtualFs::default();
    fs.insert(
        "/lib/Shared/module.jai",
        b"cleanup :: () #no_context {}\n".as_slice(),
    );
    let runtime = "#module_parameters(DEFINE_SYSTEM_ENTRY_POINT: bool, DEFINE_INITIALIZATION: bool, ENABLE_BACKTRACE_ON_CRASH: bool);\n#import \"Shared\";\n#scope_file\n__jaic_check_failed :: (r: s64, a: s64, b: s64, fatal: bool, line: s64, file: *u8) #no_context #no_abc #no_aoc { cleanup(); }\n";
    fs.insert("/lib/Runtime_Support/module.jai", runtime.as_bytes());
    let mut options = Options::host();
    options.import_paths = vec!["/lib".into()];
    let mut c = Compiler::new(options, Rc::new(fs));
    let mut facts = IdeFacts::new(vec![IdeRoot::Virtual]);
    facts.lint = true;
    c.ed.record(facts);
    let source = "#import \"Shared\";\n#run cleanup();\nmain :: () {}\n";
    c.begin_sources(&[ProgramSource::String(source.into())])
        .unwrap();
    c.finish_program().unwrap();
    let _ = c.ide_check_all();
    assert!(c.out.program.check_failed.is_some());
    let summaries = Summaries::build(&c);
    let (index, _) =
        c.pr.procs
            .iter()
            .enumerate()
            .find(|(_, p)| p.name.as_str() == "cleanup")
            .unwrap();
    assert!(
        !summaries.compile_time_only.contains(&ProcId(index as u32)),
        "the check handler calls it at run time"
    );
}
