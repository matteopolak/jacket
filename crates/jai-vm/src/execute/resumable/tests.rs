use super::*;
use jai_types::{CallingConvention, ContextMode, IntegerType, ScalarType, TypeRegistry, Variadic};
use std::cell::Cell;

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
fn int(value: i128) -> IntExpr {
    IntExpr::constant(Integer::wrapping(IntegerType::S64, value))
}
fn ret(expression: IntExpr, cleanups: Vec<CleanupId>) -> Statement {
    Statement::Exit(Exit {
        cleanups,
        transfer: Transfer::ReturnInt(expression),
    })
}
fn block(statements: Vec<Statement>, flow: Flow) -> Block {
    Block {
        statements,
        flow,
    }
}
struct Provider {
    library: Library,
    pending: Cell<bool>,
    compiler: bool,
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
        if id == ProcedureId::new(1) && self.pending.get() {
            return ProcedureAvailability::Pending(Dependency::Procedure(id));
        }
        if self.compiler && matches!(id.index(), 2 | 3) {
            return ProcedureAvailability::Compiler(CompilerProcedure {
                signature: self.library.signatures()[&id],
                intrinsic: if id.index() == 2 {
                    crate::CompilerIntrinsic::CreateWorkspace
                } else {
                    crate::CompilerIntrinsic::Message(crate::MessageLevel::Info)
                },
            });
        }
        self.library
            .checked_procedure(id)
            .map_or(ProcedureAvailability::Missing, ProcedureAvailability::Ready)
    }
}
fn fixture(compiler: bool) -> Provider {
    let mut types = TypeRegistry::new();
    let word = types.scalar(ScalarType::Int(IntegerType::S64));
    let wide = types.scalar(ScalarType::Int(IntegerType::U64));
    let string = types.string();
    let main_id = ProcedureId::new(0);
    let wait_id = ProcedureId::new(1);
    let scalar = signature(&mut types, vec![], vec![word]);
    let create = signature(&mut types, vec![string], vec![wide]);
    let message = signature(&mut types, vec![string], vec![]);
    let global = Global::new(
        0,
        GlobalInitializer::Int(Integer::wrapping(IntegerType::S64, 0)),
        &types,
    );
    let place = IntPlace::try_from_place(global.place(), &types).unwrap();
    let local = Local::new(main_id, 0, ScalarType::Int(IntegerType::S64), &types);
    let local_place = IntPlace::try_from_place(local.place(), &types).unwrap();
    let increment = || {
        Statement::StoreInt(
            place,
            IntExpr::new(
                IntegerType::S64,
                IntExprKind::Binary(IntOp::Add, Box::new(IntExpr::load(place)), Box::new(int(1))),
            ),
        )
    };
    let waiting = if compiler {
        IntExpr::new(
            IntegerType::S64,
            IntExprKind::Cast(
                CastMode::Checked,
                Box::new(IntExpr::new(
                    IntegerType::U64,
                    IntExprKind::Call(Call::new(
                        ProcedureId::new(2),
                        vec![(
                            ParameterId::new(0),
                            ValueExpr::StringBytes {
                                ty: string,
                                bytes: b"child".to_vec(),
                            },
                        )],
                    )),
                )),
            ),
        )
    } else {
        IntExpr::new(
            IntegerType::S64,
            IntExprKind::Call(Call::new(wait_id, vec![])),
        )
    };
    let mut statements = vec![increment(), Statement::StoreInt(local_place, int(21))];
    if compiler {
        statements.push(Statement::CallVoid(Call::new(
            ProcedureId::new(3),
            vec![(
                ParameterId::new(0),
                ValueExpr::StringBytes {
                    ty: string,
                    bytes: b"once".to_vec(),
                },
            )],
        )));
    }
    statements.push(ret(
        IntExpr::new(
            IntegerType::S64,
            IntExprKind::Binary(
                IntOp::Add,
                Box::new(IntExpr::load(local_place)),
                Box::new(waiting),
            ),
        ),
        vec![CleanupId::new(0)],
    ));
    let procedures = vec![
        Procedure {
            id: main_id,
            signature: scalar,
            parameters: vec![],
            locals: vec![local],
            body: block(statements, Flow::Terminates),
            cleanups: vec![block(vec![increment()], Flow::FallsThrough).into()],
        },
        Procedure {
            id: wait_id,
            signature: scalar,
            parameters: vec![],
            locals: vec![],
            body: block(vec![ret(int(42), vec![])], Flow::Terminates),
            cleanups: vec![],
        },
    ];
    let mut builder = ProgramBuilder::new(types.freeze().unwrap())
        .procedures(procedures)
        .globals(vec![global]);
    if compiler {
        builder = builder.prototypes(vec![
            ProcedurePrototype {
                id: ProcedureId::new(2),
                signature: create,
                origin: PrototypeOrigin::Compiler,
            },
            ProcedurePrototype {
                id: ProcedureId::new(3),
                signature: message,
                origin: PrototypeOrigin::Compiler,
            },
        ]);
    }
    Provider {
        library: builder.finish_library().unwrap(),
        pending: Cell::new(!compiler),
        compiler,
    }
}
fn global_value(vm: &mut Vm<'_, Provider, impl CompilerEffects>) -> Value {
    let pointer = vm.globals[0].as_ref().unwrap();
    vm.memory.load(vm.provider.types(), pointer).unwrap()
}

