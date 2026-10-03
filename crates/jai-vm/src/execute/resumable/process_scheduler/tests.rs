use super::*;
use crate::process_abi::{
    ProcessAbiNominals, ProcessAbiOperation, ProcessAbiProcedure, ProcessAuthority,
};
use jai_source::Identities;
use jai_types::{
    Architecture, BuildTarget, ByteOrder, CallingConvention, ContextMode, DistinctKind,
    IntegerType, LayoutPolicy, OperatingSystem, ProcedureType, ScalarType, TypeRegistry, Variadic,
};
use std::cell::Cell;

fn main_id() -> ProcedureId {
    ProcedureId::new(0)
}
fn bump_id() -> ProcedureId {
    ProcedureId::new(1)
}
fn message_id() -> ProcedureId {
    ProcedureId::new(2)
}
fn fork_id() -> ProcedureId {
    ProcedureId::new(10)
}
fn pipe_id() -> ProcedureId {
    ProcedureId::new(11)
}
fn close_id() -> ProcedureId {
    ProcedureId::new(12)
}
fn read_id() -> ProcedureId {
    ProcedureId::new(13)
}
fn write_id() -> ProcedureId {
    ProcedureId::new(14)
}
fn exit_id() -> ProcedureId {
    ProcedureId::new(15)
}
fn wait_id() -> ProcedureId {
    ProcedureId::new(16)
}

fn int(ty: IntegerType, value: i128) -> IntExpr {
    IntExpr::constant(Integer::checked(ty, value).unwrap())
}
fn arithmetic(ty: IntegerType, op: IntOp, left: IntExpr, right: IntExpr) -> IntExpr {
    IntExpr::new(ty, IntExprKind::Binary(op, Box::new(left), Box::new(right)))
}
fn call(id: ProcedureId, arguments: Vec<ValueExpr>) -> Call {
    Call::new(
        id,
        arguments
            .into_iter()
            .enumerate()
            .map(|(index, value)| (ParameterId::new(index), value))
            .collect(),
    )
}
fn scalar_call(ty: IntegerType, id: ProcedureId, arguments: Vec<ValueExpr>) -> IntExpr {
    IntExpr::new(ty, IntExprKind::Call(call(id, arguments)))
}
fn cast(ty: IntegerType, value: IntExpr) -> IntExpr {
    IntExpr::new(ty, IntExprKind::Cast(CastMode::Checked, Box::new(value)))
}
fn block(statements: Vec<Statement>, flow: Flow) -> Block {
    Block { statements, flow }
}
fn ret(value: IntExpr, cleanups: Vec<CleanupId>) -> Statement {
    Statement::Exit(Exit {
        cleanups,
        transfer: Transfer::ReturnInt(value),
    })
}
fn signature(
    types: &mut TypeRegistry,
    parameters: Vec<TypeId>,
    results: Vec<TypeId>,
    convention: CallingConvention,
) -> TypeId {
    types
        .procedure(ProcedureType {
            parameters: parameters.into(),
            results: results.into(),
            convention,
            context: ContextMode::None,
            variadic: Variadic::None,
        })
        .unwrap()
}
fn address(place: Place, ty: TypeId) -> ValueExpr {
    ValueExpr::AddressOf { place, ty }
}

