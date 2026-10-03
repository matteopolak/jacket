//! Read-only consumers use a proof from the original successful application.
//!
//! This helper is staged until the retained type-preparation consumer is ready.
//! It performs no declaration lookup, constant evaluation, or type reservation.
use super::{RecordSpecializationKey, RecordSpecializations};
use crate::local_declarations::LexicalScopeId;
use crate::polymorphism::Substitution;
use crate::{Diagnostic, Span, TypeId, TypeRegistry, syntax};
use jai_modules::FileInstanceId;
use jai_source::{SourceId, Symbols};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) enum ApplicationEnvironment {
    Graph,
    Lexical {
        procedure: jai_ir::ProcedureId,
        source: Option<SourceId>,
        scopes: Vec<LexicalScopeId>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum ApplicationAnnotationOwner {
    None,
    Record(TypeId),
    Forbidden,
}

/// Site identity includes the actual file instance, substitution and scope.
/// `None` and an explicitly supplied empty substitution remain distinct.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct ApplicationProofKey {
    file: FileInstanceId,
    start: usize,
    end: usize,
    environment: ApplicationEnvironment,
    owner: ApplicationAnnotationOwner,
    enclosing_record: Option<TypeId>,
    target: Option<jai_types::LayoutPolicy>,
    substitution: Option<Substitution>,
}

pub(super) struct ApplicationProofSite<'a> {
    pub(super) file: FileInstanceId,
    pub(super) source: &'a syntax::TypeApplicationSyntax,
    pub(super) environment: ApplicationEnvironment,
    pub(super) owner: ApplicationAnnotationOwner,
    pub(super) enclosing_record: Option<TypeId>,
    pub(super) target: Option<jai_types::LayoutPolicy>,
    pub(super) substitution: Option<&'a Substitution>,
}

impl ApplicationProofKey {
    pub(super) fn new(site: ApplicationProofSite<'_>, symbols: &Symbols) -> Self {
        let substitution = site.substitution.map(|substitution| {
            let mut substitution = substitution.clone();
            substitution
                .types
                .sort_by_key(|binding| symbols.name(binding.name));
            substitution
                .constants
                .sort_by_key(|binding| symbols.name(binding.name));
            substitution
                .callables
                .sort_by_key(|binding| (symbols.name(binding.name), binding.occurrence));
            substitution
        });
        Self {
            file: site.file,
            start: site.source.span.start,
            end: site.source.span.end,
            environment: site.environment,
            owner: site.owner,
            enclosing_record: site.enclosing_record,
            target: site.target,
            substitution,
        }
    }
}

/// Initial inputs and final nominal arguments are separate checked facts.
/// An accepted modifier can change either values or their canonical types.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ReadyApplicationProof {
    ty: TypeId,
    initial: RecordSpecializationKey,
    final_key: RecordSpecializationKey,
}

impl ReadyApplicationProof {
    pub(super) fn checked(
        ty: TypeId,
        initial: RecordSpecializationKey,
        types: &TypeRegistry,
        records: &RecordSpecializations,
        span: Span,
    ) -> Result<Option<Self>, Diagnostic> {
        // Recursive pointers can successfully resolve the current incomplete
        // reservation. That is legal resolution, but it is not a ready proof.
        let Some(record) = records.record(ty) else {
            return Ok(None);
        };
        let final_key = records.key_for_type(ty).ok_or_else(|| {
            Diagnostic::new(span, "a checked application has no canonical template key")
        })?;
        if record.nested
            || record.origin != Some(initial.template)
            || final_key.template != initial.template
            || initial.arguments.len() != final_key.arguments.len()
        {
            return Err(Diagnostic::new(
                span,
                "a checked application differs from its original template origin",
            ));
        }
        types
            .record_definition(ty)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?;
        Ok(Some(Self {
            ty,
            initial,
            final_key: final_key.clone(),
        }))
    }

    pub(super) fn ty(&self) -> TypeId {
        self.ty
    }
}

#[derive(Default)]
pub(super) struct ApplicationProofs {
    ready: HashMap<ApplicationProofKey, ReadyApplicationProof>,
}

impl ApplicationProofs {
    pub(super) fn remember(
        &mut self,
        key: ApplicationProofKey,
        proof: ReadyApplicationProof,
        span: Span,
    ) -> Result<(), Diagnostic> {
        if let Some(previous) = self.ready.get(&key) {
            if previous != &proof {
                return Err(Diagnostic::new(
                    span,
                    "a source type application changed within its checked environment",
                ));
            }
            return Ok(());
        }
        self.ready.insert(key, proof);
        Ok(())
    }

