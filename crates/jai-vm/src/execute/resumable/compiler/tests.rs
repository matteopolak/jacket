use super::*;
use crate::compiler_code_plan::{CompilerCodePlan, CompilerCodePlanBuilder, CompilerRuntimeLeaf};
use crate::{
    CompilerIntrinsic, CompilerRequest, CompilerResponse, EffectKey, EffectOutcome, WorkspaceId,
};
use jai_types::{CallingConvention, ContextMode, IntegerType, ScalarType, TypeRegistry, Variadic};
use std::cell::Cell;

fn owner() -> ProcedureId {
    ProcedureId::new(0)
}
fn wait() -> ProcedureId {
    ProcedureId::new(1)
}
fn create() -> ProcedureId {
    ProcedureId::new(2)
}

struct Provider {
    library: Library,
    wait: Cell<bool>,
    string: TypeId,
}
impl ProcedureProvider for Provider {
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
        if id == wait() && self.wait.get() {
            return ProcedureAvailability::Pending(Dependency::Procedure(id));
        }
        if id == create() {
            return ProcedureAvailability::Compiler(CompilerProcedure {
                signature: self.signatures()[&id],
                intrinsic: CompilerIntrinsic::CreateWorkspace,
            });
        }
        self.library
            .checked_procedure(id)
            .map_or(ProcedureAvailability::Missing, ProcedureAvailability::Ready)
    }
}
fn signature(types: &mut TypeRegistry, parameters: Vec<TypeId>, results: Vec<TypeId>) -> TypeId {
    types
        .procedure(ProcedureType {
            parameters: parameters.into(),
            results: results.into(),
            return_abi: jai_types::ForeignReturnAbi::Natural,
            convention: CallingConvention::Jai,
            context: ContextMode::None,
            variadic: Variadic::None,
        })
        .unwrap()
}
fn int(n: i128) -> IntExpr {
    IntExpr::constant(Integer::wrapping(IntegerType::S64, n))
}
fn fixture() -> Provider {
    fixture_with_globals(1)
}
fn fixture_with_globals(count: usize) -> Provider {
    let mut types = TypeRegistry::new();
    let word = types.scalar(ScalarType::Int(IntegerType::S64));
    let wide = types.scalar(ScalarType::Int(IntegerType::U64));
    let boolean = types.scalar(ScalarType::Bool);
    let string = types.string();
    let writer = signature(&mut types, vec![], vec![word]);
    let wait_signature = signature(&mut types, vec![], vec![boolean]);
    let create_signature = signature(&mut types, vec![string], vec![wide]);
    let globals: Vec<_> = (0..count)
        .map(|index| {
            Global::new(
                index,
                GlobalInitializer::Int(Integer::wrapping(IntegerType::S64, 41)),
                &types,
            )
        })
        .collect();
    let place = IntPlace::try_from_place(globals[0].place(), &types).unwrap();
    let procedures = vec![
        Procedure {
            id: owner(),
            signature: writer,
            parameters: vec![],
            locals: vec![],
            cleanups: vec![],
            body: Block {
                flow: Flow::Terminates,
                statements: vec![
                    Statement::StoreInt(
                        place,
                        IntExpr::new(
                            IntegerType::S64,
                            IntExprKind::Binary(
                                IntOp::Add,
                                Box::new(IntExpr::load(place)),
                                Box::new(int(1)),
                            ),
                        ),
                    ),
                    Statement::Exit(Exit {
                        cleanups: vec![],
                        transfer: Transfer::ReturnInt(IntExpr::load(place)),
                    }),
                ],
            },
        },
        Procedure {
            id: wait(),
            signature: wait_signature,
            parameters: vec![],
            locals: vec![],
            cleanups: vec![],
            body: Block {
                flow: Flow::Terminates,
                statements: vec![Statement::Exit(Exit {
                    cleanups: vec![],
                    transfer: Transfer::ReturnBool(BoolExpr::Constant(true)),
                })],
            },
        },
    ];
    Provider {
        library: ProgramBuilder::new(types.freeze().unwrap())
            .procedures(procedures)
            .globals(globals)
            .prototypes(vec![ProcedurePrototype {
                id: create(),
                signature: create_signature,
                origin: PrototypeOrigin::Compiler,
            }])
            .finish_library()
            .unwrap(),
        wait: Cell::new(true),
        string,
    }
}
struct FixturePlan {
    plan: CompilerCodePlan,
    word: CompilerSlotId,
    workspace: CompilerSlotId,
    yes: CompilerReturnSiteId,
    no: CompilerReturnSiteId,
}
fn plan(provider: &Provider) -> FixturePlan {
    let mut b = CompilerCodePlanBuilder::new(Default::default()).unwrap();
    let word_ty = provider.types().scalar(ScalarType::Int(IntegerType::S64));
    let word = b.slot(word_ty).unwrap();
    let workspace = b
        .slot(provider.types().scalar(ScalarType::Int(IntegerType::U64)))
        .unwrap();
    let create = b
        .assign(
            workspace,
            CompilerRuntimeLeaf::call(Call::new(
                create(),
                vec![(
                    ParameterId::new(0),
                    ValueExpr::StringBytes {
                        ty: provider.string,
                        bytes: b"compiler-case".to_vec(),
                    },
                )],
            )),
        )
        .unwrap();
    let write = b
        .assign(word, CompilerRuntimeLeaf::call(Call::new(owner(), vec![])))
        .unwrap();
    // The fixture's owner is a real native writer. Compiler plan/frame identities
    // are independent; no Code-returning native signature is constructed.
    let binding = ExpressionBindingId::new(owner(), 0);
    let read = IntExpr::new(
        IntegerType::S64,
        IntExprKind::Value(Box::new(ValueExpr::Bound {
            binding,
            ty: word_ty,
        })),
    );
    let add = b
        .assign(
            word,
            CompilerRuntimeLeaf::expression(ValueExpr::Int(IntExpr::new(
                IntegerType::S64,
                IntExprKind::Binary(IntOp::Add, Box::new(read), Box::new(int(5))),
            )))
            .with_inputs(vec![CompilerRuntimeInput {
                binding,
                slot: word,
                ty: word_ty,
            }]),
        )
        .unwrap();
    let yes = b
        .reserve_return_site_with_captures(vec![word, workspace])
        .unwrap();
    let no = b
        .reserve_return_site_with_captures(vec![workspace])
        .unwrap();
    let y = b.return_code(yes).unwrap();
    let n = b.return_code(no).unwrap();
    let branch = b
        .branch(
            ValueExpr::Bool(BoolExpr::Call(Call::new(wait(), vec![]))),
            y,
            n,
        )
        .unwrap();
    let root = b
        .block_with_locals(vec![create, write, add, branch], vec![word, workspace])
        .unwrap();
    FixturePlan {
        plan: b.finish(root).unwrap(),
        word,
        workspace,
        yes,
        no,
    }
}
#[derive(Default)]
struct Effects {
    begins: usize,
    requests: usize,
    polls: usize,
    parked: bool,
    ready: bool,
    finishes: Vec<bool>,
    origin: Option<crate::SourceOrigin>,
}
impl CompilerEffects for Effects {
    fn set_source_origin(&mut self, origin: crate::SourceOrigin) {
        self.origin = Some(origin);
    }
    fn begin(&mut self) {
        self.begins += 1;
    }
    fn suspend(&mut self) -> std::result::Result<(), Error> {
        self.parked = true;
        Ok(())
    }
    fn resume(&mut self) -> std::result::Result<(), Error> {
        assert!(self.parked);
        self.parked = false;
        Ok(())
    }
    fn request(&mut self, request: CompilerRequest) -> EffectOutcome {
        assert!(matches!(request, CompilerRequest::CreateWorkspace { .. }));
        self.requests += 1;
        EffectOutcome::Pending(EffectKey(42))
    }
    fn poll_request(&mut self, request: &CompilerRequest, key: EffectKey) -> EffectOutcome {
        assert!(matches!(request, CompilerRequest::CreateWorkspace { .. }));
        assert_eq!(key, EffectKey(42));
        self.polls += 1;
        if self.ready {
            EffectOutcome::Ready(CompilerResponse::Workspace(
                WorkspaceId::from_raw(7).unwrap(),
            ))
        } else {
            EffectOutcome::Pending(key)
        }
    }
    fn finish(&mut self, commit: bool) -> std::result::Result<(), Error> {
        self.finishes.push(commit);
        self.parked = false;
        Ok(())
    }
}
fn origin() -> crate::SourceOrigin {
    crate::SourceOrigin {
        workspace: WorkspaceId::from_raw(1).unwrap(),
        path: "compiler-case.jai".into(),
        start: 8,
        end: 24,
        body_hash: 0,
        body: b"#run code_case()".to_vec(),
        specialization: vec![],
    }
}
fn start_waiting(vm: &mut Vm<'_, Provider, &mut Effects>, p: &FixturePlan) {
    assert_eq!(
        vm.start_resumable_compiler_code(&p.plan, owner(), origin())
            .outcome,
        ResumableOutcome::Suspended(vec![Dependency::Effect(EffectKey(42))])
    );
    vm.effects_mut().ready = true;
    assert_eq!(
        vm.resume_resumable().outcome,
        ResumableOutcome::Suspended(vec![Dependency::Procedure(wait())])
    );
    let pointer = vm.globals[0].as_ref().unwrap();
    assert_eq!(
        vm.memory.load(vm.provider.types(), pointer).unwrap(),
        Value::Int(Integer::wrapping(IntegerType::S64, 42))
    );
    assert!(vm.effects().finishes.is_empty());
    assert_eq!(vm.limits.value_cells, Limits::default().value_cells);
    let limit = vm.limits.value_cells;
    assert_eq!(vm.memory.replace_value_cell_limit(limit).unwrap(), limit);
}
fn finish_capture(
    vm: &mut Vm<'_, Provider, &mut Effects>,
    p: &FixturePlan,
) -> (CompilerFrameId, u64) {
    assert!(vm.resumable_values().is_none());
    let rejected =
        vm.finish_resumable_validated(|_, _| panic!("Code must not enter native publication"));
    assert!(matches!(
        rejected.outcome,
        Outcome::Failed(Error::InvalidIr(_))
    ));
    assert!(vm.continuation.is_some());
    assert!(vm.effects().finishes.is_empty());
    let pointer = vm.globals[0].as_ref().unwrap();
    assert_eq!(
        vm.memory.load(vm.provider.types(), pointer).unwrap(),
        Value::Int(Integer::wrapping(IntegerType::S64, 42))
    );
    vm.finish_resumable_compiler_code(|_, selection| {
        assert_eq!(selection.site(), p.yes);
        assert_eq!(selection.publication_occurrence(), p.yes.index() as u64 + 1);
        assert_ne!(selection.site(), p.no);
        assert_eq!(selection.frame().plan(), p.plan.id());
        assert_eq!(
            selection.native_value(p.word)?.1,
            &Value::Int(Integer::wrapping(IntegerType::S64, 47))
        );
        assert_eq!(
            selection.native_value(p.workspace)?.1,
            &Value::Int(Integer::wrapping(IntegerType::U64, 7))
        );
        Ok((selection.frame(), selection.publication_occurrence()))
    })
    .unwrap()
}
#[test]
fn effects_locals_condition_and_selected_capture_resume_without_replay() {
    let provider = fixture();
    let p = plan(&provider);
    let mut effects = Effects::default();
    let mut vm = Vm::new(&provider, &mut effects, Limits::default()).unwrap();
    start_waiting(&mut vm, &p);
    provider.wait.set(false);
    assert_eq!(
        vm.resume_resumable().outcome,
        ResumableOutcome::AwaitingPublication
    );
    finish_capture(&mut vm, &p);
    assert_eq!(vm.effects().begins, 1);
    assert_eq!(vm.effects().requests, 1);
    assert_eq!(vm.effects().polls, 1);
    assert_eq!(vm.effects().finishes, vec![true]);
}
#[test]
fn publication_rejection_restores_native_writes_and_aborts_the_one_journal() {
    let provider = fixture();
    let p = plan(&provider);
    let mut effects = Effects::default();
    let mut vm = Vm::new(&provider, &mut effects, Limits::default()).unwrap();
    start_waiting(&mut vm, &p);
    provider.wait.set(false);
    vm.resume_resumable();
    let result: std::result::Result<(), Error> =
        vm.finish_resumable_compiler_code(|_, _| Err(Error::InvalidIr("rejected source graph")));
    assert!(result.is_err());
    assert!(vm.globals[0].is_none());
    assert!(vm.continuation.is_none());
    assert_eq!(vm.effects().begins, 1);
    assert_eq!(vm.effects().requests, 1);
    assert_eq!(vm.effects().finishes, vec![false]);
}
#[test]
fn detached_compiler_cursor_keeps_slots_origin_and_journal() {
    let provider = fixture();
    let p = plan(&provider);
    let mut effects = Effects::default();
    let mut vm = Vm::new(&provider, &mut effects, Limits::default()).unwrap();
    start_waiting(&mut vm, &p);
    let state = vm.into_continuation().unwrap();
    assert_eq!(state.source_origin(), Some(&origin()));
    provider.wait.set(false);
    effects.origin = None;
    let mut vm = Vm::with_continuation(&provider, &mut effects, state).unwrap();
    assert_eq!(
        vm.resume_resumable().outcome,
        ResumableOutcome::AwaitingPublication
    );
    finish_capture(&mut vm, &p);
    assert_eq!(vm.effects().origin.as_ref(), Some(&origin()));
    assert_eq!(vm.effects().begins, 1);
    assert_eq!(vm.effects().requests, 1);
    assert_eq!(vm.effects().finishes, vec![true]);
}
#[test]
fn cancellation_after_a_completed_native_leaf_discards_the_frame_and_journal() {
    let provider = fixture();
    let p = plan(&provider);
    let mut effects = Effects::default();
    let mut vm = Vm::new(&provider, &mut effects, Limits::default()).unwrap();
    start_waiting(&mut vm, &p);
    vm.cancel_resumable().unwrap();
    assert!(vm.globals[0].is_none());
    assert_eq!(vm.expression_bindings.depth(), 0);
    assert_eq!(vm.effects().finishes, vec![false]);
    assert_eq!(vm.effects().requests, 1);
}
#[test]
fn source_validation_wait_retains_the_same_reached_frame() {
    let provider = fixture();
    let p = plan(&provider);
    let mut effects = Effects::default();
    let mut vm = Vm::new(&provider, &mut effects, Limits::default()).unwrap();
    start_waiting(&mut vm, &p);
    provider.wait.set(false);
    vm.resume_resumable();
    let mut frame = None;
    let ty = provider.types().scalar(ScalarType::Int(IntegerType::S64));
    let result: std::result::Result<(), Error> =
        vm.finish_resumable_compiler_code(|_, selection| {
            frame = Some((selection.frame(), selection.publication_occurrence()));
            Err(Error::Type(TypeError::Incomplete(ty)))
        });
    assert_eq!(result, Err(Error::Type(TypeError::Incomplete(ty))));
    assert!(vm.effects().finishes.is_empty());
    assert_eq!(
        vm.resume_resumable().outcome,
        ResumableOutcome::AwaitingPublication
    );
    assert_eq!(Some(finish_capture(&mut vm, &p)), frame);
    assert_eq!(vm.effects().requests, 1);
    assert_eq!(vm.effects().polls, 1);
    assert_eq!(vm.effects().finishes, vec![true]);
}

