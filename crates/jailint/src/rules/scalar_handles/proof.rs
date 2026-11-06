//! Invocation-local bounds for capability predicates and validity partitions.
use super::{Expr, HashMap, NativeKind, Relation, Token, guards};

const MAX_NODES: usize = 4096;
const MAX_SAMPLES: usize = 4096;

// Keeps the checked File wrapper (66 return alternatives) within the bound.
const MAX_WORK: usize = 8_388_608;

pub(super) struct CapabilityProof {
    partitions: HashMap<NativeKind, Vec<u64>>,
}

impl CapabilityProof {
    /// Bound the complete product before extraction. CFG and individual tree
    /// bounds alone do not bound exits × predicates × validity samples.
    pub(super) fn prepare(relation: &Relation) -> Option<Self> {
        let mut nodes = 0usize;
        let mut fresh = 0usize;
        let mut returns = 0usize;
        let mut scalar_returns = 0usize;
        let mut width = 1usize;
        let mut frames = 0usize;
        let mut kinds = HashMap::<NativeKind, &Token>::default();
        for state in &relation.exits {
            if jaic::interrupt::should_stop() {
                return None;
            }
            returns = returns.checked_add(state.returns.len())?;
            frames = frames.max(state.frames.values().map(Vec::len).max().unwrap_or(0));
            scalar_returns = scalar_returns.max(
                state
                    .returns
                    .iter()
                    .filter(|(_, slot, _)| slot.is_none())
                    .count(),
            );
            for expr in state
                .conditions
                .iter()
                .map(|(x, _)| x)
                .chain(state.returns.iter().map(|(_, _, x)| x))
            {
                nodes = nodes.checked_add(expr.shape()?.nodes)?;
                width = width.max(token_width(expr));
                if nodes > MAX_NODES {
                    return None;
                }
            }
            for (_, _, expr) in &state.returns {
                if let Expr::Fresh(token) = expr
                    && !state.invalid.contains(token)
                {
                    fresh = fresh.checked_add(1)?;
                    kinds.entry(token.kind).or_insert(token);
                }
            }
        }
        if fresh == 0 {
            return Some(Self {
                partitions: HashMap::default(),
            });
        }
        let raw = nodes.checked_mul(3)?.checked_add(7)?;
        let sorting = raw.checked_mul((usize::BITS - raw.leading_zeros()) as usize + 1)?;
        let mut work = sorting.checked_add(nodes)?.checked_mul(kinds.len())?;
        if work > MAX_WORK {
            return None;
        }
        let mut partitions = HashMap::default();
        for (kind, token) in kinds {
            if jaic::interrupt::should_stop() {
                return None;
            }
            let samples = guards::samples(relation, token);
            if samples.len() > MAX_SAMPLES {
                return None;
            }
            partitions.insert(kind, samples);
        }
        let samples = partitions.values().map(Vec::len).max().unwrap_or(0);
        let evaluations = fresh
            .checked_mul(scalar_returns.checked_mul(2)?.checked_add(1)?)?
            .checked_mul(samples)?
            .checked_mul(nodes)?
            .checked_mul(width)?;
        let comparisons = fresh
            .checked_mul(2)?
            .checked_mul(scalar_returns)?
            .checked_mul(returns)?
            .checked_mul(width.checked_add(4)?)?;
        let dedup = fresh
            .checked_mul(fresh)?
            .checked_mul(width.checked_add(frames)?.checked_add(8)?)?;
        work = work
            .checked_add(evaluations)?
            .checked_add(comparisons)?
            .checked_add(dedup)?;
        (work <= MAX_WORK && !jaic::interrupt::should_stop()).then_some(Self {
            partitions,
        })
    }

    pub(super) fn samples(&self, kind: NativeKind) -> Option<&[u64]> {
        self.partitions.get(&kind).map(Vec::as_slice)
    }
}

/// Token comparison cost includes its bounded structural call chain.
fn token_width(expr: &Expr) -> usize {
    match expr {
        Expr::Fresh(token) => token.sites.len() + 1,
        Expr::Compare(child, ..) | Expr::Not(child) => token_width(child),
        Expr::And(left, right) | Expr::Or(left, right) => token_width(left).max(token_width(right)),
        _ => 1,
    }
}
