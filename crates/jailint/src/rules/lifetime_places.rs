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
    /// Body-local checked byte probe; never exported as a symbolic field path.
    Inline {
        offset: u64,
        ty: TypeId,
    },
    Element(i128),
    DynamicElement(Val),
    Deref,
}

#[derive(Clone, Default)]
pub(super) struct Places {
    pub paths: Vec<((Val, TypeId), Vec<Projection>)>,
    root_ids: HashMap<(Val, TypeId), usize>,
    child_ids: HashMap<(usize, Projection), usize>,
    values: HashMap<(Val, TypeId), usize>,
    pub types: HashMap<usize, TypeId>,
    pub indirect: HashSet<usize>,
    /// A built-in descriptor's data field, not a user field named data.
    pub data: HashMap<usize, (usize, jaic::source::Span)>,
    pub descriptor_fields: HashMap<usize, usize>,
    children: HashMap<usize, Vec<usize>>,
    parents: Vec<Option<usize>>,
    roots: Vec<usize>,
    stable_parameters: jaic::fxhash::HashSet<Val>,
}

impl Places {
    /// Clone reservation includes nested projection vectors and all owned maps.
    pub(super) fn clone_work(&self) -> Option<usize> {
        let sizes = [
            self.paths.capacity(),
            self.root_ids.capacity(),
            self.child_ids.capacity(),
            self.values.capacity(),
            self.types.capacity(),
            self.indirect.capacity(),
            self.data.capacity(),
            self.descriptor_fields.capacity(),
            self.children.capacity(),
            self.parents.capacity(),
            self.roots.capacity(),
            self.stable_parameters.capacity(),
        ];
        sizes
            .into_iter()
            .chain(self.paths.iter().map(|(_, path)| path.capacity()))
            .chain(self.children.values().map(Vec::capacity))
            .try_fold(0usize, usize::checked_add)
    }

    pub fn new(compiler: &jaic::sema::Compiler, body: &LifetimeBody) -> Self {
        let mut this = Self::default();
        // Lowering emits each projection after its base. Definitions precede their uses even
        // when a later block is checked before an earlier block's cleanup is filled in.
        let events: Vec<_> = body.blocks.iter().flat_map(|b| &b.events).collect();
        let contexts = context::aliases(body);
        let slots: HashMap<_, _> = body.slot_aliases.iter().copied().collect();
        for event in &events {
            let typed = match **event {
                Event::ContextSet {place,ty,..}
                | Event::ContextRead {place,ty,..}
                | Event::Store {
                    place,
                    ty,
                    ..
                }
                | Event::Load {
                    place,
                    ty,
                    ..
                }
                | Event::Initialize {
                    place,
                    ty,
                    ..
                }
                | Event::Address {
                    place,
                    ty,
                    ..
                } => Some((place, ty)),
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
                && let Some(&canonical) = slots.get(&value)
            {
                let id = this.place(canonical, ty);
                this.values.insert((value, ty), id);
            }
        }

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
        let id = self.intern_root((value, ty));
        self.values.insert((value, ty), id);
        id
    }

    fn intern_root(&mut self, root: (Val, TypeId)) -> usize {
        if let Some(&id) = self.root_ids.get(&root) {
            return id;
        }
        let id = self.paths.len();
        self.paths.push((root, Vec::new()));
        self.parents.push(None);
        self.roots.push(id);
        self.root_ids.insert(root, id);
        id
    }

    pub fn project(&mut self, base: usize, projection: Projection) -> usize {
        if let Some(&id) = self.child_ids.get(&(base, projection)) {
            return id;
        }
        let (root, mut path) = self.paths[base].clone();
        path.push(projection);
        let id = self.paths.len();
        self.paths.push((root, path));
        self.parents.push(Some(base));
        self.roots.push(self.roots[base]);
        self.child_ids.insert((base, projection), id);
        self.children.entry(base).or_default().push(id);
        id
    }

    /// Register a typed probe under an existing original stack root. For example,
    /// a descriptor count probe need not invent an SSA load or a named field.
    pub(super) fn inline_probe(
        &mut self,
        compiler: &jaic::sema::Compiler,
        roots: &super::lifetime_facts::StackRoots,
        root: usize,
        offset: u64,
        ty: TypeId,
    ) -> Option<usize> {
        use super::lifetime_facts::ByteRange;
        let ((value, root_ty), path) = self.paths.get(root)?;
        if !path.is_empty() {
            return None;
        }
        let extent = roots.extent(roots.canonical(*value)).flatten();
        let width = compiler.ty.types.size_of(ty);
        if !(ByteRange {
            offset,
            width,
        })
        .within(extent)
            || !(ByteRange {
                offset,
                width,
            })
            .within(Some(compiler.ty.types.size_of(*root_ty)))
        {
            return None;
        }
        let place = self.project(
            root,
            Projection::Inline {
                offset,
                ty,
            },
        );
        self.types.insert(place, ty);
        Some(place)
    }

    /// The selected projection type, or an unprojected root's recorded type.
    /// Missing projection metadata never falls back to its enclosing root type.
    pub fn selected_type(&self, place: usize) -> Option<TypeId> {
        self.types.get(&place).copied().or_else(|| {
            let ((_, ty), path) = self.paths.get(place)?;
            path.is_empty().then_some(*ty)
        })
    }

    /// The place with no projections that `place` hangs off.
    pub fn root_of(&self, place: usize) -> usize {
        self.roots[place]
    }

    pub fn contains(&self, parent: usize, child: usize) -> bool {
        let (a, pa) = &self.paths[parent];
        let (b, pb) = &self.paths[child];
        a == b && pb.starts_with(pa)
    }

    pub fn import(&mut self, other: &Self, id: usize) -> usize {
        let (root, path) = &other.paths[id];
        let mut target = self.intern_root(*root);
        for &projection in path {
            target = self.project(target, projection);
        }
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
        self.paths[place].1.contains(&Projection::Deref) || self.under_indirect(place)
    }

    fn under_indirect(&self, place: usize) -> bool {
        !self.indirect.is_empty()
            && std::iter::once(place)
                .chain(self.ancestors(place))
                .any(|base| self.indirect.contains(&base))
    }

    pub fn stable_projection(&self, place: usize) -> bool {
        let ((root, _), path) = &self.paths[place];
        self.stable_parameters.contains(root)
            && path.first() == Some(&Projection::Deref)
            && path.iter().filter(|p| **p == Projection::Deref).count() == 1
            && !self.under_indirect(place)
    }

    pub fn ancestors(&self, place: usize) -> impl Iterator<Item = usize> + '_ {
        std::iter::successors(self.parents[place], |&p| self.parents[p])
    }

