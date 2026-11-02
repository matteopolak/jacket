//! Bounded identities for element selectors. A mutable or conditionally defined source
//! never becomes a constant merely because lowering recorded one assignment to it.
use super::Selector;
use crate::rules::lifetime_places::{Places, Projection};
use jaic::fxhash::{HashMap, HashSet};
use jaic::ide::lifetimes::{LifetimeBody, LifetimeEvent as Event, LifetimeOperand};
use jaic::ir::Val;
use jaic::sema::Compiler;
use jaic::types::{ArrayKind, TypeId, TypeKind};
use std::cell::RefCell;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(in crate::rules) struct Site {
    pub block: usize,
    pub event: usize,
}

#[derive(Clone, Copy, Debug)]
pub(in crate::rules) struct Position {
    pub region: usize,
    pub selector: Selector,
    pub element: TypeId,
    pub bounded: bool,
}

#[derive(Clone, Copy)]
enum Def {
    Load(usize, Site),
    Copy(Val),
    Index(usize, Val, Option<i128>, TypeId, Site),
    Offset(Val, Option<i128>, TypeId),
    Field(Val),
    Parameter(usize),
    Integer(i128),
    LoopIndex,
    Binary(jaic::ir::BinOp, Val, Val),
}

#[derive(Clone, Copy)]
enum Number {
    Constant(i128),
    Formal(usize),
    Token(Val),
    Count(usize, i128, Site),
    Unknown,
}

pub(in crate::rules) struct Facts<'a> {
    pub places: Places,
    compiler: &'a Compiler,
    body: &'a LifetimeBody,
    defs: HashMap<Val, Vec<Def>>,
    stores: HashMap<usize, Vec<(Val, Site)>>,
    counts: HashMap<usize, usize>,
    data: HashMap<usize, (usize, TypeId)>,
    exposed: HashSet<usize>,
    local: HashSet<Val>,
    descendant_writes: HashMap<usize, Vec<Site>>,
    numeric_cache: RefCell<HashMap<(Val, Site), Number>>,
}

impl<'a> Facts<'a> {
    pub fn new(compiler: &'a Compiler, body: &'a LifetimeBody) -> Self {
        let places = Places::new(compiler, body);
        let mut this = Self {
            places,
            compiler,
            body,
            defs: HashMap::default(),
            stores: HashMap::default(),
            counts: HashMap::default(),
            data: HashMap::default(),
            exposed: HashSet::default(),
            local: body.stack_places.iter().copied().collect(),
            descendant_writes: HashMap::default(),
            numeric_cache: RefCell::default(),
        };
        for (block, b) in body.blocks.iter().enumerate() {
            for (event, e) in b.events.iter().enumerate() {
                let site = Site {
                    block,
                    event,
                };
                match *e {
                    Event::Parameter {
                        incoming,
                        formal,
                        ..
                    } => this.def(incoming, Def::Parameter(formal)),
                    Event::Load {
                        place,
                        value,
                        ty,
                        ..
                    } => {
                        let p = this.places.place(place, ty);
                        if value != place || compiler.ir_ty(ty).is_some() {
                            this.def(value, Def::Load(p, site));
                        }
                    }
                    Event::Store {
                        place,
                        value,
                        ty,
                        constant,
                        ..
                    } => {
                        let p = this.places.place(place, ty);
                        this.stores.entry(p).or_default().push((value, site));
                        if let Some(n) = constant {
                            this.def(value, Def::Integer(n as i64 as i128));
                        }
                    }
                    Event::Iteration {
                        index, ..
                    } => {
                        let p = this.places.place(index, TypeId::S64);
                        this.exposed.insert(p);
                        this.def(index, Def::LoopIndex);
                    }
                    Event::Copy {
                        value,
                        source,
                    } => this.def(value, Def::Copy(source)),
                    Event::Offset {
                        value,
                        source,
                        stride,
                        offset,
                        ..
                    } => this.def(value, Def::Offset(source, offset, stride)),
                    Event::IntegerBinary {
                        value,
                        left,
                        right,
                        op,
                        left_constant,
                        right_constant,
                    } => {
                        if let Some(n) = left_constant {
                            this.def(left, Def::Integer(n as i64 as i128));
                        }
                        if let Some(n) = right_constant {
                            this.def(right, Def::Integer(n as i64 as i128));
                        }
                        this.def(value, Def::Binary(op, left, right));
                    }
                    Event::Index {
                        base,
                        result,
                        base_ty,
                        result_ty,
                        index,
                        subscript,
                    } => {
                        let p = this.places.place(base, base_ty);
                        this.def(result, Def::Index(p, subscript, index, result_ty, site));
                    }
                    Event::Field {
                        base: LifetimeOperand::Place(base),
                        result: LifetimeOperand::Place(result),
                        base_ty,
                        result_ty,
                        name,
                        descriptor,
                        ..
                    } => {
                        let p = this.places.place(result, result_ty);
                        if descriptor && name.as_str() == "count" {
                            if let Some(parent) = this.places.descriptor_fields.get(&p) {
                                this.counts.insert(p, *parent);
                            }
                        } else if descriptor && name.as_str() == "data" {
                            if let Some(&(parent, _)) = this.places.data.get(&p) {
                                let ty =
                                    compiler.ty.types.pointee(result_ty).unwrap_or(TypeId::VOID);
                                this.data.insert(p, (parent, ty));
                            }
                        } else {
                            let _ = base_ty;
                            this.def(result, Def::Field(base));
                        }
                    }
                    Event::Address {
                        place,
                        ty,
                        ..
                    } => {
                        let p = this.places.place(place, ty);
                        this.exposed.insert(p);
                    }
                    _ => {}
                }
            }
        }
        for (&p, writes) in &this.stores {
            for ancestor in this.places.ancestors(p) {
                this.descendant_writes
                    .entry(ancestor)
                    .or_default()
                    .extend(writes.iter().map(|(_, site)| *site));
            }
        }
        this
    }

