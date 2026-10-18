//! Reading the records handed to Jai and building them into program memory.

use super::*;

pub(super) fn call(cx: &Call, op: MetaOp, interp: &mut Interp) -> Result<Rets, Trap> {
    let shared = cx.shared;
    let arg = |i: usize| cx.arg(i);
    let text = |interp: &Interp, i: usize| cx.text(interp, i);
    match op {
        MetaOp::SyntaxInt | MetaOp::SyntaxString | MetaOp::SyntaxChild => {
            let key = SyntaxKey::from_raw(arg(1))
                .ok_or_else(|| Trap::new("internal: unknown syntax key"))?;
            let mut reg = shared.borrow_mut();
            // Association lookup only: never dereference arbitrary program pointers.
            let found = reg
                .builders
                .iter()
                .enumerate()
                .find_map(|(index, builder)| {
                    let id = builder.record_of(arg(0));
                    (id != 0).then_some((index, id))
                });
            match op {
                MetaOp::SyntaxInt => Ok(Rets::one(
                    found
                        .and_then(|(_, id)| match reg.records.syntax_field(id, key) {
                            Some(crate::records::FieldRef::Int(value)) => Some(value as u64),
                            _ => None,
                        })
                        .unwrap_or(0),
                )),
                MetaOp::SyntaxString => {
                    let value = found
                        .and_then(|(_, id)| match reg.records.syntax_field(id, key) {
                            Some(crate::records::FieldRef::Str(value)) => Some(value.clone()),
                            _ => None,
                        })
                        .unwrap_or_default();
                    store_pair(interp, arg(2), value.len() as u64, value.as_ptr() as u64)?;
                    reg.kept.push(value);
                    Ok(Rets::default())
                }
                MetaOp::SyntaxChild => {
                    let Some((index, id)) = found else {
                        return Ok(Rets::one(0));
                    };
                    let child = match reg.records.syntax_field(id, key) {
                        Some(crate::records::FieldRef::Ref(child)) => child,
                        _ => 0,
                    };
                    let Workspaces {
                        records,
                        kept,
                        builders,
                        ..
                    } = &mut *reg;
                    Ok(Rets::one(builders[index].build(
                        records,
                        kept,
                        interp,
                        child,
                        arg(2),
                    )?))
                }
                _ => unreachable!(),
            }
        }
        MetaOp::RecTag => {
            // Tags are static: hand out the text itself.
            let tag = shared
                .borrow()
                .records
                .get(arg(0) as i64)
                .map_or("", |r| r.tag.name());
            store_pair(interp, arg(1), tag.len() as u64, tag.as_ptr() as u64)?;
            Ok(Rets::default())
        }
        MetaOp::RecTagId => {
            // A small number per distinct tag, so Jai can cache tag -> struct type in an array.
            let mut reg = shared.borrow_mut();
            let Some(tag) = reg.records.get(arg(0) as i64).map(|r| r.tag) else {
                return Ok(Rets::one(u64::MAX));
            };
            let next = reg.tags.len();
            let id = *reg.tags.entry(tag).or_insert(next);
            Ok(Rets::one(id as u64))
        }
        MetaOp::RecBuilder => {
            let mut reg = shared.borrow_mut();
            reg.builders.push(crate::record_build::Builder::default());
            Ok(Rets::one(reg.builders.len() as u64))
        }
        MetaOp::RecType | MetaOp::RecBuiltin => {
            let name = text(interp, 1)?;
            let mut reg = shared.borrow_mut();
            let builder = builder(&mut reg, arg(0))?;
            if op == MetaOp::RecType {
                builder.add_type(name, arg(2));
            } else {
                builder.add_builtin(name, arg(2));
            }
            Ok(Rets::default())
        }
        MetaOp::RecPlan => {
            let mut reg = shared.borrow_mut();
            let builder = builder(&mut reg, arg(0))?;
            builder.add_plan(interp, arg(1), arg(2), arg(3), arg(4), arg(5) & 1 != 0)?;
            Ok(Rets::default())
        }
        MetaOp::RecBuild => {
            let mut reg = shared.borrow_mut();
            let Workspaces {
                records,
                kept,
                builders,
                ..
            } = &mut *reg;
            let builder = (arg(0) as usize)
                .checked_sub(1)
                .and_then(|at| builders.get_mut(at))
                .ok_or_else(|| Trap::new("records: no such builder"))?;
            Ok(Rets::one(builder.build(
                records,
                kept,
                interp,
                arg(1) as i64,
                arg(2),
            )?))
        }
        MetaOp::RecOf => {
            let mut reg = shared.borrow_mut();
            Ok(Rets::one(
                builder(&mut reg, arg(0))?.record_of(arg(1)) as u64
            ))
        }
        MetaOp::RecField => Ok(Rets::one(field_name(interp, arg(1), |name| {
            shared.borrow().records.kind(arg(0) as i64, name) as u64
        }))),
        MetaOp::RecCount => {
            let reg = shared.borrow();
            Ok(Rets::one(field_name(interp, arg(1), |name| {
                match reg.records.field(arg(0) as i64, name) {
                    Some(FieldRef::List(items)) => items.len() as u64,
                    _ => 0,
                }
            })))
        }
        MetaOp::RecInt | MetaOp::RecRef | MetaOp::RecItemInt | MetaOp::RecItemRef => {
            let index =
                matches!(op, MetaOp::RecItemInt | MetaOp::RecItemRef).then(|| arg(2) as usize);
            let reg = shared.borrow();
            Ok(Rets::one(field_name(interp, arg(1), |name| {
                match reg.records.item(arg(0) as i64, name, index) {
                    Some(Item::Int(v) | Item::Ref(v)) => v as u64,
                    _ => 0,
                }
            })))
        }
        MetaOp::RecString | MetaOp::RecItemString => {
            let (index, out) = if op == MetaOp::RecItemString {
                (Some(arg(2) as usize), 3)
            } else {
                (None, 2)
            };
            let mut reg = shared.borrow_mut();
            let found = field_name(interp, arg(1), |name| {
                match reg.records.item(arg(0) as i64, name, index) {
                    Some(Item::Str(s)) => Some(s),
                    _ => None,
                }
            });
            let (count, data) = found.map_or((0, 0), |s| reg.keep_rc(&s));
            store_pair(interp, arg(out), count, data)?;
            Ok(Rets::default())
        }
        _ => unreachable!("not a records primitive"),
    }
}

/// Run `f` on the Jai `string` at `p` as a field name. Field names are ASCII identifiers, so
/// anything else (a null string, or one that cannot be read) names no field: `f` gets "".
fn field_name<R>(interp: &Interp, p: u64, f: impl FnOnce(&str) -> R) -> R {
    // Names are short: they are read onto the stack, not into an allocation.
    let mut stack = [0u8; 48];
    let heap;
    let bytes = match interp.read_pair(p) {
        Some((count, data)) if count as usize <= stack.len() => {
            let out = &mut stack[..count as usize];
            interp.read_into(data, out).then_some(&*out)
        }
        Some((count, data)) => {
            heap = interp.read(data, count as usize);
            heap.as_deref()
        }
        None => None,
    };
    f(bytes
        .and_then(|bytes| std::str::from_utf8(bytes).ok())
        .unwrap_or(""))
}

/// The builder `__jaic_rec_builder` numbered `handle`.
fn builder(reg: &mut Workspaces, handle: u64) -> Result<&mut crate::record_build::Builder, Trap> {
    (handle as usize)
        .checked_sub(1)
        .and_then(|at| reg.builders.get_mut(at))
        .ok_or_else(|| Trap::new("records: no such builder"))
}
