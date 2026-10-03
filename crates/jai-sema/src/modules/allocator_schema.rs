//! Bind allocator storage only to the selected Preload's own declarations.
use super::*;
use jai_modules::Binding as ModuleBinding;

pub(super) fn bind(
    graph: &ModuleGraph,
    nominals: &Nominals<'_>,
    types: &mut TypeRegistry,
) -> Result<(), LocatedDiagnostic> {
    let Some(prelude) = graph.prelude() else {
        return Ok(());
    };
    let module = graph.module(prelude).expect("selected Preload module");
    let export = |name: &str| {
        graph
            .symbols()
            .find(name)
            .and_then(|name| module.exports().get(&name))
    };
    // Small bootstrap schemas may omit allocation entirely. Application types
    // with these spellings never designate either role.
    if export("Allocator").is_none() && export("Allocator_Mode").is_none() {
        return Ok(());
    }
    let error = |message: String| {
        located(
            graph,
            module.entry(),
            Diagnostic::new(Span::new(0, 0), message),
        )
    };
    let declaration = |name: &str| -> Result<DeclarationId, LocatedDiagnostic> {
        let Some(ModuleBinding::Declaration(id)) = export(name) else {
            return Err(error(format!("selected Preload must declare {name}")));
        };
        let source = graph.declaration(*id).expect("exported declaration");
        if graph
            .file(source.file())
            .expect("declaration file")
            .module()
            != prelude
        {
            return Err(error(format!(
                "selected Preload {name} must belong to that module, not a reexport"
            )));
        }
        Ok(*id)
    };
    let allocator_id = declaration("Allocator")?;
    let mode_id = declaration("Allocator_Mode")?;
    if !matches!(
        graph.declaration(allocator_id).unwrap().syntax().kind,
        FileDeclarationKind::Record(_)
    ) || !matches!(
        graph.declaration(mode_id).unwrap().syntax().kind,
        FileDeclarationKind::Enum(_)
    ) {
        return Err(error(
            "selected Preload allocator roles require original record and enum declarations".into(),
        ));
    }
    let nominal = |id| {
        nominals.declarations.get(&id).copied().ok_or_else(|| {
            error("selected Preload allocator declarations must be nominal types".into())
        })
    };
    let allocator = nominal(allocator_id)?;
    let mode = nominal(mode_id)?;
    let Some(record) = nominals.records.get(&allocator) else {
        return Err(error(
            "selected Preload Allocator must be a struct declaration".into(),
        ));
    };
    if record.declaration != allocator_id
        || record.fields.len() != 2
        || !record
            .fields
            .iter()
            .zip(["proc", "data"])
            .all(|(field, name)| graph.symbols().get(field.name) == Some(name))
    {
        return Err(error(
            "selected Preload Allocator requires proc and data fields in source order".into(),
        ));
    }
    let Some(enumeration) = nominals.enums.get(&mode) else {
        return Err(error(
            "selected Preload Allocator_Mode must be an enum declaration".into(),
        ));
    };
    let names = [
        "ALLOCATE",
        "RESIZE",
        "FREE",
        "STARTUP",
        "SHUTDOWN",
        "THREAD_START",
        "THREAD_STOP",
        "CREATE_HEAP",
        "DESTROY_HEAP",
        "IS_THIS_YOURS",
        "CAPS",
    ];
    if enumeration.flags
        || enumeration.members.len() != names.len()
        || !names
            .into_iter()
            .zip(jai_types::AllocatorMode::ALL)
            .all(|(name, mode)| {
                graph
                    .symbols()
                    .find(name)
                    .and_then(|name| enumeration.members.get(&name))
                    == Some(&mode.value())
            })
    {
        return Err(error(
            "selected Preload Allocator_Mode has incompatible operation names or values".into(),
        ));
    }
    types
        .bind_allocator(allocator, mode)
        .map_err(|cause| error(format!("selected Preload allocator schema: {cause}")))?;
    Ok(())
}

#[cfg(test)]
mod tests;
