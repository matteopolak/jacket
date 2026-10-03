//! Validate instance promotion paths once their owned field schemas are ready.
use super::*;
use jai_types::TypeKind;

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
            let mut names = HashSet::new();
            let mut pending = vec![(root, HashSet::new())];
            while let Some((ty, mut ancestors)) = pending.pop() {
                if !ancestors.insert(ty) {
                    return Err(located(
                        graph,
                        root_file,
                        Diagnostic::new(root_span, "cyclic using field promotion"),
                    ));
                }
                let shape = self
                    .record(ty)
                    .map(|record| (record.file, record.shape.clone()))
                    .or_else(|| {
                        nominals.records.get(&ty).map(|record| {
                            (
                                record.file,
                                RecordMetadata {
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
                                },
                            )
                        })
                    });
                let Some((file, shape)) = shape else {
                    if allow_pending {
                        break;
                    }
                    return Err(located(
                        graph,
                        root_file,
                        Diagnostic::new(root_span, "using field requires a ready record schema"),
                    ));
                };
                for field in shape.fields {
                    if let Some(name) = field.name
                        && !names.insert(name)
                    {
                        return Err(located(
                            graph,
                            file,
                            Diagnostic::new(
                                field.syntax.span(),
                                format!(
                                    "ambiguous promoted record member '{}'",
                                    graph.symbols().name(name)
                                ),
                            ),
                        ));
                    }
                    if field.syntax.using() {
                        if !matches!(types.kind(field.ty), Ok(TypeKind::Record(_))) {
                            return Err(located(
                                graph,
                                file,
                                Diagnostic::new(
                                    field.syntax.span(),
                                    "using field requires a record value",
                                ),
                            ));
                        }
                        pending.push((field.ty, ancestors.clone()));
                    }
                }
            }
        }
        Ok(())
    }
}
