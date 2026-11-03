//! An aggregate's storage fields can have different return origins.
use super::{
    Def, Event, HashMap, HashSet, LifetimeArgument, Path, ProcId, Projection, Resolver, Root,
    Selector, Step, Summary, TypeId, TypeKind, Val, value,
};

impl Resolver<'_> {
    pub(super) fn finish_definition_index(&mut self) {
        let stack: HashSet<_> = self.body.stack_places.iter().copied().collect();
        for (id, ((value, _), path)) in self.places.paths.iter().enumerate() {
            if path.is_empty() && stack.contains(value) {
                self.owned.insert(id);
            }
        }
        self.write_sources = self
            .writes
            .iter()
            .filter_map(|(&p, values)| {
                let first = *values.first()?;
                values.iter().all(|v| *v == first).then_some((p, first))
            })
            .collect();
        let empty_descriptors: HashMap<_, _> = self
            .body
            .blocks
            .iter()
            .flat_map(|b| &b.events)
            .filter_map(|e| match e {
                Event::EmptyDescriptor {
                    value,
                    ty,
                } => Some((*value, *ty)),
                _ => None,
            })
            .collect();
        self.nullable_sources = self
            .writes
            .iter()
            .filter_map(|(&place, sources)| {
                if self.write_sources.contains_key(&place) {
                    return None;
                }
                let mut selected = None;
                for &source in sources {
                    let empty = empty_descriptors.get(&source).is_some_and(|ty| {
                        self.places.existing(source, *ty).is_none_or(|p| {
                            !self.exposed.contains(&self.places.root_of(p))
                                && !self.writes.contains_key(&p)
                                && !self
                                    .places
                                    .descendants(p)
                                    .iter()
                                    .any(|p| self.writes.contains_key(p))
                        })
                    });
                    if empty {
                        continue;
                    }
                    if selected.is_some_and(|old| old != source) {
                        return None;
                    }
                    selected = Some(source);
                }
                selected.map(|source| (place, source))
            })
            .collect();
    }

    pub(in crate::rules) fn storage_fields(
        &self,
        ty: TypeId,
        depth: usize,
    ) -> Vec<Vec<Projection>> {
        if depth > super::MAX_PATH {
            return Vec::new();
        }
        let Some(st) = self.compiler.ty.types.as_struct(ty) else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for field in self.compiler.ty.types.struct_info(st).fields.iter() {
            let Some(name) = field.name else {
                continue;
            };
            let head = Projection::Field(name);
            if self.compiler.ty.types.as_struct(field.ty).is_some() {
                for path in self.storage_fields(field.ty, depth + 1) {
                    out.push(std::iter::once(head).chain(path).collect());
                }
            } else if self.compiler.ir_ty(field.ty).is_none()
                || self.compiler.ty.types.is_pointer(field.ty)
            {
                out.push(vec![head]);
            }
        }
        out
    }

    fn return_definitions(&self) -> &(HashSet<usize>, HashSet<usize>) {
        self.return_definitions.get_or_init(|| {
            let doms = super::flow::return_dominators(self.body);
            let mut writes = HashSet::default();
            let mut initialized = HashSet::default();
            for event in self
                .body
                .blocks
                .iter()
                .enumerate()
                .filter(|(i, _)| doms[*i])
                .flat_map(|(_, b)| &b.events)
            {
                match event {
                    Event::Store {
                        place,
                        ty,
                        ..
                    } => {
                        if let Some(p) = self.places.existing(*place, *ty) {
                            writes.insert(p);
                        }
                    }
                    Event::Initialize {
                        place,
                        ty,
                        ..
                    } => {
                        if let Some(p) = self.places.existing(*place, *ty) {
                            initialized.insert(p);
                        }
                    }
                    _ => {}
                }
            }
            (writes, initialized)
        })
    }

    fn return_definition(&self, place: usize) -> bool {
        if self.return_definitions().0.contains(&place) {
            return true;
        }
        if let Some(&proven) = self.return_cuts.borrow().get(&place) {
            return proven;
        }
        let mut queue = std::collections::VecDeque::from([0]);
        let mut seen = vec![false; self.body.blocks.len()];
        let mut proven = true;
        while let Some(i) = queue.pop_front() {
            if std::mem::replace(&mut seen[i], true) {
                continue;
            }
            let b = &self.body.blocks[i];
            if b.events.iter().any(|e| matches!(e, Event::Store{place:p,ty,..} if self.places.existing(*p,*ty) == Some(place))) {continue;}
            if b.normal_return {
                proven = false;
                break;
            }
            queue.extend(&b.successors);
        }
        self.return_cuts.borrow_mut().insert(place, proven);
        proven
    }

    // Empty defaults carry no storage. Conditional definitions may still describe
    // every nonempty result, as with a nullable allocation or a default error record.
    fn empty_field_default(&self, place: usize, projections: &[Projection]) -> bool {
        if !self.return_definitions().1.contains(&place)
            || self.exposed.contains(&self.places.root_of(place))
        {
            return false;
        }
        let mut ty = self.places.paths[place].0.1;
        let Some(image) = self.compiler.ty.default_images.get(&ty) else {
            return false;
        };
        let mut offset = 0u64;
        for projection in projections {
            let Projection::Field(name) = projection else {
                return false;
            };
            let Some(st) = self.compiler.ty.types.as_struct(ty) else {
                return false;
            };
            let Some(field) = self
                .compiler
                .ty
                .types
                .struct_info(st)
                .fields
                .iter()
                .find(|f| f.name == Some(*name))
            else {
                return false;
            };
            offset += field.offset;
            ty = field.ty;
        }
        let Some(image) = image else {
            return true;
        };
        let end = offset + self.compiler.ty.types.size_of(ty);
        image
            .bytes
            .get(offset as usize..end as usize)
            .is_some_and(|b| b.iter().all(|&v| v == 0))
            && !image
                .relocs
                .iter()
                .any(|r| r.offset >= offset && r.offset < end)
    }

    // Null descriptors contribute no storage to a return-origin join. Only fresh
    // constants with no writes or exposed address can be discarded this way.
    pub(super) fn return_source(&self, place: usize) -> Option<Val> {
        if let Some(&source) = self.write_sources.get(&place) {
            return Some(source);
        }
        if !self.return_mode.get() {
            return None;
        }
        self.nullable_sources.get(&place).copied()
    }

    pub(super) fn returned_field_origin(
        &self,
        value: Val,
        ty: TypeId,
        projections: &[Projection],
        all: &HashMap<ProcId, Summary>,
        depth: usize,
    ) -> Option<Path> {
        if depth > 24 {
            return None;
        }
        for def in self.values.get(&value).into_iter().flatten() {
            match def {
                Def::Call(..) => return self.field_origin(value, ty, projections, all, depth + 1),
                Def::Copy(source) if *source != value => {
                    return self.returned_field_origin(*source, ty, projections, all, depth + 1);
                }
                _ => {}
            }
        }
        let root = self.places.existing(value, ty)?;
        if let Some(source) = self.return_source(root)
            && source != value
            && (self.return_definition(root) || self.empty_field_default(root, projections))
        {
            return self.returned_field_origin(source, ty, projections, all, depth + 1);
        }
        None
    }

    pub(in crate::rules) fn field_origin(
        &self,
        value: Val,
        ty: TypeId,
        projections: &[Projection],
        all: &HashMap<ProcId, Summary>,
        depth: usize,
    ) -> Option<Path> {
        let key = (value, ty, projections.to_vec());
        if let Some(path) = self.field_origins.borrow().get(&key).cloned() {
            return Some(path);
        }
        let path = self.compute_field_origin(value, ty, projections, all, depth)?;
        self.field_origins.borrow_mut().insert(key, path.clone());
        Some(path)
    }

    fn compute_field_origin(
        &self,
        value: Val,
        ty: TypeId,
        projections: &[Projection],
        all: &HashMap<ProcId, Summary>,
        depth: usize,
    ) -> Option<Path> {
        if depth > 24 {
            return None;
        }
        let steps = projections
            .iter()
            .map(|p| match p {
                Projection::Field(n) => Some(Step::Field(*n)),
                Projection::Element(n) => Some(Step::Element(*n)),
                Projection::Deref => Some(Step::Deref),
                Projection::DynamicElement(_) => None,
            })
            .collect::<Option<Vec<_>>>()?;
        for def in self.values.get(&value).into_iter().flatten() {
            match def {
                Def::Call(callee, args, index) => {
                    let path = all
                        .get(callee)?
                        .return_fields
                        .get(&(*index, steps.clone()))?;
                    let mut path = path.clone();
                    path.root = self.allocation_root(&path.root, value, args)?;
                    return self.substitute(&path, args, all, &mut HashSet::default(), depth + 1);
                }
                Def::Copy(source) if *source != value => {
                    return self.field_origin(*source, ty, projections, all, depth + 1);
                }
                _ => {}
            }
        }
        let root = self.places.existing(value, ty)?;
        let field = projections
            .iter()
            .try_fold(root, |p, projection| self.places.projected(p, *projection));
        if let Some(sources) = field.and_then(|p| self.writes.get(&p)) {
            if !(self.return_definition(field?)
                || self.field_defined_before_uses(root, field?)
                || !self.writes.contains_key(&root) && self.empty_field_default(root, projections))
            {
                return None;
            }
            let mut origin = None;
            for source in sources {
                if self
                    .return_guard(*source, all, &mut HashSet::default())?
                    .is_some()
                {
                    return None;
                }
                let path = self.trace(*source, all, &mut HashSet::default(), depth + 1)?;
                if origin.as_ref().is_some_and(|old| old != &path) {
                    return None;
                }
                origin = Some(path);
            }
            return origin;
        }
        if let Some(source) = self.return_source(root)
            && source != value
            && (self.return_definition(root) || self.empty_field_default(root, projections))
        {
            return self.field_origin(source, ty, projections, all, depth + 1);
        }
        // Memory-class results share one physical return slot across branches.
        // Different headers can still select exactly the same storage origin.
        if self.return_mode.get()
            && !self.exposed.contains(&self.places.root_of(root))
            && self.return_definition(root)
        {
            let mut origin = None;
            for source in self.writes.get(&root)? {
                let path = self.field_origin(*source, ty, projections, all, depth + 1)?;
                if origin.as_ref().is_some_and(|old| old != &path) {
                    return None;
                }
                origin = Some(path);
            }
            return origin;
        }
        None
    }
}

