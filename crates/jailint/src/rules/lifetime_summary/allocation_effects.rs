//! Allocating storage is separate from returning it: void helpers and heap copies
//! can allocate temporary scratch without handing that scratch to their callers.
use super::allocator_context::Selection;
use super::{Event, HashMap, ProcId, Resolver, Root, Summary};

impl Resolver<'_> {
    // Direct calls of a known allocator provider obey the same ALLOCATE protocol
    // as calls through Allocator.proc. CAPS/FREE results are not allocated bytes.
    pub(super) fn infer_provider_allocation(&self, next: &mut Summary) {
        if self.allocators.temporary.contains(&self.body.procedure) {
            let guard = Some((0, 0));
            next.allocations.insert(
                0,
                (
                    super::Path {
                        root: Root::Temporary,
                        steps: Vec::new(),
                    },
                    guard,
                ),
            );
            next.allocation_effects.insert((Root::Temporary, guard));
        }
    }

    pub(super) fn infer_allocations(
        &self,
        event: &Event,
        site: (usize, usize),
        all: &HashMap<ProcId, Summary>,
        next: &mut Summary,
    ) {
        match event {
            Event::IndirectCall {
                callee,
                allocator: true,
                args,
                ..
            } => {
                let Some(root) = self.context.indirect.get(callee).and_then(|s| s.root()) else {
                    return;
                };
                let Some(mode) = args.first() else {
                    return;
                };
                for selected in [0, 1] {
                    if let Some(guard) = self.mode_guard(mode, selected, all) {
                        next.allocation_effects.insert((root.clone(), guard));
                    }
                }
            }
            Event::Call {
                callee,
                args,
                expands: false,
                ..
            } => {
                if self.allocators.temporary.contains(callee)
                    && self
                        .compiler
                        .ed
                        .facts()
                        .and_then(|f| f.lifetime_bodies.get(callee))
                        .is_some_and(|body| super::allocators::nonallocating_call(body, args))
                {
                    return;
                }
                let Some(summary) = all.get(callee) else {
                    return;
                };
                for (root, guard) in &summary.allocation_effects {
                    let guard = if let Some((formal, expected)) = guard {
                        let Some(arg) = args.iter().find(|a| a.formal == *formal) else {
                            continue;
                        };
                        let Some(guard) = self.mode_guard(arg, *expected, all) else {
                            continue;
                        };
                        guard
                    } else {
                        None
                    };
                    if let Some(root) = self.allocation_effect_root(root, site) {
                        next.allocation_effects.insert((root, guard));
                    }
                }
            }
            _ => {}
        }
    }

    fn allocation_effect_root(&self, root: &Root, site: (usize, usize)) -> Option<Root> {
        match root {
            Root::CallerAllocator => self
                .context
                .sites
                .get(&site)
                .copied()
                .unwrap_or(Selection::Caller)
                .root(),
            Root::AllocatorParameter(n) => self
                .context
                .site_arguments
                .get(&(site, *n))
                .copied()
                .unwrap_or_default()
                .root(),
            Root::Heap | Root::Temporary => Some(root.clone()),
            _ => None,
        }
    }
}