struct Fixture {
    library: Library,
    proofs: HashMap<ProcedureId, ProcessAbiProcedure>,
    message_signature: TypeId,
    word: TypeId,
    fork_pending: Cell<bool>,
}
impl Fixture {
    fn new() -> Self {
        Self::with_reader_forks(None)
    }
    fn for_reader_forks(count: usize) -> Self {
        Self::with_reader_forks(Some(count))
    }
    fn with_reader_forks(reader_forks: Option<usize>) -> Self {
        let mut types = TypeRegistry::new();
        let s32 = types.scalar(ScalarType::Int(IntegerType::S32));
        let word = types.scalar(ScalarType::Int(IntegerType::S64));
        let count = types.scalar(ScalarType::Int(IntegerType::U64));
        let string = types.string();
        let pair = types.fixed_array(s32, 2).unwrap();
        let pair_pointer = types.pointer(pair).unwrap();
        let status_pointer = types.pointer(s32).unwrap();
        let word_pointer = types.pointer(count).unwrap();
        let void_pointer = types.pointer(types.void()).unwrap();
        let error_code = types.reserve_distinct(DistinctKind::IsA);
        types.define_distinct(error_code, s32).unwrap();
        let main_signature = signature(&mut types, vec![], vec![word], CallingConvention::Jai);
        let message_signature = signature(&mut types, vec![string], vec![], CallingConvention::Jai);
        let target = BuildTarget {
            operating_system: OperatingSystem::MacOS,
            architecture: Architecture::Arm64,
            layout: LayoutPolicy::lp64(),
            byte_order: ByteOrder::Little,
        };
        let foreign = ForeignLibrary {
            id: ForeignLibraryId::new(Identities::default().declaration()),
            kind: ForeignLibraryKind::System {
                name: "libc".into(),
            },
            options: Default::default(),
        };
        let definitions = [
            (
                fork_id(),
                ProcessAbiOperation::Fork,
                "fork",
                vec![],
                vec![s32],
            ),
            (
                pipe_id(),
                ProcessAbiOperation::Pipe,
                "pipe",
                vec![pair_pointer],
                vec![s32],
            ),
            (
                close_id(),
                ProcessAbiOperation::Close,
                "close",
                vec![s32],
                vec![s32],
            ),
            (
                read_id(),
                ProcessAbiOperation::Read,
                "read",
                vec![s32, void_pointer, count],
                vec![word],
            ),
            (
                write_id(),
                ProcessAbiOperation::Write,
                "write",
                vec![s32, void_pointer, count],
                vec![word],
            ),
            (
                exit_id(),
                ProcessAbiOperation::Exit,
                "_exit",
                vec![s32],
                vec![],
            ),
            (
                wait_id(),
                ProcessAbiOperation::WaitPid,
                "waitpid",
                vec![s32, status_pointer, s32],
                vec![s32],
            ),
        ];
        let mut proofs = HashMap::new();
        let mut prototypes = vec![ProcedurePrototype {
            id: message_id(),
            signature: message_signature,
            origin: PrototypeOrigin::Compiler,
        }];
        for (id, operation, symbol, parameters, results) in definitions {
            let signature = signature(&mut types, parameters, results, CallingConvention::C);
            let authority = ProcessAuthority::from_verified_source(
                target.clone(),
                foreign.clone(),
                ProcessAbiNominals {
                    error_code,
                    socket: None,
                },
                [(id, operation, signature)],
                &types,
            )
            .unwrap();
            let prototype = ProcedurePrototype {
                id,
                signature,
                origin: PrototypeOrigin::Foreign {
                    symbol: symbol.into(),
                    library: Some(foreign.clone()),
                },
            };
            proofs.insert(id, authority.bind(&prototype, &target, &types).unwrap());
            prototypes.push(prototype);
        }
        let globals: Vec<_> = [0, 0, 41, 0, 0, 0, 0]
            .into_iter()
            .enumerate()
            .map(|(index, value)| {
                Global::new(
                    index,
                    GlobalInitializer::Int(Integer::checked(IntegerType::S64, value).unwrap()),
                    &types,
                )
            })
            .collect();
        let global_places: Vec<_> = globals
            .iter()
            .map(|global| IntPlace::try_from_place(global.place(), &types).unwrap())
            .collect();
        let locals: Vec<_> = [pair, s32, count, s32, word, word, s32]
            .into_iter()
            .enumerate()
            .map(|(index, ty)| Local::new_typed(main_id(), index, ty, &types).unwrap())
            .collect();
        let local_int =
            |index: usize| IntPlace::try_from_place(locals[index].place(), &types).unwrap();
        let pid = local_int(1);
        let buffer = local_int(2);
        let status = local_int(3);
        let frame_marker = local_int(4);
        let read_count = local_int(5);
        let wait_result = local_int(6);
        let mut places = PlaceRegistry::new();
        let reader = IntPlace::try_from_place(
            places
                .index(locals[0].place(), int(IntegerType::S64, 0), &types)
                .unwrap(),
            &types,
        )
        .unwrap();
        let writer = IntPlace::try_from_place(
            places
                .index(locals[0].place(), int(IntegerType::S64, 1), &types)
                .unwrap(),
            &types,
        )
        .unwrap();
        let buffer_address = || ValueExpr::PointerCast {
            value: Box::new(address(locals[2].place(), word_pointer)),
            ty: void_pointer,
            mode: CastMode::Checked,
        };
        let message = |text: &[u8]| {
            Statement::CallVoid(call(
                message_id(),
                vec![ValueExpr::StringBytes {
                    ty: string,
                    bytes: text.to_vec(),
                }],
            ))
        };
        let close = |place: IntPlace| {
            Statement::DiscardInt(scalar_call(
                IntegerType::S32,
                close_id(),
                vec![ValueExpr::Int(IntExpr::load(place))],
            ))
        };
        let capture = ExpressionBindingId::new(main_id(), 0);
        let left = IntExpr::new(
            IntegerType::S64,
            IntExprKind::Value(Box::new(ValueExpr::Bound {
                binding: capture,
                ty: word,
            })),
        );
        let fork_sum = ValueExpr::Bind {
            bindings: vec![(
                capture,
                ValueExpr::Int(scalar_call(IntegerType::S64, bump_id(), vec![])),
            )],
            body: Box::new(ValueExpr::Int(arithmetic(
                IntegerType::S32,
                IntOp::Add,
                cast(IntegerType::S32, left),
                scalar_call(IntegerType::S32, fork_id(), vec![]),
            ))),
            ty: s32,
        };
        let fork_pid = arithmetic(
            IntegerType::S32,
            IntOp::Subtract,
            IntExpr::new(IntegerType::S32, IntExprKind::Value(Box::new(fork_sum))),
            int(IntegerType::S32, 1),
        );
        let child = block(
            vec![
                close(reader),
                Statement::StoreInt(frame_marker, int(IntegerType::S64, 999)),
                Statement::StoreInt(global_places[2], int(IntegerType::S64, 999)),
                Statement::StoreInt(global_places[0], int(IntegerType::S64, 999)),
                Statement::StoreInt(buffer, int(IntegerType::U64, 42)),
                Statement::DiscardInt(scalar_call(
                    IntegerType::S64,
                    write_id(),
                    vec![
                        ValueExpr::Int(IntExpr::load(writer)),
                        buffer_address(),
                        ValueExpr::Int(int(IntegerType::U64, 1)),
                    ],
                )),
                close(writer),
                Statement::CallVoid(call(
                    exit_id(),
                    vec![ValueExpr::Int(int(IntegerType::S32, 7))],
                )),
                message(b"after-exit"),
                ret(int(IntegerType::S64, -999), vec![CleanupId::new(0)]),
            ],
            Flow::Terminates,
        );
        let parent = block(
            vec![
                close(writer),
                Statement::StoreInt(
                    read_count,
                    scalar_call(
                        IntegerType::S64,
                        read_id(),
                        vec![
                            ValueExpr::Int(IntExpr::load(reader)),
                            buffer_address(),
                            ValueExpr::Int(int(IntegerType::U64, 1)),
                        ],
                    ),
                ),
                close(reader),
                Statement::StoreInt(
                    wait_result,
                    scalar_call(
                        IntegerType::S32,
                        wait_id(),
                        vec![
                            ValueExpr::Int(IntExpr::load(pid)),
                            address(locals[3].place(), status_pointer),
                            ValueExpr::Int(int(IntegerType::S32, 0)),
                        ],
                    ),
                ),
                Statement::StoreInt(global_places[3], IntExpr::load(frame_marker)),
                Statement::StoreInt(
                    global_places[4],
                    cast(IntegerType::S64, IntExpr::load(buffer)),
                ),
                Statement::StoreInt(
                    global_places[5],
                    cast(IntegerType::S64, IntExpr::load(status)),
                ),
                Statement::StoreInt(global_places[6], cast(IntegerType::S64, IntExpr::load(pid))),
                Statement::If(
                    BoolExpr::CompareInts(
                        Relation::Equal,
                        Box::new(IntExpr::load(wait_result)),
                        Box::new(IntExpr::load(pid)),
                    ),
                    block(
                        vec![ret(
                            arithmetic(
                                IntegerType::S64,
                                IntOp::Add,
                                IntExpr::load(frame_marker),
                                IntExpr::load(read_count),
                            ),
                            vec![CleanupId::new(0)],
                        )],
                        Flow::Terminates,
                    ),
                    block(
                        vec![ret(int(IntegerType::S64, -888), vec![CleanupId::new(0)])],
                        Flow::Terminates,
                    ),
                ),
            ],
            Flow::Terminates,
        );
        let body = block(
            vec![
                // Realize these inherited roots in source before splitting them.
                Statement::StoreInt(global_places[0], IntExpr::load(global_places[0])),
                Statement::StoreInt(global_places[2], IntExpr::load(global_places[2])),
                Statement::StoreInt(frame_marker, int(IntegerType::S64, 41)),
                Statement::StoreInt(buffer, int(IntegerType::U64, 0)),
                Statement::DiscardInt(scalar_call(
                    IntegerType::S32,
                    pipe_id(),
                    vec![address(locals[0].place(), pair_pointer)],
                )),
                message(b"before-fork"),
                Statement::StoreInt(pid, fork_pid),
                Statement::If(
                    BoolExpr::CompareInts(
                        Relation::Equal,
                        Box::new(IntExpr::load(pid)),
                        Box::new(int(IntegerType::S32, 0)),
                    ),
                    child,
                    parent,
                ),
            ],
            Flow::Terminates,
        );
        let cleanup = block(
            vec![
                message(b"cleanup"),
                Statement::StoreInt(
                    global_places[1],
                    arithmetic(
                        IntegerType::S64,
                        IntOp::Add,
                        IntExpr::load(global_places[1]),
                        int(IntegerType::S64, 1),
                    ),
                ),
            ],
            Flow::FallsThrough,
        )
        .into();
        let bump = Procedure {
            id: bump_id(),
            signature: main_signature,
            parameters: vec![],
            locals: vec![],
            body: block(
                vec![
                    Statement::StoreInt(
                        global_places[0],
                        arithmetic(
                            IntegerType::S64,
                            IntOp::Add,
                            IntExpr::load(global_places[0]),
                            int(IntegerType::S64, 1),
                        ),
                    ),
                    ret(int(IntegerType::S64, 1), vec![]),
                ],
                Flow::Terminates,
            ),
            cleanups: vec![],
        };
        // Each real child stops in a checked read; only the parent continues
        // constructing siblings. This exposes readiness without inventing events.
        let body = if let Some(count) = reader_forks {
            let mut statements = vec![
                Statement::StoreInt(buffer, int(IntegerType::U64, 0)),
                Statement::DiscardInt(scalar_call(
                    IntegerType::S32,
                    pipe_id(),
                    vec![address(locals[0].place(), pair_pointer)],
                )),
            ];
            for _ in 0..count {
                statements.push(Statement::StoreInt(
                    pid,
                    scalar_call(IntegerType::S32, fork_id(), vec![]),
                ));
                statements.push(Statement::If(
                    BoolExpr::CompareInts(
                        Relation::Equal,
                        Box::new(IntExpr::load(pid)),
                        Box::new(int(IntegerType::S32, 0)),
                    ),
                    block(
                        vec![
                            Statement::DiscardInt(scalar_call(
                                IntegerType::S64,
                                read_id(),
                                vec![
                                    ValueExpr::Int(IntExpr::load(reader)),
                                    buffer_address(),
                                    ValueExpr::Int(int(IntegerType::U64, 1)),
                                ],
                            )),
                            ret(int(IntegerType::S64, 42), vec![]),
                        ],
                        Flow::Terminates,
                    ),
                    block(vec![], Flow::FallsThrough),
                ));
            }
            statements.push(ret(int(IntegerType::S64, 42), vec![]));
            block(statements, Flow::Terminates)
        } else {
            body
        };
        let main = Procedure {
            id: main_id(),
            signature: main_signature,
            parameters: vec![],
            locals,
            body,
            cleanups: vec![cleanup],
        };
        let library = ProgramBuilder::new(types.freeze().unwrap())
            .procedures(vec![main, bump])
            .prototypes(prototypes)
            .foreign_libraries(vec![foreign])
            .globals(globals)
            .places(places.freeze())
            .finish_library()
            .unwrap();
        Self {
            library,
            proofs,
            message_signature,
            word,
            fork_pending: Cell::new(false),
        }
    }
}
impl ProcedureProvider for Fixture {
    fn types(&self) -> &dyn TypeView {
        self.library.types()
    }
    fn signatures(&self) -> &HashMap<ProcedureId, TypeId> {
        self.library.signatures()
    }
    fn globals(&self) -> &[Global] {
        self.library.globals()
    }
    fn places(&self) -> Option<&Places> {
        Some(self.library.places())
    }
    fn procedure(&self, id: ProcedureId) -> ProcedureAvailability<'_> {
        if id == fork_id() && self.fork_pending.get() {
            return ProcedureAvailability::Pending(Dependency::Procedure(fork_id()));
        }
        if id == message_id() {
            return ProcedureAvailability::Compiler(CompilerProcedure {
                signature: self.message_signature,
                intrinsic: crate::CompilerIntrinsic::Message(crate::MessageLevel::Info),
            });
        }
        if let Some(proof) = self.proofs.get(&id) {
            return ProcedureAvailability::ProcessAbi(proof.clone());
        }
        self.library
            .checked_procedure(id)
            .map_or(ProcedureAvailability::Missing, ProcedureAvailability::Ready)
    }
}