    fn def(&mut self, value: Val, def: Def) {
        self.defs.entry(value).or_default().push(def);
    }

    fn before(def: Site, use_: Site) -> bool {
        (def.block == use_.block && def.event < use_.event) || (def.block == 0 && use_.block != 0)
    }

    fn single_store(&self, p: usize, at: Site) -> Option<Val> {
        let [(value, site)] = self.stores.get(&p)?.as_slice() else {
            return None;
        };
        if !Self::before(*site, at) || self.exposed.iter().any(|&e| self.places.contains(e, p)) {
            return None;
        }
        if self.descendant_writes.get(&p).is_some_and(|writes| {
            writes
                .iter()
                .any(|s| Self::before(*site, *s) && Self::before(*s, at))
        }) {
            return None;
        }
        for ancestor in self.places.ancestors(p) {
            if self.stores.get(&ancestor).is_some_and(|writes| {
                writes
                    .iter()
                    .any(|(_, s)| Self::before(*site, *s) && Self::before(*s, at))
            }) {
                return None;
            }
        }
        Some(*value)
    }

    fn number(&self, value: Val, at: Site, seen: &mut HashSet<Val>) -> Number {
        if let Some(&cached) = self.numeric_cache.borrow().get(&(value, at)) {
            return cached;
        }
        if seen.len() >= 24 || !seen.insert(value) {
            return Number::Unknown;
        }
        let result = match self.defs.get(&value).map(Vec::as_slice) {
            Some(defs) if defs.iter().all(|d| matches!(d, Def::Integer(_))) => match defs.first() {
                Some(Def::Integer(n)) => Number::Constant(*n),
                _ => Number::Unknown,
            },
            Some([Def::Parameter(p)]) => Number::Formal(*p),
            Some([Def::Copy(v)]) => self.number(*v, at, seen),
            Some([Def::Load(p, site)]) => {
                if let Some(&region) = self.counts.get(p) {
                    Number::Count(region, 0, *site)
                } else if let Some(source) = self.single_store(*p, *site) {
                    self.number(source, *site, seen)
                } else {
                    Number::Unknown
                }
            }
            Some([Def::Binary(op, a, b)]) => {
                let a = self.number(*a, at, &mut seen.clone());
                let b = self.number(*b, at, &mut seen.clone());
                match (op, a, b) {
                    (jaic::ir::BinOp::Add, Number::Constant(a), Number::Constant(b)) => {
                        a.checked_add(b).map_or(Number::Unknown, Number::Constant)
                    }
                    (jaic::ir::BinOp::Sub, Number::Constant(a), Number::Constant(b)) => {
                        a.checked_sub(b).map_or(Number::Unknown, Number::Constant)
                    }
                    (jaic::ir::BinOp::Add, Number::Count(p, n, s), Number::Constant(b)) => n
                        .checked_add(b)
                        .map_or(Number::Unknown, |n| Number::Count(p, n, s)),
                    (jaic::ir::BinOp::Sub, Number::Count(p, n, s), Number::Constant(b)) => n
                        .checked_sub(b)
                        .map_or(Number::Unknown, |n| Number::Count(p, n, s)),
                    _ => Number::Unknown,
                }
            }
            None => {
                let p = self
                    .places
                    .paths
                    .iter()
                    .position(|((v, _), path)| *v == value && path.is_empty());
                if let Some(p) = p
                    && let Some(source) = self.single_store(p, at)
                {
                    self.number(source, at, seen)
                } else if self.local.contains(&value)
                    && p.is_some_and(|p| self.stores.contains_key(&p))
                {
                    Number::Unknown
                } else {
                    Number::Token(value)
                }
            }
            _ => Number::Unknown,
        };
        seen.remove(&value);
        self.numeric_cache.borrow_mut().insert((value, at), result);
        result
    }

