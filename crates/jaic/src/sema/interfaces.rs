//! Structural interface validation at generic binding time.
use super::scope::Resolved;
use super::*;

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
            let Some(contract) = self.types.as_struct(self.types.repr_struct(required)) else {
                return err(span, "an interface restriction must name a struct contract");
            };
            if self
                .types
                .as_struct(self.types.repr_struct(actual))
                .is_none()
            {
                return err(
                    span,
                    format!(
                        "{} does not satisfy interface {}: expected a struct",
                        self.types.name(actual),
                        self.types.name(required)
                    ),
                );
            }
            self.layout_struct(contract, span)?;
            let fields = self.types.struct_info(contract).fields.clone();
            for field in fields {
                if field.using {
                    pending.push((actual, self.types.pointee(field.ty).unwrap_or(field.ty)));
                    continue;
                }
                let Some(name) = field.name else {
                    continue;
                };
                let found = self.find_member(actual, name, span)?;
                if !found.is_some_and(|(_, ty)| ty == field.ty) {
                    return err(
                        span,
                        format!(
                            "{} does not satisfy interface {}: missing or incompatible field `{}`",
                            self.types.name(actual),
                            self.types.name(required),
                            name
                        ),
                    );
                }
            }
            let Some(source) = self.struct_asts.get(&contract) else {
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
                        span,
                        format!(
                            "{} does not satisfy interface {}: missing member `{name}`",
                            self.types.name(actual),
                            self.types.name(required)
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
                let compatible = if !required_procs.is_empty() {
                    let mut ok = true;
                    for required_proc in required_procs {
                        let target = self.proc_type(required_proc, span)?;
                        let mut matched = false;
                        for value in &found {
                            let procedures = match value {
                                Resolved::Proc(p) => vec![*p],
                                Resolved::ProcSet(p) => p.clone(),
                                _ => Vec::new(),
                            };
                            for procedure in procedures {
                                if self.proc(procedure).is_macro
                                    != self.proc(required_proc).is_macro
                                {
                                    continue;
                                }
                                let candidate = if self.proc(procedure).is_poly {
                                    self.instantiate_for_proc_type(procedure, target, span)
                                } else {
                                    Some(procedure)
                                };
                                if let Some(candidate) = candidate {
                                    let ty = self.proc_type(candidate, span)?;
                                    matched |= matches!((self.types.kind(ty),self.types.kind(target)),(crate::types::TypeKind::Proc(a),crate::types::TypeKind::Proc(b)) if a==b);
                                }
                            }
                        }
                        ok &= matched;
                    }
                    ok
                } else {
                    found.iter().any(|value| matches!((&wanted,value),
                        (Resolved::Const{ty:a,value:Value::Type(x)},Resolved::Const{ty:b,value:Value::Type(y)}) if a==b && x==y)
                        || matches!((&wanted,value),(Resolved::Const{ty:a,..},Resolved::Const{ty:b,..}) if a==b && *a!=TypeId::TYPE))
                };
                if !compatible {
                    return err(
                        span,
                        format!(
                            "{} does not satisfy interface {}: incompatible member `{name}`",
                            self.types.name(actual),
                            self.types.name(required)
                        ),
                    );
                }
            }
        }
        Ok(())
    }
}
