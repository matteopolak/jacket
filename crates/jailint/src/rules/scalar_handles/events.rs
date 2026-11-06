//! Checked local addressing and scalar storage event transfers.
use super::{
    Compiler, Event, Expr, HandleEffect, LifetimeBoolean, Site, Slot, State, Val, leaves, related,
    scalar,
};

impl State {
    /// Apply a non-call event; true preserves the unsupported-store skip path.
    pub(super) fn event(
        &mut self,
        c: &Compiler,
        e: &Event,
        site: Site,
        branch: &mut Option<(Val, usize, usize)>,
    ) -> bool {
        match e {
            Event::IndirectCall {
                args,
                results,
                ..
            } => {
                self.opaque(args);
                self.global(site.clone());
                for (_, v, _) in results {
                    self.values.insert(*v, Expr::Unknown);
                }
            }
            Event::Parameter {
                ..
            }
            | Event::Local {
                ..
            }
            | Event::FieldProjection {
                ..
            }
            | Event::Offset {
                ..
            }
            | Event::Copy {
                ..
            }
            | Event::Load {
                ..
            } => self.address_event(c, e),
            Event::Store {
                place,
                value,
                ty,
                constant,
                ..
            } => {
                if !self.addresses.contains_key(place)
                    || !self.checked(self.address(*place), c.ty.types.size_of(*ty))
                {
                    self.interferes = true;
                    self.global(site.clone());
                    let x = self.value(*value);
                    if let Expr::Fresh(t) = &x {
                        self.invalid.insert(t.clone());
                    }
                    if related(&x) {
                        self.uncertain.push(HandleEffect {
                            value: x,
                            kind: None,
                        });
                    }
                    self.slots.clear();
                    return true;
                }
                if scalar(c, *ty) {
                    let slot = Slot {
                        offset: 0,
                        width: c.ty.types.size_of(*ty),
                        ty: *ty,
                    };
                    let x = constant.map_or_else(|| self.value(*value), Expr::Constant);
                    self.put(*place, slot, x);
                } else {
                    self.copy_slots(*place, *value, Some(c.ty.types.size_of(*ty)));
                }
                let target = self.address(*place).root;
                if !self.original(target) {
                    let roots = &self.roots;
                    let exposed = &self.exposed;
                    self.slots.retain(|(a, _), _| {
                        a.root == target
                            || roots
                                .as_ref()
                                .is_some_and(|roots| roots.extent(a.root).is_some())
                                && !exposed.contains(&a.root)
                    });
                    self.interferes = true;
                    self.global(site.clone());
                }
            }
            Event::Initialize {
                place,
                ty,
                ..
            } => {
                if !self.addresses.contains_key(place)
                    || !self.checked(self.address(*place), c.ty.types.size_of(*ty))
                {
                    self.interferes = true;
                    self.global(site.clone());
                    self.slots.clear();
                }
                self.clear(*place, c.ty.types.size_of(*ty));
                for slot in leaves(c, *ty) {
                    self.put(*place, slot, Expr::Unknown);
                }
            }
            Event::Return {
                index,
                value,
                ty,
                constant,
                ..
            } => {
                if scalar(c, *ty) {
                    self.returns.push((
                        *index,
                        None,
                        constant.map_or_else(|| self.value(*value), Expr::Constant),
                    ));
                } else {
                    for slot in leaves(c, *ty) {
                        self.returns
                            .push((*index, Some(slot), self.get(*value, slot)));
                    }
                }
            }
            Event::Boolean {
                value,
                operation,
            } => {
                let expression = match *operation {
                    LifetimeBoolean::Not(source) => Expr::not(self.values.get(&source)),
                    LifetimeBoolean::And(left, right) => {
                        Expr::and(self.values.get(&left), self.values.get(&right))
                    }
                    LifetimeBoolean::Or(left, right) => {
                        Expr::or(self.values.get(&left), self.values.get(&right))
                    }
                };
                self.values.insert(*value, expression);
            }
            Event::Branch {
                condition,
                then_block,
                else_block,
                ..
            } => {
                *branch = Some((*condition, *then_block, *else_block));
            }
            Event::Address {
                place, ..
            } => {
                if let Some(address) = self.addresses.get(place) {
                    self.exposed.insert(address.root);
                }
            }
            Event::Barrier => {
                self.slots.clear();
                self.interferes = true;
                self.global(site.clone());
            }
            _ => {}
        }
        false
    }

    fn address_event(&mut self, c: &Compiler, e: &Event) {
        match e {
            Event::Parameter {
                formal,
                place,
                incoming,
                ty,
                ..
            } => {
                self.region(*place, c.ty.types.size_of(*ty));
                for slot in leaves(c, *ty) {
                    self.put(*place, slot, Expr::Input(*formal, slot));
                }
                if scalar(c, *ty) {
                    let slot = Slot {
                        offset: 0,
                        width: c.ty.types.size_of(*ty),
                        ty: *ty,
                    };
                    self.values.insert(*incoming, Expr::Input(*formal, slot));
                }
            }
            Event::Local {
                ..
            } => {}
            Event::FieldProjection {
                base,
                result,
                result_ty,
                offset,
                ..
            } => {
                let base_address = self.addresses.get(base).copied();
                if result != base {
                    self.addresses.remove(result);
                }
                let width = c.ty.types.size_of(*result_ty);
                if width > 0
                    && let Some(a) = base_address
                    && let Some(projected) = a.project(*offset as i128, width, self.extent(a.root))
                {
                    self.addresses.insert(*result, projected);
                }
            }
            Event::Offset {
                value,
                source,
                stride,
                offset,
                subscript,
                ..
            } => {
                let source_address = self.addresses.get(source).copied();
                if value != source {
                    self.addresses.remove(value);
                    self.values.remove(value);
                }
                let delta = offset.or_else(|| {
                    subscript.and_then(|v| match self.values.get(&v) {
                        Some(Expr::Constant(k)) => Some(*k as i64 as i128),
                        _ => None,
                    })
                });
                if let (Some(a), Some(delta)) = (source_address, delta)
                    && let Some(delta) = delta.checked_mul(c.ty.types.size_of(*stride) as i128)
                    && let Some(projected) = a.project(delta, 1, self.extent(a.root))
                {
                    self.addresses.insert(*value, projected);
                }
            }
            Event::Copy {
                value,
                source,
            } => {
                self.values.insert(*value, self.value(*source));
                if let Some(a) = self.addresses.get(source).copied() {
                    self.addresses.insert(*value, a);
                }
            }
            Event::Load {
                place,
                value,
                ty,
                ..
            } => {
                if scalar(c, *ty) {
                    let slot = Slot {
                        offset: 0,
                        width: c.ty.types.size_of(*ty),
                        ty: *ty,
                    };
                    self.values.insert(*value, self.get(*place, slot));
                } else {
                    self.copy_slots(*value, *place, Some(c.ty.types.size_of(*ty)));
                }
            }

            _ => {}
        }
    }
}