impl Resolver<'_> {
    // An early return of a different aggregate does not require this aggregate's
    // fields to have been initialized. Cut paths at the field definition and check
    // the uses of this header, rather than every return in the procedure.
    fn field_defined_before_uses(&self, root: usize, field: usize) -> bool {
        if self.exposed.contains(&self.places.root_of(root)) {
            return false;
        }
        let (value, ty) = self.places.paths[root].0;
        let mut queue = std::collections::VecDeque::from([0]);
        let mut seen = vec![false; self.body.blocks.len()];
        while let Some(i) = queue.pop_front() {
            if std::mem::replace(&mut seen[i], true) {
                continue;
            }
            let block = &self.body.blocks[i];
            let mut defined = false;
            for event in &block.events {
                if matches!(event, Event::Store{place,ty,..} if self.places.existing(*place,*ty)==Some(field))
                {
                    defined = true;
                    break;
                }
                let used = match event {
                    Event::Return {
                        value: source,
                        ty: source_ty,
                        ..
                    }
                    | Event::Store {
                        value: source,
                        ty: source_ty,
                        ..
                    } => *source == value && *source_ty == ty,
                    Event::Call {
                        args, ..
                    }
                    | Event::IndirectCall {
                        args, ..
                    } => args.iter().any(|a| {
                        (a.ty == ty || self.compiler.ty.types.pointee(a.ty) == Some(ty))
                            && super::value(a) == Some(value)
                    }),
                    Event::Copy {
                        source, ..
                    } => *source == value,
                    _ => false,
                };
                if used {
                    return false;
                }
            }
            if !defined {
                queue.extend(&block.successors);
            }
        }
        true
    }
}

