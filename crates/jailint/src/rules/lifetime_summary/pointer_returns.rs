//! Scalar pointer returns retain physical slot reads and caller evaluation epochs.
use super::value_writes::{Installed, Query};
use super::{BodyState, Event, HashMap, HashSet, Path, ProcId, Resolver, Summary, Val};
use crate::rules::lifetime_movement::facts::Site;

/// A structural read template, not an entry-allocation or runtime token equality.
/// Callers retain its checked argument evaluation context through structural sites.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(in crate::rules) struct CapturedRead {
    pub procedure: ProcId,
    pub read: Site,
    pub slot: ReadFootprint,
    pub callers: Vec<(ProcId, Site)>,
    /// Every reachable write after this read preserves the selector bytes.
    /// This is separate from captured pointer provenance and allocation liveness.
    pub selector_preserved: bool,
}

/// Selected bytes read through an immutable input address. This is metadata,
/// not a bounded allocation slot or a proof of readable storage.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(in crate::rules) struct ReadFootprint {
    pub formal: usize,
    pub offset: u64,
    pub width: u64,
}

impl ReadFootprint {
    fn disjoint(self, written: super::physical_slots::Slot) -> bool {
        self.formal == written.formal
            && (self
                .offset
                .checked_add(self.width)
                .is_some_and(|end| end <= written.offset)
                || written
                    .offset
                    .checked_add(written.width)
                    .is_some_and(|end| end <= self.offset))
    }
}

pub(super) type Returns = HashMap<usize, HashSet<Installed>>;

