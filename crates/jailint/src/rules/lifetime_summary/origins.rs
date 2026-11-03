//! Storage provenance of checked values and selected allocation results.
use super::{
    ArrayKind, Def, HashMap, HashSet, Path, ProcId, Projection, Resolver, Root, Step, Summary,
    TypeKind, Val, value,
};

impl Resolver<'_> {
    pub(super) fn trace(
        &self,
        v: Val,
        summaries: &HashMap<ProcId, Summary>,
        visiting: &mut HashSet<Val>,
        depth: usize,
    ) -> Option<Path> {
        if depth > 24 || !visiting.insert(v) {
            return None;
        }
        if let Some(cast) = self.address_casts.get(&v).filter(|c| c.to_pointer) {
            let original = self.integer_address_source(
                cast.source,
                summaries,
                &mut HashSet::default(),
                depth + 1,
            )?;
            let path = self.trace(original, summaries, visiting, depth + 1);
            visiting.remove(&v);
            return path;
        }
        let defs = self.values.get(&v)?;
        let mut result = None;
        let derived = defs.iter().any(|d| matches!(d, Def::Borrow(_)));
        for def in defs {
            if derived && !matches!(def, Def::Borrow(_)) {
                continue;
            }
            let path = match def {
                Def::Parameter(formal) => Path {
                    root: Root::Parameter(*formal),
                    steps: Vec::new(),
                },
                Def::Address(p) => {
                    let ty = self.places.types.get(p).copied().or_else(|| {
                        self.places.paths[*p]
                            .1
                            .is_empty()
                            .then_some(self.places.paths[*p].0.1)
                    });
                    let descriptor = ty.is_some_and(|ty| {
                        matches!(
                            self.compiler.ty.types.kind(ty),
                            TypeKind::String
                                | TypeKind::Array {
                                    kind: ArrayKind::View,
                                    ..
                                }
                        )
                    });
                    if self.return_mode.get()
                        && descriptor
                        && self.places.paths[*p].1.is_empty()
                        && !self.params.contains_key(p)
                    {
                        self.field_origin(
                            v,
                            ty?,
                            &[Projection::Field(jaic::intern::Sym::intern("data"))],
                            summaries,
                            depth + 1,
                        )
                        .or_else(|| self.location(*p, summaries, visiting, depth + 1))?
                    } else {
                        self.location(*p, summaries, visiting, depth + 1)?
                    }
                }
                Def::Borrow(p) => self
                    .location(*p, summaries, visiting, depth + 1)?
                    .append(&[Step::Storage])?,
                Def::Copy(v) => self.trace(*v, summaries, visiting, depth + 1)?,
                Def::Load(p, ty) => {
                    if let Some((formal, _)) = self.params.get(p) {
                        if self.exposed.contains(p)
                            || self.writes.get(p).is_none_or(|w| w.len() != 1)
                        {
                            return None;
                        }
                        Path {
                            root: Root::Parameter(*formal),
                            steps: Vec::new(),
                        }
                    } else if !self.compiler.ty.types.is_pointer(*ty) {
                        let path = self.location(*p, summaries, visiting, depth + 1)?;
                        if matches!(path.root, Root::Parameter(_)) && path.steps.is_empty() {
                            path.append(&[Step::Deref])?
                        } else {
                            path
                        }
                    } else if !self.places.paths[*p].1.is_empty() {
                        self.location(*p, summaries, visiting, depth + 1)?
                            .append(&[Step::Deref])?
                    } else if let Some(writes) = self.writes.get(p) {
                        let [v] = writes.as_slice() else {
                            return None;
                        };
                        self.trace(*v, summaries, visiting, depth + 1)?
                    } else {
                        let path = self.location(*p, summaries, visiting, depth + 1)?;
                        if self.compiler.ty.types.is_pointer(*ty) {
                            path.append(&[Step::Deref])?
                        } else {
                            path
                        }
                    }
                }
                Def::Call(callee, args, i) => {
                    let summary = summaries.get(callee)?;
                    let (path, guard) = if self.allocation_mode.get() {
                        summary
                            .allocations
                            .get(i)
                            .or_else(|| summary.returns.get(i))?
                    } else {
                        summary
                            .allocations
                            .get(i)
                            .filter(|(_, g)| {
                                g.is_some_and(|(f, c)| {
                                    args.iter().any(|a| a.formal == f && a.constant == Some(c))
                                })
                            })
                            .or_else(|| summary.returns.get(i))?
                    };
                    if guard.is_some_and(|(formal, constant)| {
                        args.iter()
                            .find(|a| a.formal == formal)
                            .and_then(|a| a.constant)
                            .is_some_and(|c| c != constant)
                    }) {
                        return None;
                    }
                    let mut path = path.clone();
                    path.root = self.allocation_root(&path.root, v, args)?;
                    self.substitute(&path, args, summaries, visiting, depth + 1)?
                }
                Def::Indirect(callee, args, index) => {
                    self.indirect_return(*callee, args, *index, summaries, visiting, depth + 1)?
                }
                Def::Resize(callee, args) => {
                    let mode = args.first()?;
                    if mode.constant == Some(0)
                        || self.allocation_mode.get() && mode.constant.is_none()
                    {
                        if let Some(root) = self.context.indirect.get(callee).and_then(|s| s.root())
                        {
                            Path {
                                root,
                                steps: Vec::new(),
                            }
                        } else {
                            self.indirect_return(*callee, args, 0, summaries, visiting, depth + 1)
                                .unwrap_or(Path {
                                    root: Root::Heap,
                                    steps: Vec::new(),
                                })
                        }
                    } else {
                        if mode.constant.is_some_and(|c| c != 1) {
                            return None;
                        }
                        self.trace(value(args.get(3)?)?, summaries, visiting, depth + 1)?
                    }
                }
            };
            if result.as_ref().is_some_and(|old| old != &path) {
                return None;
            }
            result = Some(path);
        }
        visiting.remove(&v);
        result
    }
}