impl Resolver<'_> {
    pub(super) fn return_guard(
        &self,
        v: Val,
        all: &HashMap<ProcId, Summary>,
        visiting: &mut HashSet<Val>,
    ) -> Option<Option<(usize, u64)>> {
        if !visiting.insert(v) {
            return None;
        }
        if let Some(cast) = self.address_casts.get(&v).filter(|c| c.to_pointer) {
            let original =
                self.integer_address_source(cast.source, all, &mut HashSet::default(), 0)?;
            let guard = self.return_guard(original, all, visiting);
            visiting.remove(&v);
            return guard;
        }
        let mut guard = None;
        for def in self.values.get(&v)? {
            let next = match def {
                Def::Load(p, _) => {
                    if !self.places.paths[*p].1.is_empty() {
                        None
                    } else if let Some(writes) = self.writes.get(p) {
                        let [value] = writes.as_slice() else {
                            return None;
                        };
                        self.return_guard(*value, all, visiting)?
                    } else {
                        None
                    }
                }
                Def::Copy(value) => self.return_guard(*value, all, visiting)?,
                Def::Resize(_, args) => {
                    if args.first()?.constant == Some(0) {
                        None
                    } else {
                        self.mode_guard(args.first()?, u64::from(!self.allocation_mode.get()), all)?
                    }
                }
                Def::Call(callee, args, index) => {
                    let summary = all.get(callee)?;
                    let (_, condition) = if self.allocation_mode.get() {
                        summary
                            .allocations
                            .get(index)
                            .or_else(|| summary.returns.get(index))?
                    } else {
                        summary
                            .allocations
                            .get(index)
                            .filter(|(_, g)| {
                                g.is_some_and(|(f, c)| {
                                    args.iter().any(|a| a.formal == f && a.constant == Some(c))
                                })
                            })
                            .or_else(|| summary.returns.get(index))?
                    };
                    if let Some((formal, expected)) = condition {
                        self.mode_guard(args.iter().find(|a| a.formal == *formal)?, *expected, all)?
                    } else {
                        None
                    }
                }
                Def::Indirect(callee, args, index) => {
                    self.indirect_return(*callee, args, *index, all, &mut HashSet::default(), 0)?;
                    None
                }
                Def::Address(_) | Def::Borrow(_) | Def::Parameter(_) => None,
            };
            if guard.is_some() && next != guard {
                return None;
            }
            guard = next;
        }
        visiting.remove(&v);
        Some(guard)
    }

    pub(super) fn mode_guard(
        &self,
        mode: &LifetimeArgument,
        expected: u64,
        all: &HashMap<ProcId, Summary>,
    ) -> Option<Option<(usize, u64)>> {
        if let Some(actual) = mode.constant {
            return (actual == expected).then_some(None);
        }
        let path = self.trace(value(mode)?, all, &mut HashSet::default(), 0)?;
        let Root::Parameter(formal) = path.root else {
            return None;
        };
        path.steps.is_empty().then_some(Some((formal, expected)))
    }

    pub(super) fn result_carries_storage(&self, ty: TypeId, path: &Path) -> bool {
        if self.compiler.ir_ty(ty).is_none()
            || self.compiler.ty.types.is_pointer(ty)
            || matches!(self.compiler.ty.types.kind(ty), TypeKind::Proc(_))
        {
            return true;
        }
        let Root::Parameter(formal) = path.root else {
            return false;
        };
        path.steps.is_empty()
            && self.params.values().any(|(n, t)| {
                *n == formal
                    && self.compiler.ir_ty(*t).is_some()
                    && !self.compiler.ty.types.is_pointer(*t)
            })
    }
}