struct Effects {
    ready: bool,
    parked: bool,
    begins: usize,
    requests: Vec<String>,
    polls: usize,
    finishes: Vec<bool>,
}
impl Default for Effects {
    fn default() -> Self {
        Self {
            ready: true,
            parked: false,
            begins: 0,
            requests: vec![],
            polls: 0,
            finishes: vec![],
        }
    }
}
impl CompilerEffects for Effects {
    fn begin(&mut self) {
        self.begins += 1;
    }
    fn suspend(&mut self) -> std::result::Result<(), Error> {
        assert!(!self.parked);
        self.parked = true;
        Ok(())
    }
    fn resume(&mut self) -> std::result::Result<(), Error> {
        assert!(self.parked);
        self.parked = false;
        Ok(())
    }
    fn request(&mut self, request: crate::CompilerRequest) -> crate::EffectOutcome {
        let crate::CompilerRequest::Message { text, .. } = request else {
            panic!("unexpected scheduler fixture request")
        };
        self.requests.push(text.clone());
        if text == "before-fork" && !self.ready {
            crate::EffectOutcome::Pending(crate::EffectKey(23))
        } else {
            crate::EffectOutcome::Ready(crate::CompilerResponse::Unit)
        }
    }
    fn poll_request(
        &mut self,
        request: &crate::CompilerRequest,
        key: crate::EffectKey,
    ) -> crate::EffectOutcome {
        assert_eq!(key, crate::EffectKey(23));
        assert!(
            matches!(request, crate::CompilerRequest::Message { text, .. } if text == "before-fork")
        );
        self.polls += 1;
        if self.ready {
            crate::EffectOutcome::Ready(crate::CompilerResponse::Unit)
        } else {
            crate::EffectOutcome::Pending(key)
        }
    }
    fn finish(&mut self, commit: bool) -> std::result::Result<(), Error> {
        self.finishes.push(commit);
        self.parked = false;
        Ok(())
    }
}
fn global(vm: &Vm<'_, Fixture, Effects>, index: usize) -> i128 {
    vm.memory
        .load(vm.provider.types(), vm.globals[index].as_ref().unwrap())
        .unwrap()
        .integer()
        .unwrap()
        .value()
}
fn publish_and_check(vm: &mut Vm<'_, Fixture, Effects>) {
    assert_eq!(
        vm.resumable_values(),
        Some([Value::Int(Integer::checked(IntegerType::S64, 42).unwrap())].as_slice())
    );
    assert_eq!(vm.statistics.calls, 14);
    assert_eq!((global(vm, 0), global(vm, 1), global(vm, 2)), (1, 1, 41));
    assert_eq!(
        (global(vm, 3), global(vm, 4), global(vm, 5)),
        (41, 42, 7 << 8)
    );
    let state = vm.processes.as_ref().unwrap();
    let child = state
        .world
        .resolve_pid(
            state.branch.current(),
            i32::try_from(global(vm, 6)).unwrap(),
        )
        .unwrap();
    assert_eq!(
        state.world.parent(child).unwrap(),
        Some(state.branch.current())
    );
    state
        .world
        .require_quiescent(state.branch.current())
        .unwrap();
    assert_eq!(vm.expression_bindings.cells(), 0);
    assert_eq!(vm.effects().requests, ["before-fork", "cleanup"]);
    assert_eq!(
        vm.finish_resumable_validated(|vm, values| {
            assert_eq!(
                values,
                [Value::Int(Integer::checked(IntegerType::S64, 42).unwrap())]
            );
            assert_eq!(global(vm, 0), 1);
            Ok(())
        })
        .outcome,
        Outcome::Complete(vec![Value::Int(
            Integer::checked(IntegerType::S64, 42).unwrap()
        )])
    );
    assert_eq!(vm.effects().begins, 1);
    assert_eq!(vm.effects().finishes, [true]);
}

