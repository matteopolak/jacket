use super::domain::{Number, Place, State, Value, number, truth};
use super::*;
use jaic::ide::lifetimes::{LifetimeBody, LifetimeBoolean, LifetimeEvent as Event};
use jaic::ir::BinOp;
use jaic::types::{ArrayKind, TypeKind};

const MAX_STEPS: usize = 8192;
const MAX_ALTERNATIVES: usize = 128;

/// Prove the exact old-last-to-selected map under explicit index/separation requirements.
/// For example, old identities `[A,B,C]` with index zero become `[C,B]`.
pub(super) fn infer(
    c: &Compiler,
    b: &LifetimeBody,
    receiver: usize,
    index: usize,
    element: TypeId,
) -> Option<UnorderedRemove> {
    let roots = Rc::new(StackRoots::capture(b, MAX_STEPS)?);
    let initial = State {
        addresses: roots.addresses(),
        roots: Some(roots),
        ..State::default()
    };
    let mut work = vec![(0, initial)];
    let mut steps = 0;
    let mut exits = 0;
    let mut evidence = Vec::new();
    while let Some((block, mut state)) = work.pop() {
        if jaic::interrupt::should_stop() || !state.visited.insert(block) {
            return None;
        }
        let bb = b.blocks.get(block)?;
        let mut branch = None;
        for (event_index, event) in bb.events.iter().enumerate() {
            steps += 1;
            if steps > MAX_STEPS || jaic::interrupt::should_stop() {
                return None;
            }
            comparisons(b, &mut state);
            if computed_value(c, &mut state, event)? {
                comparisons(b, &mut state);
                continue;
            }
            match event {
                Event::Parameter {
                    formal,
                    place,
                    incoming,
                    ty,
                    ..
                } => {
                    // Only the retained physical root certificate authorizes local storage.
                    let value = if *formal == receiver {
                        Value::Pointer(Place::Descriptor(receiver, c.ty.types.pointee(*ty)?))
                    } else if c.ty.types.int_info(*ty) == Some((64, true)) {
                        Value::Number(Number::Formal(*formal))
                    } else {
                        Value::Unknown
                    };
                    state.values.insert(*incoming, value.clone());
                    if matches!(state.place(c, *place, *ty), Place::Local(_)) {
                        state.slots.insert(state.local(*place), value);
                    }
                }
                Event::Local {
                    ..
                } => {}
                Event::FieldProjection {
                    base,
                    result,
                    base_ty,
                    result_ty,
                    offset,
                } => {
                    state.project(
                        *result,
                        *base,
                        *offset as i128,
                        c.ty.types.size_of(*result_ty),
                    );
                    if let Value::Pointer(Place::Descriptor(formal, descriptor)) =
                        state.value(*base)
                    {
                        if c.ty.types.repr(*base_ty) != c.ty.types.repr(descriptor) {
                            return None;
                        }
                        let TypeKind::Array {
                            elem,
                            kind: ArrayKind::View | ArrayKind::Resizable,
                        } = *c.ty.types.kind(c.ty.types.repr(descriptor))
                        else {
                            return None;
                        };
                        let place =
                            if *offset == jaic::layout::view::COUNT && *result_ty == TypeId::S64 {
                                Place::Count(formal)
                            } else if *offset == jaic::layout::view::DATA
                                && c.ty.types.pointee(*result_ty) == Some(elem)
                            {
                                Place::Data(formal, elem)
                            } else {
                                Place::Unknown
                            };
                        state.places.insert((*result, *result_ty), place);
                    }
                }
                Event::Copy {
                    value,
                    source,
                } => {
                    state.values.insert(*value, state.value(*source));
                    if let Some(a) = state.addresses.get(&state.local(*source)).copied() {
                        state.addresses.insert(*value, a);
                    }
                }
                Event::Load {
                    place,
                    value,
                    ty,
                    ..
                } => {
                    let loaded = match state.place(c, *place, *ty) {
                        Place::Local(local) => {
                            state.slots.get(&local).cloned().unwrap_or(Value::Unknown)
                        }
                        Place::Count(formal) if formal == receiver => {
                            Value::Number(Number::Count(receiver, state.count_delta))
                        }
                        Place::Data(formal, elem) => Value::Pointer(Place::Buffer(formal, elem)),
                        Place::Element(formal, position, elem)
                            if state.moved.is_none() && elem == *ty =>
                        {
                            Value::Payload(formal, position, elem)
                        }
                        _ => Value::Unknown,
                    };
                    state.values.insert(*value, loaded);
                }
                Event::Store {
                    place,
                    value,
                    ty,
                    constant,
                    ..
                } => {
                    let source = constant.map_or_else(
                        || state.operand(c, *value, *ty),
                        |k| Value::Number(literal(c, *ty, k)),
                    );
                    match state.place(c, *place, *ty) {
                        Place::Local(local) => {
                            state.write_local(local, source);
                        }
                        Place::Count(formal)
                            if formal == receiver
                                && state.count_writes == 0
                                && number(&source) == Number::Count(receiver, -1) =>
                        {
                            state.count_writes = 1;
                            state.count_site = Some(Site {
                                procedure: b.procedure,
                                block,
                                event: event_index,
                            });
                            state.count_delta = -1;
                        }
                        Place::Element(formal, Number::Formal(selected), elem)
                            if formal == receiver
                                && selected == index
                                && elem == element
                                && *ty == element
                                && state.count_writes == 1
                                && state.moved.is_none() =>
                        {
                            if source
                                != Value::Payload(receiver, Number::Count(receiver, -1), element)
                            {
                                return None;
                            }
                            state.moved = Some(index);
                            state.move_site = Some(Site {
                                procedure: b.procedure,
                                block,
                                event: event_index,
                            });
                        }
                        _ => return None,
                    }
                }
                Event::Initialize {
                    place,
                    ty,
                    ..
                } => {
                    let Place::Local(local) = state.place(c, *place, *ty) else {
                        return None;
                    };
                    state.write_local(local, Value::Unknown);
                }
                Event::Branch {
                    condition,
                    then_block,
                    else_block,
                    ..
                } => {
                    branch = Some((*condition, *then_block, *else_block));
                }
                Event::Call {
                    expands: true, ..
                } => {}
                Event::Call {
                    ..
                }
                | Event::IndirectCall {
                    ..
                }
                | Event::Barrier => return None,
                _ => {}
            }
            comparisons(b, &mut state);
        }
        if bb.normal_return {
            if state.count_writes != 1 || state.count_delta != -1 || state.moved != Some(index) {
                return None;
            }
            exits += 1;
            let witness = Evidence {
                count_write: state.count_site?,
                element_write: state.move_site?,
            };
            if !evidence.contains(&witness) {
                evidence.push(witness);
            }
        } else if let Some((condition, yes, no)) = branch {
            let known = truth(&state.value(condition), receiver, index);
            for (next, expected) in [(yes, true), (no, false)] {
                if known.is_none_or(|v| v == expected) {
                    work.push((next, state.clone()));
                }
            }
        } else {
            for next in &bb.successors {
                work.push((*next, state.clone()));
            }
        }
        if work.len() > MAX_ALTERNATIVES {
            return None;
        }
    }
    (exits > 0).then_some(UnorderedRemove {
        receiver_formal: receiver,
        index_formal: index,
        element,
        preconditions: Preconditions {
            valid_incoming_index: true,
            descriptor_disjoint_from_storage: true,
        },
        count_delta: -1,
        evidence,
    })
}