#[test]
fn logical_publication_occurrence_survives_equivalent_plan_reconstruction() {
    let provider = fixture();
    let first = plan(&provider);
    let second = plan(&provider);
    assert_ne!(first.plan.id(), second.plan.id());
    let mut events = Vec::new();
    for source in [&first, &second] {
        provider.wait.set(true);
        let mut effects = Effects::default();
        let mut vm = Vm::new(&provider, &mut effects, Limits::default()).unwrap();
        start_waiting(&mut vm, source);
        provider.wait.set(false);
        assert_eq!(
            vm.resume_resumable().outcome,
            ResumableOutcome::AwaitingPublication
        );
        events.push(finish_capture(&mut vm, source));
        assert_eq!(vm.effects().begins, 1);
        assert_eq!(vm.effects().finishes, vec![true]);
    }
    assert_ne!(events[0].0, events[1].0);
    assert_eq!(events[0].1, events[1].1);
}
#[test]
fn compiler_publication_counts_frame_rollback_and_flushes_validation_work() {
    let provider = fixture();
    let p = plan(&provider);
    let mut effects = Effects::default();
    let mut vm = Vm::new(&provider, &mut effects, Limits::default()).unwrap();
    start_waiting(&mut vm, &p);
    provider.wait.set(false);
    assert_eq!(
        vm.resume_resumable().outcome,
        ResumableOutcome::AwaitingPublication
    );
    let before = vm.statistics.steps;
    let remaining = vm
        .finish_resumable_compiler_code(|vm, selection| {
            assert_eq!(selection.publication_occurrence(), p.yes.index() as u64 + 1);
            assert_eq!(vm.publication_source_origin(), Some(&origin()));
            assert_eq!(
                vm.publication_value_cell_limit(),
                Limits::default().value_cells
            );
            let session = vm.continuation.as_ref().unwrap();
            let root_and_rollback = session
                .root
                .retained_cells()?
                .checked_add(session.compiler_reserved_cells)
                .ok_or(Error::Limit(LimitKind::ValueCells))?;
            let retained = vm.publication_retained_cells()?;
            assert!(retained >= root_and_rollback + vm.memory.value_cells());
            let remaining = vm.publication_remaining_value_cells()?;
            assert_eq!(remaining + retained, vm.publication_value_cell_limit());
            vm.charge_publication_work(11)?;
            Ok(remaining)
        })
        .unwrap();
    assert!(remaining < Limits::default().value_cells);
    assert!(vm.statistics.steps >= before + 11);
    assert_eq!(vm.publication_work.get(), 0);
    assert!(vm.transaction_value_cell_limit.is_none());
    assert!(vm.publication_origin.is_none());
    assert_eq!(vm.effects().finishes, vec![true]);
}

