//! Bind explicitly marked runtime prototype identities after normal type resolution.
use super::*;
use jai_ir::{ProcedurePrototype, PrototypeOrigin, RuntimeIntrinsic};
use jai_types::{LayoutPolicy, TypeView};
use jai_vm::RuntimeProcedure;

/// Shared by fixed, local, and selected generic prototype declarations.
pub(crate) fn bind_prototype(
    source: &syntax::ProcedurePrototype,
    declared_name: &str,
    signature: &Signature,
    types: &dyn TypeView,
    layout: Option<LayoutPolicy>,
) -> Result<ProcedurePrototype, Diagnostic> {
    let syntax::PrototypeBinding::Intrinsic { tag } = &source.binding else {
        return Err(Diagnostic::new(
            source.span,
            "runtime intrinsic binding requires an explicitly marked #intrinsic prototype",
        ));
    };
    let name = tag.as_deref().unwrap_or(declared_name);
    if source.parameters.iter().any(|parameter| {
        parameter.using
            || parameter.baking != syntax::ParameterBaking::None
            || parameter.variadic
            || parameter.evaluation != syntax::ParameterEvaluation::Evaluate
    }) {
        return Err(Diagnostic::new(
            source.span,
            "#intrinsic requires fixed evaluated runtime parameters without using, baked, or #discard modifiers",
        ));
    }
    let layout = layout.ok_or_else(|| {
        Diagnostic::new(
            source.span,
            "#intrinsic binding requires an explicitly selected target layout",
        )
    })?;
    let intrinsic = RuntimeIntrinsic::bind(name, signature.ty, types, layout)
        .map_err(|error| Diagnostic::new(source.span, error.to_string()))?;
    Ok(ProcedurePrototype {
        id: signature.id,
        signature: signature.ty,
        origin: PrototypeOrigin::Intrinsic(intrinsic),
    })
}

/// This map contains concrete declaration identities, never a runtime name lookup.
pub(super) fn bind(
    graph: &ModuleGraph,
    types: &TypeRegistry,
    declarations: &ScopedDeclarations<'_>,
    layout: Option<LayoutPolicy>,
) -> Result<HashMap<ProcedureId, RuntimeProcedure>, LocatedDiagnostic> {
    let mut bindings = HashMap::new();
    for declaration in graph.declarations() {
        let FileDeclarationKind::ProcedurePrototype(source) = &declaration.syntax().kind else {
            continue;
        };
        let syntax::PrototypeBinding::Intrinsic { tag } = &source.binding else {
            continue;
        };
        let name = tag
            .as_deref()
            .unwrap_or_else(|| graph.symbols().name(source.name));
        RuntimeIntrinsic::validate_name(name)
            .map_err(|error| graph.diagnostic(declaration.location(), error.to_string()))?;
        // Generic headers have no concrete procedure identity until selection.
        let Some(signature) = declarations.signatures.get(&declaration.id()) else {
            continue;
        };
        let prototype = bind_prototype(
            source,
            graph.symbols().name(source.name),
            signature,
            types,
            layout,
        )
        .map_err(|error| located(graph, declaration.file(), error))?;
        if let PrototypeOrigin::Intrinsic(intrinsic) = prototype.origin {
            bindings.insert(
                prototype.id,
                RuntimeProcedure {
                    signature: prototype.signature,
                    intrinsic,
                },
            );
        }
    }
    Ok(bindings)
}

#[cfg(test)]
mod tests;