    pub fn existing(&self, value: Val, ty: TypeId) -> Option<usize> {
        self.values.get(&(value, ty)).copied()
    }

    pub fn lookup(&self, other: &Self, place: usize) -> Option<usize> {
        let (root, path) = &other.paths[place];
        path.iter()
            .try_fold(*self.root_ids.get(root)?, |at, projection| {
                self.child_ids.get(&(at, *projection)).copied()
            })
    }

    pub fn projected(&self, parent: usize, projection: Projection) -> Option<usize> {
        self.child_ids.get(&(parent, projection)).copied()
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

impl Places {
    /// Prove two direct field slots occupy different bytes of the same object.
    /// Following a pointer after the differing fields abandons that proof: the
    /// separate pointer slots can still point into the same allocation.
    pub(super) fn disjoint_paths(
        compiler: &jaic::sema::Compiler,
        mut ty: TypeId,
        a: &[Projection],
        b: &[Projection],
    ) -> bool {
        for (index, (a_step, b_step)) in a.iter().zip(b).enumerate() {
            match (a_step, b_step) {
                (Projection::Field(a_name), Projection::Field(b_name)) => {
                    let Some(st) = compiler.ty.types.as_struct(ty) else {
                        return false;
                    };
                    let info = compiler.ty.types.struct_info(st);
                    let Some(a_field) = info.fields.iter().find(|f| f.name == Some(*a_name)) else {
                        return false;
                    };
                    let Some(b_field) = info.fields.iter().find(|f| f.name == Some(*b_name)) else {
                        return false;
                    };
                    if a_name != b_name {
                        // An indexed view/pointer also leaves inline storage even
                        // without an explicit Deref event. Until a typed bounded
                        // fixed-array suffix is proved, only field suffixes stay inline.
                        if !a[index + 1..]
                            .iter()
                            .chain(&b[index + 1..])
                            .all(|p| matches!(p, Projection::Field(_)))
                        {
                            return false;
                        }
                        let a_size = compiler.ty.types.size_of(a_field.ty);
                        let b_size = compiler.ty.types.size_of(b_field.ty);
                        return a_field
                            .offset
                            .checked_add(a_size)
                            .is_some_and(|end| end <= b_field.offset)
                            || b_field
                                .offset
                                .checked_add(b_size)
                                .is_some_and(|end| end <= a_field.offset);
                    }
                    ty = a_field.ty;
                }
                (Projection::Deref, Projection::Deref) => {
                    let Some(inner) = compiler.ty.types.pointee(ty) else {
                        return false;
                    };
                    ty = inner;
                }
                _ => return false,
            }
        }
        false
    }

    pub(super) fn disjoint_slots(
        &self,
        compiler: &jaic::sema::Compiler,
        a: usize,
        b: usize,
    ) -> bool {
        let (a_root, a_path) = &self.paths[a];
        let (b_root, b_path) = &self.paths[b];
        a_root == b_root && Self::disjoint_paths(compiler, a_root.1, a_path, b_path)
    }
}