#[test]
fn real_pipe_fork_parent_read_child_write_exit_wait_reap_from_both_public_entries() {
    for expression_entry in [false, true] {
        let fixture = Fixture::new();
        let mut vm = Vm::new(&fixture, Effects::default(), Limits::default()).unwrap();
        let progress = if expression_entry {
            vm.start_resumable_expression(&ValueExpr::Call {
                call: call(main_id(), vec![]),
                ty: fixture.word,
            })
        } else {
            vm.start_resumable_procedure(main_id(), vec![])
        };
        assert_eq!(progress.outcome, ResumableOutcome::AwaitingPublication);
        publish_and_check(&mut vm);
    }
}

#[test]
fn prior_effect_bind_capture_and_left_operand_survive_external_readiness_and_fork_once() {
    let fixture = Fixture::new();
    let mut vm = Vm::new(
        &fixture,
        Effects {
            ready: false,
            ..Effects::default()
        },
        Limits::default(),
    )
    .unwrap();
    assert_eq!(
        vm.start_resumable_procedure(main_id(), vec![]).outcome,
        ResumableOutcome::Suspended(vec![Dependency::Effect(crate::EffectKey(23))])
    );
    assert_eq!(vm.effects().requests, ["before-fork"]);
    assert_eq!(global(&vm, 0), 0);
    vm.effects_mut().ready = true;
    assert_eq!(
        vm.resume_resumable().outcome,
        ResumableOutcome::AwaitingPublication
    );
    assert_eq!(vm.effects().polls, 1);
    publish_and_check(&mut vm);
}

