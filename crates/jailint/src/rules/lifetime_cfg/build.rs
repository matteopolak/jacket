//! Lower selected events into common storage, resource and logical-element operations.
use super::{
    Action, Block, Cell, Cells, Cx, Event, Facts, HashMap, HashSet, Kind, LifetimeBody,
    LifetimeOperand, Mutation, Op, Places, Resolver, Selector, Site, Span, Summaries, TypeId,
    resizable, resources, view,
};

pub(super) fn build(
    cx: &Cx,
    body: &LifetimeBody,
    places: &mut Places,
    cells: &mut Cells,
    summaries: &Summaries,
    chains: &mut HashMap<(Span, usize), Vec<Span>>,
) -> Vec<Block> {
    let mut resolver = Resolver::new(cx.compiler, body, &summaries.procs, &summaries.targets);
    let tasks = super::tasks::Builder::new(cx, body, summaries, places, &mut resolver);
    let mut builder = Builder {
        cx,
        places,
        cells,
        summaries,
        chains,
        resolver,
        tasks,
        resources: resources::Builder::new(cx, body, summaries),
        elements: Facts::new(cx.compiler, body),
        grown: HashMap::default(),
    };
    let mut blocks: Vec<_> = body
        .blocks
        .iter()
        .map(|_| Block {
            op: Op::Nop,
            next: Vec::new(),
            site: None,
        })
        .collect();
    let mut ranges = vec![Vec::new(); body.blocks.len()];
    for range in &body.source_ranges {
        ranges[range.block].push(range);
    }
    for (i, block) in body.blocks.iter().enumerate() {
        let mut previous = i;
        let mut range_index = 0;
        for (ordinal, event) in block.events.iter().enumerate() {
            while ranges[i]
                .get(range_index)
                .is_some_and(|r| r.end_event <= ordinal)
            {
                range_index += 1;
            }
            let site = ranges[i]
                .get(range_index)
                .filter(|r| r.first_event <= ordinal && ordinal < r.end_event)
                .map(|r| r.site);
            let ops = builder.operations(
                event,
                Site {
                    block: i,
                    event: ordinal,
                },
            );
            for op in ops {
                let index = blocks.len();
                blocks.push(Block {
                    op,
                    next: Vec::new(),
                    site,
                });
                if !matches!(blocks[previous].op, Op::Stop) {
                    blocks[previous].next.push(index);
                }
                previous = index;
            }
        }
        if block.normal_return && !matches!(blocks[previous].op, Op::Stop) {
            let index = blocks.len();
            blocks.push(Block {
                op: Op::Resource(resources::Op::Exit),
                next: Vec::new(),
                site: None,
            });
            blocks[previous].next.push(index);
            previous = index;
        }
        if !matches!(blocks[previous].op, Op::Stop) {
            blocks[previous].next.clone_from(&block.successors);
        }
    }
    blocks
}

struct Builder<'a> {
    cx: &'a Cx<'a>,
    places: &'a mut Places,
    cells: &'a mut Cells,
    summaries: &'a Summaries,
    chains: &'a mut HashMap<(Span, usize), Vec<Span>>,
    resolver: Resolver<'a>,
    resources: resources::Builder<'a>,
    elements: Facts<'a>,
    grown: HashMap<Span, usize>,
    tasks: super::tasks::Builder,
}

