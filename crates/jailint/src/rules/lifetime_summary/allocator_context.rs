//! Statement-ordered allocator selections, including saved values and scope restoration.
use super::allocators::Allocators;
use super::{
    Event, HashMap, LifetimeBody, LifetimeOperand, Places, ProcId, Projection, Root, Summary,
};
use jaic::{ir::Val, sema::Compiler};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::rules) enum Selection {
    Caller,
    Parameter(usize),
    Heap,
    Temporary,
    #[default]
    Unknown,
}

impl Selection {
    pub fn root(self) -> Option<Root> {
        Some(match self {
            Self::Caller => Root::CallerAllocator,
            Self::Parameter(n) => Root::AllocatorParameter(n),
            Self::Heap => Root::Heap,
            Self::Temporary => Root::Temporary,
            Self::Unknown => return None,
        })
    }
}

#[derive(Default)]
pub(super) struct Context {
    pub calls: HashMap<Val, Selection>,
    pub arguments: HashMap<(Val, usize), Selection>,
    pub changes_allocator: bool,
    pub indirect: HashMap<Val, Selection>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct State {
    values: HashMap<Val, Selection>,
    slots: HashMap<usize, Selection>,
    contexts: HashMap<usize, Selection>,
    current: Option<usize>,
}

impl State {
    fn merge(&mut self, other: &Self) -> bool {
        let before = self.clone();
        self.values
            .retain(|key, value| other.values.get(key) == Some(value));
        self.slots.retain(|k, v| other.slots.get(k) == Some(v));
        self.contexts
            .retain(|k, v| other.contexts.get(k) == Some(v));
        if self.current != other.current {
            self.current = None;
        }
        *self != before
    }
}

impl Context {
    pub fn build(
        c: &Compiler,
        body: &LifetimeBody,
        places: &mut Places,
        allocators: &Allocators,
        all: &HashMap<ProcId, Summary>,
    ) -> Self {
        let mut out = Self::default();
        let Some(allocator_ty) = allocators.ty else {
            return out;
        };
        let relevant = body.blocks.iter().flat_map(|b| &b.events).any(|e| matches!(e, Event::Field {result_ty:ty,..} | Event::Load {ty,..} | Event::Store {ty,..} | Event::Parameter{ty,..} if *ty == allocator_ty) || matches!(e, Event::Call {callee,..} if all.get(callee).is_some_and(|s|s.changes_allocator)));
        if !relevant {
            return out;
        }
        let context_ty = c.ty.context_type;
        let constants: HashMap<_, _> = body
            .globals
            .iter()
            .filter_map(|(v, g)| allocators.constants.get(g).copied().map(|s| (*v, s)))
            .collect();
        let mut states = vec![None; body.blocks.len()];
        if states.is_empty() {
            return out;
        }
        states[0] = Some(State {
            values: constants,
            ..State::default()
        });
        let mut queue = std::collections::VecDeque::from([0]);
        let mut queued = vec![false; body.blocks.len()];
        queued[0] = true;
        while let Some(block) = queue.pop_front() {
            queued[block] = false;
            let mut state = states[block].clone().unwrap();
            for event in &body.blocks[block].events {
                match event {
                    Event::Parameter {
                        formal,
                        incoming,
                        place,
                        ty,
                        ..
                    } if *ty == allocator_ty => {
                        let role = Selection::Parameter(*formal);
                        state.values.insert(*incoming, role);
                        state.slots.insert(places.place(*place, *ty), role);
                    }
                    Event::ContextSet {
                        value,
                    } => {
                        if let Some(ty) = context_ty {
                            let id = places.place(*value, ty);
                            if *value == Val(0) {
                                state.contexts.entry(id).or_insert(Selection::Caller);
                            }
                            state.current = Some(id);
                        }
                    }
                    Event::Field {
                        result,
                        result_ty,
                        name,
                        base_ty,
                        base,
                        ..
                    } if *result_ty == allocator_ty && Some(*base_ty) == context_ty => {
                        if let (Some(result), Some(base)) = (operand(*result), operand(*base)) {
                            let id = places.place(base, *base_ty);
                            let root = places.root_of(id);
                            let role = if name.as_str() == "allocator" {
                                state.contexts.get(&root).copied().unwrap_or_default()
                            } else {
                                Selection::Unknown
                            };
                            state.values.insert(result, role);
                        }
                    }
                    Event::Load {
                        place,
                        value,
                        ty,
                        ..
                    } if *ty == allocator_ty => {
                        let id = places.place(*place, *ty);
                        let ((_, _), path) = &places.paths[id];
                        let root = places.root_of(id);
                        let role = match path.as_slice() {
                            [Projection::Field(n)] if n.as_str() == "allocator" => {
                                state.contexts.get(&root).copied().unwrap_or_default()
                            }
                            _ => state
                                .slots
                                .get(&id)
                                .copied()
                                .or_else(|| state.values.get(place).copied())
                                .unwrap_or_default(),
                        };
                        state.values.insert(*value, role);
                    }
                    Event::Load {
                        place,
                        value,
                        ty,
                        ..
                    } => {
                        let id = places.place(*place, *ty);
                        let root = places.root_of(id);
                        if matches!(places.paths[id].1.as_slice(), [Projection::Field(n)] if n.as_str() == "proc")
                            && places.paths[root].0.1 == allocator_ty
                        {
                            let role = state
                                .slots
                                .get(&root)
                                .copied()
                                .or_else(|| state.values.get(&places.paths[root].0.0).copied())
                                .unwrap_or_default();
                            state.values.insert(*value, role);
                        }
                    }
                    Event::Store {
                        place,
                        value,
                        ty,
                        ..
                    } if *ty == allocator_ty => {
                        let id = places.place(*place, *ty);
                        let role = state.values.get(value).copied().unwrap_or_default();
                        state.slots.insert(id, role);
                        state.values.insert(*place, role);
                        if matches!(places.paths[id].1.as_slice(), [Projection::Field(n)] if n.as_str() == "allocator")
                        {
                            state.contexts.insert(places.root_of(id), role);
                        }
                    }
                    Event::Store {
                        place,
                        value,
                        ty,
                        ..
                    } if Some(*ty) == context_ty => {
                        let source = places.place(*value, *ty);
                        let target = places.place(*place, *ty);
                        let role = state.contexts.get(&source).copied().unwrap_or_default();
                        state.contexts.insert(target, role);
                    }
                    Event::Copy {
                        value,
                        source,
                    } => {
                        if let Some(role) = state.values.get(source).copied() {
                            state.values.insert(*value, role);
                        }
                    }
                    Event::Call {
                        args,
                        results,
                        expands: false,
                        callee,
                        ..
                    } => {
                        let role = state
                            .current
                            .and_then(|p| state.contexts.get(&p).copied())
                            .unwrap_or(Selection::Caller);
                        for &(_, v, _) in results {
                            join(&mut out.calls, v, role);
                            for arg in args.iter().filter(|a| a.ty == allocator_ty) {
                                let arg_role = operand(arg.operand)
                                    .and_then(|v| state.values.get(&v).copied())
                                    .unwrap_or_default();
                                join(&mut out.arguments, (v, arg.formal), arg_role);
                            }
                        }
                        if all.get(callee).is_some_and(|s| s.changes_allocator)
                            && let Some(p) = state.current
                        {
                            state.contexts.insert(p, Selection::Unknown);
                        }
                    }
                    Event::IndirectCall {
                        callee,
                        allocator: true,
                        ..
                    } => {
                        let role = state.values.get(callee).copied().unwrap_or_default();
                        join(&mut out.indirect, *callee, role);
                    }
                    Event::Return {
                        ..
                    } => {
                        if let Some(ty) = context_ty {
                            let entry = places.place(Val(0), ty);
                            out.changes_allocator |= c.pr.procs[body.procedure.0 as usize]
                                .sig
                                .as_ref()
                                .is_some_and(|s| s.has_context)
                                && state
                                    .contexts
                                    .get(&entry)
                                    .copied()
                                    .unwrap_or(Selection::Unknown)
                                    != Selection::Caller;
                        }
                    }
                    _ => {}
                }
            }
            for &next in &body.blocks[block].successors {
                let changed = if let Some(old) = &mut states[next] {
                    old.merge(&state)
                } else {
                    states[next] = Some(state.clone());
                    true
                };
                if changed && !queued[next] {
                    queue.push_back(next);
                    queued[next] = true;
                }
            }
        }
        out
    }
}

fn operand(op: LifetimeOperand) -> Option<Val> {
    match op {
        LifetimeOperand::Value(v) | LifetimeOperand::Place(v) => Some(v),
        LifetimeOperand::Other => None,
    }
}

fn join<K: Eq + std::hash::Hash>(map: &mut HashMap<K, Selection>, key: K, role: Selection) {
    map.entry(key)
        .and_modify(|old| {
            if *old != role {
                *old = Selection::Unknown;
            }
        })
        .or_insert(role);
}

impl super::Resolver<'_> {
    pub(super) fn refresh_context(&mut self, all: &HashMap<ProcId, Summary>) {
        self.field_origins.borrow_mut().clear();
        self.may_field_origins.borrow_mut().clear();
        self.context = Context::build(
            self.compiler,
            self.body,
            &mut self.places,
            self.allocators,
            all,
        );
    }

    pub(super) fn allocation_root(
        &self,
        root: &Root,
        result: Val,
        args: &[super::LifetimeArgument],
    ) -> Option<Root> {
        match root {
            Root::CallerAllocator => self
                .context
                .calls
                .get(&result)
                .copied()
                .unwrap_or(Selection::Caller)
                .root(),
            Root::AllocatorParameter(n) => self
                .context
                .arguments
                .get(&(result, *n))
                .copied()
                .unwrap_or_else(|| {
                    args.iter()
                        .find(|a| a.formal == *n)
                        .and_then(|a| operand(a.operand))
                        .and_then(|v| self.context.indirect.get(&v).copied())
                        .unwrap_or_default()
                })
                .root(),
            _ => Some(root.clone()),
        }
    }
}
