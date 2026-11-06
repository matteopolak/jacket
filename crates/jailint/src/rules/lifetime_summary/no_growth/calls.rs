//! Actual stack-call applicability, separate from exported formal templates.
use super::super::{
    Event, HashMap, HashSet, LifetimeArgument, LifetimeBody, LifetimeOperand, Resolver, Summaries,
    TypeId, TypeKind, Val,
};
use crate::rules::lifetime_facts::{Address, ByteRange, StackRoots, current_body};
use crate::rules::lifetime_movement::facts::Site;
use jaic::sema::Compiler;

mod native;

pub(in crate::rules) const MAX_WORK: usize = 8192;

/// One successful acyclic invocation, valid at the admitted call prestate only.
#[derive(Clone, Copy, Debug)]
pub(in crate::rules) struct Backing {
    pub root: Val,
    pub bytes: u64,
    pub alignment: u64,
    pub acquisition: Site,
}

#[derive(Clone, Copy, Debug)]
pub(in crate::rules) struct Write {
    pub address: Address,
    pub bytes: u64,
}

#[derive(Clone, Debug)]
pub(in crate::rules) struct Call {
    pub span: jaic::source::Span,
    pub count_origin: Option<jaic::source::Span>,
    pub source: Val,
    pub result: Val,
    pub header: usize,
    pub header_value: Val,
    pub header_type: TypeId,
    pub receiver: usize,
    pub count: i64,
    pub native: Option<Box<Backing>>,
    pub writes: [Write; 2],
}

#[derive(Default)]
pub(in crate::rules) struct Calls {
    pub addresses: Option<Addresses>,
    pub work: usize,
    entries: HashMap<Site, Call>,
    grows: HashSet<Site>,
}

impl Calls {
    pub fn collect(r: &Resolver<'_>, all: &Summaries) -> Self {
        let (c, body) = (r.compiler, r.body);
        let total = body.blocks.iter().try_fold(0usize, |n, b| {
            n.checked_add(b.events.len() + b.successors.len() + 1)
        });
        if body.blocks.is_empty()
            || total.is_none_or(|n| n > MAX_WORK)
            || body.blocks.iter().any(|b| b.successors.contains(&0))
            || !current_body(c, body.procedure, body)
            || !r.call_slots.iter().any(|((site, _), s)| {
                site.event < 128
                    && s.bounds().is_some_and(|(n, cap)| {
                        (n >= 0 && n < cap) || (n == 0 && cap == 0 && s.null_data(c))
                    })
            })
        {
            return Self::default();
        }
        let mut work = total.unwrap().saturating_mul(3).saturating_add(
            body.stack_roots.len() + body.slot_aliases.len() + body.address_casts.len(),
        );
        if work > MAX_WORK {
            return Self::default();
        }
        let Some(roots) = StackRoots::capture(body, MAX_WORK - work) else {
            return Self::default();
        };
        let Some(addresses) = Addresses::collect(c, body, roots) else {
            return Self::default();
        };
        let mut out = Self::default();
        let mut capture_index = None;
        let mut requests: Vec<_> = r.call_slots.iter().collect();
        if requests.len() > 64 {
            return out;
        }
        work = work.saturating_add(requests.len() * requests.len());
        requests.sort_by_key(|((s, f), _)| (s.block, s.event, *f));
        for (&(site, formal), snapshot) in requests {
            if site.event >= 128 {
                continue;
            }
            work = work.saturating_add(total.unwrap() + 128);
            if work > MAX_WORK || jaic::interrupt::should_stop() {
                return Self::default();
            }
            let Some(call) = Self::candidate(
                r,
                &mut capture_index,
                &addresses,
                all,
                (site, formal),
                snapshot,
                &mut work,
            ) else {
                continue;
            };
            if snapshot.prestate != site {
                let Some(Event::Grow {
                    call_event,
                    region,
                    ty,
                    ..
                }) = body.blocks[site.block].events.get(snapshot.prestate.event)
                else {
                    continue;
                };
                if snapshot.prestate.block != site.block
                    || snapshot.prestate.event + 1 != site.event
                    || *call_event != site.event
                    || *region != call.header_value
                    || *ty != call.header_type
                {
                    continue;
                }
                out.grows.insert(snapshot.prestate);
            }
            out.entries.insert(site, call);
        }
        if out.entries.is_empty() {
            return Self::default();
        }
        if work > MAX_WORK || jaic::interrupt::should_stop() {
            return Self::default();
        }
        out.work = work;
        out.addresses = Some(addresses);
        out
    }

    pub fn charge(&mut self, n: usize) -> Option<()> {
        self.work = self.work.checked_add(n)?;
        (self.work <= MAX_WORK && !jaic::interrupt::should_stop()).then_some(())
    }