#[test]
fn compile_admission_fails_before_any_effect_transaction() {
    let provider = fixture();
    let p = plan(&provider);
    let mut effects = Effects::default();
    let limits = Limits {
        value_cells: 4,
        ..Limits::default()
    };
    let mut vm = Vm::new(&provider, &mut effects, limits).unwrap();
    assert!(matches!(
        vm.start_resumable_compiler_code(&p.plan, owner(), origin())
            .outcome,
        ResumableOutcome::Failed(Error::Limit(LimitKind::ValueCells))
    ));
    assert_eq!(vm.effects().begins, 0);
    assert_eq!(vm.effects().requests, 0);
}

#[test]
fn selected_capture_rejects_initialized_uncaptured_and_foreign_slots() {
    let provider = fixture();
    let mut b = CompilerCodePlanBuilder::new(Default::default()).unwrap();
    let ty = provider.types().scalar(ScalarType::Int(IntegerType::S64));
    let visible = b.slot(ty).unwrap();
    let hidden = b.slot(ty).unwrap();
    let a = b
        .assign(
            visible,
            CompilerRuntimeLeaf::expression(ValueExpr::Int(int(3))),
        )
        .unwrap();
    let h = b
        .assign(
            hidden,
            CompilerRuntimeLeaf::expression(ValueExpr::Int(int(9))),
        )
        .unwrap();
    let site = b.reserve_return_site_with_captures(vec![visible]).unwrap();
    let ret = b.return_code(site).unwrap();
    let root = b.block(vec![a, h, ret]).unwrap();
    let p = b.finish(root).unwrap();
    let mut foreign = CompilerCodePlanBuilder::new(Default::default()).unwrap();
    let foreign_slot = foreign.slot(ty).unwrap();
    let mut effects = Effects::default();
    let mut vm = Vm::new(&provider, &mut effects, Limits::default()).unwrap();
    assert_eq!(
        vm.start_resumable_compiler_code(&p, owner(), origin())
            .outcome,
        ResumableOutcome::AwaitingPublication
    );
    vm.finish_resumable_compiler_code(|_, selection| {
        assert_eq!(
            selection.native_value(visible)?.1,
            &Value::Int(Integer::wrapping(IntegerType::S64, 3))
        );
        assert!(selection.native_value(hidden).is_err());
        assert!(selection.native_value(foreign_slot).is_err());
        Ok(())
    })
    .unwrap();
    assert_eq!(vm.effects().requests, 0);
    assert_eq!(vm.effects().finishes, vec![true]);
}
#[test]
fn untaken_compiler_branch_never_runs_its_effect_leaf() {
    let provider = fixture();
    let mut b = CompilerCodePlanBuilder::new(Default::default()).unwrap();
    let yes = b.reserve_return_site().unwrap();
    let no = b.reserve_return_site().unwrap();
    let effect = b
        .evaluate(CompilerRuntimeLeaf::call(Call::new(
            create(),
            vec![(
                ParameterId::new(0),
                ValueExpr::StringBytes {
                    ty: provider.string,
                    bytes: b"untaken".to_vec(),
                },
            )],
        )))
        .unwrap();
    let y = b.return_code(yes).unwrap();
    let n = b.return_code(no).unwrap();
    let effect_branch = b.block(vec![effect, n]).unwrap();
    let root = b
        .branch(ValueExpr::Bool(BoolExpr::Constant(true)), y, effect_branch)
        .unwrap();
    let p = b.finish(root).unwrap();
    let mut effects = Effects::default();
    let mut vm = Vm::new(&provider, &mut effects, Limits::default()).unwrap();
    assert_eq!(
        vm.start_resumable_compiler_code(&p, owner(), origin())
            .outcome,
        ResumableOutcome::AwaitingPublication
    );
    vm.finish_resumable_compiler_code(|_, selection| {
        assert_eq!(selection.site(), yes);
        Ok(())
    })
    .unwrap();
    assert_eq!(vm.effects().begins, 1);
    assert_eq!(vm.effects().requests, 0);
    assert_eq!(vm.effects().finishes, vec![true]);
}

