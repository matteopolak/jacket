//! Mixed return origins remain may contracts, so callers retain the temporary risk.
use super::{
    Def, Event, HashMap, HashSet, LifetimeArgument, Path, ProcId, Projection, Resolver, Root, Step,
    Summary, TypeId, Val,
};

impl Resolver<'_> {
    pub(in crate::rules) fn field_may_temporary(
        &self,
        value: Val,
        ty: TypeId,
        projections: &[Projection],
        all: &HashMap<ProcId, Summary>,
        depth: usize,
    ) -> bool {
        if depth > 24 {
            return false;
        }
        let key = (value, ty, projections.to_vec());
        if let Some(&temporary) = self.may_field_origins.borrow().get(&key) {
            return temporary;
        }
        let temporary = self.compute_field_may(value, ty, projections, all, depth);
        // A shallow negative is reusable within this inference iteration. A
        // recursive query's negative may instead be the traversal depth limit.
        if temporary || depth == 0 {
            self.may_field_origins.borrow_mut().insert(key, temporary);
        }
        temporary
    }

    fn compute_field_may(
        &self,
        value: Val,
        ty: TypeId,
        projections: &[Projection],
        all: &HashMap<ProcId, Summary>,
        depth: usize,
    ) -> bool {
        let Some(steps) = steps(projections) else {
            return false;
        };
        if self
            .field_origin(value, ty, projections, all, depth + 1)
            .is_some_and(|p| p.root == Root::Temporary)
        {
            return true;
        }
        for def in self.values.get(&value).into_iter().flatten() {
            match def {
                Def::Call(callee, args, index) => {
                    let Some(s) = all.get(callee) else {
                        continue;
                    };
                    if s.may_return_fields.contains(&(*index, steps.clone())) {
                        return true;
                    }
                    if let Some(path) = s.return_fields.get(&(*index, steps.clone())) {
                        if self.return_path_may(path, args, all, depth + 1) {
                            return true;
                        }
                    } else if s
                        .returns
                        .get(index)
                        .is_some_and(|(p, _)| p.root == Root::Temporary)
                    {
                        return true;
                    }
                }
                Def::Copy(source)
                    if *source != value
                        && self.field_may_temporary(*source, ty, projections, all, depth + 1) =>
                {
                    return true;
                }
                _ => {}
            }
        }
        let Some(root) = self.places.existing(value, ty) else {
            return false;
        };
        if let Some(field) = projections
            .iter()
            .try_fold(root, |p, proj| self.places.projected(p, *proj))
            && self
                .writes
                .get(&field)
                .into_iter()
                .flatten()
                .any(|v| self.may_temporary(*v, all, &mut HashSet::default(), depth + 1))
        {
            return true;
        }
        self.writes.get(&root).into_iter().flatten().any(|source| {
            *source != value && self.field_may_temporary(*source, ty, projections, all, depth + 1)
        })
    }

    fn return_path_may(
        &self,
        path: &Path,
        args: &[LifetimeArgument],
        all: &HashMap<ProcId, Summary>,
        depth: usize,
    ) -> bool {
        if path.root == Root::Temporary {
            return true;
        }
        if self
            .substitute(path, args, all, &mut HashSet::default(), depth + 1)
            .is_some_and(|p| p.root == Root::Temporary)
        {
            return true;
        }
        let Root::Parameter(formal) = path.root else {
            return false;
        };
        let Some(arg) = args.iter().find(|a| a.formal == formal) else {
            return false;
        };
        let Some(value) = super::value(arg) else {
            return false;
        };
        if path.steps.is_empty() {
            return self.may_temporary(value, all, &mut HashSet::default(), depth + 1);
        }
        let projections = path
            .steps
            .iter()
            .map(|s| match s {
                Step::Field(n) => Some(Projection::Field(*n)),
                Step::Element(n) => Some(Projection::Element(*n)),
                _ => None,
            })
            .collect::<Option<Vec<_>>>();
        projections.is_some_and(|p| self.field_may_temporary(value, arg.ty, &p, all, depth + 1))
    }

    pub(super) fn infer_may_returns(
        &self,
        next: &mut Summary,
        all: &HashMap<ProcId, Summary>,
        live: &super::flow::Live,
    ) {
        next.may_returns.clear();
        next.may_return_fields.clear();
        for event in self
            .body
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
                ..
            } = event
            {
                if self.compiler.ty.types.as_struct(*ty).is_none()
                    && self.may_temporary(*value, all, &mut HashSet::default(), 0)
                {
                    next.may_returns.insert(*index);
                }
                for projections in self.storage_fields(*ty, 0) {
                    if self.field_may_temporary(*value, *ty, &projections, all, 0)
                        && let Some(path) = steps(&projections)
                    {
                        next.may_return_fields.insert((*index, path));
                    }
                }
            }
        }
    }
}

fn steps(projections: &[Projection]) -> Option<Vec<Step>> {
    projections
        .iter()
        .map(|p| match p {
            Projection::Field(n) => Some(Step::Field(*n)),
            Projection::Element(n) => Some(Step::Element(*n)),
            _ => None,
        })
        .collect()
}
