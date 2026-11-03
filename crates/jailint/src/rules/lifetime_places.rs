//! Canonical projections. Descriptor storage and an element's address are distinct:
//! projecting `.inner` through an outer element does not make inner storage part of outer storage.
use jaic::fxhash::{HashMap, HashSet};
use jaic::ide::lifetimes::{LifetimeBody, LifetimeEvent as Event, LifetimeOperand};
use jaic::intern::Sym;
use jaic::ir::Val;
use jaic::types::TypeId;

mod context;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum Projection {
    Field(Sym),
    Element(i128),
    DynamicElement(Val),
    Deref,
}

#[derive(Default)]
pub(super) struct Places {
    pub paths: Vec<((Val, TypeId), Vec<Projection>)>,
    ids: HashMap<((Val, TypeId), Vec<Projection>), usize>,
    values: HashMap<(Val, TypeId), usize>,
    pub types: HashMap<usize, TypeId>,
    pub indirect: HashSet<usize>,
    /// A built-in descriptor's data field, not a user field named data.
    pub data: HashMap<usize, (usize, jaic::source::Span)>,
    pub descriptor_fields: HashMap<usize, usize>,
    children: HashMap<usize, Vec<usize>>,
    parents: Vec<Option<usize>>,
    stable_parameters: jaic::fxhash::HashSet<Val>,
}

impl Places {
    pub fn new(compiler: &jaic::sema::Compiler, body: &LifetimeBody) -> Self {
        let mut this = Self::default();
        // Lowering emits each projection after its base. Definitions precede their uses even
        // when a later block is checked before an earlier block's cleanup is filled in.
        let events: Vec<_> = body.blocks.iter().flat_map(|b| &b.events).collect();
        let contexts = context::aliases(body);
        for event in &events {
            let typed = match **event {
                Event::Expression {
                    operand: LifetimeOperand::Place(v),
                    ty,
                    ..
                } => Some((v, ty)),
                Event::Field {
                    base: LifetimeOperand::Place(v),
                    base_ty,
                    ..
                } => Some((v, base_ty)),
                _ => None,
            };
            if let Some((value, ty)) = typed
                && let Some(&source) = contexts.get(&value)
            {
                let id = this.place(source, ty);
                this.values.insert((value, ty), id);
            }
        }
        let mut globals = HashMap::default();
        for event in &events {
            if let Event::Expression {
                operand: LifetimeOperand::Place(v),
                ty,
                entity: Some(entity),
                ..
            } = **event
                && let jaic::sema::scope::EntityState::Done(jaic::sema::scope::Resolved::Global {
                    storage,
                    ..
                }) = compiler.entity(entity).state
            {
                let key = match storage {
                    jaic::ir::Storage::Data(id) => (false, id.0, ty),
                    jaic::ir::Storage::Foreign(id) => (true, id.0, ty),
                };
                let id = *globals.entry(key).or_insert_with(|| this.place(v, ty));
                this.values.insert((v, ty), id);
            }
        }
        // A single, unexposed pointer slot is an immutable SSA source for this body.
        // Repeated loads must project from the same holder, including pointer parameters.
        let mut pointer_stores = HashMap::<Val, Vec<(Val, usize, usize)>>::default();
        let exposed_slots: jaic::fxhash::HashSet<_> = events
            .iter()
            .filter_map(|e| match **e {
                Event::Address {
                    place, ..
                } => Some(place),
                _ => None,
            })
            .collect();
        for (block, b) in body.blocks.iter().enumerate() {
            for (ordinal, event) in b.events.iter().enumerate() {
                if let Event::Store {
                    place,
                    value,
                    ty,
                    ..
                } = *event
                    && compiler.ty.types.is_pointer(ty)
                {
                    pointer_stores
                        .entry(place)
                        .or_default()
                        .push((value, block, ordinal));
                }
            }
        }
        for (block, b) in body.blocks.iter().enumerate() {
            for (ordinal, event) in b.events.iter().enumerate() {
                if let Event::Parameter {
                    incoming,
                    place,
                    ty,
                    ..
                } = *event
                    && compiler.ty.types.is_pointer(ty)
                    && !exposed_slots.contains(&place)
                    && pointer_stores.get(&place).is_some_and(|s| s.len() == 1)
                {
                    this.stable_parameters.insert(incoming);
                    let root = this.place(incoming, ty);
                    this.values.insert((place, ty), root);
                }
                if let Event::Load {
                    place,
                    value,
                    ty,
                    ..
                } = *event
                    && compiler.ty.types.is_pointer(ty)
                    && !exposed_slots.contains(&place)
                    && let Some([(source, from, at)]) =
                        pointer_stores.get(&place).map(Vec::as_slice)
                    && ((*from == block && *at < ordinal) || (*from == 0 && block != 0))
                {
                    let source = this.place(*source, ty);
                    this.values.insert((value, ty), source);
                }
            }
        }
        for event in &events {
            match **event {
                Event::Field {
                    base,
                    result: LifetimeOperand::Place(result),
                    name,
                    indirect,
                    descriptor,
                    base_ty,
                    result_ty,
                    span,
                } => {
                    let (LifetimeOperand::Place(base) | LifetimeOperand::Value(base)) = base else {
                        continue;
                    };
                    let mut parent = this.place(base, base_ty);
                    if indirect {
                        parent = this.project(parent, Projection::Deref);
                    }
                    this.types.insert(
                        parent,
                        if indirect {
                            compiler.ty.types.pointee(base_ty).unwrap()
                        } else {
                            base_ty
                        },
                    );
                    let field = this.project(parent, Projection::Field(name));
                    this.values.insert((result, result_ty), field);
                    this.types.insert(field, result_ty);
                    if descriptor {
                        this.descriptor_fields.insert(field, parent);
                        if name.as_str() == "data" {
                            this.data.insert(field, (parent, span));
                        }
                    }
                }
                Event::Index {
                    base,
                    result,
                    base_ty,
                    result_ty,
                    index,
                    ..
                } => {
                    let parent = this.place(base, base_ty);
                    let projection =
                        index.map_or(Projection::DynamicElement(result), Projection::Element);
                    let element = this.project(parent, projection);
                    this.values.insert((result, result_ty), element);
                    this.types.insert(element, result_ty);
                }
                _ => {}
            }
        }
        for event in events {
            if let Event::Indirect {
                place,
                ty,
            } = *event
            {
                let id = this.place(place, ty);
                this.indirect.insert(id);
            }
            if let Event::Expression {
                operand: LifetimeOperand::Place(v),
                ty,
                ..
            } = *event
            {
                let id = this.place(v, ty);
                this.types.insert(id, ty);
            }
        }
        this
    }

