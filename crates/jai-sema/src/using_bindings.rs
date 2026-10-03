//! Using parameters expose aliases to their real projected storage.
use super::*;
use std::collections::HashSet;

impl Resolver<'_> {
    pub(crate) fn using_record(&mut self, storage: Storage) -> Result<(), Diagnostic> {
        let mut base = storage.place();
        if matches!(
            self.types.kind(base.ty()),
            Ok(jai_types::TypeKind::Pointer(_))
        ) {
            base = self
                .places
                .dereference(ValueExpr::Load(base), self.types)
                .map_err(|error| self.error(error.to_string()))?;
        }
        let mut pending = vec![(base.ty(), Vec::new(), HashSet::new())];
        let mut names = HashSet::new();
        let mut bindings = Vec::new();
        while let Some((ty, prefix, mut ancestors)) = pending.pop() {
            if !ancestors.insert(ty) {
                return Err(self.error("cyclic using field promotion"));
            }
            let record = self.record_metadata(ty, self.span)?;
            for field in record.fields {
                if field.name.is_some_and(|name| !names.insert(name)) {
                    return Err(self.error("ambiguous using parameter member"));
                }
                let mut path = prefix.clone();
                path.push(field.id);
                if let Some(name) = field.name {
                    bindings.push((name, path.clone()));
                }
                if field.syntax.using() {
                    pending.push((field.ty, path, ancestors.clone()));
                }
            }
        }
        for (name, path) in bindings {
            let mut place = base;
            for field in path {
                place = self
                    .places
                    .field(place, field, self.types)
                    .map_err(|error| self.error(error.to_string()))?;
            }
            let storage = Storage::from_place(place, self.types)
                .map_err(|error| self.error(error.to_string()))?;
            self.bind_name(name, Binding::Storage(storage))?;
        }
        Ok(())
    }
}
