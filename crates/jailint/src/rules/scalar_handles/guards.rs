use super::{
    CmpOp, Exit, Expr, NativeKind, Opening, Relation, ResultGuard, ReturnCapability, State, Token,
    comparison_constant, proof,
};

/// Evaluate only comparisons whose truth is invariant for a validity class.
fn evaluate(x: &Expr, token: Option<&Token>, sample: u64) -> Option<u64> {
    match x {
        Expr::Constant(k) => Some(*k),
        Expr::Fresh(t) if Some(t) == token => Some(sample),
        Expr::Not(a) => Some(u64::from(evaluate(a, token, sample)? == 0)),
        Expr::And(a, b) => {
            let left = evaluate(a, token, sample);
            if left == Some(0) {
                Some(0)
            } else if left.is_some() {
                Some(u64::from(evaluate(b, token, sample)? != 0))
            } else if evaluate(b, token, sample) == Some(0) {
                Some(0)
            } else {
                None
            }
        }
        Expr::Or(a, b) => {
            let left = evaluate(a, token, sample);
            if left.is_some_and(|v| v != 0) {
                Some(1)
            } else if left.is_some() {
                Some(u64::from(evaluate(b, token, sample)? != 0))
            } else if evaluate(b, token, sample).is_some_and(|v| v != 0) {
                Some(1)
            } else {
                None
            }
        }
        Expr::Compare(a, op, k, ty) => {
            let value = comparison_constant(*ty, *op, evaluate(a, token, sample)?)?;
            let k = comparison_constant(*ty, *op, *k)?;
            Some(u64::from(match op {
                CmpOp::Eq => value == k,
                CmpOp::Ne => value != k,
                CmpOp::SLt => (value as i64) < (k as i64),
                CmpOp::SLe => (value as i64) <= (k as i64),
                CmpOp::SGt => (value as i64) > (k as i64),
                CmpOp::SGe => (value as i64) >= (k as i64),
                CmpOp::ULt => value < k,
                CmpOp::ULe => value <= k,
                CmpOp::UGt => value > k,
                CmpOp::UGe => value >= k,
                _ => return None,
            }))
        }
        _ => None,
    }
}

pub(super) fn feasible(conditions: &[(Expr, bool)], token: &Token, sample: u64) -> bool {
    conditions
        .iter()
        .all(|(x, truth)| evaluate(x, Some(token), sample).is_none_or(|v| (v != 0) == *truth))
}

pub(super) fn capabilities(relation: &Relation) -> Opening {
    let Some(proof) = proof::CapabilityProof::prepare(relation) else {
        return Opening::default();
    };
    let mut out = Opening::default();
    for exit in &relation.exits {
        if jaic::interrupt::should_stop() {
            return Opening::default();
        }
        for (result, slot, x) in &exit.returns {
            if jaic::interrupt::should_stop() {
                return Opening::default();
            }
            let Expr::Fresh(token) = x else {
                continue;
            };
            if exit.invalid.contains(token) {
                continue;
            }
            let Some(samples) = proof.samples(token.kind) else {
                return Opening::default();
            };
            if !samples
                .iter()
                .any(|v| token.validity.accepts(*v) && feasible(&exit.conditions, token, *v))
            {
                continue;
            }
            let guard = result_guard(relation, exit, token, *result, *slot, samples);
            if jaic::interrupt::should_stop() {
                return Opening::default();
            }
            let capability = ReturnCapability {
                result: *result,
                slot: *slot,
                kind: token.kind,
                validity: token.validity,
                guard,
                template: token.sites.clone(),
                requires_noninterference: exit.frames.get(token).cloned().unwrap_or_default(),
            };
            out.incomplete |= !capability.requires_noninterference.is_empty();
            if !out.alternatives.contains(&capability) {
                out.alternatives.push(capability);
            }
        }
    }
    out
}