    pub fn selector(&self, value: Val, region: Option<usize>, at: Site) -> Selector {
        match self.number(value, at, &mut HashSet::default()) {
            Number::Constant(n) => Selector::Constant(n),
            Number::Formal(n) => Selector::Formal(n),
            Number::Token(v) => Selector::Local(v),
            Number::Count(p, delta, site)
                if region == Some(p)
                    && site.block == at.block
                    && self.body.blocks[at.block].events[site.event + 1..at.event]
                        .iter()
                        .all(|e| {
                            !matches!(
                                e,
                                Event::Store { .. }
                                    | Event::Call { .. }
                                    | Event::IndirectCall { .. }
                            )
                        }) =>
            {
                if delta == -1 {
                    Selector::Last
                } else {
                    Selector::CountDelta(delta)
                }
            }
            _ => Selector::Unknown,
        }
    }

    pub fn position(&self, value: Val, _at: Site) -> Option<Position> {
        self.position_inner(value, &mut HashSet::default())
    }

    fn position_inner(&self, value: Val, seen: &mut HashSet<Val>) -> Option<Position> {
        if seen.len() >= 24 || !seen.insert(value) {
            return None;
        }
        let result = match self.defs.get(&value)?.as_slice() {
            [Def::Index(region, index, constant, element, site)] => Some(Position {
                region: *region,
                selector: constant.map_or_else(
                    || self.selector(*index, Some(*region), *site),
                    Selector::Constant,
                ),
                element: *element,
                bounded: true,
            }),
            [Def::Field(base) | Def::Copy(base)] => self.position_inner(*base, seen),
            [Def::Offset(base, offset, stride)] => {
                let mut p = self.position_inner(*base, seen)?;
                p.selector = match (p.selector, offset, p.element == *stride) {
                    (Selector::Constant(n), Some(delta), true) => n
                        .checked_add(*delta)
                        .map_or(Selector::Unknown, Selector::Constant),
                    _ => Selector::Unknown,
                };
                p.bounded = matches!(p.selector,Selector::Constant(n) if self.fixed_length(p.region).is_some_and(|len| n>=0 && n<len));
                Some(p)
            }
            [Def::Load(place, site)] => {
                if let Some(&(region, element)) = self.data.get(place) {
                    Some(Position {
                        region,
                        selector: Selector::Constant(0),
                        element,
                        bounded: false,
                    })
                } else {
                    self.position_inner(self.single_store(*place, *site)?, seen)
                }
            }
            _ => None,
        };
        seen.remove(&value);
        result
    }

    pub fn fixed_length(&self, region: usize) -> Option<i128> {
        match self
            .compiler
            .ty
            .types
            .kind(*self.places.types.get(&region)?)
        {
            TypeKind::Array {
                kind: ArrayKind::Fixed(n),
                ..
            } => Some(*n as i128),
            _ => None,
        }
    }

    pub fn place_of(&mut self, value: Val, ty: TypeId) -> usize {
        self.places.place(value, ty)
    }

    pub fn count_parent(&self, place: usize) -> Option<usize> {
        self.counts.get(&place).copied()
    }

    pub fn is_count(&self, place: usize) -> bool {
        matches!(self.places.paths[place].1.last(),Some(Projection::Field(name)) if name.as_str()=="count")
            && self.counts.contains_key(&place)
    }
}
