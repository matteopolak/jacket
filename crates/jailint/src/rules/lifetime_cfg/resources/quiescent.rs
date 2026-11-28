//! Prove quiet candidates without a value-state fixed point when no observable kill exists.
use super::super::tasks::Op as Task;
use super::{
    CfgOp, Cx, DEAD, HashMap, HashSet, LIVE, Model, Op, Origin, Span, UNKNOWN, VecDeque, kills,
};

/// Candidate-independent operation sets. Unknown future resource operations stay in
/// the general bucket, where the authoritative kill predicate still decides them.
#[derive(Default)]
pub(super) struct Index {
    observed: HashSet<usize>,
    /// The blocks that observe each cell.
    observations: HashMap<usize, Vec<usize>>,
    /// Every released call's span, for the spans a use may sit inside.
    frees: FreeSpans,
    /// The distinct task tokens, in the order their blocks appear.
    tokens: Vec<super::super::tasks::Token>,
    /// What is copied into each cell.
    sources: HashMap<usize, Vec<usize>>,
    /// The spans of the heap allocations seeding each cell.
    heap_seeds: HashMap<usize, Vec<Span>>,
    escaped: Vec<usize>,
    retained: Vec<usize>,
    general: Vec<usize>,
    temporary: Vec<usize>,
    scope: Vec<usize>,
    statement: Vec<usize>,
    until: Vec<usize>,
}

impl Index {
    pub(super) fn new(model: &Model) -> Self {
        let mut out = Self::default();
        let mut frees = Vec::new();

        for (i, block) in model.blocks.iter().enumerate() {
            match &block.op {
                CfgOp::Use(cell, _)
                | CfgOp::Resource(
                    Op::Read {
                        cell, ..
                    }
                    | Op::Free {
                        cell, ..
                    }
                    | Op::FreeRegion {
                        cell, ..
                    },
                ) => {
                    out.observed.insert(*cell);
                    out.observations.entry(*cell).or_default().push(i);
                }
                CfgOp::Resource(
                    Op::Return {
                        cell, ..
                    }
                    | Op::Escape {
                        cell, ..
                    },
                ) => {
                    out.observed.insert(*cell);
                    out.observations.entry(*cell).or_default().push(i);
                    out.escaped.push(*cell);
                }
                CfgOp::Task(Task::Start {
                    cells, ..
                }) => {
                    out.observed.extend(cells);

                    for &cell in cells {
                        out.observations.entry(cell).or_default().push(i);
                    }

                    out.retained.extend(cells);
                }
                CfgOp::Copy(pairs) => {
                    for &(target, source) in pairs {
                        out.sources.entry(target).or_default().push(source);
                    }
                }
                CfgOp::Resource(Op::Seed {
                    cell,
                    origin: Origin::Heap,
                    span,
                    ..
                }) => out.heap_seeds.entry(*cell).or_default().push(*span),
                _ => {}
            }

            if let CfgOp::Task(
                Task::Start {
                    token, ..
                }
                | Task::End(token)
                | Task::MayEnd(token),
            ) = &block.op
                && !out.tokens.contains(token)
            {
                out.tokens.push(*token);
            }

            if let CfgOp::Resource(Op::Free {
                span, ..
            }) = &block.op
            {
                frees.push(*span);
            }

            if let CfgOp::Resource(op) = &block.op {
                match op {
                    Op::ScopeEnd(..) => out.scope.push(i),
                    Op::StatementEnd(..) => out.statement.push(i),
                    Op::Reset(..) | Op::MayReset(..) | Op::Restore(..) | Op::MayRestore(..) => {
                        out.temporary.push(i);
                    }
                    Op::Advance(..) | Op::MayAdvance(..) => out.until.push(i),
                    Op::Read {
                        ..
                    }
                    | Op::Seed {
                        ..
                    }
                    | Op::Return {
                        ..
                    }
                    | Op::Escape {
                        ..
                    }
                    | Op::Mark(..)
                    | Op::Exit => {}
                    _ => out.general.push(i),
                }
            }
        }

        out.frees = FreeSpans::new(frees);
        out
    }

    /// The blocks that observe any of `cells`.
    pub(super) fn observations<'a>(
        &'a self,
        cells: impl IntoIterator<Item = &'a usize> + 'a,
    ) -> impl Iterator<Item = usize> + 'a {
        cells
            .into_iter()
            .filter_map(|c| self.observations.get(c))
            .flatten()
            .copied()
    }

    pub(super) fn frees(&self) -> &FreeSpans {
        &self.frees
    }

    pub(super) fn tokens(&self) -> &[super::super::tasks::Token] {
        &self.tokens
    }

    pub(super) fn sources(&self, cell: usize) -> &[usize] {
        self.sources.get(&cell).map_or(&[], Vec::as_slice)
    }

    pub(super) fn heap_seeds(&self, cell: usize) -> &[Span] {
        self.heap_seeds.get(&cell).map_or(&[], Vec::as_slice)
    }

    /// Without any applicable invalidation, a non-local region cannot produce a
    /// release diagnostic, regardless of its copy closure or retained borrowers.
    /// Local regions still need escape checks even when their storage stays live.
    pub(super) fn independent_quiet(&self, kind: Origin) -> bool {
        self.general.is_empty()
            && match kind {
                Origin::Heap | Origin::Incoming => true,
                Origin::Until(_) => self.until.is_empty(),
                Origin::Stack(_) | Origin::Statement(_) | Origin::Temporary => false,
            }
    }

    pub(super) fn used(&self, relevant: &HashMap<usize, usize>) -> bool {
        relevant.keys().any(|c| self.observed.contains(c))
    }
}

