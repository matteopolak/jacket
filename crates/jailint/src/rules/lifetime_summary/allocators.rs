//! Allocator identities come from the checked protocol and capability-query body.
use super::{Compiler, Event, HashMap, HashSet, LifetimeBody, ProcId};
use jaic::intern::Sym;
use jaic::ir::{CmpOp, FuncId, Val};
use jaic::sema::{
    Value,
    procs::ProcTarget,
    scope::{EntityState, Resolved},
};
use jaic::types::TypeId;

#[derive(Default)]
pub(in crate::rules) struct Allocators {
    pub constants: HashMap<jaic::ir::GlobalId, super::allocator_context::Selection>,
    pub ty: Option<TypeId>,
    pub temporary: HashSet<ProcId>,
    pub heap_functions: HashSet<FuncId>,
    pub temporary_functions: HashSet<FuncId>,
}

impl Allocators {
    pub fn build(c: &Compiler, valid: &HashMap<ProcId, &LifetimeBody>) -> Self {
        let Some((ty, signature)) = allocator_types(c) else {
            return Self::default();
        };
        let mut result = Self {
            ty: Some(ty),
            ..Self::default()
        };
        for (&id, body) in valid {
            if c.pr.procs[id.0 as usize]
                .sig
                .as_ref()
                .is_none_or(|s| s.ty != signature)
            {
                continue;
            }
            let Some(bits) = capabilities(body) else {
                continue;
            };
            let context_storage = bits & 0x0600_0000 != 0
                && super::allocator_storage::returns_context_storage(c, body);
            if bits & 0x0200_0000 != 0
                && !context_storage
                && let Some(ProcTarget::Func(f)) = c.pr.procs[id.0 as usize].target
            {
                result.heap_functions.insert(f);
            }
            // The capability bit alone also describes independent pools. Only the
            // runtime context's temporary-storage provider has the global reset epoch.
            if bits & 0x0400_0000 != 0 && context_storage {
                result.temporary.insert(id);
                if let Some(ProcTarget::Func(f)) = c.pr.procs[id.0 as usize].target {
                    result.temporary_functions.insert(f);
                }
            }
        }
        if let Some(facts) = c.ed.facts() {
            for (&id, relocs) in &facts.lifetime_globals {
                if c.out.program.globals[id.0 as usize].size != c.ty.types.size_of(ty) {
                    continue;
                }

                if relocs.iter().any(|r| r.offset == 0 && r.addend == 0 && matches!(r.target, jaic::ir::RelocTarget::Func(f) if result.heap_functions.contains(&f))) {
                    result.constants.insert(id,super::allocator_context::Selection::Heap);
                }

                if relocs.iter().any(|r| r.offset == 0 && r.addend == 0 && matches!(r.target, jaic::ir::RelocTarget::Func(f) if result.temporary_functions.contains(&f))) {
                    result.constants.insert(id,super::allocator_context::Selection::Temporary);
                }
            }
        }
        result
    }
}

fn allocator_types(c: &Compiler) -> Option<(TypeId, TypeId)> {
    let module = c.ns.preload?;
    let scope = c.ns.modules[module.0 as usize].scope;
    for entity in c.scope(scope).names.get(&Sym::intern("Allocator"))? {
        if let EntityState::Done(Resolved::Const {
            value: Value::Type(ty),
            ..
        }) = c.entity(*entity).state
        {
            let st = c.ty.types.as_struct(ty)?;
            let field =
                c.ty.types
                    .struct_info(st)
                    .fields
                    .iter()
                    .find(|f| f.name == Some(Sym::intern("proc")))?;
            return Some((ty, field.ty));
        }
    }
    None
}

/// Statically evaluate CAPS with null optional output/data, never execute user code.
/// Unknown calls, comparisons or loops make the query unsupported, not trusted.
fn capabilities(body: &LifetimeBody) -> Option<u64> {
    static_result(
        body,
        &HashMap::from_iter([(0, 10), (1, 0), (2, 0), (3, 0), (4, 0)]),
    )
}

