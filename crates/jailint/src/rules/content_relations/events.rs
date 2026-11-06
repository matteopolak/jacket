//! Typed descriptor/value transfers; CFG and summary publication remain in checked.
use super::*;

impl Solver<'_> {
    /// True preserves the original early continuation of a widened store state.
    pub(super) fn event(
        &self,
        b: &LifetimeBody,
        e: &Event,
        site: &Site,
        state: &mut State,
        branch: &mut Option<(Val, usize, usize)>,
    ) -> Option<bool> {
        if matches!(e, Event::Store { .. }) {
            return store(self.c, self.layout, b, e, site, state);
        }
        if inputs(self.c, self.layout, e, state)?
            || computed(self.c, e, state)?
            || local(self.c, b, e, site, state)
        {
            return Some(false);
        }
        match e {
            Event::Branch {
                condition,
                then_block,
                else_block,
                ..
            } => {
                *branch = Some((*condition, *then_block, *else_block));
            }
            Event::IndirectCall {
                args,
                results,
                ..
            } => {
                state.opaque(args);
                self.results(state, results, site);
            }
            Event::Return {
                index,
                value,
                ty: TypeId::STRING,
                ..
            } => {
                state
                    .returns
                    .insert(*index, state.current_descriptor(*value, self.layout));
            }
            Event::Barrier => state.opaque(&[]),
            _ => {}
        }
        Some(false)
    }
}

fn inputs(c: &Compiler, layout: Layout, e: &Event, state: &mut State) -> Option<bool> {
    match e {
        Event::Parameter {
            formal,
            place,
            incoming,
            ty,
            ..
        } => {
            state.root(*place, c.ty.types.size_of(*ty), false);
            if *ty == TypeId::STRING {
                let descriptor = Descriptor {
                    count: Scalar::StringCount(*formal),
                    data: Some(Pointer {
                        base: Base::InputData(*formal),
                        offset: 0,
                    }),
                };
                state.descriptor_at(*place, &descriptor, layout);
                state.snapshots.insert(*incoming, descriptor);
            } else {
                let value = if c.ty.types.pointee(*ty).is_some() {
                    Value::Pointer(Pointer {
                        base: Base::Parameter(*formal),
                        offset: 0,
                    })
                } else if c.ty.types.int_info(*ty).is_some() || *ty == TypeId::BOOL {
                    Value::Scalar(Scalar::Formal(*formal))
                } else {
                    Value::Unknown
                };
                state.values.insert(*incoming, value.clone());
                state.put(
                    state.address(*place)?,
                    ByteSlot {
                        offset: 0,
                        width: c.ty.types.size_of(*ty),
                        ty: *ty,
                    },
                    value,
                );
            }
        }
        Event::Local {
            place,
            ty,
            ..
        } => {
            if state.addresses.contains_key(place) {
                state.root(*place, c.ty.types.size_of(*ty), false);
            }
        }
        Event::FieldProjection {
            base,
            result,
            base_ty,
            result_ty,
            offset,
        } => {
            let base_address = state.address(*base);
            if result != base {
                state.addresses.remove(result);
            }
            if let Some(base) = base_address {
                let address = Address {
                    root: base.root,
                    offset: base.offset.checked_add(*offset)?,
                };
                if state.checked(address, c.ty.types.size_of(*result_ty))
                    && state.checked(base, c.ty.types.size_of(*base_ty))
                {
                    state.addresses.insert(*result, address);
                    if *result_ty == TypeId::STRING {
                        state.headers.insert(address);
                    }
                }
            }
        }
        Event::Copy {
            value,
            source,
        } => {
            state.values.insert(*value, state.value(*source));
            if let Some(a) = state.address(*source) {
                state.addresses.insert(*value, a);
            }
            if let Some(snapshot) = state.snapshots.get(source).cloned() {
                state.snapshots.insert(*value, snapshot);
            }
        }
        Event::Load {
            place,
            value,
            ty,
            ..
        } => {
            if *ty == TypeId::STRING {
                let descriptor = state.current_descriptor(*place, layout);
                state.snapshots.insert(*value, descriptor);
                state.invalid_snapshots.remove(value);
            } else {
                let loaded = state.address(*place).map_or(Value::Unknown, |a| {
                    state.get(
                        a,
                        ByteSlot {
                            offset: 0,
                            width: c.ty.types.size_of(*ty),
                            ty: *ty,
                        },
                    )
                });
                state.values.insert(*value, loaded);
            }
        }
        _ => return Some(false),
    }
    Some(true)
}

