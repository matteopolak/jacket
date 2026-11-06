//! Exact captured pointer derivations for labels; never a resource/liveness proof.
use super::{
    Block, Cell, Cells, CfgOp, Cx, Event, HashMap, HashSet, LifetimeBody, Op, Prepared, Reports,
    Span, Val,
};

const MAX_GRAPH: usize = 8192;
const MAX_QUERY: usize = 256;
const MAX_WORK: usize = 65536;

pub(super) struct Index {
    seeds: HashMap<Val, Option<Span>>,
    edges: HashMap<Val, Option<Vec<Val>>>,
    memo: HashMap<Val, Option<Span>>,
    remaining: usize,
}

/// Resolve derivation labels without changing diagnostic eligibility or witnesses.
pub(super) fn annotate(
    index: &mut Option<Option<Index>>,
    reports: &mut Reports,
    before: [usize; 4],
    value: Val,
    build: impl FnOnce() -> Option<Index>,
) {
    if !reports.added(before).any(|finding| {
        finding
            .secondary
            .iter()
            .any(|(_, label)| origin_label(label))
    }) {
        return;
    }
    let resolved = index
        .get_or_insert_with(build)
        .as_mut()
        .and_then(|i| i.origin(value));
    if let Some(origin) = resolved {
        for finding in reports.added(before) {
            for (span, label) in &mut finding.secondary {
                if origin_label(label) {
                    *span = origin;
                }
            }
        }
    }
}

fn origin_label(label: &str) -> bool {
    matches!(
        label,
        "storage derived here" | "borrowed storage originated here"
    )
}

impl Index {
    /// Build all producers before publishing any provenance. Missing/partial
    /// capture is unknown, not a later single-store or symbolic-name lookup.
    pub(super) fn build(
        cx: &Cx,
        body: &LifetimeBody,
        prepared: &Prepared<'_>,
        blocks: &[Block],
        cells: &Cells,
    ) -> Option<Self> {
        let mut index = Self {
            seeds: HashMap::default(),
            edges: HashMap::default(),
            memo: HashMap::default(),
            remaining: MAX_WORK,
        };
        let mut budget = MAX_GRAPH;
        for block in blocks {
            charge(&mut budget)?;
            if let CfgOp::Resource(Op::Seed {
                cell,
                span,
                ..
            }) = block.op
                && let Cell::Value(value) = cells.values[cell]
            {
                index
                    .seeds
                    .entry(value)
                    .and_modify(|old| {
                        if *old != Some(span) {
                            *old = None;
                        }
                    })
                    .or_insert(Some(span));
            }
        }
        for event in body.blocks.iter().flat_map(|b| &b.events) {
            charge(&mut budget)?;
            match event {
                Event::Copy {
                    value,
                    source,
                } if value != source => {
                    index.edge(*value, std::iter::once(*source), &mut budget)?;
                }
                Event::Load {
                    value,
                    ty,
                    ..
                } if cx.compiler.ty.types.is_pointer(*ty) => {
                    if let Some(sources) = prepared.captured_pointer_values(*value, &mut budget) {
                        index.edge(*value, sources, &mut budget)?;
                    } else {
                        index.edges.insert(*value, None);
                    }
                    if budget == 0 {
                        return None;
                    }
                }
                Event::Offset {
                    value,
                    source,
                    ..
                }
                | Event::View {
                    value,
                    source,
                    ..
                } if value != source => {
                    index.edges.insert(*value, None);
                }
                _ => {}
            }
        }
        Some(index)
    }

    fn edge(
        &mut self,
        value: Val,
        sources: impl Iterator<Item = Val>,
        budget: &mut usize,
    ) -> Option<()> {
        for source in sources {
            charge(budget)?;
            if let Some(values) = self.edges.entry(value).or_insert_with(|| Some(Vec::new())) {
                // Captured sources are a set. Canonical order keeps fuel-limited
                // label precision independent of its randomized iteration order.
                *budget = budget.checked_sub(values.len().checked_mul(2)?)?;
                let at = values
                    .iter()
                    .position(|old| old.0 > source.0)
                    .unwrap_or(values.len());
                values.insert(at, source);
            }
        }
        Some(())
    }

    /// All captured alternatives must terminate at the same actual origin.
    /// A failed query retains the candidate's original label.
    pub(super) fn origin(&mut self, value: Val) -> Option<Span> {
        let mut query = Query {
            seeds: &self.seeds,
            edges: &self.edges,
            memo: &mut self.memo,
            remaining: &mut self.remaining,
            local: MAX_QUERY,
            active: HashSet::default(),
        };
        query.resolve(value)
    }
}

fn charge(budget: &mut usize) -> Option<()> {
    if jaic::interrupt::should_stop() {
        return None;
    }
    *budget = budget.checked_sub(1)?;
    Some(())
}

struct Query<'a> {
    seeds: &'a HashMap<Val, Option<Span>>,
    edges: &'a HashMap<Val, Option<Vec<Val>>>,
    memo: &'a mut HashMap<Val, Option<Span>>,
    remaining: &'a mut usize,
    local: usize,
    active: HashSet<Val>,
}

