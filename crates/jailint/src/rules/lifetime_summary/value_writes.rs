//! May relations for values installed in pointer slots, with typed byte identity.
//!
//! A read of an incoming slot is instantiated at the caller's actual call epoch.
//! A finite query graph expands captured SSA values, pre-call reads and call writes;
//! it never recursively appends symbolic field names or infers purity from absence.
use super::physical_slots::Slot;
use super::{Def, Event, HashMap, HashSet, Path, ProcId, Resolver, Root, Summary, Val};
use crate::rules::lifetime_movement::facts::Site;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(in crate::rules) struct Target {
    path: Option<Path>,
    slot: Option<Slot>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(in crate::rules) enum Installed {
    Value(Path),
    Entry(Slot),
    Address(Slot),
    Unknown,
}

pub(in crate::rules) type Relations = HashMap<Target, HashSet<Installed>>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum Query {
    Value(Val),
    Aggregate(Val, jaic::types::TypeId, usize, usize, Site),
    Before(Site, Slot),
    Call(Site, Slot),
    Result(Site, usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum Source {
    Entry,
    Value(Val),
    Call(Site),
    Unknown,
}

type Sources = HashSet<Source>;

const MAX_SLOT_QUERIES: usize = 4096;

impl Resolver<'_> {
    pub(super) fn infer_value_writes(
        &self,
        all: &HashMap<ProcId, Summary>,
        live: &super::flow::Live,
        out: &mut Relations,
    ) {
        let _storage = self.storage_query();
        for (block, events) in self
            .body
            .blocks
            .iter()
            .enumerate()
            .filter(|(i, _)| live.contains(*i))
        {
            for (ordinal, event) in events.events.iter().enumerate() {
                let site = Site {
                    block,
                    event: ordinal,
                };
                match event {
                    Event::Store {
                        place,
                        value,
                        ty,
                        ..
                    } if self.compiler.ty.types.is_pointer(*ty) => {
                        let Some(place) = self.places.existing(*place, *ty) else {
                            continue;
                        };
                        if !self.places.unresolved(place) {
                            continue;
                        }
                        let path = self.location(place, all, &mut HashSet::default(), 0);
                        if path
                            .as_ref()
                            .is_some_and(|p| !matches!(p.root, Root::Parameter(_)))
                        {
                            continue;
                        }
                        let target = Target {
                            path,
                            slot: self.physical_slot(place, self.compiler.ty.types.size_of(*ty)),
                        };
                        out.entry(target)
                            .or_default()
                            .extend(self.installed(Query::Value(*value), all));
                    }
                    Event::Call {
                        expands: false,
                        args,
                        ..
                    }
                    | Event::IndirectCall {
                        args, ..
                    } => {
                        for callee in self.call_targets(event).procedures {
                            let Some(summary) = all.get(&callee) else {
                                continue;
                            };
                            for (target, sources) in &summary.value_writes {
                                self.forward_written(
                                    (target, sources),
                                    args,
                                    site,
                                    all,
                                    out,
                                    summary.recursive,
                                );
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }


    /// Substitute one callee write without converting input pointers back to field paths.
    fn forward_written(
        &self,
        write: (&Target, &HashSet<Installed>),
        args: &[super::LifetimeArgument],
        site: Site,
        all: &HashMap<ProcId, Summary>,
        out: &mut Relations,
        recursive: bool,
    ) {
        let (target, sources) = write;
        if recursive {
            // Recursive offset composition has no practical bound from a declared
            // pointee extent. Retain may values but widen physical reads/addresses.
            let values = self
                .instantiate(sources, args, site, all)
                .into_iter()
                .map(|source| match source {
                    Installed::Entry(_) | Installed::Address(_) => Installed::Unknown,
                    source => source,
                });
            out.entry(Target {
                path: None,
                slot: None,
            })
            .or_default()
            .extend(values);
            return;
        }
        let path = target
            .path
            .as_ref()
            .and_then(|path| self.substitute(path, args, all, &mut HashSet::default(), 0));
        if path
            .as_ref()
            .is_some_and(|p| !matches!(p.root, Root::Parameter(_)))
        {
            return;
        }
        let target = Target {
            path,
            slot: target.slot.and_then(|s| self.substitute_slot(s, args)),
        };
        let values = self.instantiate(sources, args, site, all);
        out.entry(target).or_default().extend(values);
    }

    fn instantiate(
        &self,
        sources: &HashSet<Installed>,
        args: &[super::LifetimeArgument],
        site: Site,
        all: &HashMap<ProcId, Summary>,
    ) -> HashSet<Installed> {
        let mut out = HashSet::default();
        for source in sources {
            match source {
                Installed::Value(path) => {
                    if let Root::Parameter(formal) = path.root {
                        if path.steps.is_empty()
                            && let Some(actual) = args
                                .iter()
                                .find(|a| a.formal == formal && !a.spread)
                                .and_then(super::value)
                        {
                            out.extend(self.installed(Query::Value(actual), all));
                        } else {
                            out.insert(Installed::Unknown);
                        }
                    } else if path.steps.is_empty() {
                        out.insert(Installed::Value(path.clone()));
                    } else {
                        out.insert(Installed::Unknown);
                    }
                }
                Installed::Address(slot) => {
                    out.insert(
                        self.substitute_slot(*slot, args)
                            .map_or(Installed::Unknown, Installed::Address),
                    );
                }
                Installed::Entry(slot) => {
                    if let Some(slot) = self.substitute_slot(*slot, args) {
                        out.extend(self.installed(Query::Before(site, slot), all));
                    } else {
                        out.insert(Installed::Unknown);
                    }
                }
                Installed::Unknown => {
                    out.insert(Installed::Unknown);
                }
            }
        }
        out
    }

    /// Each query is expanded once. Unknown/cancellation/exhaustion is explicit and
    /// cannot be consumed as an absent write or a guaranteed origin.
    pub(super) fn installed(
        &self,
        initial: Query,
        all: &HashMap<ProcId, Summary>,
    ) -> HashSet<Installed> {
        let mut queue = vec![initial];
        let mut seen = HashSet::default();
        let mut out = HashSet::default();
        let mut remaining = 8192usize;
        let mut index = None;
        while let Some(query) = queue.pop() {
            if !seen.insert(query) {
                continue;
            }
            let Some(next) = remaining.checked_sub(1) else {
                out.insert(Installed::Unknown);
                break;
            };
            remaining = next;
            // Object extents make offsets finite; this budget also bounds huge
            // fixed arrays and mutually recursive shifted forwarding.
            if seen.len() > MAX_SLOT_QUERIES || jaic::interrupt::should_stop() {
                out.insert(Installed::Unknown);
                break;
            }
            match query {
                Query::Value(value) => self.installed_value(value, all, &mut queue, &mut out),
                Query::Aggregate(value, ty, base, place, site) => {
                    if let Some(read) = self.aggregate_read(value, ty, base, place)
                        && let Some(slot) = read.slot
                    {
                        queue.push(Query::Before(site, slot));
                    } else {
                        out.insert(Installed::Unknown);
                    }
                }
                Query::Before(site, slot) => {
                    for source in self.before_counted(site, slot, all, &mut remaining, &mut index) {
                        match source {
                            Source::Entry => {
                                out.insert(Installed::Entry(slot));
                            }
                            Source::Value(value) => queue.push(Query::Value(value)),
                            Source::Call(site) => queue.push(Query::Call(site, slot)),
                            Source::Unknown => {
                                out.insert(Installed::Unknown);
                            }
                        }
                    }
                }
                Query::Call(site, slot) => {
                    self.installed_call(site, slot, all, &mut queue, &mut out);
                }
                Query::Result(site, result) => {
                    self.installed_result(site, result, all, &mut queue, &mut out);
                }
            }
        }
        out
    }


    /// Expand one captured SSA definition into the finite source-query worklist.
    fn installed_value(
        &self,
        value: Val,
        all: &HashMap<ProcId, Summary>,
        queue: &mut Vec<Query>,
        out: &mut HashSet<Installed>,
    ) {
        if let Some(sites) = self.pointer_result_sites(value) {
            queue.extend(
                sites
                    .into_iter()
                    .map(|(site, index)| Query::Result(site, index)),
            );
            return;
        }
        let Some(definitions) = self.values.get(&value) else {
            // Checked address-rounding facts can exist without a value event.
            self.installed_direct(value, all, out);
            return;
        };
        for definition in definitions {
            match definition {
                Def::Parameter(formal, ..) => {
                    if !self.params.values().any(|(index, ty)| {
                        index == formal && self.compiler.ty.types.is_pointer(*ty)
                    }) {
                        out.insert(Installed::Unknown);
                        continue;
                    }
                    out.insert(Installed::Value(Path {
                        root: Root::Parameter(*formal),
                        steps: Vec::new(),
                    }));
                }
                Def::Address(place) => {
                    let slot = self.places.types.get(place).and_then(|ty| {
                        self.physical_slot(*place, self.compiler.ty.types.size_of(*ty))
                    });
                    if let Some(slot) = slot {
                        out.insert(Installed::Address(slot));
                    } else {
                        self.installed_direct(value, all, out);
                    }
                }
                Def::Copy(source) => queue.push(Query::Value(*source)),
                Def::Load(place, ty) => self.installed_load(value, *place, *ty, queue, out),
                _ => self.installed_direct(value, all, out),
            }
        }
    }

    /// Only layout-independent legacy origins can enter the physical relation.
    fn installed_direct(
        &self,
        value: Val,
        all: &HashMap<ProcId, Summary>,
        out: &mut HashSet<Installed>,
    ) {
        let source = self
            .trace(value, all, &mut HashSet::default(), 0)
            .filter(|p| p.steps.is_empty() && !matches!(p.root, Root::CapturedRead(_)))
            .map_or(Installed::Unknown, Installed::Value);
        out.insert(source);
    }

    /// Pointer loads use their captured storage epoch, including whole-object copies.
    fn installed_load(
        &self,
        value: Val,
        place: usize,
        ty: jaic::types::TypeId,
        queue: &mut Vec<Query>,
        out: &mut HashSet<Installed>,
    ) {
        if !self.places.unresolved(place) {
            if let Some(sources) = self.storage_versions.get(&value) {
                for source in sources {
                    match source {
                        super::storage_versions::Source::Value(value) => {
                            queue.push(Query::Value(*value));
                        }
                        super::storage_versions::Source::Aggregate(
                            value,
                            ty,
                            base,
                            place,
                            site,
                        ) => queue.push(Query::Aggregate(*value, *ty, *base, *place, *site)),
                        _ => {
                            out.insert(Installed::Unknown);
                        }
                    }
                }
            } else if let Some([source]) = self.writes.get(&place).map(Vec::as_slice) {
                queue.push(Query::Value(*source));
            } else {
                out.insert(Installed::Unknown);
            }
            return;
        }
        if let Some(slot) = self.physical_slot(place, self.compiler.ty.types.size_of(ty)) {
            if let Some(sources) = self.storage_versions.get(&value) {
                for source in sources {
                    match source {
                        super::storage_versions::Source::Initial => {
                            out.insert(Installed::Entry(slot));
                        }
                        super::storage_versions::Source::Value(v) => {
                            queue.push(Query::Value(*v));
                        }
                        super::storage_versions::Source::Call(site) => {
                            queue.push(Query::Call(*site, slot));
                        }
                        super::storage_versions::Source::Aggregate(
                            value,
                            ty,
                            base,
                            place,
                            site,
                        ) => queue.push(Query::Aggregate(*value, *ty, *base, *place, *site)),
                        super::storage_versions::Source::ScalarValue {
                            ..
                        } => {
                            out.insert(Installed::Unknown);
                        }
                        super::storage_versions::Source::Unknown
                        | super::storage_versions::Source::Constant {
                            ..
                        } => {
                            out.insert(Installed::Unknown);
                            out.insert(Installed::Entry(slot));
                        }
                    }
                }
            } else {
                out.insert(Installed::Entry(slot));
            }
        } else if !self.places.unresolved(place) {
            if let Some(sources) = self.writes.get(&place) {
                queue.extend(sources.iter().map(|v| Query::Value(*v)));
            }
        } else {
            out.insert(Installed::Unknown);
        }
    }

    /// Resolve writes at a captured call epoch using certified physical slot overlap.
    fn installed_call(
        &self,
        site: Site,
        slot: Slot,
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
        for callee in self.call_targets(event).procedures {
            for (target, sources) in all.get(&callee).into_iter().flat_map(|s| &s.value_writes) {
                let target = target.slot.and_then(|s| self.substitute_slot(s, args));
                if target.is_some_and(|target| target.disjoint(slot)) {
                    continue;
                }
                if target.is_some_and(|target| target.formal == slot.formal && target != slot) {
                    out.insert(Installed::Unknown);
                    continue;
                }
                for source in sources {
                    self.queue_installed(source, args, site, queue, out);
                }
            }
        }
    }

    /// Preserve captured pointer actuals while enqueueing a callee's installed source.
    pub(super) fn queue_installed(
        &self,
        source: &Installed,
        args: &[super::LifetimeArgument],
        site: Site,
        queue: &mut Vec<Query>,
        out: &mut HashSet<Installed>,
    ) {
        match source {
            Installed::Value(path) => {
                if let Root::Parameter(formal) = path.root {
                    if path.steps.is_empty()
                        && let Some(actual) = args
                            .iter()
                            .find(|a| a.formal == formal && !a.spread)
                            .and_then(super::value)
                    {
                        queue.push(Query::Value(actual));
                    } else {
                        out.insert(Installed::Unknown);
                    }
                } else if path.steps.is_empty() {
                    out.insert(Installed::Value(path.clone()));
                } else {
                    out.insert(Installed::Unknown);
                }
            }
            Installed::Address(source) => {
                out.insert(
                    self.substitute_slot(*source, args)
                        .map_or(Installed::Unknown, Installed::Address),
                );
            }
            Installed::Entry(source) => {
                if let Some(source) = self.substitute_slot(*source, args) {
                    queue.push(Query::Before(site, source));
                } else {
                    out.insert(Installed::Unknown);
                }
            }
            Installed::Unknown => {
                out.insert(Installed::Unknown);
            }
        }
    }

    pub(super) fn before(&self, at: Site, slot: Slot, all: &HashMap<ProcId, Summary>) -> Sources {
        self.before_counted(at, slot, all, &mut 8192, &mut None)
    }

    fn before_counted(
        &self,
        at: Site,
        slot: Slot,
        all: &HashMap<ProcId, Summary>,
        remaining: &mut usize,
        index: &mut Option<Option<super::captured_values::QueryIndex>>,
    ) -> Sources {
        if index.is_none() {
            *index = Some(self.capture_index(remaining));
        }
        let Some(Some(index)) = index.as_ref() else {
            return HashSet::from_iter([Source::Entry, Source::Unknown]);
        };
        let Some(sources) = super::storage_versions::before(self, at, slot, all, remaining, index)
        else {
            return HashSet::from_iter([Source::Entry, Source::Unknown]);
        };
        sources
            .into_iter()
            .map(|source| match source {
                super::storage_versions::Source::Initial => Source::Entry,
                super::storage_versions::Source::Value(value) => Source::Value(value),
                super::storage_versions::Source::Call(site) => Source::Call(site),
                _ => Source::Unknown,
            })
            .collect()
    }

    pub(super) fn written_sources_at(
        &self,
        site: Site,
        place: usize,
        all: &HashMap<ProcId, Summary>,
        _seen: &mut HashSet<Val>,
        _depth: usize,
    ) -> HashSet<Path> {
        let Some(ty) = self.places.types.get(&place) else {
            return HashSet::default();
        };
        let Some(slot) = self.physical_slot(place, self.compiler.ty.types.size_of(*ty)) else {
            return HashSet::default();
        };
        let mut out = HashSet::default();
        for source in self.installed(Query::Call(site, slot), all) {
            match source {
                Installed::Value(path) => {
                    out.insert(path);
                }
                Installed::Entry(slot) => {
                    if let Some(path) = self.entry_path(slot) {
                        out.insert(path);
                    }
                }
                Installed::Address(slot) => {
                    if let Some(path) = self.address_path(slot) {
                        out.insert(path);
                    }
                }
                Installed::Unknown => {}
            }
        }
        out
    }
}

/// Requested bytes only. Membership proves no value, provider or effect closure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::rules) struct ContextReads {
    owner: crate::rules::lifetime_facts::BodyIdentity,
    pub ty: jaic::types::TypeId,
    pub fields: Vec<(jaic::types::TypeId, crate::rules::lifetime_facts::ByteRange)>,
}

pub(super) struct ReadRequests {
    selected: Option<(
        jaic::types::TypeId,
        Vec<(jaic::types::TypeId, crate::rules::lifetime_facts::ByteRange)>,
    )>,
    remaining: usize,
    failed: bool,
}
impl ReadRequests {
    pub fn new() -> Self {
        Self {
            selected: None,
            remaining: 8192,
            failed: false,
        }
    }
    fn charge(&mut self, n: usize) -> Option<()> {
        self.remaining = self.remaining.checked_sub(n)?;
        (!jaic::interrupt::should_stop()).then_some(())
    }
    fn add(
        &mut self,
        context: jaic::types::TypeId,
        field: (jaic::types::TypeId, crate::rules::lifetime_facts::ByteRange),
    ) -> Option<()> {
        self.charge(4)?;
        let (ty, fields) = self.selected.get_or_insert_with(|| (context, Vec::new()));
        if *ty != context || field.1.width != 8 {
            return None;
        }
        if fields.contains(&field) {
            return Some(());
        }
        if fields.len() == 3 || fields.iter().any(|old| !old.1.separated(field.1)) {
            if jacket_config::env::LIFETIME_STATS.is_set() {
                eprintln!(
                    "lifetime-context-request-conflict incoming={field:?} existing={fields:?}"
                );
            }
            return None;
        }
        fields.push(field);
        Some(())
    }
    pub fn observe(
        &mut self,
        r: &Resolver<'_>,
        event: &Event,
        site: Site,
        all: &HashMap<ProcId, Summary>,
        local: bool,
    ) {
        if !self.failed && self.event(r, event, site, all, local).is_none() {
            if jacket_config::env::LIFETIME_STATS.is_set() {
                eprintln!(
                    "lifetime-context-request-stop proc={} local={} site={site:?} selected={:?} remaining={} event={event:?}",
                    r.body.procedure.0, local, self.selected, self.remaining
                );
            }
            self.failed = true;
        }
    }
    fn event(
        &mut self,
        r: &Resolver<'_>,
        event: &Event,
        site: Site,
        all: &HashMap<ProcId, Summary>,
        local: bool,
    ) -> Option<()> {
        self.charge(1)?;
        if local
            && let Event::FieldProjection {
                base_ty,
                result,
                result_ty,
                offset,
                ..
            } = event
        {
            if r.body.context_types.len() > 8 {
                return None;
            }
            self.charge(r.body.context_types.len() + 1)?;
            if !r.body.context_types.contains(base_ty) {
                return Some(());
            }
            if result_ty.0 as usize >= r.compiler.ty.types.type_count() {
                return None;
            }
            if Some(*result_ty) == r.allocators.ty {
                self.charge(
                    r.compiler
                        .ty
                        .types
                        .struct_info(r.compiler.ty.types.as_struct(*result_ty)?)
                        .fields
                        .len()
                        .checked_mul(2)?,
                )?;
                for (ty, mut bytes) in r.allocators.selected_fields(r.compiler)? {
                    bytes.offset = bytes.offset.checked_add(*offset)?;
                    self.add(*base_ty, (ty, bytes))?;
                }
            } else if matches!(
                r.compiler.ty.types.kind(*result_ty),
                jaic::types::TypeKind::Proc(_)
            ) && self.returned_callback(r, *result, *result_ty)?
            {
                self.add(
                    *base_ty,
                    (
                        *result_ty,
                        crate::rules::lifetime_facts::ByteRange {
                            offset: *offset,
                            width: r.compiler.ty.types.size_of(*result_ty),
                        },
                    ),
                )?;
            }
        }
        if let Event::Call {
            callee,
            expands: false,
            ..
        } = event
        {
            let target = if let Some(edge) = r.compiler.lifetime_generated_entry(*callee) {
                if edge.context != jaic::ide::lifetimes::LifetimeContextTransfer::ForwardHidden {
                    return Some(());
                }
                edge.target
            } else {
                if !r
                    .compiler
                    .pr
                    .procs
                    .get(callee.0 as usize)?
                    .sig
                    .as_ref()?
                    .has_context
                {
                    return Some(());
                }
                *callee
            };
            if let Some(reads) = all
                .get(&target)
                .and_then(|summary| summary.context_reads.as_deref())
            {
                self.charge(r.body.call_contexts.len() + 1)?;
                let mut rows = r
                    .body
                    .call_contexts
                    .iter()
                    .filter(|row| row.block == site.block && row.event == site.event);
                let argument = rows.next()?.argument;
                if rows.next().is_some() {
                    return None;
                }
                if !matches!(
                    argument,
                    jaic::ide::lifetimes::LifetimeContextArgument::Value(_)
                ) {
                    return Some(());
                }
                self.charge(reads.fields.len() + 1)?;
                if !reads.owner.current(r.compiler) {
                    return None;
                }
                for field in &reads.fields {
                    self.add(reads.ty, *field)?;
                }
            }
        }
        Some(())
    }
    /// Existing definitions discover demand only; they prove no target/value or return behavior.
    fn returned_callback(
        &mut self,
        r: &Resolver<'_>,
        address: Val,
        ty: jaic::types::TypeId,
    ) -> Option<bool> {
        let Some(slot) = r.places.existing(address, ty) else {
            return Some(false);
        };
        self.charge(r.body.blocks.len() + 1)?;
        let events = r
            .body
            .blocks
            .iter()
            .try_fold(0usize, |n, block| n.checked_add(block.events.len()))?;
        self.charge(events)?;
        for event in r.body.blocks.iter().flat_map(|block| &block.events) {
            let Event::Return {
                value,
                ty: returned,
                ..
            } = event
            else {
                continue;
            };
            if r.compiler.ir_ty(*returned).is_none() {
                continue;
            }
            let Some(defs) = r.values.get(value) else {
                continue;
            };
            self.charge(defs.len() + 1)?;
            // The map key is the actual returned SSA; the definition retains
            // that call's exact result index. Ambiguous/forwarded rows reject.
            let [Def::Indirect(callee, _, result_index)] = defs.as_slice() else {
                continue;
            };
            let jaic::types::TypeKind::Proc(signature) = r.compiler.ty.types.kind(ty) else {
                return Some(false);
            };
            if signature.returns.get(*result_index) != Some(returned) {
                continue;
            }
            let Some(loads) = r.values.get(callee) else {
                continue;
            };
            self.charge(loads.len() + 1)?;
            if matches!(loads.as_slice(),[Def::Load(place,loaded)] if *place==slot && *loaded==ty) {
                return Some(true);
            }
        }
        Some(false)
    }

    pub fn finish(self, r: &Resolver<'_>) -> Option<std::rc::Rc<ContextReads>> {
        if self.failed || jaic::interrupt::should_stop() {
            return None;
        }
        let (ty, mut fields) = self.selected?;
        fields.sort_by_key(|(_, range)| range.offset);
        Some(std::rc::Rc::new(ContextReads {
            owner: crate::rules::lifetime_facts::BodyIdentity::capture(r.compiler, r.body)?,
            ty,
            fields,
        }))
    }
}

/// Conditional exact bytes of the one returned object. Native success, lifetime
/// and caller preservation remain obligations; this is never an ordinary Frame.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::rules) struct ReturnedSlots {
    owner: crate::rules::lifetime_facts::BodyIdentity,
    pub ty: jaic::types::TypeId,
    pub fields: Vec<jaic::ide::lifetimes::LifetimeDefaultLeaf>,
    acquisition: Site,
    range: super::native_returns::ranges::Range,
    pub lifetime: Option<super::native_returns::ReturnLifetime>,
}
impl ReturnedSlots {
    pub(super) fn capture(
        r: &Resolver<'_>,
        all: &HashMap<ProcId, Summary>,
        requests: &ContextReads,
    ) -> Option<std::rc::Rc<Self>> {
        if all.get(&r.body.procedure)?.recursive
            || all.get(&r.body.procedure)?.trusted.is_some()
            || !requests.owner.current(r.compiler)
        {
            return None;
        }
        let sig = r.compiler.proc(r.body.procedure).sig.as_ref()?;
        if sig.returns.len() != 1
            || r.compiler.ty.types.pointee(sig.returns[0]) != Some(requests.ty)
        {
            return None;
        }
        let mut remaining = 8192usize;
        let (acquisition, range, fields, lifetime) = super::native_returns::captured_object_fields(
            r,
            all,
            None,
            &requests.fields,
            &mut remaining,
        )?;
        remaining = remaining.checked_sub(fields.len() + 1)?;
        let _ = remaining;
        if jaic::interrupt::should_stop() {
            return None;
        }
        Some(std::rc::Rc::new(Self {
            owner: crate::rules::lifetime_facts::BodyIdentity::capture(r.compiler, r.body)?,
            ty: requests.ty,
            fields,
            acquisition,
            range,
            lifetime,
        }))
    }
}

/// Discover requests before value inference. Only metadata is propagated here;
/// later resolvers are rebuilt against completed child effects.
pub(super) fn collect_requests<'c>(
    compiler: &'c jaic::sema::Compiler,
    cache: &mut super::prepared::Cache<'c>,
    components: &[Vec<ProcId>],
    valid: &HashMap<ProcId, &'c jaic::ide::lifetimes::LifetimeBody>,
    native_demand: &HashSet<ProcId>,
    local_demand: &HashSet<ProcId>,
    all: &mut super::Summaries,
) -> HashMap<ProcId, ContextReads> {
    for component in components {
        loop {
            let mut changed = false;
            for id in component {
                if jaic::interrupt::should_stop() {
                    return HashMap::default();
                }
                let prepared = cache.prepare_requested(
                    *id,
                    &all.procs,
                    &all.targets,
                    native_demand.contains(id)
                        && !all.procs[id].recursive
                        && all.procs[id].trusted.is_none(),
                );
                let r =
                    Resolver::from_prepared(prepared, &all.procs, &all.targets, &all.allocators);
                let mut requests = ReadRequests::new();
                for (block, b) in r.body.blocks.iter().enumerate() {
                    for (event, e) in b.events.iter().enumerate() {
                        requests.observe(
                            &r,
                            e,
                            Site {
                                block,
                                event,
                            },
                            &all.procs,
                            local_demand.contains(id),
                        );
                    }
                }
                let next = requests.finish(&r);
                if next != all.procs[id].context_reads {
                    all.procs.get_mut(id).unwrap().context_reads = next;
                    changed = true;
                }
            }
            // Recursive request sets cannot authorize a returned-object proof.
            if !changed || component.iter().any(|id| all.procs[id].recursive) {
                break;
            }
        }
    }
    let mut demands = HashMap::default();
    let mut failed = HashSet::default();
    for (id, body) in valid {
        let Some(reads) = all.procs[id].context_reads.as_deref() else {
            continue;
        };
        let prepared = cache.prepare_requested(
            *id,
            &all.procs,
            &all.targets,
            native_demand.contains(id)
                && !all.procs[id].recursive
                && all.procs[id].trusted.is_none(),
        );
        let r = Resolver::from_prepared(prepared, &all.procs, &all.targets, &all.allocators);
        let mut remaining = 8192usize;
        let Some(index) = r.capture_index(&mut remaining) else {
            continue;
        };
        let mut selected = HashSet::default();
        for block in &body.blocks {
            let Some(next) = remaining.checked_sub(block.events.len() + 1) else {
                selected.clear();
                break;
            };
            remaining = next;
            for event in &block.events {
                let Event::ContextSet {
                    value, ..
                } = event
                else {
                    continue;
                };
                if let Some((value, _)) = index.pointer_leaf(&r, *value, &mut remaining, 0) {
                    selected.insert(value);
                }
            }
        }
        if jacket_config::env::LIFETIME_STATS.is_set() {
            eprintln!(
                "lifetime-returned-demand caller={id:?} selected={selected:?} fields={:?} remaining={remaining}",
                reads.fields
            );
        }
        for block in &body.blocks {
            let Some(next) = remaining.checked_sub(block.events.len() + 1) else {
                break;
            };
            remaining = next;
            for event in &block.events {
                let Event::Call {
                    callee,
                    results,
                    expands: false,
                    ..
                } = event
                else {
                    continue;
                };
                let Some(next) = remaining.checked_sub(results.len() + 1) else {
                    break;
                };
                remaining = next;
                if results.len() != 1
                    || !selected.contains(&results[0].1)
                    || !valid.contains_key(callee)
                {
                    continue;
                }
                let Some(owner) =
                    crate::rules::lifetime_facts::BodyIdentity::capture(compiler, valid[callee])
                else {
                    continue;
                };
                let mut request = ReadRequests::new();
                if let Some(old) = demands.get(callee) {
                    let old: &ContextReads = old;
                    for field in &old.fields {
                        if request.add(old.ty, *field).is_none() {
                            failed.insert(*callee);
                        }
                    }
                }
                for field in &reads.fields {
                    if request.add(reads.ty, *field).is_none() {
                        failed.insert(*callee);
                    }
                }
                if failed.contains(callee) {
                    demands.remove(callee);
                    continue;
                }
                if let Some((ty, fields)) = request.selected {
                    demands.insert(
                        *callee,
                        ContextReads {
                            owner,
                            ty,
                            fields,
                        },
                    );
                }
            }
        }
    }
    demands
}

impl ReturnedSlots {
    fn current(
        &self,
        r: &Resolver<'_>,
        all: &HashMap<ProcId, Summary>,
        remaining: &mut usize,
    ) -> Option<()> {
        *remaining = remaining.checked_sub(self.fields.len().checked_mul(3)?.checked_add(2)?)?;
        if self.fields.is_empty()
            || self.fields.len() > 3
            || !self.owner.current(r.compiler)
            || !self.lifetime.as_ref()?.current(r.compiler, all, remaining)
        {
            return None;
        }
        for leaf in &self.fields {
            if leaf.width != r.compiler.ty.types.size_of(leaf.ty) {
                return None;
            }
            match leaf.value {
                jaic::ide::lifetimes::LifetimeDefaultValue::Procedure {
                    procedure,
                    function,
                } => {
                    let p = r.compiler.pr.procs.get(procedure.0 as usize)?;
                    if !matches!(p.target,Some(jaic::sema::procs::ProcTarget::Func(actual)) if actual==function)
                        || p.sig.as_ref()?.ty != leaf.ty
                    {
                        return None;
                    }
                }
                jaic::ide::lifetimes::LifetimeDefaultValue::Scalar(_) => {}
                jaic::ide::lifetimes::LifetimeDefaultValue::Unknown => return None,
            }
        }
        (!jaic::interrupt::should_stop()).then_some(())
    }
}
impl Resolver<'_> {
    /// Project one actual field Load from the guarded call snapshot.
    pub(super) fn returned_constant(
        &self,
        all: &HashMap<ProcId, Summary>,
        value: Val,
        remaining: &mut usize,
    ) -> Option<(
        jaic::types::TypeId,
        jaic::ide::lifetimes::LifetimeDefaultValue,
    )> {
        let index = self.capture_index(remaining)?;
        let super::captured_values::Producer::Load(ty, at) = index.get(value)? else {
            return None;
        };
        let Event::Load {
            place,
            value: actual,
            ty: selected,
            ..
        } = self.body.blocks.get(at.block)?.events.get(at.event)?
        else {
            return None;
        };
        if *actual != value || *selected != ty {
            return None;
        }
        let (root, field) = index.write_location(self, *place, ty, remaining, 0)?;
        let captured = self.returned_slots_at(all, &index, root, at, remaining)?;
        let mut selected = None;
        for leaf in captured {
            if leaf.offset == field.offset && leaf.width == field.width && leaf.ty == ty {
                if selected.replace(leaf.value).is_some() {
                    return None;
                }
            }
        }
        Some((ty, selected?))
    }

    /// Observe the exact evaluated hidden operand before this contextual call.
    /// Incoming symbolic context and missing/Null/Absent ABI remain unknown.
    pub(super) fn context_constants(
        &self,
        all: &HashMap<ProcId, Summary>,
        at: Site,
        remaining: &mut usize,
    ) -> Option<Vec<jaic::ide::lifetimes::LifetimeDefaultLeaf>> {
        let Event::Call {
            callee,
            expands: false,
            ..
        } = self.body.blocks.get(at.block)?.events.get(at.event)?
        else {
            return None;
        };
        if !self.compiler.proc(*callee).sig.as_ref()?.has_context {
            return None;
        }
        if let Some(edge) = self.compiler.lifetime_generated_entry(*callee)
            && edge.context != jaic::ide::lifetimes::LifetimeContextTransfer::ForwardHidden
        {
            return None;
        }
        *remaining = remaining.checked_sub(self.body.call_contexts.len() + 1)?;
        let mut rows = self
            .body
            .call_contexts
            .iter()
            .filter(|row| row.block == at.block && row.event == at.event);
        let jaic::ide::lifetimes::LifetimeContextArgument::Value(value) = rows.next()?.argument
        else {
            return None;
        };
        if rows.next().is_some() {
            return None;
        }
        let index = self.capture_index(remaining)?;
        let super::captured_values::Producer::Load(jaic::types::TypeId::VOID_PTR, read) =
            index.get(value)?
        else {
            return None;
        };
        if !matches!(self.body.blocks.get(read.block)?.events.get(read.event),Some(Event::ContextRead{value:actual,..}) if *actual==value)
        {
            return None;
        }
        let root = index.pointer_leaf(self, value, remaining, 0)?.0;
        self.returned_slots_at(all, &index, root, at, remaining)
    }

    fn returned_slots_at(
        &self,
        all: &HashMap<ProcId, Summary>,
        index: &super::captured_values::QueryIndex,
        root: Val,
        at: Site,
        remaining: &mut usize,
    ) -> Option<Vec<jaic::ide::lifetimes::LifetimeDefaultLeaf>> {
        let mut stage = "callee-current";
        let outcome = (|| {
            let super::captured_values::Producer::Call(callee, call, 0, returned_ty) =
                index.get(root)?
            else {
                return None;
            };
            let proof = all.get(&callee)?.returned_slots.as_deref()?;
            if proof.owner.procedure() != callee
                || self.compiler.ty.types.pointee(returned_ty) != Some(proof.ty)
            {
                return None;
            }
            stage = "callee-current";
            proof.current(self, all, remaining)?;
            stage = "binding";
            let signature = self.compiler.proc(callee).sig.as_ref()?;
            let Event::Call {
                callee: actual,
                args,
                results,
                expands: false,
                ..
            } = self.body.blocks.get(call.block)?.events.get(call.event)?
            else {
                return None;
            };
            if *actual != callee
                || signature.returns.as_slice() != [returned_ty]
                || signature.has_context
                || args.len() != signature.params.len()
                || args.len() > 32
                || results.as_slice() != [(0, root, returned_ty)]
            {
                return None;
            }
            *remaining = remaining.checked_sub(args.len().checked_mul(args.len() + 1)?)?;
            for (formal, param) in signature.params.iter().enumerate() {
                let mut actual = args.iter().filter(|arg| arg.formal == formal);
                let arg = actual.next()?;
                if actual.next().is_some() || arg.ty != param.ty || arg.spread {
                    return None;
                }
            }
            stage = "nonnull-flow";
            let blocks = self.body.blocks.len();
            if blocks == 0 || blocks > 32 || call.block == at.block {
                return None;
            }
            *remaining = remaining.checked_sub(blocks)?;
            let count = self.body.blocks.iter().try_fold(0usize, |n, b| {
                n.checked_add(b.events.len() + b.successors.len() + 1)
            })?;
            if count > 256
                || self
                    .body
                    .blocks
                    .iter()
                    .any(|b| b.successors.iter().any(|next| *next >= blocks))
            {
                return None;
            }
            *remaining = remaining.checked_sub(count)?;
            let budget = std::cell::Cell::new(*remaining);
            let failed = std::cell::Cell::new(false);
            let debit = |n: usize| {
                if let Some(next) = budget.get().checked_sub(n) {
                    budget.set(next);
                    !jaic::interrupt::should_stop()
                } else {
                    budget.set(0);
                    failed.set(true);
                    false
                }
            };
            let mut edge = |_: usize, to: usize, visited: &mut u32| {
                if !debit(3) {
                    return None;
                }
                let bit = 1u32.checked_shl(u32::try_from(to).ok()?)?;
                if *visited & bit != 0 {
                    return None;
                }
                *visited |= bit;
                Some(())
            };
            let reached = super::flow::states_except_edges(
                self.body,
                &super::flow::Live::all(self.body),
                1u32,
                &HashSet::default(),
                |into, from| {
                    let old = *into;
                    *into |= *from;
                    old != *into
                },
                |_, _, _| {
                    let _ = debit(1);
                },
                Some(&mut edge),
            );
            *remaining = budget.get();
            let reached = reached?;
            if failed.get() || reached[at.block].is_none() {
                return None;
            }
            let block = &self.body.blocks[call.block];
            *remaining = remaining.checked_sub(block.events.len())?;
            let (branch, condition, then_block, else_block) = block
                .events
                .iter()
                .enumerate()
                .skip(call.event + 1)
                .find_map(|(event, e)| match e {
                    Event::Branch {
                        condition,
                        then_block,
                        else_block,
                        ..
                    } => Some((event, *condition, *then_block, *else_block)),
                    _ => None,
                })?;
            let _ = branch;
            if then_block == else_block || block.successors != [then_block, else_block] {
                return None;
            }
            let truth = index.null_test(
                self,
                &self.core().storage_versions,
                condition,
                root,
                remaining,
                0,
            )?;
            let success = if truth {
                then_block
            } else {
                else_block
            };
            *remaining = remaining.checked_sub(count)?;
            let mut bypass = false;
            super::flow::states_except(
                self.body,
                &super::flow::Live::all(self.body),
                (),
                &HashSet::from_iter([(call.block, success)]),
                |(), ()| false,
                |(), site, _| {
                    bypass |= site == at;
                },
            )?;
            if bypass || jaic::interrupt::should_stop() {
                return None;
            }
            *remaining = remaining.checked_sub(proof.fields.len() * 2)?;
            let fields: Vec<_> = proof
                .fields
                .iter()
                .map(|leaf| {
                    (
                        leaf.ty,
                        crate::rules::lifetime_facts::ByteRange {
                            offset: leaf.offset,
                            width: leaf.width,
                        },
                    )
                })
                .collect();
            let mut values = [None; 3];
            for (i, leaf) in proof.fields.iter().enumerate() {
                values[i] = Some(leaf.value);
            }
            stage = "call-transfer";
            let (captured, access) = super::storage_versions::requested_call_constants(
                self,
                all,
                index,
                (root, proof.ty, &fields, values, call, at),
                remaining,
            )?;
            if !proof.range.covers_relative(access) {
                return None;
            }
            proof.current(self, all, remaining)?;
            Some(captured)
        })();
        if outcome.is_none() && jacket_config::env::LIFETIME_STATS.is_set() {
            eprintln!(
                "lifetime-call-constant-stop proc={:?} root={root:?} at={at:?} stage={stage} remaining={remaining}",
                self.body.procedure
            );
        }
        outcome
    }
}
