//! Select application entry points from checked source bindings.
use super::{located, path};
use crate::{EntryPoint, Library};
use jai_modules::{Binding, ModuleGraph};
use jai_source::{Diagnostic, LocatedDiagnostic, Span};
use jai_types::{IntegerType, ScalarType};

/// Return the application's real `main`, if its root namespace contains one.
///
/// Imported procedures do not become application entries. The declaration and
/// procedure identities must belong to the same graph/library resolution.
pub fn select_entry(
    graph: &ModuleGraph,
    library: &Library,
) -> Result<Option<EntryPoint>, LocatedDiagnostic> {
    let file = graph.module(graph.root()).expect("root module").entry();
    let Some(name) = graph.symbols().find("main") else {
        return Ok(None);
    };
    let binding = match graph.lookup(file, &path(name)) {
        Ok(binding) => binding,
        Err(jai_modules::LookupError::UnknownName(_)) => return Ok(None),
        Err(error) => {
            return Err(located(
                graph,
                file,
                Diagnostic::new(
                    Span::default(),
                    format!("cannot select application main: {error:?}"),
                ),
            ));
        }
    };
    let Binding::Declaration(id) = binding else {
        return Err(located(
            graph,
            file,
            Diagnostic::new(
                Span::default(),
                "application main must denote a single procedure",
            ),
        ));
    };
    let declaration = graph.declaration(id).expect("bound declaration");
    if graph
        .file(declaration.file())
        .expect("declaration file")
        .module()
        != graph.root()
    {
        return Ok(None);
    }
    let error = |message| {
        located(
            graph,
            declaration.file(),
            Diagnostic::new(declaration.location().span, message),
        )
    };
    let procedure = library.procedure(id).ok_or_else(|| {
        error("application main must be a checked procedure with a body".to_owned())
    })?;
    let signature = library
        .types()
        .procedure_definition(procedure.signature)
        .map_err(|cause| error(cause.to_string()))?;
    if !signature.parameters.is_empty() {
        return Err(error("main cannot take parameters".to_owned()));
    }
    match signature.results.as_ref() {
        [] => Ok(Some(EntryPoint::Void(procedure.id))),
        [ty] if *ty == library.types().scalar(ScalarType::Int(IntegerType::S64)) => {
            Ok(Some(EntryPoint::Int(procedure.id)))
        }
        _ => Err(error("main must return int or void".to_owned())),
    }
}