#[test]
fn low_combined_cells_and_fuel_reject_private_fork_then_restore_the_source_checkpoint() {
    for limit in [LimitKind::ValueCells, LimitKind::Fuel] {
        let fixture = Fixture::new();
        fixture.fork_pending.set(true);
        let mut vm = Vm::new(&fixture, Effects::default(), Limits::default()).unwrap();
        let baseline = (vm.memory.value_cells(), vm.memory.allocation_count());
        assert_eq!(
            vm.start_resumable_procedure(main_id(), vec![]).outcome,
            ResumableOutcome::Suspended(vec![Dependency::Procedure(fork_id())])
        );
        assert_eq!(global(&vm, 0), 1);
        assert!(vm.expression_bindings.cells() > 0);
        assert_eq!(vm.effects().requests, ["before-fork"]);
        match limit {
            LimitKind::ValueCells => {
                let occupied = vm.memory.value_cells()
                    + vm.expression_bindings.cells()
                    + vm.continuation
                        .as_ref()
                        .unwrap()
                        .root
                        .retained_cells()
                        .unwrap()
                    + vm.processes.as_ref().unwrap().cells();
                let scheduler = &vm.continuation.as_ref().unwrap().process_scheduler;
                let reserved = scheduler.rollback_cells + scheduler.resident_ancillary_cells;
                // Admit the actual rollback/initial ancillary owners and current
                // fork call/stop, but not a second complete private branch.
                vm.limits.value_cells = reserved + occupied + 16;
            }
            LimitKind::Fuel => vm.limits.fuel = vm.statistics.steps + 1,
            _ => unreachable!(),
        }
        fixture.fork_pending.set(false);
        assert_eq!(
            vm.resume_resumable().outcome,
            ResumableOutcome::Failed(Error::Limit(limit))
        );
        assert_eq!(vm.statistics.calls, 5);
        assert_eq!(
            (vm.memory.value_cells(), vm.memory.allocation_count()),
            baseline
        );
        assert!(vm.processes.is_none());
        assert!(vm.frames.is_empty());
        assert_eq!(vm.expression_bindings.cells(), 0);
        assert!(vm.resumable_values().is_none());
        assert_eq!(vm.effects().finishes, [false]);
    }
}

