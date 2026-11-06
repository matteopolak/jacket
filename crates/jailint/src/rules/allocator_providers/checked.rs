#[path = "calls.rs"]
mod calls;
#[path = "events.rs"]
mod events;

use super::bindings::Bindings;
use super::domain::*;
use super::*;
use jaic::fxhash::HashSet;
use jaic::ide::lifetimes::{
    LifetimeArgument, LifetimeBody, LifetimeBoolean, LifetimeEvent as Event, LifetimeOperand,
};
use jaic::ir::BinOp;
use jaic::sema::procs::BodyState;

pub(super) struct Solver<'a> {
    c: &'a Compiler,
    bodies: HashMap<ProcId, &'a LifetimeBody>,
    cache: HashMap<ProcId, Option<Summary>>,
    active: HashSet<ProcId>,
    candidates: HashSet<ProcId>,
}

impl<'a> Solver<'a> {
    pub fn new(c: &'a Compiler) -> Self {
        let bodies =
            c.ed.facts()
                .into_iter()
                .flat_map(|f| &f.lifetime_bodies)
                .filter(|(id, b)| {
                    !jaic::interrupt::should_stop()
                        && b.procedure == **id
                        && c.pr.procs[id.0 as usize].body_state == BodyState::Done
                        && std::sync::Arc::ptr_eq(&c.pr.procs[id.0 as usize].lit, &b.syntax)
                })
                .map(|(id, b)| (*id, b))
                .collect();
        let candidates = candidates(&bodies);
        Self {
            c,
            bodies,
            cache: HashMap::default(),
            active: HashSet::default(),
            candidates,
        }
    }

    pub fn candidate(&self, id: ProcId) -> bool {
        self.candidates.contains(&id)
    }

    pub fn summary(&mut self, id: ProcId) -> Option<Summary> {
        if jaic::interrupt::should_stop() {
            return None;
        }
        if let Some(s) = self.cache.get(&id) {
            return s.clone();
        }
        if self.active.len() >= MAX_DEPTH || !self.active.insert(id) {
            return None;
        }
        let result = self.bodies.get(&id).copied().and_then(|b| self.body(b));
        self.active.remove(&id);
        if jaic::interrupt::should_stop() {
            return None;
        }
        self.cache.insert(id, result.clone());
        result
    }

    fn initial(&self, b: &LifetimeBody) -> Option<(State, Summary)> {
        if b.blocks.is_empty()
            || b.stack_roots.len() > MAX_STEPS
            || b.slot_aliases.len() > MAX_STEPS
        {
            return None;
        }
        let mut initial = State::default();
        for root in &b.stack_roots {
            if jaic::interrupt::should_stop() {
                return None;
            }
            initial.extents.insert(root.value, root.bytes);
            initial.addresses.insert(
                root.value,
                Address {
                    root: Root::Private(root.value),
                    offset: 0,
                },
            );
        }
        for (alias, root) in &b.slot_aliases {
            initial.addresses.insert(
                *alias,
                Address {
                    root: Root::Private(*root),
                    offset: 0,
                },
            );
        }
        let mut summary = Summary {
            pointer_results: self.c.pr.procs[b.procedure.0 as usize]
                .sig
                .as_ref()
                .into_iter()
                .flat_map(|s| s.returns.iter().enumerate())
                .filter_map(|(i, t)| {
                    (self.c.ty.types.pointee(*t).is_some()
                        || !scalar(self.c, *t) && self.c.ty.types.size_of(*t) > 0)
                        .then_some(i)
                })
                .collect(),
            ..Summary::default()
        };
        let mut seen_stores = Vec::new();
        for (step, (block, event, e)) in b
            .blocks
            .iter()
            .enumerate()
            .flat_map(|(block, b)| {
                b.events
                    .iter()
                    .enumerate()
                    .map(move |(event, e)| (block, event, e))
            })
            .enumerate()
        {
            if step >= MAX_STEPS || jaic::interrupt::should_stop() {
                return None;
            }
            match e {
                Event::ContextSet {
                    ..
                } if summary.initial_context.is_none()
                    && self.c.pr.procs[b.procedure.0 as usize]
                        .sig
                        .as_ref()
                        .is_some_and(|s| s.has_context) =>
                {
                    summary.initial_context = Some(site(b.procedure, block, event));
                }
                Event::Store {
                    place,
                    value,
                    ty,
                    ..
                } => seen_stores.push((*place, *value, *ty, site(b.procedure, block, event))),
                Event::Parameter {
                    formal,
                    place,
                    incoming,
                    ty,
                    ..
                } if !scalar(self.c, *ty) => {
                    let a = initial.address(*place)?;
                    if initial.checked(self.c, &a, self.c.ty.types.size_of(*ty))
                        && let Some((_, _, _, copy)) = seen_stores
                            .iter()
                            .rev()
                            .find(|(p, v, t, _)| p == place && v == incoming && t == ty)
                    {
                        summary.parameter_copies.insert(*formal, copy.clone());
                    }
                }
                _ => {}
            }
        }
        Some((initial, summary))
    }

