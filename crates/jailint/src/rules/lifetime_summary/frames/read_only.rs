//! Conditional read-only effects retain their exact checked comparison/load epochs.
use super::super::{Event, HashSet, TypeId, Val, flow};
use super::{Builder, Frame, HashMap, ProcId, Resolver, Slot, Summary};
use crate::rules::lifetime_movement::facts::Site;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::rules) struct Read {
    pub slot: Slot,
    pub site: Site,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::rules) struct Guard {
    pub branch: Site,
    /// Index in the current retained comparison vector (not a fabricated event).
    pub comparison: usize,
    pub left: Read,
    pub right: Read,
    pub operation: jaic::ir::CmpOp,
    pub when: bool,
}

impl Guard {
    /// Evaluate only exact checked signed64 prestate values for both operand slots.
    pub fn evaluate(&self, values: impl Fn(Slot) -> Option<i64>) -> Option<bool> {
        let (left, right) = (values(self.left.slot)?, values(self.right.slot)?);
        use jaic::ir::CmpOp;
        let value = match self.operation {
            CmpOp::Eq => left == right,
            CmpOp::Ne => left != right,
            CmpOp::SLt => left < right,
            CmpOp::SLe => left <= right,
            CmpOp::SGt => left > right,
            CmpOp::SGe => left >= right,
            _ => return None,
        };
        Some(value == self.when)
    }
}

pub(in crate::rules::lifetime_summary) fn infer(
    r: &Resolver<'_>,
    all: &HashMap<ProcId, Summary>,
    prior: &Builder,
) -> Option<Frame> {
    let mut selector = Builder::read_only_after(r, prior)?;
    if !selector.active() || r.body.comparisons.len() > super::MAX_EVENTS {
        return None;
    }
    selector.charge(r.body.comparisons.len() + r.body.blocks[0].events.len())?;
    selector.charge(selector.events)?;
    if r.body
        .blocks
        .iter()
        .flat_map(|b| &b.successors)
        .any(|&s| s >= r.body.blocks.len())
    {
        return None;
    }
    // The first slice admits the entry block's sole branch. Prefixes containing
    // another branch remain unknown rather than guessing which guard ran first.
    let mut branches = r.body.blocks[0]
        .events
        .iter()
        .enumerate()
        .filter_map(|(i, e)| match e {
            Event::Branch {
                condition,
                then_block,
                else_block,
                ..
            } => Some((i, *condition, *then_block, *else_block)),
            _ => None,
        });
    let (ordinal, condition, yes, no) = branches.next()?;
    if branches.next().is_some()
        || yes == no
        || r.body.blocks[0].successors.len() != 2
        || !r.body.blocks[0].successors.contains(&yes)
        || !r.body.blocks[0].successors.contains(&no)
    {
        return None;
    }
    let mut comparisons = r
        .body
        .comparisons
        .iter()
        .enumerate()
        .filter(|(_, c)| c.value == condition);
    let (comparison, selected) = comparisons.next()?;
    use jaic::ir::{CmpOp, Ty};
    if comparisons.next().is_some()
        || selected.ty != Ty::I64
        || !matches!(
            selected.op,
            CmpOp::Eq | CmpOp::Ne | CmpOp::SLt | CmpOp::SLe | CmpOp::SGt | CmpOp::SGe
        )
    {
        return None;
    }
    let branch = Site {
        block: 0,
        event: ordinal,
    };
    let guard = Guard {
        branch,
        comparison,
        left: entry_read(r, &mut selector, selected.left, branch)?,
        right: entry_read(r, &mut selector, selected.right, branch)?,
        operation: selected.op,
        when: true,
    };
    let mut prior = selector;
    let live = flow::Live::all(r.body);
    for when in [true, false] {
        let mut builder = Builder::read_only_after(r, &prior)?;
        if !builder.active() {
            return None;
        }
        builder.charge(builder.events.checked_mul(4)?.checked_add(16)?)?;
        let skipped = HashSet::from_iter([(
            0,
            if when {
                no
            } else {
                yes
            },
        )]);
        let mut repeated = false;
        let states = flow::states_except(
            r.body,
            &live,
            (),
            &skipped,
            |(), ()| {
                repeated = true;
                false
            },
            |(), site, event| builder.observe(r, site.block, site.event, event, all),
        )?;
        let returned = states
            .iter()
            .enumerate()
            .any(|(i, s)| s.is_some() && r.body.blocks[i].normal_return);
        if repeated || !returned {
            builder.owner = None;
        }
        let result = builder.finish(r, &live, all);
        prior = builder;
        if let Some(mut frame) = result {
            frame.condition = Some(Guard {
                when,
                ..guard
            });
            return Some(frame);
        }
        if jaic::interrupt::should_stop() {
            return None;
        }
    }
    None
}

fn entry_read(r: &Resolver<'_>, builder: &mut Builder, value: Val, branch: Site) -> Option<Read> {
    builder.charge(builder.events)?;
    let mut found = None;
    for (block, b) in r.body.blocks.iter().enumerate() {
        for (event, e) in b.events.iter().enumerate() {
            if let Event::Load {
                place,
                value: loaded,
                ty: TypeId::S64,
                ..
            } = e
                && *loaded == value
            {
                let site = Site {
                    block,
                    event,
                };
                if found.is_some() || block != branch.block || event >= branch.event {
                    return None;
                }
                let slot = builder.slot(r, *place, TypeId::S64, 8)??;
                found = Some(Read {
                    slot,
                    site,
                });
            }
        }
    }
    found
}