fn stopped_main<'a>(fixture: &'a Fixture) -> (Vm<'a, Fixture, Effects>, machine::Machine) {
    let mut vm = Vm::new(fixture, Effects::default(), Limits::default()).unwrap();
    let mut machine = machine::Machine::procedure(main_id(), vec![]);
    assert_eq!(
        machine.drive(&mut vm).unwrap(),
        machine::DriveStatus::ProcessControl
    );
    assert!(matches!(
        machine.process_control(),
        Some(ProcessControl::Fork(_))
    ));
    (vm, machine)
}

struct ResidentBefore {
    world_work: u64,
    world_cells: usize,
    current: ProcessId,
    retained: usize,
    memory_cells: usize,
    allocations: usize,
    slots: Vec<Vec<Pointer>>,
    frame_owners: Vec<(ProcedureId, usize)>,
    globals: Vec<Option<Pointer>>,
    bindings: usize,
    queue: Vec<(usize, ProcessId, usize, Option<ProcessEvent>)>,
    parked_cells: usize,
    root: Option<ProcessId>,
    next_ordinal: usize,
}
impl ResidentBefore {
    fn capture(
        scheduler: &ProcessScheduler,
        vm: &Vm<'_, Fixture, Effects>,
        machine: &machine::Machine,
    ) -> Self {
        let state = vm.processes.as_ref().unwrap();
        Self {
            world_work: state.world.work_cost().unwrap(),
            world_cells: state.cells(),
            current: state.branch.current(),
            retained: machine.retained_cells(),
            memory_cells: vm.memory.value_cells(),
            allocations: vm.memory.allocation_count(),
            slots: vm.frames.iter().map(|frame| frame.slots.clone()).collect(),
            frame_owners: vm
                .frames
                .iter()
                .map(|frame| match &frame.procedure {
                    FrameCode::Owned(code) => (code.id, Arc::strong_count(code)),
                    FrameCode::Borrowed(_) => {
                        panic!("scheduler source frame must own checked code")
                    }
                })
                .collect(),
            globals: vm.globals.clone(),
            bindings: vm.expression_bindings.cells(),
            queue: scheduler
                .parked
                .iter()
                .map(|(ordinal, branch)| {
                    (
                        *ordinal,
                        branch.storage.process_branch().current(),
                        branch.cells,
                        branch.pending,
                    )
                })
                .collect(),
            parked_cells: scheduler.parked_cells,
            root: scheduler.root,
            next_ordinal: scheduler.next_ordinal,
        }
    }
    fn assert_preserved(
        &self,
        scheduler: &ProcessScheduler,
        vm: &Vm<'_, Fixture, Effects>,
        machine: &machine::Machine,
    ) {
        let actual = Self::capture(scheduler, vm, machine);
        assert_eq!(
            (actual.world_work, actual.world_cells, actual.current),
            (self.world_work, self.world_cells, self.current)
        );
        assert_eq!(
            (actual.retained, actual.memory_cells, actual.allocations),
            (self.retained, self.memory_cells, self.allocations)
        );
        assert_eq!(actual.slots, self.slots);
        assert_eq!(actual.frame_owners, self.frame_owners);
        assert_eq!(actual.globals, self.globals);
        assert_eq!(actual.bindings, self.bindings);
        assert_eq!(actual.queue, self.queue);
        assert_eq!(
            (actual.parked_cells, actual.root, actual.next_ordinal),
            (self.parked_cells, self.root, self.next_ordinal)
        );
        for (_, child, _, _) in &self.queue {
            assert_eq!(
                vm.processes.as_ref().unwrap().world.parent(*child).unwrap(),
                Some(self.current)
            );
            vm.processes
                .as_ref()
                .unwrap()
                .world
                .validate_running(*child)
                .unwrap();
        }
        assert!(
            matches!(machine.process_control(), Some(ProcessControl::Fork(control))
                        if control.parent() == self.current)
        );
        assert!(machine.values().is_none());
    }
}