fn binding_fixture(provider: &Provider) -> ValueExpr {
    let owner = ProcedureId::new(0);
    let word = provider.types().scalar(ScalarType::Int(IntegerType::S64));
    let first = ExpressionBindingId::new(owner, 0);
    let second = ExpressionBindingId::new(owner, 1);
    let read = |binding| {
        IntExpr::new(
            IntegerType::S64,
            IntExprKind::Value(Box::new(ValueExpr::Bound {
                binding,
                ty: word,
            })),
        )
    };
    ValueExpr::Bind {
        ty: word,
        bindings: vec![
            (first, ValueExpr::Int(int(7))),
            (
                second,
                ValueExpr::Int(IntExpr::new(
                    IntegerType::S64,
                    IntExprKind::Call(Call::new(owner, vec![])),
                )),
            ),
        ],
        body: Box::new(ValueExpr::Int(IntExpr::new(
            IntegerType::S64,
            IntExprKind::Binary(IntOp::Add, Box::new(read(first)), Box::new(read(second))),
        ))),
    }
}

#[test]
fn bindings_survive_nested_suspend_and_detached_transfer_without_repeating_producers() {
    let provider = fixture(false);
    let expression = binding_fixture(&provider);
    let mut vm = Vm::new(&provider, crate::NoEffects, Limits::default()).unwrap();
    assert!(matches!(
        vm.start_resumable_expression(&expression).outcome,
        ResumableOutcome::Failed(Error::IrValidation(_))
    ));
    assert_eq!(
        vm.start_resumable_expression_owned(ProcedureId::new(0), &expression)
            .outcome,
        ResumableOutcome::Suspended(vec![Dependency::Procedure(ProcedureId::new(1))])
    );
    assert_eq!(vm.expression_bindings.depth(), 1);
    assert_eq!(vm.expression_bindings.cells(), 3);
    assert_eq!(
        global_value(&mut vm),
        Value::Int(Integer::wrapping(IntegerType::S64, 1))
    );
    let state = vm.into_continuation().unwrap();
    provider.pending.set(false);
    let mut vm = Vm::with_continuation(&provider, crate::NoEffects, state).unwrap();
    assert_eq!(
        vm.resume_resumable().outcome,
        ResumableOutcome::AwaitingPublication
    );
    assert_eq!(vm.expression_bindings.depth(), 0);
    assert_eq!(
        global_value(&mut vm),
        Value::Int(Integer::wrapping(IntegerType::S64, 2))
    );
    assert_eq!(vm.statistics.calls, 2);
    assert_eq!(
        vm.finish_resumable_validated(|_, _| Ok(())).outcome,
        Outcome::Complete(vec![Value::Int(Integer::wrapping(IntegerType::S64, 70))])
    );
}

