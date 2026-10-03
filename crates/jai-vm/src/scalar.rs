use crate::{AddressProvenance, ArithmeticError, Error, Number};
use jai_ir::CheckMode;
use jai_types::{CastMode, Integer, IntegerType};
use jai_types::{IntOp, Relation};

fn derived(integer: Integer, source: &Number) -> Number {
    match source.memory_identity() {
        Some(memory) => Number::address(
            integer,
            AddressProvenance::Derived {
                memory,
                allocations: source.allocation_ids().into_boxed_slice(),
            },
        ),
        None => Number::plain(integer),
    }
}

fn union_origins<'a>(a: &'a Number, b: &'a Number) -> impl Iterator<Item = u64> + 'a {
    let mut left = a.allocation_ids_iter().peekable();
    let mut right = b.allocation_ids_iter().peekable();
    std::iter::from_fn(
        move || match (left.peek().copied(), right.peek().copied()) {
            (Some(a), Some(b)) if a == b => {
                left.next();
                right.next();
                Some(a)
            }
            (Some(a), Some(b)) if a < b => left.next(),
            (Some(_), Some(_)) => right.next(),
            (Some(_), None) => left.next(),
            (None, Some(_)) => right.next(),
            (None, None) => None,
        },
    )
}

fn bounded_union(a: &Number, b: &Number, maximum: usize) -> Result<Box<[u64]>, Error> {
    let count = union_origins(a, b).try_fold(0usize, |count, _| {
        count
            .checked_add(1)
            .filter(|count| *count <= maximum)
            .ok_or(Error::Limit(crate::LimitKind::ValueCells))
    })?;
    let mut origins = Vec::with_capacity(count);
    origins.extend(union_origins(a, b));
    Ok(origins.into_boxed_slice())
}

pub(crate) fn binary_number(
    ty: IntegerType,
    op: IntOp,
    a: Number,
    b: Number,
    check: CheckMode,
    maximum_origins: usize,
) -> Result<Number, Error> {
    let memory = match (a.memory_identity(), b.memory_identity()) {
        (Some(a), Some(b)) if a != b => return Err(Error::ForeignPointer),
        (Some(memory), _) | (_, Some(memory)) => Some(memory),
        (None, None) => None,
    };
    let origins = if memory.is_some() {
        bounded_union(&a, &b, maximum_origins)?
    } else {
        [].into()
    };
    let integer = binary_with_check(ty, op, a.integer(), b.integer(), check)?;
    Ok(match memory {
        Some(memory) => Number::address(
            integer,
            AddressProvenance::Derived {
                memory,
                allocations: origins,
            },
        ),
        None => Number::plain(integer),
    })
}

pub(crate) fn negate_number(
    ty: IntegerType,
    value: Number,
    check: CheckMode,
) -> Result<Number, Error> {
    let integer = negate_with_check(ty, value.integer(), check)?;
    Ok(derived(integer, &value))
}

pub(crate) fn cast_number(ty: IntegerType, value: Number, mode: CastMode) -> Result<Number, Error> {
    let integer = match mode {
        CastMode::Checked => Integer::checked(ty, value.value()).ok_or(Error::CheckedCast)?,
        CastMode::Unchecked | CastMode::Truncate => Integer::wrapping(ty, value.value()),
        CastMode::Force(_) => {
            return Err(Error::InvalidIr("force requires a checked storage bitcast"));
        }
    };
    Ok(derived(integer, &value))
}

pub(crate) fn complement_number(ty: IntegerType, value: Number) -> Result<Number, Error> {
    if value.ty() != ty {
        return Err(Error::InvalidIr("integer operand types differ"));
    }
    Ok(derived(Integer::wrapping(ty, !value.value()), &value))
}

#[cfg(test)]
pub(crate) fn binary(ty: IntegerType, op: IntOp, a: Integer, b: Integer) -> Result<Integer, Error> {
    binary_with_check(ty, op, a, b, CheckMode::Disabled)
}

pub(crate) fn binary_with_check(
    ty: IntegerType,
    op: IntOp,
    a: Integer,
    b: Integer,
    check: CheckMode,
) -> Result<Integer, Error> {
    if a.ty() != ty || b.ty() != ty {
        return Err(Error::InvalidIr("integer operand types differ"));
    }
    let (a, b) = (a.value(), b.value());
    if check.enabled() && matches!(op, IntOp::Add | IntOp::Subtract | IntOp::Multiply) {
        let value = match op {
            IntOp::Add => a.checked_add(b),
            IntOp::Subtract => a.checked_sub(b),
            IntOp::Multiply => a.checked_mul(b),
            _ => unreachable!(),
        };
        return value
            .and_then(|value| Integer::checked(ty, value))
            .ok_or(Error::Arithmetic(ArithmeticError::IntegerOverflow));
    }
    let value = match op {
        IntOp::Add => a.wrapping_add(b),
        IntOp::Subtract => a.wrapping_sub(b),
        IntOp::Multiply => a.wrapping_mul(b),
        IntOp::BitAnd => a & b,
        IntOp::BitOr => a | b,
        IntOp::BitXor => a ^ b,
        IntOp::Divide | IntOp::Remainder => {
            if b == 0 {
                return Err(Error::Arithmetic(ArithmeticError::ZeroDivisor));
            }
            if check.enabled() && ty.signed() && a == ty.min() && b == -1 {
                return Err(Error::Arithmetic(ArithmeticError::SignedDivisionOverflow));
            }
            if op == IntOp::Divide { a / b } else { a % b }
        }
        IntOp::ShiftLeft | IntOp::ShiftRight => {
            let count = u32::try_from(b)
                .ok()
                .filter(|count| *count < ty.bits())
                .ok_or(Error::Arithmetic(ArithmeticError::ShiftCount))?;
            if op == IntOp::ShiftLeft {
                a.wrapping_shl(count)
            } else {
                a >> count
            }
        }
    };
    Ok(Integer::wrapping(ty, value))
}

pub(crate) fn negate_with_check(
    ty: IntegerType,
    value: Integer,
    check: CheckMode,
) -> Result<Integer, Error> {
    if value.ty() != ty {
        return Err(Error::InvalidIr("integer operand types differ"));
    }
    if check.enabled() {
        value
            .value()
            .checked_neg()
            .and_then(|value| Integer::checked(ty, value))
            .ok_or(Error::Arithmetic(ArithmeticError::IntegerOverflow))
    } else {
        Ok(Integer::wrapping(ty, value.value().wrapping_neg()))
    }
}
pub(crate) fn compare(op: Relation, a: Integer, b: Integer) -> Result<bool, Error> {
    if a.ty() != b.ty() {
        return Err(Error::InvalidIr("comparison operand types differ"));
    }
    let (a, b) = (a.value(), b.value());
    Ok(match op {
        Relation::Equal => a == b,
        Relation::NotEqual => a != b,
        Relation::Less => a < b,
        Relation::LessEqual => a <= b,
        Relation::Greater => a > b,
        Relation::GreaterEqual => a >= b,
    })
}
