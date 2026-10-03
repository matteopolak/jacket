//! Script entries are ordinary checked procedures, without a native executable ABI.
use crate::{Error, ScriptEntry, ScriptParameters, ScriptResult};
use jai_modules::{Binding, ModuleGraph};
use jai_sema::Library;

pub(crate) fn select(graph: &ModuleGraph, library: &Library) -> Result<ScriptEntry, Error> {
    let name = graph.symbols().find("main").ok_or(Error::Entry("script requires a root main procedure"))?;
    let file = graph.module(graph.root()).expect("checked graph root").entry();
    let path = jai_syntax::NamePath { root: name, members: vec![] };
    let Binding::Declaration(id) = graph.lookup(file, &path)
        .map_err(|_| Error::Entry("script main must denote a single root procedure"))? else {
        return Err(Error::Entry("script main must denote a single root procedure"));
    };
    let declaration = graph.declaration(id).expect("checked source declaration");
    if graph.file(declaration.file()).expect("checked declaration file").module() != graph.root() {
        return Err(Error::Entry("an imported main is not the script entry"));
    }
    let procedure = library.procedure(id).ok_or(Error::Entry("script main requires a checked body"))?;
    if library.procedure_phase(procedure.id) == jai_types::ProcedureExecution::CompileTimeOnly {
        return Err(Error::Entry("script main cannot be compile-time only"));
    }
    let signature = library.types().procedure_definition(procedure.signature)
        .map_err(|error| Error::Types(error.to_string()))?;
    let parameters = match signature.parameters.as_ref() {
        [] => ScriptParameters::None,
        [ty] if matches!(library.types().kind(*ty), Ok(jai_types::TypeKind::Slice(element))
            if matches!(library.types().kind(*element), Ok(jai_types::TypeKind::String))) => ScriptParameters::Arguments(*ty),
        _ => return Err(Error::Entry("script main must take no parameters or one []string argument")),
    };
    let result = match signature.results.as_ref() {
        [] => ScriptResult::Void,
        [ty] if *ty == library.types().scalar(jai_types::ScalarType::Int(jai_types::IntegerType::S64)) => ScriptResult::Int,
        _ => return Err(Error::Entry("script main must return int or void")),
    };
    Ok(ScriptEntry { procedure: procedure.id, parameters, result })
}