#[test]
fn unloaded_global_checkpoint_owners_are_admitted_before_journal_begin() {
    let provider = fixture_with_globals(128);
    let p = plan(&provider);
    let proof = p
        .plan
        .verify(
            provider.types(),
            provider.signatures(),
            provider.globals(),
            provider.places().unwrap(),
            owner(),
            None,
        )
        .unwrap();
    let controller = CompilerController::compile(proof, Limits::default()).unwrap();
    let root_cells = controller.retained_cells().unwrap();
    let limits = Limits {
        value_cells: root_cells + 128,
        ..Limits::default()
    };
    let mut effects = Effects::default();
    let mut vm = Vm::new(&provider, &mut effects, limits).unwrap();
    assert!(vm.globals.iter().all(Option::is_none));
    assert_eq!(vm.memory.value_cells(), 0);
    assert_eq!(
        vm.start_resumable_compiler_code(&p.plan, owner(), origin())
            .outcome,
        ResumableOutcome::Failed(Error::Limit(LimitKind::ValueCells))
    );
    assert_eq!(vm.effects().begins, 0);
    assert_eq!(vm.effects().requests, 0);
    assert!(vm.effects().finishes.is_empty());
    assert!(vm.continuation.is_none());
    assert_eq!(vm.memory.allocation_count(), 0);
    assert!(vm.globals.iter().all(Option::is_none));
}

#[test]
fn runtime_phase_rejects_compiler_control_before_any_effect_transaction() {
    let provider = fixture();
    let p = plan(&provider);
    let mut effects = Effects::default();
    let mut vm = Vm::new_with_execution_phase(
        &provider,
        &mut effects,
        Limits::default(),
        crate::ByteTarget::default(),
        crate::ExecutionPhase::Runtime,
    )
    .unwrap();
    assert_eq!(
        vm.start_resumable_compiler_code(&p.plan, owner(), origin())
            .outcome,
        ResumableOutcome::Failed(Error::InvalidIr(
            "compiler Code plans require compile-time execution"
        ))
    );
    assert_eq!(vm.effects().begins, 0);
    assert_eq!(vm.effects().requests, 0);
    assert!(vm.effects().finishes.is_empty());
    assert!(vm.continuation.is_none());
}
