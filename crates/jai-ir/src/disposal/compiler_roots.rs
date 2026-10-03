//! Retire independently owned compiler expression roots without recursive drops.
use crate::{Call, ValueExpr};

pub fn discard_value_expression(expression: ValueExpr) {
    super::run(vec![super::Work::Value(expression)]);
}

pub fn discard_call(call: Call) {
    super::run(vec![super::Work::Call(call)]);
}
