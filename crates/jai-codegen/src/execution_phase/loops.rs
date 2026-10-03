//! Native control paths retain exact loop identities across nested transfers.
use super::{PhaseBindings, native_condition_with_bindings};
use jai_ir::{Block, Cases, LoopCondition, LoopId, Statement, Transfer};
use std::collections::HashSet;

struct Paths {
    falls_through: bool,
    breaks: HashSet<LoopId>,
}
impl Paths {
    fn open() -> Self {
        Self {
            falls_through: true,
            breaks: HashSet::new(),
        }
    }
    fn merge(&mut self, other: Self) {
        self.falls_through |= other.falls_through;
        self.breaks.extend(other.breaks);
    }
}

pub(crate) fn native_loop_condition(
    condition: &LoopCondition,
    facts: &PhaseBindings,
) -> Option<bool> {
    match condition {
        LoopCondition::Value(value) | LoopCondition::BoundBool(_, value) => {
            native_condition_with_bindings(value, facts)
        }
        LoopCondition::BoundInt(_, _) => None,
    }
}

pub(crate) fn native_while_terminates(
    id: LoopId,
    condition: &LoopCondition,
    body: &Block,
    facts: &PhaseBindings,
) -> bool {
    native_loop_condition(condition, facts) == Some(true)
        && !block_paths(body, facts).breaks.contains(&id)
}

pub(crate) fn native_statement_terminates_with_bindings(
    statement: &Statement,
    facts: &PhaseBindings,
) -> bool {
    !statement_paths(statement, facts).falls_through
}

pub(crate) fn native_cases_terminates_with_bindings(case: &Cases, facts: &PhaseBindings) -> bool {
    !case_paths(case, facts).falls_through
}

fn block_paths(block: &Block, facts: &PhaseBindings) -> Paths {
    let mut result = Paths::open();
    for statement in &block.statements {
        if !result.falls_through {
            break;
        }
        let paths = statement_paths(statement, facts);
        result.falls_through = paths.falls_through;
        result.breaks.extend(paths.breaks);
    }
    result
}

fn statement_paths(statement: &Statement, facts: &PhaseBindings) -> Paths {
    match statement {
        Statement::Exit(exit) => {
            let mut result = Paths::open();
            result.falls_through = false;
            if let Transfer::Break(id) = exit.transfer {
                result.breaks.insert(id);
            }
            result
        }
        Statement::Block(block) | Statement::PushContext { body: block, .. } => {
            block_paths(block, facts)
        }
        Statement::If(condition, yes, no) => match native_condition_with_bindings(condition, facts)
        {
            Some(true) => block_paths(yes, facts),
            Some(false) => block_paths(no, facts),
            None => {
                let mut paths = block_paths(yes, facts);
                paths.merge(block_paths(no, facts));
                paths
            }
        },
        Statement::Cases(case) => case_paths(case, facts),
        Statement::While {
            id,
            condition,
            body,
        } => {
            let condition = native_loop_condition(condition, facts);
            if condition == Some(false) {
                return Paths::open();
            }
            let mut paths = block_paths(body, facts);
            let own_break = paths.breaks.remove(id);
            paths.falls_through = condition != Some(true) || own_break;
            paths
        }
        Statement::Range(range) => {
            let mut paths = block_paths(&range.body, facts);
            paths.breaks.remove(&range.id);
            // Range bounds are not phase boolean proofs: retain possible completion.
            paths.falls_through = true;
            paths
        }
        _ => Paths::open(),
    }
}

fn case_paths(case: &Cases, facts: &PhaseBindings) -> Paths {
    let mut result = case
        .default
        .as_ref()
        .map_or_else(Paths::open, |block| block_paths(block, facts));
    let mut next_falls_through = result.falls_through;
    if case.default.is_none() {
        result.falls_through = !case.exhaustive;
    }
    for arm in case.arms.iter().rev() {
        let mut paths = block_paths(&arm.body, facts);
        paths.falls_through &= !arm.through || next_falls_through;
        next_falls_through = paths.falls_through;
        result.merge(paths);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_ir::{BoolExpr, Exit, Flow};

    fn block(statements: Vec<Statement>) -> Block {
        Block {
            statements,
            flow: Flow::FallsThrough,
        }
    }
    fn break_to(id: LoopId) -> Statement {
        Statement::Exit(Exit {
            cleanups: vec![],
            transfer: Transfer::Break(id),
        })
    }
    fn forever(id: LoopId, body: Block) -> Statement {
        Statement::While {
            id,
            condition: LoopCondition::Value(BoolExpr::Not(Box::new(BoolExpr::CompileTime))),
            body,
        }
    }

    #[test]
    fn nested_break_targets_are_consumed_only_by_their_own_loop() {
        let outer = LoopId::new(0);
        let inner = LoopId::new(1);
        let facts = PhaseBindings::new();
        assert!(native_statement_terminates_with_bindings(
            &forever(
                outer,
                block(vec![forever(inner, block(vec![break_to(inner)]))])
            ),
            &facts
        ));
        assert!(!native_statement_terminates_with_bindings(
            &forever(
                outer,
                block(vec![forever(inner, block(vec![break_to(outer)]))])
            ),
            &facts
        ));
    }

    #[test]
    fn only_reachable_phase_selected_breaks_restore_loop_completion() {
        let id = LoopId::new(0);
        let facts = PhaseBindings::new();
        let guarded = Statement::If(
            BoolExpr::CompileTime,
            block(vec![break_to(id)]),
            block(vec![]),
        );
        assert!(native_statement_terminates_with_bindings(
            &forever(id, block(vec![guarded])),
            &facts
        ));
        let continuing = Statement::Exit(Exit {
            cleanups: vec![],
            transfer: Transfer::Continue(id),
        });
        assert!(native_statement_terminates_with_bindings(
            &forever(id, block(vec![continuing, break_to(id)])),
            &facts
        ));
    }

    #[test]
    fn scoped_captured_facts_select_break_guards_without_guessing_unknowns() {
        let id = LoopId::new(0);
        let binding = jai_ir::ExpressionBindingId::new(jai_ir::ProcedureId::new(0), 0);
        let types = jai_types::TypeRegistry::new();
        let guard = BoolExpr::Value(Box::new(jai_ir::ValueExpr::Bound {
            binding,
            ty: types.scalar(jai_types::ScalarType::Bool),
        }));
        let statement = forever(
            id,
            block(vec![Statement::If(
                guard,
                block(vec![break_to(id)]),
                block(vec![]),
            )]),
        );
        let mut facts = PhaseBindings::new();
        assert!(!native_statement_terminates_with_bindings(
            &statement, &facts
        ));
        facts.insert(binding, Some(false));
        assert!(native_statement_terminates_with_bindings(
            &statement, &facts
        ));
        facts.insert(binding, None);
        assert!(!native_statement_terminates_with_bindings(
            &statement, &facts
        ));
    }
}
