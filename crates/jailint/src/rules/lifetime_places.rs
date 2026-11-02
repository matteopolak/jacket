//! Canonical projections. Descriptor storage and an element's address are distinct:
//! projecting `.inner` through an outer element does not make inner storage part of outer storage.
use jaic::fxhash::{HashMap, HashSet};
use jaic::ide::lifetimes::{LifetimeBody, LifetimeEvent as Event, LifetimeOperand};
use jaic::intern::Sym;
use jaic::ir::Val;
use jaic::types::TypeId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum Projection {
    Field(Sym),
    Element(i128),
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
}

impl Places {
    pub fn new(compiler: &jaic::sema::Compiler, body: &LifetimeBody) -> Self {
        let mut this = Self::default();
        // Lowering emits each projection after its base. Definitions precede their uses even
        // when a later block is checked before an earlier block's cleanup is filled in.
        let events: Vec<_> = body.blocks.iter().flat_map(|b| &b.events).collect();
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
                    index: Some(index),
                } => {
                    let parent = this.place(base, base_ty);
                    let element = this.project(parent, Projection::Element(index));
                    this.values.insert((result, result_ty), element);
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
        let id = self.paths.len();
        self.paths.push((root, path.clone()));
        self.ids.insert((root, path), id);
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

    pub fn unresolved(&self, place: usize) -> bool {
        self.paths[place].1.contains(&Projection::Deref)
            || self.indirect.iter().any(|&base| self.contains(base, place))
    }

    pub fn copied_fields(&mut self, target: usize, source: usize) -> Vec<(usize, usize)> {
        let n = self.paths.len();
        let depth = self.paths[target].1.len();
        let mut pairs = Vec::new();
        for id in 0..n {
            if id != target && self.contains(target, id) {
                let suffix = self.paths[id].1[depth..].to_vec();
                let mut other = source;
                for projection in suffix {
                    other = self.project(other, projection);
                }
                pairs.push((id, other));
            }
        }
        pairs
    }
}
