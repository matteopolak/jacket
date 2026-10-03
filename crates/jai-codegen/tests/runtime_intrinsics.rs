//! Native intrinsic execution from checked identities, independent of source binding.
#[path = "support/native_tools.rs"]
mod native_tools;
use jai_ir::*;
use jai_types::*;
use std::{
    fs,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

struct Fixture {
    types: TypeRegistry,
    places: PlaceRegistry,
    prototypes: Vec<ProcedurePrototype>,
    locals: Vec<Local>,
    statements: Vec<Statement>,
    int: TypeId,
}
impl Fixture {
    fn new() -> Self {
        let types = TypeRegistry::new();
        let int = types.scalar(ScalarType::Int(IntegerType::S64));
        Self {
            types,
            int,
            places: PlaceRegistry::new(),
            prototypes: vec![],
            locals: vec![],
            statements: vec![],
        }
    }
    fn local(&mut self, ty: TypeId, value: ValueExpr) -> Local {
        let local =
            Local::new_typed(ProcedureId::new(0), self.locals.len(), ty, &self.types).unwrap();
        self.locals.push(local);
        self.statements.push(Statement::Store(local.place(), value));
        local
    }
    fn signature(&mut self, parameters: Vec<TypeId>, results: Vec<TypeId>) -> TypeId {
        self.types
            .procedure(ProcedureType {
                parameters: parameters.into_boxed_slice(),
                results: results.into_boxed_slice(),
                return_abi: jai_types::ForeignReturnAbi::Natural,
                convention: CallingConvention::Jai,
                context: ContextMode::None,
                variadic: Variadic::None,
            })
            .unwrap()
    }
    fn intrinsic(
        &mut self,
        operation: RuntimeIntrinsic,
        parameters: Vec<TypeId>,
        results: Vec<TypeId>,
    ) -> (ProcedureId, TypeId) {
        let signature = self.signature(parameters, results);
        let id = ProcedureId::new(self.prototypes.len() + 1);
        self.prototypes.push(ProcedurePrototype {
            id,
            signature,
            origin: PrototypeOrigin::Intrinsic(operation),
        });
        (id, signature)
    }
    fn address(&mut self, place: Place) -> ValueExpr {
        let ty = self.types.pointer(place.ty()).unwrap();
        ValueExpr::AddressOf {
            place,
            ty,
        }
    }
    fn void_address(&mut self, place: Place) -> ValueExpr {
        let ty = self.types.pointer(self.types.void()).unwrap();
        ValueExpr::PointerCast {
            value: Box::new(self.address(place)),
            ty,
            mode: CastMode::Checked,
        }
    }
    fn call(
        &mut self,
        id: ProcedureId,
        signature: TypeId,
        indirect: bool,
        values: Vec<ValueExpr>,
        destinations: Vec<Option<Place>>,
    ) {
        let arguments = values
            .into_iter()
            .enumerate()
            .map(|(index, value)| (ParameterId::new(index), value))
            .collect();
        self.statements.push(if indirect {
            Statement::IndirectCallResults {
                inline_hint: jai_types::InlineHint::Automatic,
                callee: Box::new(ValueExpr::ProcedureValue {
                    procedure: id,
                    ty: signature,
                }),
                arguments,
                destinations,
            }
        } else {
            Statement::CallResults {
                call: Call::new(id, arguments),
                destinations,
            }
        });
    }
    fn finish(mut self, value: ValueExpr) -> Program {
        let signature = self.signature(vec![], vec![self.int]);
        self.statements.push(Statement::Exit(Exit {
            cleanups: vec![],
            transfer: Transfer::ReturnValues(vec![value]),
        }));
        ProgramBuilder::new(self.types.freeze().unwrap())
            .places(self.places.freeze())
            .prototypes(self.prototypes)
            .procedures(vec![Procedure {
                id: ProcedureId::new(0),
                signature,
                parameters: vec![],
                locals: self.locals,
                body: Block {
                    flow: Flow::Terminates,
                    statements: self.statements,
                },
                cleanups: vec![],
            }])
            .finish(EntryPoint::Int(ProcedureId::new(0)))
            .unwrap()
    }
}
fn integer(ty: IntegerType, n: i128) -> ValueExpr {
    ValueExpr::Int(IntExpr::constant(Integer::checked(ty, n).unwrap()))
}
fn int(n: i128) -> ValueExpr {
    integer(IntegerType::S64, n)
}
fn load_int(place: Place, types: &dyn TypeView) -> IntExpr {
    IntExpr::load(IntPlace::try_from_place(place, types).unwrap())
}
fn equals(place: Place, n: i128, types: &dyn TypeView) -> BoolExpr {
    let value = load_int(place, types);
    BoolExpr::CompareInts(
        Relation::Equal,
        Box::new(value.clone()),
        Box::new(IntExpr::constant(Integer::checked(value.ty(), n).unwrap())),
    )
}
fn check(condition: BoolExpr) -> ValueExpr {
    ValueExpr::Int(IntExpr::new(
        IntegerType::S64,
        IntExprKind::Conditional(Box::new(Conditional {
            condition,
            then_value: match int(42) {
                ValueExpr::Int(value) => value,
                _ => unreachable!(),
            },
            else_value: match int(0) {
                ValueExpr::Int(value) => value,
                _ => unreachable!(),
            },
        })),
    ))
}
fn status(program: &Program) -> std::process::ExitStatus {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    struct Scratch(std::path::PathBuf);
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let scratch = Scratch(std::env::temp_dir().join(format!(
        "jai-native-intrinsic-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )));
    fs::create_dir(&scratch.0).unwrap();
    let target = jai_codegen::target::NativeTarget::new().unwrap();
    let context = jai_codegen::Context::create();
    let module = jai_codegen::lower_for_target(&context, program, &target).unwrap();
    let object = scratch.0.join("program.o");
    let executable = scratch.0.join("program");
    target.write_object(&module, &object).unwrap();
    let output = native_tools::clang_command()
        .arg(&object)
        .arg("-o")
        .arg(&executable)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stderr),
        module.print_to_string()
    );
    run_bounded(&executable)
}

fn run_bounded(executable: &std::path::Path) -> std::process::ExitStatus {
    let mut child = Command::new(executable).spawn().unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            return status;
        }
        if std::time::Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("native intrinsic fixture timed out");
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

#[test]
fn memory_wrappers_preserve_pointer_bytes_and_direct_indirect_calls() {
    let mut f = Fixture::new();
    let pointer = f.types.pointer(f.int).unwrap();
    let record = f.types.reserve_record(RecordKind::Struct);
    f.types.define_record(record, [f.int, pointer]).unwrap();
    let value = f.local(f.int, int(42));
    let address = f.address(value.place());
    let source = f.local(
        record,
        ValueExpr::Record {
            ty: record,
            fields: vec![int(42), address],
        },
    );
    let destination = f.local(record, ValueExpr::Zero(record));
    let void_pointer = f.types.pointer(f.types.void()).unwrap();
    let s16 = f.types.scalar(ScalarType::Int(IntegerType::S16));
    let u8 = f.types.scalar(ScalarType::Int(IntegerType::U8));
    let compare = f.local(s16, integer(IntegerType::S16, 0));
    let (copy, copy_signature) = f.intrinsic(
        RuntimeIntrinsic::MemoryCopy,
        vec![void_pointer, void_pointer, f.int],
        vec![],
    );
    let (cmp, cmp_signature) = f.intrinsic(
        RuntimeIntrinsic::MemoryCompare,
        vec![void_pointer, void_pointer, f.int],
        vec![s16],
    );
    let (set, set_signature) = f.intrinsic(
        RuntimeIntrinsic::MemorySet,
        vec![void_pointer, u8, f.int],
        vec![],
    );
    let target = jai_codegen::target::NativeTarget::new().unwrap();
    let count = LayoutEngine::new(&f.types, target.layout_policy().unwrap())
        .layout(record)
        .unwrap()
        .size as i128;
    let destination_address = f.void_address(destination.place());
    let source_address = f.void_address(source.place());
    f.call(
        copy,
        copy_signature,
        true,
        vec![
            destination_address.clone(),
            source_address.clone(),
            int(count),
        ],
        vec![],
    );
    f.call(
        cmp,
        cmp_signature,
        false,
        vec![
            source_address.clone(),
            destination_address.clone(),
            int(count),
        ],
        vec![Some(compare.place())],
    );
    let bool_ty = f.types.scalar(ScalarType::Bool);
    let first_equal = f.local(
        bool_ty,
        ValueExpr::Bool(equals(compare.place(), 0, &f.types)),
    );
    f.call(
        set,
        set_signature,
        false,
        vec![
            destination_address.clone(),
            integer(IntegerType::U8, 0),
            int(8),
        ],
        vec![],
    );
    f.call(
        cmp,
        cmp_signature,
        true,
        vec![source_address, destination_address, int(count)],
        vec![Some(compare.place())],
    );
    let positive = equals(compare.place(), 1, &f.types);
    let field = f.types.field(record, 1).unwrap().id;
    let copied_pointer = f
        .places
        .field(destination.place(), field, &f.types)
        .unwrap();
    let copied_value = f
        .places
        .dereference(ValueExpr::Load(copied_pointer), &f.types)
        .unwrap();
    let pointed = equals(copied_value, 42, &f.types);
    let result = check(BoolExpr::And(
        Box::new(BoolExpr::Load(
            BoolPlace::try_from_place(first_equal.place(), &f.types).unwrap(),
        )),
        Box::new(BoolExpr::And(Box::new(positive), Box::new(pointed))),
    ));
    assert_eq!(status(&f.finish(result)).code(), Some(42));
}

#[test]
fn compare_and_swap_returns_success_and_observed_scalar_values() {
    for boolean in [false, true] {
        let mut f = Fixture::new();
        let bool_ty = f.types.scalar(ScalarType::Bool);
        let value_ty = if boolean {
            bool_ty
        } else {
            f.int
        };
        let initial = if boolean {
            ValueExpr::Bool(BoolExpr::Constant(false))
        } else {
            int(17)
        };
        let value = f.local(value_ty, initial.clone());
        let success = f.local(bool_ty, ValueExpr::Bool(BoolExpr::Constant(false)));
        let observed = f.local(value_ty, initial.clone());
        let pointer_ty = f.types.pointer(value_ty).unwrap();
        let (id, signature) = f.intrinsic(
            RuntimeIntrinsic::CompareAndSwap {
                value: value_ty,
            },
            vec![pointer_ty, value_ty, value_ty],
            vec![bool_ty, value_ty],
        );
        if boolean {
            let u8_ty = f.types.scalar(ScalarType::Int(IntegerType::U8));
            let byte_pointer = f.types.pointer(u8_ty).unwrap();
            let address = f.address(value.place());
            let byte = f
                .places
                .dereference(
                    ValueExpr::PointerCast {
                        value: Box::new(address),
                        ty: byte_pointer,
                        mode: CastMode::Checked,
                    },
                    &f.types,
                )
                .unwrap();
            f.statements
                .push(Statement::Store(byte, integer(IntegerType::U8, 2)));
        }
        let address = f.address(value.place());
        let replacement = if boolean {
            ValueExpr::Bool(BoolExpr::Constant(true))
        } else {
            int(42)
        };
        f.call(
            id,
            signature,
            false,
            vec![address.clone(), initial.clone(), replacement.clone()],
            vec![Some(success.place()), Some(observed.place())],
        );
        let succeeded =
            BoolExpr::Load(BoolPlace::try_from_place(success.place(), &f.types).unwrap());
        let old_correct = if boolean {
            BoolExpr::Not(Box::new(BoolExpr::Load(
                BoolPlace::try_from_place(observed.place(), &f.types).unwrap(),
            )))
        } else {
            equals(observed.place(), 17, &f.types)
        };
        let snapshot = f.local(
            bool_ty,
            ValueExpr::Bool(BoolExpr::And(Box::new(succeeded), Box::new(old_correct))),
        );
        f.call(
            id,
            signature,
            true,
            vec![address, initial, replacement],
            vec![Some(success.place()), Some(observed.place())],
        );
        let failed = BoolExpr::Not(Box::new(BoolExpr::Load(
            BoolPlace::try_from_place(success.place(), &f.types).unwrap(),
        )));
        let current_correct = if boolean {
            BoolExpr::Load(BoolPlace::try_from_place(observed.place(), &f.types).unwrap())
        } else {
            equals(observed.place(), 42, &f.types)
        };
        let result = check(BoolExpr::And(
            Box::new(BoolExpr::Load(
                BoolPlace::try_from_place(snapshot.place(), &f.types).unwrap(),
            )),
            Box::new(BoolExpr::And(Box::new(failed), Box::new(current_correct))),
        ));
        assert_eq!(status(&f.finish(result)).code(), Some(42));
    }
}

#[test]
fn atomic_integer_widths_nominal_enums_variants_and_pointer_values_use_real_storage() {
    let widths = [
        IntegerType::S8,
        IntegerType::U8,
        IntegerType::S16,
        IntegerType::U16,
        IntegerType::S32,
        IntegerType::U32,
        IntegerType::S64,
        IntegerType::U64,
    ];
    for kind in 0..11 {
        let mut f = Fixture::new();
        let boolean = f.types.scalar(ScalarType::Bool);
        let (value_ty, initial, replacement) = if kind < widths.len() {
            let ty = f.types.scalar(ScalarType::Int(widths[kind]));
            (ty, integer(widths[kind], 17), integer(widths[kind], 42))
        } else if kind == 8 {
            let ty = f.types.reserve_enum(IntegerType::U16);
            let old = Integer::checked(IntegerType::U16, 17).unwrap();
            let new = Integer::checked(IntegerType::U16, 42).unwrap();
            f.types.define_enum(ty, [old, new]).unwrap();
            (
                ty,
                ValueExpr::Enum {
                    ty,
                    value: old,
                },
                ValueExpr::Enum {
                    ty,
                    value: new,
                },
            )
        } else if kind == 9 {
            let ty = f.types.reserve_distinct(DistinctKind::Distinct);
            f.types.define_distinct(ty, f.int).unwrap();
            (
                ty,
                ValueExpr::Distinct {
                    ty,
                    value: Box::new(int(17)),
                },
                ValueExpr::Distinct {
                    ty,
                    value: Box::new(int(42)),
                },
            )
        } else {
            let target = f.local(f.int, int(42));
            let pointer = f.types.pointer(f.int).unwrap();
            (pointer, ValueExpr::Zero(pointer), f.address(target.place()))
        };
        let value = f.local(value_ty, initial.clone());
        let observed = f.local(value_ty, initial.clone());
        let success = f.local(boolean, ValueExpr::Bool(BoolExpr::Constant(false)));
        let pointer = f.types.pointer(value_ty).unwrap();
        let (id, signature) = f.intrinsic(
            RuntimeIntrinsic::CompareAndSwap {
                value: value_ty,
            },
            vec![pointer, value_ty, value_ty],
            vec![boolean, value_ty],
        );
        let address = f.address(value.place());
        f.call(
            id,
            signature,
            true,
            vec![address, initial.clone(), replacement.clone()],
            vec![Some(success.place()), Some(observed.place())],
        );
        let numeric = |place| {
            if kind < widths.len() {
                load_int(place, &f.types)
            } else if kind == 8 {
                IntExpr::new(
                    IntegerType::U16,
                    IntExprKind::EnumValue(Box::new(ValueExpr::Load(place))),
                )
            } else {
                IntExpr::new(
                    IntegerType::S64,
                    IntExprKind::Value(Box::new(ValueExpr::UnwrapDistinct {
                        value: Box::new(ValueExpr::Load(place)),
                        ty: f.int,
                    })),
                )
            }
        };
        let (old, current) = if kind == 10 {
            (
                BoolExpr::ComparePointers(
                    Equality::Equal,
                    Box::new(ValueExpr::Load(observed.place())),
                    Box::new(initial),
                ),
                BoolExpr::ComparePointers(
                    Equality::Equal,
                    Box::new(ValueExpr::Load(value.place())),
                    Box::new(replacement),
                ),
            )
        } else {
            let old = numeric(observed.place());
            let current = numeric(value.place());
            (
                BoolExpr::CompareInts(
                    Relation::Equal,
                    Box::new(old.clone()),
                    Box::new(IntExpr::constant(Integer::checked(old.ty(), 17).unwrap())),
                ),
                BoolExpr::CompareInts(
                    Relation::Equal,
                    Box::new(current.clone()),
                    Box::new(IntExpr::constant(
                        Integer::checked(current.ty(), 42).unwrap(),
                    )),
                ),
            )
        };
        let succeeded =
            BoolExpr::Load(BoolPlace::try_from_place(success.place(), &f.types).unwrap());
        let result = check(BoolExpr::And(
            Box::new(succeeded),
            Box::new(BoolExpr::And(Box::new(old), Box::new(current))),
        ));
        assert_eq!(
            status(&f.finish(result)).code(),
            Some(42),
            "atomic domain fixture {kind}"
        );
    }
}

#[test]
fn zero_count_accepts_null_while_negative_null_nonzero_overlap_and_debugtrap_fail() {
    for invalid in 0..5 {
        let mut f = Fixture::new();
        let void_pointer = f.types.pointer(f.types.void()).unwrap();
        let (id, signature) = f.intrinsic(
            RuntimeIntrinsic::MemoryCopy,
            vec![void_pointer, void_pointer, f.int],
            vec![],
        );
        let storage = f.local(f.int, int(42));
        let address = f.void_address(storage.place());
        let count = match invalid {
            0 => 0,
            1 => -1,
            _ => 1,
        };
        let left = if invalid == 3 {
            address.clone()
        } else {
            ValueExpr::Zero(void_pointer)
        };
        let right = if invalid == 3 {
            address
        } else {
            ValueExpr::Zero(void_pointer)
        };
        if invalid == 4 {
            let (id, signature) = f.intrinsic(RuntimeIntrinsic::DebugTrap, vec![], vec![]);
            f.call(id, signature, false, vec![], vec![]);
        } else {
            f.call(id, signature, false, vec![left, right, int(count)], vec![]);
        }
        let exit = status(&f.finish(int(42)));
        if invalid == 0 {
            assert_eq!(exit.code(), Some(42));
        } else {
            assert!(!exit.success());
        }
    }
}

#[test]
fn debugtrap_wrapper_returns_to_the_checked_continuation_after_debugger_resume() {
    let mut fixture = Fixture::new();
    let (id, signature) = fixture.intrinsic(RuntimeIntrinsic::DebugTrap, vec![], vec![]);
    fixture.call(id, signature, false, vec![], vec![]);
    let program = fixture.finish(int(42));
    let context = jai_codegen::Context::create();
    let module = jai_codegen::lower(&context, &program).unwrap();
    module.verify().unwrap();
    let trap = module
        .get_function(&format!("jai.p{}", id.index()))
        .unwrap();
    let block = trap.get_first_basic_block().unwrap();
    assert_eq!(
        block.get_terminator().unwrap().get_opcode(),
        inkwell::values::InstructionOpcode::Return
    );
    let call = block.get_first_instruction().unwrap();
    assert_eq!(call.get_opcode(), inkwell::values::InstructionOpcode::Call);
    assert_eq!(
        call.get_next_instruction().unwrap(),
        block.get_terminator().unwrap()
    );
    let main = module.get_function("jai.p0").unwrap();
    assert_eq!(
        main.get_last_basic_block()
            .unwrap()
            .get_terminator()
            .unwrap()
            .get_opcode(),
        inkwell::values::InstructionOpcode::Return
    );
}

#[test]
fn self_written_c_threads_observe_native_sequentially_consistent_cas() {
    let mut f = Fixture::new();
    let boolean = f.types.scalar(ScalarType::Bool);
    let pointer = f.types.pointer(f.int).unwrap();
    let (intrinsic, intrinsic_signature) = f.intrinsic(
        RuntimeIntrinsic::CompareAndSwap {
            value: f.int,
        },
        vec![pointer, f.int, f.int],
        vec![boolean, f.int],
    );
    let parameters: Vec<_> = [pointer, f.int, f.int]
        .into_iter()
        .map(|ty| {
            let local =
                Local::new_typed(ProcedureId::new(0), f.locals.len(), ty, &f.types).unwrap();
            f.locals.push(local);
            local
        })
        .collect();
    let success = Local::new_typed(ProcedureId::new(0), f.locals.len(), boolean, &f.types).unwrap();
    f.locals.push(success);
    let observed = Local::new_typed(ProcedureId::new(0), f.locals.len(), f.int, &f.types).unwrap();
    f.locals.push(observed);
    f.call(
        intrinsic,
        intrinsic_signature,
        false,
        parameters
            .iter()
            .map(|parameter| ValueExpr::Load(parameter.place()))
            .collect(),
        vec![Some(success.place()), Some(observed.place())],
    );
    let signature = f
        .types
        .procedure(ProcedureType {
            parameters: vec![pointer, f.int, f.int].into_boxed_slice(),
            results: vec![boolean].into_boxed_slice(),
            return_abi: jai_types::ForeignReturnAbi::Natural,
            convention: CallingConvention::C,
            context: ContextMode::None,
            variadic: Variadic::None,
        })
        .unwrap();
    f.statements.push(Statement::Exit(Exit {
        cleanups: vec![],
        transfer: Transfer::ReturnValues(vec![ValueExpr::Load(success.place())]),
    }));
    let library = ProgramBuilder::new(f.types.freeze().unwrap())
        .prototypes(f.prototypes)
        .procedures(vec![Procedure {
            id: ProcedureId::new(0),
            signature,
            parameters,
            locals: f.locals,
            cleanups: vec![],
            body: Block {
                flow: Flow::Terminates,
                statements: f.statements,
            },
        }])
        .finish_library()
        .unwrap();
    let target = jai_codegen::target::NativeTarget::new().unwrap();
    let context = jai_codegen::Context::create();
    let module = jai_codegen::lower_library_for_target(
        &context,
        &library,
        &jai_codegen::native_reachability::Publication::Selected(vec![ProcedureId::new(0)]),
        &target,
    )
    .unwrap();
    module
        .get_function("jai.p0")
        .unwrap()
        .as_global_value()
        .set_name("attempt");
    let directory = std::env::temp_dir().join(format!("jai-cas-threads-{}", std::process::id()));
    fs::create_dir(&directory).unwrap();
    let object = directory.join("atomic.o");
    let source = directory.join("threads.c");
    let executable = directory.join("threads");
    target.write_object(&module, &object).unwrap();
    fs::write(
        &source,
        r#"
#include <stdbool.h>
#include <stdint.h>
#include <stdatomic.h>
#include <pthread.h>
extern bool attempt(int64_t *, int64_t, int64_t);
static _Atomic(int64_t) count;
static void *worker(void *unused) {
    (void)unused;
    for (int i = 0; i < 2000; ++i) {
        int64_t expected = atomic_load_explicit(&count, memory_order_seq_cst);
        while (!attempt((int64_t *)&count, expected, expected + 1))
            expected = atomic_load_explicit(&count, memory_order_seq_cst);
    }
    return 0;
}
int main(void) {
    pthread_t threads[4];
    for (int i = 0; i < 4; ++i) if (pthread_create(&threads[i], 0, worker, 0)) return 1;
    for (int i = 0; i < 4; ++i) if (pthread_join(threads[i], 0)) return 2;
    return atomic_load_explicit(&count, memory_order_seq_cst) == 8000 ? 42 : 3;
}
"#,
    )
    .unwrap();
    let output = native_tools::clang_command()
        .arg(&object)
        .arg(&source)
        .arg("-pthread")
        .arg("-o")
        .arg(&executable)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(run_bounded(&executable).code(), Some(42));
    fs::remove_dir_all(directory).unwrap();
}
