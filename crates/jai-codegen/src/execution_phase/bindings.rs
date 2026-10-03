//! Scoped facts preserve phase selection through already evaluated SSA captures.
use jai_ir::{BoolExpr, Equality, ExpressionBindingId, ValueExpr};
use std::collections::HashMap;

pub(crate) type PhaseBindings = HashMap<ExpressionBindingId, Option<bool>>;

struct Facts<'a> {
    outer: &'a PhaseBindings,
    local: PhaseBindings,
    #[cfg(test)]
    installations: usize,
    #[cfg(test)]
    lookups: std::cell::Cell<usize>,
}
impl<'a> Facts<'a> {
    fn new(outer: &'a PhaseBindings) -> Self {
        Self {
            outer,
            local: PhaseBindings::new(),
            #[cfg(test)]
            installations: 0,
            #[cfg(test)]
            lookups: std::cell::Cell::new(0),
        }
    }
    fn get(&self, id: &ExpressionBindingId) -> Option<&Option<bool>> {
        #[cfg(test)]
        self.lookups.set(self.lookups.get() + 1);
        // A stored unknown is present and must suppress an outer known fact.
        self.local.get(id).or_else(|| self.outer.get(id))
    }
}

pub(crate) fn native_value_condition(value: &ValueExpr, bindings: &PhaseBindings) -> Option<bool> {
    value_condition(value, &mut Facts::new(bindings))
}
pub(crate) fn native_condition_with_bindings(
    value: &BoolExpr,
    bindings: &PhaseBindings,
) -> Option<bool> {
    condition(value, &mut Facts::new(bindings))
}
pub(crate) fn native_boolean_with_bindings(
    value: &BoolExpr,
    bindings: &PhaseBindings,
) -> Option<bool> {
    boolean(value, &mut Facts::new(bindings))
}

fn value_condition(value: &ValueExpr, bindings: &mut Facts<'_>) -> Option<bool> {
    match value {
        ValueExpr::StorageBitcast {
            source: jai_ir::StorageBitcastSource::Value(source),
            cast,
        } if cast.source_type() == cast.target_type() => value_condition(source, bindings),
        ValueExpr::Bool(value) => condition(value, bindings),
        ValueExpr::Bound { binding, .. } => bindings.get(binding).copied().flatten(),
        ValueExpr::Bind {
            bindings: producers,
            body,
            ..
        } => {
            let mut previous = Vec::with_capacity(producers.len());
            for (id, producer) in producers {
                let fact = value_condition(producer, bindings);
                previous.push((*id, bindings.local.insert(*id, fact)));
                #[cfg(test)]
                {
                    bindings.installations += 1;
                }
            }
            let result = value_condition(body, bindings);
            for (id, old) in previous.into_iter().rev() {
                match old {
                    Some(fact) => {
                        bindings.local.insert(id, fact);
                    }
                    None => {
                        bindings.local.remove(&id);
                    }
                }
            }
            result
        }
        ValueExpr::Conditional { expression, .. } => {
            match condition(&expression.condition, bindings) {
                Some(true) => value_condition(&expression.then_value, bindings),
                Some(false) => value_condition(&expression.else_value, bindings),
                None => {
                    let left = value_condition(&expression.then_value, bindings)?;
                    (value_condition(&expression.else_value, bindings)? == left).then_some(left)
                }
            }
        }
        _ => None,
    }
}

/// This result does not authorize discarding executed condition producers.
fn condition(value: &BoolExpr, bindings: &mut Facts<'_>) -> Option<bool> {
    if let Some(result) = boolean(value, bindings) {
        return Some(result);
    }
    match value {
        BoolExpr::Value(value) => value_condition(value, bindings),
        BoolExpr::Not(value) => condition(value, bindings).map(|value| !value),
        BoolExpr::And(left, right) => match (condition(left, bindings), condition(right, bindings))
        {
            (Some(false), _) | (_, Some(false)) => Some(false),
            (Some(true), value) | (value, Some(true)) => value,
            _ => None,
        },
        BoolExpr::Or(left, right) => {
            match (condition(left, bindings), condition(right, bindings)) {
                (Some(true), _) | (_, Some(true)) => Some(true),
                (Some(false), value) | (value, Some(false)) => value,
                _ => None,
            }
        }
        BoolExpr::CompareBools(op, left, right) => {
            let left = condition(left, bindings)?;
            let right = condition(right, bindings)?;
            Some(match op {
                Equality::Equal => left == right,
                Equality::NotEqual => left != right,
            })
        }
        BoolExpr::Conditional(value) => match condition(&value.condition, bindings) {
            Some(true) => condition(&value.then_value, bindings),
            Some(false) => condition(&value.else_value, bindings),
            None => {
                let left = condition(&value.then_value, bindings)?;
                (condition(&value.else_value, bindings)? == left).then_some(left)
            }
        },
        _ => None,
    }
}

/// Already bound values are pure reads; a Bind itself must emit all producers.
fn boolean(value: &BoolExpr, bindings: &mut Facts<'_>) -> Option<bool> {
    match value {
        BoolExpr::CompileTime => Some(false),
        BoolExpr::Constant(value) => Some(*value),
        BoolExpr::Value(value) => native_value_boolean(value, bindings),
        BoolExpr::Not(value) => boolean(value, bindings).map(|value| !value),
        BoolExpr::And(left, right) => match boolean(left, bindings)? {
            false => Some(false),
            true => boolean(right, bindings),
        },
        BoolExpr::Or(left, right) => match boolean(left, bindings)? {
            true => Some(true),
            false => boolean(right, bindings),
        },
        BoolExpr::CompareBools(op, left, right) => {
            let left = boolean(left, bindings)?;
            let right = boolean(right, bindings)?;
            Some(match op {
                Equality::Equal => left == right,
                Equality::NotEqual => left != right,
            })
        }
        BoolExpr::Conditional(value) => match boolean(&value.condition, bindings)? {
            true => boolean(&value.then_value, bindings),
            false => boolean(&value.else_value, bindings),
        },
        _ => None,
    }
}

