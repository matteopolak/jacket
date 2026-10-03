use super::*;

impl Context<'_> {
    pub(super) fn block(&mut self, block: &Block) -> Result<Flow, IrError> {
        let _depth = self.enter()?;
        let mut flow = Flow::FallsThrough;
        for statement in &block.statements {
            if flow == Flow::Terminates {
                return Err(IrError::InvalidFlow);
            }
            flow = self.statement(statement)?;
        }
        if flow != block.flow {
            return Err(IrError::InvalidFlow);
        }
        Ok(flow)
    }
    fn loop_start(&mut self, id: LoopId) -> Result<(), IrError> {
        if !self.loop_ids.insert(id) {
            return Err(IrError::DuplicateIdentity {
                kind: "loop",
                index: id.index(),
            });
        }
        self.loops.push(id);
        Ok(())
    }
    fn statement(&mut self, statement: &Statement) -> Result<Flow, IrError> {
        let _depth = self.enter()?;
        match statement {
            Statement::Simd(block) => {
                block.validate(self.types).map_err(IrError::Simd)?;
                for instruction in block.instructions() {
                    match instruction {
                        SimdInstruction::Load { address, .. }
                        | SimdInstruction::Store { address, .. } => {
                            self.value(address)?;
                        }
                        SimdInstruction::Add { .. }
                        | SimdInstruction::DebugTrap
                        | SimdInstruction::Arm64DebugTrap => {}
                    }
                }
            }
            Statement::PushContext { id, value, body } => {
                let expected = self.context.ok_or(IrError::MissingContext)?.record_type;
                same_type(expected, self.value(value)?)?;
                let expected_parent = self
                    .push_definitions
                    .and_then(|pushes| pushes.get(id))
                    .ok_or_else(|| unknown("push context", id.index()))?;
                if *expected_parent != self.active_push.last().copied() {
                    return Err(IrError::MissingContext);
                }
                let previous = self.context_available;
                self.context_available = true;
                self.active_push.push(*id);
                let flow = self.block(body);
                self.active_push.pop();
                self.context_available = previous;
                return flow;
            }
            Statement::Store(place, value) => {
                same_type(self.place(*place)?, self.value(value)?)?;
            }
            Statement::DiscardValue(value) => {
                self.value(value)?;
            }
            Statement::CallResults { call, destinations } => {
                let results = self.call(call)?;
                arity("call destinations", results.len(), destinations.len())?;
                for (destination, &ty) in destinations.iter().zip(results) {
                    if let Some(place) = destination {
                        same_type(ty, self.place(*place)?)?;
                    }
                }
            }
            Statement::IndirectCallResults {
                callee,
                arguments,
                destinations,
                inline_hint,
            } => {
                let results = self.indirect_call(callee, arguments, *inline_hint)?;
                arity(
                    "indirect call destinations",
                    results.len(),
                    destinations.len(),
                )?;
                for (destination, &ty) in destinations.iter().zip(&results) {
                    if let Some(place) = destination {
                        same_type(ty, self.place(*place)?)?;
                    }
                }
            }
            Statement::StoreInt(place, value) => {
                self.integer_place(*place)?;
                self.integer(value)?;
                same_integer(place.ty(), value.ty())?;
            }
            Statement::StoreBool(place, value) => {
                self.boolean_place(*place)?;
                self.boolean(value)?;
            }
            Statement::Exit(exit) => {
                let mut cleanups = HashSet::new();
                for id in &exit.cleanups {
                    self.cleanup_id(*id)?;
                    if !cleanups.insert(*id) {
                        return Err(IrError::DuplicateIdentity {
                            kind: "exit cleanup",
                            index: id.index(),
                        });
                    }
                }
                match &exit.transfer {
                    Transfer::ReturnVoid => {
                        self.return_types(&[])?;
                    }
                    Transfer::ReturnInt(value) => {
                        self.integer(value)?;
                        self.return_types(&[value.type_id(self.types)])?;
                    }
                    Transfer::ReturnBool(value) => {
                        self.boolean(value)?;
                        self.return_types(&[value.type_id(self.types)])?;
                    }
                    Transfer::ReturnValues(values) => {
                        let types = values
                            .iter()
                            .map(|value| self.value(value))
                            .collect::<Result<Vec<_>, _>>()?;
                        self.return_types(&types)?;
                    }
                    Transfer::Break(id) | Transfer::Continue(id) => {
                        if !self.loops.contains(id) {
                            return Err(unknown("loop exit", id.index()));
                        }
                    }
                }
                return Ok(Flow::Terminates);
            }
            Statement::Cleanup(id) => self.cleanup_id(*id)?,
            Statement::DiscardInt(value) => self.integer(value)?,
            Statement::DiscardBool(value) => self.boolean(value)?,
            Statement::CallVoid(call) => {
                arity("void call results", 0, self.call(call)?.len())?;
            }
            Statement::If(condition, yes, no) => {
                self.boolean(condition)?;
                let yes = self.block(yes)?;
                let no = self.block(no)?;
                return Ok(if yes == Flow::Terminates && no == Flow::Terminates {
                    Flow::Terminates
                } else {
                    Flow::FallsThrough
                });
            }
            Statement::Cases(cases) => return self.cases(cases),
            Statement::While {
                id,
                condition,
                body,
            } => {
                match condition {
                    LoopCondition::Value(value) => self.boolean(value)?,
                    LoopCondition::BoundInt(local, value) => {
                        self.integer_place(local.place())?;
                        self.integer(value)?;
                        same_integer(local.ty(), value.ty())?;
                    }
                    LoopCondition::BoundBool(local, value) => {
                        self.boolean_place(local.place())?;
                        self.boolean(value)?;
                    }
                }
                self.loop_start(*id)?;
                self.block(body)?;
                self.loops.pop();
            }
            Statement::Range(range) => {
                self.integer_place(range.iterator.place())?;
                self.integer(&range.start)?;
                self.integer(&range.end)?;
                same_integer(range.iterator.ty(), range.start.ty())?;
                same_integer(range.iterator.ty(), range.end.ty())?;
                self.loop_start(range.id)?;
                self.block(&range.body)?;
                self.loops.pop();
            }
            Statement::Block(block) => return self.block(block),
        }
        Ok(Flow::FallsThrough)
    }
    fn return_types(&self, actual: &[TypeId]) -> Result<(), IrError> {
        if self.cleanup {
            return Err(IrError::ReturnFromCleanup);
        }
        let procedure = self.procedure.ok_or(IrError::InvalidFlow)?;
        let signature = self.types.procedure_definition(procedure.signature)?;
        arity("return values", signature.results.len(), actual.len())?;
        for (&expected, &actual) in signature.results.iter().zip(actual) {
            same_type(expected, actual)?;
        }
        Ok(())
    }
    fn cleanup_id(&self, id: CleanupId) -> Result<(), IrError> {
        let cleanup = self
            .procedure
            .and_then(|procedure| procedure.cleanups.get(id.index()))
            .ok_or_else(|| unknown("cleanup", id.index()))?;
        if let CleanupContext::Push(push) = cleanup.context
            && !self.active_push.contains(&push)
        {
            return Err(IrError::MissingContext);
        }
        Ok(())
    }
    fn cases(&mut self, cases: &Cases) -> Result<Flow, IrError> {
        if self.statement(&cases.subject)? != Flow::FallsThrough {
            return Err(IrError::InvalidFlow);
        }
        let mut terminal_arms = true;
        for arm in &cases.arms {
            self.boolean(&arm.condition)?;
            let flow = self.block(&arm.body)?;
            if arm.through && flow == Flow::Terminates {
                return Err(IrError::InvalidFlow);
            }
            terminal_arms &= arm.through || flow == Flow::Terminates;
        }
        if cases.arms.last().is_some_and(|arm| arm.through) && cases.default.is_none() {
            return Err(IrError::InvalidFlow);
        }
        if cases.exhaustive && !boolean_coverage(cases) {
            return Err(IrError::InvalidExhaustiveness);
        }
        let terminal_default = if let Some(default) = &cases.default {
            self.block(default)? == Flow::Terminates
        } else {
            cases.exhaustive
        };
        let flow = if terminal_arms && terminal_default {
            Flow::Terminates
        } else {
            Flow::FallsThrough
        };
        if flow != cases.flow {
            return Err(IrError::InvalidFlow);
        }
        Ok(flow)
    }
}

