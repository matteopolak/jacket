//! The expression tree and exact key are distinct real owners.
use super::*;
use crate::retained_metadata::{EvalRetainedMetadataError, admit, push};
use std::sync::Arc;

enum NodeRef<'a> {
    Weak(&'a Arc<WeakFloatValue>),
    Float(&'a FloatExpr),
    Number(&'a NumberExpr),
    Bool(&'a BoolExpr),
}
pub(crate) fn visit<E>(
    value: &Arc<WeakFloatValue>,
    charge: &mut impl FnMut(usize, usize) -> Result<(), E>,
) -> Result<(), EvalRetainedMetadataError<E>> {
    let mut pending = Vec::new();
    let mut visited = Vec::new();
    push(&mut pending, NodeRef::Weak(value), charge)?;
    while let Some(node) = pending.pop() {
        admit(1, 0, charge)?;
        match node {
            NodeRef::Weak(value) => {
                let pointer = Arc::as_ptr(value);
                let mut repeated = false;
                for previous in &visited {
                    admit(1, 0, charge)?;
                    if *previous == pointer {
                        repeated = true;
                        break;
                    }
                }
                if repeated {
                    continue;
                }
                push(&mut visited, pointer, charge)?;
                admit(
                    1,
                    std::mem::size_of::<WeakFloatValue>() + 3 * std::mem::size_of::<usize>(),
                    charge,
                )?;
                value.key.visit_retained_metadata(charge)?;
                push(&mut pending, NodeRef::Float(&value.expression), charge)?;
            }
            NodeRef::Float(value) => match &value.kind {
                FloatKind::Bound(value) => push(&mut pending, NodeRef::Weak(value), charge)?,
                FloatKind::Decimal(value) => admit(1, value.retained_spelling_capacity(), charge)?,
                FloatKind::Constant(_) => {}
                FloatKind::Number(value) => push(&mut pending, NodeRef::Number(value), charge)?,
                FloatKind::Cast(_, value) | FloatKind::Negate(value) => {
                    admit(1, std::mem::size_of::<FloatExpr>(), charge)?;
                    push(&mut pending, NodeRef::Float(value), charge)?;
                }
                FloatKind::Binary(_, a, b) => {
                    admit(1, 2 * std::mem::size_of::<FloatExpr>(), charge)?;
                    push(&mut pending, NodeRef::Float(a), charge)?;
                    push(&mut pending, NodeRef::Float(b), charge)?;
                }
                FloatKind::Conditional(value) => {
                    admit(1, std::mem::size_of_val(&**value), charge)?;
                    push(&mut pending, NodeRef::Bool(&value.condition), charge)?;
                    push(&mut pending, NodeRef::Float(&value.then_value), charge)?;
                    push(&mut pending, NodeRef::Float(&value.else_value), charge)?;
                }
            },
            NodeRef::Number(value) => match &value.kind {
                NumberKind::Literal(_) | NumberKind::Typed(_) => {}
                NumberKind::FromBool(value) => {
                    admit(1, std::mem::size_of::<BoolExpr>(), charge)?;
                    push(&mut pending, NodeRef::Bool(value), charge)?;
                }
                NumberKind::FromFloat(_, value, _) => {
                    admit(1, std::mem::size_of::<FloatExpr>(), charge)?;
                    push(&mut pending, NodeRef::Float(value), charge)?;
                }
                NumberKind::Cast(_, value, _)
                | NumberKind::Negate(value, _)
                | NumberKind::Complement(value) => {
                    admit(1, std::mem::size_of::<NumberExpr>(), charge)?;
                    push(&mut pending, NodeRef::Number(value), charge)?;
                }
                NumberKind::Binary(_, a, b, _) => {
                    admit(1, 2 * std::mem::size_of::<NumberExpr>(), charge)?;
                    push(&mut pending, NodeRef::Number(a), charge)?;
                    push(&mut pending, NodeRef::Number(b), charge)?;
                }
                NumberKind::Conditional(value) => {
                    admit(1, std::mem::size_of_val(&**value), charge)?;
                    push(&mut pending, NodeRef::Bool(&value.condition), charge)?;
                    push(&mut pending, NodeRef::Number(&value.then_value), charge)?;
                    push(&mut pending, NodeRef::Number(&value.else_value), charge)?;
                }
            },
            NodeRef::Bool(value) => match value {
                BoolExpr::Constant(_) => {}
                BoolExpr::FromNumber(value) => {
                    admit(1, std::mem::size_of::<NumberExpr>(), charge)?;
                    push(&mut pending, NodeRef::Number(value), charge)?;
                }
                BoolExpr::FromFloat(value) => {
                    admit(1, std::mem::size_of::<FloatExpr>(), charge)?;
                    push(&mut pending, NodeRef::Float(value), charge)?;
                }
                BoolExpr::CompareFloats(_, a, b, _) => {
                    admit(1, 2 * std::mem::size_of::<FloatExpr>(), charge)?;
                    push(&mut pending, NodeRef::Float(a), charge)?;
                    push(&mut pending, NodeRef::Float(b), charge)?;
                }
                BoolExpr::CompareNumbers(_, a, b) => {
                    admit(1, 2 * std::mem::size_of::<NumberExpr>(), charge)?;
                    push(&mut pending, NodeRef::Number(a), charge)?;
                    push(&mut pending, NodeRef::Number(b), charge)?;
                }
                BoolExpr::Not(value) => {
                    admit(1, std::mem::size_of::<BoolExpr>(), charge)?;
                    push(&mut pending, NodeRef::Bool(value), charge)?;
                }
                BoolExpr::CompareBools(_, a, b) | BoolExpr::And(a, b) | BoolExpr::Or(a, b) => {
                    admit(1, 2 * std::mem::size_of::<BoolExpr>(), charge)?;
                    push(&mut pending, NodeRef::Bool(a), charge)?;
                    push(&mut pending, NodeRef::Bool(b), charge)?;
                }
                BoolExpr::Conditional(value) => {
                    admit(1, std::mem::size_of_val(&**value), charge)?;
                    push(&mut pending, NodeRef::Bool(&value.condition), charge)?;
                    push(&mut pending, NodeRef::Bool(&value.then_value), charge)?;
                    push(&mut pending, NodeRef::Bool(&value.else_value), charge)?;
                }
            },
        }
    }
    Ok(())
}
