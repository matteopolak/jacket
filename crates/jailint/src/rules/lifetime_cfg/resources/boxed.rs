//! A local Any owns the compiler's boxed spill through that holder's scope.
//! A transient call argument still has statement lifetime; a global/heap store is no holder.
use super::{Cx, Event, HashMap, LifetimeBody, Places, ScopeId, TypeId, Val, inside};

pub(super) fn holders(
    cx: &Cx,
    body: &LifetimeBody,
    places: &Places,
    locals: &HashMap<Val, ScopeId>,
) -> HashMap<Val, ScopeId> {
    let mut holders = HashMap::default();
    for event in body.blocks.iter().flat_map(|b| &b.events) {
        let Event::Store {
            place,
            value,
            ty: TypeId::ANY,
            ..
        } = *event
        else {
            continue;
        };
        let root = if let Some(p) = places.existing(place, TypeId::ANY) {
            if places.unresolved(p) {
                continue;
            }
            places.paths[p].0.0
        } else {
            place
        };
        let Some(&scope) = locals.get(&root) else {
            continue;
        };
        holders
            .entry(value)
            .and_modify(|old: &mut ScopeId| {
                let mut ancestor = Some(scope);
                while let Some(parent) = ancestor {
                    if inside(cx, *old, parent) {
                        *old = parent;
                        break;
                    }
                    ancestor = cx
                        .compiler
                        .ns
                        .scopes
                        .get(parent.0 as usize)
                        .and_then(|s| s.parent);
                }
            })
            .or_insert(scope);
    }
    holders
}

pub(super) fn origin(
    builder: &super::Builder<'_>,
    places: &mut Places,
    value: Val,
    place: Val,
    ty: TypeId,
    temporary: bool,
    span: super::Span,
) -> Option<super::Origin> {
    let p = places.place(place, ty);
    if temporary {
        Some(
            builder
                .boxed
                .get(&value)
                .copied()
                .map_or(super::Origin::Statement(span), super::Origin::Stack),
        )
    } else {
        builder
            .locals
            .get(&places.paths[p].0.0)
            .copied()
            .or_else(|| {
                builder
                    .parameters
                    .contains(&place)
                    .then_some(ScopeId(u32::MAX))
            })
            .filter(|_| !places.unresolved(p))
            .map(super::Origin::Stack)
    }
}
