//! Structural interface validation at generic binding time.
use super::scope::Resolved;
use super::{Compiler, HashSet, ProcId, Result, Span, TypeId, Value, err};
use crate::diagnostics::Code;

impl Compiler {
    pub(super) fn validate_interface(
        &mut self,
        actual: TypeId,
        required: TypeId,
        span: Span,
    ) -> Result<()> {
        let mut pending = vec![(actual, required)];
        let mut checked = HashSet::default();
        while let Some((actual, required)) = pending.pop() {
            if !checked.insert((actual, required)) {
                continue;
            }
            let Some(contract) = self.ty.types.as_struct(self.ty.types.repr_struct(required))
            else {
                return err(
                    Code::InterfaceRestriction,
                    span,
                    "an interface restriction must name a struct contract",
                );
            };
            if self
                .ty
                .types
                .as_struct(self.ty.types.repr_struct(actual))
                .is_none()
            {
                return err(
                    Code::InterfaceConstraint,
                    span,
                    format!(
                        "{} does not satisfy interface {}: expected a struct",
                        self.ty.types.name(actual),
                        self.ty.types.name(required)
                    ),
                );
            }
            self.layout_struct(contract, span)?;
            let fields = self.ty.types.struct_info(contract).fields.clone();
            for field in fields.iter() {
                if field.using {
                    pending.push((actual, self.ty.types.pointee(field.ty).unwrap_or(field.ty)));
                    continue;
                }
                let Some(name) = field.name else {
                    continue;
                };
                let found = self.find_member(actual, name, span)?;
                if found.is_none_or(|(_, ty)| ty != field.ty) {
                    return err(
                        Code::InterfaceConstraint,
                        span,
                        format!(
                            "{} does not satisfy interface {}: missing or incompatible field `{}`",
                            self.ty.types.name(actual),
                            self.ty.types.name(required),
                            name
                        ),
                    );
                }
            }
            let Some(source) = self.ty.struct_asts.get(&contract) else {
                continue;
            };
            let mut declarations: Vec<_> = self
                .scope(source.scope)
                .names
                .iter()
                .flat_map(|(&name, ids)| ids.iter().map(move |&id| (id, name)))
                .collect();
            declarations.sort_by_key(|&(id, _)| id);
            for (id, name) in declarations {
                let wanted = self.resolve_entity(id)?;
                let Some(candidates) = self.struct_constant(actual, name)? else {
                    return err(
                        Code::InterfaceConstraint,
                        span,
                        format!(
                            "{} does not satisfy interface {}: missing member `{name}`",
                            self.ty.types.name(actual),
                            self.ty.types.name(required)
                        ),
                    );
                };
                let mut found = Vec::new();
                for id in candidates {
                    found.push(self.resolve_entity(id)?);
                }
                let required_procs = match &wanted {
                    Resolved::Proc(p) => vec![*p],
                    Resolved::ProcSet(p) => p.clone(),
                    _ => Vec::new(),
                };
                let compatible = if required_procs.is_empty() {
                    found.iter().any(|value| matches!((&wanted,value),
                        (Resolved::Const{ty:a,value:Value::Type(x)},Resolved::Const{ty:b,value:Value::Type(y)}) if a==b && x==y)
                        || matches!((&wanted,value),(Resolved::Const{ty:a,..},Resolved::Const{ty:b,..}) if a==b && *a!=TypeId::TYPE))
                } else {
                    let mut ok = true;
                    for required_proc in required_procs {
                        let target = self.proc_type(required_proc, span)?;
                        let mut matched = false;
                        for procedure in found.iter().flat_map(procs_of) {
                            matched |=
                                self.satisfies_proc(procedure, required_proc, target, span)?;
                        }
                        ok &= matched;
                    }
                    ok
                };
                if !compatible {
                    return err(
                        Code::InterfaceConstraint,
                        span,
                        format!(
                            "{} does not satisfy interface {}: incompatible member `{name}`",
                            self.ty.types.name(actual),
                            self.ty.types.name(required)
                        ),
                    );
                }
            }
        }
        Ok(())
    }

    /// Whether `procedure` (instantiated for `target` when polymorphic) has exactly the type `target`
    /// and the same macro-ness as `required`.
    fn satisfies_proc(
        &mut self,
        procedure: ProcId,
        required: ProcId,
        target: TypeId,
        span: Span,
    ) -> Result<bool> {
        if self.proc(procedure).is_macro != self.proc(required).is_macro {
            return Ok(false);
        }
        let candidate = if self.proc(procedure).is_poly {
            self.instantiate_for_proc_type(procedure, target, span)
        } else {
            Some(procedure)
        };
        let Some(candidate) = candidate else {
            return Ok(false);
        };
        let ty = self.proc_type(candidate, span)?;
        Ok(matches!(
            (self.ty.types.kind(ty), self.ty.types.kind(target)),
            (crate::types::TypeKind::Proc(a), crate::types::TypeKind::Proc(b)) if a == b
        ))
    }
}

/// The procedures a resolved member stands for.
fn procs_of(value: &Resolved) -> Vec<ProcId> {
    match value {
        Resolved::Proc(p) => vec![*p],
        Resolved::ProcSet(p) => p.clone(),
        _ => Vec::new(),
    }
}
