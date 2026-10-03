use super::*;

const MAX_STATIC_WORK: usize = 1_048_576;

/// Published object prefixes are immutable and arena identities are never reused.
/// A ready proof may therefore reuse a closure check under its fixed environment.
#[derive(Default)]
pub(super) struct StaticClosures {
    validated: HashSet<(u64, usize)>,
    procedures: HashSet<(u64, usize)>,
    nodes: usize,
}

impl StaticClosures {
    fn key(data: &StaticData) -> (u64, usize) {
        (data.identity(), data.objects().len())
    }

    fn charge(&mut self, count: usize) -> Result<(), IrError> {
        self.nodes = self
            .nodes
            .checked_add(count)
            .filter(|nodes| *nodes <= MAX_STATIC_WORK)
            .ok_or(IrError::VerificationDepth)?;
        Ok(())
    }

    pub(super) fn validate(
        &mut self,
        data: &StaticData,
        types: &dyn TypeView,
    ) -> Result<(), IrError> {
        let key = Self::key(data);
        if self.validated.contains(&key) {
            return Ok(());
        }
        enum Value<'a> {
            Static(&'a StaticValue),
            Constant(&'a ConstantValue),
        }
        self.charge(data.objects().len())?;
        let mut pending: Vec<_> = data
            .objects()
            .iter()
            .map(|object| Value::Static(object.value()))
            .collect();
        while let Some(value) = pending.pop() {
            match value {
                Value::Static(value) => match &value.kind {
                    StaticValueKind::Constant(value) => {
                        self.charge(1)?;
                        pending.push(Value::Constant(value));
                    }
                    StaticValueKind::Record(values) | StaticValueKind::Array(values) => {
                        self.charge(values.len())?;
                        pending.extend(values.iter().map(Value::Static));
                    }
                    StaticValueKind::Address(address)
                    | StaticValueKind::Slice {
                        data: Some(address),
                        ..
                    } => self.charge(address.path().len())?,
                    StaticValueKind::Slice {
                        data: None, ..
                    } => {}
                },
                Value::Constant(value) => match &value.kind {
                    // Cross-graph Type constants cannot be retained by StaticData.
                    ConstantKind::RuntimeType(_) => {
                        return Err(IrError::InvalidConstant(value.ty));
                    }
                    ConstantKind::Record(values) | ConstantKind::Array(values) => {
                        self.charge(values.len())?;
                        pending.extend(values.iter().map(Value::Constant));
                    }
                    ConstantKind::Union {
                        value, ..
                    }
                    | ConstantKind::Distinct(value) => {
                        self.charge(1)?;
                        pending.push(Value::Constant(value));
                    }
                    _ => {}
                },
            }
        }
        data.validate(types)?;
        self.validated.insert(key);
        Ok(())
    }

    pub(super) fn procedures(
        &mut self,
        data: &StaticData,
        types: &dyn TypeView,
        signatures: &HashMap<ProcedureId, TypeId>,
    ) -> Result<(), IrError> {
        let key = Self::key(data);
        self.validate(data, types)?;
        if self.procedures.contains(&key) {
            return Ok(());
        }
        let mut pending: Vec<_> = data.objects().iter().map(|object| object.value()).collect();
        while let Some(value) = pending.pop() {
            match &value.kind {
                StaticValueKind::Constant(constant) => {
                    // The validated static shape excludes nested graph constants.
                    // This pass therefore stays linear in the already charged tree.
                    expressions::constant_procedures_with_closures(
                        types, constant, signatures, self,
                    )?;
                }
                StaticValueKind::Record(children) | StaticValueKind::Array(children) => {
                    pending.extend(children);
                }
                StaticValueKind::Address(_)
                | StaticValueKind::Slice {
                    ..
                } => {}
            }
        }
        self.procedures.insert(key);
        Ok(())
    }
}

#[cfg(test)]
mod tests;
