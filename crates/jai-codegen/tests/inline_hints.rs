//! Verify source policy against real LLVM attributes and independently emitted code.
#[path = "support/native_tools.rs"]
mod native_tools;
use inkwell::attributes::{Attribute, AttributeLoc};
use jai_types::InlineHint;
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "jai-inline-hints-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn graph(&self, source: &str) -> jai_modules::ModuleGraph {
        let path = self.0.join("main.jai");
        fs::write(&path, source).unwrap();
        jai_modules::ModuleGraph::load(&path, jai_modules::GraphOptions::default()).unwrap()
    }
    fn check(&self, program: &jai_ir::Program, expected: i32) {
        self.check_policy(program, expected, true);
    }
    fn check_calls(&self, program: &jai_ir::Program, expected: i32) {
        self.check_policy(program, expected, false);
    }
    fn check_policy(&self, program: &jai_ir::Program, expected: i32, declarations: bool) {
        let execution = jai_vm::execute(program, jai_vm::Limits::default());
        assert!(
            matches!(execution.outcome, jai_vm::Outcome::Complete(ref values) if matches!(values.as_slice(), [jai_vm::Value::Int(value)] if value.value() == i128::from(expected))),
            "{execution:?}"
        );
        let context = jai_codegen::Context::create();
        let target = jai_codegen::target::NativeTarget::new().unwrap();
        let module = jai_codegen::lower_for_target(&context, program, &target).unwrap();
        let mut counts = [0; 3];
        for procedure in program.procedures() {
            let Some(function) = module.get_function(&format!("jai.p{}", procedure.id.index()))
            else {
                continue;
            };
            let always = function
                .get_enum_attribute(
                    AttributeLoc::Function,
                    Attribute::get_named_enum_kind_id("alwaysinline"),
                )
                .is_some();
            let never = function
                .get_enum_attribute(
                    AttributeLoc::Function,
                    Attribute::get_named_enum_kind_id("noinline"),
                )
                .is_some();
            let (wanted, index) = match program.library().inline_hint(procedure.id) {
                InlineHint::Automatic => ((false, false), 0),
                InlineHint::Always => ((true, false), 1),
                InlineHint::Never => ((false, true), 2),
            };
            assert_eq!(
                (always, never),
                wanted,
                "policy for {:?}\n{}",
                procedure.id,
                module.print_to_string()
            );
            counts[index] += 1;
        }
        assert!(
            !declarations || counts.iter().all(|count| *count > 0),
            "fixture covers all three source policies: {counts:?}"
        );
        if !declarations {
            let counts = call_policies(&module);
            assert!(
                counts[1] > 0 && counts[2] > 0,
                "fixture must emit both actual LLVM call-site policies: {counts:?}\n{}",
                module.print_to_string()
            );
        }
        let object = self.0.join("program.o");
        target.write_object(&module, &object).unwrap();
        let optimized = module.print_to_string().to_string();
        if !declarations {
            let counts = call_policies(&module);
            assert_eq!(
                counts[1], 0,
                "forced fixture calls must inline at O0: {optimized}"
            );
            assert!(
                counts[2] > 0,
                "no_inline fixture call policy must survive O0: {optimized}"
            );
        }
        for procedure in program.procedures() {
            let called = optimized.lines().any(|line| {
                line.contains("call ") && line.contains(&format!("@jai.p{}(", procedure.id.index()))
            });
            match program.library().inline_hint(procedure.id) {
                InlineHint::Always => assert!(
                    !called,
                    "explicit inline fixture must be substituted by the O0 always-inliner: {optimized}"
                ),
                InlineHint::Never => assert!(
                    called,
                    "explicit no_inline fixture must retain its call: {optimized}"
                ),
                InlineHint::Automatic => {}
            }
        }
        self.run_object(&object, expected);
    }
    fn run_object(&self, object: &std::path::Path, expected: i32) {
        let executable = self.0.join("program");
        let output = native_tools::clang_command()
            .arg(object)
            .arg("-o")
            .arg(&executable)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let mut child = Command::new(executable).spawn().unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(status) = child.try_wait().unwrap() {
                assert_eq!(status.code(), Some(expected));
                break;
            }
            if Instant::now() >= deadline {
                child.kill().unwrap();
                child.wait().unwrap();
                panic!("generated procedure policy fixture timed out");
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}
fn call_policies(module: &inkwell::module::Module<'_>) -> [usize; 3] {
    let mut counts = [0; 3];
    for function in module.get_functions() {
        for block in function.get_basic_blocks() {
            for instruction in block.get_instructions() {
                if let Ok(call) = inkwell::values::CallSiteValue::try_from(instruction) {
                    let always = call
                        .get_enum_attribute(
                            AttributeLoc::Function,
                            Attribute::get_named_enum_kind_id("alwaysinline"),
                        )
                        .is_some();
                    let never = call
                        .get_enum_attribute(
                            AttributeLoc::Function,
                            Attribute::get_named_enum_kind_id("noinline"),
                        )
                        .is_some();
                    assert!(!(always && never));
                    counts[if always {
                        1
                    } else if never {
                        2
                    } else {
                        0
                    }] += 1;
                }
            }
        }
    }
    counts
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn procedure_attributes_do_not_depend_on_debug_source_or_procedure_names() {
    let source = "inline_named_automatic::(x:int)->int{return x;} no_inline_named_forced::inline(x:int)->int{return x+1;} inline_named_blocked::no_inline(x:int)->int{return x+2;} main::()->int{return inline_named_automatic(12)+no_inline_named_forced(13)+inline_named_blocked(14);}";
    let program = jai_sema::resolve(&jai_syntax::parse(source).unwrap()).unwrap();
    assert!(program.library().debug_sources().is_none());
    assert_eq!(
        program.procedures()[0].signature,
        program.procedures()[1].signature
    );
    assert_eq!(
        program.procedures()[1].signature,
        program.procedures()[2].signature
    );
    Fixture::new().check(&program, 42);
}

#[test]
fn generic_specializations_and_nested_definitions_preserve_the_source_policy() {
    let fixture = Fixture::new();
    let graph = fixture.graph("forced::inline(value:$T)->T{return value;} blocked::no_inline(value:$T)->T{return value;} main::()->int{ nested::inline(value:int)->int{return value+2;} return forced(19)+blocked(19)+nested(2); }");
    let program = jai_sema::resolve_graph(&graph).unwrap();
    let hinted: Vec<_> = program
        .procedures()
        .iter()
        .map(|procedure| program.library().inline_hint(procedure.id))
        .filter(|hint| *hint != InlineHint::Automatic)
        .collect();
    assert_eq!(
        hinted
            .iter()
            .filter(|hint| **hint == InlineHint::Always)
            .count(),
        2
    );
    assert_eq!(
        hinted
            .iter()
            .filter(|hint| **hint == InlineHint::Never)
            .count(),
        1
    );
    fixture.check(&program, 42);
}

#[test]
fn direct_call_site_hints_preserve_c_abi_without_changing_the_declaration() {
    let fixture = Fixture::new();
    let graph = fixture.graph("target::(value:int)->int #c_call{return value;} main::()->int{return inline target(20)+no_inline target(22);}");
    let program = jai_sema::resolve_graph(&graph).unwrap();
    assert!(
        program
            .procedures()
            .iter()
            .all(|procedure| program.library().inline_hint(procedure.id) == InlineHint::Automatic)
    );
    fixture.check_calls(&program, 42);
}

#[test]
fn no_inline_call_overrides_inline_definition_at_o0_and_o2() {
    use jai_codegen::{
        optimization::Optimization,
        target::{NativeTarget, TargetOptions},
    };
    use jai_types::BitcodeOptimization;

    // The pinned upstream inlining example demonstrates this direction of override.
    // An exported caller with an unknown input prevents constant folding from
    // obscuring whether the call was retained by the optimizer.
    let fixture = Fixture::new();
    let graph = fixture.graph("target::inline(value:int)->int #c_call{return value+1;} #program_export \"hinted_run\" run::(value:int)->int #c_call{return no_inline target(value);} main::()->int{return run(41);}");
    let program = jai_sema::resolve_graph(&graph).unwrap();
    let execution = jai_vm::execute(&program, jai_vm::Limits::default());
    assert!(
        matches!(execution.outcome, jai_vm::Outcome::Complete(ref values) if matches!(values.as_slice(), [jai_vm::Value::Int(value)] if value.value() == 42)),
        "{execution:?}"
    );
    let target_id = program
        .procedures()
        .iter()
        .find(|procedure| program.library().inline_hint(procedure.id) == InlineHint::Always)
        .unwrap()
        .id;
    let legacy = jai_sema::resolve(&jai_syntax::parse("main::()->int{return no_inline target(41);} target::inline(value:int)->int{return value+1;}").unwrap()).unwrap();
    assert!(
        matches!(jai_vm::execute(&legacy, jai_vm::Limits::default()).outcome, jai_vm::Outcome::Complete(values) if matches!(values.as_slice(), [jai_vm::Value::Int(value)] if value.value() == 42))
    );
    for bitcode in [BitcodeOptimization::O0, BitcodeOptimization::O2] {
        let target = NativeTarget::select(&TargetOptions {
            optimization: Optimization {
                bitcode,
                ..Optimization::default()
            },
            ..TargetOptions::default()
        })
        .unwrap();
        let context = jai_codegen::Context::create();
        let module = jai_codegen::lower_for_target(&context, &program, &target).unwrap();
        assert!(
            module
                .get_function(&format!("jai.p{}", target_id.index()))
                .unwrap()
                .get_enum_attribute(
                    AttributeLoc::Function,
                    Attribute::get_named_enum_kind_id("alwaysinline")
                )
                .is_some()
        );
        assert!(call_policies(&module)[2] > 0);
        let object = fixture.0.join("override.o");
        target.write_object(&module, &object).unwrap();
        let optimized = module.print_to_string().to_string();
        assert!(
            call_policies(&module)[2] > 0,
            "no_inline call-site attribute must survive {bitcode:?}: {optimized}"
        );
        assert!(
            optimized.lines().any(|line| line.contains("call ")
                && line.contains(&format!("@jai.p{}(", target_id.index()))),
            "inline definition must retain its prohibited call at {bitcode:?}: {optimized}"
        );
        fixture.run_object(&object, 42);
    }
}

#[test]
fn call_site_hints_cover_generic_multiple_results_context_overrides_and_dynamic_no_inline() {
    let fixture = Fixture::new();
    let graph = fixture.graph("#add_context number:int=1; pair::(value:$T)->(T,T){return value,value;} read::(value:int)->int{return value+context.number;} main::()->int{a,b:=inline pair(10); callback:=read; value:=no_inline callback(1,,number=2); result:=inline read(1,,number=3); return a+b+value+result+15;}");
    let program = jai_sema::resolve_graph(&graph).unwrap();
    fixture.check_calls(&program, 42);
}

#[test]
fn hinted_result_obligations_and_unsupported_force_inline_targets_remain_precise() {
    for (source, message) in [
        (
            "read::()->int #must{return 1;} main::(){inline read();}",
            "is marked #must and cannot be discarded",
        ),
        (
            "read::()->int{return 1;} main::()->int{callback:=read;return inline callback();}",
            "inline requires a constant procedure target",
        ),
        (
            "external::()->int #foreign; main::()->int{return inline external();}",
            "inline call requires a source procedure body",
        ),
        (
            "main::()->int{return inline target(42);} target::no_inline(value:$T)->T{return value;}",
            "contradictory declaration and call-site inlining policy is unsupported",
        ),
    ] {
        let fixture = Fixture::new();
        let graph = fixture.graph(source);
        let error = jai_sema::resolve_graph(&graph).unwrap_err();
        assert!(error.message.contains(message), "{error:?}");
        assert!(error.location.span.end > error.location.span.start);
    }
    let module = jai_syntax::parse(
        "main::()->int{return inline target();} target::no_inline()->int{return 42;}",
    )
    .unwrap();
    let error = jai_sema::resolve(&module).unwrap_err();
    assert!(
        error
            .message
            .contains("contradictory declaration and call-site inlining policy is unsupported")
    );
}

#[test]
fn hinted_record_result_can_be_projected_without_losing_the_call_policy() {
    let fixture = Fixture::new();
    let graph=fixture.graph("Point::struct{value:int;} make::()->Point{return .{value=20};} read::()->int{return 22;} main::()->int{return inline make().value+no_inline read();}");
    let program = jai_sema::resolve_graph(&graph).unwrap();
    fixture.check_calls(&program, 42);
}

#[test]
fn native_policy_validation_also_checks_programs_built_without_source_binding() {
    use jai_ir::{
        Block, Call, EntryPoint, Exit, Flow, Procedure, ProcedureId, ProgramBuilder, Statement,
        Transfer, ValueExpr,
    };
    use jai_types::{
        CallingConvention, ContextMode, Integer, IntegerType, ProcedureType, ScalarType,
        TypeRegistry, Variadic,
    };
    for (declaration, call_hint, bodyless) in [
        (InlineHint::Never, InlineHint::Always, false),
        (InlineHint::Automatic, InlineHint::Always, true),
    ] {
        let mut types = TypeRegistry::new();
        let int = types.scalar(ScalarType::Int(IntegerType::S64));
        let signature = types
            .procedure(ProcedureType {
                parameters: vec![].into(),
                results: vec![int].into(),
                convention: CallingConvention::C,
                context: ContextMode::None,
                variadic: Variadic::None,
            })
            .unwrap();
        let main = ProcedureId::new(0);
        let target = ProcedureId::new(1);
        let body = |value| Block {
            flow: Flow::Terminates,
            statements: vec![Statement::Exit(Exit {
                cleanups: vec![],
                transfer: Transfer::ReturnValues(vec![value]),
            })],
        };
        let mut procedures = vec![Procedure {
            id: main,
            signature,
            parameters: vec![],
            locals: vec![],
            body: body(ValueExpr::Call {
                call: Call::new(target, vec![]).with_inline_hint(call_hint),
                ty: int,
            }),
            cleanups: vec![],
        }];
        let mut prototypes = vec![];
        if bodyless {
            prototypes.push(jai_ir::ProcedurePrototype {
                id: target,
                signature,
                origin: jai_ir::PrototypeOrigin::Foreign {
                    symbol: "external_target".into(),
                    library: None,
                },
            });
        } else {
            procedures.push(Procedure {
                id: target,
                signature,
                parameters: vec![],
                locals: vec![],
                body: body(ValueExpr::Int(jai_ir::IntExpr::constant(
                    Integer::checked(IntegerType::S64, 42).unwrap(),
                ))),
                cleanups: vec![],
            });
        }
        let hints = if bodyless {
            std::collections::HashMap::new()
        } else {
            std::collections::HashMap::from([(target, declaration)])
        };
        let program = ProgramBuilder::new(types.freeze().unwrap())
            .procedures(procedures)
            .prototypes(prototypes)
            .procedure_hints(hints)
            .finish(EntryPoint::Int(main))
            .unwrap();
        let context = jai_codegen::Context::create();
        let error = jai_codegen::lower(&context, &program).unwrap_err();
        assert!(
            matches!(error, jai_codegen::Error::UnsupportedInlining { procedure, .. } if procedure == target),
            "{error:?}"
        );
        assert!(error.to_string().contains(if bodyless {
            "no available native body"
        } else {
            "precedence evidence"
        }));
    }
}

#[test]
fn hinted_calls_preserve_the_inner_call_location_for_defaults() {
    let fixture = Fixture::new();
    let mut source = "Source_Code_Location::struct{fully_pathed_filename:string;line_number:s64;character_number:s64;}\nprobe::(loc:=#caller_location)->int{return loc.line_number*1000+loc.character_number;}\nmain::()->int{text:=\"é🦀\";callback:=probe;return ifx inline probe()==EXPECTED_A && no_inline callback()==EXPECTED_B then 42 else 1;}".to_owned();
    for (call, placeholder) in [("probe()", "EXPECTED_A"), ("callback()", "EXPECTED_B")] {
        let prefix = &source[..source.rfind(call).unwrap()];
        let line = prefix.bytes().filter(|byte| *byte == b'\n').count() + 1;
        let column = prefix.rsplit('\n').next().unwrap().chars().count() + 1;
        source = source.replace(placeholder, &(line * 1000 + column).to_string());
    }
    let graph = fixture.graph(&source);
    let target = jai_codegen::target::NativeTarget::new().unwrap();
    let program = jai_sema::resolve_graph_with_options(
        &graph,
        &jai_sema::ResolveOptions {
            layout: Some(target.layout_policy().unwrap()),
            ..jai_sema::ResolveOptions::default()
        },
        &mut jai_vm::NoEffects,
    )
    .unwrap();
    fixture.check_calls(&program, 42);
}