fn comparisons(body: &LifetimeBody, state: &mut State) {
    for comparison in &body.comparisons {
        let left = comparison.left_constant.map_or_else(
            || state.value(comparison.left),
            |k| Value::Number(Number::Constant(k as i128)),
        );
        let right = comparison.right_constant.map_or_else(
            || state.value(comparison.right),
            |k| Value::Number(Number::Constant(k as i128)),
        );
        let normalize = |value: Value| match value {
            Value::Number(Number::Constant(k)) => {
                comparison_literal(comparison.ty, comparison.op, k as u64)
                    .map_or(Value::Unknown, |k| Value::Number(Number::Constant(k)))
            }
            other => other,
        };
        let (left, right) = (normalize(left), normalize(right));
        if left != Value::Unknown || right != Value::Unknown {
            state.values.insert(
                comparison.value,
                Value::compare(&left, comparison.op, &right),
            );
        }
    }
}

fn bits(value: u64, width: u8, signed: bool) -> Option<i128> {
    if width == 0 || width > 64 {
        return None;
    }
    let mask = if width == 64 {
        u64::MAX
    } else {
        (1_u64 << width) - 1
    };
    let raw = value & mask;
    Some(if signed && raw & (1_u64 << (width - 1)) != 0 {
        raw as i128 - (1_i128 << width)
    } else {
        raw as i128
    })
}

fn literal(c: &Compiler, ty: TypeId, value: u64) -> Number {
    if ty == TypeId::BOOL {
        return if value <= 1 {
            Number::Constant(value as i128)
        } else {
            Number::Unknown
        };
    }
    c.ty.types
        .int_info(ty)
        .and_then(|(width, signed)| bits(value, width, signed))
        .map_or(Number::Unknown, Number::Constant)
}