fn computed(c: &Compiler, e: &Event, state: &mut State) -> Option<bool> {
    match e {
        Event::Offset {
            value,
            source,
            stride,
            offset,
            subscript,
            ..
        } => {
            let source_pointer = state.pointer(*source);
            if value != source {
                state.addresses.remove(value);
                state.values.remove(value);
            }
            let delta = offset.or_else(|| {
                subscript.and_then(|v| {
                    if let Scalar::Constant(k) = state.scalar(v) {
                        Some(k)
                    } else {
                        None
                    }
                })
            });
            if let (Some(mut pointer), Some(delta)) = (source_pointer, delta) {
                let bytes = delta.checked_mul(c.ty.types.size_of(*stride) as i128)?;
                i64::try_from(bytes).ok()?;
                pointer.offset = pointer.offset.checked_add(bytes)?;
                i64::try_from(pointer.offset).ok()?;
                state.values.insert(*value, Value::Pointer(pointer));
            }
        }
        Event::IntegerBinary {
            value,
            left,
            right,
            op,
            left_constant,
            right_constant,
            ty,
        } => {
            let left = left_constant.map_or_else(|| state.value(*left), |k| literal(c, *ty, k));
            let right = right_constant.map_or_else(|| state.value(*right), |k| literal(c, *ty, k));
            state
                .values
                .insert(*value, arithmetic(c, *ty, &left, *op, &right));
        }
        Event::Boolean {
            value,
            operation,
        } => {
            let expression = match operation {
                LifetimeBoolean::Not(v) => Value::not(state.reference(*v)),
                LifetimeBoolean::And(a, b) => Value::and(state.reference(*a), state.reference(*b)),
                LifetimeBoolean::Or(a, b) => Value::or(state.reference(*a), state.reference(*b)),
            };
            state.values.insert(*value, expression);
        }
        _ => return Some(false),
    }
    Some(true)
}

fn local(c: &Compiler, b: &LifetimeBody, e: &Event, site: &Site, state: &mut State) -> bool {
    match e {
        Event::Initialize {
            place,
            ty,
            ..
        } => {
            if let Some(address) = state.address(*place) {
                super::ranges::private_write(
                    state,
                    address,
                    c.ty.types.size_of(*ty),
                    site,
                    b.procedure,
                );
                state.clear(address, Some(c.ty.types.size_of(*ty)));
                if *ty == TypeId::STRING {
                    state.headers.insert(address);
                }
            } else {
                state.opaque(&[]);
            }
        }
        Event::EmptyDescriptor {
            value,
            ty: TypeId::STRING,
        } => {
            state.snapshots.insert(
                *value,
                Descriptor {
                    count: Scalar::Constant(0),
                    data: Some(Pointer {
                        base: Base::Null,
                        offset: 0,
                    }),
                },
            );
            state.invalid_snapshots.remove(value);
        }
        Event::Address {
            place, ..
        } => {
            if let Some(a) = state.address(*place) {
                state.exposed.insert(a.root);
            }
        }
        _ => return false,
    }
    true
}

fn store(
    c: &Compiler,
    layout: Layout,
    b: &LifetimeBody,
    e: &Event,
    site: &Site,
    state: &mut State,
) -> Option<bool> {
    let Event::Store {
        place,
        value,
        ty,
        constant,
        ..
    } = e
    else {
        return Some(false);
    };
    let width = c.ty.types.size_of(*ty);
    if let Some(address) = state.address(*place) {
        if !state.checked(address, width) {
            state.opaque(&[]);
            return Some(true);
        }
        super::ranges::private_write(state, address, width, site, b.procedure);
        if *ty == TypeId::STRING {
            let snapshot = state.descriptor(*value, layout);
            state.clear(address, Some(width));
            state.put(address, layout.count, Value::Scalar(snapshot.count));
            state.put(
                address,
                layout.data,
                snapshot.data.map_or(Value::Unknown, Value::Pointer),
            );
            state.headers.insert(address);
        } else {
            let source = constant.map_or_else(|| state.value(*value), |k| literal(c, *ty, k));
            state.put(
                address,
                ByteSlot {
                    offset: 0,
                    width,
                    ty: *ty,
                },
                source,
            );
        }
    } else {
        let pointer = state.pointer(*place);
        if !super::ranges::write(state, pointer, width, site, b.procedure) {
            return None;
        }
    }
    Some(false)
}
