//! Checked event transfer. CFG coverage, fuel and publication stay in `checked`.
use super::domain::*;
use super::entry::Overlays;
use super::*;
use jaic::ide::lifetimes::{
    LifetimeBody, LifetimeBoolean, LifetimeEvent as Event, LifetimeOperand,
};
use jaic::ir::BinOp;

pub(super) struct Transfer<'a> {
    pub c: &'a Compiler,
    pub b: &'a LifetimeBody,
    pub entry_copies: &'a HashMap<usize, Site>,
    pub overlays: &'a Overlays<'a>,
}

impl Transfer<'_> {
    /// Apply one event after its fuel and comparison refresh, for example an actual Store.
    pub fn apply(&self, s: &mut State, e: &Event, at: &Site) {
        match e {
            Event::Parameter {
                ..
            } => self.parameter(s, e, at),
            Event::FieldProjection {
                ..
            }
            | Event::Copy {
                ..
            }
            | Event::View {
                ..
            }
            | Event::Load {
                ..
            } => self.memory(s, e, at),
            Event::Initialize {
                ..
            }
            | Event::Store {
                ..
            } => self.write(s, e, at),
            Event::Offset {
                ..
            }
            | Event::IntegerBinary {
                ..
            }
            | Event::Boolean {
                ..
            } => self.arithmetic(s, e, at),
            Event::ContextSet {
                ..
            }
            | Event::ContextRead {
                ..
            }
            | Event::ContextAlias {
                ..
            } => self.context(s, e, at),
            Event::Call {
                ..
            }
            | Event::IndirectCall {
                ..
            } => self.call(s, e, at),
            Event::Return {
                ..
            } => self.returned(s, e, at),
            Event::Barrier => self.barrier(s, e, at),
            _ => {}
        }
    }

    fn parameter(&self, s: &mut State, e: &Event, at: &Site) {
        let c = self.c;
        let entry_copies = self.entry_copies;
        if let Event::Parameter {
            formal,
            place,
            incoming,
            ty,
            ..
        } = e
        {
            if scalar(c, *ty) {
                let mut value = if let Some(view) = c.ty.types.pointee(*ty) {
                    Expr::Instance(Instance {
                        pointer: Captured::FormalValue {
                            formal: *formal,
                            slot: ByteSlot {
                                offset: 0,
                                width: c.ty.types.size_of(*ty),
                                ty: *ty,
                            },
                        },
                        view_ty: view,
                    })
                } else {
                    Expr::Scalar(Operand::Captured(Captured::FormalValue {
                        formal: *formal,
                        slot: ByteSlot {
                            offset: 0,
                            width: c.ty.types.size_of(*ty),
                            ty: *ty,
                        },
                    }))
                };
                if let Some(input) = s.bypass.clone() {
                    if *formal == input.stop_formal {
                        value = Expr::Scalar(Operand::Constant {
                            ty: *ty,
                            bits: 0,
                        });
                    }
                    if *formal == input.receiver_formal {
                        if let Expr::Instance(instance) = &value {
                            let a = Address {
                                root: Root::Object(InstanceKey(instance.clone())),
                                offset: input.head_slot.offset,
                            };
                            s.slots.insert(
                                (
                                    a,
                                    ByteSlot {
                                        offset: 0,
                                        ..input.head_slot
                                    },
                                ),
                                Expr::Scalar(Operand::Constant {
                                    ty: input.head_slot.ty,
                                    bits: 0,
                                }),
                            );
                        } else {
                            s.bypass_failed = true;
                        }
                    }
                }
                s.values.insert(*incoming, value.clone());
                if let Some(a) = s.address(*place) {
                    s.write(
                        c,
                        &a,
                        ByteSlot {
                            offset: 0,
                            width: c.ty.types.size_of(*ty),
                            ty: *ty,
                        },
                        value,
                        at,
                    );
                }
            } else if let (Some(a), Some(fields)) = (s.address(*place), leaves(c, *ty))
                && s.checked(c, &a, c.ty.types.size_of(*ty))
            {
                for slot in fields {
                    s.write(
                        c,
                        &a,
                        slot,
                        Expr::Scalar(Operand::Captured(Captured::FormalValue {
                            formal: *formal,
                            slot,
                        })),
                        at,
                    );
                }
            }
            if !scalar(c, *ty)
                && let Some(fields) = leaves(c, *ty).filter(|fields| !fields.is_empty())
                && let (Some(copy), Some(a)) = (entry_copies.get(formal), s.address(*place))
            {
                s.install_whole(
                    c,
                    &a,
                    *ty,
                    WholeOrigin {
                        formal: *formal,
                        ty: *ty,
                        entry_copy: copy.clone(),
                        last_copy: copy.clone(),
                        fields: fields.into(),
                    },
                );
            }
        }
    }

    fn memory(&self, s: &mut State, e: &Event, at: &Site) {
        let c = self.c;
        let last = at.chain.last().expect("transfer site");
        let block = last.block;
        let event = last.event;
        match e {
            Event::FieldProjection {
                base,
                result,
                base_ty,
                result_ty,
                offset,
            } => {
                let a = s.address(*base);
                s.addresses.remove(result);
                if let Some(base_address) = a
                    && c.ty.types.size_of(*base_ty) > 0
                {
                    if let Some(a) = s.project(
                        c,
                        &base_address,
                        *offset as i128,
                        c.ty.types.size_of(*result_ty),
                    ) {
                        s.projections.insert(
                            (block, event),
                            Projection {
                                base: base_address.clone(),
                                result: a.clone(),
                                base_ty: *base_ty,
                                result_ty: *result_ty,
                                offset: *offset,
                            },
                        );
                        s.addresses.insert(*result, a);
                    } else {
                        s.values.insert(*result, Expr::Unknown);
                    }
                }
            }
            Event::Copy {
                value,
                source,
            } => {
                let v = s.value(*source);
                let a = s.address(*source);
                s.values.insert(*value, v);
                s.addresses.remove(value);
                if let Some(a) = a {
                    s.addresses.insert(*value, a);
                }
            }
            Event::View {
                value,
                source,
                from,
                to,
                ..
            } => {
                let v = s.value(*source);
                let mut a = s.address(*source);
                if c.ty.types.pointee(*from).is_some() && c.ty.types.pointee(*to).is_some() {
                    if let (
                        Some(Address {
                            root: Root::Object(i),
                            ..
                        }),
                        Some(view),
                    ) = (&mut a, c.ty.types.pointee(*to))
                    {
                        i.0.view_ty = view;
                    }
                    s.values.insert(*value, v);
                    s.addresses.remove(value);
                    if let Some(a) = a {
                        s.addresses.insert(*value, a);
                    }
                } else if c.ty.types.size_of(*from) == c.ty.types.size_of(*to) {
                    s.values.insert(*value, v);
                }
            }
            Event::Load {
                place,
                value,
                ty,
                ..
            } => {
                if scalar(c, *ty) {
                    let v = s.address(*place).map_or(Expr::Unknown, |a| {
                        s.read(
                            c,
                            &a,
                            ByteSlot {
                                offset: 0,
                                width: c.ty.types.size_of(*ty),
                                ty: *ty,
                            },
                            at,
                        )
                    });
                    s.addresses.remove(value);
                    if let Some(view) = c.ty.types.pointee(*ty) {
                        let pointer = v.captured();
                        if pointer != Captured::Unknown {
                            s.addresses.insert(
                                *value,
                                Address {
                                    root: Root::Object(InstanceKey(Instance {
                                        pointer,
                                        view_ty: view,
                                    })),
                                    offset: 0,
                                },
                            );
                        }
                    }
                    s.values.insert(*value, v);
                } else {
                    let a = s.address(*place);
                    s.addresses.remove(value);
                    if let Some(a) = a {
                        s.addresses.insert(*value, a);
                    }
                }
            }
            _ => {}
        }
    }

    fn write(&self, s: &mut State, e: &Event, at: &Site) {
        let c = self.c;
        match e {
            Event::Initialize {
                place,
                ty,
                ..
            } => {
                if let Some(a) = s.address(*place) {
                    s.note_write(&a);
                    if !matches!(a.root, Root::Local(_) | Root::Output { .. })
                        || matches!(a.root,Root::Local(v) if s.exposed.contains(&v))
                    {
                        s.opaque();
                    }
                    if s.checked(c, &a, c.ty.types.size_of(*ty)) {
                        s.invalidate_aliases(&a);
                        s.clear(&a, c.ty.types.size_of(*ty));
                    } else {
                        s.opaque();
                    }
                } else {
                    s.opaque();
                }
            }
            Event::Store {
                place,
                value,
                ty,
                constant,
                ..
            } => {
                let width = c.ty.types.size_of(*ty);
                if let Some(a) = s.address(*place) {
                    s.note_write(&a);
                    if !s.checked(c, &a, width) {
                        s.opaque();
                    } else if scalar(c, *ty) {
                        let v = constant.map_or_else(
                            || {
                                let v = s.value(*value);
                                if v == Expr::Unknown && c.ty.types.pointee(*ty).is_some() {
                                    s.address(*value).map(Expr::Address).unwrap_or(v)
                                } else {
                                    v
                                }
                            },
                            |n| literal(c, *ty, n),
                        );
                        s.write(
                            c,
                            &a,
                            ByteSlot {
                                offset: 0,
                                width,
                                ty: *ty,
                            },
                            v,
                            at,
                        );
                    } else {
                        if !matches!(a.root, Root::Local(_) | Root::Output { .. })
                            || matches!(a.root,Root::Local(v) if s.exposed.contains(&v))
                        {
                            s.opaque();
                        }
                        let origin = s.address(*value).and_then(|from| s.whole(c, &from, *ty));
                        let mut copied = Vec::new();
                        if let (Some(from), Some(fields)) = (s.address(*value), leaves(c, *ty))
                            && s.checked(c, &from, width)
                        {
                            for slot in fields {
                                copied.push((slot, s.read(c, &from, slot, at)));
                            }
                        }
                        s.invalidate_aliases(&a);
                        s.clear(&a, width);
                        for (slot, v) in copied {
                            s.write(c, &a, slot, v, at);
                        }
                        if let Some(mut origin) = origin
                            && matches!(a.root, Root::Local(_) | Root::Output { .. })
                        {
                            origin.last_copy = at.clone();
                            s.install_whole(c, &a, *ty, origin);
                        }
                    }
                } else {
                    s.opaque();
                }
            }
            _ => {}
        }
    }

    fn arithmetic(&self, s: &mut State, e: &Event, _at: &Site) {
        let c = self.c;
        match e {
            Event::Offset {
                value,
                source,
                stride,
                offset,
                subscript,
                ..
            } => {
                let old = s.value(*source);
                let a = s.address(*source);
                let index = offset
                    .map(|n| literal(c, TypeId::S64, n as u64))
                    .or_else(|| subscript.map(|v| s.value(v)))
                    .unwrap_or(Expr::Unknown);
                let allowed = old
                    .size()
                    .zip(index.size())
                    .is_some_and(|((n, d), (m, e))| {
                        n.checked_add(m)
                            .and_then(|n| n.checked_add(1))
                            .is_some_and(|n| n <= 128)
                            && d.max(e) < 16
                    });
                let v = if allowed {
                    Expr::Offset {
                        base: Box::new(old),
                        index: Box::new(index),
                        stride: c.ty.types.size_of(*stride),
                    }
                } else {
                    Expr::Unknown
                };
                s.values.insert(*value, v);
                s.addresses.remove(value);
                if let (Some(a), Some(delta)) = (a, *offset)
                    && let Some(a) = delta
                        .checked_mul(c.ty.types.size_of(*stride) as i128)
                        .and_then(|d| s.project(c, &a, d, 1))
                {
                    s.addresses.insert(*value, a);
                }
            }
            Event::IntegerBinary {
                value,
                left,
                right,
                ty,
                op,
                left_constant,
                right_constant,
            } => {
                let a = left_constant.map_or_else(|| s.value(*left), |n| literal(c, *ty, n));
                let d = right_constant.map_or_else(|| s.value(*right), |n| literal(c, *ty, n));
                let safe = a.size().zip(d.size()).is_some_and(|((n, x), (m, y))| {
                    n.checked_add(m)
                        .and_then(|n| n.checked_add(1))
                        .is_some_and(|n| n <= 128)
                        && x.max(y) < 16
                });
                let v = if safe && c.ty.types.size_of(*ty) == 8 {
                    match op {
                        BinOp::Add => Expr::Add {
                            left: Box::new(a),
                            right: Box::new(d),
                            ty: *ty,
                        },
                        BinOp::Sub => Expr::Sub {
                            left: Box::new(a),
                            right: Box::new(d),
                            ty: *ty,
                        },
                        _ => Expr::Unknown,
                    }
                } else {
                    Expr::Unknown
                };
                s.values.insert(*value, v);
            }
            Event::Boolean {
                value,
                operation,
            } => {
                let make = |v| match s.value(v) {
                    Expr::Predicate(p) => p,
                    _ => Predicate::Unknown,
                };
                let p = match operation {
                    LifetimeBoolean::Not(a) => {
                        let p = make(*a);
                        if predicate_size(&p).is_some_and(|(n, d)| n < 128 && d < 16) {
                            Predicate::Not(Box::new(p))
                        } else {
                            Predicate::Unknown
                        }
                    }
                    LifetimeBoolean::And(a, d) | LifetimeBoolean::Or(a, d) => {
                        let a = make(*a);
                        let d = make(*d);
                        if predicate_size(&a).zip(predicate_size(&d)).is_some_and(
                            |((n, x), (m, y))| {
                                n.checked_add(m)
                                    .and_then(|n| n.checked_add(1))
                                    .is_some_and(|n| n <= 128)
                                    && x.max(y) < 16
                            },
                        ) {
                            if matches!(operation, LifetimeBoolean::And(..)) {
                                Predicate::And(Box::new(a), Box::new(d))
                            } else {
                                Predicate::Or(Box::new(a), Box::new(d))
                            }
                        } else {
                            Predicate::Unknown
                        }
                    }
                };
                s.values.insert(*value, Expr::Predicate(p));
            }
            _ => {}
        }
    }

    fn context(&self, s: &mut State, e: &Event, at: &Site) {
        let b = self.b;
        match e {
            Event::ContextSet {
                value, ..
            } => {
                if s.context.is_none() {
                    if let [ty] = b.context_types.as_slice() {
                        s.context = Some((at.clone(), *ty));
                        s.context_values.insert(*value, (at.clone(), *ty));
                    }
                } else {
                    s.context = s.context_values.get(value).cloned();
                }
            }
            Event::ContextRead {
                value, ..
            } => {
                if let Some((selection, ty)) = s.context.clone() {
                    s.addresses.insert(
                        *value,
                        Address {
                            root: Root::Context {
                                site: selection.clone(),
                                ty,
                            },
                            offset: 0,
                        },
                    );
                    s.context_values.insert(*value, (selection, ty));
                }
            }
            Event::ContextAlias {
                value,
                source,
            } => {
                if let Some(x) = s.context_values.get(source).cloned() {
                    s.context_values.insert(*value, x);
                }
            }
            _ => {}
        }
    }

    fn call(&self, s: &mut State, e: &Event, _at: &Site) {
        match e {
            Event::Call {
                args, ..
            }
            | Event::IndirectCall {
                args, ..
            } => {
                if let Some(Address {
                    root: Root::Local(v),
                    ..
                }) = s.current_context_address.clone()
                {
                    s.expose(v);
                }
                for arg in args {
                    if let LifetimeOperand::Value(v) | LifetimeOperand::Place(v) = arg.operand
                        && let Some(Address {
                            root: Root::Local(root),
                            ..
                        }) = s.address(v)
                    {
                        s.expose(root);
                    }
                }
                s.callback();
            }
            _ => {}
        }
    }

    fn returned(&self, s: &mut State, e: &Event, at: &Site) {
        let c = self.c;
        if let Event::Return {
            index,
            value,
            ty,
            constant,
            ..
        } = e
        {
            let v = constant.map_or_else(|| s.value(*value), |n| literal(c, *ty, n));
            s.returns
                .push((*index, *ty, v, s.address(*value), at.clone()));
        }
    }

    fn barrier(&self, s: &mut State, e: &Event, at: &Site) {
        let c = self.c;
        let b = self.b;
        let overlays = self.overlays;
        let last = at.chain.last().expect("transfer site");
        let block = last.block;
        let event = last.event;
        if let Event::Barrier = e {
            let certified = overlays.get(&(block, event)).is_some_and(|access| {
                let key = (access.projection_block, access.projection_event);
                let Some(projection) = s.projections.get(&key) else {
                    return false;
                };
                let Some(Event::FieldProjection {
                    result,
                    base_ty,
                    result_ty,
                    offset,
                    ..
                }) = b.blocks.get(key.0).and_then(|b| b.events.get(key.1))
                else {
                    return false;
                };
                *result == access.result
                    && *base_ty == projection.base_ty
                    && *result_ty == projection.result_ty
                    && *offset == projection.offset
                    && c.ty.types.repr(*base_ty) == c.ty.types.repr(access.descriptor_ty)
                    && s.address(access.result).as_ref() == Some(&projection.result)
                    && s.project(
                        c,
                        &projection.base,
                        projection.offset as i128,
                        c.ty.types.size_of(projection.result_ty),
                    )
                    .as_ref()
                        == Some(&projection.result)
            });
            if !certified {
                s.opaque();
            }
        }
    }
}