impl Query<'_> {
    fn resolve(&mut self, value: Val) -> Option<Span> {
        charge(&mut self.local)?;
        charge(self.remaining)?;
        if let Some(result) = self.memo.get(&value) {
            return *result;
        }
        if !self.active.insert(value) {
            return None;
        }
        let result = self.producers(value);
        self.active.remove(&value);
        self.memo.insert(value, result);
        result
    }

    fn producers(&mut self, value: Val) -> Option<Span> {
        let mut found = match self.seeds.get(&value) {
            Some(seed) => Some((*seed)?),
            None => None,
        };
        if let Some(edges) = self.edges.get(&value) {
            let count = edges.as_ref()?.len();
            if count == 0 {
                return None;
            }
            for i in 0..count {
                let source = self.edges.get(&value)?.as_ref()?[i];
                let origin = self.resolve(source)?;
                if found.is_some_and(|old| old != origin) {
                    return None;
                }
                found = Some(origin);
            }
        }
        found
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jaic::source::FileId;

    #[test]
    fn origin_index_is_not_built_without_new_origin_labels() {
        let mut reports = Reports::default();
        let mut index = None;
        let before = reports.counts();
        annotate(&mut index, &mut reports, before, Val(0), || {
            panic!("quiet body built origin graph")
        });
        reports.use_after_free.push(crate::Finding::default());
        annotate(&mut index, &mut reports, before, Val(0), || {
            panic!("unlabelled finding built origin graph")
        });
        assert!(index.is_none());
    }

    #[test]
    fn origin_index_attempt_and_label_roles_are_preserved() {
        let original = Span {
            file: FileId(0),
            start: 1,
            end: 2,
        };
        let allocation = Span {
            file: FileId(0),
            start: 3,
            end: 4,
        };
        let mut reports = Reports::default();
        reports.use_after_free.push(crate::Finding {
            secondary: vec![
                (original, "storage derived here".into()),
                (original, "storage invalidated here".into()),
                (original, "borrowed storage originated here".into()),
            ],
            ..crate::Finding::default()
        });
        let mut index = None;
        annotate(&mut index, &mut reports, [0; 4], Val(0), || {
            Some(Index {
                seeds: HashMap::from_iter([(Val(0), Some(allocation))]),
                edges: HashMap::default(),
                memo: HashMap::default(),
                remaining: MAX_WORK,
            })
        });
        assert_eq!(
            reports.use_after_free[0]
                .secondary
                .iter()
                .map(|(span, _)| *span)
                .collect::<Vec<_>>(),
            [allocation, original, allocation]
        );
        annotate(&mut index, &mut reports, [0; 4], Val(0), || {
            panic!("rebuilt graph")
        });
        let mut failed = None;
        annotate(&mut failed, &mut reports, [0; 4], Val(0), || None);
        annotate(&mut failed, &mut reports, [0; 4], Val(0), || {
            panic!("retried failed graph")
        });
    }

    #[test]
    fn captured_origin_requires_complete_acyclic_agreement_and_fuel() {
        let a = Span {
            file: FileId(0),
            start: 1,
            end: 2,
        };
        let b = Span {
            file: FileId(0),
            start: 3,
            end: 4,
        };
        let mut index = Index {
            seeds: HashMap::from_iter([(Val(0), Some(a)), (Val(5), Some(b))]),
            edges: HashMap::from_iter([
                (Val(1), Some(vec![Val(0)])),
                (Val(2), Some(vec![Val(3)])),
                (Val(3), Some(vec![Val(2)])),
                (Val(4), Some(vec![Val(0), Val(5)])),
                (Val(6), None),
                (Val(7), Some(vec![Val(0), Val(6)])),
            ]),
            memo: HashMap::default(),
            remaining: MAX_WORK,
        };
        let mut insertion_budget = MAX_GRAPH;
        index
            .edge(
                Val(9),
                [Val(5), Val(0), Val(5)].into_iter(),
                &mut insertion_budget,
            )
            .unwrap();
        assert_eq!(index.edges[&Val(9)], Some(vec![Val(0), Val(5), Val(5)]));
        assert_eq!(insertion_budget, MAX_GRAPH - 9);
        assert_eq!(index.origin(Val(1)), Some(a));
        for value in [2, 3, 4, 6, 7, 8] {
            assert_eq!(index.origin(Val(value)), None);
        }
        // An actual seed cannot hide a conflicting or unclassified producer.
        index.seeds.insert(Val(6), Some(a));
        index.memo.clear();
        assert_eq!(index.origin(Val(6)), None);
        for value in 10..300 {
            index.edges.insert(Val(value), Some(vec![Val(value + 1)]));
        }
        index.seeds.insert(Val(300), Some(a));
        assert_eq!(index.origin(Val(10)), None);
        index.remaining = 0;
        assert_eq!(index.origin(Val(1)), None);
    }
}
