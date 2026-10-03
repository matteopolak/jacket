//! Ordered construction writes change only this record's owned field defaults.
use super::*;

impl Resolver<'_> {
    pub(super) fn apply_local_record_default_overrides(
        &mut self,
        owner: TypeId,
        members: &[syntax::RecordMember],
    ) -> Result<(), Diagnostic> {
        if self
            .meta
            .local_declarations
            .record_overrides_ready
            .contains(&owner)
        {
            return Ok(());
        }
        if self.local_no_write_field(owner).is_some() {
            // Sparse construction must interpret these original assignments
            // with the no-write field recipe before any value can be supplied.
            self.meta
                .local_declarations
                .no_write_record_overrides
                .insert(
                    owner,
                    members
                        .iter()
                        .filter(|member| {
                            matches!(member, syntax::RecordMember::DefaultOverride { .. })
                        })
                        .cloned()
                        .collect(),
                );
            return Ok(());
        }
        if !self
            .meta
            .local_declarations
            .active_record_overrides
            .insert(owner)
        {
            return Err(Diagnostic::new(
                self.span,
                "cyclic record default overrides",
            ));
        }
        let result = (|| {
            let mut defaults: HashMap<FieldId, jai_ir::ConstantValue> = HashMap::new();
            for member in members {
                let syntax::RecordMember::DefaultOverride {
                    target,
                    value,
                    span,
                } = member
                else {
                    continue;
                };
                let names: Vec<_> = match &target.kind {
                    syntax::PlaceKind::Name(name) => vec![*name],
                    syntax::PlaceKind::Qualified(path) => std::iter::once(path.root)
                        .chain(path.members.iter().copied())
                        .collect(),
                    _ => {
                        return Err(Diagnostic::new(
                            target.span,
                            "record default overrides require a named field path",
                        ));
                    }
                };
                if names.len() > crate::constant_limits::MAX_CONSTANT_DEPTH {
                    return Err(Diagnostic::new(
                        target.span,
                        "record default field path exceeds compiler depth budget",
                    ));
                }
                let mut ty = owner;
                let mut path = Vec::new();
                for name in names {
                    if self.record_metadata(ty, target.span)?.kind != RecordKind::Struct {
                        return Err(Diagnostic::new(
                            target.span,
                            "record default overrides require a struct field path",
                        ));
                    }
                    let projected = self.field_path(ty, name, target.span)?;
                    for field in projected {
                        ty = self
                            .types
                            .validate_field(ty, field)
                            .map_err(|error| Diagnostic::new(target.span, error.to_string()))?;
                        path.push(field);
                    }
                    if path.len() > crate::constant_limits::MAX_CONSTANT_DEPTH {
                        return Err(Diagnostic::new(
                            target.span,
                            "record default field path exceeds compiler depth budget",
                        ));
                    }
                }
                let replacement = self.local_typed_constant(value, ty)?;
                let (&root, nested) = path.split_first().ok_or_else(|| {
                    Diagnostic::new(target.span, "record default override requires a field")
                })?;
                let initial = match defaults.get(&root) {
                    Some(value) => value.clone(),
                    None => self.field_initial_value(root, *span)?,
                };
                let updated = crate::record_default_overrides::replace_constant(
                    initial,
                    nested,
                    replacement,
                    self.types,
                    *span,
                )?;
                defaults.insert(root, updated);
            }
            Ok::<_, Diagnostic>(defaults)
        })();
        self.meta
            .local_declarations
            .active_record_overrides
            .remove(&owner);
        // A failed dependency leaves the original field defaults intact. Retry
        // reuses the same source identities and the checked #run cache.
        self.meta.local_declarations.defaults.extend(result?);
        self.meta
            .local_declarations
            .record_overrides_ready
            .insert(owner);
        Ok(())
    }
}
