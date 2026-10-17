//! Isolated host evaluation transfers values, never target lexical bindings.
use jaic::sema::{Compiler, Options, TargetCpu, TargetOs, VirtualFs};
use std::{path::Path, rc::Rc};

fn compile(source: &str, cross: bool) -> Result<(), String> {
    compile_configured(source, cross, |_| {})
}

fn compile_configured(
    source: &str,
    cross: bool,
    configure: impl FnOnce(&mut Compiler),
) -> Result<(), String> {
    let mut fs = VirtualFs::default();
    fs.insert("/main.jai", source.as_bytes());
    fs.insert("/prelude/module.jai", "Operating_System_Tag :: enum u32 { WINDOWS; LINUX; MACOS; WASM; } CPU_Tag :: enum u32 { X64; ARM64; CUSTOM; }".as_bytes());
    fs.insert(
        "/modules/Probe/module.jai",
        "IS_HOST :: OS==BUILD_OS && CPU==BUILD_CPU;".as_bytes(),
    );
    let mut options = Options::host();
    options.runtime_support = false;
    options.preload = Some("/prelude/module.jai".into());
    options.import_paths = vec!["/modules".into()];
    if cross {
        options.os = if options.os == TargetOs::Linux {
            TargetOs::Windows
        } else {
            TargetOs::Linux
        };
        options.cpu = if options.cpu == TargetCpu::X64 {
            TargetCpu::Arm64
        } else {
            TargetCpu::X64
        };
    }
    let mut compiler = Compiler::new(options, Rc::new(fs));
    configure(&mut compiler);
    compiler
        .compile_program(Path::new("/main.jai"))
        .map_err(|error| error.render(&compiler.sources))
}

#[test]
fn cross_target_host_constants_are_independent_of_target_constants() {
    compile("HOST :: #run,host -> bool {return OS==BUILD_OS && CPU==BUILD_CPU;}; #assert HOST; #assert OS!=BUILD_OS; main :: () {}", true).unwrap();
}

#[test]
fn host_imports_are_resolved_in_the_isolated_procedure() {
    // No bootstrap or native functions are needed to establish host isolation.
    compile("ANSWER :: #run,host -> int {return 6*7;}; TEXT :: #run,host -> string {return \"host value\";}; #assert ANSWER==42; #assert TEXT==\"host value\"; main :: () {}", true).unwrap();
    compile("Target :: #import \"Probe\"; HOST :: #run,host -> bool {Local :: #import \"Probe\"; return Local.IS_HOST;} #assert HOST; #assert !Target.IS_HOST; main :: () {}", true).unwrap();
}

#[test]
fn primitive_arrays_transfer_backing_storage_to_the_target() {
    compile("FIXED :: #run,host -> [3]int {return .[5,8,13];}; VIEW :: #run,host -> []int {v:[3]int=.[2,3,7];return v;} #assert FIXED[2]==13; #assert VIEW.count==3 && VIEW[0]==2 && VIEW[2]==7; main :: () {}", true).unwrap();
}

#[test]
fn host_result_lists_keep_every_primitive_value_at_both_binding_sites() {
    compile("N, OK :: #run,host -> int, bool {return 23, OS==BUILD_OS;} #assert N==23 && OK; exercise :: ()->int {n,ok:=#run,host -> int,bool {return 31, CPU==BUILD_CPU;} if !ok return -1; return n;} #assert exercise()==31; main :: () {}",true).unwrap();
}

#[test]
fn host_procedure_cannot_have_even_defaulted_parameters() {
    let error = compile(
        "N :: #run,host (arg:int=3)->int {return arg;}(); main :: () {}",
        true,
    )
    .expect_err("host entry procedure must have no parameters");
    assert!(error.contains("must take no arguments"), "{error}");
}

#[test]
fn unsupported_host_result_domains_keep_explicit_diagnostics() {
    for source in [
        "N :: #run,host -> *int {return null;}; main :: () {}",
        "N :: #run,host -> Type {return int;}; main :: () {}",
    ] {
        let error =
            compile(source, true).expect_err("host pointers and types cannot escape isolation");
        assert!(error.contains("#run,host"), "{error}");
    }
}

#[test]
fn cross_target_host_run_rejects_target_capture() {
    let error = compile(
        "target_only :: 17; ANSWER :: #run,host -> int {return target_only;}; main :: () {}",
        true,
    )
    .expect_err("isolated host evaluation cannot capture target declarations");
    assert!(
        error.contains("unknown identifier") && error.contains("target_only"),
        "{error}"
    );
}

#[test]
fn host_evaluation_shares_the_execution_budget() {
    let error = compile_configured(
        "#run,host {while true {}} main :: () {}",
        true,
        |compiler| compiler.interp.block_budget = Some(10),
    )
    .expect_err("isolated execution must not evade the caller's budget");
    assert!(error.contains("execution budget exhausted"), "{error}");
}

#[test]
fn host_effects_are_not_replayed_after_an_unresolved_target_body() {
    use std::cell::Cell;
    struct Observer(Rc<Cell<usize>>);
    impl jaic::interp::Host for Observer {
        fn native_linking(&self) -> bool {
            false
        }
        fn isolated_host(&self) -> Option<Box<dyn jaic::interp::Host>> {
            Some(Box::new(Self(self.0.clone())))
        }
        fn write(&mut self, _: &[u8], _: bool) {
            panic!("unexpected write");
        }
        fn foreign(
            &mut self,
            symbol: &str,
            _: &[u64],
            _: &jaic::ir::Sig,
        ) -> Option<Result<Vec<u64>, String>> {
            if symbol != "observe_host" {
                return None;
            }
            self.0.set(self.0.get() + 1);
            Some(Ok(vec![1]))
        }
    }
    let effects = Rc::new(Cell::new(0));
    let counter = effects.clone();
    let error=compile_configured("#placeholder Body; generator :: () { value := #run,host -> int { L :: #library \"observer\"; observe_host :: ()->int #foreign L; return observe_host(); } #insert Body; } #run generator(); main :: () {}",true,move |compiler|compiler.interp=jaic::interp::Interp::new(Box::new(Observer(counter)))).expect_err("unresolved body must remain a real failure");
    assert!(
        error.contains("Body") || error.contains("placeholder"),
        "{error}"
    );
    assert_eq!(
        effects.get(),
        1,
        "executed host work must not be treated as an unstarted target attempt"
    );
}

#[test]
fn custom_provider_without_isolation_cannot_fall_back_to_native_execution() {
    struct RestrictedHost;
    impl jaic::interp::Host for RestrictedHost {
        fn native_linking(&self) -> bool {
            false
        }
        fn write(&mut self, _: &[u8], _: bool) {
            panic!("unexpected host write");
        }
        fn foreign(
            &mut self,
            _: &str,
            _: &[u64],
            _: &jaic::ir::Sig,
        ) -> Option<Result<Vec<u64>, String>> {
            panic!("unexpected foreign call");
        }
    }
    let error = compile_configured("#run,host {} main :: () {}", true, |compiler| {
        compiler.interp = jaic::interp::Interp::new(Box::new(RestrictedHost))
    })
    .expect_err("an unsupported isolation provider must be diagnosed");
    assert!(
        error.contains("does not provide isolated #run,host"),
        "{error}"
    );
}

#[test]
fn non_cross_host_runs_keep_ordinary_lexical_behavior() {
    compile("local :: 17; ANSWER :: #run,host -> int {return local;}; #assert ANSWER==17; main :: () {}", false).unwrap();
}
