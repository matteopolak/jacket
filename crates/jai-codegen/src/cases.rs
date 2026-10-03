//! Sequential tests share a cached subject; #through jumps directly to a body.
use super::*;
impl Generator<'_, '_, '_> {
    pub(super) fn cases(&mut self, case: &jai_ir::Cases) -> Result<(), Error> {
        let previous = self.builder.get_current_debug_location();
        if let Some(debug) = &self.debug {
            debug
                .case_subject(self.context, &self.builder)
                .map_err(Error::Debug)?;
        }
        match case.subject.as_ref() {
            Statement::StoreInt(id, e) => {
                let v = self.int(e)?;
                let slot = self.int_place(*id)?;
                memory::store(&self.builder, slot.0, v.0.into(), slot.1)?;
            }
            Statement::StoreBool(id, e) => {
                let v = self.boolean(e)?;
                let slot = self.bool_place(*id)?;
                memory::store(&self.builder, slot.0, v.0.into(), slot.1)?;
            }
            Statement::Store(place, value) => self.store(*place, value)?,
            _ => return Err(Error::Invariant),
        }
        self.debug_restore_location(previous);
        let end =
            (!execution_phase::native_cases_terminates_with_bindings(case, &self.phase_bindings))
                .then(|| self.label("case.end"));
        let bodies: Vec<_> = case.arms.iter().map(|_| self.label("case.body")).collect();
        let tests: Vec<_> = case.arms.iter().map(|_| self.label("case.test")).collect();
        let default = case.default.as_ref().map(|_| self.label("case.default"));
        let impossible =
            (case.exhaustive && default.is_none()).then(|| self.label("case.unmatched"));
        let unmatched = default.or(impossible).or(end).ok_or(Error::Invariant)?;
        self.builder
            .build_unconditional_branch(tests.first().copied().unwrap_or(unmatched))?;
        for (i, arm) in case.arms.iter().enumerate() {
            self.builder.position_at_end(tests[i]);
            let condition = self.boolean(&arm.condition)?;
            self.builder.build_conditional_branch(
                condition.0,
                bodies[i],
                tests.get(i + 1).copied().unwrap_or(unmatched),
            )?;
            self.builder.position_at_end(bodies[i]);
            self.debug_child_block(jai_ir::DebugBranch::CaseArm(i), &arm.body)?;
            if self
                .builder
                .get_insert_block()
                .is_some_and(|block| block.get_terminator().is_none())
            {
                let next = if arm.through {
                    bodies.get(i + 1).copied().or(default)
                } else {
                    end
                };
                self.builder
                    .build_unconditional_branch(next.ok_or(Error::Invariant)?)?;
            }
        }
        if let (Some(label), Some(block)) = (default, &case.default) {
            self.builder.position_at_end(label);
            self.debug_child_block(jai_ir::DebugBranch::CaseDefault, block)?;
            if self
                .builder
                .get_insert_block()
                .is_some_and(|block| block.get_terminator().is_none())
            {
                self.builder
                    .build_unconditional_branch(end.ok_or(Error::Invariant)?)?;
            }
        }
        if let Some(impossible) = impossible {
            self.builder.position_at_end(impossible);
            self.builder.build_unreachable()?;
        }
        if let Some(end) = end {
            self.builder.position_at_end(end);
        }
        Ok(())
    }
}