impl Resolver<'_> {
    /// Preserve logical borrow evidence without reinterpreting an uncertain read
    /// as its procedure-entry allocation or the selector's post-call value.
    pub(super) fn captured_read(&self, value: Val, place: usize) -> Option<Path> {
        let width = self
            .compiler
            .ty
            .types
            .size_of(*self.places.types.get(&place)?);
        let mut footprint = None;
        let mut site = None;
        for (block, b) in self.body.blocks.iter().enumerate() {
            for (event, e) in b.events.iter().enumerate() {
                if matches!(e,Event::Load {value:result,place:source,ty,..} if *result==value && self.places.existing(*source,*ty)==Some(place))
                {
                    let read = Site {
                        block,
                        event,
                    };
                    if site.is_some_and(|old| old != read) {
                        return None;
                    }
                    let Event::Load {
                        place: address, ..
                    } = e
                    else {
                        return None;
                    };
                    let object = self.address_object(*address)?;
                    let super::address_objects::Base::Input(formal) = object.base else {
                        return None;
                    };
                    object.offset.checked_add(width)?;
                    let current = ReadFootprint {
                        formal,
                        offset: object.offset,
                        width,
                    };
                    if footprint.is_some_and(|old| old != current) {
                        return None;
                    }
                    footprint = Some(current);
                    site = Some(read);
                }
            }
        }
        Some(Path {
            root: super::Root::CapturedRead(CapturedRead {
                procedure: self.body.procedure,
                read: site?,
                selector_preserved: self.selector_preserved(site?, footprint?),
                slot: footprint?,
                callers: Vec::new(),
            }),
            steps: Vec::new(),
        })
    }

    /// A deliberately bounded frame proof for pure checked getters. Calls and
    /// uncertain writes invalidate the proof rather than changing the read epoch.
    fn selector_preserved(&self, read: Site, slot: ReadFootprint) -> bool {
        let mut queue = vec![(read.block, read.event + 1)];
        let mut seen = HashSet::default();
        let mut budget = 8192usize;
        while let Some((block, start)) = queue.pop() {
            if jaic::interrupt::should_stop() {
                return false;
            }
            if !seen.insert((block, start)) {
                continue;
            }
            for event in &self.body.blocks[block].events[start..] {
                let Some(remaining) = budget.checked_sub(1) else {
                    return false;
                };
                budget = remaining;
                let (place, ty) = match event {
                    Event::Store {
                        place,
                        ty,
                        ..
                    }
                    | Event::Initialize {
                        place,
                        ty,
                        ..
                    } => (*place, *ty),
                    Event::Call {
                        ..
                    }
                    | Event::IndirectCall {
                        ..
                    }
                    | Event::Barrier
                    | Event::Grow {
                        ..
                    } => return false,
                    Event::Return {
                        ..
                    } => break,
                    _ => continue,
                };
                let Some(place) = self.places.existing(place, ty) else {
                    return false;
                };
                if self.immutable_private_slot(place) {
                    continue;
                }
                let Some(written) = self.physical_slot(place, self.compiler.ty.types.size_of(ty))
                else {
                    return false;
                };
                if !slot.disjoint(written) {
                    return false;
                }
            }
            queue.extend(self.body.blocks[block].successors.iter().map(|b| (*b, 0)));
        }
        true
    }

    /// Materialize a captured selector only when its checked frame proof permits
    /// reading the caller's current slot. Cleanup substitution never uses this.
    pub(in crate::rules) fn captured_borrow_location(
        &mut self,
        event: &Event,
        index: usize,
        all: &HashMap<ProcId, Summary>,
    ) -> Option<usize> {
        let (Event::Call {
            args, ..
        }
        | Event::IndirectCall {
            args, ..
        }) = event
        else {
            return None;
        };
        let targets = self.call_targets(event);
        if targets.unknown || targets.procedures.is_empty() {
            return None;
        }
        let mut chosen = None;
        for id in targets.ordered() {
            let (path, None) = all.get(&id)?.returns.get(&index)? else {
                return None;
            };
            let super::Root::CapturedRead(read) = &path.root else {
                return None;
            };
            if !read.selector_preserved
                || !read.callers.is_empty()
                || !path.steps.is_empty()
                || read.procedure != id
            {
                return None;
            }
            let arg = args
                .iter()
                .find(|arg| arg.formal == read.slot.formal && !arg.spread)?;
            let object = self.address_object(super::value(arg)?)?;
            let offset = object.offset.checked_add(read.slot.offset)?;
            let (root, ty) = match object.base {
                super::address_objects::Base::Input(formal) => {
                    let (_, ty) = self.params.values().find(|(n, _)| *n == formal)?;
                    (
                        super::Root::Parameter(formal),
                        self.compiler.ty.types.pointee(*ty)?,
                    )
                }
                super::address_objects::Base::Local(value) => {
                    let mut roots = self
                        .places
                        .paths
                        .iter()
                        .enumerate()
                        .filter(|(_, ((v, _), steps))| *v == value && steps.is_empty());
                    let (place, ((_, ty), _)) = roots.next()?;
                    if roots.next().is_some() || !self.owned.contains(&place) {
                        return None;
                    }
                    (super::Root::Local(place), *ty)
                }
                super::address_objects::Base::Entry(_) => return None,
            };
            if offset.checked_add(read.slot.width)? > self.compiler.ty.types.size_of(ty) {
                return None;
            }
            let mut paths = Vec::new();
            self.entry_fields(
                ty,
                (offset, read.slot.width),
                Vec::new(),
                &mut paths,
                super::physical_slots::Leaf::Borrow,
                0,
            );
            let [steps] = paths.as_slice() else {
                return None;
            };
            let current = Path {
                root,
                steps: steps.clone(),
            }
            .append(&[super::Step::Deref])?;
            if chosen.as_ref().is_some_and(|old| *old != current) {
                return None;
            }
            chosen = Some(current);
        }
        self.materialize(&chosen?)
    }

    /// Forward only an explicitly captured read template. It remains unusable
    /// as an exact cleanup origin until a consumer proves its frame conditions.
    pub(super) fn captured_result_origin(
        &self,
        value: Val,
        all: &HashMap<ProcId, Summary>,
    ) -> Option<Path> {
        let sites = self.pointer_result_sites(value)?;
        let mut chosen = None;
        for (site, result) in sites {
            let event = &self.body.blocks[site.block].events[site.event];
            let targets = self.call_targets(event);
            if targets.unknown || targets.procedures.is_empty() {
                return None;
            }
            for id in targets.ordered() {
                let (path, None) = all.get(&id)?.returns.get(&result)? else {
                    return None;
                };
                let super::Root::CapturedRead(mut read) = path.root.clone() else {
                    return None;
                };
                let caller = (self.body.procedure, site);
                if read.callers.len() >= super::MAX_CHAIN || read.callers.contains(&caller) {
                    return None;
                }
                read.callers.push(caller);
                read.selector_preserved = false;
                let path = Path {
                    root: super::Root::CapturedRead(read),
                    steps: path.steps.clone(),
                };
                if chosen.as_ref().is_some_and(|old| *old != path) {
                    return None;
                }
                chosen = Some(path);
            }
        }
        chosen
    }

    /// Return alternatives are captured values, not field expressions re-evaluated
    /// after a callee has overwritten its receiver. Recursive slot growth widens.
    pub(super) fn infer_pointer_returns(
        &self,
        next: &mut Summary,
        all: &HashMap<ProcId, Summary>,
        live: &super::flow::Live,
    ) {
        next.pointer_returns.clear();
        for (block, b) in self.body.blocks.iter().enumerate() {
            if !live.contains(block) {
                continue;
            }
            for event in &b.events {
                let Event::Return {
                    value,
                    index,
                    ty,
                    ..
                } = event
                else {
                    continue;
                };
                if !self.compiler.ty.types.is_pointer(*ty) {
                    continue;
                }
                let sources = self
                    .installed(Query::Value(*value), all)
                    .into_iter()
                    .map(|source| match source {
                        Installed::Entry(_) | Installed::Address(_) if next.recursive => {
                            Installed::Unknown
                        }
                        Installed::Value(ref path)
                            if matches!(
                                path.root,
                                super::Root::Local(_) | super::Root::CapturedRead(_)
                            ) =>
                        {
                            Installed::Unknown
                        }
                        source => source,
                    });
                next.pointer_returns
                    .entry(*index)
                    .or_default()
                    .extend(sources);
            }
        }
    }

    /// Structural checked call results are worklist nodes. Bodyless boundaries
    /// keep their explicit primitive contracts; missing checked relations do not.
    pub(super) fn pointer_result_sites(&self, value: Val) -> Option<Vec<(Site, usize)>> {
        let mut sites = Vec::new();
        for (block, b) in self.body.blocks.iter().enumerate() {
            for (event, e) in b.events.iter().enumerate() {
                let results = match e {
                    Event::Call {
                        callee,
                        results,
                        expands: false,
                        args,
                        ..
                    } if self.compiler.proc(*callee).body_state == BodyState::Done
                        && !(self.allocators.temporary.contains(callee)
                            && args.iter().any(|a| a.formal == 0 && a.constant == Some(0))) =>
                    {
                        results
                    }
                    Event::IndirectCall {
                        results,
                        allocator: false,
                        ..
                    } if self
                        .call_targets(e)
                        .procedures
                        .iter()
                        .any(|id| self.compiler.proc(*id).body_state == BodyState::Done) =>
                    {
                        results
                    }
                    _ => continue,
                };
                for &(index, result, ty) in results {
                    if result == value && self.compiler.ty.types.is_pointer(ty) {
                        sites.push((
                            Site {
                                block,
                                event,
                            },
                            index,
                        ));
                    }
                }
            }
        }
        (!sites.is_empty()).then_some(sites)
    }

    pub(super) fn installed_result(
        &self,
        site: Site,
        result: usize,
        all: &HashMap<ProcId, Summary>,
        queue: &mut Vec<Query>,
        out: &mut HashSet<Installed>,
    ) {
        let event = &self.body.blocks[site.block].events[site.event];
        let (Event::Call {
            args, ..
        }
        | Event::IndirectCall {
            args, ..
        }) = event
        else {
            out.insert(Installed::Unknown);
            return;
        };
        let targets = self.call_targets(event);
        if targets.unknown || targets.procedures.is_empty() {
            out.insert(Installed::Unknown);
        }
        for id in targets.ordered() {
            if let Some(source) = self.selected_allocator_result(event, id, result, args, all) {
                out.insert(source);
                continue;
            }
            let Some(sources) = all.get(&id).and_then(|s| s.pointer_returns.get(&result)) else {
                out.insert(Installed::Unknown);
                continue;
            };
            for source in sources {
                if let Installed::Value(path) = source
                    && !matches!(path.root, super::Root::Parameter(_))
                {
                    let result_value = match event {
                        Event::Call {
                            results, ..
                        }
                        | Event::IndirectCall {
                            results, ..
                        } => results
                            .iter()
                            .find(|(index, _, _)| *index == result)
                            .map(|(_, v, _)| *v),
                        _ => None,
                    };
                    let root = result_value.and_then(|v| self.allocation_root(&path.root, v, args));
                    out.insert(root.map_or(Installed::Unknown, |root| {
                        Installed::Value(Path {
                            root,
                            steps: path.steps.clone(),
                        })
                    }));
                } else {
                    self.queue_installed(source, args, site, queue, out);
                }
            }
        }
    }

    /// A positively selected operation can use its checked allocation-return
    /// guard. An allocation elsewhere in the body is not a returned allocation.
    fn selected_allocator_result(
        &self,
        event: &Event,
        callee: ProcId,
        result: usize,
        args: &[super::LifetimeArgument],
        all: &HashMap<ProcId, Summary>,
    ) -> Option<Installed> {
        let (path, Some((formal, expected))) = all.get(&callee)?.allocations.get(&result)? else {
            return None;
        };
        if !path.steps.is_empty()
            || !matches!(
                path.root,
                super::Root::Heap
                    | super::Root::CallerAllocator
                    | super::Root::AllocatorParameter(_)
                    | super::Root::Temporary
            )
            || !args
                .iter()
                .any(|arg| arg.formal == *formal && !arg.spread && arg.constant == Some(*expected))
        {
            return None;
        }
        let value = match event {
            Event::Call {
                results, ..
            }
            | Event::IndirectCall {
                results, ..
            } => results
                .iter()
                .find(|(index, _, _)| *index == result)
                .map(|(_, v, _)| *v),
            _ => None,
        }?;
        Some(
            self.allocation_root(&path.root, value, args)
                .map_or(Installed::Unknown, |root| {
                    Installed::Value(Path {
                        root,
                        steps: Vec::new(),
                    })
                }),
        )
    }

    /// None means this is not a checked scalar pointer call; an unknown checked
    /// result is represented explicitly inside the returned alternatives.
    pub(super) fn pointer_result_sources(
        &self,
        value: Val,
        all: &HashMap<ProcId, Summary>,
    ) -> Option<HashSet<Installed>> {
        self.pointer_result_sites(value)?;
        Some(self.installed(Query::Value(value), all))
    }

    pub(super) fn pointer_result_origin(
        &self,
        value: Val,
        all: &HashMap<ProcId, Summary>,
    ) -> Option<Path> {
        let sources = self.pointer_result_sources(value, all)?;
        let mut chosen = None;
        for source in sources {
            let path = self.installed_path(source)?;
            if chosen.as_ref().is_some_and(|old| *old != path) {
                return None;
            }
            chosen = Some(path);
        }
        chosen
    }

    pub(super) fn installed_path(&self, source: Installed) -> Option<Path> {
        match source {
            Installed::Value(path) => Some(path),
            Installed::Entry(slot) => self.entry_path(slot),
            Installed::Address(slot) => self.address_path(slot),
            Installed::Unknown => None,
        }
    }
}
