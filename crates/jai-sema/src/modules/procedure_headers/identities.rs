//! Reserve authentic source identities without publishing unfinished signatures.
use super::*;

pub(in crate::modules) struct SourceProcedures {
    declarations: HashMap<DeclarationId, ProcedureId>,
}

impl SourceProcedures {
    pub(in crate::modules) fn len(&self) -> usize {
        self.declarations.len()
    }

    pub(in crate::modules) fn get(&self, declaration: DeclarationId) -> Option<ProcedureId> {
        self.declarations.get(&declaration).copied()
    }
}

pub(in crate::modules) fn is_concrete(declaration: &jai_modules::Declaration) -> bool {
    match &declaration.syntax().kind {
        FileDeclarationKind::Procedure(procedure) => {
            !procedure.expands && !crate::polymorphism::is_polymorphic(procedure)
        }
        FileDeclarationKind::ProcedurePrototype(prototype) => {
            !crate::polymorphism::is_polymorphic_prototype(prototype)
        }
        _ => false,
    }
}

pub(in crate::modules) fn reserve(
    graph: &ModuleGraph,
    callable_aliases: &HashMap<DeclarationId, Vec<DeclarationId>>,
) -> Result<SourceProcedures, LocatedDiagnostic> {
    let mut declarations = HashMap::new();
    for declaration in super::dependencies::ordered(graph, callable_aliases)? {
        let concrete = is_concrete(declaration);
        if concrete {
            let id = ProcedureId::new(declarations.len());
            declarations.insert(declaration.id(), id);
        }
    }
    Ok(SourceProcedures { declarations })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn graph(source: &str) -> ModuleGraph {
        let path = std::path::Path::new("/source-procedure-identities/main.jai");
        let mut provider = jai_modules::SourceOverlay::new();
        provider.insert(path, source.as_bytes().to_vec()).unwrap();
        ModuleGraph::load_with_provider(path, jai_modules::GraphOptions::default(), &provider)
            .unwrap()
    }

    fn declaration(graph: &ModuleGraph, name: &str) -> DeclarationId {
        graph
            .declarations()
            .iter()
            .find(|declaration| graph.symbols().name(declaration.name()) == name)
            .unwrap()
            .id()
    }

    fn inventory(graph: &ModuleGraph) -> SourceProcedures {
        reserve(
            graph,
            &crate::polymorphism::integration::callable_aliases(graph),
        )
        .unwrap()
    }

    #[test]
    fn authentic_concrete_headers_reserve_ids_without_templates_or_aliases() {
        let graph = graph(
            "identity::(value:$T)->T{return value;} recipe::()#expand{} \
             service::()->int #compiler; seed::()->int{return 42;} \
             alias::seed; main::()->int{return alias();}",
        );
        let inventory = inventory(&graph);
        assert_eq!(inventory.len(), 3);
        for name in ["service", "seed", "main"] {
            assert!(inventory.get(declaration(&graph, name)).is_some());
        }
        for name in ["identity", "recipe", "alias"] {
            assert!(inventory.get(declaration(&graph, name)).is_none());
        }
    }

    #[test]
    fn inferred_header_dependencies_keep_the_original_checked_id_order() {
        let graph = graph(
            "use::(value:=seed())->int{return value;} seed::()->int{return 42;} \
             main::()->int{return use();}",
        );
        let inventory = inventory(&graph);
        assert_eq!(
            inventory.get(declaration(&graph, "seed")),
            Some(ProcedureId::new(0))
        );
        assert_eq!(
            inventory.get(declaration(&graph, "use")),
            Some(ProcedureId::new(1))
        );
        assert_eq!(
            inventory.get(declaration(&graph, "main")),
            Some(ProcedureId::new(2))
        );
    }

    #[test]
    fn auxiliary_jobs_reserved_before_signatures_cannot_reuse_file_ids() {
        let graph = graph("seed::()->int{return 42;} main::()->int{return seed();}");
        let inventory = inventory(&graph);
        let mut generics = crate::polymorphism::integration::GenericContext::new(inventory.len());
        let first_auxiliary = generics.reserve_local_procedure().unwrap();
        let second_auxiliary = generics.reserve_local_procedure().unwrap();
        assert_eq!(first_auxiliary, ProcedureId::new(2));
        assert_eq!(second_auxiliary, ProcedureId::new(3));
        assert_eq!(
            inventory.get(declaration(&graph, "seed")),
            Some(ProcedureId::new(0))
        );
        assert_eq!(
            inventory.get(declaration(&graph, "main")),
            Some(ProcedureId::new(1))
        );
    }
}