    fn body(&mut self, b: &LifetimeBody) -> Option<Summary> {
        let (initial, mut summary) = self.initial(b)?;
        let acyclic = acyclic_prefix(b)?;
        let mut work = vec![(0, initial)];
        let mut steps = 0;
        while let Some((block, mut state)) = work.pop() {
            if jaic::interrupt::should_stop() {
                return None;
            }
            if !acyclic.contains(&block) || !state.visited.insert(block) {
                summary.partial = true;
                observe(&mut summary, &state);
                continue;
            }
            let bb = b.blocks.get(block)?;
            let mut states = vec![state];
            let mut branch = None;
            for (event, e) in bb.events.iter().enumerate() {
                steps += states.len();
                if jaic::interrupt::should_stop() {
                    return None;
                }
                if steps > MAX_STEPS {
                    summary.partial = true;
                    return Some(summary);
                }
                let anchor = site(b.procedure, block, event);
                let mut next = Vec::new();
                for mut s in states {
                    comparisons(self.c, b, &mut s);
                    if matches!(
                        e,
                        Event::Call {
                            expands: false,
                            ..
                        }
                    ) {
                        match self.call(e, &anchor, &mut s, &mut summary, &mut next) {
                            calls::Next::Continue => {}
                            calls::Next::Forked => continue,
                            calls::Next::Partial => return Some(summary),
                        }
                    } else {
                        events::transfer(
                            self.c,
                            b,
                            e,
                            &anchor,
                            &mut s,
                            summary.initial_context.as_ref(),
                            &mut branch,
                        );
                    }
                    comparisons(self.c, b, &mut s);
                    observe(&mut summary, &s);
                    if !s.bounded() {
                        summary.partial = true;
                        return Some(summary);
                    }
                    next.push(s);
                }
                if next.len() + work.len() > MAX_ITEMS {
                    summary.partial = true;
                    return Some(summary);
                }
                states = next;
            }
            for s in states {
                if bb.normal_return {
                    summary.exits.push(s);
                } else if let Some((condition, yes, no)) = branch {
                    let p = s.simplified(&s.value(condition).boolean(self.c));
                    for (target, truth) in [(yes, true), (no, false)] {
                        if p.known().is_some_and(|k| k != truth) {
                            continue;
                        }
                        let mut fork = s.clone();
                        if p.known().is_none() {
                            fork.incomplete |= !p.supported();
                            fork.guards.push(Guard {
                                predicate: p.clone(),
                                truth,
                            });
                        }
                        work.push((target, fork));
                    }
                } else {
                    for next in &bb.successors {
                        work.push((*next, s.clone()));
                    }
                }
            }
            if summary.exits.len() + work.len() > MAX_ITEMS {
                summary.partial = true;
                return Some(summary);
            }
        }
        Some(summary)
    }
}

pub(super) fn operand(c: &Compiler, s: &State, v: Val, ty: TypeId) -> Value {
    let mut value = s.value(v).retyped(c, ty);
    value.unsupported_address |=
        s.unsupported_addresses.contains(&v) || s.value(v).unsupported_address;
    if c.ty.types.pointee(ty).is_some()
        && let Some(address) = s.address(v)
    {
        if value.address.is_none() {
            value.address = Some(address.clone());
        }
        if value.parameter_pointer.is_none()
            && let Root::Receiver {
                formal, ..
            } = address.root
            && let Ok(offset) = i64::try_from(address.offset)
        {
            value.parameter_pointer = Some((formal, offset));
        }
    }
    value
}

fn argument(c: &Compiler, s: &State, args: &[LifetimeArgument], n: usize) -> Value {
    let Some(a) = args.iter().find(|a| a.formal == n && !a.spread) else {
        return Value::unknown(TypeId::VOID);
    };
    if let Some(bits) = a.constant {
        return literal(c, a.ty, bits);
    }
    match a.operand {
        LifetimeOperand::Value(v) => operand(c, s, v, a.ty),
        _ => Value::unknown(a.ty),
    }
}

