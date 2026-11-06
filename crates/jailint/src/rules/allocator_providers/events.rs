//! Typed value, address and storage transfers; CFG sequencing stays in checked.
use super::*;

pub(super) fn transfer(
    c: &Compiler,
    b: &LifetimeBody,
    e: &Event,
    anchor: &Site,
    s: &mut State,
    initial_context: Option<&Site>,
    branch: &mut Option<(Val, usize, usize)>,
) {
    if inputs(c, b, e, anchor, s)
        || storage(c, e, anchor, s)
        || values(c, e, s)
        || context(e, anchor, s, initial_context)
    {
        return;
    }
    match e {
        Event::Branch {
            condition,
            then_block,
            else_block,
            ..
        } => *branch = Some((*condition, *then_block, *else_block)),
        Event::IndirectCall {
            callee,
            args,
            results,
            ..
        } => {
            if let Some(p) = protocol(c, e) {
                let procedure = s.value(*callee);
                let op = argument(c, s, args, p.operation_formal);
                let data = argument(c, s, args, p.data_formal);
                let row = ProviderUse {
                    site: anchor.clone(),
                    operation: operation(c, &op, Some(&p)),
                    provider: Pair {
                        procedure_value: procedure.capture.clone(),
                        data_value: data.capture.clone(),
                    },
                    guards: s.guards.clone(),
                };
                s.uses.push(Use {
                    row,
                    operation: op,
                    procedure,
                    data,
                    protocol: Some(p.clone()),
                });
                opaque(c, s, args);
                s.memory_effects = true;
                for (index, v, ty) in results {
                    s.values.insert(
                        *v,
                        if *index == p.result_position && c.ty.types.pointee(*ty).is_some() {
                            Value {
                                source: ValueSource::Callback {
                                    use_site: anchor.clone(),
                                    result: *index,
                                    byte_offset: 0,
                                },
                                ..Value::unknown(*ty)
                            }
                        } else {
                            Value::unknown(*ty)
                        },
                    );
                }
            } else {
                opaque(c, s, args);
                s.memory_effects = true;
                s.incomplete = true;
                for (_, v, ty) in results {
                    s.values.insert(*v, Value::unknown(*ty));
                }
            }
        }
        Event::Return {
            index,
            value,
            ty,
            constant,
            ..
        } => {
            let v = constant.map_or_else(|| operand(c, s, *value, *ty), |n| literal(c, *ty, n));
            s.return_values.insert(*index, v.clone());
            if !scalar(c, *ty) && c.ty.types.size_of(*ty) > 0 {
                s.incomplete = true;
                s.returns.push(ReturnSource {
                    result: *index,
                    source: ValueSource::Unknown,
                    guards: s.guards.clone(),
                });
            }
            if c.ty.types.pointee(*ty).is_some() {
                s.returns.push(ReturnSource {
                    result: *index,
                    source: v.source,
                    guards: s.guards.clone(),
                });
            }
        }
        Event::Barrier => {
            opaque(c, s, &[]);
            s.memory_effects = true;
            s.incomplete = true;
        }
        _ => {}
    }
}