fn boolean_coverage(cases: &Cases) -> bool {
    let Statement::StoreBool(subject, _) = cases.subject.as_ref() else {
        return false;
    };
    if !matches!(subject.place().kind(), PlaceKind::Local(_)) {
        return false;
    }
    let mut labels = [false; 2];
    for arm in &cases.arms {
        let BoolExpr::CompareBools(Equality::Equal, left, right) = &arm.condition else {
            return false;
        };
        let (BoolExpr::Load(place), BoolExpr::Constant(value)) = (left.as_ref(), right.as_ref())
        else {
            return false;
        };
        if place.place() != subject.place() {
            return false;
        }
        labels[usize::from(*value)] = true;
    }
    labels.into_iter().all(|label| label)
}

pub(super) fn cleanup_dependencies(procedure: &Procedure) -> Result<(), IrError> {
    let dependencies: Vec<Vec<CleanupId>> = procedure
        .cleanups
        .iter()
        .map(|cleanup| {
            let mut pending: Vec<&Statement> = cleanup.body.statements.iter().collect();
            let mut dependencies = Vec::new();
            while let Some(statement) = pending.pop() {
                match statement {
                    Statement::Cleanup(id) => dependencies.push(*id),
                    Statement::Exit(exit) => dependencies.extend(exit.cleanups.iter().copied()),
                    Statement::If(_, yes, no) => {
                        pending.extend(&yes.statements);
                        pending.extend(&no.statements);
                    }
                    Statement::While { body, .. }
                    | Statement::Block(body)
                    | Statement::PushContext { body, .. } => pending.extend(&body.statements),
                    Statement::Range(range) => pending.extend(&range.body.statements),
                    Statement::Cases(cases) => {
                        pending.push(&cases.subject);
                        for arm in &cases.arms {
                            pending.extend(&arm.body.statements);
                        }
                        if let Some(default) = &cases.default {
                            pending.extend(&default.statements);
                        }
                    }
                    _ => {}
                }
            }
            dependencies
        })
        .collect();
    for id in dependencies.iter().flatten() {
        if id.index() >= dependencies.len() {
            return Err(unknown("cleanup", id.index()));
        }
    }
    let mut state = vec![0_u8; dependencies.len()];
    for root in 0..dependencies.len() {
        if state[root] != 0 {
            continue;
        }
        let mut pending = vec![(root, false)];
        while let Some((index, finish)) = pending.pop() {
            if finish {
                state[index] = 2;
                continue;
            }
            match state[index] {
                2 => continue,
                1 => return Err(IrError::CleanupCycle(CleanupId::new(index))),
                _ => {}
            }
            state[index] = 1;
            pending.push((index, true));
            for dependency in dependencies[index].iter().rev() {
                pending.push((dependency.index(), false));
            }
        }
    }
    Ok(())
}
