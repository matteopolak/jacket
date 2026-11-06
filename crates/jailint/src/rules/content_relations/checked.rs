#[path = "calls.rs"]
mod calls;
#[path = "events.rs"]
mod events;

use super::bindings::{Bindings, bits, literal};
use super::domain::*;
use super::*;
use jaic::fxhash::HashSet;
use jaic::ide::lifetimes::{LifetimeBody, LifetimeBoolean, LifetimeEvent as Event};
use jaic::ir::{BinOp, CmpOp, Ty};
use jaic::sema::procs::BodyState;

pub(super) struct Solver<'a> {
    c: &'a Compiler,
    layout: Layout,
    bodies: HashMap<ProcId, &'a LifetimeBody>,
    cache: HashMap<ProcId, Option<Summary>>,
    active: HashSet<ProcId>,
}

impl<'a> Solver<'a> {
    pub fn new(c: &'a Compiler, layout: Layout) -> Self {
        let bodies =
            c.ed.facts()
                .into_iter()
                .flat_map(|f| &f.lifetime_bodies)
                .filter(|(id, b)| {
                    if jaic::interrupt::should_stop() {
                        return false;
                    }
                    let p = &c.pr.procs[id.0 as usize];
                    b.procedure == **id
                        && p.body_state == BodyState::Done
                        && std::sync::Arc::ptr_eq(&p.lit, &b.syntax)
                })
                .map(|(id, b)| (*id, b))
                .collect();
        Self {
            c,
            layout,
            bodies,
            cache: HashMap::default(),
            active: HashSet::default(),
        }
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

    fn initial(&self, b: &LifetimeBody) -> Option<State> {
        if b.blocks.is_empty() {
            return None;
        }
        let mut initial = State::default();
        if b.stack_roots.len() > 8192 || b.slot_aliases.len() > 8192 {
            return None;
        }
        for root in &b.stack_roots {
            if jaic::interrupt::should_stop() {
                return None;
            }
            initial.owned.insert(root.value);
            initial.allocation_extents.insert(root.value, root.bytes);
            initial.addresses.insert(
                root.value,
                Address {
                    root: root.value,
                    offset: 0,
                },
            );
            if let Some(bytes) = root.bytes {
                initial.roots.insert(root.value, bytes);
            }
        }
        for (alias, root) in &b.slot_aliases {
            initial.addresses.insert(
                *alias,
                Address {
                    root: *root,
                    offset: 0,
                },
            );
        }
        for (initialization_step, e) in b.blocks.iter().flat_map(|b| &b.events).enumerate() {
            if initialization_step >= 8192 || jaic::interrupt::should_stop() {
                return None;
            }
            let root = match e {
                Event::Local {
                    place,
                    ty,
                    ..
                }
                | Event::Initialize {
                    place,
                    ty,
                    ..
                }
                | Event::Parameter {
                    place,
                    ty,
                    ..
                } => Some((*place, *ty)),
                Event::Return {
                    value,
                    ty: TypeId::STRING,
                    ..
                } => Some((*value, TypeId::STRING)),
                _ => None,
            };
            if let Some((v, ty)) = root {
                if matches!(e, Event::Local { .. } | Event::Initialize { .. })
                    && !initial.addresses.contains_key(&v)
                {
                    continue;
                }
                initial.root(v, self.c.ty.types.size_of(ty), false);
                if ty == TypeId::STRING {
                    initial.headers.insert(initial.address(v)?);
                }
            }
            if let Event::Call {
                results, ..
            }
            | Event::IndirectCall {
                results, ..
            } = e
            {
                for (_, v, ty) in results {
                    if *ty == TypeId::STRING {
                        initial.root(*v, jaic::layout::view::SIZE, initial.owned.contains(v));
                        initial.headers.insert(initial.address(*v)?);
                    }
                }
            }
        }
        if !initial.bounded() {
            return None;
        }
        Some(initial)
    }

    fn body(&mut self, b: &LifetimeBody) -> Option<Summary> {
        let initial = self.initial(b)?;
        let mut work = vec![(0, initial)];
        let mut summary = Summary::default();
        let mut steps = 0;
        while let Some((block, mut state)) = work.pop() {
            if jaic::interrupt::should_stop() || !state.visited.insert(block) {
                return None;
            }
            let bb = b.blocks.get(block)?;
            let mut states = vec![state];
            let mut branch = None;
            for (event, e) in bb.events.iter().enumerate() {
                if jaic::interrupt::should_stop() {
                    return None;
                }
                steps += states.len();
                if steps > 8192 {
                    return None;
                }
                let anchor = EventSite {
                    procedure: b.procedure,
                    block,
                    event,
                };
                let site = Site {
                    chain: vec![anchor],
                };
                let mut next = Vec::new();
                for mut state in states {
                    comparisons(b, &mut state);
                    if matches!(
                        e,
                        Event::Call {
                            expands: false,
                            ..
                        }
                    ) {
                        match self.call(b, e, anchor, &site, &mut state, &mut next)? {
                            calls::Next::Continue => {}
                            calls::Next::FinishState => {
                                next.push(state);
                                continue;
                            }
                            calls::Next::Forked => continue,
                        }
                    } else if self.event(b, e, &site, &mut state, &mut branch)? {
                        next.push(state);
                        continue;
                    }
                    comparisons(b, &mut state);
                    if !state.bounded() {
                        return None;
                    }
                    next.push(state);
                }
                if next.len() + work.len() > MAX_ITEMS {
                    return None;
                }
                states = next;
            }
            for state in states {
                if bb.normal_return {
                    let exit = Exit {
                        returns: state.returns,
                        copies: state.copies,
                    };
                    if !summary.exits.contains(&exit) {
                        summary.exits.push(exit);
                    }
                } else if let Some((condition, yes, no)) = branch {
                    let known = state.value(condition).truth();
                    for (target, truth) in [(yes, true), (no, false)] {
                        if known.is_none_or(|k| k == truth) {
                            work.push((target, state.clone()));
                        }
                    }
                } else {
                    for next in &bb.successors {
                        work.push((*next, state.clone()));
                    }
                }
            }
            if work.len() + summary.exits.len() > MAX_ITEMS {
                return None;
            }
        }
        Some(summary)
    }

    fn results(&self, state: &mut State, results: &[(usize, Val, TypeId)], site: &Site) {
        for (index, value, ty) in results {
            if self.c.ty.types.pointee(*ty).is_some() {
                state.values.insert(
                    *value,
                    Value::Pointer(Pointer {
                        base: Base::Call(site.clone(), *index),
                        offset: 0,
                    }),
                );
            } else if *ty == TypeId::STRING {
                state.descriptor_at(*value, &Descriptor::default(), self.layout);
            } else {
                state.values.insert(*value, Value::Unknown);
            }
        }
    }
}

fn selected_copy(
    memory: Option<&jaic::ide::lifetimes::LifetimeMemoryEffects>,
) -> Option<(usize, usize, usize, bool)> {
    let copy = memory?.copy()?;
    Some((
        copy.destination_formal as usize,
        copy.source_formal as usize,
        copy.count_formal as usize,
        copy.requires_disjoint,
    ))
}

fn arithmetic(c: &Compiler, ty: TypeId, left: &Value, op: BinOp, right: &Value) -> Value {
    let (Value::Scalar(a), Value::Scalar(b)) = (left, right) else {
        return Value::Unknown;
    };
    if *b == Scalar::Constant(0) && matches!(op, BinOp::Add | BinOp::Sub) {
        return left.clone();
    }
    let (Scalar::Constant(a), Scalar::Constant(b)) = (a, b) else {
        return Value::Unknown;
    };
    let Some((width, signed)) = c.ty.types.int_info(ty) else {
        return Value::Unknown;
    };
    if width == 0 || width > 64 {
        return Value::Unknown;
    }
    let (Some(a), Some(b)) = (
        bits(*a as u64, width, signed),
        bits(*b as u64, width, signed),
    ) else {
        return Value::Unknown;
    };
    let result = match op {
        BinOp::Add => a.checked_add(b),
        BinOp::Sub => a.checked_sub(b),
        _ => None,
    };
    let (min, max) = if signed {
        (-(1_i128 << (width - 1)), (1_i128 << (width - 1)) - 1)
    } else {
        (0, (1_i128 << width) - 1)
    };
    result
        .filter(|k| *k >= min && *k <= max)
        .map_or(Value::Unknown, |k| Value::Scalar(Scalar::Constant(k)))
}

fn comparisons(body: &LifetimeBody, state: &mut State) {
    for c in &body.comparisons {
        let width = match c.ty {
            Ty::I8 => 8,
            Ty::I16 => 16,
            Ty::I32 => 32,
            Ty::I64 => 64,
            _ => continue,
        };
        let signed = matches!(c.op, CmpOp::SLt | CmpOp::SLe | CmpOp::SGt | CmpOp::SGe);
        let normalize = |value: Value| {
            if let Value::Scalar(Scalar::Constant(k)) = value {
                bits(k as u64, width, signed)
                    .map_or(Value::Unknown, |k| Value::Scalar(Scalar::Constant(k)))
            } else {
                value
            }
        };
        let left = normalize(c.left_constant.map_or_else(
            || state.value(c.left),
            |k| Value::Scalar(Scalar::Constant(k as i128)),
        ));
        let right = normalize(c.right_constant.map_or_else(
            || state.value(c.right),
            |k| Value::Scalar(Scalar::Constant(k as i128)),
        ));
        if left != Value::Unknown || right != Value::Unknown {
            state
                .values
                .insert(c.value, Value::compare(&left, c.op, &right));
        }
    }
}