fn fork_first_then_stop_second<'a>(
    fixture: &'a Fixture,
) -> (Vm<'a, Fixture, Effects>, machine::Machine, ProcessScheduler) {
    let (mut vm, mut machine) = stopped_main(fixture);
    let parent = vm.processes.as_ref().unwrap().branch.current();
    // This private fixture has no Session checkpoint. Public entry tests above
    // cover retained rollback state and its final transaction restoration.
    let mut scheduler = ProcessScheduler::new(0, 0);
    scheduler.fork(&mut vm, &mut machine, parent).unwrap();
    assert_eq!(scheduler.parked.len(), 1);
    assert_eq!(
        machine.drive(&mut vm).unwrap(),
        machine::DriveStatus::ProcessControl
    );
    (vm, machine, scheduler)
}

#[test]
fn queue_ordinal_overflow_stops_fork_before_copy_or_resident_mutation() {
    let fixture = Fixture::for_reader_forks(2);
    let (mut vm, mut machine, mut scheduler) = fork_first_then_stop_second(&fixture);
    scheduler.next_ordinal = usize::MAX;
    let before = ResidentBefore::capture(&scheduler, &vm, &machine);
    let statistics = vm.statistics;
    assert!(matches!(
        scheduler.fork(&mut vm, &mut machine, before.current),
        Err(Halt::Failed(Error::Limit(LimitKind::Fuel)))
    ));
    // The checked ordinal gate precedes even the cached world inspection charge.
    assert_eq!(vm.statistics.steps, statistics.steps);
    assert_eq!(vm.statistics.calls, statistics.calls);
    before.assert_preserved(&scheduler, &vm, &machine);
}

#[test]
fn queue_ordinal_overflow_cannot_remove_a_ready_child_or_swap_frames() {
    let fixture = Fixture::for_reader_forks(2);
    let (mut vm, mut machine, mut scheduler) = fork_first_then_stop_second(&fixture);
    scheduler.next_ordinal = usize::MAX;
    let before = ResidentBefore::capture(&scheduler, &vm, &machine);
    let calls = vm.statistics.calls;
    assert!(matches!(
        scheduler.switch(&mut vm, &mut machine, None, false),
        Err(Halt::Failed(Error::Limit(LimitKind::Fuel)))
    ));
    assert_eq!(vm.statistics.calls, calls);
    before.assert_preserved(&scheduler, &vm, &machine);
}

#[test]
fn preclone_combined_quota_denial_preserves_real_controls_frames_and_existing_queue() {
    for queued_child in [false, true] {
        let fixture = if queued_child {
            Fixture::for_reader_forks(2)
        } else {
            Fixture::new()
        };
        let (mut vm, mut machine, mut scheduler) = if queued_child {
            fork_first_then_stop_second(&fixture)
        } else {
            let (vm, machine) = stopped_main(&fixture);
            (vm, machine, ProcessScheduler::new(0, 0))
        };
        if !queued_child {
            assert_eq!(global(&vm, 0), 1);
            assert!(vm.expression_bindings.cells() > 0);
        }
        let before = ResidentBefore::capture(&scheduler, &vm, &machine);
        let calls = vm.statistics.calls;
        // Admit the actual resident owners, without reserving a second private
        // storage/machine owner or a candidate world. No configured maxima or
        // private snapshot formula is reproduced here.
        vm.limits.value_cells = vm.memory.value_cells()
            + vm.expression_bindings.cells()
            + machine.retained_cells()
            + vm.processes.as_ref().unwrap().cells()
            + scheduler.parked_cells;
        assert!(matches!(
            scheduler.fork(&mut vm, &mut machine, before.current),
            Err(Halt::Failed(Error::Limit(LimitKind::ValueCells)))
        ));
        assert_eq!(vm.statistics.calls, calls);
        before.assert_preserved(&scheduler, &vm, &machine);
        if !queued_child {
            assert_eq!(global(&vm, 0), 1);
        }
    }
}

