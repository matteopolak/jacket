use super::*;
#[cfg(test)]
mod tests;
impl<P: ProcedureProvider + ?Sized, E: CompilerEffects> Vm<'_, P, E> {
    pub(super) fn charge_work(&mut self, work: usize) -> Result<()> {
        self.flush_publication_work()?;
        self.statistics.steps = self
            .statistics
            .steps
            .checked_add(u64::try_from(work).map_err(|_| Error::Limit(LimitKind::Fuel))?)
            .filter(|steps| *steps <= self.limits.fuel)
            .ok_or(Error::Limit(LimitKind::Fuel))?;
        Ok(())
    }
    pub(super) fn pointer_truth(&mut self, pointer: &Pointer) -> Result<bool> {
        if pointer.code_pointer().is_some() || pointer.is_opaque() {
            self.prepare_pointer_layouts(pointer, false)?;
        }
        Ok(!pointer.is_null())
    }
    pub(super) fn number_truth(&mut self, number: &Number) -> Result<bool> {
        Ok(match number.provenance() {
            None => number.value() != 0,
            Some(crate::AddressProvenance::Pointer(pointer)) => self.pointer_truth(pointer)?,
            Some(_) => {
                return Err(Error::UnsupportedPointerOperation(
                    "truth test of transformed address integer is not target independent",
                )
                .into());
            }
        })
    }
    pub(super) fn number_cast(
        &mut self,
        ty: jai_types::IntegerType,
        source: Number,
        mode: jai_types::CastMode,
    ) -> Result<Number> {
        self.charge_work(source.metadata_cells())?;
        self.prepare_number_address(&source)?;
        if matches!(
            source.provenance(),
            Some(crate::AddressProvenance::Pointer(_))
        ) && u64::from(ty.bits()) < self.memory.target().policy.pointer().size * 8
        {
            return Err(Error::UnsupportedPointerOperation(
                "nonnull address integer cast to a narrower type cannot model native address range",
            )
            .into());
        }
        let result = scalar::cast_number(ty, source.clone(), mode)?;
        if let Some(crate::AddressProvenance::Pointer(pointer)) = source.provenance()
            && let Some(number) =
                self.memory
                    .affine_address_number(self.provider.types(), &result, pointer, 0)?
        {
            return Ok(number);
        }
        Ok(result)
    }
    pub(super) fn compare_numbers(
        &mut self,
        relation: jai_types::Relation,
        left: &Number,
        right: &Number,
    ) -> Result<bool> {
        use crate::AddressProvenance;
        use jai_types::Relation;
        self.charge_work(left.metadata_cells().saturating_add(right.metadata_cells()))?;
        self.prepare_number_address(left)?;
        self.prepare_number_address(right)?;
        match (left.provenance(), right.provenance()) {
            (None, None) => Ok(scalar::compare(relation, left.integer(), right.integer())?),
            (Some(AddressProvenance::Pointer(a)), Some(AddressProvenance::Pointer(b))) => {
                if matches!(relation, Relation::Equal | Relation::NotEqual) {
                    let equal = self.memory.same_address(self.provider.types(), a, b)?;
                    return Ok(equal == (relation == Relation::Equal));
                }
                let difference = self
                    .memory
                    .independent_address_difference(self.provider.types(), a, b)?
                    .ok_or(Error::UnsupportedPointerOperation(
                        "ordering addresses from different allocations is not target independent",
                    ))?;
                Ok(match relation {
                    Relation::Less => difference < 0,
                    Relation::LessEqual => difference <= 0,
                    Relation::Greater => difference > 0,
                    Relation::GreaterEqual => difference >= 0,
                    _ => unreachable!(),
                })
            }
            (Some(AddressProvenance::Pointer(pointer)), None)
                if right.value() == 0
                    && matches!(relation, Relation::Equal | Relation::NotEqual) =>
            {
                Ok(pointer.is_null() == (relation == Relation::Equal))
            }
            (None, Some(AddressProvenance::Pointer(pointer)))
                if left.value() == 0
                    && matches!(relation, Relation::Equal | Relation::NotEqual) =>
            {
                Ok(pointer.is_null() == (relation == Relation::Equal))
            }
            _ => Err(Error::UnsupportedPointerOperation(
                "comparison of transformed or absolute address integer is not target independent",
            )
            .into()),
        }
    }
    pub(super) fn number_binary(
        &mut self,
        ty: jai_types::IntegerType,
        op: jai_types::IntOp,
        left: Number,
        right: Number,
        check: CheckMode,
    ) -> Result<Number> {
        self.charge_work(
            left.metadata_cells()
                .checked_add(right.metadata_cells())
                .ok_or(Error::Limit(LimitKind::Fuel))?,
        )?;
        self.prepare_number_address(&left)?;
        self.prepare_number_address(&right)?;
        let result = scalar::binary_number(
            ty,
            op,
            left.clone(),
            right.clone(),
            check,
            self.limits.value_cells,
        )?;
        if op == jai_types::IntOp::Subtract
            && let (
                Some(crate::AddressProvenance::Pointer(a)),
                Some(crate::AddressProvenance::Pointer(b)),
            ) = (left.provenance(), right.provenance())
            && let Some(value) =
                self.memory
                    .independent_address_difference(self.provider.types(), a, b)?
        {
            // The common virtual base cancels; only target byte offsets remain.
            return Ok(Number::plain(Integer::wrapping(ty, value)));
        }
        if let Some(crate::AddressProvenance::Pointer(pointer)) = left.provenance()
            && right.provenance().is_none()
        {
            let modulus = match op {
                jai_types::IntOp::Remainder => u64::try_from(right.value()).ok(),
                jai_types::IntOp::BitAnd => u64::try_from(right.value())
                    .ok()
                    .and_then(|mask| mask.checked_add(1)),
                _ => None,
            };
            if let Some(modulus) = modulus
                && let Some(value) = self.memory.independent_address_alignment(
                    self.provider.types(),
                    pointer,
                    modulus,
                )?
            {
                return Ok(Number::plain(Integer::wrapping(ty, i128::from(value))));
            }
        }
        let affine = match (op, left.provenance(), right.provenance()) {
            (jai_types::IntOp::Add, Some(crate::AddressProvenance::Pointer(pointer)), None) => {
                Some((pointer, right.value()))
            }
            (jai_types::IntOp::Add, None, Some(crate::AddressProvenance::Pointer(pointer))) => {
                Some((pointer, left.value()))
            }
            (
                jai_types::IntOp::Subtract,
                Some(crate::AddressProvenance::Pointer(pointer)),
                None,
            ) => Some((pointer, -right.value())),
            _ => None,
        };
        if let Some((pointer, delta)) = affine
            && let Some(number) =
                self.memory
                    .affine_address_number(self.provider.types(), &result, pointer, delta)?
        {
            return Ok(number);
        }
        Ok(result)
    }
}