    fn candidate(
        r: &Resolver<'_>,
        capture_index: &mut Option<Option<super::super::captured_values::QueryIndex>>,
        addresses: &Addresses,
        all: &Summaries,
        request: (Site, usize),
        snapshot: &super::super::call_slots::Snapshot,
        work: &mut usize,
    ) -> Option<Call> {
        let (site, formal) = request;
        let (c, body) = (r.compiler, r.body);
        let roots = &addresses.roots;
        let Event::Call {
            callee,
            args,
            results,
            expands: false,
            ..
        } = body.blocks.get(site.block)?.events.get(site.event)?
        else {
            return None;
        };
        let Event::Call {
            span, ..
        } = &body.blocks[site.block].events[site.event]
        else {
            return None;
        };
        // Source spans identify diagnostic witnesses, not dynamic call events.
        // Shared-span chains remain untouched even when the event has a proof.
        let unique_span = body
            .blocks
            .iter()
            .flat_map(|b| &b.events)
            .filter(
                |e| matches!(e,Event::Call {span:s,..}|Event::IndirectCall {span:s,..} if s==span),
            )
            .count()
            == 1;
        let template = all.procs.get(callee)?.no_growth.as_ref()?;
        *work = work.checked_add(template.compatibility_work(&all.procs))?;
        if *work > MAX_WORK {
            return None;
        }
        if !template.current(c)
            || !template.compatible(&all.procs)
            || template.receiver != formal
            || template.header != snapshot.request.ty
            || template.element != snapshot.request.element
            || args.len() != 1
            || results.len() != 1
        {
            return None;
        }
        let sig = c.pr.procs.get(callee.0 as usize)?.sig.as_ref()?;
        let arg = &args[0];
        let LifetimeOperand::Value(header_value) = arg.operand else {
            return None;
        };
        if arg.spread
            || arg.formal != formal
            || sig.params.len() != 1
            || sig.params[0].ty != arg.ty
            || c.ty.types.pointee(arg.ty) != Some(template.header)
        {
            return None;
        }
        let header = roots.canonical(header_value);
        if roots.canonical(r.places.paths[snapshot.request.header].0.0) != header {
            return None;
        }
        let header_bytes = c.ty.types.size_of(template.header);
        if !(ByteRange {
            offset: 0,
            width: header_bytes,
        })
        .within(roots.extent(header).flatten())
        {
            return None;
        }
        let (count, cap) = snapshot.bounds()?;
        if count == 0 && cap == 0 && snapshot.null_data(c) {
            // An empty descriptor enters the allocating path, not the child's
            // read-only fast path. Keep ordinary Grow/Call effects until the
            // selected provider range and post-growth DATA bridge are proved.
            if jacket_config::env::LIFETIME_STATS.is_set() {
                eprintln!(
                    "lifetime-initial-append proc={} block={} event={} stop=provider-range-context",
                    body.procedure.0, site.block, site.event
                );
            }
            return None;
        }
        if count < 0 || cap <= count || !template.guards(count, cap) {
            return None;
        }
        let next = count.checked_add(1)?;
        let source = snapshot.data()?;
        let native = if addresses.get(source).is_none() {
            Some(native::backing(
                r,
                capture_index,
                addresses,
                all,
                source,
                snapshot.prestate,
                work,
            )?)
        } else {
            None
        };
        if site.block != 0 && native.is_none() {
            return None;
        }
        let data = addresses.get(source).or_else(|| {
            native.map(|n| Address {
                root: n.root,
                offset: 0,
            })
        })?;
        if data.root == header {
            return None;
        }
        let available = if let Some(native) = native {
            if roots.extent(native.root).is_some()
                || c.ty.types.align_of(template.element) > native.alignment
            {
                return None;
            }
            native.bytes
        } else {
            let mut declared = None;
            for e in &body.blocks[0].events {
                if let Event::Local {
                    place,
                    ty,
                    ..
                } = e
                    && roots.canonical(*place) == data.root
                {
                    let TypeKind::Array {
                        kind: super::super::ArrayKind::Fixed(_),
                        elem,
                    } = *c.ty.types.kind(*ty)
                    else {
                        return None;
                    };
                    if elem != template.element {
                        return None;
                    }
                    let bytes = c.ty.types.size_of(*ty);
                    if declared.is_some_and(|old| old != bytes) {
                        return None;
                    }
                    declared = Some(bytes);
                }
            }
            declared?.min(roots.extent(data.root).flatten()?)
        };
        let width = c.ty.types.size_of(template.element);
        if width == 0 || data.offset % width != 0 {
            return None;
        }
        let capacity = u64::try_from(cap).ok()?.checked_mul(width)?;
        if !(ByteRange {
            offset: data.offset,
            width: capacity,
        })
        .within(Some(available))
        {
            return None;
        }
        let offset = data
            .offset
            .checked_add(u64::try_from(count).ok()?.checked_mul(width)?)?;
        if !(ByteRange {
            offset,
            width,
        })
        .within(Some(available))
        {
            return None;
        }
        let (index, result, ty) = results[0];
        if index != template.result
            || c.ty.types.pointee(ty) != Some(template.element)
            || sig.returns.as_slice() != [ty]
        {
            return None;
        }
        let Event::Store {
            span: count_origin,
            ..
        } =
            c.ed.facts()?
                .lifetime_bodies
                .get(callee)?
                .blocks
                .get(template.count_write.block)?
                .events
                .get(template.count_write.event)?
        else {
            return None;
        };
        Some(Call {
            span: *span,
            count_origin: unique_span.then_some(*count_origin),
            source,
            result,
            header: snapshot.request.header,
            header_value,
            header_type: template.header,
            receiver: formal,
            count: next,
            native: native.map(Box::new),
            writes: [
                Write {
                    address: Address {
                        root: data.root,
                        offset,
                    },
                    bytes: width,
                },
                Write {
                    address: Address {
                        root: header,
                        offset: jaic::layout::resizable::COUNT,
                    },
                    bytes: 8,
                },
            ],
        })
    }