fn comparison_literal(ty: jaic::ir::Ty, op: jaic::ir::CmpOp, value: u64) -> Option<i128> {
    use jaic::ir::{CmpOp, Ty};
    let width = match ty {
        Ty::I8 => 8,
        Ty::I16 => 16,
        Ty::I32 => 32,
        Ty::I64 => 64,
        _ => return None,
    };
    bits(
        value,
        width,
        matches!(op, CmpOp::SLt | CmpOp::SLe | CmpOp::SGt | CmpOp::SGe),
    )
}

fn arithmetic(c: &Compiler, ty: TypeId, left: Number, op: BinOp, right: Number) -> Number {
    let Some((width, signed)) = c.ty.types.int_info(ty) else {
        return Number::Unknown;
    };
    if width == 0 || width > 64 {
        return Number::Unknown;
    }
    let (min, max) = if signed {
        (-(1_i128 << (width - 1)), (1_i128 << (width - 1)) - 1)
    } else {
        (0, (1_i128 << width) - 1)
    };
    match (left, op, right) {
        (Number::Constant(a), BinOp::Add | BinOp::Sub, Number::Constant(b)) => {
            let result = if op == BinOp::Add {
                a.checked_add(b)
            } else {
                a.checked_sub(b)
            };
            result
                .filter(|n| *n >= min && *n <= max)
                .map_or(Number::Unknown, Number::Constant)
        }
        (Number::Count(receiver, delta), BinOp::Add | BinOp::Sub, Number::Constant(k))
            if width == 64 && signed =>
        {
            let result = if op == BinOp::Add {
                delta.checked_add(k)
            } else {
                delta.checked_sub(k)
            };
            // The query requires incoming count in 1..=s64::MAX. Preserve an affine
            // relation only if both its input and output ranges avoid machine overflow.
            let bounded = |d: i128| {
                1_i128.checked_add(d).is_some_and(|n| n >= min)
                    && max.checked_add(d).is_some_and(|n| n <= max)
            };
            if !bounded(delta) {
                return Number::Unknown;
            }
            result
                .filter(|d| bounded(*d))
                .map_or(Number::Unknown, |d| Number::Count(receiver, d))
        }
        _ => Number::Unknown,
    }
}

/// Evaluate checked arithmetic/address/Boolean facts without changing CFG sequencing.
fn computed_value(c: &Compiler, state: &mut State, event: &Event) -> Option<bool> {
    match event {
        Event::IntegerBinary {
            value,
            left,
            right,
            op,
            left_constant,
            right_constant,
            ty,
        } => {
            let left =
                left_constant.map_or_else(|| number(&state.value(*left)), |k| literal(c, *ty, k));
            let right =
                right_constant.map_or_else(|| number(&state.value(*right)), |k| literal(c, *ty, k));
            let normalize = |n| {
                if let Number::Constant(k) = n {
                    literal(c, *ty, k as u64)
                } else {
                    n
                }
            };
            let result = arithmetic(c, *ty, normalize(left), *op, normalize(right));
            state.values.insert(*value, Value::Number(result));
        }
        Event::Offset {
            value,
            source,
            stride,
            offset,
            subscript,
            ..
        } => {
            let position = if let Some(subscript) = subscript {
                number(&state.value(*subscript))
            } else {
                offset.map_or(Number::Unknown, Number::Constant)
            };
            let constant = if let Number::Constant(k) = position {
                Some(k)
            } else {
                None
            };
            if value != source {
                state.addresses.remove(value);
            }
            if let Some(bytes) =
                constant.and_then(|n| n.checked_mul(c.ty.types.size_of(*stride) as i128))
            {
                state.project(*value, *source, bytes, 1);
            }
            if let Value::Pointer(Place::Buffer(formal, elem)) = state.value(*source) {
                if elem != *stride || position == Number::Unknown {
                    return None;
                }
                state.values.insert(
                    *value,
                    Value::Pointer(Place::Element(formal, position, elem)),
                );
            }
        }
        Event::Boolean {
            value,
            operation,
        } => {
            let expression = match operation {
                LifetimeBoolean::Not(source) => Value::not(state.reference(*source)),
                LifetimeBoolean::And(left, right) => {
                    Value::and(state.reference(*left), state.reference(*right))
                }
                LifetimeBoolean::Or(left, right) => {
                    Value::or(state.reference(*left), state.reference(*right))
                }
            };
            state.values.insert(*value, expression);
        }
        _ => return Some(false),
    }
    Some(true)
}