impl Builder<'_> {
    fn operations(&mut self, event: &Event, at: Site) -> Vec<Op> {
        let mut ops = if matches!(
            event,
            Event::Call {
                expands: false,
                ..
            } | Event::IndirectCall {
                allocator: false,
                ..
            }
        ) {
            self.call(event, at)
        } else {
            self.memory(event, at)
        };
        ops.extend(self.returns(event, at));
        ops.extend(
            self.tasks
                .operations(
                    self.cx,
                    event,
                    at,
                    &self.elements,
                    (self.places, self.cells),
                    &mut self.resolver,
                )
                .into_iter()
                .map(Op::Task),
        );
        let extra = self
            .resources
            .operations(event, self.places, self.cells, &mut self.resolver);
        if matches!(event, Event::Store { .. }) {
            ops.splice(0..0, extra.into_iter().map(Op::Resource));
        } else {
            ops.extend(extra.into_iter().map(Op::Resource));
        }
        ops
    }

    fn memory(&mut self, event: &Event, at: Site) -> Vec<Op> {
        let Self {
            cx,
            places,
            cells,
            elements,
            grown,
            resources,
            tasks,
            ..
        } = self;
        let mut ops = Vec::new();
        match *event {
            Event::Branch {
                condition,
                span,
                then_block,
                else_block,
            } => ops.push(Op::Decision(
                span,
                then_block,
                else_block,
                tasks.completion(condition, elements, at),
            )),
            Event::Iteration {
                region,
                ty,
                index,
                reverse,
                name,
                index_name,
                span,
                advance,
            } => {
                let region = places.place(region, ty);
                let index = places.place(index, TypeId::S64);
                ops.push(Op::Iteration {
                    region,
                    index,
                    reverse,
                    name,
                    index_name,
                    span,
                    advance,
                });
            }
            Event::ReadMemory {
                value,
                span,
            } => ops.push(Op::Use(cells.id(Cell::Value(value)), span)),
            Event::Use {
                place,
                ty,
                span,
            } => {
                let p = places.place(place, ty);
                ops.push(Op::Use(cells.id(Cell::Place(p)), span));
            }
            Event::Borrow {
                value,
                region,
                region_ty,
                span,
            } => {
                let r = places.place(region, region_ty);
                let target = cells.id(Cell::Value(value));
                if let Some(position) = elements.position(value, at).filter(|p| p.bounded) {
                    let region = places.import(&elements.places, position.region);
                    ops.push(Op::ElementBorrow(
                        target,
                        region,
                        position.selector,
                        elements.fixed_length(position.region),
                        span,
                    ));
                } else if resizable(cx, places, r) {
                    ops.push(Op::Borrow(target, r, span));
                } else {
                    ops.push(Op::Copy(vec![(target, cells.id(Cell::Place(r)))]));
                }
            }
            Event::Load {
                place,
                value,
                ty,
                span,
            } => {
                let p = places.place(place, ty);
                let target = cells.id(Cell::Value(value));
                if let Some(&(r, at)) = places.data.get(&p) {
                    if resizable(cx, places, r) {
                        ops.push(Op::Borrow(target, r, at));
                    } else {
                        ops.push(Op::Copy(vec![(target, cells.id(Cell::Place(r)))]));
                    }
                } else if cx.compiler.ty.types.is_pointer(ty)
                    || view(cx, ty)
                    || resources.resource_load(value)
                {
                    ops.push(Op::PointerLoad(target, cells.id(Cell::Place(p)), p, span));
                } else {
                    ops.push(Op::Copy(vec![(target, cells.id(Cell::Place(p)))]));
                }
            }
            Event::Store {
                place,
                value,
                ty,
                span,
                ..
            } => {
                let p = places.place(place, ty);
                ops.push(Op::IndexWrite(p));
                let ep = elements.place_of(place, ty);
                if let Some(parent) = elements.count_parent(ep) {
                    let region = places.import(&elements.places, parent);
                    let selector = elements.selector(value, Some(parent), at);
                    ops.push(Op::Mutate(
                        region,
                        Mutation::new(Action::Count, selector),
                        span,
                    ));
                }
                let source = places.place(value, ty);
                if places.unresolved(p) {
                    ops.push(Op::Barrier);
                }
                ops.push(Op::Replace(p));
                if let Some(&r) = places.descriptor_fields.get(&p)
                    && !elements.is_count(ep)
                {
                    ops.push(Op::Replace(r));
                }
                let target = cells.id(Cell::Place(p));
                if cx.compiler.ty.types.is_pointer(ty) {
                    ops.push(Op::Copy(vec![(target, cells.id(Cell::Value(value)))]));
                } else {
                    let mut copies = vec![(target, cells.id(Cell::Place(source)))];
                    for (dst, src) in places.copied_fields(p, source) {
                        copies.push((cells.id(Cell::Place(dst)), cells.id(Cell::Place(src))));
                    }
                    ops.push(Op::Copy(copies));
                }
                if let Some(&(parent, _)) = places.data.get(&p)
                    && places.types.get(&parent).is_some_and(|&ty| view(cx, ty))
                {
                    ops.push(Op::Copy(vec![(
                        cells.id(Cell::Place(parent)),
                        cells.id(Cell::Value(value)),
                    )]));
                }
            }
            Event::Copy {
                value,
                source,
            } => ops.push(Op::Copy(vec![(
                cells.id(Cell::Value(value)),
                cells.id(Cell::Value(source)),
            )])),
            Event::Offset {
                value,
                source,
                span,
                ..
            } => {
                let target = cells.id(Cell::Value(value));
                ops.push(Op::Copy(vec![(target, cells.id(Cell::Value(source)))]));
                ops.push(Op::ForgetElement(target));
                if let Some(position) = elements.position(value, at).filter(|p| p.bounded) {
                    let region = places.import(&elements.places, position.region);
                    ops.push(Op::ElementBorrow(
                        target,
                        region,
                        position.selector,
                        elements.fixed_length(position.region),
                        span,
                    ));
                }
            }
            Event::View {
                value,
                source,
                from,
                to,
                span,
            } => {
                let source = places.place(source, from);
                let target = cells.id(Cell::Value(value));
                if resizable(cx, places, source) {
                    ops.push(Op::Borrow(target, source, span));
                } else {
                    ops.push(Op::Copy(vec![(target, cells.id(Cell::Place(source)))]));
                }
                let target_place = places.place(value, to);
                ops.push(Op::Copy(vec![(
                    cells.id(Cell::Place(target_place)),
                    target,
                )]));
            }
            Event::Grow {
                region,
                ty,
                span,
            } => {
                let r = places.place(region, ty);
                grown.insert(span, r);
                ops.push(Op::Grow(r, span));
            }
            Event::Barrier => ops.push(Op::Barrier),
            _ => {}
        }

        ops
    }

    fn call(&mut self, event: &Event, at: Site) -> Vec<Op> {
        let (args, _results, span, direct) = match event {
            Event::Call {
                callee,
                args,
                results,
                span,
                ..
            } => (args, results, *span, Some(*callee)),
            Event::IndirectCall {
                args,
                results,
                span,
                ..
            } => (args, results, *span, None),
            _ => return Vec::new(),
        };
        let Self {
            cx,
            places,
            resolver,
            summaries,
            elements,
            grown,
            chains,
            ..
        } = self;
        let mut ops = Vec::new();
        if direct.is_none() {
            ops.push(Op::Indirect(resolver.call_targets(event), span));
        }

        if direct.is_some_and(|callee| cx.compiler.proc_has_note(callee, "SwapsArguments")) {
            let position = |formal| {
                args.iter()
                    .find(|a| a.formal == formal)
                    .and_then(|a| match a.operand {
                        LifetimeOperand::Value(v) | LifetimeOperand::Place(v) => {
                            elements.position(v, at)
                        }
                        LifetimeOperand::Other => None,
                    })
            };
            if let (Some(a), Some(b)) = (position(0), position(1))
                && a.region == b.region
                && a.bounded
                && b.bounded
            {
                let region = places.import(&elements.places, a.region);
                ops.push(Op::Mutate(
                    region,
                    Mutation {
                        action: Action::Swap,
                        first: a.selector,
                        second: b.selector,
                    },
                    span,
                ));
            }
        }
        let guaranteed = resolver.call_effects(event, &summaries.procs, true);
        let mut may = resolver.call_effects(event, &summaries.procs, false);
        may.extend(
            resolver
                .unknown_effects(event, &summaries.procs)
                .into_iter()
                .map(|e| (e, vec![span])),
        );
        for (effect, chain) in may {
            if cx.lifetime_mode == jacket_config::LifetimeMode::Strict
                && !guaranteed.iter().any(|(e, _)| e == &effect)
                && let Some(id) = resolver.materialize(&effect.target)
            {
                let region = places.import(&resolver.places, id);
                match effect.kind {
                    Kind::Resize => ops.push(Op::MayGrow(region, span)),
                    Kind::Unknown => ops.push(Op::MayReach(region, span)),
                    Kind::Mutation(mut m) => {
                        let p = elements.places.lookup(places, region);
                        let resolve = |s| {
                            if let Selector::Local(v) = s {
                                elements.selector(v, p, at)
                            } else {
                                s
                            }
                        };
                        m.first = resolve(m.first);
                        m.second = resolve(m.second);
                        ops.push(Op::MayMutate(region, m, span));
                    }
                    _ => {}
                }
                chains.entry((span, region)).or_insert(chain);
            }
            if let Kind::Mutation(mutation) = effect.kind
                && effect.guard.is_none()
                && let Some(id) = resolver.materialize(&effect.target)
            {
                let region = places.import(&resolver.places, id);
                ops.push(Op::IterationMutation(region, mutation, span));
            }
            if effect.kind != Kind::Write {
                continue;
            }
            if let Some(id) = resolver.materialize(&effect.target) {
                let p = places.import(&resolver.places, id);
                ops.push(Op::Replace(
                    places.data.get(&p).map_or(p, |(owner, _)| *owner),
                ));
            }
        }
        for (effect, chain) in guaranteed {
            if let Kind::Mutation(mut mutation) = effect.kind
                && effect.guard.is_none()
                && let Some(id) = resolver.materialize(&effect.target)
            {
                let region = places.import(&resolver.places, id);
                let local_region = elements.places.lookup(places, region);
                let resolve = |s| match s {
                    Selector::Local(v) => elements.selector(v, local_region, at),
                    other => other,
                };
                mutation.first = resolve(mutation.first);
                mutation.second = resolve(mutation.second);
                ops.push(Op::Mutate(region, mutation, span));
                chains.insert((span, region), chain.clone());
            }
            if effect.kind != Kind::Resize || effect.guard.is_some() {
                continue;
            }
            if let Some(id) = resolver.materialize(&effect.target) {
                let r = places.import(&resolver.places, id);
                if grown.get(&span) != Some(&r) {
                    ops.push(Op::Grow(r, span));
                    chains.insert((span, r), chain);
                }
            }
        }
        if resolver.call_stops(event, &summaries.procs) {
            ops.push(Op::Stop);
        }
        ops
    }

    fn returns(&mut self, event: &Event, at: Site) -> Vec<Op> {
        let (results, span) = match event {
            Event::Call {
                results,
                span,
                expands: false,
                ..
            }
            | Event::IndirectCall {
                results,
                span,
                ..
            } => (results, *span),
            _ => return Vec::new(),
        };
        let Self {
            cx,
            places,
            cells,
            resolver,
            summaries,
            elements,
            ..
        } = self;
        let mut resized = HashSet::default();
        for (e, _) in resolver.call_effects(event, &summaries.procs, false) {
            if e.kind == Kind::Resize
                && let Some(p) = resolver.materialize(&e.target)
            {
                resized.insert(places.import(&resolver.places, p));
            }
        }
        let allocator_resize = matches!(event, Event::IndirectCall { allocator: true, args, .. } if args.first().and_then(|a| a.constant) == Some(1));
        let mut ops = Vec::new();
        for &(index, value, ty) in results {
            let target = cells.id(Cell::Value(value));
            if let Some(arg) = resolver.passthrough(event, index, &summaries.procs) {
                if let LifetimeOperand::Value(source) = arg.operand {
                    ops.push(Op::Copy(vec![(target, cells.id(Cell::Value(source)))]));
                }
            } else if let Some(path) = resolver.value(value, &summaries.procs)
                && let Some(p) = resolver.materialize(&path)
            {
                let region = places.import(&resolver.places, p);
                if cx.compiler.ty.types.is_pointer(ty)
                    && places
                        .types
                        .get(&region)
                        .is_some_and(|&t| cx.compiler.ty.types.is_pointer(t))
                    && !allocator_resize
                    && !resized.contains(&region)
                {
                    ops.push(Op::PointerLoad(
                        target,
                        cells.id(Cell::Place(region)),
                        region,
                        span,
                    ));
                } else {
                    ops.push(Op::Borrow(target, region, span));
                }
            }
            if let Some((path, key)) = resolver.element(value, &summaries.procs)
                && let Some(id) = resolver.materialize(&path)
            {
                let region = places.import(&resolver.places, id);
                let key = match key {
                    Selector::Local(v) => elements.selector(v, None, at),
                    Selector::Result(index) => results
                        .iter()
                        .find(|(n, _, _)| *n == index)
                        .map_or(Selector::Unknown, |(_, v, _)| {
                            elements.selector(*v, None, at)
                        }),
                    other => other,
                };
                ops.push(Op::LogicalBorrow(target, region, key, None, span));
            }
        }
        ops
    }
}
