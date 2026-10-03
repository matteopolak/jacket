//! Inspect backing bytes only after both string descriptors have been evaluated.
use super::*;

impl<P: ProcedureProvider + ?Sized, E: CompilerEffects> Vm<'_, P, E> {
    pub(super) fn compare_strings(
        &mut self,
        relation: Equality,
        left: &ValueExpr,
        right: &ValueExpr,
        depth: usize,
    ) -> Result<bool> {
        let left = self.value(left, depth + 1)?;
        let right = self.value(right, depth + 1)?;
        let length = |value: &Value| match value {
            Value::String(bytes) => i64::try_from(bytes.len()).map_err(|_| Error::CheckedCast),
            Value::StringView { count, .. } => Ok(*count),
            _ => Err(Error::InvalidIr(
                "string comparison requires string descriptors",
            )),
        };
        let count = length(&left)?;
        let mut equal = count == length(&right)?;
        if equal {
            let count = crate::checked_sequence_count(count)?;
            if count != 0 {
                for value in [&left, &right] {
                    if let Value::StringView { pointer, .. } = value {
                        self.prepare_pointer_layouts(pointer, true)?;
                        self.memory
                            .validate_slice(self.provider.types(), pointer, count)?;
                    }
                }
            }
            for index in 0..count {
                self.step(depth + 1)?;
                if self.string_byte(&left, index)? != self.string_byte(&right, index)? {
                    equal = false;
                    break;
                }
            }
        }
        Ok(match relation {
            Equality::Equal => equal,
            Equality::NotEqual => !equal,
        })
    }

    fn string_byte(&mut self, value: &Value, index: usize) -> Result<u8> {
        match value {
            Value::String(bytes) => bytes
                .get(index)
                .copied()
                .ok_or_else(|| Error::InvalidIr("string byte is outside its count").into()),
            Value::StringView { pointer, .. } => {
                self.prepare_pointer_layouts(pointer, true)?;
                let offset = isize::try_from(index).map_err(|_| Error::CheckedCast)?;
                let pointer = self.memory.offset(self.provider.types(), pointer, offset)?;
                let work = self
                    .memory
                    .load_work_cost(self.provider.types(), &pointer)?;
                self.charge_work(work)?;
                let value = self
                    .memory
                    .load(self.provider.types(), &pointer)?
                    .number()?;
                if value.provenance().is_some() {
                    return Err(Error::UnsupportedPointerOperation(
                        "address-derived bytes cannot control string comparisons",
                    )
                    .into());
                }
                if value.ty() != jai_types::IntegerType::U8 {
                    return Err(Error::InvalidIr("string comparison has non-byte backing").into());
                }
                u8::try_from(value.value()).map_err(|_| Error::CheckedCast.into())
            }
            _ => Err(Error::InvalidIr("string comparison requires string descriptors").into()),
        }
    }
}