/// Compare one other scalar return against native validity across every matching exit.
fn result_guard(
    relation: &Relation,
    exit: &Exit,
    token: &Token,
    result: usize,
    slot: Option<super::Slot>,
    samples: &[u64],
) -> Option<ResultGuard> {
    for (index, other_slot, _) in &exit.returns {
        if *index == result || other_slot.is_some() {
            continue;
        }
        for valid_when in [true, false] {
            if jaic::interrupt::should_stop() {
                return None;
            }
            let matches = relation
                .exits
                .iter()
                .filter(|state| {
                    state.returns.iter().any(|(r, p, x)| {
                        *r == result && *p == slot && x == &Expr::Fresh(token.clone())
                    })
                })
                .all(|state| guard_exit(state, token, *index, valid_when, samples));
            if matches {
                return Some(ResultGuard {
                    result: *index,
                    valid_when,
                });
            }
        }
    }
    None
}

fn guard_exit(
    state: &Exit,
    token: &Token,
    index: usize,
    valid_when: bool,
    samples: &[u64],
) -> bool {
    if jaic::interrupt::should_stop() {
        return false;
    }
    let Some((_, _, predicate)) = state
        .returns
        .iter()
        .find(|(i, p, _)| *i == index && p.is_none())
    else {
        return false;
    };
    samples.iter().all(|value| {
        !jaic::interrupt::should_stop()
            && (!feasible(&state.conditions, token, *value)
                || evaluate(predicate, Some(token), *value)
                    .is_some_and(|b| (b != 0) == (token.validity.accepts(*value) == valid_when)))
    })
}

// Comparisons partition the integer line at their constants. Include both adjacent
// intervals and signed/unsigned boundaries; this is exhaustive for this expression algebra.
pub(super) fn samples(relation: &Relation, token: &Token) -> Vec<u64> {
    fn constants(x: &Expr, out: &mut Vec<u64>) {
        match x {
            Expr::Compare(a, _, k, _) => {
                out.extend([*k, k.wrapping_add(1), k.wrapping_sub(1)]);
                constants(a, out);
            }
            Expr::Not(a) => constants(a, out),
            Expr::And(a, b) | Expr::Or(a, b) => {
                constants(a, out);
                constants(b, out);
            }
            _ => {}
        }
    }
    let mut out = vec![0, 1, u64::MAX, i64::MAX as u64, i64::MIN as u64];
    for s in &relation.exits {
        for (x, _) in &s.conditions {
            constants(x, &mut out);
        }
        for (_, _, x) in &s.returns {
            constants(x, &mut out);
        }
    }
    if token.kind == NativeKind::Descriptor {
        out = out.into_iter().map(|v| (v as i32 as i64) as u64).collect();
        out.extend([i32::MIN as i64 as u64, i32::MAX as u64]);
    }
    out.sort_unstable();
    out.dedup();
    out
}

pub(super) fn constant_truth(x: &Expr) -> Option<bool> {
    evaluate(x, None, 0).map(|v| v != 0)
}

pub(super) fn may_alias(state: &State, token: &Token, value: &Expr) -> bool {
    match value {
        Expr::Fresh(other) => {
            if other == token {
                return true;
            }
            // Distinct currently-live acquisitions cannot share one API value.
            // Once an older acquisition is closed or uncertain, reuse is possible.
            !(other.kind == token.kind
                && !state.invalid.contains(other)
                && state.frames.get(other).is_none_or(Vec::is_empty)
                && state.frames.get(token).is_none_or(Vec::is_empty))
        }
        Expr::Constant(value) => {
            let value = if token.kind == NativeKind::Descriptor {
                (*value as i32 as i64) as u64
            } else {
                *value
            };
            token.validity.accepts(value) && feasible(&state.conditions, token, value)
        }
        Expr::Input(..) | Expr::Unknown => true,
        // Semantic boolean expressions have only API values zero and one.
        _ => [0, 1].into_iter().any(|sample| {
            token.validity.accepts(sample)
                && feasible(&state.conditions, token, sample)
                && evaluate(value, Some(token), sample).is_none_or(|v| v == sample)
        }),
    }
}