impl Resolver<'_> {
    pub(super) fn infer_return_origins(
        &mut self,
        next: &mut Summary,
        all: &HashMap<ProcId, Summary>,
        live: &super::flow::Live,
    ) {
        self.field_origins.borrow_mut().clear();
        self.return_mode.set(true);
        next.returns.clone_from(&self.return_contracts);
        next.return_fields.clear();
        next.adjusted_returns.clear();
        next.allocations.clear();
        let body = self.body;
        let mut returns = HashMap::default();
        let mut elements = HashMap::default();
        let mut fields: HashMap<usize, HashMap<Vec<super::Step>, super::Path>> = HashMap::default();
        for event in body
            .blocks
            .iter()
            .enumerate()
            .filter(|(i, _)| live.contains(*i))
            .flat_map(|(_, b)| &b.events)
        {
            if let Event::Return {
                value,
                index,
                ty,
                constant,
                ..
            } = event
            {
                if self.adjusted_pointer(*value, all, &mut HashSet::default(), 0) {
                    next.adjusted_returns.insert(*index);
                }
                let selected: HashMap<_, _> = self
                    .storage_fields(*ty, 0)
                    .into_iter()
                    .filter_map(|projections| {
                        let origin = self.field_origin(*value, *ty, &projections, all, 0)?;
                        if matches!(origin.root, Root::Local(_)) {
                            return None;
                        }
                        let steps = projections
                            .into_iter()
                            .map(|p| match p {
                                super::Projection::Field(n) => super::Step::Field(n),
                                _ => unreachable!(),
                            })
                            .collect();
                        Some((steps, origin))
                    })
                    .collect();
                if let Some(old) = fields.get_mut(index) {
                    old.retain(|path, origin| selected.get(path) == Some(origin));
                } else {
                    fields.insert(*index, selected);
                }
                if *constant == Some(0)
                    && (self.compiler.ty.types.is_pointer(*ty)
                        || matches!(self.compiler.ty.types.kind(*ty), TypeKind::Proc(_)))
                {
                    continue;
                }
                let element = self.element(*value, all).filter(|(p, s)| {
                    matches!(p.root, Root::Parameter(_))
                        && !matches!(s, Selector::Local(_) | Selector::Unknown)
                });
                elements
                    .entry(*index)
                    .and_modify(|old| {
                        if old != &element {
                            *old = None;
                        }
                    })
                    .or_insert(element);
                let path = self
                    .trace(*value, all, &mut HashSet::default(), 0)
                    .filter(|p| self.result_carries_storage(*ty, p))
                    .filter(|p| {
                        matches!(
                            p.root,
                            Root::Parameter(_)
                                | Root::Heap
                                | Root::CallerAllocator
                                | Root::AllocatorParameter(_)
                                | Root::Temporary
                                | Root::Watermark
                                | Root::Until(_)
                        )
                    })
                    .zip(self.return_guard(*value, all, &mut HashSet::default()));
                returns
                    .entry(*index)
                    .and_modify(|previous| {
                        if previous != &path {
                            *previous = None;
                        }
                    })
                    .or_insert(path);
            }
        }
        for (index, selected) in fields {
            for (path, origin) in selected {
                next.return_fields.insert((index, path), origin);
            }
        }
        for (index, element) in elements {
            if let Some(e) = element {
                next.elements.insert(index, e);
            }
        }
        for (index, path) in returns {
            if let Some(path) = path {
                if index == 0
                    && self
                        .compiler
                        .proc_has_note(self.body.procedure, "ValidUntilNextCall")
                {
                    continue;
                }
                next.returns.insert(index, path);
            }
        }
        self.field_origins.borrow_mut().clear();
        self.allocation_mode.set(true);
        let mut allocations = HashMap::default();
        for event in body
            .blocks
            .iter()
            .enumerate()
            .filter(|(i, _)| live.contains(*i))
            .flat_map(|(_, b)| &b.events)
        {
            if let Event::Return {
                value,
                index,
                ty,
                constant,
                ..
            } = event
            {
                if *constant == Some(0) && self.compiler.ty.types.is_pointer(*ty) {
                    continue;
                }
                let path = self
                    .trace(*value, all, &mut HashSet::default(), 0)
                    .filter(|p| {
                        !matches!(p.root, Root::Local(_)) && self.result_carries_storage(*ty, p)
                    })
                    .zip(self.return_guard(*value, all, &mut HashSet::default()));
                allocations
                    .entry(*index)
                    .and_modify(|old| {
                        if *old != path {
                            *old = None;
                        }
                    })
                    .or_insert(path);
            }
        }
        for (index, path) in allocations {
            if let Some(path) = path
                && path.1.is_some()
                && next.returns.get(&index) != Some(&path)
            {
                next.allocations.insert(index, path);
            }
        }
        self.allocation_mode.set(false);
        self.return_mode.set(false);
        self.field_origins.borrow_mut().clear();
    }
}
