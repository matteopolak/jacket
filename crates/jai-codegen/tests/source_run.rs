//! Execute newly emitted code to prove source runs disappear from runtime bodies.
#[path = "support/native_tools.rs"]
mod native_tools;
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

#[test]
fn compile_time_procedure_result_is_embedded_in_a_native_binary() {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let root = std::env::temp_dir().join(format!(
        "jai-run-native-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&root).unwrap();
    struct Scratch(PathBuf);
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let scratch = Scratch(root);
    let source = scratch.0.join("main.jai");
    fs::write(&source, "answer :: #run multiply(6,7); multiply :: (a:int,b:int)->int { return a*b; } main :: ()->int { return answer; }").unwrap();
    let graph =
        jai_modules::ModuleGraph::load(&source, jai_modules::GraphOptions::default()).unwrap();
    let program = jai_sema::resolve_graph(&graph).unwrap();
    let jai_ir::EntryPoint::Int(entry) = program.entry() else {
        panic!("expected integer entry");
    };
    let main = program
        .procedures()
        .iter()
        .find(|procedure| procedure.id == entry)
        .unwrap();
    assert!(
        !format!("{:?}", main.body).contains("Call"),
        "source run must be constant IR"
    );
    let llvm = jai_codegen::emit(&program).unwrap();
    let input = scratch.0.join("main.ll");
    let executable = scratch.0.join("main");
    fs::write(&input, llvm).unwrap();
    let output = native_tools::clang_command()
        .arg(&input)
        .arg("-o")
        .arg(&executable)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(Command::new(executable).status().unwrap().code(), Some(42));
}

#[derive(Default)]
struct WorkspaceEffects {
    staged: Vec<jai_vm::CompilerRequest>,
    committed: Vec<jai_vm::CompilerRequest>,
}
impl jai_vm::CompilerEffects for WorkspaceEffects {
    fn begin(&mut self) {
        self.staged.clear();
    }
    fn request(&mut self, request: jai_vm::CompilerRequest) -> jai_vm::EffectOutcome {
        match &request {
            jai_vm::CompilerRequest::CreateWorkspace { name } if name == "phase-fixture" => {
                self.staged.push(request);
                jai_vm::EffectOutcome::Ready(jai_vm::CompilerResponse::Workspace(
                    jai_vm::WorkspaceId::from_raw(42).unwrap(),
                ))
            }
            _ => jai_vm::EffectOutcome::Rejected("unexpected fixture request".into()),
        }
    }
    fn finish(&mut self, commit: bool) -> Result<(), jai_vm::Error> {
        if commit {
            self.committed.append(&mut self.staged);
        }
        self.staged.clear();
        Ok(())
    }
}

fn compiler_program(source: &str, effects: &mut WorkspaceEffects) -> jai_ir::Program {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let directory = std::env::temp_dir().join(format!(
        "jai-compiler-reachability-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&directory).unwrap();
    let path = directory.join("main.jai");
    fs::write(&path, source).unwrap();
    let graph =
        jai_modules::ModuleGraph::load(&path, jai_modules::GraphOptions::default()).unwrap();
    let options = jai_sema::ResolveOptions {
        compiler: Some(jai_sema::CompilerBindingContext::from_graph(
            &graph,
            &[],
            jai_vm::WorkspaceId::from_raw(1).unwrap(),
        )),
        ..Default::default()
    };
    let result = jai_sema::resolve_graph_with_options(&graph, &options, effects);
    fs::remove_dir_all(directory).unwrap();
    result.unwrap()
}

#[test]
fn compiler_effect_runs_in_rust_and_its_helpers_are_omitted_from_native_code() {
    let mut effects = WorkspaceEffects::default();
    let program = compiler_program(
        "compiler_create_workspace :: (name:string)->s64 #compiler;\n\
         recipe :: ()->int { return compiler_create_workspace(\"phase-fixture\"); }\n\
         answer :: #run recipe();\n\
         main :: ()->int { return answer; }",
        &mut effects,
    );
    assert_eq!(
        effects.committed,
        [jai_vm::CompilerRequest::CreateWorkspace {
            name: "phase-fixture".into()
        }]
    );
    let target = jai_codegen::target::NativeTarget::new().unwrap();
    let context = jai_codegen::Context::create();
    let module = jai_codegen::lower_for_target(&context, &program, &target).unwrap();
    let jai_ir::EntryPoint::Int(entry) = program.entry() else {
        panic!("integer entry")
    };
    for procedure in program.procedures() {
        assert_eq!(
            module
                .get_function(&format!("jai.p{}", procedure.id.index()))
                .is_some(),
            procedure.id == entry,
        );
    }
    assert!(!program.library().prototypes().is_empty());
    for prototype in program.library().prototypes() {
        assert!(
            module
                .get_function(&format!("jai.p{}", prototype.id.index()))
                .is_none()
        );
    }
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let directory = std::env::temp_dir().join(format!(
        "jai-compiler-native-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&directory).unwrap();
    let object = directory.join("program.o");
    let executable = directory.join("program");
    target.write_object(&module, &object).unwrap();
    let result = native_tools::clang_command()
        .arg(&object)
        .arg("-o")
        .arg(&executable)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(Command::new(&executable).status().unwrap().code(), Some(42));
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn runtime_compiler_request_reports_the_typed_call_chain() {
    let mut effects = WorkspaceEffects::default();
    let program = compiler_program(
        "compiler_create_workspace :: (name:string)->s64 #compiler;\n\
         recipe :: ()->int { return compiler_create_workspace(\"phase-fixture\"); }\n\
         main :: ()->int { return recipe(); }",
        &mut effects,
    );
    assert!(effects.committed.is_empty());
    let context = jai_codegen::Context::create();
    let error = jai_codegen::lower(&context, &program).unwrap_err();
    let jai_codegen::Error::Reachability(
        jai_codegen::native_reachability::Error::CompilerRequest { chain },
    ) = error
    else {
        panic!("expected compiler-request reachability diagnostic");
    };
    let jai_ir::EntryPoint::Int(entry) = program.entry() else {
        panic!("integer entry")
    };
    assert_eq!(chain.len(), 3);
    assert_eq!(chain[0], entry);
    assert_eq!(chain[2], program.library().prototypes()[0].id);
}
