//! The global reset epoch belongs to returned context-owned bytes, not every bump allocator.
use super::{Event, HashMap, HashSet, LifetimeBody, LifetimeOperand};
use jaic::ir::Val;

pub(super) fn returns_context_storage(c: &super::Compiler, body: &LifetimeBody) -> bool {
    let contexts: HashSet<_> = body
        .blocks
        .iter()
        .flat_map(|b| &b.events)
        .filter_map(|e| {
            if let Event::ContextRead {
                value,
            } = e
            {
                Some(*value)
            } else {
                None
            }
        })
        .collect();
    let mut marks = HashMap::<Val, u8>::default();
    loop {
        let mut changed = false;
        for e in body.blocks.iter().flat_map(|b| &b.events) {
            let update = match e {
                Event::Field {
                    base,
                    result,
                    name,
                    indirect,
                    ..
                } => operand(*base)
                    .zip(operand(*result))
                    .and_then(|(base, result)| {
                        if name.as_str() == "temporary_storage" && contexts.contains(&base) {
                            Some((result, 4))
                        } else if name.as_str() == "data"
                            && marks
                                .get(&base)
                                .is_some_and(|m| m & 1 != 0 || *indirect && m & 4 != 0)
                        {
                            Some((result, 8))
                        } else {
                            None
                        }
                    }),
                Event::Load {
                    place,
                    value,
                    ..
                } => {
                    let mark = marks.get(place).copied().unwrap_or(0);
                    Some((
                        *value,
                        (mark & 3)
                            | u8::from(mark & 4 != 0)
                            | if mark & 8 != 0 {
                                2
                            } else {
                                0
                            },
                    ))
                }
                Event::Store {
                    place,
                    value,
                    ..
                } => Some((*place, marks.get(value).copied().unwrap_or(0) & 3)),
                Event::Copy {
                    value,
                    source,
                }
                | Event::Offset {
                    value,
                    source,
                    ..
                } => Some((*value, marks.get(source).copied().unwrap_or(0) & 3)),
                Event::View {
                    value,
                    source,
                    from,
                    to,
                    ..
                } if c.ty.types.is_pointer(*from) && c.ty.types.is_pointer(*to) => {
                    Some((*value, marks.get(source).copied().unwrap_or(0) & 3))
                }
                // No transfer through calls: copying bytes into a heap allocation breaks provenance.
                _ => None,
            };
            if let Some((value, mark)) = update {
                let old = marks.entry(value).or_default();
                if mark & !*old != 0 {
                    *old |= mark;
                    changed = true;
                }
            }
        }
        if !changed {
            break;
        }
    }
    body.blocks.iter().flat_map(|b| &b.events).any(
        |e| matches!(e, Event::Return{value,..} if marks.get(value).is_some_and(|m| m & 2 != 0)),
    )
}

fn operand(op: LifetimeOperand) -> Option<Val> {
    match op {
        LifetimeOperand::Place(v) | LifetimeOperand::Value(v) => Some(v),
        LifetimeOperand::Other => None,
    }
}
