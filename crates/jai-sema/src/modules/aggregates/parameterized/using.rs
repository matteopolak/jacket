//! Validate instance promotion paths once their owned field schemas are ready.
use super::*;

impl RecordSpecializations {
    pub(crate) fn validate_using(
        &self,
        graph: &ModuleGraph,
        nominals: &Nominals<'_>,
        types: &TypeRegistry,
        allow_pending: bool,
    ) -> Result<(), LocatedDiagnostic> {
        let roots = self
            .records()
            .map(|(ty, record)| {
                (
                    ty,
                    record.file,
                    record
                        .shape
                        .fields
                        .first()
                        .map_or(Span::default(), |field| field.syntax.span()),
                )
            })
            .chain(nominals.records.iter().map(|(&ty, record)| {
                (
                    ty,
                    record.file,
                    record
                        .fields
                        .first()
                        .map_or(Span::default(), |field| field.syntax.span),
                )
            }));
        for (root, root_file, root_span) in roots {
            let incomplete = std::cell::Cell::new(false);
            let shape = |ty| {
                self.record(ty)
                    .map(|record| record.shape.clone())
                    .or_else(|| {
                        nominals.records.get(&ty).map(|record| RecordMetadata {
                            using: Vec::new(),
                            name: None,
                            kind: record.kind,
                            fields: record
                                .fields
                                .iter()
                                .map(|field| FieldMetadata {
                                    name: Some(field.name),
                                    id: field.id,
                                    ty: field.ty,
                                    syntax: field.syntax.clone().into(),
                                })
                                .collect(),
                        })
                    })
                    .ok_or_else(|| {
                        incomplete.set(true);
                        Diagnostic::new(root_span, "using field requires a ready record schema")
                    })
            };
            let names = match crate::record_using::visible_names(root, root_span, types, shape) {
                Ok(names) => names,
                Err(_) if allow_pending && incomplete.get() => continue,
                Err(error) => return Err(located(graph, root_file, error)),
            };
            for name in names {
                crate::record_default_overrides::optional_field_path(
                    root, name, root_span, types, shape,
                )
                .map_err(|e| located(graph, root_file, e))?;
            }
        }
        Ok(())
    }
}
