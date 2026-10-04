//! Ready source header facts promote types without evaluating declarations.
use super::*;
use jai_source::SourceSpan;

const MAX_SOURCE_FACTS: usize = 1_048_576;

impl FileScope<'_> {
    pub(crate) fn reflection_source_facts(
        &self,
        types: &TypeRegistry,
        constants: &crate::typed_constants::ConstantPool,
        records: &aggregates::parameterized::RecordSpecializations,
        demand: SourceSpan,
    ) -> Result<Vec<(TypeId, SourceSpan)>, Diagnostic> {
        let mut facts = Vec::new();
        let mut owner_locations = HashMap::new();
        let mut visited = 0usize;
        // Declaration IDs retain actual source discovery order. An alias does
        // not fabricate a new type, and a generic template has no row until an
        // actual specialization has a checked canonical type.
        for declaration in self.declarations.graph.declarations() {
            let id = declaration.id();
            let location = declaration.location();
            if let Some(&ty) = self.declarations.nominals.declarations.get(&id) {
                owner_locations.entry(ty).or_insert(location);
                source_fact(types, &mut facts, ty, location)?;
            }
            if let Some(&ty) = self.declarations.nominals.value_types.get(&id) {
                source_fact(types, &mut facts, ty, location)?;
            }
            if let Some(signature) = self.declarations.signatures.get(&id) {
                source_fact(types, &mut facts, signature.ty, location)?;
            }
            match self.declarations.values.get(&id) {
                Some(Binding::Type(ty))
                | Some(Binding::Procedure {
                    ty, ..
                })
                | Some(Binding::Enum(aggregates::EnumConstant {
                    ty, ..
                })) => {
                    source_fact(types, &mut facts, *ty, location)?;
                }
                Some(Binding::Storage(storage)) => {
                    source_fact(types, &mut facts, storage.place().ty(), location)?;
                }
                Some(Binding::TypedConstant(id)) => {
                    let constant = constants.get(*id).ok_or_else(|| {
                        Diagnostic::at_source(
                            location,
                            "source reflection constant belongs to another semantic context",
                        )
                    })?;
                    source_constant_facts(types, &mut facts, constant, location, &mut visited)?;
                }
                _ => {}
            }
            if let Some(constant) = self.declarations.nominals.value_constants.get(&id) {
                source_constant_facts(types, &mut facts, constant, location, &mut visited)?;
            }
        }
        let mut parameters = self
            .declarations
            .nominals
            .module_parameter_types
            .iter()
            .collect::<Vec<_>>();
        parameters.sort_by_key(|(id, _)| id.index());
        for (&id, &ty) in parameters {
            let parameter = self.declarations.graph.parameter(id).ok_or_else(|| {
                Diagnostic::at_source(
                    demand,
                    "source reflection module parameter has no original source receipt",
                )
            })?;
            source_fact(types, &mut facts, ty, parameter.location)?;
        }
        // Nested Type constants are checked source values too. Their actual
        // namespace storage is published independently of catalog admission.
        let mut namespaces = Vec::new();
        let mut namespace_facts = 0usize;
        for (owner, names) in records.source_namespaces() {
            let Some(namespace) = records.member_bindings(owner) else {
                continue;
            };
            let mut values = Vec::new();
            for name in names {
                if let Some(crate::polymorphism::BakedValue::Type(ty)) = namespace.constant(*name) {
                    namespace_facts += 1;
                    if namespace_facts > MAX_SOURCE_FACTS.saturating_sub(facts.len()) {
                        return Err(Diagnostic::at_source(
                            demand,
                            "source reflection namespaces exceed their fact limit",
                        ));
                    }
                    values.push(*ty);
                }
            }
            if values.is_empty() {
                continue;
            }
            let location = records
                .source_location(owner)
                .or_else(|| owner_locations.get(&owner).copied())
                .ok_or_else(|| {
                    Diagnostic::at_source(
                        demand,
                        "source reflection namespace has no original checked owner location",
                    )
                })?;
            namespaces.push((location, owner, values));
        }
        namespaces.sort_by_key(|(location, owner, _)| {
            (
                location.source.index(),
                location.span.start,
                location.span.end,
                owner.index(),
            )
        });
        for (location, _, values) in namespaces {
            for ty in values {
                source_fact(types, &mut facts, ty, location)?;
            }
        }
        Ok(facts)
    }
}

fn source_fact(
    types: &TypeRegistry,
    facts: &mut Vec<(TypeId, SourceSpan)>,
    ty: TypeId,
    location: SourceSpan,
) -> Result<(), Diagnostic> {
    types
        .kind(ty)
        .map_err(|error| Diagnostic::at_source(location, error.to_string()))?;
    if facts.len() >= MAX_SOURCE_FACTS {
        return Err(Diagnostic::at_source(
            location,
            "source reflection headers exceed their fact limit",
        ));
    }
    facts.push((ty, location));
    Ok(())
}

fn source_constant_facts(
    types: &TypeRegistry,
    facts: &mut Vec<(TypeId, SourceSpan)>,
    constant: &jai_ir::ConstantValue,
    location: SourceSpan,
    visited: &mut usize,
) -> Result<(), Diagnostic> {
    let mut pending = vec![constant];
    while let Some(constant) = pending.pop() {
        *visited = visited.checked_add(1).ok_or_else(|| {
            Diagnostic::at_source(
                location,
                "source reflection constants exceed their fact limit",
            )
        })?;
        if *visited > MAX_SOURCE_FACTS {
            return Err(Diagnostic::at_source(
                location,
                "source reflection constants exceed their fact limit",
            ));
        }
        source_fact(types, facts, constant.ty, location)?;
        match &constant.kind {
            jai_ir::ConstantKind::RuntimeType(value) => {
                value
                    .identity()
                    .validate(types)
                    .map_err(|error| Diagnostic::at_source(location, error.to_string()))?;
                source_fact(types, facts, value.identity().ty(), location)?;
            }
            jai_ir::ConstantKind::Record(values) | jai_ir::ConstantKind::Array(values) => {
                if pending.len().saturating_add(values.len()) > MAX_SOURCE_FACTS {
                    return Err(Diagnostic::at_source(
                        location,
                        "source reflection constants exceed their fact limit",
                    ));
                }
                pending.extend(values.iter().rev());
            }
            jai_ir::ConstantKind::Distinct(value)
            | jai_ir::ConstantKind::Union {
                value, ..
            } => pending.push(value),
            _ => {}
        }
    }
    Ok(())
}