fn opaque(c: &Compiler, s: &mut State, args: &[LifetimeArgument]) {
    if let Some(Address {
        root: Root::Private(v),
        ..
    }) = s.current_context_address.clone()
    {
        s.expose(v);
    }
    for a in args {
        let value = argument(c, s, args, a.formal);
        if c.ty.types.pointee(a.ty).is_some() {
            if let Some(Address {
                root: Root::Private(v),
                ..
            }) = value.address
            {
                s.expose(v);
            }
        } else {
            let (LifetimeOperand::Value(v) | LifetimeOperand::Place(v)) = a.operand else {
                continue;
            };
            if let Some(address) = s.address(v) {
                let nested: Vec<_> = s
                    .slots
                    .iter()
                    .filter(|((b, _), _)| b.root == address.root)
                    .filter_map(|(_, v)| match &v.address {
                        Some(Address {
                            root: Root::Private(v),
                            ..
                        }) => Some(*v),
                        _ => None,
                    })
                    .collect();
                for v in nested {
                    s.expose(v);
                }
            }
        }
    }
    s.clobber();
}

fn protocol(c: &Compiler, e: &Event) -> Option<Protocol> {
    let Event::IndirectCall {
        allocator_protocol,
        args,
        results,
        ..
    } = e
    else {
        return None;
    };
    let p = allocator_protocol.as_deref()?;
    let mode = args
        .iter()
        .find(|a| a.formal == p.operation_formal && !a.spread)?;
    let data = args
        .iter()
        .find(|a| a.formal == p.data_formal && !a.spread)?;
    if mode.ty != p.operation_ty
        || c.ty.types.pointee(data.ty).is_none()
        || c.ty.types.size_of(data.ty) != 8
    {
        return None;
    }
    if !results.iter().any(|(i, _, ty)| {
        *i == p.result_position && c.ty.types.pointee(*ty).is_some() && c.ty.types.size_of(*ty) == 8
    }) {
        return None;
    }
    Some(Protocol {
        operation_formal: p.operation_formal,
        data_formal: p.data_formal,
        operation_ty: p.operation_ty,
        allocate_value: p.allocate_value,
        resize_value: p.resize_value,
        free_value: p.free_value,
        result_position: p.result_position,
    })
}

fn acyclic_prefix(b: &LifetimeBody) -> Option<HashSet<usize>> {
    if b.blocks.len() > MAX_STEPS {
        return None;
    }
    let mut incoming = vec![0usize; b.blocks.len()];
    let mut edges = 0usize;
    for block in &b.blocks {
        for to in &block.successors {
            edges = edges.checked_add(1)?;
            if edges > 65536 || jaic::interrupt::should_stop() {
                return None;
            }
            *incoming.get_mut(*to)? += 1;
        }
    }
    let mut work: Vec<_> = incoming
        .iter()
        .enumerate()
        .filter_map(|(i, n)| (*n == 0).then_some(i))
        .collect();
    let mut out = HashSet::default();
    while let Some(i) = work.pop() {
        if jaic::interrupt::should_stop() {
            return None;
        }
        out.insert(i);
        for to in &b.blocks[i].successors {
            incoming[*to] -= 1;
            if incoming[*to] == 0 {
                work.push(*to);
            }
        }
    }
    Some(out)
}

fn comparisons(c: &Compiler, b: &LifetimeBody, s: &mut State) {
    for cmp in &b.comparisons {
        let a = cmp
            .left_constant
            .map_or_else(|| s.value(cmp.left), |n| Value::constant(TypeId::S64, n));
        let b = cmp
            .right_constant
            .map_or_else(|| s.value(cmp.right), |n| Value::constant(TypeId::S64, n));
        let p = if let Captured::Constant {
            bits, ..
        } = b.capture
        {
            compare(c, &a, cmp.op, bits, cmp.ty)
        } else if let Captured::Constant {
            bits, ..
        } = a.capture
        {
            compare(c, &b, reverse(cmp.op), bits, cmp.ty)
        } else {
            Predicate::Unknown
        };
        if p != Predicate::Unknown {
            s.values.insert(
                cmp.value,
                Value {
                    predicate: Some(s.simplified(&p)),
                    ..Value::unknown(TypeId::BOOL)
                },
            );
        }
    }
}

fn reverse(op: CmpOp) -> CmpOp {
    match op {
        CmpOp::SLt => CmpOp::SGt,
        CmpOp::SLe => CmpOp::SGe,
        CmpOp::SGt => CmpOp::SLt,
        CmpOp::SGe => CmpOp::SLe,
        CmpOp::ULt => CmpOp::UGt,
        CmpOp::ULe => CmpOp::UGe,
        CmpOp::UGt => CmpOp::ULt,
        CmpOp::UGe => CmpOp::ULe,
        x => x,
    }
}