impl<P: ProcedureProvider + ?Sized, E: CompilerEffects> Vm<'_, P, E> {
    /// Immutable publication callbacks debit the same job fuel as VM actions.
    /// The pending debit is common execution state and is never snapshotted.
    pub fn charge_publication_work(&self, work: usize) -> std::result::Result<(), Error> {
        let work = u64::try_from(work).map_err(|_| Error::Limit(LimitKind::Fuel))?;
        let pending = self
            .publication_work
            .get()
            .checked_add(work)
            .filter(|pending| {
                self.statistics
                    .steps
                    .checked_add(*pending)
                    .is_some_and(|total| total <= self.limits.fuel)
            })
            .ok_or(Error::Limit(LimitKind::Fuel))?;
        self.publication_work.set(pending);
        Ok(())
    }
    pub fn publication_remaining_fuel(&self) -> u64 {
        self.limits
            .fuel
            .saturating_sub(self.statistics.steps)
            .saturating_sub(self.publication_work.get())
    }
    pub(super) fn flush_publication_work(&mut self) -> Result<()> {
        let total = self
            .statistics
            .steps
            .checked_add(self.publication_work.get())
            .filter(|total| *total <= self.limits.fuel)
            .ok_or(Error::Limit(LimitKind::Fuel))?;
        self.statistics.steps = total;
        self.publication_work.set(0);
        Ok(())
    }
    pub(super) fn publication_statistics(&self) -> Statistics {
        let mut statistics = self.statistics;
        statistics.steps = statistics.steps.saturating_add(self.publication_work.get());
        statistics
    }
}