fn native_value_boolean(value: &ValueExpr, bindings: &mut Facts<'_>) -> Option<bool> {
    match value {
        ValueExpr::Bool(value) => boolean(value, bindings),
        ValueExpr::Bound { binding, .. } => bindings.get(binding).copied().flatten(),
        ValueExpr::Conditional { expression, .. } => {
            match boolean(&expression.condition, bindings)? {
                true => native_value_boolean(&expression.then_value, bindings),
                false => native_value_boolean(&expression.else_value, bindings),
            }
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn captured_phase_facts_select_nested_bodies_without_discarding_producers() {
        let types = jai_types::TypeRegistry::new();
        let ty = types.scalar(jai_types::ScalarType::Bool);
        let id = ExpressionBindingId::new(jai_ir::ProcedureId::new(0), 0);
        let next = ExpressionBindingId::new(id.procedure(), 1);
        let capture = ValueExpr::Bound { binding: id, ty };
        let expression = ValueExpr::Bind {
            bindings: vec![
                (id, ValueExpr::Bool(BoolExpr::CompileTime)),
                (
                    next,
                    ValueExpr::Bool(BoolExpr::Not(Box::new(BoolExpr::Value(Box::new(capture))))),
                ),
            ],
            body: Box::new(ValueExpr::Bound { binding: next, ty }),
            ty,
        };
        assert_eq!(
            native_value_condition(&expression, &PhaseBindings::new()),
            Some(true)
        );
        let condition = BoolExpr::Value(Box::new(expression));
        assert_eq!(
            native_condition_with_bindings(&condition, &PhaseBindings::new()),
            Some(true)
        );
        assert_eq!(
            native_boolean_with_bindings(&condition, &PhaseBindings::new()),
            None
        );
    }

    #[test]
    fn scoped_unknown_fact_never_inherits_an_unrelated_phase_value() {
        let types = jai_types::TypeRegistry::new();
        let ty = types.scalar(jai_types::ScalarType::Bool);
        let outer = ExpressionBindingId::new(jai_ir::ProcedureId::new(0), 0);
        let inner = ExpressionBindingId::new(outer.procedure(), 1);
        let facts = PhaseBindings::from([(outer, Some(false)), (inner, None)]);
        assert_eq!(
            native_value_condition(&ValueExpr::Bound { binding: outer, ty }, &facts),
            Some(false)
        );
        assert_eq!(
            native_value_condition(&ValueExpr::Bound { binding: inner, ty }, &facts),
            None
        );
        assert_eq!(
            native_value_condition(
                &ValueExpr::Bound { binding: outer, ty },
                &PhaseBindings::new()
            ),
            None
        );
        let mut overlay = Facts::new(&facts);
        overlay.local.insert(outer, None);
        assert_eq!(
            value_condition(&ValueExpr::Bound { binding: outer, ty }, &mut overlay),
            None
        );
        overlay.local.remove(&outer);
        assert_eq!(
            value_condition(&ValueExpr::Bound { binding: outer, ty }, &mut overlay),
            Some(false)
        );
    }

    #[test]
    fn nested_empty_scopes_do_not_copy_or_scan_a_large_outer_environment() {
        let types = jai_types::TypeRegistry::new();
        let ty = types.scalar(jai_types::ScalarType::Bool);
        let procedure = jai_ir::ProcedureId::new(0);
        let id = ExpressionBindingId::new(procedure, 0);
        let outer: PhaseBindings = (0..10_000)
            .map(|index| (ExpressionBindingId::new(procedure, index), Some(false)))
            .collect();
        let mut value = ValueExpr::Bound { binding: id, ty };
        for _ in 0..500 {
            value = ValueExpr::Bind {
                bindings: vec![],
                body: Box::new(value),
                ty,
            };
        }
        let mut facts = Facts::new(&outer);
        assert_eq!(value_condition(&value, &mut facts), Some(false));
        assert_eq!(facts.lookups.get(), 1);
        assert_eq!(facts.installations, 0);
        assert_eq!(facts.local.capacity(), 0);
        assert_eq!(outer.len(), 10_000);
    }

    #[test]
    fn identity_storage_cast_preserves_a_fact_but_keeps_its_evaluation() {
        let types = jai_types::TypeRegistry::new();
        let ty = types.scalar(jai_types::ScalarType::Bool);
        let cast = jai_types::StorageBitcast::prove(
            &types,
            jai_types::LayoutPolicy::lp64(),
            ty,
            ty,
            jai_types::StorageBitcastStrength::EqualSize,
        )
        .unwrap();
        let value = ValueExpr::StorageBitcast {
            source: jai_ir::StorageBitcastSource::Value(Box::new(ValueExpr::Bool(
                BoolExpr::CompileTime,
            ))),
            cast,
        };
        assert_eq!(
            native_value_condition(&value, &PhaseBindings::new()),
            Some(false)
        );
        assert_eq!(
            native_boolean_with_bindings(&BoolExpr::Value(Box::new(value)), &PhaseBindings::new()),
            None
        );
    }
}
