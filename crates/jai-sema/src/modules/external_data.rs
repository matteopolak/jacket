//! File external storage retains actual graph declarations and resolved libraries.
use super::*;

pub(super) fn declaration(
    graph: &ModuleGraph,
    declaration: &jai_modules::Declaration,
    binding: &syntax::ExternalDataBinding,
    ty: TypeId,
    index: usize,
    types: &TypeRegistry,
) -> Result<Global, LocatedDiagnostic> {
    let source = match &binding.source {
        syntax::ExternalDataSource::Program => jai_ir::ExternalDataSource::Program,
        syntax::ExternalDataSource::Library(path) => {
            let id = declaration_id(graph, declaration.file(), path, binding.span)
                .map_err(|error| located(graph, declaration.file(), error))?;
            jai_ir::ExternalDataSource::Library(foreign_libraries::declaration(
                graph,
                graph
                    .declaration(id)
                    .expect("resolved graph declaration owns its identity"),
            )?)
        }
    };
    let metadata = jai_ir::ExternalData::new(
        jai_ir::ExternalDataId::File(declaration.id()),
        ty,
        source,
        binding
            .symbol
            .clone()
            .unwrap_or_else(|| graph.symbols().name(declaration.name()).to_owned()),
        declaration.location(),
        types,
    )
    .map_err(|error| graph.diagnostic(declaration.location(), error.to_string()))?;
    Ok(Global::new_external(index, metadata))
}
