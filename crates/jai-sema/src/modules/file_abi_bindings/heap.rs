//! Bind selected allocator roles to checked virtual heap capabilities.
use super::*;

/// Bind only the actual selected Default_Allocator C fallback declarations. The
/// allocator's ordinary dispatch body still runs; these calls never use native malloc.
fn bind_heap_headers(
    graph: &ModuleGraph,
    types: &TypeRegistry,
    declarations: &ScopedDeclarations<'_>,
    context: Option<&FileAbiBindingContext>,
    target: Option<&BuildTarget>,
    availability: HeaderAvailability,
) -> Result<HashMap<ProcedureId, jai_vm::heap_abi::HeapAbiProcedure>, LocatedDiagnostic> {
    let Some(context) = context else {
        return Ok(HashMap::new());
    };
    let file = context
        .allocator
        .first()
        .map(|source| source.entry.file)
        .or_else(|| context.stdio.as_ref().map(|stdio| stdio.entry.file));
    let site = file
        .and_then(|file| graph.locate(file, Span::default()))
        .unwrap_or_else(|| {
            graph
                .locate(graph.module(graph.root()).unwrap().entry(), Span::default())
                .unwrap()
        });
    let error = |message: String| graph.diagnostic(site, message);
    if !matches_context(graph, context, target) {
        return Err(error(
            "default allocator receipt differs from this graph or target".into(),
        ));
    }
    let mut bindings = HashMap::new();
    for allocator in &context.allocator {
        for (procedure, capability) in
            bind_selected_allocator(graph, types, declarations, allocator, availability)?
        {
            if bindings.insert(procedure, capability).is_some() {
                return Err(error(
                    "duplicate original allocator procedure identity".into(),
                ));
            }
        }
    }
    Ok(bindings)
}
fn bind_selected_allocator(
    graph: &ModuleGraph,
    types: &TypeRegistry,
    declarations: &ScopedDeclarations<'_>,
    allocator: &AllocatorSourceReceipt,
    availability: HeaderAvailability,
) -> Result<HashMap<ProcedureId, jai_vm::heap_abi::HeapAbiProcedure>, LocatedDiagnostic> {
    use jai_vm::heap_abi::HeapAuthority;
    let site = graph.locate(allocator.entry.file, Span::default()).unwrap();
    let error = |message: String| graph.diagnostic(site, message);
    let mut candidates = Vec::new();
    let mut library = None;
    for &(declaration_id, operation) in &allocator.roles {
        let declaration = graph
            .declaration(declaration_id)
            .ok_or_else(|| error("allocator source role has no original declaration".into()))?;
        if declaration.file() != allocator.entry.file {
            return Err(error(
                "allocator role is outside its selected source receipt".into(),
            ));
        }
        let FileDeclarationKind::ProcedurePrototype(prototype) = &declaration.syntax().kind else {
            return Err(error(
                "allocator source role is not its original prototype".into(),
            ));
        };
        let Some(signature) = declarations.signatures.get(&declaration_id) else {
            if availability == HeaderAvailability::Ready {
                continue;
            }
            return Err(error(
                "allocator foreign declaration has no checked signature".into(),
            ));
        };
        let origin = foreign_libraries::origin(graph, declaration.file(), prototype)?;
        let PrototypeOrigin::Foreign {
            library: Some(actual),
            ..
        } = &origin
        else {
            return Err(error(
                "allocator declaration has no canonical library".into(),
            ));
        };
        let jai_ir::ForeignLibraryId::File(id) = actual.id else {
            return Err(error(
                "allocator library must be a defining file declaration".into(),
            ));
        };
        if graph
            .declaration(id)
            .is_none_or(|declaration| declaration.file() != allocator.entry.file)
            || library.as_ref().is_some_and(|previous| previous != actual)
        {
            return Err(error(
                "allocator library differs from its selected source receipt".into(),
            ));
        }
        library = Some(actual.clone());
        candidates.push((
            operation,
            ProcedurePrototype {
                id: signature.id,
                signature: signature.ty,
                origin,
            },
        ));
    }
    let Some(library) = library else {
        return Ok(HashMap::new());
    };
    let authority = HeapAuthority::from_verified_source(
        library,
        candidates
            .iter()
            .map(|(operation, prototype)| (prototype.id, *operation)),
    )
    .map_err(|failure| error(failure.to_string()))?;
    candidates
        .iter()
        .map(|(_, prototype)| {
            authority
                .bind(prototype, types)
                .map(|binding| (prototype.id, binding))
                .map_err(|failure| error(failure.to_string()))
        })
        .collect()
}

pub(in crate::modules) fn bind_heap(
    graph: &ModuleGraph,
    types: &TypeRegistry,
    declarations: &ScopedDeclarations<'_>,
    context: Option<&FileAbiBindingContext>,
    target: Option<&BuildTarget>,
) -> Result<HashMap<ProcedureId, jai_vm::heap_abi::HeapAbiProcedure>, LocatedDiagnostic> {
    bind_heap_headers(
        graph,
        types,
        declarations,
        context,
        target,
        HeaderAvailability::Complete,
    )
}
/// Missing checked identities remain pending; they never mint a guessed capability.
pub(in crate::modules) fn bind_heap_ready(
    graph: &ModuleGraph,
    types: &TypeRegistry,
    declarations: &ScopedDeclarations<'_>,
    context: Option<&FileAbiBindingContext>,
    target: Option<&BuildTarget>,
) -> Result<HashMap<ProcedureId, jai_vm::heap_abi::HeapAbiProcedure>, LocatedDiagnostic> {
    bind_heap_headers(
        graph,
        types,
        declarations,
        context,
        target,
        HeaderAvailability::Ready,
    )
}
