//! Sequential tests share a cached subject; #through jumps directly to a body.
use super::*;
impl Generator<'_, '_> {
    pub(super) fn cases(&mut self, case: &jai_sema::Cases) -> Result<(), Error> {
        match case.subject.as_ref() {
            Statement::StoreInt(id, e) => {
                let v = self.int(e)?;
                self.builder.build_store(self.int_place(*id)?.0, v.0)?;
            }
            Statement::StoreBool(id, e) => {
                let v = self.boolean(e)?;
                self.builder.build_store(self.bool_place(*id)?.0, v.0)?;
            }
            _ => return Err(Error::Invariant),
        }
        let end = (case.flow == Flow::FallsThrough).then(|| self.label("case.end"));
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
            self.block(&arm.body)?;
            if arm.body.flow == Flow::FallsThrough {
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
            self.block(block)?;
            if block.flow == Flow::FallsThrough {
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
