//! Bound compiler-owned expansion while retaining compact all-zero defaults.
use jai_ir::{ConstantKind, ConstantValue};
pub(crate) const MAX_CONSTANT_CELLS: usize = 1_048_576;
pub(crate) const MAX_CONSTANT_DEPTH: usize = 128;

pub(crate) fn is_zero(value: &ConstantValue) -> bool {
    let mut pending = vec![value];
    while let Some(value) = pending.pop() {
        match &value.kind {
            ConstantKind::Zero => {}
            ConstantKind::Distinct(value)
            | ConstantKind::Union {
                value, ..
            } => pending.push(value),
            ConstantKind::Int(value) | ConstantKind::Enum(value) if value.bits() == 0 => {}
            ConstantKind::Bool(false) => {}
            ConstantKind::Float(jai_types::FloatValue::F32(0) | jai_types::FloatValue::F64(0)) => {}
            ConstantKind::StringBytes(bytes) if bytes.is_empty() => {}
            ConstantKind::Array(elements) | ConstantKind::Record(elements) => {
                pending.extend(elements)
            }
            _ => return false,
        }
    }
    true
}

pub(crate) fn cells(value: &ConstantValue) -> Option<usize> {
    let mut pending = vec![value];
    let mut count = 0usize;
    while let Some(value) = pending.pop() {
        count = count.checked_add(1)?;
        match &value.kind {
            ConstantKind::Array(elements) | ConstantKind::Record(elements) => {
                pending.extend(elements)
            }
            ConstantKind::Distinct(value)
            | ConstantKind::Union {
                value, ..
            } => pending.push(value),
            ConstantKind::StringBytes(bytes) => count = count.checked_add(bytes.len())?,
            _ => {}
        }
        if count > MAX_CONSTANT_CELLS {
            return None;
        }
    }
    Some(count)
}
