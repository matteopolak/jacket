//! Independently authored event schemas exercise source binding and actual VM storage.
use super::*;

const SOURCE: &str = r#"
Intercept_Flags :: enum_flags u32 {
    SKIP_EXPRESSIONS_WITHOUT_NOTES :: 1; SKIP_DECLARATIONS :: 2;
    SKIP_PROCEDURE_HEADERS :: 4; SKIP_PROCEDURE_BODIES :: 8;
    SKIP_STRUCTS :: 16; SKIP_OTHERS :: 32; SKIP_ALL :: 63;
    DO_PERFORMANCE_REPORT_POLYMORPHS :: 0x1000;
    DO_PERFORMANCE_REPORT_RUNS :: 0x2000;
}
Message :: struct {
    kind: enum u8 { UNINITIALIZED; FILE; IMPORT; FAILED_IMPORT; PHASE;
        TYPECHECKED; COMPLETE; DEBUG_DUMP; ERROR; PERFORMANCE_REPORT; };
    workspace: s64;
}
Message_Phase :: struct {
    #as using m: Message;
    phase: enum u32 { ALL_SOURCE_CODE_PARSED; TYPECHECKED_ALL_WE_CAN;
        ALL_TARGET_CODE_BUILT; PRE_WRITE_EXECUTABLE; POST_WRITE_EXECUTABLE;
        READY_FOR_CUSTOM_LINK_COMMAND; };
    executable_name: string;
    executable_write_failed: bool;
    linker_exit_code: s32;
    num_items_waiting_to_typecheck: s32;
    compiler_generated_object_files: []string;
    support_object_files: []string;
    system_libraries: []string;
    user_libraries: []string;
}
Message_Complete :: struct {
    #as using m: Message;
    error_code: enum u8 { NONE; COMPILATION_FAILED; COMPILER_SHUTDOWN; };
}
compiler_begin_intercept :: (w: s64, flags: Intercept_Flags = 0) #compiler;
compiler_end_intercept :: (w: s64) #compiler;
compiler_wait_for_message :: () -> *Message #compiler;
"#;

fn options(graph: &ModuleGraph) -> crate::ResolveOptions {
    crate::ResolveOptions {
        compiler: Some(CompilerBindingContext::from_graph(graph, &[], workspace())),
        ..crate::ResolveOptions::default()
    }
}

#[test]
fn complete_source_event_schema_binds_and_incompatible_members_reject() {
    with_graph(SOURCE, |graph| {
        let library =
            crate::resolve_library_with_options(graph, &options(graph), &mut jai_vm::NoEffects)
                .unwrap();
        assert_eq!(library.prototypes().len(), 3);
    });
    for source in [
        SOURCE.replace("SKIP_ALL :: 63", "SKIP_ALL :: 62"),
        SOURCE.replace(
            "num_items_waiting_to_typecheck: s32",
            "num_items_waiting_to_typecheck: s64",
        ),
        SOURCE.replace("#as using m", "using m"),
        SOURCE.replace("COMPILER_SHUTDOWN;", "COMPILER_SHUTDOWN :: 17;"),
    ] {
        with_graph(&source, |graph| {
            let error =
                crate::resolve_library_with_options(graph, &options(graph), &mut jai_vm::NoEffects)
                    .unwrap_err();
            assert!(error.message.contains("#compiler"), "{error:?}");
        });
    }
}

#[derive(Default)]
struct Effects {
    staged: Vec<jai_vm::CompilerRequest>,
    committed: Vec<jai_vm::CompilerRequest>,
}
impl jai_vm::CompilerEffects for Effects {
    fn begin(&mut self) {
        self.staged.clear();
    }
    fn request(&mut self, request: jai_vm::CompilerRequest) -> jai_vm::EffectOutcome {
        let response = match request {
            jai_vm::CompilerRequest::WaitForMessage => {
                jai_vm::CompilerResponse::Message(jai_vm::CompilerEvent::Phase {
                    workspace: workspace(),
                    phase: jai_vm::CompilerPhase::Typechecked { pending_count: 9 },
                })
            }
            _ => jai_vm::CompilerResponse::Unit,
        };
        self.staged.push(request);
        jai_vm::EffectOutcome::Ready(response)
    }
    fn finish(&mut self, commit: bool) -> Result<(), jai_vm::Error> {
        if commit {
            self.committed.append(&mut self.staged);
        }
        self.staged.clear();
        Ok(())
    }
}

#[test]
fn omitted_source_flags_forward_the_exact_zero_default_to_host_policy() {
    let source = format!("{SOURCE}\n#run compiler_begin_intercept(-1);");
    with_graph(&source, |graph| {
        let mut effects = Effects::default();
        crate::resolve_library_with_options(graph, &options(graph), &mut effects).unwrap();
        assert_eq!(
            effects.committed,
            vec![jai_vm::CompilerRequest::BeginIntercept {
                workspace: workspace(),
                flags: jai_vm::InterceptFlags::NONE,
            }]
        );
    });
}

#[test]
fn source_wait_reads_real_phase_record_and_begin_end_preserve_typed_flags() {
    let source = format!(
        "{SOURCE}\nreceive :: () -> s32 {{ m := compiler_wait_for_message(); p := cast(*Message_Phase)m; return p.num_items_waiting_to_typecheck; }}\nanswer :: #run receive();\n#run compiler_begin_intercept(-1, Intercept_Flags.SKIP_ALL);\n#run compiler_end_intercept(-1);\nmain :: () -> s32 {{ return answer; }}"
    );
    with_graph(&source, |graph| {
        let mut effects = Effects::default();
        let library =
            crate::resolve_library_with_options(graph, &options(graph), &mut effects).unwrap();
        let main = graph
            .declarations()
            .iter()
            .find(|declaration| graph.symbols().name(declaration.name()) == "main")
            .unwrap();
        let id = library.procedure(main.id()).unwrap().id;
        let mut vm =
            jai_vm::Vm::new(&library, jai_vm::NoEffects, jai_vm::Limits::default()).unwrap();
        assert_eq!(
            vm.execute(id, vec![]).outcome,
            jai_vm::Outcome::Complete(vec![jai_vm::Value::Int(
                jai_types::Integer::checked(IntegerType::S32, 9).unwrap()
            )])
        );
        assert!(
            effects
                .committed
                .contains(&jai_vm::CompilerRequest::WaitForMessage)
        );
        assert!(
            effects
                .committed
                .contains(&jai_vm::CompilerRequest::BeginIntercept {
                    workspace: workspace(),
                    flags: jai_vm::InterceptFlags::SKIP_ALL
                })
        );
        assert!(
            effects
                .committed
                .contains(&jai_vm::CompilerRequest::EndIntercept {
                    workspace: workspace()
                })
        );
    });
}
