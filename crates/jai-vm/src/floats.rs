//! VM adapters use the shared IEEE domain rather than duplicating numeric rules.
use crate::Error;
use jai_types::{
    CastMode, FloatOp, FloatToIntMode, FloatType, FloatValue, Integer, IntegerType, Relation,
};

pub(crate) fn binary(
    ty: FloatType,
    op: FloatOp,
    left: FloatValue,
    right: FloatValue,
) -> Result<FloatValue, Error> {
    if left.ty() != ty || right.ty() != ty {
        return Err(Error::InvalidIr("floating-point operand types differ"));
    }
    Ok(left.binary(op, right)?)
}

pub(crate) fn compare(
    relation: Relation,
    left: FloatValue,
    right: FloatValue,
) -> Result<bool, Error> {
    Ok(left.compare(relation, right)?)
}

pub(crate) fn to_integer(
    value: FloatValue,
    target: IntegerType,
    mode: CastMode,
) -> Result<Integer, Error> {
    if mode != CastMode::Checked {
        return Err(Error::InvalidIr(match mode {
            CastMode::Unchecked => {
                "unchecked float-to-integer conversion has no defined source policy"
            }
            _ => "cast,trunc float-to-integer conversion has no defined source policy",
        }));
    }
    value
        .to_integer(target, FloatToIntMode::Truncate)
        .map_err(|_| Error::CheckedCast)
}