    pub fn place(&mut self, value: Val, ty: TypeId) -> usize {
        if let Some(&id) = self.values.get(&(value, ty)) {
            return id;
        }
        let id = self.intern((value, ty), Vec::new());
        self.values.insert((value, ty), id);
        id
    }

    fn intern(&mut self, root: (Val, TypeId), path: Vec<Projection>) -> usize {
        if let Some(&id) = self.ids.get(&(root, path.clone())) {
            return id;
        }
        let parent = (!path.is_empty()).then(|| self.intern(root, path[..path.len() - 1].to_vec()));
        let id = self.paths.len();
        self.paths.push((root, path.clone()));
        self.parents.push(parent);
        self.ids.insert((root, path), id);
        if let Some(parent) = parent {
            self.children.entry(parent).or_default().push(id);
        }
        id
    }

    pub fn project(&mut self, base: usize, projection: Projection) -> usize {
        let (root, mut path) = self.paths[base].clone();
        path.push(projection);
        self.intern(root, path)
    }

    pub fn contains(&self, parent: usize, child: usize) -> bool {
        let (a, pa) = &self.paths[parent];
        let (b, pb) = &self.paths[child];
        a == b && pb.starts_with(pa)
    }

    pub fn import(&mut self, other: &Self, id: usize) -> usize {
        let (root, path) = &other.paths[id];
        let target = self.intern(*root, path.clone());
        if let Some(&ty) = other.types.get(&id) {
            self.types.insert(target, ty);
        }
        if let Some(&(owner, span)) = other.data.get(&id) {
            let owner = self.import(other, owner);
            self.data.insert(target, (owner, span));
        }
        target
    }

    pub fn unresolved(&self, place: usize) -> bool {
        self.paths[place].1.contains(&Projection::Deref)
            || self.indirect.iter().any(|&base| self.contains(base, place))
    }

    pub fn stable_projection(&self, place: usize) -> bool {
        let ((root, _), path) = &self.paths[place];
        self.stable_parameters.contains(root)
            && path.first() == Some(&Projection::Deref)
            && path.iter().filter(|p| **p == Projection::Deref).count() == 1
            && !self.indirect.iter().any(|&base| self.contains(base, place))
    }

    pub fn ancestors(&self, place: usize) -> impl Iterator<Item = usize> + '_ {
        std::iter::successors(self.parents[place], |&p| self.parents[p])
    }

    pub fn existing(&self, value: Val, ty: TypeId) -> Option<usize> {
        self.values.get(&(value, ty)).copied()
    }

    pub fn lookup(&self, other: &Self, place: usize) -> Option<usize> {
        self.ids.get(&other.paths[place]).copied()
    }

    pub fn projected(&self, parent: usize, projection: Projection) -> Option<usize> {
        let (root, path) = &self.paths[parent];
        let mut path = path.clone();
        path.push(projection);
        self.ids.get(&(*root, path)).copied()
    }

    pub fn descendants(&self, parent: usize) -> Vec<usize> {
        let mut result = Vec::new();
        let mut queue = self.children.get(&parent).cloned().unwrap_or_default();
        while let Some(id) = queue.pop() {
            result.push(id);
            if let Some(children) = self.children.get(&id) {
                queue.extend(children);
            }
        }
        result
    }

    pub fn copied_fields(&mut self, target: usize, source: usize) -> Vec<(usize, usize)> {
        let mut pairs = Vec::new();
        // Either side can introduce the field: a returned aggregate's output buffer
        // need not have an explicit member expression in this body.
        for (from, to, reverse) in [(target, source, false), (source, target, true)] {
            let depth = self.paths[from].1.len();
            for id in self.descendants(from) {
                let suffix = self.paths[id].1[depth..].to_vec();
                let mut other = to;
                for projection in suffix {
                    other = self.project(other, projection);
                }
                pairs.push(if reverse {
                    (other, id)
                } else {
                    (id, other)
                });
            }
        }
        pairs.sort_unstable();
        pairs.dedup();
        pairs
    }
}