fn inputs(c: &Compiler, b: &LifetimeBody, e: &Event, anchor: &Site, s: &mut State) -> bool {
    match e {
        Event::Parameter {
            formal,
            place,
            incoming,
            ty,
            ..
        } => {
            if scalar(c, *ty) {
                let slot = ByteSlot {
                    offset: 0,
                    width: c.ty.types.size_of(*ty),
                    ty: *ty,
                };
                let mut v = Value::captured(
                    *ty,
                    Captured::FormalValue {
                        formal: *formal,
                        slot,
                    },
                );
                if let Some(pointee) = c.ty.types.pointee(*ty) {
                    v.address = Some(Address {
                        root: Root::Receiver {
                            formal: *formal,
                            ty: pointee,
                        },
                        offset: 0,
                    });
                    v.parameter_pointer = Some((*formal, 0));
                }
                s.values.insert(*incoming, v.clone());
                if let Some(a) = s.address(*place) {
                    s.write(c, &a, slot, v);
                }
            } else if let (Some(a), Some(fields)) = (s.address(*place), leaves(c, *ty)) {
                if s.checked(c, &a, c.ty.types.size_of(*ty)) {
                    for slot in fields {
                        s.write(
                            c,
                            &a,
                            slot,
                            Value::captured(
                                slot.ty,
                                Captured::FormalValue {
                                    formal: *formal,
                                    slot,
                                },
                            ),
                        );
                    }
                } else {
                    s.incomplete = true;
                }
            } else {
                s.incomplete = true;
            }
        }
        Event::FieldProjection {
            base,
            result,
            base_ty,
            result_ty,
            offset,
        } => {
            let mut a = s.address(*base);
            if a.is_none()
                && !s.unsupported_addresses.contains(base)
                && !s.value(*base).unsupported_address
                && let Some(selection) = s.context_values.get(base)
                && b.context_types.contains(base_ty)
            {
                a = Some(Address {
                    root: Root::Context {
                        selection: selection.clone(),
                        ty: *base_ty,
                    },
                    offset: 0,
                });
            }
            if result != base {
                s.addresses.remove(result);
            }
            let unsupported = s.unsupported_addresses.contains(base)
                || s.value(*base).unsupported_address
                || a.is_some();
            if let Some(a) = a {
                if let Some(a) = s.project(c, &a, *offset as i128, c.ty.types.size_of(*result_ty)) {
                    s.addresses.insert(*result, a);
                } else if unsupported {
                    s.unsupported_addresses.insert(*result);
                }
            } else if unsupported {
                s.unsupported_addresses.insert(*result);
            }
        }
        Event::Copy {
            value,
            source,
        } => {
            let v = s.value(*source);
            let a = s.address(*source);
            s.values.insert(*value, v);
            if s.unsupported_addresses.contains(source) {
                s.unsupported_addresses.insert(*value);
            }
            if let Some(a) = a {
                s.addresses.insert(*value, a);
            }
            if let Some(c) = s.context_values.get(source).cloned() {
                s.context_values.insert(*value, c);
            }
        }
        Event::Load {
            place,
            value,
            ty,
            ..
        } => {
            if scalar(c, *ty) {
                let v = s.address(*place).map_or_else(
                    || Value::unknown(*ty),
                    |a| {
                        s.read(
                            c,
                            &a,
                            ByteSlot {
                                offset: 0,
                                width: c.ty.types.size_of(*ty),
                                ty: *ty,
                            },
                            ReadEpoch::Load(anchor.clone()),
                        )
                    },
                );
                s.values.insert(*value, v);
            } else if let Some(a) = s.address(*place) {
                s.addresses.insert(*value, a);
            }
        }
        _ => return false,
    }
    true
}

fn storage(c: &Compiler, e: &Event, anchor: &Site, s: &mut State) -> bool {
    match e {
        Event::Initialize {
            place,
            ty,
            ..
        } => {
            if let Some(a) = s.address(*place) {
                if s.checked(c, &a, c.ty.types.size_of(*ty)) {
                    if !matches!(a.root, Root::Private(_)) {
                        s.clobber();
                        s.memory_effects = true;
                    }
                    s.clear(&a, c.ty.types.size_of(*ty));
                } else {
                    s.slots.clear();
                    s.memory_effects = true;
                    s.incomplete = true;
                }
            } else {
                s.slots.clear();
                s.memory_effects = true;
                s.incomplete = true;
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
                if scalar(c, *ty) {
                    let v =
                        constant.map_or_else(|| operand(c, s, *value, *ty), |k| literal(c, *ty, k));
                    if s.checked(c, &a, width)
                        && let Root::Receiver {
                            formal,
                            ty: receiver_ty,
                        } = a.root
                        && c.ty.types.pointee(*ty).is_some()
                    {
                        s.stores.push(StoreSource {
                            site: anchor.clone(),
                            receiver_formal: formal,
                            receiver_ty,
                            slot: ByteSlot {
                                offset: a.offset,
                                width,
                                ty: *ty,
                            },
                            source: v.source.clone(),
                            guards: s.guards.clone(),
                        });
                    }
                    if !matches!(a.root, Root::Private(_)) {
                        s.memory_effects = true;
                    }
                    s.write(
                        c,
                        &a,
                        ByteSlot {
                            offset: 0,
                            width,
                            ty: *ty,
                        },
                        v,
                    );
                } else if s.checked(c, &a, width) {
                    let from = s.address(*value);
                    let mut captured = Vec::new();
                    if let (Some(from), Some(fields)) = (from, leaves(c, *ty))
                        && s.checked(c, &from, width)
                    {
                        for slot in fields {
                            captured.push((
                                slot,
                                s.read(c, &from, slot, ReadEpoch::AggregateCopy(anchor.clone())),
                            ));
                        }
                    }
                    if !matches!(a.root, Root::Private(_)) {
                        s.memory_effects = true;
                        s.clobber();
                    }
                    s.clear(&a, width);
                    for (slot, v) in captured {
                        s.write(c, &a, slot, v);
                    }
                } else {
                    s.slots.clear();
                    s.memory_effects = true;
                    s.incomplete = true;
                }
            } else {
                if s.unsupported_addresses.contains(place) || s.value(*place).unsupported_address {
                    s.slots.clear();
                    s.incomplete = true;
                } else {
                    s.clobber();
                }
                s.memory_effects = true;
            }
        }
        Event::Address {
            place, ..
        } => {
            if let Some(Address {
                root: Root::Private(v),
                ..
            }) = s.address(*place)
            {
                s.expose(v);
            }
        }
        _ => return false,
    }
    true
}