    pub fn get(&self, site: Site) -> Option<&Call> {
        self.entries.get(&site)
    }
    pub fn grow(&self, site: Site) -> bool {
        self.grows.contains(&site)
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    pub fn preserves(&self, site: Site, arg: &LifetimeArgument) -> bool {
        self.get(site).is_some_and(|call| {
            arg.formal == call.receiver
                && matches!(arg.operand,LifetimeOperand::Value(v) if v==call.header_value)
        })
    }
}

/// Original roots and one selected transformation from an original root only.
/// This indexes immutable checked facts; it does not interpret loads or infer aliases.
pub(in crate::rules) struct Addresses {
    pub roots: StackRoots,
    selected: HashMap<Val, Option<Address>>,
    casts: HashSet<Val>,
}

impl Addresses {
    fn collect(c: &Compiler, body: &LifetimeBody, roots: StackRoots) -> Option<Self> {
        let mut selected = HashMap::<Val, Option<Address>>::default();
        for event in body.blocks.iter().flat_map(|b| &b.events) {
            if jaic::interrupt::should_stop() {
                return None;
            }
            let (value, address) = match event {
                Event::FieldProjection {
                    base,
                    result,
                    offset,
                    ..
                } if base != result || *offset != 0 => {
                    let root = roots.canonical(*base);
                    (
                        *result,
                        roots
                            .extent(root)
                            .flatten()
                            .filter(|bytes| offset <= bytes)
                            .map(|_| Address {
                                root,
                                offset: *offset,
                            }),
                    )
                }
                Event::Offset {
                    value,
                    source,
                    stride,
                    offset,
                    ..
                } => {
                    let root = roots.canonical(*source);
                    let offset = offset
                        .and_then(|n| n.checked_mul(i128::from(c.ty.types.size_of(*stride))))
                        .and_then(|n| u64::try_from(n).ok());
                    (
                        *value,
                        offset
                            .zip(roots.extent(root).flatten())
                            .filter(|(offset, bytes)| offset <= bytes)
                            .map(|(offset, _)| Address {
                                root,
                                offset,
                            }),
                    )
                }
                Event::Copy {
                    value,
                    source,
                } if value != source => {
                    let root = roots.canonical(*source);
                    (
                        *value,
                        roots.extent(root).flatten().map(|_| Address {
                            root,
                            offset: 0,
                        }),
                    )
                }
                Event::View {
                    value,
                    from,
                    to,
                    ..
                } if !c.ty.types.is_pointer(*from) || !c.ty.types.is_pointer(*to) => (*value, None),
                _ => continue,
            };
            if let Some(old) = selected.get_mut(&value) {
                if *old != address {
                    *old = None;
                }
            } else {
                selected.insert(value, address);
            }
        }
        Some(Self {
            roots,
            selected,
            casts: body.address_casts.iter().map(|c| c.value).collect(),
        })
    }

    pub fn get(&self, value: Val) -> Option<Address> {
        if self.casts.contains(&value) {
            return None;
        }
        let root = self.roots.canonical(value);
        let actual = match self.roots.extent(root) {
            Some(None) => return None,
            Some(Some(_)) => Some(Address {
                root,
                offset: 0,
            }),
            None => None,
        };
        match self.selected.get(&value) {
            Some(Some(address)) if actual.is_none_or(|old| old == *address) => Some(*address),
            Some(_) => None,
            None => actual,
        }
    }
}
