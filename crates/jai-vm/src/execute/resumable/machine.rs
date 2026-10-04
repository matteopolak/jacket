//! Explicit expression, block, call and unwind frames. No source replay.
use super::plan::*;
use super::*;
mod fork;
pub(super) mod process_control;
mod readiness;
pub(in crate::execute::resumable) use process_control::DriveStatus;
use process_control::ProcessStop;
#[cfg(test)]
mod tests;

#[derive(Clone)]
struct Task {
    code: Option<Arc<Plan>>,
    depth: usize,
    action: Action,
    cells: usize,
}
#[derive(Clone)]
enum Action {
    ProcessStop(Box<ProcessStop>),
    InitializeContext,
    Eval(NodeId),
    BindNext {
        node: NodeId,
        index: usize,
        scope: bindings::ScopeToken,
    },
    BindEnd(bindings::ScopeToken),
    Collect {
        nodes: Vec<NodeId>,
        index: usize,
        values: Vec<Operand>,
        cells: usize,
        goal: Goal,
    },
    Apply {
        node: NodeId,
        values: Vec<Operand>,
        cells: usize,
        place: bool,
    },
    Choose {
        then_node: NodeId,
        else_node: NodeId,
    },
    Short {
        op: ShortCircuitOp,
        right: NodeId,
    },
    Validate(Option<TypeId>),
    CallCallee(NodeId),
    CallReady {
        node: NodeId,
        id: ProcedureId,
        signature: TypeId,
    },
    Invoke {
        id: ProcedureId,
        signature: Option<TypeId>,
        arguments: Vec<Value>,
        cells: usize,
        expected: Option<TypeId>,
        charged: bool,
        retry_leaf: bool,
    },
    CallBoundary {
        signature: TypeId,
        previous_context: Option<Pointer>,
        expected: Option<TypeId>,
    },
    Block {
        block: BlockId,
        pc: usize,
    },
    Branch {
        then_block: BlockId,
        else_block: BlockId,
    },
    IndexBase {
        node: NodeId,
    },
    IndexFinish {
        node: NodeId,
        pointer: Pointer,
        snapshot: Option<Value>,
    },
    RecordStart {
        node: NodeId,
    },
    RecordNext {
        node: NodeId,
        index: usize,
        record: Value,
    },
    OrderedRecordStart {
        node: NodeId,
    },
    OrderedRecordNext {
        node: NodeId,
        index: usize,
        record: super::super::ordered_records::OrderedRecordState,
    },
    PackNext {
        node: NodeId,
        index: usize,
        pack: super::pack::PackState,
    },
    Store,
    Discard,
    BindResults {
        destinations: Vec<Option<Pointer>>,
        call: NodeId,
    },
    StoreResults(Vec<Option<Pointer>>),
    ExitChain {
        cleanups: Vec<CleanupId>,
        index: usize,
        transfer: TransferCode,
        values: Vec<Value>,
        cells: usize,
    },
    Transfer {
        transfer: TransferCode,
        values: Vec<Value>,
    },
    CleanupRestore {
        previous: Option<Pointer>,
    },
    PushStart {
        id: PushContextId,
        body: BlockId,
    },
    PushRestore {
        id: PushContextId,
        pointer: Pointer,
        previous: Option<Pointer>,
    },
    LoopBoundary(LoopState),
    WhileCheck {
        id: LoopId,
        condition: ConditionCode,
        body: BlockId,
    },
    WhileDecision {
        id: LoopId,
        condition: ConditionCode,
        body: BlockId,
    },
    RangeSetup {
        range: Box<RangeState>,
        start: Number,
        end: Number,
    },
    RangeBody(Box<RangeState>),
    RangeNext(Box<RangeState>),
    CasesNext {
        block: BlockId,
        pc: usize,
        index: usize,
        through: bool,
    },
    CasesDecision {
        block: BlockId,
        pc: usize,
        index: usize,
    },
    SubjectEnd,
    CasesEnd(Flow),
    Simd {
        block: BlockId,
        pc: usize,
        index: usize,
        registers: Vec<Option<Vec<u8>>>,
        reserved: usize,
    },
    SimdAddress {
        block: BlockId,
        pc: usize,
        index: usize,
        registers: Vec<Option<Vec<u8>>>,
        reserved: usize,
    },
    Complete,
}
impl Action {
    // Cache concrete backing headers when an action enters the stack. Payload
    // caches account nested values; these capacities include empty spare slots.
    fn backing_cells(&self) -> std::result::Result<usize, Error> {
        let (first, second, goal) = match self {
            Self::Collect {
                nodes,
                values,
                goal,
                ..
            } => {
                let goal = match goal {
                    Goal::Exit {
                        cleanups, ..
                    } => cleanups.capacity(),
                    Goal::BindResults {
                        destinations, ..
                    } => destinations.capacity(),
                    _ => 0,
                };
                (nodes.capacity(), values.capacity(), goal)
            }
            Self::Apply {
                values, ..
            } => (values.capacity(), 0, 0),
            Self::Invoke {
                arguments, ..
            } => (arguments.capacity(), 0, 0),
            Self::BindResults {
                destinations, ..
            }
            | Self::StoreResults(destinations) => (destinations.capacity(), 0, 0),
            Self::ExitChain {
                cleanups,
                values,
                ..
            } => (cleanups.capacity(), values.capacity(), 0),
            Self::Transfer {
                values, ..
            } => (values.capacity(), 0, 0),
            Self::Simd {
                registers, ..
            }
            | Self::SimdAddress {
                registers, ..
            } => (registers.capacity(), 0, 0),
            _ => (0, 0, 0),
        };
        first
            .checked_add(second)
            .and_then(|cells| cells.checked_add(goal))
            .ok_or(Error::Limit(LimitKind::ValueCells))
    }
}
#[derive(Clone)]
enum Goal {
    Apply {
        node: NodeId,
        place: bool,
    },
    Call {
        node: NodeId,
        id: ProcedureId,
        signature: TypeId,
    },
    Exit {
        cleanups: Vec<CleanupId>,
        transfer: TransferCode,
    },
    Store,
    BindResults {
        destinations: Vec<bool>,
        call: NodeId,
    },
    BoundCondition {
        boolean: bool,
    },
    Range {
        id: LoopId,
        iterator: NodeId,
        ty: jai_types::IntegerType,
        direction: jai_types::Direction,
        body: BlockId,
    },
}
struct Invocation<'a> {
    id: ProcedureId,
    signature: Option<TypeId>,
    arguments: &'a [Value],
    expected: Option<TypeId>,
    charged: bool,
    retry_leaf: bool,
}
#[derive(Clone)]
struct RangeState {
    id: LoopId,
    pointer: Option<Pointer>,
    ty: jai_types::IntegerType,
    end: Option<Number>,
    direction: jai_types::Direction,
    body: BlockId,
}
#[derive(Clone)]
enum LoopState {
    While {
        id: LoopId,
        condition: ConditionCode,
        body: BlockId,
    },
    Range(Box<RangeState>),
}
impl LoopState {
    fn id(&self) -> LoopId {
        match self {
            Self::While {
                id, ..
            } => *id,
            Self::Range(range) => range.id,
        }
    }
}
impl RangeState {
    fn retained_cells(&self) -> usize {
        self.pointer
            .as_ref()
            .map_or(0, Pointer::metadata_cells)
            .saturating_add(self.end.as_ref().map_or(0, Number::metadata_cells))
            .saturating_add(1)
    }
}