fn values(c: &Compiler, e: &Event, s: &mut State) -> bool {
    match e {
        Event::Procedure {
            value,
            callee,
            ty,
            ..
        } => {
            s.values
                .insert(*value, Value::captured(*ty, Captured::Procedure(*callee)));
        }
        Event::Offset {
            value,
            source,
            stride,
            offset,
            subscript,
            ..
        } => {
            let old = s.value(*source);
            let old_address = s.address(*source);
            let delta = offset.or_else(|| subscript.and_then(|v| constant_signed(c, &s.value(v))));
            let derived_storage = old_address.is_some()
                || s.unsupported_addresses.contains(source)
                || old.unsupported_address;
            if value != source {
                s.values.remove(value);
                s.addresses.remove(value);
            }
            if derived_storage {
                s.unsupported_addresses.insert(*value);
            }
            if let Some(bytes) = delta
                .and_then(|n| n.checked_mul(c.ty.types.size_of(*stride) as i128))
                .and_then(|n| i64::try_from(n).ok())
            {
                let mut v = displacement(c, old, bytes);
                v.unsupported_address = derived_storage;
                v.address = old_address.and_then(|a| s.project(c, &a, bytes as i128, 1));
                if let Some(a) = v.address.clone() {
                    s.addresses.insert(*value, a);
                    s.unsupported_addresses.remove(value);
                    v.unsupported_address = false;
                }
                s.values.insert(*value, v);
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
            let b = right_constant.map_or_else(|| s.value(*right), |n| literal(c, *ty, n));
            s.values.insert(*value, arithmetic(c, *ty, &a, *op, &b));
        }
        Event::Boolean {
            value,
            operation,
        } => {
            let p = match operation {
                LifetimeBoolean::Not(a) => Predicate::not(&s.simplified(&s.value(*a).boolean(c))),
                LifetimeBoolean::And(a, b) => Predicate::and(
                    &s.simplified(&s.value(*a).boolean(c)),
                    &s.simplified(&s.value(*b).boolean(c)),
                ),
                LifetimeBoolean::Or(a, b) => Predicate::or(
                    &s.simplified(&s.value(*a).boolean(c)),
                    &s.simplified(&s.value(*b).boolean(c)),
                ),
            };
            s.values.insert(
                *value,
                Value {
                    predicate: Some(s.simplified(&p)),
                    ..Value::unknown(TypeId::BOOL)
                },
            );
        }
        _ => return false,
    }
    true
}

fn context(e: &Event, anchor: &Site, s: &mut State, initial_context: Option<&Site>) -> bool {
    match e {
        Event::ContextSet {
            value, ..
        } => {
            s.current_context = Some(anchor.clone());
            s.current_context_valid = initial_context == Some(anchor)
                || s.context_valid_values.contains(value)
                || s.address(*value).is_some();
            s.current_context_unsupported = !s.current_context_valid
                || s.unsupported_addresses.contains(value)
                || s.value(*value).unsupported_address;
            s.current_context_address = s
                .address(*value)
                .or_else(|| s.context_addresses.get(value).cloned());
        }
        Event::ContextRead {
            value, ..
        } => {
            if let Some(ctx) = s.current_context.clone() {
                s.context_values.insert(*value, ctx);
                if s.current_context_unsupported {
                    s.unsupported_addresses.insert(*value);
                } else {
                    s.context_valid_values.insert(*value);
                }
                if let Some(a) = s.current_context_address.clone() {
                    s.context_addresses.insert(*value, a.clone());
                    s.addresses.insert(*value, a);
                }
            }
        }
        Event::ContextAlias {
            value,
            source,
        } => {
            if let Some(ctx) = s.context_values.get(source).cloned() {
                s.context_values.insert(*value, ctx);
                if s.unsupported_addresses.contains(source) || s.value(*source).unsupported_address
                {
                    s.unsupported_addresses.insert(*value);
                }
                if s.context_valid_values.contains(source) {
                    s.context_valid_values.insert(*value);
                }
                if let Some(a) = s.context_addresses.get(source).cloned() {
                    s.context_addresses.insert(*value, a.clone());
                    s.addresses.insert(*value, a);
                }
            }
        }
        _ => return false,
    }
    true
}