fn constant_signed(c: &Compiler, v: &Value) -> Option<i128> {
    let Captured::Constant {
        bits, ..
    } = v.capture
    else {
        return None;
    };
    let ty = ir_type(c, v.ty)?;
    let signed = c.ty.types.int_info(v.ty).is_some_and(|(_, s)| s);
    Some(if signed {
        domain::bits(ty, CmpOp::SLt, bits)? as i64 as i128
    } else {
        domain::bits(ty, CmpOp::ULt, bits)? as i128
    })
}

fn arithmetic(c: &Compiler, ty: TypeId, a: &Value, op: BinOp, b: &Value) -> Value {
    let (Some(a), Some(b), Some((width, signed))) = (
        constant_signed(c, a),
        constant_signed(c, b),
        c.ty.types.int_info(ty),
    ) else {
        return Value::unknown(ty);
    };
    if width == 0 || width > 64 {
        return Value::unknown(ty);
    }
    let value = match op {
        BinOp::Add => a.checked_add(b),
        BinOp::Sub => a.checked_sub(b),
        BinOp::Mul => a.checked_mul(b),
        _ => None,
    };
    let (min, max) = if signed {
        (-(1i128 << (width - 1)), (1i128 << (width - 1)) - 1)
    } else {
        (0, (1i128 << width) - 1)
    };
    value
        .filter(|v| *v >= min && *v <= max)
        .map_or_else(|| Value::unknown(ty), |v| literal(c, ty, v as u64))
}

fn observe(summary: &mut Summary, s: &State) {
    for u in &s.uses {
        if u.row.guards.len() <= 32 && !summary.observed_uses.contains(u) {
            if summary.observed_uses.len() < MAX_ITEMS {
                summary.observed_uses.push(u.clone());
            } else {
                summary.partial = true;
            }
        }
    }
    for u in &s.stores {
        if u.guards.len() <= 32 && !summary.observed_stores.contains(u) {
            if summary.observed_stores.len() < MAX_ITEMS {
                summary.observed_stores.push(u.clone());
            } else {
                summary.partial = true;
            }
        }
    }
}

pub(super) fn relations(c: &Compiler, s: &Summary) -> Option<Relations> {
    let mut out = Relations {
        incomplete: s.partial,
        ..Relations::default()
    };
    for u in &s.observed_uses {
        let mut row = u.row.clone();
        row.operation = operation(c, &u.operation, u.protocol.as_ref());
        out.incomplete |= row.operation == Operation::Unknown
            || row.provider.procedure_value == Captured::Unknown
            || row.provider.data_value == Captured::Unknown
            || row.guards.iter().any(|g| !g.predicate.supported());
        if !out.uses.contains(&row) {
            out.uses.push(row);
        }
    }
    out.stores = s.observed_stores.clone();
    for exit in &s.exits {
        out.incomplete |= exit.incomplete;
        for r in &exit.returns {
            out.incomplete |= r.source == ValueSource::Unknown;
            if !out.returns.contains(r) {
                out.returns.push(r.clone());
            }
        }
    }
    if s.partial {
        for result in &s.pointer_results {
            if !out
                .returns
                .iter()
                .any(|r| r.result == *result && r.source == ValueSource::Unknown)
            {
                out.returns.push(ReturnSource {
                    result: *result,
                    source: ValueSource::Unknown,
                    guards: Vec::new(),
                });
            }
        }
    }
    if out.uses.is_empty() {
        None
    } else {
        Some(out)
    }
}

fn candidates(bodies: &HashMap<ProcId, &LifetimeBody>) -> HashSet<ProcId> {
    let mut reverse: HashMap<ProcId, Vec<ProcId>> = HashMap::default();
    let mut found = HashSet::default();
    let mut steps = 0;
    for (id, body) in bodies {
        for e in body.blocks.iter().flat_map(|b| &b.events) {
            steps += 1;
            if steps > MAX_STEPS * 32 || jaic::interrupt::should_stop() {
                return HashSet::default();
            }
            match e {
                Event::IndirectCall {
                    allocator: true, ..
                } => {
                    found.insert(*id);
                }
                Event::Call {
                    callee, ..
                } => reverse.entry(*callee).or_default().push(*id),
                _ => {}
            }
        }
    }
    let mut work: Vec<_> = found.iter().copied().collect();
    while let Some(id) = work.pop() {
        if jaic::interrupt::should_stop() {
            return HashSet::default();
        }
        for caller in reverse.get(&id).into_iter().flatten() {
            if found.insert(*caller) {
                work.push(*caller);
            }
        }
    }
    found
}
