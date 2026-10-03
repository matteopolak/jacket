//! Context execution uses only our checked IR and freshly compiled executables.
#[path = "support/native_tools.rs"]
mod native_tools;
use jai_ir::*;
use jai_modules::{GraphOptions, ModuleGraph};
use jai_types::*;
use std::{
    fs,
    io::Write,
    process::{Command, Stdio},
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};

struct Fixture(std::path::PathBuf);
impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let fixture = Self(std::env::temp_dir().join(format!(
            "jai-context-native-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        )));
        fs::create_dir_all(&fixture.0).unwrap();
        fixture
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn check_program(program: &Program, fixture: &Fixture, expected: i32) -> String {
    let outcome = jai_vm::execute(program, jai_vm::Limits::default()).outcome;
    let jai_vm::Outcome::Complete(values) = outcome else {
        panic!("{outcome:?}")
    };
    assert_eq!(values[0].integer().unwrap().value(), i128::from(expected));
    let llvm = jai_codegen::emit(program).unwrap();
    let executable = fixture.0.join("program");
    let mut compiler = native_tools::clang_command()
        .args(["-x", "ir", "-", "-o"])
        .arg(&executable)
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    compiler
        .stdin
        .take()
        .unwrap()
        .write_all(llvm.as_bytes())
        .unwrap();
    let compiled = compiler.wait_with_output().unwrap();
    assert!(
        compiled.status.success(),
        "{}\n{llvm}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let mut child = Command::new(executable).spawn().unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert_eq!(status.code(), Some(expected), "{llvm}");
            break;
        }
        if Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("generated context fixture exceeded deadline");
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    llvm
}

fn check(source: &str, expected: i32) -> String {
    let fixture = Fixture::new();
    let path = fixture.0.join("main.jai");
    fs::write(&path, source).unwrap();
    let graph = ModuleGraph::load(&path, GraphOptions::default()).unwrap();
    let program = jai_sema::resolve_graph(&graph).unwrap();
    check_program(&program, &fixture, expected)
}

#[test]
fn direct_recursive_and_indirect_calls_share_mutable_context() {
    let llvm = check(
        "#add_context number: int = 1;
         bump :: (n:int) { context.number += n; }
         descend :: (n:int) { if n > 0 { bump(1); descend(n-1); } }
         main :: ()->int { f := bump; f(35); descend(6); return context.number; }",
        42,
    );
    assert!(llvm.contains("context.default = alloca"));
    assert!(llvm.contains("ptr %context"));
}

#[test]
fn source_context_global_uses_ordinary_storage_without_hidden_context() {
    check(
        "Context::struct{value:int=40;} context:Context; main::()->int #no_context{context.value+=2;return context.value;}",
        42,
    );
}

#[test]
fn local_source_context_does_not_alias_global_or_implicit_context() {
    check(
        "#add_context value:int=1; Context::struct{value:int=40;} context:Context; main::()->int{context:Context;context.value+=2;return context.value;}",
        42,
    );
}

#[test]
fn context_snapshot_and_address_projection_have_different_ownership() {
    check(
        "#add_context number: int = 1;
         main :: ()->int { saved := context; p := *context.number;
             p.* = 42; return context.number + saved.number - 1; }",
        42,
    );
}

#[test]
fn packed_context_members_preserve_snapshot_and_field_access_alignment() {
    check(
        "Packed :: struct { tag:u8; number:int; } #no_padding
         #add_context packed: Packed = .{tag=1, number=20};
         bump :: () { context.packed.number += 1; }
         main :: ()->int { saved := context;
             push_context saved { bump(); if context.packed.number != 21 return 0; }
             bump(); if saved.packed.tag != 1 return 0;
             return context.packed.number + saved.packed.number + 1; }",
        42,
    );
}

#[test]
fn push_copies_record_and_nested_push_restores_previous_pointer() {
    check(
        "#add_context number: int = 1;
         bump :: () { context.number += 1; }
         main :: ()->int { saved := context; saved.number = 20;
             push_context saved { bump();
                 push_context { context.number = 99; }
                 if context.number != 21 return 0;
             }
             return context.number + saved.number + 21; }",
        42,
    );
}

#[test]
fn no_context_and_c_procedures_establish_local_defaults() {
    check(
        "#add_context number: int = 1;
         read :: ()->int { return context.number; }
         detached :: ()->int #no_context { push_context { context.number=20; return read(); } }
         callback :: ()->int #c_call { push_context { context.number=21; return read(); } }
         main :: ()->int { f := callback; return detached() + f() + context.number; }",
        42,
    );
}

#[test]
fn continue_and_break_restore_outer_context() {
    check(
        "#add_context number: int = 42;
         main :: ()->int { n := 0;
             while n < 3 { n += 1; push_context { context.number=0;
                 if n == 1 continue; break;
             } }
             return context.number; }",
        42,
    );
}

#[test]
fn deferred_cleanup_uses_its_lexical_context_on_early_return() {
    check(
        "#add_context number: int = 1; trace: int;
         helper :: ()->int {
             defer trace = trace * 10 + context.number;
             push_context { context.number=2;
                 defer trace = trace * 10 + context.number;
                 return context.number;
             }
         }
         main :: ()->int { result := helper(); return trace + result + 19; }",
        42,
    );
}

#[test]
fn deferred_cleanup_on_continue_and_break_retains_each_context() {
    check(
        "#add_context number: int = 4; trace: int;
         main :: ()->int { n := 0;
             while n < 3 { n += 1; defer trace += context.number;
                 push_context { context.number=9; defer trace += context.number;
                     if n == 1 continue; break;
                 }
             }
             return trace + 16;
         }",
        42,
    );
}

#[test]
fn returned_context_field_is_snapshotted_before_defer_mutates_carrier() {
    check(
        "#add_context number: int = 1;
         helper :: ()->int { defer context.number=99; return context.number; }
         main :: ()->int { result := helper();
             if result == 1 && context.number == 99 return 42;
             return 0;
         }",
        42,
    );
}

#[test]
fn checked_ir_context_defaults_and_hidden_arguments_execute_without_sema() {
    let mut types = TypeRegistry::new();
    let int = types.scalar(ScalarType::Int(IntegerType::S64));
    let record = types.reserve_record(RecordKind::Struct);
    types.define_record(record, [int]).unwrap();
    let pointer = types.pointer(record).unwrap();
    let field = types.field(record, 0).unwrap().id;
    let signature = types
        .procedure(ProcedureType {
            parameters: Box::new([]),
            results: Box::new([int]),
            convention: CallingConvention::Jai,
            context: ContextMode::Implicit,
            variadic: Variadic::None,
        })
        .unwrap();
    let mut places = PlaceRegistry::new();
    let root = Place::context(record, &types).unwrap();
    let value = places.field(root, field, &types).unwrap();
    let read_id = ProcedureId::new(0);
    let main_id = ProcedureId::new(1);
    let read = Procedure {
        id: read_id,
        signature,
        parameters: vec![],
        locals: vec![],
        cleanups: vec![],
        body: Block {
            flow: Flow::Terminates,
            statements: vec![Statement::Exit(Exit {
                cleanups: vec![],
                transfer: Transfer::ReturnValues(vec![ValueExpr::Load(value)]),
            })],
        },
    };
    let main = Procedure {
        id: main_id,
        signature,
        parameters: vec![],
        locals: vec![],
        cleanups: vec![],
        body: Block {
            flow: Flow::Terminates,
            statements: vec![Statement::Exit(Exit {
                cleanups: vec![],
                transfer: Transfer::ReturnValues(vec![ValueExpr::Call {
                    ty: int,
                    call: Call::new(read_id, vec![]),
                }]),
            })],
        },
    };
    let default = ConstantValue {
        ty: record,
        kind: ConstantKind::Record(vec![ConstantValue {
            ty: int,
            kind: ConstantKind::Int(Integer::checked(IntegerType::S64, 42).unwrap()),
        }]),
    };
    let program = ProgramBuilder::new(types.freeze().unwrap())
        .context(ContextDefinition {
            record_type: record,
            pointer_type: pointer,
            default,
        })
        .places(places.freeze())
        .procedures(vec![read, main])
        .finish(EntryPoint::Int(main_id))
        .unwrap();
    check_program(&program, &Fixture::new(), 42);
}

#[test]
fn checked_ir_push_and_captured_cleanup_execute_without_sema() {
    let number = |n| IntExpr::constant(Integer::checked(IntegerType::S64, n).unwrap());
    let binary = |op, left, right| {
        IntExpr::new(
            IntegerType::S64,
            IntExprKind::Binary(op, Box::new(left), Box::new(right)),
        )
    };
    let mut types = TypeRegistry::new();
    let int = types.scalar(ScalarType::Int(IntegerType::S64));
    let record = types.reserve_record(RecordKind::Struct);
    types.define_record(record, [int]).unwrap();
    let pointer = types.pointer(record).unwrap();
    let field = types.field(record, 0).unwrap().id;
    let signature = types
        .procedure(ProcedureType {
            parameters: Box::new([]),
            results: Box::new([int]),
            convention: CallingConvention::Jai,
            context: ContextMode::Implicit,
            variadic: Variadic::None,
        })
        .unwrap();
    let helper_id = ProcedureId::new(0);
    let main_id = ProcedureId::new(1);
    let push = PushContextId::new(helper_id, 0);
    let mut places = PlaceRegistry::new();
    let field = places
        .field(Place::context(record, &types).unwrap(), field, &types)
        .unwrap();
    let field = IntPlace::try_from_place(field, &types).unwrap();
    let trace = Global::new(
        0,
        GlobalInitializer::Int(Integer::checked(IntegerType::S64, 0).unwrap()),
        &types,
    );
    let trace_place = IntPlace::try_from_place(trace.place(), &types).unwrap();
    let result = Local::new_typed(main_id, 0, int, &types).unwrap();
    let result_place = IntPlace::try_from_place(result.place(), &types).unwrap();
    let cleanup = |context| Cleanup {
        context,
        body: Block {
            flow: Flow::FallsThrough,
            statements: vec![Statement::StoreInt(
                trace_place,
                binary(
                    IntOp::Add,
                    binary(IntOp::Multiply, IntExpr::load(trace_place), number(10)),
                    IntExpr::load(field),
                ),
            )],
        },
    };
    let helper = Procedure {
        id: helper_id,
        signature,
        parameters: vec![],
        locals: vec![],
        cleanups: vec![
            cleanup(CleanupContext::Procedure),
            cleanup(CleanupContext::Push(push)),
        ],
        body: Block {
            flow: Flow::Terminates,
            statements: vec![Statement::PushContext {
                id: push,
                value: ValueExpr::Record {
                    ty: record,
                    fields: vec![ValueExpr::Int(number(3))],
                },
                body: Block {
                    flow: Flow::Terminates,
                    statements: vec![Statement::Exit(Exit {
                        cleanups: vec![CleanupId::new(1), CleanupId::new(0)],
                        transfer: Transfer::ReturnInt(IntExpr::load(field)),
                    })],
                },
            }],
        },
    };
    let main = Procedure {
        id: main_id,
        signature,
        parameters: vec![],
        locals: vec![result],
        cleanups: vec![],
        body: Block {
            flow: Flow::Terminates,
            statements: vec![
                Statement::Store(
                    result.place(),
                    ValueExpr::Call {
                        ty: int,
                        call: Call::new(helper_id, vec![]),
                    },
                ),
                Statement::Exit(Exit {
                    cleanups: vec![],
                    transfer: Transfer::ReturnInt(binary(
                        IntOp::Add,
                        binary(
                            IntOp::Add,
                            IntExpr::load(trace_place),
                            IntExpr::load(result_place),
                        ),
                        number(8),
                    )),
                }),
            ],
        },
    };
    let program = ProgramBuilder::new(types.freeze().unwrap())
        .context(ContextDefinition {
            record_type: record,
            pointer_type: pointer,
            default: ConstantValue {
                ty: record,
                kind: ConstantKind::Record(vec![ConstantValue {
                    ty: int,
                    kind: ConstantKind::Int(Integer::checked(IntegerType::S64, 1).unwrap()),
                }]),
            },
        })
        .globals(vec![trace])
        .places(places.freeze())
        .procedures(vec![helper, main])
        .finish(EntryPoint::Int(main_id))
        .unwrap();
    check_program(&program, &Fixture::new(), 42);
}