#[test]
fn ordinary_owned_bindings_run_producers_once_and_restore_after_publication_failure() {
    let provider = fixture(false);
    provider.pending.set(false);
    let expression = binding_fixture(&provider);
    let mut vm = Vm::new(&provider, crate::NoEffects, Limits::default()).unwrap();
    assert_eq!(
        vm.evaluate_owned_validated(ProcedureId::new(0), &expression, |_, _| Err(
            Error::RuntimeTrap
        ))
        .outcome,
        Outcome::Failed(Error::RuntimeTrap)
    );
    assert_eq!(vm.expression_bindings.depth(), 0);
    assert_eq!(vm.memory.allocation_count(), 0);
    assert_eq!(
        vm.evaluate_owned(ProcedureId::new(0), &expression).outcome,
        Outcome::Complete(vec![Value::Int(Integer::wrapping(IntegerType::S64, 70))])
    );
    assert_eq!(
        global_value(&mut vm),
        Value::Int(Integer::wrapping(IntegerType::S64, 2))
    );
    assert_eq!(vm.statistics.calls, 2);
    assert_eq!(vm.expression_bindings.cells(), 0);
}

#[test]
fn cancellation_retires_capture_scope_and_rolls_back_the_partial_producer() {
    let provider = fixture(false);
    let expression = binding_fixture(&provider);
    let mut vm = Vm::new(&provider, crate::NoEffects, Limits::default()).unwrap();
    assert!(matches!(
        vm.start_resumable_expression_owned(ProcedureId::new(0), &expression)
            .outcome,
        ResumableOutcome::Suspended(_)
    ));
    vm.cancel_resumable().unwrap();
    assert_eq!(vm.expression_bindings.cells(), 0);
    assert_eq!(vm.memory.allocation_count(), 0);
    provider.pending.set(false);
    assert_eq!(
        vm.evaluate_owned(ProcedureId::new(0), &expression).outcome,
        Outcome::Complete(vec![Value::Int(Integer::wrapping(IntegerType::S64, 70))])
    );
}
#[test]
fn scalar_expression_is_uncommitted_until_publication_validation() {
    let provider = fixture(false);
    let mut vm = Vm::new(&provider, crate::NoEffects, Limits::default()).unwrap();
    let expression = ValueExpr::Int(IntExpr::new(
        IntegerType::S64,
        IntExprKind::Binary(IntOp::Add, Box::new(int(20)), Box::new(int(22))),
    ));
    assert_eq!(
        vm.start_resumable_expression(&expression).outcome,
        ResumableOutcome::AwaitingPublication
    );
    assert_eq!(
        vm.resumable_values(),
        Some([Value::Int(Integer::wrapping(IntegerType::S64, 42))].as_slice())
    );
    assert_eq!(
        vm.finish_resumable_validated(|_, _| Ok(())).outcome,
        Outcome::Complete(vec![Value::Int(Integer::wrapping(IntegerType::S64, 42))])
    );
}
#[test]
fn pending_nested_call_retains_locals_left_operand_and_exact_cleanup_progress() {
    let provider = fixture(false);
    let mut vm = Vm::new(&provider, crate::NoEffects, Limits::default()).unwrap();
    assert_eq!(
        vm.start_resumable_procedure(ProcedureId::new(0), vec![])
            .outcome,
        ResumableOutcome::Suspended(vec![Dependency::Procedure(ProcedureId::new(1))])
    );
    assert_eq!(
        global_value(&mut vm),
        Value::Int(Integer::wrapping(IntegerType::S64, 1))
    );
    assert_eq!(vm.frames.len(), 1);
    assert_eq!(vm.statistics.calls, 2);
    provider.pending.set(false);
    assert_eq!(
        vm.resume_resumable().outcome,
        ResumableOutcome::AwaitingPublication
    );
    assert_eq!(
        global_value(&mut vm),
        Value::Int(Integer::wrapping(IntegerType::S64, 2))
    );
    assert_eq!(vm.frames.len(), 0);
    assert_eq!(vm.statistics.calls, 2);
    assert_eq!(
        vm.finish_resumable_validated(|_, _| Ok(())).outcome,
        Outcome::Complete(vec![Value::Int(Integer::wrapping(IntegerType::S64, 63))])
    );
}
#[test]
fn owned_continuation_moves_between_provider_borrows_without_replaying_source() {
    let provider = fixture(false);
    let mut vm = Vm::new(&provider, crate::NoEffects, Limits::default()).unwrap();
    assert!(matches!(
        vm.start_resumable_procedure(ProcedureId::new(0), vec![])
            .outcome,
        ResumableOutcome::Suspended(_)
    ));
    let state = vm.into_continuation().unwrap();
    provider.pending.set(false);
    let mut vm = Vm::with_continuation(&provider, crate::NoEffects, state).unwrap();
    assert_eq!(
        vm.resume_resumable().outcome,
        ResumableOutcome::AwaitingPublication
    );
    assert_eq!(
        global_value(&mut vm),
        Value::Int(Integer::wrapping(IntegerType::S64, 2))
    );
    assert!(matches!(
        vm.finish_resumable_validated(|_, _| Ok(())).outcome,
        Outcome::Complete(_)
    ));
}
#[derive(Default)]
struct Effects {
    begins: usize,
    requests: Vec<crate::CompilerRequest>,
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
    fn request(&mut self, request: crate::CompilerRequest) -> crate::EffectOutcome {
        self.requests.push(request.clone());
        match request {
            crate::CompilerRequest::CreateWorkspace {
                ..
            } => crate::EffectOutcome::Pending(crate::EffectKey(42)),
            crate::CompilerRequest::Message {
                ..
            } => crate::EffectOutcome::Ready(crate::CompilerResponse::Unit),
            _ => panic!("unexpected request"),
        }
    }
    fn poll_request(
        &mut self,
        request: &crate::CompilerRequest,
        key: crate::EffectKey,
    ) -> crate::EffectOutcome {
        assert_eq!(key, crate::EffectKey(42));
        assert!(matches!(
            request,
            crate::CompilerRequest::CreateWorkspace { .. }
        ));
        self.polls += 1;
        if self.ready {
            crate::EffectOutcome::Ready(crate::CompilerResponse::Workspace(
                crate::WorkspaceId::from_raw(42).unwrap(),
            ))
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
#[test]
fn compiler_wait_polls_only_the_retained_leaf_and_publication_rejection_rolls_back() {
    let provider = fixture(true);
    let mut effects = Effects::default();
    let mut vm = Vm::new(&provider, &mut effects, Limits::default()).unwrap();
    assert_eq!(
        vm.start_resumable_procedure(ProcedureId::new(0), vec![])
            .outcome,
        ResumableOutcome::Suspended(vec![Dependency::Effect(crate::EffectKey(42))])
    );
    assert_eq!(vm.effects().begins, 1);
    assert_eq!(vm.effects().requests.len(), 2);
    assert!(vm.effects().finishes.is_empty());
    assert!(matches!(
        vm.resume_resumable().outcome,
        ResumableOutcome::Suspended(_)
    ));
    vm.effects_mut().ready = true;
    assert_eq!(
        vm.resume_resumable().outcome,
        ResumableOutcome::AwaitingPublication
    );
    assert_eq!(vm.effects().requests.len(), 2);
    assert_eq!(vm.effects().polls, 2);
    assert!(vm.effects().finishes.is_empty());
    assert_eq!(
        vm.finish_resumable_validated(|_, _| Err(Error::CheckedCast))
            .outcome,
        Outcome::Failed(Error::CheckedCast)
    );
    assert_eq!(vm.effects().finishes, vec![false]);
    assert_eq!(vm.memory.allocation_count(), 0);
    assert!(vm.globals[0].is_none());
}
#[test]
fn cancel_parked_call_rolls_back_all_staged_storage() {
    let provider = fixture(false);
    let mut vm = Vm::new(&provider, crate::NoEffects, Limits::default()).unwrap();
    assert!(matches!(
        vm.start_resumable_procedure(ProcedureId::new(0), vec![])
            .outcome,
        ResumableOutcome::Suspended(_)
    ));
    vm.cancel_resumable().unwrap();
    assert_eq!(vm.memory.allocation_count(), 0);
    assert!(vm.frames.is_empty());
    assert!(vm.globals[0].is_none());
}

fn origin(label: &str) -> crate::SourceOrigin {
    crate::SourceOrigin {
        workspace: crate::WorkspaceId::from_raw(1).unwrap(),
        path: label.into(),
        start: 0,
        end: 4,
        body_hash: 0,
        body: b"#run".to_vec(),
        specialization: vec![],
    }
}

#[test]
fn exact_source_origin_survives_sibling_service_transfer_and_cancellation() {
    let provider = fixture(true);
    let mut effects = Effects::default();
    let expected = origin("original.jai");
    let mut vm = Vm::new(&provider, &mut effects, Limits::default()).unwrap();
    assert!(matches!(
        vm.start_resumable_procedure_at(ProcedureId::new(0), vec![], expected.clone())
            .outcome,
        ResumableOutcome::Suspended(_)
    ));
    vm.effects_mut().set_source_origin(origin("sibling.jai"));
    assert!(matches!(
        vm.resume_resumable().outcome,
        ResumableOutcome::Suspended(_)
    ));
    assert_eq!(vm.effects().origin.as_ref(), Some(&expected));
    let state = vm.into_continuation().unwrap();
    assert_eq!(state.source_origin(), Some(&expected));
    effects.set_source_origin(origin("another.jai"));
    state.cancel(&mut effects).unwrap();
    assert_eq!(effects.origin.as_ref(), Some(&expected));
    assert_eq!(effects.finishes, vec![false]);
}

struct ChangedLedger<'a> {
    original: &'a Provider,
    signatures: HashMap<ProcedureId, TypeId>,
}
impl ProcedureProvider for ChangedLedger<'_> {
    fn types(&self) -> &dyn TypeView {
        self.original.types()
    }
    fn signatures(&self) -> &HashMap<ProcedureId, TypeId> {
        &self.signatures
    }
    fn globals(&self) -> &[Global] {
        self.original.globals()
    }
    fn places(&self) -> Option<&Places> {
        self.original.places()
    }
    fn procedure(&self, id: ProcedureId) -> ProcedureAvailability<'_> {
        self.original.procedure(id)
    }
}

#[test]
fn detached_continuation_rejects_changed_declared_signatures_and_retires_its_job() {
    let provider = fixture(true);
    let mut effects = Effects::default();
    let expected = origin("job.jai");
    let mut vm = Vm::new(&provider, &mut effects, Limits::default()).unwrap();
    assert!(matches!(
        vm.start_resumable_procedure_at(ProcedureId::new(0), vec![], expected.clone())
            .outcome,
        ResumableOutcome::Suspended(_)
    ));
    let state = vm.into_continuation().unwrap();
    let mut signatures = provider.signatures().clone();
    signatures.insert(
        ProcedureId::new(2),
        provider.signatures()[&ProcedureId::new(0)],
    );
    let changed = ChangedLedger {
        original: &provider,
        signatures,
    };
    assert!(matches!(
        Vm::with_continuation(&changed, &mut effects, state),
        Err(Error::InvalidIr(
            "continuation procedure signature ledger changed"
        ))
    ));
    assert_eq!(effects.origin.as_ref(), Some(&expected));
    assert_eq!(effects.finishes, vec![false]);
}
