//! Required target libraries and embedding host services have separate phases.
use jaic::{
    interp::{Host, Interp},
    ir,
    sema::{Compiler, Options, ProgramSource, TargetCpu, TargetOs, VirtualFs},
};
use std::{cell::Cell, rc::Rc};

struct Observer {
    calls: Rc<Cell<usize>>,
    value: Box<i64>,
}
impl Host for Observer {
    fn write(&mut self, _: &[u8], _: bool) {
    }
    fn native_linking(&self) -> bool {
        true
    }
    fn foreign(&mut self, _: &str, _: &[u64], _: &ir::Sig) -> Option<Result<Vec<u64>, String>> {
        None
    }
    fn foreign_data(&mut self, symbol: &str) -> Option<u64> {
        if symbol != "independent_embedding_data_603" {
            return None;
        }
        self.calls.set(self.calls.get() + 1);
        Some((&*self.value as *const i64) as u64)
    }
}

fn compiler(cross_os: bool, cross_cpu: bool) -> (Compiler, Rc<Cell<usize>>) {
    let mut options = Options::host();
    options.runtime_support = false;
    if cross_os {
        options.os = if options.os == TargetOs::Windows {
            TargetOs::Linux
        } else {
            TargetOs::Windows
        };
    }
    if cross_cpu {
        options.cpu = if options.cpu == TargetCpu::X64 {
            TargetCpu::Arm64
        } else {
            TargetCpu::X64
        };
    }
    let mut compiler = Compiler::new(options, Rc::new(VirtualFs::default()));
    let calls = Rc::new(Cell::new(0));
    compiler.interp = Interp::new(Box::new(Observer {
        calls: calls.clone(),
        value: Box::new(37),
    }));
    (compiler, calls)
}
const REQUIREMENT: &str =
    "required :: #library,link_always \"independent_missing_target_requirement_603\";";
const HOST_CALL: &str = "independent_embedding_data_603: int #elsewhere; #assert independent_embedding_data_603==37; main :: () {}";

#[test]
fn cross_target_host_service_does_not_load_unrelated_target_requirements() {
    for (os, cpu) in [(true, false), (false, true), (true, true)] {
        let (mut c, calls) = compiler(os, cpu);
        let source = format!("{REQUIREMENT}{HOST_CALL}");
        if let Err(error) = c.compile_sources(&[ProgramSource::String(source)]) {
            panic!("{}", error.render(&c.sources));
        }
        assert_eq!(calls.get(), 1);
        assert!(c.program.cross_target);
        assert_eq!(c.program.libraries.len(), 1);
        assert!(c.program.libraries[0].link_always);
    }
}

#[test]
fn same_target_host_service_still_initializes_required_libraries() {
    let (mut c, calls) = compiler(false, false);
    let error = c
        .compile_sources(&[ProgramSource::String(format!("{REQUIREMENT}{HOST_CALL}"))])
        .expect_err("missing required library must fail");
    assert!(
        error.message.contains("required native library"),
        "{}",
        error.render(&c.sources)
    );
    assert_eq!(calls.get(), 0);
}

#[test]
fn runtime_requirement_is_not_removed_from_cross_target_program() {
    let (mut c, _) = compiler(true, true);
    c.compile_sources(&[ProgramSource::String(format!(
        "{REQUIREMENT}main :: () {{}}"
    ))])
    .unwrap();
    c.interp.compile_time = false;
    let error = c
        .interp
        .call(&c.program, ir::FuncId(0), &[])
        .expect_err("runtime required library remains mandatory");
    assert!(
        error.message.contains("required native library"),
        "{}",
        error.message
    );
}

#[test]
fn explicit_unavailable_target_foreign_is_still_rejected() {
    let (mut c, calls) = compiler(true, true);
    let source = format!(
        "{REQUIREMENT}target_call :: () -> int #foreign required \"independent_absent_target_symbol_603\"; #assert target_call()==1; main :: () {{}}"
    );
    let error = c
        .compile_sources(&[ProgramSource::String(source)])
        .expect_err("an actual unavailable target call must fail");
    assert!(
        error.message.contains("foreign procedure") && error.message.contains("not available"),
        "{}",
        error.render(&c.sources)
    );
    assert_eq!(calls.get(), 0);
}