pub(super) fn observable(model: &Model) -> Vec<bool> {
    let mut incoming = vec![Vec::new(); model.blocks.len()];

    for (from, block) in model.blocks.iter().enumerate() {
        for &to in &block.next {
            incoming[to].push(from);
        }
    }

    let mut queue: VecDeque<_> = model
        .blocks
        .iter()
        .enumerate()
        .filter_map(|(i, b)| {
            matches!(
                b.op,
                CfgOp::Use(..)
                    | CfgOp::Task(Task::Start { .. })
                    | CfgOp::Resource(Op::Read { .. } | Op::Free { .. } | Op::FreeRegion { .. })
            )
            .then_some(i)
        })
        .collect();
    let mut reachable = vec![false; model.blocks.len()];

    while let Some(i) = queue.pop_front() {
        if !reachable[i] {
            reachable[i] = true;
            queue.extend(&incoming[i]);
        }
    }

    reachable
}

pub(super) fn quiet(
    cx: &Cx,
    model: &Model,
    kind: Origin,
    region: Option<usize>,
    relevant: &HashMap<usize, usize>,
    observable: &[bool],
    operations: &Index,
) -> bool {
    let index = |cell| relevant.get(&cell).copied();
    let local = matches!(
        kind,
        Origin::Stack(_) | Origin::Statement(_) | Origin::Temporary
    );
    let escapes = operations.escaped.iter().any(|c| relevant.contains_key(c));
    let retained = operations.retained.iter().any(|c| relevant.contains_key(c));

    if local && escapes || retained && matches!(kind, Origin::Stack(_)) {
        return false;
    }

    // This state includes every possible live alias and every pre-birth watermark. If even
    // it cannot be killed, no concrete state in the existing solver can be killed either.
    let width = relevant.len();
    let mut possible = vec![LIVE; width * 2 + 1];

    possible[width..width * 2].fill(DEAD);
    possible[width * 2] = UNKNOWN;
    let specific: &[usize] = match kind {
        Origin::Heap | Origin::Incoming => &[],
        Origin::Temporary => &operations.temporary,
        Origin::Stack(_) => &operations.scope,
        Origin::Statement(_) => &operations.statement,
        Origin::Until(_) => &operations.until,
    };

    !operations.general.iter().chain(specific).any(|&i| {
        let CfgOp::Resource(op) = &model.blocks[i].op else {
            unreachable!()
        };

        // A terminal release cannot be observed by this closure. Return/store and
        // asynchronous observations are guarded separately because they outlive uses.
        if !escapes && !retained && !model.blocks[i].next.iter().any(|&next| observable[next]) {
            return false;
        }

        if matches!(op, Op::ScopeEnd(..) | Op::StatementEnd(..)) && !observable[i] && !retained {
            return false;
        }

        kills(cx, op, kind, region, &possible, &index)
    })
}

/// An allocation result's immediate self-load is already covered by its fresh seed.
/// Giving that load a second, incoming region would merge successive loop generations.
pub(super) fn covered_loads(model: &Model) -> Vec<bool> {
    let mut incoming = vec![0usize; model.blocks.len()];

    for block in model.blocks.iter() {
        for &to in &block.next {
            incoming[to] += 1;
        }
    }

    let mut covered = vec![false; model.blocks.len()];

    for (i, pair) in model.blocks.windows(2).enumerate() {
        if let CfgOp::Resource(Op::Seed {
            cell,
            origin: Origin::Heap | Origin::Temporary,
            possible: false,
            ..
        }) = pair[0].op
            && pair[0].next.as_slice() == [i + 1]
            && incoming[i + 1] == 1
            && matches!(pair[1].op, CfgOp::PointerLoad(target, source, ..) if target == cell && source == cell)
        {
            covered[i + 1] = true;
        }
    }

    covered
}

/// The spans of released calls, queried for a span one of them contains.
#[derive(Default)]
pub(super) struct FreeSpans {
    /// Sorted by file and start, with the greatest end seen so far in the same file.
    spans: Vec<(u32, u32, u32)>,
}

impl FreeSpans {
    fn new(mut spans: Vec<Span>) -> Self {
        spans.sort_unstable_by_key(|s| (s.file.0, s.start));
        let mut widest = 0;
        let mut file = None;

        Self {
            spans: spans
                .into_iter()
                .map(|s| {
                    if file != Some(s.file.0) {
                        file = Some(s.file.0);
                        widest = 0;
                    }

                    widest = widest.max(s.end);
                    (s.file.0, s.start, widest)
                })
                .collect(),
        }
    }

    /// Whether some released call's span contains `inner`.
    pub(super) fn contain(&self, inner: Span) -> bool {
        let before = self
            .spans
            .partition_point(|&(file, start, _)| (file, start) <= (inner.file.0, inner.start));

        before > 0 && {
            let (file, _, widest) = self.spans[before - 1];
            file == inner.file.0 && widest >= inner.end
        }
    }
}
