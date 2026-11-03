//! A null argument has no live allocation. Excluding only its null edge permits
//! nullable cleanup wrappers without turning unrelated conditions into releases.
use super::{Effect, Event, HashMap, HashSet, Kind, ProcId, Resolver, Span, Summary};
use jaic::ir::CmpOp;

impl Resolver<'_> {
    pub(super) fn nullable_guarantee(
        &self,
        effects: &[Vec<(Effect, Vec<Span>)>],
        target: &Effect,
        all: &HashMap<ProcId, Summary>,
    ) -> Option<Vec<Span>> {
        if target.kind != Kind::Free {
            return None;
        }
        let comparisons = self
            .comparisons
            .get_or_init(|| self.body.comparisons.iter().map(|c| (c.value, c)).collect());
        let mut skipped = HashSet::default();
        for (i, block) in self.body.blocks.iter().enumerate() {
            for event in &block.events {
                let Event::Branch {
                    condition,
                    then_block,
                    else_block,
                    ..
                } = event
                else {
                    continue;
                };
                if then_block == else_block {
                    continue;
                }
                let Some(cmp) = comparisons.get(condition) else {
                    continue;
                };
                if cmp.ty != jaic::ir::Ty::Ptr {
                    continue;
                }
                let value = if cmp.right_constant == Some(0) {
                    cmp.left
                } else if cmp.left_constant == Some(0) {
                    cmp.right
                } else {
                    continue;
                };
                let zero = match cmp.op {
                    CmpOp::Eq => *then_block,
                    CmpOp::Ne => *else_block,
                    _ => continue,
                };
                if self.value(value, all).as_ref() == Some(&target.target) {
                    skipped.insert((i, zero));
                }
            }
        }
        if skipped.is_empty() {
            return None;
        }
        super::flow::guaranteed_except(self.body, effects, target, &skipped)
    }
}