#[test]
fn full_clone_fuel_is_rejected_before_candidate_world_or_private_owner_changes() {
    let fixture = Fixture::new();
    let (mut vm, mut machine) = stopped_main(&fixture);
    let mut scheduler = ProcessScheduler::new(0, 0);
    let before = ResidentBefore::capture(&scheduler, &vm, &machine);
    let statistics = vm.statistics;
    // Calibrate only the admitted borrowed inspection. Leave enough fuel to
    // inspect the genuine world and branch, but no clone/injection budget.
    branches::measured(&mut vm, true).unwrap();
    let inspection_work = vm.statistics.steps - statistics.steps;
    vm.statistics = statistics;
    vm.limits.fuel =
        statistics.steps + vm.processes.as_ref().unwrap().work_cost() + inspection_work + 1;
    assert!(matches!(
        scheduler.fork(&mut vm, &mut machine, before.current),
        Err(Halt::Failed(Error::Limit(LimitKind::Fuel)))
    ));
    assert_eq!(vm.statistics.calls, statistics.calls);
    before.assert_preserved(&scheduler, &vm, &machine);
}

#[test]
fn fifo_switching_serves_each_genuinely_ready_pipe_reader_before_revisiting_one() {
    let fixture = Fixture::for_reader_forks(3);
    let (mut vm, mut machine) = stopped_main(&fixture);
    let root = vm.processes.as_ref().unwrap().branch.current();
    let mut scheduler = ProcessScheduler::new(0, 0);
    for index in 0..3 {
        scheduler.fork(&mut vm, &mut machine, root).unwrap();
        if index != 2 {
            assert_eq!(
                machine.drive(&mut vm).unwrap(),
                machine::DriveStatus::ProcessControl
            );
        }
    }
    let children: Vec<_> = scheduler
        .parked
        .values()
        .map(|branch| branch.storage.process_branch().current())
        .collect();
    assert_eq!(children.len(), 3);
    assert!(
        scheduler
            .switch(&mut vm, &mut machine, None, false)
            .unwrap()
    );
    for child in &children {
        assert_eq!(vm.processes.as_ref().unwrap().branch.current(), *child);
        let Err(Halt::Pending(Dependency::Process(event))) = machine.drive(&mut vm) else {
            panic!("real child must suspend in its checked pipe read")
        };
        assert!(
            !vm.processes
                .as_ref()
                .unwrap()
                .world
                .event_ready(event)
                .unwrap()
        );
        assert!(
            scheduler
                .switch(&mut vm, &mut machine, Some(event), false)
                .unwrap()
        );
    }
    assert_eq!(vm.processes.as_ref().unwrap().branch.current(), root);
    assert_eq!(scheduler.ready(&mut vm).unwrap(), None);
    let pair = &vm.frames.last().unwrap().slots[0];
    let writer = vm.memory.index(vm.provider.types(), pair, 1).unwrap();
    let writer = vm
        .memory
        .load(vm.provider.types(), &writer)
        .unwrap()
        .integer()
        .unwrap()
        .value();
    let state = vm.processes.as_mut().unwrap();
    let writer = state
        .world
        .descriptor(root, i32::try_from(writer).unwrap())
        .unwrap();
    assert_eq!(state.world.write(writer, b"ABC").unwrap(), 3);
    state.refresh(vm.limits).unwrap();
    for branch in scheduler.parked.values() {
        let event = branch
            .pending
            .expect("child retains its actual pending read event");
        assert!(
            vm.processes
                .as_ref()
                .unwrap()
                .world
                .event_ready(event)
                .unwrap()
        );
    }
    let calls = vm.statistics.calls;
    // Switching preserves the stopped read; leaving the selected machine
    // unstepped keeps it runnable, and it is reinserted behind existing peers.
    let expected = [children[0], children[1], children[2], root, children[0]];
    for process in expected {
        assert!(
            scheduler
                .switch(&mut vm, &mut machine, None, false)
                .unwrap()
        );
        assert_eq!(vm.processes.as_ref().unwrap().branch.current(), process);
    }
    assert_eq!(vm.statistics.calls, calls);
}
