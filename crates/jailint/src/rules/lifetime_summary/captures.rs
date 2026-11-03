//! Capture paths remain stable when a callback writes scalar contents, rather than pointers.
use super::{Compiler, Kind, LifetimeBody, Path, Root, Step, Summary, TypeKind};

impl Summary {
    pub(in crate::rules) fn stable_captures(
        &self,
        compiler: &Compiler,
        body: Option<&LifetimeBody>,
    ) -> bool {
        if !self.unknown_write.is_empty()
            || !self.escapes.is_empty()
            || self
                .may
                .keys()
                .any(|e| matches!(e.kind, Kind::Unknown | Kind::Free))
        {
            return false;
        }
        let Some(body) = body else {
            return self.memory.is_some();
        };
        self.may
            .keys()
            .filter(|e| e.kind == Kind::Write)
            .all(|e| scalar_destination(compiler, body, &e.target))
    }
}

fn scalar_destination(compiler: &Compiler, body: &LifetimeBody, path: &Path) -> bool {
    let Root::Parameter(formal) = path.root else {
        return false;
    };
    let Some(mut ty) = body
        .blocks
        .iter()
        .flat_map(|b| &b.events)
        .find_map(|e| match e {
            super::Event::Parameter {
                formal: n,
                ty,
                ..
            } if *n == formal => Some(*ty),
            _ => None,
        })
    else {
        return false;
    };
    ty = compiler.ty.types.pointee(ty).unwrap_or(ty);
    for step in &path.steps {
        let next = match step {
            Step::Deref => compiler.ty.types.pointee(ty),
            Step::Field(name) => match compiler.ty.types.kind(ty) {
                TypeKind::Struct(st) => {
                    let info = compiler.ty.types.struct_info(*st);
                    if info.is_union {
                        return false;
                    }
                    info.fields
                        .iter()
                        .find(|f| f.name == Some(*name))
                        .map(|f| f.ty)
                }
                TypeKind::Array {
                    ..
                }
                | TypeKind::String
                    if name.as_str() == "count" =>
                {
                    Some(jaic::types::TypeId::S64)
                }
                _ => None,
            },
            // Unknown storage/index writes can replace pointer-bearing elements.
            Step::Storage | Step::Element(_) => None,
        };
        let Some(next) = next else {
            return false;
        };
        ty = next;
    }
    compiler.ir_ty(ty).is_some()
        && !compiler.ty.types.is_pointer(ty)
        && !matches!(compiler.ty.types.kind(ty), TypeKind::Proc(_))
}