pub(super) fn nonallocating_call(body: &LifetimeBody, args: &[super::LifetimeArgument]) -> bool {
    let constants = args
        .iter()
        .filter_map(|a| a.constant.map(|v| (a.formal, v)))
        .collect();
    static_result(body, &constants).is_some()
}

fn static_result(body: &LifetimeBody, arguments: &HashMap<usize, u64>) -> Option<u64> {
    let stack: HashSet<_> = body.stack_places.iter().copied().collect();
    let mut values = HashMap::default();
    for event in body.blocks.iter().flat_map(|b| &b.events) {
        if let Event::Parameter {
            formal,
            incoming,
            ..
        } = event
            && let Some(value) = arguments.get(formal)
        {
            values.insert(*incoming, *value);
        }
    }
    let mut queue = std::collections::VecDeque::from([(0, values)]);
    let mut answer = None;
    let mut visits = 0;
    while let Some((block, mut values)) = queue.pop_front() {
        visits += 1;
        if visits > 128 {
            return None;
        }
        let block = &body.blocks[block];
        let mut successors = block.successors.clone();
        for event in &block.events {
            match event {
                Event::Initialize {
                    place, ..
                } => {
                    if !stack.contains(place) {
                        return None;
                    }
                    values.insert(*place, 0);
                }
                Event::Store {
                    place,
                    value,
                    constant,
                    ..
                } => {
                    if !stack.contains(place) {
                        return None;
                    }
                    let source = constant.or_else(|| scalar(body, *value, &values, 0));
                    assign(&mut values, *place, source);
                }
                Event::Load {
                    place,
                    value,
                    ..
                }
                | Event::Copy {
                    source: place,
                    value,
                } => {
                    let source = scalar(body, *place, &values, 0);
                    assign(&mut values, *value, source);
                }
                Event::IntegerBinary {
                    value,
                    left,
                    right,
                    op,
                    left_constant,
                    right_constant,
                } => {
                    let a = left_constant.or_else(|| scalar(body, *left, &values, 0));
                    let b = right_constant.or_else(|| scalar(body, *right, &values, 0));
                    let result = a.zip(b).and_then(|(a, b)| match op {
                        jaic::ir::BinOp::And => Some(a & b),
                        jaic::ir::BinOp::Or => Some(a | b),
                        jaic::ir::BinOp::Xor => Some(a ^ b),
                        _ => None,
                    });
                    assign(&mut values, *value, result);
                }
                Event::Branch {
                    condition,
                    then_block,
                    else_block,
                    ..
                } => {
                    if let Some(value) = scalar(body, *condition, &values, 0) {
                        let selected = if value == 0 {
                            else_block
                        } else {
                            then_block
                        };
                        successors.retain(|to| to == selected);
                    }
                }
                Event::Return {
                    value,
                    constant,
                    index: 0,
                    ..
                } => {
                    let flags = constant.or_else(|| scalar(body, *value, &values, 0))?;
                    if answer.is_some_and(|previous| previous != flags) {
                        return None;
                    }
                    answer = Some(flags);
                }
                Event::Call {
                    expands: false, ..
                }
                | Event::IndirectCall {
                    ..
                }
                | Event::Barrier => return None,
                _ => {}
            }
        }
        queue.extend(successors.into_iter().map(|to| (to, values.clone())));
    }
    answer
}

fn assign(values: &mut HashMap<Val, u64>, target: Val, value: Option<u64>) {
    if let Some(value) = value {
        values.insert(target, value);
    } else {
        values.remove(&target);
    }
}

fn scalar(
    body: &LifetimeBody,
    value: Val,
    values: &HashMap<Val, u64>,
    depth: usize,
) -> Option<u64> {
    if depth > 16 {
        return None;
    }
    if let Some(&value) = values.get(&value) {
        return Some(value);
    }
    let comparison = body.comparisons.iter().find(|c| c.value == value)?;
    let a = comparison
        .left_constant
        .or_else(|| scalar(body, comparison.left, values, depth + 1))?;
    let b = comparison
        .right_constant
        .or_else(|| scalar(body, comparison.right, values, depth + 1))?;
    Some(u64::from(match comparison.op {
        CmpOp::Eq => a == b,
        CmpOp::Ne => a != b,
        _ => return None,
    }))
}