    pub(super) fn ready(&self, key: &ApplicationProofKey) -> Option<&ReadyApplicationProof> {
        self.ready.get(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::aggregates::parameterized::{TypeRequest, resolve_type};
    use crate::modules::aggregates::types::Nominals;
    use crate::polymorphism::BakedValue;
    use jai_modules::{GraphOptions, ModuleGraph, SourceOverlay};
    use jai_types::{Integer, IntegerType, ScalarType};

    fn graph() -> ModuleGraph {
        let path = std::path::Path::new("/jai-application-proof/main.jai");
        let mut overlay = SourceOverlay::new();
        overlay
            .insert(
                path,
                b"Box::struct(T:Type,U:Type,N:int,M:int){left:T;right:U;values:[N+M]int;} First::#type Box(int,u8,3,4); Second::#type Box(int,u8,3,4);".to_vec(),
            )
            .unwrap();
        ModuleGraph::load_with_provider(path, GraphOptions::default(), &overlay).unwrap()
    }

    fn source_application<'a>(
        graph: &'a ModuleGraph,
        name: &str,
    ) -> (FileInstanceId, &'a syntax::TypeApplicationSyntax) {
        let declaration = graph
            .declarations()
            .iter()
            .find(|declaration| graph.symbols().name(declaration.name()) == name)
            .unwrap();
        let syntax::FileDeclarationKind::TypeAlias(alias) = &declaration.syntax().kind else {
            panic!("original type alias")
        };
        let syntax::TypeSyntax::Application(application) = &alias.ty else {
            panic!("original application")
        };
        (declaration.file(), application)
    }

    fn key(
        graph: &ModuleGraph,
        name: &str,
        substitution: Option<&Substitution>,
    ) -> ApplicationProofKey {
        let (file, source) = source_application(graph, name);
        ApplicationProofKey::new(
            ApplicationProofSite {
                file,
                source,
                environment: ApplicationEnvironment::Graph,
                owner: ApplicationAnnotationOwner::None,
                enclosing_record: None,
                target: None,
                substitution,
            },
            graph.symbols(),
        )
    }

    #[test]
    fn binding_order_normalizes_without_erasing_supplied_environment() {
        let graph = graph();
        let types = TypeRegistry::new();
        let declaration = &graph.declarations()[0];
        let syntax::FileDeclarationKind::Record(record) = &declaration.syntax().kind else {
            panic!("record formals")
        };
        let mut substitution = Substitution::default();
        substitution.bind_type(
            record.parameters[0].name,
            types.scalar(ScalarType::Int(IntegerType::S64)),
        );
        substitution.bind_type(
            record.parameters[1].name,
            types.scalar(ScalarType::Int(IntegerType::U8)),
        );
        for (parameter, value) in record.parameters[2..].iter().zip([3, 4]) {
            substitution.bind_constant(
                parameter.name,
                BakedValue::Value(jai_ir::ConstantValue {
                    ty: types.scalar(ScalarType::Int(IntegerType::S64)),
                    kind: jai_ir::ConstantKind::Int(Integer::wrapping(IntegerType::S64, value)),
                }),
            );
        }
        let original = key(&graph, "First", Some(&substitution));
        substitution.types.reverse();
        substitution.constants.reverse();
        assert_eq!(original, key(&graph, "First", Some(&substitution)));
        assert_ne!(original, key(&graph, "Second", Some(&substitution)));
        assert_ne!(
            key(&graph, "First", None),
            key(&graph, "First", Some(&Substitution::default()))
        );
        let mut forbidden = original.clone();
        forbidden.owner = ApplicationAnnotationOwner::Forbidden;
        assert_ne!(original, forbidden);
    }

    #[test]
    fn a_ready_proof_does_not_make_another_source_application_ready() {
        let graph = graph();
        let mut types = TypeRegistry::new();
        let nominals = Nominals::reserve(&graph, &mut types).unwrap();
        let mut records = RecordSpecializations::default();
        let (file, application) = source_application(&graph, "First");
        let annotation = syntax::TypeSyntax::Application(application.clone());
        let ty = resolve_type(
            &graph,
            TypeRequest::new(file, &annotation, application.span),
            &mut types,
            &nominals,
            &mut records,
            &mut |file, expression| {
                jai_eval::evaluate_paths(expression, |_, span| {
                    Err(Diagnostic::new(span, "unexpected source dependency"))
                })
                .map_err(|error| crate::modules::located(&graph, file, error))
            },
        )
        .unwrap();
        let initial = records.key_for_type(ty).unwrap().clone();
        let proof = ReadyApplicationProof::checked(ty, initial, &types, &records, application.span)
            .unwrap()
            .unwrap();
        let mut proofs = ApplicationProofs::default();
        proofs
            .remember(key(&graph, "First", None), proof.clone(), application.span)
            .unwrap();
        proofs
            .remember(key(&graph, "First", None), proof, application.span)
            .unwrap();
        assert_eq!(proofs.ready(&key(&graph, "First", None)).unwrap().ty(), ty);
        assert!(proofs.ready(&key(&graph, "Second", None)).is_none());
        assert_eq!(records.records().count(), 1);
    }
}
