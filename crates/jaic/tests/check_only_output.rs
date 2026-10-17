//! A check-only child must retain checked IR for subsequent semantic analysis.
use jaic::build::{BuildEnv, BuildSettings, OutputBackend, WorkspaceObserver, Workspaces};
use jaic::sema::procs::ProcTarget;
use jaic::sema::{Compiler, NativeFs, Options, ProgramSource};
use std::{
    cell::RefCell,
    path::{Path, PathBuf},
    rc::Rc,
};

struct Retain(Rc<RefCell<Vec<Compiler>>>);

impl WorkspaceObserver for Retain {
    fn created(&mut self, _: &mut Compiler) {
    }

    fn finished(&mut self, compiler: Box<Compiler>, failed: bool) {
        assert!(!failed, "independent child must compile");
        self.0.borrow_mut().push(*compiler);
    }
}

fn checked_child(backend: Option<Box<dyn OutputBackend>>) -> Rc<RefCell<Vec<Compiler>>> {
    let stdlib = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    let mut options = Options::host();
    options.import_paths = vec![stdlib.clone()];
    options.preload = Some(stdlib.join(jaic::PRELOAD_ENTRY));
    let fs = Rc::new(NativeFs);
    let captured = Rc::new(RefCell::new(Vec::new()));
    let workspaces = Workspaces::new(BuildEnv {
        fs: fs.clone(),
        options: options.clone(),
        backend,
        unwritten_output_hint: None,
        command_line: Vec::new(),
        make_host: Box::new(|_| Box::new(jaic::interp::SandboxHost::default())),
        report: Box::new(|message| panic!("unexpected child diagnostic: {message}")),
        observer: Some(Box::new(Retain(captured.clone()))),
    });
    let mut compiler = Compiler::new(options, fs);
    compiler.interp.host = Box::new(jaic::interp::SandboxHost::default());
    compiler.attach_workspaces(workspaces.clone());
    compiler
        .compile_sources(&[ProgramSource::String(
            r#"
        #import "Compiler";
        #run {
            w := compiler_create_workspace("checked");
            add_build_string("predicate :: (T: Type) -> bool {return T == s64;} main :: () {}", w);
        }
    "#
            .into(),
        )])
        .unwrap_or_else(|error| panic!("{}", error.render(&compiler.sources)));
    jaic::build::finish_all(&workspaces, &mut compiler.interp.block_budget).unwrap();
    captured
}

#[test]
fn check_only_child_keeps_unreferenced_checked_bodies_for_later_analysis() {
    let captured = checked_child(None);
    let children = captured.borrow();
    assert_eq!(children.len(), 1);
    let child = &children[0];
    let procedure = child
        .procs
        .iter()
        .find(|p| p.name.as_str() == "predicate")
        .unwrap();
    let Some(ProcTarget::Func(function)) = procedure.target else {
        panic!("predicate was not checked");
    };
    assert!(
        child.program.funcs[function.0 as usize]
            .as_ref()
            .is_some_and(|body| !body.blocks.is_empty()),
        "check-only output preparation discarded a checked body needed by editor #modify analysis"
    );
}


struct InspectEmission(Rc<RefCell<Vec<bool>>>);

impl OutputBackend for InspectEmission {
    fn write_output(
        &mut self,
        program: &jaic::ir::Program,
        _: &BuildSettings,
        _: &Path,
    ) -> Result<(), String> {
        let predicate = program
            .func_names
            .iter()
            .position(|name| name == "predicate")
            .unwrap();
        self.0.borrow_mut().push(program.funcs[predicate].is_none());
        Ok(())
    }
}

#[test]
fn an_output_backend_still_receives_pruned_emission_ir() {
    let observations = Rc::new(RefCell::new(Vec::new()));
    let captured = checked_child(Some(Box::new(InspectEmission(observations.clone()))));
    assert_eq!(
        &*observations.borrow(),
        &[true],
        "an actual output backend must retain normal dead-code elimination"
    );
    assert_eq!(captured.borrow().len(), 1);
}
