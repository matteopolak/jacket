//! Side-effect-free native branch selection for the execution-phase predicate.
#[cfg(test)]
use jai_ir::BoolExpr;
mod bindings;
pub(crate) use bindings::{
    PhaseBindings, native_boolean_with_bindings, native_condition_with_bindings,
    native_value_condition,
};

mod loops;
pub(crate) use loops::{
    native_cases_terminates_with_bindings, native_loop_condition,
    native_statement_terminates_with_bindings, native_while_terminates,
};

/// Prove a branch result while allowing executed condition operands to have
/// effects. Callers must still emit the condition before selecting its arm.
#[cfg(test)]
pub(crate) fn native_condition(value: &BoolExpr) -> Option<bool> {
    native_condition_with_bindings(value, &PhaseBindings::new())
}

/// Only prove values whose executed operands have no runtime effects.
/// A discarded short-circuit operand need not be known or effect-free.
#[cfg(test)]
pub(crate) fn native_boolean(value: &BoolExpr) -> Option<bool> {
    native_boolean_with_bindings(value, &PhaseBindings::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_ir::Equality;
    #[test]
    fn native_phase_is_false_without_evaluating_skipped_operands() {
        assert_eq!(native_boolean(&BoolExpr::CompileTime), Some(false));
        assert_eq!(
            native_boolean(&BoolExpr::Not(Box::new(BoolExpr::CompileTime))),
            Some(true)
        );
        // The opposite order remains unknown if its executed operand can have effects.
        let unknown = BoolExpr::CompareInts(
            jai_ir::Relation::Equal,
            Box::new(jai_ir::IntExpr::constant(jai_types::Integer::wrapping(
                jai_types::IntegerType::S64,
                1,
            ))),
            Box::new(jai_ir::IntExpr::constant(jai_types::Integer::wrapping(
                jai_types::IntegerType::S64,
                2,
            ))),
        );
        assert_eq!(
            native_boolean(&BoolExpr::And(
                Box::new(BoolExpr::CompileTime),
                Box::new(unknown.clone())
            )),
            Some(false)
        );
        let with_unknown_left = BoolExpr::And(Box::new(unknown), Box::new(BoolExpr::CompileTime));
        assert_eq!(native_boolean(&with_unknown_left), None);
        assert_eq!(native_condition(&with_unknown_left), Some(false));
        let comparison = BoolExpr::CompareBools(
            Equality::Equal,
            Box::new(with_unknown_left),
            Box::new(BoolExpr::Constant(false)),
        );
        assert_eq!(native_boolean(&comparison), None);
        assert_eq!(native_condition(&comparison), Some(true));
    }
}