#[derive(Clone)]
pub(super) struct Machine {
    tasks: Vec<Task>,
    operands: Vec<Operand>,
    retained: usize,
    plans: HashMap<ProcedureId, Arc<ProcedurePlan>>,
    plan_cells: usize,
    result: Option<Vec<Value>>,
    initial_plan: Option<Arc<Plan>>,
    initialized: bool,
    next_process_stop: u64,
    source_retired: bool,
}
impl Machine {
    pub(super) fn expression(code: Arc<Plan>) -> Self {
        Self {
            tasks: vec![],
            operands: vec![],
            retained: 0,
            plans: HashMap::new(),
            plan_cells: code.metadata_cells,
            result: None,
            initial_plan: Some(code.clone()),
            initialized: false,
            next_process_stop: 1,
            source_retired: false,
        }
    }
    pub(super) fn procedure(id: ProcedureId, arguments: Vec<Value>) -> Self {
        let cells = arguments.iter().fold(arguments.capacity(), |cells, value| {
            cells.saturating_add(value.cells(usize::MAX).unwrap_or(usize::MAX))
        });
        Self {
            tasks: vec![Task {
                code: None,
                depth: 0,
                action: Action::Invoke {
                    id,
                    signature: None,
                    arguments,
                    cells,
                    expected: None,
                    charged: false,
                    retry_leaf: false,
                },
                cells: cells.saturating_add(1),
            }],
            operands: vec![],
            retained: cells.saturating_add(1),
            plans: HashMap::new(),
            plan_cells: 0,
            result: None,
            initial_plan: None,
            initialized: false,
            next_process_stop: 1,
            source_retired: false,
        }
    }
    pub(super) fn values(&self) -> Option<&[Value]> {
        self.result.as_deref()
    }
    pub(super) fn retain_metadata(
        &mut self,
        cells: usize,
        limit: usize,
    ) -> std::result::Result<(), Error> {
        self.plan_cells = self
            .plan_cells
            .checked_add(cells)
            .filter(|cells| cells.saturating_add(self.retained) <= limit)
            .ok_or(Error::Limit(LimitKind::ValueCells))?;
        Ok(())
    }
    pub(super) fn take_values(&mut self) -> Option<Vec<Value>> {
        let values = self.result.take();
        if values.is_some() {
            self.retained = 0;
        }
        values
    }
    fn push<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
        &mut self,
        vm: &Vm<'_, P, E>,
        code: Option<Arc<Plan>>,
        depth: usize,
        action: Action,
        payload: usize,
    ) -> Result<()> {
        let range_cells = match &action {
            Action::RangeSetup {
                range, ..
            }
            | Action::RangeBody(range)
            | Action::RangeNext(range)
            | Action::LoopBoundary(LoopState::Range(range)) => range.retained_cells(),
            _ => 0,
        };
        let cells = payload
            .checked_add(action.backing_cells()?)
            .and_then(|cells| cells.checked_add(range_cells))
            .ok_or(Error::Limit(LimitKind::ValueCells))?
            .checked_add(1)
            .ok_or(Error::Limit(LimitKind::ValueCells))?;
        self.retain_task(
            vm,
            Task {
                code,
                depth,
                action,
                cells,
            },
        )
    }
    fn retain_task<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
        &mut self,
        vm: &Vm<'_, P, E>,
        task: Task,
    ) -> Result<()> {
        self.retained = self
            .retained
            .checked_add(task.cells)
            .filter(|cells| {
                cells
                    .saturating_add(self.plan_cells)
                    .saturating_add(vm.memory.value_cells())
                    .saturating_add(vm.expression_bindings.cells())
                    .saturating_add(
                        vm.processes
                            .as_ref()
                            .map_or(0, process::ProcessState::cells),
                    )
                    <= vm.limits.value_cells
            })
            .ok_or(Error::Limit(LimitKind::ValueCells))?;
        self.tasks.push(task);
        Ok(())
    }
    fn operand<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
        &mut self,
        vm: &Vm<'_, P, E>,
        value: Operand,
    ) -> Result<()> {
        let cells = operand_cells(&value, vm.limits.value_cells)?;
        self.retained = self
            .retained
            .checked_add(cells)
            .filter(|cells| {
                cells
                    .saturating_add(self.plan_cells)
                    .saturating_add(vm.memory.value_cells())
                    .saturating_add(vm.expression_bindings.cells())
                    .saturating_add(
                        vm.processes
                            .as_ref()
                            .map_or(0, process::ProcessState::cells),
                    )
                    <= vm.limits.value_cells
            })
            .ok_or(Error::Limit(LimitKind::ValueCells))?;
        self.operands.push(value);
        Ok(())
    }
    fn pop<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
        &mut self,
        vm: &Vm<'_, P, E>,
    ) -> Result<Operand> {
        let operand = self
            .operands
            .pop()
            .ok_or(Error::InvalidIr("missing continuation operand"))?;
        self.retained = self
            .retained
            .checked_sub(operand_cells(&operand, vm.limits.value_cells)?)
            .ok_or(Error::InvalidIr("continuation cell accounting underflow"))?;
        Ok(operand)
    }
    fn eval<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
        &mut self,
        vm: &Vm<'_, P, E>,
        code: Arc<Plan>,
        node: NodeId,
        depth: usize,
    ) -> Result<()> {
        self.push(vm, Some(code.clone()), depth, Action::Eval(node), 0)
    }
    fn block<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
        &mut self,
        vm: &Vm<'_, P, E>,
        code: Arc<Plan>,
        block: BlockId,
        depth: usize,
    ) -> Result<()> {
        self.push(
            vm,
            Some(code.clone()),
            depth,
            Action::Block {
                block,
                pc: 0,
            },
            0,
        )
    }
    fn collect<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
        &mut self,
        vm: &mut Vm<'_, P, E>,
        code: Arc<Plan>,
        nodes: &[NodeId],
        goal: Goal,
        depth: usize,
    ) -> Result<()> {
        vm.charge_work(nodes.len())?;
        self.push(
            vm,
            Some(code.clone()),
            depth,
            Action::Collect {
                nodes: nodes.to_vec(),
                index: 0,
                values: vec![],
                cells: 0,
                goal,
            },
            nodes.len(),
        )
    }
    #[cfg(test)]
    pub(super) fn drive<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
        &mut self,
        vm: &mut Vm<'_, P, E>,
    ) -> Result<DriveStatus> {
        self.drive_budgeted(vm, |_, _| Ok(()))
    }
    pub(super) fn drive_budgeted<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
        &mut self,
        vm: &mut Vm<'_, P, E>,
        mut admit: impl FnMut(&mut Vm<'_, P, E>, &Machine) -> Result<()>,
    ) -> Result<DriveStatus> {
        admit(vm, self)?;
        if self.source_retired {
            return Ok(DriveStatus::Retired);
        }
        if !self.initialized {
            self.initialized = true;
            if let Some(code) = self.initial_plan.take() {
                vm.charge_work(
                    usize::try_from(code.planning_work)
                        .map_err(|_| Error::Limit(LimitKind::Fuel))?,
                )?;
                self.push(vm, Some(code.clone()), 0, Action::Complete, 0)?;
                match code.entry {
                    Entry::Node(node) => self.eval(vm, code.clone(), node, 0)?,
                    Entry::Block(block) => self.block(vm, code.clone(), block, 0)?,
                }
            } else {
                self.tasks.insert(
                    0,
                    Task {
                        code: None,
                        depth: 0,
                        action: Action::Complete,
                        cells: 1,
                    },
                );
                self.retained = self
                    .retained
                    .checked_add(1)
                    .ok_or(Error::Limit(LimitKind::ValueCells))?;
            }
            self.push(vm, None, 0, Action::InitializeContext, 0)?;
        }
        while !self.tasks.is_empty() {
            admit(vm, self)?;
            if self.process_control().is_some() {
                return Ok(DriveStatus::ProcessControl);
            }
            let task = self.tasks.pop().unwrap();
            self.retained = self
                .retained
                .checked_sub(task.cells)
                .ok_or(Error::InvalidIr("continuation task accounting underflow"))?;
            if let Err(error) = self.prepare_task(vm, &task) {
                if matches!(error, Halt::Pending(_)) {
                    // Readiness preparation is pure with respect to source state:
                    // keep this exact action and its still-unconsumed operands.
                    self.retain_task(vm, task)?;
                }
                return Err(error);
            }
            self.process(vm, task)?;
        }
        if self.result.is_none() {
            return Err(Error::InvalidIr("continuation finished without results").into());
        }
        admit(vm, self)?;
        Ok(DriveStatus::Complete)
    }
    fn process<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
        &mut self,
        vm: &mut Vm<'_, P, E>,
        task: Task,
    ) -> Result<()> {
        let depth = task.depth;
        let code = task.code;
        match task.action {
            Action::ProcessStop(_) => {
                return Err(
                    Error::InvalidIr("stopped process action was dispatched as source").into(),
                );
            }
            Action::InitializeContext => match vm.initialize_context() {
                Err(Halt::Pending(dependency)) => {
                    self.push(vm, None, depth, Action::InitializeContext, 0)?;
                    return Err(Halt::Pending(dependency));
                }
                other => other?,
            },
            Action::Complete => {
                // Move the already-admitted operand into the publication owner.
                // Its cached payload remains retained; no unmetered value walk.
                let operand = self
                    .operands
                    .pop()
                    .ok_or(Error::InvalidIr("missing continuation result operand"))?;
                self.result = Some(operand.into_results()?);
            }
            Action::Eval(node) => {
                vm.step(depth)?;
                let code = code.unwrap();
                let expression = &code.nodes[node.index()];
                match &expression.kind {
                    NodeKind::Bind {
                        bindings, ..
                    } => {
                        if bindings
                            .iter()
                            .any(|(id, _)| Some(id.procedure()) != code.binding_owner)
                        {
                            return Err(
                                Error::InvalidIr("expression binding has another owner").into()
                            );
                        }
                        let scope =
                            vm.begin_bindings(self.retained.saturating_add(self.plan_cells))?;
                        self.push(
                            vm,
                            Some(code.clone()),
                            depth,
                            Action::BindNext {
                                node,
                                index: 0,
                                scope,
                            },
                            0,
                        )?;
                    }
                    NodeKind::Apply {
                        operands, ..
                    } => self.collect(
                        vm,
                        code.clone(),
                        operands,
                        Goal::Apply {
                            node,
                            place: false,
                        },
                        depth + 1,
                    )?,
                    NodeKind::Place {
                        operands, ..
                    } => self.collect(
                        vm,
                        code.clone(),
                        operands,
                        Goal::Apply {
                            node,
                            place: true,
                        },
                        depth + 1,
                    )?,
                    NodeKind::Conditional {
                        condition,
                        then_node,
                        else_node,
                    } => {
                        self.push(
                            vm,
                            Some(code.clone()),
                            depth,
                            Action::Validate(expression.ty),
                            0,
                        )?;
                        self.push(
                            vm,
                            Some(code.clone()),
                            depth,
                            Action::Choose {
                                then_node: *then_node,
                                else_node: *else_node,
                            },
                            0,
                        )?;
                        self.eval(vm, code.clone(), *condition, depth + 1)?;
                    }
                    NodeKind::ShortCircuit {
                        op,
                        left,
                        right,
                    } => {
                        self.push(
                            vm,
                            Some(code.clone()),
                            depth,
                            Action::Short {
                                op: *op,
                                right: *right,
                            },
                            0,
                        )?;
                        self.eval(vm, code.clone(), *left, depth + 1)?;
                    }
                    NodeKind::Call {
                        target,
                        signature,
                        arguments,
                    } => match target {
                        CallTarget::Direct(id) => {
                            let nodes: Vec<_> = arguments.iter().map(|(_, node)| *node).collect();
                            self.collect(
                                vm,
                                code.clone(),
                                &nodes,
                                Goal::Call {
                                    node,
                                    id: *id,
                                    signature: *signature,
                                },
                                depth + 1,
                            )?;
                        }
                        CallTarget::Indirect(callee) => {
                            self.push(vm, Some(code.clone()), depth, Action::CallCallee(node), 0)?;
                            self.eval(vm, code.clone(), *callee, depth + 1)?;
                        }
                    },
                    NodeKind::IndexPlace {
                        base, ..
                    } => {
                        self.push(
                            vm,
                            Some(code.clone()),
                            depth,
                            Action::IndexBase {
                                node,
                            },
                            0,
                        )?;
                        self.eval(vm, code.clone(), *base, depth + 1)?;
                    }
                    NodeKind::OrderedRecord {
                        ..
                    } => self.push(
                        vm,
                        Some(code.clone()),
                        depth,
                        Action::OrderedRecordStart {
                            node,
                        },
                        0,
                    )?,
                    NodeKind::RecordBuild {
                        ..
                    } => self.push(
                        vm,
                        Some(code.clone()),
                        depth,
                        Action::RecordStart {
                            node,
                        },
                        0,
                    )?,
                    NodeKind::SequencePack {
                        ty, ..
                    } => {
                        let pack = super::pack::PackState::new(vm, *ty)?;
                        self.push(
                            vm,
                            Some(code.clone()),
                            depth,
                            Action::PackNext {
                                node,
                                index: 0,
                                pack,
                            },
                            0,
                        )?;
                    }
                }
            }
            Action::Collect {
                nodes,
                index,
                mut values,
                mut cells,
                goal,
            } => {
                let code = code.unwrap();
                if index != 0 {
                    let value = self.pop(vm)?;
                    let child = operand_cells(&value, vm.limits.value_cells)?;
                    cells = cells
                        .checked_add(child)
                        .filter(|cells| *cells <= vm.limits.value_cells)
                        .ok_or(Error::Limit(LimitKind::ValueCells))?;
                    vm.charge_work(child)?;
                    values.push(value);
                }
                if index < nodes.len() {
                    let next = nodes[index];
                    let payload = cells
                        .checked_add(nodes.len())
                        .ok_or(Error::Limit(LimitKind::ValueCells))?;
                    self.push(
                        vm,
                        Some(code.clone()),
                        depth,
                        Action::Collect {
                            nodes,
                            index: index + 1,
                            values,
                            cells,
                            goal,
                        },
                        payload,
                    )?;
                    self.eval(vm, code.clone(), next, depth + 1)?;
                } else {
                    self.collected(vm, code.clone(), goal, values, cells, depth)?;
                }
            }
            Action::BindNext {
                node,
                index,
                scope,
            } => {
                let code = code.unwrap();
                let NodeKind::Bind {
                    bindings,
                    body,
                } = &code.nodes[node.index()].kind
                else {
                    return Err(Error::InvalidIr("invalid continuation binding node").into());
                };
                if index != 0 {
                    let value = self.pop(vm)?.into_value()?;
                    vm.capture_binding(
                        bindings[index - 1].0,
                        value,
                        self.retained.saturating_add(self.plan_cells),
                    )?;
                }
                if index < bindings.len() {
                    self.push(
                        vm,
                        Some(code.clone()),
                        depth,
                        Action::BindNext {
                            node,
                            index: index + 1,
                            scope,
                        },
                        0,
                    )?;
                    self.eval(vm, code.clone(), bindings[index].1, depth + 1)?;
                } else {
                    self.push(vm, Some(code.clone()), depth, Action::BindEnd(scope), 0)?;
                    self.push(
                        vm,
                        Some(code.clone()),
                        depth,
                        Action::Validate(code.nodes[node.index()].ty),
                        0,
                    )?;
                    self.eval(vm, code.clone(), *body, depth + 1)?;
                }
            }
            Action::BindEnd(scope) => vm.end_bindings(scope)?,
            Action::Apply {
                node,
                values,
                cells,
                place,
            } => {
                let code = code.unwrap();
                let expression = &code.nodes[node.index()];
                let copied = clone_operands(vm, &values)?;
                let result = match &expression.kind {
                    NodeKind::Apply {
                        op, ..
                    } if !place => {
                        if let ApplyOp::Bound(binding) = op {
                            vm.bound_value(*binding, self.retained.saturating_add(self.plan_cells))
                                .map(Operand::Value)
                        } else {
                            super::apply::apply(vm, op, copied, depth)
                        }
                    }
                    NodeKind::Place {
                        op, ..
                    } if place => super::apply::apply_place(vm, op, copied, depth),
                    _ => return Err(Error::InvalidIr("invalid continuation apply node").into()),
                };
                match result {
                    Ok(value) => {
                        validate_operand(vm, &value, expression.ty)?;
                        self.operand(vm, value)?;
                    }
                    Err(Halt::Pending(dependency)) => {
                        self.push(
                            vm,
                            Some(code.clone()),
                            depth,
                            Action::Apply {
                                node,
                                values,
                                cells,
                                place,
                            },
                            cells,
                        )?;
                        return Err(Halt::Pending(dependency));
                    }
                    Err(error) => return Err(error),
                }
            }
            Action::Choose {
                then_node,
                else_node,
            } => {
                let condition = self.pop(vm)?.into_value()?.boolean()?;
                self.eval(
                    vm,
                    code.unwrap(),
                    if condition {
                        then_node
                    } else {
                        else_node
                    },
                    depth + 1,
                )?;
            }
            Action::Short {
                op,
                right,
            } => {
                let left = self.pop(vm)?.into_value()?.boolean()?;
                if matches!(op, ShortCircuitOp::And) && !left
                    || matches!(op, ShortCircuitOp::Or) && left
                {
                    self.operand(vm, Operand::Value(Value::Bool(left)))?;
                } else {
                    self.eval(vm, code.unwrap(), right, depth + 1)?;
                }
            }
            Action::Validate(ty) => {
                let value = self.pop(vm)?;
                validate_operand(vm, &value, ty)?;
                self.operand(vm, value)?;
            }
            Action::CallCallee(node) => {
                let code = code.unwrap();
                let Value::Procedure {
                    signature,
                    procedure,
                } = self.pop(vm)?.into_value()?
                else {
                    return Err(Error::InvalidIr(
                        "indirect continuation callee is not a procedure",
                    )
                    .into());
                };
                let id = procedure.ok_or(Error::NullProcedure)?;
                let NodeKind::Call {
                    signature: expected,
                    ..
                } = &code.nodes[node.index()].kind
                else {
                    unreachable!()
                };
                if signature != *expected {
                    return Err(Error::TypeMismatch {
                        expected: *expected,
                    }
                    .into());
                }
                self.push(
                    vm,
                    Some(code),
                    depth,
                    Action::CallReady {
                        node,
                        id,
                        signature,
                    },
                    0,
                )?;
            }
            Action::CallReady {
                node,
                id,
                signature,
            } => {
                vm.step(depth)?;
                let actual = match vm.provider.procedure(id) {
                    ProcedureAvailability::Ready(checked) => {
                        validate_checked_environment(vm, &checked)?;
                        checked.procedure().signature
                    }
                    ProcedureAvailability::Compiler(procedure) => procedure.signature,
                    ProcedureAvailability::Runtime(procedure) => procedure.signature,
                    ProcedureAvailability::FileAbi(procedure) => procedure.signature,
                    ProcedureAvailability::HeapAbi(procedure) => procedure.signature,
                    ProcedureAvailability::ProcessAbi(procedure) => procedure.signature(),
                    ProcedureAvailability::Pending(dependency) => {
                        self.push(
                            vm,
                            code,
                            depth,
                            Action::CallReady {
                                node,
                                id,
                                signature,
                            },
                            0,
                        )?;
                        return Err(Halt::Pending(dependency));
                    }
                    ProcedureAvailability::Failed(error) => return Err(error.into()),
                    ProcedureAvailability::Missing => {
                        return Err(Error::MissingProcedure(id).into());
                    }
                    ProcedureAvailability::Foreign => {
                        return Err(Error::UnsupportedForeignProcedure(id).into());
                    }
                };
                vm.validate_provided_signature(id, actual)?;
                if actual != signature {
                    return Err(Error::TypeMismatch {
                        expected: signature,
                    }
                    .into());
                }
                let code = code.unwrap();
                let NodeKind::Call {
                    arguments, ..
                } = &code.nodes[node.index()].kind
                else {
                    unreachable!()
                };
                let nodes: Vec<_> = arguments.iter().map(|(_, node)| *node).collect();
                self.collect(
                    vm,
                    code.clone(),
                    &nodes,
                    Goal::Call {
                        node,
                        id,
                        signature,
                    },
                    depth + 1,
                )?;
            }
            Action::Invoke {
                id,
                signature,
                arguments,
                cells,
                expected,
                charged,
                retry_leaf,
            } => {
                let result = self.invoke(
                    vm,
                    Invocation {
                        id,
                        signature,
                        arguments: &arguments,
                        expected,
                        charged,
                        retry_leaf,
                    },
                    depth,
                );
                if let Err(Halt::Pending(dependency)) = result {
                    let retry_leaf =
                        !matches!(&dependency, Dependency::Process(_)) && vm.effects.has_journal();
                    self.push(
                        vm,
                        code.clone(),
                        depth,
                        Action::Invoke {
                            id,
                            signature,
                            arguments,
                            cells,
                            expected,
                            charged: true,
                            retry_leaf,
                        },
                        cells,
                    )?;
                    return Err(Halt::Pending(dependency));
                }
                result?;
            }
            Action::CallBoundary {
                signature,
                previous_context,
                expected,
            } => {
                if !vm.signature(signature)?.results.is_empty() {
                    return Err(
                        Error::InvalidIr("value procedure fell through without returning").into(),
                    );
                }
                self.finish_call(vm, signature, previous_context, expected, vec![])?;
            }
            Action::Block {
                block,
                pc,
            } => {
                let code = code.unwrap();
                let block_code = &code.blocks[block.index()];
                if pc == block_code.statements.len() {
                    if block_code.flow == Flow::Terminates {
                        return Err(Error::InvalidIr(
                            "terminating continuation block fell through",
                        )
                        .into());
                    }
                    return Ok(());
                }
                vm.step(depth)?;
                self.push(
                    vm,
                    Some(code.clone()),
                    depth,
                    Action::Block {
                        block,
                        pc: pc + 1,
                    },
                    0,
                )?;
                self.statement(vm, code.clone(), block, pc, depth + 1)?;
            }
            Action::Branch {
                then_block,
                else_block,
            } => {
                let condition = self.pop(vm)?.into_value()?.boolean()?;
                self.block(
                    vm,
                    code.unwrap(),
                    if condition {
                        then_block
                    } else {
                        else_block
                    },
                    depth + 1,
                )?;
            }
            Action::IndexBase {
                node,
            } => {
                let code = code.unwrap();
                let NodeKind::IndexPlace {
                    index,
                    base_type,
                    ..
                } = code.nodes[node.index()].kind
                else {
                    unreachable!()
                };
                let pointer = self.pop(vm)?.into_place()?;
                let snapshot =
                    super::apply::snapshot_index_base(vm, &pointer, base_type, depth + 1)?;
                let cells = pointer
                    .metadata_cells()
                    .checked_add(
                        snapshot
                            .as_ref()
                            .map_or(Ok(0), |value| value.cells(vm.limits.value_cells))?,
                    )
                    .ok_or(Error::Limit(LimitKind::ValueCells))?;
                self.push(
                    vm,
                    Some(code.clone()),
                    depth,
                    Action::IndexFinish {
                        node,
                        pointer,
                        snapshot,
                    },
                    cells,
                )?;
                self.eval(vm, code.clone(), index, depth + 1)?;
            }
            Action::IndexFinish {
                node,
                pointer,
                snapshot,
            } => {
                let code = code.unwrap();
                let NodeKind::IndexPlace {
                    base_type,
                    check,
                    ..
                } = code.nodes[node.index()].kind
                else {
                    unreachable!()
                };
                let index = self.pop(vm)?.into_value()?;
                let ty = code.nodes[node.index()]
                    .ty
                    .ok_or(Error::InvalidIr("index place has no type"))?;
                let pointer =
                    super::apply::index_place(vm, pointer, snapshot, index, base_type, ty, check)?;
                self.operand(vm, Operand::Place(pointer))?;
            }
            Action::RecordStart {
                node,
            } => {
                let code = code.unwrap();
                let NodeKind::RecordBuild {
                    ty, ..
                } = code.nodes[node.index()].kind
                else {
                    unreachable!()
                };
                let record = vm.zero_value(ty)?;
                let cells = record.cells(vm.limits.value_cells)?;
                self.push(
                    vm,
                    Some(code.clone()),
                    depth,
                    Action::RecordNext {
                        node,
                        index: 0,
                        record,
                    },
                    cells,
                )?;
            }
            Action::RecordNext {
                node,
                index,
                mut record,
            } => {
                let code = code.unwrap();
                let NodeKind::RecordBuild {
                    ty,
                    initializers,
                } = &code.nodes[node.index()].kind
                else {
                    unreachable!()
                };
                if index != 0 {
                    let (field, _) = initializers[index - 1];
                    let value = self.pop(vm)?.into_value()?;
                    record = super::apply::replace_record_field(
                        vm,
                        record,
                        *ty,
                        field,
                        value,
                        depth + 1,
                    )?;
                }
                if index == initializers.len() {
                    self.operand(vm, Operand::Value(record))?;
                } else {
                    let next = initializers[index].1;
                    let cells = record.cells(vm.limits.value_cells)?;
                    self.push(
                        vm,
                        Some(code.clone()),
                        depth,
                        Action::RecordNext {
                            node,
                            index: index + 1,
                            record,
                        },
                        cells,
                    )?;
                    self.eval(vm, code.clone(), next, depth + 1)?;
                }
            }
            Action::OrderedRecordStart {
                node,
            } => {
                let code = code.unwrap();
                let NodeKind::OrderedRecord {
                    ty,
                    backing,
                    initializers,
                } = &code.nodes[node.index()].kind
                else {
                    unreachable!()
                };
                let record = vm.start_ordered_record(
                    *ty,
                    *backing,
                    initializers.iter().map(|(path, _)| path.as_ref()),
                    self.retained.saturating_add(self.plan_cells),
                )?;
                let cells = record.cells(vm.limits.value_cells)?;
                self.push(
                    vm,
                    Some(code.clone()),
                    depth,
                    Action::OrderedRecordNext {
                        node,
                        index: 0,
                        record,
                    },
                    cells,
                )?;
            }
            Action::OrderedRecordNext {
                node,
                index,
                mut record,
            } => {
                let code = code.unwrap();
                let NodeKind::OrderedRecord {
                    initializers, ..
                } = &code.nodes[node.index()].kind
                else {
                    unreachable!()
                };
                if index != 0 {
                    let value = self.pop(vm)?.into_value()?;
                    vm.write_ordered_record(
                        &mut record,
                        index - 1,
                        &value,
                        self.retained.saturating_add(self.plan_cells),
                    )?;
                }
                if index == initializers.len() {
                    let value = vm.finish_ordered_record(
                        record,
                        self.retained.saturating_add(self.plan_cells),
                    )?;
                    self.operand(vm, Operand::Value(value))?;
                } else {
                    let next = initializers[index].1;
                    let cells = record.cells(vm.limits.value_cells)?;
                    self.push(
                        vm,
                        Some(code.clone()),
                        depth,
                        Action::OrderedRecordNext {
                            node,
                            index: index + 1,
                            record,
                        },
                        cells,
                    )?;
                    self.eval(vm, code.clone(), next, depth + 1)?;
                }
            }
            Action::PackNext {
                node,
                index,
                mut pack,
            } => {
                let code = code.unwrap();
                let NodeKind::SequencePack {
                    parts, ..
                } = &code.nodes[node.index()].kind
                else {
                    unreachable!()
                };
                if index != 0 {
                    let value = self.pop(vm)?;
                    pack.capture(vm, parts[index - 1].mode, value, depth + 1)?;
                }
                if index == parts.len() {
                    let value = pack.finish(vm)?;
                    self.operand(vm, Operand::Value(value))?;
                } else {
                    let next = parts[index].node;
                    let cells = pack.cells();
                    self.push(
                        vm,
                        Some(code.clone()),
                        depth,
                        Action::PackNext {
                            node,
                            index: index + 1,
                            pack,
                        },
                        cells,
                    )?;
                    self.eval(vm, code.clone(), next, depth + 1)?;
                }
            }
            Action::Store => {
                let value = self.pop(vm)?.into_value()?;
                let destination = self.pop(vm)?.into_place()?;
                vm.store_pointer(&destination, value, depth + 1)?;
            }
            Action::Discard => {
                self.pop(vm)?;
            }
            Action::BindResults {
                destinations,
                call,
            } => {
                self.push(
                    vm,
                    code.clone(),
                    depth,
                    Action::StoreResults(destinations),
                    0,
                )?;
                self.eval(vm, code.unwrap(), call, depth + 1)?;
            }
            Action::StoreResults(destinations) => {
                let values = self.pop(vm)?.into_results()?;
                if values.len() != destinations.len() {
                    return Err(Error::InvalidIr("result binding arity differs").into());
                }
                for (pointer, value) in destinations.into_iter().zip(values) {
                    if let Some(pointer) = pointer {
                        vm.store_pointer(&pointer, value, depth + 1)?;
                    }
                }
            }
            Action::ExitChain {
                cleanups,
                index,
                transfer,
                values,
                cells,
            } => {
                let code = code.unwrap();
                if index == cleanups.len() {
                    self.push(
                        vm,
                        Some(code.clone()),
                        depth,
                        Action::Transfer {
                            transfer,
                            values,
                        },
                        cells,
                    )?;
                } else {
                    let cleanup = cleanups[index];
                    self.push(
                        vm,
                        Some(code.clone()),
                        depth,
                        Action::ExitChain {
                            cleanups,
                            index: index + 1,
                            transfer,
                            values,
                            cells,
                        },
                        cells,
                    )?;
                    self.cleanup(vm, code.clone(), cleanup, depth + 1)?;
                }
            }
            Action::Transfer {
                transfer,
                values,
            } => self.transfer(vm, transfer, values, depth)?,
            Action::CleanupRestore {
                previous,
            } => {
                vm.current_context = previous;
            }
            Action::PushStart {
                id,
                body,
            } => {
                let value = self.pop(vm)?.into_value()?;
                let ty = vm
                    .provider
                    .context()
                    .ok_or(Error::InvalidIr("push context schema missing"))?
                    .record_type;
                vm.prepare_layout(ty)?;
                value.validate(vm.provider.types(), ty, vm.limits.evaluation_depth.min(256))?;
                let pointer = vm.memory.allocate(vm.provider.types(), ty, Some(value))?;
                let frame = vm
                    .frames
                    .last_mut()
                    .ok_or(Error::InvalidIr("push context requires a frame"))?;
                if id.procedure() != frame.procedure.id
                    || frame.push_contexts.insert(id, pointer.clone()).is_some()
                {
                    return Err(Error::InvalidIr("push context identity already active").into());
                }
                let previous = vm.current_context.replace(pointer.clone());
                self.push(
                    vm,
                    code.clone(),
                    depth,
                    Action::PushRestore {
                        id,
                        pointer,
                        previous,
                    },
                    0,
                )?;
                self.block(vm, code.unwrap(), body, depth + 1)?;
            }
            Action::PushRestore {
                id,
                pointer,
                previous,
            } => {
                vm.current_context = previous;
                vm.frames
                    .last_mut()
                    .ok_or(Error::InvalidIr("push context frame missing"))?
                    .push_contexts
                    .remove(&id);
                vm.memory.release(&pointer)?;
            }
            Action::LoopBoundary(_) => {
                return Err(Error::InvalidIr("loop boundary reached without a decision").into());
            }
            Action::WhileCheck {
                id,
                condition,
                body,
            } => {
                self.push(
                    vm,
                    code.clone(),
                    depth,
                    Action::WhileDecision {
                        id,
                        condition,
                        body,
                    },
                    0,
                )?;
                match condition {
                    ConditionCode::Value(node) => self.eval(vm, code.unwrap(), node, depth + 1)?,
                    ConditionCode::BoundInt {
                        destination,
                        value,
                    }
                    | ConditionCode::BoundBool {
                        destination,
                        value,
                    } => self.collect(
                        vm,
                        code.unwrap(),
                        &[destination, value],
                        Goal::BoundCondition {
                            boolean: matches!(condition, ConditionCode::BoundBool { .. }),
                        },
                        depth + 1,
                    )?,
                }
            }
            Action::WhileDecision {
                id,
                condition,
                body,
            } => {
                if self.pop(vm)?.into_value()?.boolean()? {
                    self.push(
                        vm,
                        code.clone(),
                        depth,
                        Action::WhileCheck {
                            id,
                            condition,
                            body,
                        },
                        0,
                    )?;
                    self.block(vm, code.unwrap(), body, depth + 1)?;
                } else {
                    self.end_loop(vm, id)?;
                }
            }
            Action::RangeSetup {
                mut range,
                start,
                end,
            } => {
                let pointer = self.pop(vm)?.into_place()?;
                let reverse = range.direction == jai_types::Direction::Reverse;
                let (first, last) = if reverse {
                    (end.clone(), start.clone())
                } else {
                    (start.clone(), end.clone())
                };
                vm.store_pointer(&pointer, first.into_value(), depth + 1)?;
                if start.value() <= end.value() {
                    range.pointer = Some(pointer);
                    range.end = Some(last);
                    self.push(
                        vm,
                        code.clone(),
                        depth,
                        Action::LoopBoundary(LoopState::Range(range.clone())),
                        0,
                    )?;
                    self.push(vm, code.clone(), depth, Action::RangeBody(range), 0)?;
                } else {
                    vm.leave_loop()?;
                }
            }
            Action::RangeBody(range) => {
                self.push(vm, code.clone(), depth, Action::RangeNext(range.clone()), 0)?;
                self.block(vm, code.unwrap(), range.body, depth + 1)?;
            }
            Action::RangeNext(range) => {
                let pointer = range.pointer.as_ref().unwrap();
                vm.prepare_pointer_layouts(pointer, true)?;
                vm.charge_work(vm.memory.load_work_cost(vm.provider.types(), pointer)?)?;
                let current = vm.memory.load(vm.provider.types(), pointer)?.number()?;
                current.portable_integer()?;
                let end = range.end.as_ref().unwrap();
                let reverse = range.direction == jai_types::Direction::Reverse;
                if !reverse && current.value() >= end.value()
                    || reverse && current.value() <= end.value()
                {
                    self.end_loop(vm, range.id)?;
                } else {
                    let next = vm.number_binary(
                        range.ty,
                        IntOp::Add,
                        current,
                        Number::plain(Integer::wrapping(
                            range.ty,
                            if reverse {
                                -1
                            } else {
                                1
                            },
                        )),
                        CheckMode::Disabled,
                    )?;
                    vm.store_pointer(pointer, next.into_value(), depth + 1)?;
                    self.push(vm, code.clone(), depth, Action::RangeBody(range), 0)?;
                }
            }
            Action::CasesNext {
                block,
                pc,
                index,
                through,
            } => {
                let code = code.unwrap();
                let StatementCode::Cases(cases) = &code.blocks[block.index()].statements[pc] else {
                    unreachable!()
                };
                if !through && index < cases.arms.len() {
                    self.push(
                        vm,
                        Some(code.clone()),
                        depth,
                        Action::CasesDecision {
                            block,
                            pc,
                            index,
                        },
                        0,
                    )?;
                    self.eval(vm, code.clone(), cases.arms[index].condition, depth + 1)?;
                } else {
                    use jai_types::{CaseOrder, CaseTarget};
                    let order = CaseOrder::new(
                        cases.arms.len(),
                        cases
                            .default
                            .map(|_| cases.default_position.unwrap_or(cases.arms.len())),
                    )
                    .map_err(|_| Error::InvalidIr("invalid case order"))?;
                    let target = if index == cases.arms.len() {
                        CaseTarget::Default
                    } else {
                        CaseTarget::Arm(index)
                    };
                    let (body, falls_through) = match target {
                        CaseTarget::Arm(index) => {
                            (Some(cases.arms[index].body), cases.arms[index].through)
                        }
                        CaseTarget::Default => (cases.default, cases.default_through),
                        CaseTarget::End => unreachable!(),
                    };
                    if let Some(body) = body {
                        if falls_through {
                            let next = match order
                                .following(target)
                                .map_err(|_| Error::InvalidIr("invalid case successor"))?
                            {
                                CaseTarget::Arm(index) => index,
                                CaseTarget::Default => cases.arms.len(),
                                CaseTarget::End => {
                                    return Err(
                                        Error::InvalidIr("last case cannot fall through").into()
                                    );
                                }
                            };
                            self.push(
                                vm,
                                Some(code.clone()),
                                depth,
                                Action::CasesNext {
                                    block,
                                    pc,
                                    index: next,
                                    through: true,
                                },
                                0,
                            )?;
                        }
                        self.block(vm, code.clone(), body, depth + 1)?;
                    } else if cases.exhaustive {
                        return Err(Error::InvalidIr("exhaustive cases did not match").into());
                    }
                }
            }
            Action::CasesDecision {
                block,
                pc,
                index,
            } => {
                let code = code.unwrap();
                if self.pop(vm)?.into_value()?.boolean()? {
                    self.push(
                        vm,
                        Some(code.clone()),
                        depth,
                        Action::CasesNext {
                            block,
                            pc,
                            index,
                            through: true,
                        },
                        0,
                    )?;
                } else {
                    self.push(
                        vm,
                        Some(code.clone()),
                        depth,
                        Action::CasesNext {
                            block,
                            pc,
                            index: index + 1,
                            through: false,
                        },
                        0,
                    )?;
                }
            }
            Action::SubjectEnd => {}
            Action::CasesEnd(flow) => {
                if flow == Flow::Terminates {
                    return Err(
                        Error::InvalidIr("terminating continuation cases fell through").into(),
                    );
                }
            }
            action @ (Action::Simd {
                ..
            }
            | Action::SimdAddress {
                ..
            }) => {
                return self.simd_task(vm, code.unwrap(), depth, action);
            }
        }
        Ok(())
    }
    fn collected<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
        &mut self,
        vm: &mut Vm<'_, P, E>,
        code: Arc<Plan>,
        goal: Goal,
        mut values: Vec<Operand>,
        cells: usize,
        depth: usize,
    ) -> Result<()> {
        match goal {
            Goal::Apply {
                node,
                place,
            } => self.push(
                vm,
                Some(code.clone()),
                depth,
                Action::Apply {
                    node,
                    values,
                    cells,
                    place,
                },
                cells,
            )?,
            Goal::Call {
                node,
                id,
                signature,
            } => {
                let NodeKind::Call {
                    arguments, ..
                } = &code.nodes[node.index()].kind
                else {
                    unreachable!()
                };
                let mut args: Vec<_> = arguments
                    .iter()
                    .zip(values)
                    .map(|((parameter, _), value)| Ok((parameter.index(), value.into_value()?)))
                    .collect::<Result<_>>()?;
                args.sort_by_key(|(index, _)| *index);
                if args
                    .iter()
                    .enumerate()
                    .any(|(index, (actual, _))| index != *actual)
                {
                    return Err(Error::InvalidIr("call parameter bindings are not dense").into());
                }
                let expected = code.nodes[node.index()].ty;
                self.push(
                    vm,
                    Some(code.clone()),
                    depth,
                    Action::Invoke {
                        id,
                        signature: Some(signature),
                        arguments: args.into_iter().map(|(_, value)| value).collect(),
                        cells,
                        expected,
                        charged: false,
                        retry_leaf: false,
                    },
                    cells,
                )?;
            }
            Goal::Exit {
                cleanups,
                transfer,
            } => {
                let values = values
                    .into_iter()
                    .map(Operand::into_value)
                    .collect::<Result<Vec<_>>>()?;
                self.push(
                    vm,
                    Some(code.clone()),
                    depth,
                    Action::ExitChain {
                        cleanups,
                        index: 0,
                        transfer,
                        values,
                        cells,
                    },
                    cells,
                )?;
            }
            Goal::Store => {
                for value in values {
                    self.operand(vm, value)?;
                }
                self.push(vm, Some(code.clone()), depth, Action::Store, 0)?;
            }
            Goal::BindResults {
                destinations,
                call,
            } => {
                let mut captured = values.into_iter();
                let pointers = destinations
                    .into_iter()
                    .map(|present| {
                        if present {
                            captured
                                .next()
                                .ok_or_else(|| {
                                    Error::InvalidIr("missing result destination").into()
                                })
                                .and_then(Operand::into_place)
                                .map(Some)
                        } else {
                            Ok(None)
                        }
                    })
                    .collect::<Result<Vec<_>>>()?;
                self.push(
                    vm,
                    Some(code.clone()),
                    depth,
                    Action::BindResults {
                        destinations: pointers,
                        call,
                    },
                    cells,
                )?;
            }
            Goal::BoundCondition {
                boolean,
            } => {
                let value = values.pop().unwrap().into_value()?;
                let pointer = values.pop().unwrap().into_place()?;
                let truth = if boolean {
                    value.boolean()?
                } else {
                    vm.number_truth(&value.number()?)?
                };
                vm.store_pointer(&pointer, value, depth + 1)?;
                self.operand(vm, Operand::Value(Value::Bool(truth)))?;
            }
            Goal::Range {
                id,
                iterator,
                ty,
                direction,
                body,
            } => {
                let end = values.pop().unwrap().into_value()?.number()?;
                let start = values.pop().unwrap().into_value()?.number()?;
                start.portable_integer()?;
                end.portable_integer()?;
                if start.ty() != ty || end.ty() != ty {
                    return Err(Error::InvalidIr("range endpoint types differ").into());
                }
                self.push(
                    vm,
                    Some(code.clone()),
                    depth,
                    Action::RangeSetup {
                        range: Box::new(RangeState {
                            id,
                            pointer: None,
                            ty,
                            end: None,
                            direction,
                            body,
                        }),
                        start,
                        end,
                    },
                    cells,
                )?;
                self.eval(vm, code.clone(), iterator, depth + 1)?;
            }
        }
        Ok(())
    }
    fn statement<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
        &mut self,
        vm: &mut Vm<'_, P, E>,
        code: Arc<Plan>,
        block: BlockId,
        pc: usize,
        depth: usize,
    ) -> Result<()> {
        match &code.blocks[block.index()].statements[pc] {
            StatementCode::Store {
                destination,
                value,
            } => self.collect(
                vm,
                code.clone(),
                &[*destination, *value],
                Goal::Store,
                depth,
            )?,
            StatementCode::Discard(node) => {
                self.push(vm, Some(code.clone()), depth, Action::Discard, 0)?;
                self.eval(vm, code.clone(), *node, depth + 1)?;
            }
            StatementCode::CallResults {
                call,
                destinations,
            } => {
                let nodes: Vec<_> = destinations.iter().flatten().copied().collect();
                let present: Vec<_> = destinations.iter().map(Option::is_some).collect();
                self.collect(
                    vm,
                    code.clone(),
                    &nodes,
                    Goal::BindResults {
                        destinations: present,
                        call: *call,
                    },
                    depth,
                )?;
            }
            StatementCode::Exit(exit) => self.collect(
                vm,
                code.clone(),
                &exit.values,
                Goal::Exit {
                    cleanups: exit.cleanups.to_vec(),
                    transfer: exit.transfer,
                },
                depth,
            )?,
            StatementCode::Cleanup(id) => self.cleanup(vm, code.clone(), *id, depth)?,
            StatementCode::If {
                condition,
                then_block,
                else_block,
            } => {
                self.push(
                    vm,
                    Some(code.clone()),
                    depth,
                    Action::Branch {
                        then_block: *then_block,
                        else_block: *else_block,
                    },
                    0,
                )?;
                self.eval(vm, code.clone(), *condition, depth + 1)?;
            }
            StatementCode::Cases(cases) => {
                self.push(
                    vm,
                    Some(code.clone()),
                    depth,
                    Action::CasesEnd(cases.flow),
                    0,
                )?;
                self.push(
                    vm,
                    Some(code.clone()),
                    depth,
                    Action::CasesNext {
                        block,
                        pc,
                        index: 0,
                        through: false,
                    },
                    0,
                )?;
                self.push(vm, Some(code.clone()), depth, Action::SubjectEnd, 0)?;
                self.block(vm, code.clone(), cases.subject, depth + 1)?;
            }
            StatementCode::While {
                id,
                condition,
                body,
            } => {
                vm.enter_loop(*id)?;
                self.push(
                    vm,
                    Some(code.clone()),
                    depth,
                    Action::LoopBoundary(LoopState::While {
                        id: *id,
                        condition: *condition,
                        body: *body,
                    }),
                    0,
                )?;
                self.push(
                    vm,
                    Some(code.clone()),
                    depth,
                    Action::WhileCheck {
                        id: *id,
                        condition: *condition,
                        body: *body,
                    },
                    0,
                )?;
            }
            StatementCode::Range(range) => {
                vm.enter_loop(range.id)?;
                self.collect(
                    vm,
                    code.clone(),
                    &[range.start, range.end],
                    Goal::Range {
                        id: range.id,
                        iterator: range.iterator,
                        ty: range.integer_type,
                        direction: range.direction,
                        body: range.body,
                    },
                    depth,
                )?;
            }
            StatementCode::Block(block) => self.block(vm, code.clone(), *block, depth + 1)?,
            StatementCode::PushContext {
                id,
                value,
                body,
            } => {
                self.push(
                    vm,
                    Some(code.clone()),
                    depth,
                    Action::PushStart {
                        id: *id,
                        body: *body,
                    },
                    0,
                )?;
                self.eval(vm, code.clone(), *value, depth + 1)?;
            }
            StatementCode::Simd(simd) => {
                let register_bytes = simd.registers.iter().try_fold(0usize, |total, width| {
                    total
                        .checked_add(width.bytes())
                        .ok_or(Error::Limit(LimitKind::ValueCells))
                })?;
                let reserved = register_bytes
                    .checked_add(
                        simd.registers
                            .iter()
                            .map(|width| width.bytes())
                            .max()
                            .unwrap_or(0),
                    )
                    .ok_or(Error::Limit(LimitKind::ValueCells))?;
                let registers = vec![None; simd.registers.len()];
                self.push(
                    vm,
                    Some(code.clone()),
                    depth,
                    Action::Simd {
                        block,
                        pc,
                        index: 0,
                        registers,
                        reserved,
                    },
                    reserved,
                )?;
            }
        }
        Ok(())
    }
    fn cleanup<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
        &mut self,
        vm: &mut Vm<'_, P, E>,
        code: Arc<Plan>,
        id: CleanupId,
        depth: usize,
    ) -> Result<()> {
        let cleanup = *code
            .cleanups
            .get(id.index())
            .ok_or(Error::InvalidIr("cleanup plan missing"))?;
        let frame = vm.frame()?;
        let context = match cleanup.context {
            CleanupContext::Procedure => frame.procedure_context.clone(),
            CleanupContext::Push(id) => Some(
                frame
                    .push_contexts
                    .get(&id)
                    .cloned()
                    .ok_or(Error::InvalidIr("cleanup push context is inactive"))?,
            ),
        };
        let previous = std::mem::replace(&mut vm.current_context, context);
        self.push(
            vm,
            Some(code.clone()),
            depth,
            Action::CleanupRestore {
                previous,
            },
            0,
        )?;
        self.block(vm, code.clone(), cleanup.body, depth + 1)
    }
    fn end_loop<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
        &mut self,
        vm: &mut Vm<'_, P, E>,
        id: LoopId,
    ) -> Result<()> {
        let task = self
            .tasks
            .pop()
            .ok_or(Error::InvalidIr("loop boundary missing"))?;
        self.retained = self
            .retained
            .checked_sub(task.cells)
            .ok_or(Error::InvalidIr("loop accounting underflow"))?;
        if !matches!(task.action, Action::LoopBoundary(ref state) if state.id() == id) {
            return Err(Error::InvalidIr("loop boundary mismatch").into());
        }
        vm.leave_loop()
    }
    fn transfer<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
        &mut self,
        vm: &mut Vm<'_, P, E>,
        transfer: TransferCode,
        values: Vec<Value>,
        depth: usize,
    ) -> Result<()> {
        while let Some(task) = self.tasks.pop() {
            self.retained = self
                .retained
                .checked_sub(task.cells)
                .ok_or(Error::InvalidIr("unwind accounting underflow"))?;
            match task.action {
                Action::PushRestore {
                    id,
                    pointer,
                    previous,
                } => {
                    vm.current_context = previous;
                    vm.frames
                        .last_mut()
                        .ok_or(Error::InvalidIr("push frame missing during unwind"))?
                        .push_contexts
                        .remove(&id);
                    vm.memory.release(&pointer)?;
                }
                Action::CleanupRestore {
                    ..
                } => {
                    return Err(Error::InvalidIr("cleanup transferred outside its body").into());
                }
                Action::SubjectEnd => {
                    return Err(Error::InvalidIr("case subject transferred control").into());
                }
                Action::LoopBoundary(state) => {
                    let id = state.id();
                    vm.leave_loop()?;
                    match transfer {
                        TransferCode::Break(target) if id == target => return Ok(()),
                        TransferCode::Continue(target) if id == target => {
                            vm.enter_loop(id)?;
                            let action = match &state {
                                LoopState::While {
                                    id,
                                    condition,
                                    body,
                                } => Action::WhileCheck {
                                    id: *id,
                                    condition: *condition,
                                    body: *body,
                                },
                                LoopState::Range(range) => Action::RangeNext(range.clone()),
                            };
                            self.push(
                                vm,
                                task.code.clone(),
                                depth,
                                Action::LoopBoundary(state),
                                0,
                            )?;
                            self.push(vm, task.code, depth, action, 0)?;
                            return Ok(());
                        }
                        _ => {}
                    }
                }
                Action::CallBoundary {
                    signature,
                    previous_context,
                    expected,
                } => {
                    if !matches!(transfer, TransferCode::Return) {
                        return Err(Error::InvalidIr("loop transfer escaped a procedure").into());
                    }
                    return self.finish_call(vm, signature, previous_context, expected, values);
                }
                _ => {}
            }
        }
        Err(Error::InvalidIr("transfer has no matching continuation boundary").into())
    }
    fn finish_call<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
        &mut self,
        vm: &mut Vm<'_, P, E>,
        signature: TypeId,
        previous: Option<Pointer>,
        expected: Option<TypeId>,
        values: Vec<Value>,
    ) -> Result<()> {
        let frame = vm
            .frames
            .pop()
            .ok_or(Error::InvalidIr("continuation call frame missing"))?;
        vm.restore_call_context(previous);
        vm.validate_sequence_escape(&values, &frame.sequence_temp_roots)?;
        for pointer in frame.slots.iter().chain(&frame.temporaries) {
            vm.memory.release(pointer)?;
        }
        vm.validate_values(&values, &vm.signature(signature)?.results)?;
        self.receive(vm, values, expected)
    }
    fn receive<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
        &mut self,
        vm: &Vm<'_, P, E>,
        mut values: Vec<Value>,
        expected: Option<TypeId>,
    ) -> Result<()> {
        if let Some(ty) = expected {
            if values.len() != 1 {
                return Err(
                    Error::InvalidIr("scalar continuation call has multiple results").into(),
                );
            }
            let value = values.remove(0);
            value.validate(vm.provider.types(), ty, vm.limits.evaluation_depth.min(256))?;
            self.operand(vm, Operand::Value(value))
        } else {
            self.operand(vm, Operand::Results(values))
        }
    }
    fn invoke<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
        &mut self,
        vm: &mut Vm<'_, P, E>,
        invocation: Invocation<'_>,
        depth: usize,
    ) -> Result<()> {
        let Invocation {
            id,
            signature: requested_signature,
            arguments,
            expected,
            charged,
            retry_leaf,
        } = invocation;
        vm.require_procedure_phase(id)?;
        if !charged {
            vm.step(depth)?;
            vm.statistics.calls = vm
                .statistics
                .calls
                .checked_add(1)
                .ok_or(Error::Limit(LimitKind::Fuel))?;
        }
        if vm.frames.len() >= vm.limits.stack_depth.min(128) {
            return Err(Error::Limit(LimitKind::StackDepth).into());
        }
        let availability = vm.provider.procedure(id);
        let (ready, capability) = match availability {
            ProcedureAvailability::Ready(checked) => {
                validate_checked_environment(vm, &checked)?;
                if checked.procedure().id != id {
                    return Err(Error::InvalidIr(
                        "provider returned the wrong continuation procedure",
                    )
                    .into());
                }
                let plan = if let Some(plan) = self.plans.get(&id) {
                    plan.clone()
                } else {
                    let plan = Arc::new(plan::compile_checked_procedure(checked, vm.limits)?);
                    vm.charge_work(
                        usize::try_from(plan.code.planning_work)
                            .map_err(|_| Error::Limit(LimitKind::Fuel))?,
                    )?;
                    self.plan_cells = self
                        .plan_cells
                        .checked_add(plan.code.metadata_cells)
                        .filter(|cells| {
                            cells
                                .saturating_add(self.retained)
                                .saturating_add(vm.memory.value_cells())
                                .saturating_add(vm.expression_bindings.cells())
                                .saturating_add(
                                    vm.processes
                                        .as_ref()
                                        .map_or(0, process::ProcessState::cells),
                                )
                                <= vm.limits.value_cells
                        })
                        .ok_or(Error::Limit(LimitKind::ValueCells))?;
                    self.plans.insert(id, plan.clone());
                    plan
                };
                (Some(plan), None)
            }
            ProcedureAvailability::Pending(dependency) => return Err(Halt::Pending(dependency)),
            ProcedureAvailability::Failed(error) => return Err(error.into()),
            ProcedureAvailability::Missing => return Err(Error::MissingProcedure(id).into()),
            ProcedureAvailability::Foreign => {
                return Err(Error::UnsupportedForeignProcedure(id).into());
            }
            capability @ (ProcedureAvailability::Compiler(_)
            | ProcedureAvailability::Runtime(_)
            | ProcedureAvailability::FileAbi(_)
            | ProcedureAvailability::HeapAbi(_)
            | ProcedureAvailability::ProcessAbi(_)) => (None, Some(capability)),
        };
        if let Some(plan) = ready {
            let signature = plan.source.signature;
            vm.validate_provided_signature(id, signature)?;
            if requested_signature.is_some_and(|requested| requested != signature) {
                return Err(Error::TypeMismatch {
                    expected: requested_signature.unwrap(),
                }
                .into());
            }
            let signature_type = vm.signature(signature)?;
            vm.validate_values(arguments, &signature_type.parameters)?;
            // Complete all demanded local layouts before allocating the frame,
            // so a readiness suspension cannot leave a partly-created frame.
            for local in &plan.source.locals {
                vm.prepare_layout(local.ty())?;
            }
            let mut slots = Vec::with_capacity(plan.source.locals.len());
            for local in &plan.source.locals {
                slots.push(
                    vm.memory.allocate_with_alignment(
                        vm.provider.types(),
                        local.ty(),
                        None,
                        vm.provider
                            .storage_alignments()
                            .and_then(|alignments| alignments.local(local.id()))
                            .unwrap_or(1),
                    )?,
                );
            }
            for (parameter, value) in plan.source.parameters.iter().zip(arguments) {
                vm.charge_work(value.cells(vm.limits.value_cells)?)?;
                vm.store_pointer(
                    slots
                        .get(parameter.id().index())
                        .ok_or(Error::InvalidIr("parameter slot missing"))?,
                    value.clone(),
                    depth + 1,
                )?;
            }
            let previous_context = vm.enter_call_context(signature_type.context)?;
            vm.frames.push(Frame {
                procedure: FrameCode::Owned(plan.source.clone()),
                slots,
                loops: vec![],
                temporaries: vec![],
                sequence_temp_bytes: 0,
                sequence_temp_roots: vec![],
                procedure_context: vm.current_context.clone(),
                push_contexts: HashMap::new(),
            });
            vm.statistics.maximum_stack_depth =
                vm.statistics.maximum_stack_depth.max(vm.frames.len());
            self.push(
                vm,
                Some(plan.code.clone()),
                depth,
                Action::CallBoundary {
                    signature,
                    previous_context,
                    expected,
                },
                0,
            )?;
            self.block(vm, plan.code.clone(), plan.body, depth + 1)?;
        } else {
            if let Some(signature) = requested_signature {
                vm.validate_provided_signature(id, signature)?;
            }
            let capability = capability.unwrap();
            if let ProcedureAvailability::ProcessAbi(proof) = &capability {
                vm.statistics.maximum_stack_depth =
                    vm.statistics.maximum_stack_depth.max(vm.frames.len() + 1);
                return self.invoke_process_leaf(vm, id, proof, arguments, expected, depth);
            }
            let cells = arguments.iter().try_fold(0usize, |cells, value| {
                cells
                    .checked_add(value.cells(vm.limits.value_cells)?)
                    .ok_or(Error::Limit(LimitKind::ValueCells))
            })?;
            let occupied = self
                .retained
                .checked_add(self.plan_cells)
                .and_then(|n| n.checked_add(vm.memory.value_cells()))
                .and_then(|n| n.checked_add(vm.expression_bindings.cells()))
                .and_then(|n| {
                    n.checked_add(
                        vm.processes
                            .as_ref()
                            .map_or(0, process::ProcessState::cells),
                    )
                })
                .and_then(|n| n.checked_add(cells))
                .filter(|n| *n <= vm.limits.value_cells)
                .ok_or(Error::Limit(LimitKind::ValueCells))?;
            let remaining_fuel = vm.limits.fuel.saturating_sub(vm.statistics.steps);
            if retry_leaf {
                vm.effects.retry_leaf(remaining_fuel)?;
            } else {
                vm.effects
                    .begin_leaf(vm.limits.value_cells - occupied, remaining_fuel)?;
            }
            vm.charge_work(cells)?;
            vm.statistics.maximum_stack_depth =
                vm.statistics.maximum_stack_depth.max(vm.frames.len() + 1);
            let result = vm.invoke_available(id, arguments.to_vec(), depth + 1, capability);
            if let Some(error) = vm.effects.take_failure() {
                return Err(error.into());
            }
            let work = vm.effects.take_work();
            vm.charge_work(work)?;
            let values = result?;
            vm.effects.end_leaf()?;
            self.receive(vm, values, expected)?;
        }
        Ok(())
    }
    fn simd_task<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
        &mut self,
        vm: &mut Vm<'_, P, E>,
        code: Arc<Plan>,
        depth: usize,
        action: Action,
    ) -> Result<()> {
        let (block, pc, index, mut registers, reserved, address) = match action {
            Action::Simd {
                block,
                pc,
                index,
                registers,
                reserved,
            } => (block, pc, index, registers, reserved, None),
            Action::SimdAddress {
                block,
                pc,
                index,
                registers,
                reserved,
            } => (
                block,
                pc,
                index,
                registers,
                reserved,
                Some(self.pop(vm)?.into_value()?.pointer()?.clone()),
            ),
            _ => unreachable!(),
        };
        let StatementCode::Simd(simd) = &code.blocks[block.index()].statements[pc] else {
            unreachable!()
        };
        if index == simd.instructions.len() {
            return Ok(());
        }
        vm.memory
            .value_cells()
            .checked_add(reserved)
            .filter(|total| *total <= vm.limits.value_cells)
            .ok_or(Error::Limit(LimitKind::ValueCells))?;
        let instruction = simd.instructions[index];
        match instruction {
            SimdInstructionCode::Load {
                destination,
                interpretation,
                width,
                ..
            } => {
                validate_simd_instruction(simd, width, interpretation)?;
                if simd.registers.get(destination) != Some(&width) {
                    return Err(Error::InvalidIr("SIMD load register width changed").into());
                }
            }
            SimdInstructionCode::Store {
                source,
                interpretation,
                width,
                ..
            } => {
                validate_simd_instruction(simd, width, interpretation)?;
                if simd.registers.get(source) != Some(&width) {
                    return Err(Error::InvalidIr("SIMD store register width changed").into());
                }
            }
            SimdInstructionCode::Add {
                destination,
                left,
                right,
                interpretation,
                width,
            } => {
                validate_simd_instruction(simd, width, interpretation)?;
                if [destination, left, right]
                    .into_iter()
                    .any(|index| simd.registers.get(index) != Some(&width))
                {
                    return Err(Error::InvalidIr("SIMD arithmetic register width changed").into());
                }
            }
            SimdInstructionCode::Trap => {}
        }
        match address {
            None => {
                vm.step(depth)?;
                match instruction {
                    SimdInstructionCode::Trap => return Err(Error::RuntimeTrap.into()),
                    SimdInstructionCode::Load {
                        address, ..
                    }
                    | SimdInstructionCode::Store {
                        address, ..
                    } => {
                        self.push(
                            vm,
                            Some(code.clone()),
                            depth,
                            Action::SimdAddress {
                                block,
                                pc,
                                index,
                                registers,
                                reserved,
                            },
                            reserved,
                        )?;
                        return self.eval(vm, code.clone(), address, depth + 1);
                    }
                    SimdInstructionCode::Add {
                        destination,
                        left,
                        right,
                        interpretation,
                        width,
                    } => {
                        vm.charge_work(width.bytes())?;
                        let left = registers[left].as_deref().ok_or(Error::Uninitialized)?;
                        let right = registers[right].as_deref().ok_or(Error::Uninitialized)?;
                        if left.len() != width.bytes() || right.len() != width.bytes() {
                            return Err(
                                Error::InvalidIr("SIMD arithmetic storage width changed").into()
                            );
                        }
                        let bytes = match interpretation {
                            SimdInterpretation::U8 => left
                                .iter()
                                .zip(right)
                                .map(|(a, b)| a.wrapping_add(*b))
                                .collect(),
                            SimdInterpretation::F32 => {
                                let mut bytes = Vec::with_capacity(width.bytes());
                                for (a, b) in
                                    left.as_chunks::<4>().0.iter().zip(right.as_chunks::<4>().0)
                                {
                                    let value = jai_types::FloatValue::F32(u32::from_le_bytes(*a))
                                        .binary(
                                            jai_types::FloatOp::Add,
                                            jai_types::FloatValue::F32(u32::from_le_bytes(*b)),
                                        )
                                        .map_err(Error::from)?;
                                    bytes.extend_from_slice(&(value.bits() as u32).to_le_bytes());
                                }
                                bytes
                            }
                        };
                        registers[destination] = Some(bytes);
                    }
                }
            }
            Some(address) => {
                if vm.memory.target().policy.pointer().size != 8
                    || vm.memory.target().endian != crate::Endian::Little
                {
                    return Err(Error::InvalidIr(
                        "SIMD continuation requires a little-endian 64-bit target",
                    )
                    .into());
                }
                match instruction {
                    SimdInstructionCode::Load {
                        destination,
                        width,
                        ..
                    } => {
                        let work = vm.memory.intrinsic_work_cost(
                            vm.provider.types(),
                            &[(&address, false)],
                            width.bytes(),
                        )?;
                        vm.charge_work(
                            usize::try_from(work).map_err(|_| Error::Limit(LimitKind::Fuel))?,
                        )?;
                        registers[destination] = Some(vm.memory.simd_read_bytes(
                            vm.provider.types(),
                            &address,
                            width.bytes(),
                        )?);
                    }
                    SimdInstructionCode::Store {
                        source,
                        width,
                        ..
                    } => {
                        let bytes = registers[source].as_deref().ok_or(Error::Uninitialized)?;
                        if bytes.len() != width.bytes() {
                            return Err(Error::InvalidIr("SIMD store storage width changed").into());
                        }
                        let work = vm.memory.intrinsic_work_cost(
                            vm.provider.types(),
                            &[(&address, true)],
                            bytes.len(),
                        )?;
                        vm.charge_work(
                            usize::try_from(work).map_err(|_| Error::Limit(LimitKind::Fuel))?,
                        )?;
                        vm.memory.simd_write_bytes_reserved(
                            vm.provider.types(),
                            &address,
                            bytes,
                            reserved,
                        )?;
                    }
                    _ => {
                        return Err(Error::InvalidIr(
                            "SIMD address attached to arithmetic instruction",
                        )
                        .into());
                    }
                }
            }
        }
        self.push(
            vm,
            Some(code.clone()),
            depth,
            Action::Simd {
                block,
                pc,
                index: index + 1,
                registers,
                reserved,
            },
            reserved,
        )
    }
}
fn validate_simd_instruction(
    code: &SimdCode,
    width: SimdWidth,
    interpretation: SimdInterpretation,
) -> Result<()> {
    if width == SimdWidth::Y256
        && (!code.features.requires_avx()
            || interpretation == SimdInterpretation::U8 && !code.features.avx2)
    {
        return Err(
            Error::InvalidIr("wide SIMD operation lacks its checked feature declaration").into(),
        );
    }
    Ok(())
}
fn operand_cells(value: &Operand, limit: usize) -> std::result::Result<usize, Error> {
    match value {
        Operand::Value(value) => value.cells(limit),
        Operand::Place(pointer) => Ok(pointer.metadata_cells().saturating_add(1)),
        Operand::Results(values) => values.iter().try_fold(
            1usize.saturating_add(values.capacity() - values.len()),
            |cells, value| {
                cells
                    .checked_add(value.cells(limit)?)
                    .filter(|cells| *cells <= limit)
                    .ok_or(Error::Limit(LimitKind::ValueCells))
            },
        ),
    }
}
fn clone_operands<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
    vm: &mut Vm<'_, P, E>,
    values: &[Operand],
) -> Result<Vec<Operand>> {
    let cells = values.iter().try_fold(0usize, |cells, value| {
        cells
            .checked_add(operand_cells(value, vm.limits.value_cells)?)
            .filter(|cells| *cells <= vm.limits.value_cells)
            .ok_or(Error::Limit(LimitKind::ValueCells))
    })?;
    vm.charge_work(cells)?;
    Ok(values
        .iter()
        .map(|value| match value {
            Operand::Value(value) => Operand::Value(value.clone()),
            Operand::Place(pointer) => Operand::Place(pointer.clone()),
            Operand::Results(values) => Operand::Results(values.clone()),
        })
        .collect())
}
fn validate_operand<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
    vm: &Vm<'_, P, E>,
    value: &Operand,
    ty: Option<TypeId>,
) -> Result<()> {
    if let Some(ty) = ty {
        match value {
            Operand::Value(value) => {
                value.validate(vm.provider.types(), ty, vm.limits.evaluation_depth.min(256))?;
                vm.memory
                    .validate_runtime_type_values(vm.provider.types(), value)?;
            }
            Operand::Place(pointer) if pointer.pointee() == ty => {}
            _ => {
                return Err(Error::TypeMismatch {
                    expected: ty,
                }
                .into());
            }
        }
    }
    Ok(())
}
fn validate_checked_environment<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
    vm: &Vm<'_, P, E>,
    checked: &CheckedProcedure<'_>,
) -> Result<()> {
    if checked.types().scalar(jai_types::ScalarType::Bool)
        != vm.provider.types().scalar(jai_types::ScalarType::Bool)
        || checked.signatures() != vm.provider.signatures()
        || checked.globals() != vm.provider.globals()
        || checked.context() != vm.provider.context()
        || vm
            .provider
            .places()
            .map_or(!checked.places().is_empty(), |places| {
                places != checked.places()
            })
    {
        return Err(
            Error::InvalidIr("checked continuation environment differs from provider").into(),
        );
    }
    Ok(())
}
