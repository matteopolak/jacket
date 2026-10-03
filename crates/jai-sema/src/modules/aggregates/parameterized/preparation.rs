//! Type preparation preserves pending recipes separately from source failures.
use super::*;
use jai_source::SourceSpan;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PendingRecordModifier {
    pub(crate) id: modifier_intents::RecordModifierId,
    pub(crate) location: SourceSpan,
}

pub(crate) enum TypePreparation {
    Ready(TypeId),
    Pending(PendingType),
}

#[derive(Debug)]
pub(super) enum TypeFailure {
    Diagnostic(LocatedDiagnostic),
    Pending(PendingType),
}

pub(super) type TypeResult<T> = Result<T, TypeFailure>;

impl From<LocatedDiagnostic> for TypeFailure {
    fn from(error: LocatedDiagnostic) -> Self {
        Self::Diagnostic(error)
    }
}

impl TypeFailure {
    /// Existing one-shot callers cannot retain an unfinished type recipe.
    /// Only their boundary turns pending into an error; recursive resolution
    /// must keep the typed variant so retained preparation can resume it.
    pub(super) fn into_diagnostic(self, graph: &ModuleGraph) -> LocatedDiagnostic {
        match self {
            Self::Diagnostic(error) => error,
            Self::Pending(pending) => pending.diagnostic(graph),
        }
    }
}

impl PendingRecordModifier {
    pub(super) fn diagnostic(self) -> LocatedDiagnostic {
        LocatedDiagnostic {
            location: self.location,
            message: "record #modify requires checked specialization modifier execution".into(),
        }
    }
}

pub(super) fn failure(graph: &ModuleGraph, file: FileInstanceId, error: Diagnostic) -> TypeFailure {
    TypeFailure::Diagnostic(located(graph, file, error))
}

/// Adapt a diagnostic-only helper without losing a recursive pending type.
/// The helper may report its temporary error, but this boundary returns the
/// original typed recipe before callers inspect or retry that diagnostic.
pub(super) fn diagnostic_bridge<T>(
    graph: &ModuleGraph,
    operation: impl FnOnce(
        &mut dyn FnMut(TypeFailure) -> LocatedDiagnostic,
    ) -> Result<T, LocatedDiagnostic>,
) -> TypeResult<T> {
    let mut pending = None;
    let result = operation(&mut |error| match error {
        TypeFailure::Diagnostic(error) => error,
        TypeFailure::Pending(recipe) => {
            pending.get_or_insert(recipe);
            recipe.diagnostic(graph)
        }
    });
    match pending {
        Some(pending) => Err(TypeFailure::Pending(pending)),
        None => result.map_err(TypeFailure::from),
    }
}

pub(crate) fn prepare_type(
    graph: &ModuleGraph,
    request: TypeRequest<'_>,
    types: &mut TypeRegistry,
    nominals: &Nominals<'_>,
    records: &mut RecordSpecializations,
    evaluate: &mut impl FnMut(
        FileInstanceId,
        &syntax::Expression,
    ) -> Result<ScalarConstant, LocatedDiagnostic>,
) -> Result<TypePreparation, LocatedDiagnostic> {
    prepare_type_inner(graph, request, types, nominals, records, evaluate, None)
}

fn prepare_type_inner(
    graph: &ModuleGraph,
    request: TypeRequest<'_>,
    types: &mut TypeRegistry,
    nominals: &Nominals<'_>,
    records: &mut RecordSpecializations,
    evaluate: &mut impl FnMut(
        FileInstanceId,
        &syntax::Expression,
    ) -> Result<ScalarConstant, LocatedDiagnostic>,
    scalar_pending: Option<&std::cell::Cell<Option<PendingType>>>,
) -> Result<TypePreparation, LocatedDiagnostic> {
    let TypeRequest {
        file,
        syntax,
        substitution,
        lexical,
        nominal_context,
        span,
    } = request;
    let result = TypeResolver {
        graph,
        types,
        nominals,
        records,
        evaluate,
        scalar_pending,
        aliases: HashSet::new(),
        lexical,
        lexical_active: lexical.is_some(),
        nominal_context,
    }
    .resolve(file, syntax, substitution, span);
    match result {
        Ok(ty) => Ok(TypePreparation::Ready(ty)),
        Err(TypeFailure::Pending(pending)) => Ok(TypePreparation::Pending(pending)),
        Err(TypeFailure::Diagnostic(error)) => Err(error),
    }
}

pub(crate) fn prepare_type_paired(
    graph: &ModuleGraph,
    request: TypeRequest<'_>,
    types: &mut TypeRegistry,
    nominals: &Nominals<'_>,
    records: &mut RecordSpecializations,
    evaluate: &mut impl FnMut(
        FileInstanceId,
        &syntax::Expression,
    )
        -> Result<crate::modules::constants::ScalarPreparation, LocatedDiagnostic>,
) -> Result<TypePreparation, LocatedDiagnostic> {
    use crate::modules::constants::ScalarPreparation;
    let pending = std::cell::Cell::new(None);
    let result = prepare_type_inner(
        graph,
        request,
        types,
        nominals,
        records,
        &mut |file, expression| match evaluate(file, expression)? {
            ScalarPreparation::Ready(value) => Ok(value),
            ScalarPreparation::Pending(cause) => {
                if pending.get().is_none() {
                    pending.set(Some(cause));
                }
                Err(cause.diagnostic(graph))
            }
        },
        Some(&pending),
    );
    match pending.get() {
        Some(cause) => Ok(TypePreparation::Pending(cause)),
        None => result,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_modules::{GraphOptions, SourceOverlay};

    fn graph() -> ModuleGraph {
        let file = std::path::Path::new("/jai-type-preparation/main.jai");
        let mut overlay = SourceOverlay::new();
        overlay
            .insert(
                file,
                br#"
            Buffer::struct(N:int) #modify {if N<8 N=8;return true;} {
                values:[N]int;
                next:*#this;
            }
            Alias::#type *Buffer(3);
            Defaulted::struct(T:=Buffer(3)) {value:T;}
            DefaultedAlias::#type Defaulted();
            BadAlias::#type Buffer(true);
            Owner::struct {payload:Buffer(3);}
            QueryAlias::#type type_of(Owner.payload);
        "#
                .to_vec(),
            )
            .unwrap();
        ModuleGraph::load_with_provider(file, GraphOptions::default(), &overlay).unwrap()
    }

    fn evaluate(
        graph: &ModuleGraph,
        file: FileInstanceId,
        value: &syntax::Expression,
    ) -> Result<ScalarConstant, LocatedDiagnostic> {
        jai_eval::evaluate_paths(value, |_, span| {
            Err(Diagnostic::new(span, "unexpected dependency"))
        })
        .map_err(|error| located(graph, file, error))
    }

    #[test]
    fn nested_alias_and_inferred_default_keep_one_pending_recipe_without_reserving_types() {
        let graph = graph();
        let mut types = TypeRegistry::new();
        let nominals = Nominals::reserve(&graph, &mut types).unwrap();
        let mut records = RecordSpecializations::default();
        let before = types.reserve_record(jai_types::RecordKind::Struct);
        let mut first = None;
        for index in [1, 3, 1] {
            let source = &graph.declarations()[index];
            let syntax::FileDeclarationKind::TypeAlias(alias) = &source.syntax().kind else {
                panic!()
            };
            let outcome = prepare_type(
                &graph,
                TypeRequest::new(source.file(), &alias.ty, alias.span),
                &mut types,
                &nominals,
                &mut records,
                &mut |file, value| evaluate(&graph, file, value),
            )
            .unwrap();
            let TypePreparation::Pending(PendingType::RecordModifier(pending)) = outcome else {
                panic!("modifier must remain pending")
            };
            assert_eq!(pending.location.source, source.location().source);
            if let Some(id) = first {
                assert_eq!(pending.id, id);
            } else {
                first = Some(pending.id);
            }
        }
        assert_eq!(records.records().count(), 0);
        let after = types.reserve_record(jai_types::RecordKind::Struct);
        assert_eq!(after.index(), before.index() + 1);
        let (id, intent) = records.modifiers.next().unwrap();
        assert_eq!(Some(id), first);
        assert_eq!(
            intent.key.template,
            RecordTemplateId(graph.declarations()[0].id())
        );
        assert!(records.modifiers.next().is_none());
        assert_eq!(records.modifiers.wait_sites(id).count(), 2);
    }

    #[test]
    fn invalid_arguments_remain_source_failures_before_modifier_intake() {
        let graph = graph();
        let mut types = TypeRegistry::new();
        let nominals = Nominals::reserve(&graph, &mut types).unwrap();
        let mut records = RecordSpecializations::default();
        let source = &graph.declarations()[4];
        let syntax::FileDeclarationKind::TypeAlias(alias) = &source.syntax().kind else {
            panic!()
        };
        let Err(error) = prepare_type(
            &graph,
            TypeRequest::new(source.file(), &alias.ty, alias.span),
            &mut types,
            &nominals,
            &mut records,
            &mut |file, value| evaluate(&graph, file, value),
        ) else {
            panic!("a bool cannot bind the int formal")
        };
        assert_eq!(error.location.source, source.location().source);
        assert!(records.modifiers.next().is_none());
    }

    #[test]
    fn annotation_query_keeps_a_pending_field_type_through_static_metadata_preparation() {
        let graph = graph();
        let mut types = TypeRegistry::new();
        let nominals = Nominals::reserve(&graph, &mut types).unwrap();
        let mut records = RecordSpecializations::default();
        let source = &graph.declarations()[6];
        let syntax::FileDeclarationKind::TypeAlias(alias) = &source.syntax().kind else {
            panic!()
        };
        let outcome = prepare_type(
            &graph,
            TypeRequest::new(source.file(), &alias.ty, alias.span),
            &mut types,
            &nominals,
            &mut records,
            &mut |file, value| evaluate(&graph, file, value),
        )
        .unwrap();
        let TypePreparation::Pending(PendingType::RecordModifier(pending)) = outcome else {
            panic!("querying a pending declared field cannot become a source failure")
        };
        assert_eq!(pending.location.source, source.location().source);
        let (id, intent) = records.modifiers.next().unwrap();
        assert_eq!(id, pending.id);
        assert_eq!(
            intent.key.template,
            RecordTemplateId(graph.declarations()[0].id())
        );
        assert!(records.records().next().is_none());
    }

    #[test]
    fn accepted_recipe_materializes_only_the_final_key_and_recursive_owner() {
        let graph = graph();
        let mut types = TypeRegistry::new();
        let nominals = Nominals::reserve(&graph, &mut types).unwrap();
        let mut records = RecordSpecializations::default();
        let source = &graph.declarations()[1];
        let syntax::FileDeclarationKind::TypeAlias(alias) = &source.syntax().kind else {
            panic!()
        };
        let first = prepare_type(
            &graph,
            TypeRequest::new(source.file(), &alias.ty, alias.span),
            &mut types,
            &nominals,
            &mut records,
            &mut |file, value| evaluate(&graph, file, value),
        )
        .unwrap();
        let TypePreparation::Pending(PendingType::RecordModifier(pending)) = first else {
            panic!()
        };
        let (id, intent) = records.modifiers.next().unwrap();
        assert_eq!(id, pending.id);
        let name = intent.record.parameters[0].name;
        let ty = types.scalar(jai_types::ScalarType::Int(jai_types::IntegerType::S64));
        let mut accepted = intent.initial.clone();
        accepted
            .constants
            .iter_mut()
            .find(|binding| binding.name == name)
            .unwrap()
            .value = BakedValue::Value(jai_ir::ConstantValue {
            ty,
            kind: jai_ir::ConstantKind::Int(jai_types::Integer::wrapping(
                jai_types::IntegerType::S64,
                8,
            )),
        });
        // This injects the already-checked executor result into the readiness
        // boundary; it does not claim to execute the source modifier in this test.
        records.modifiers.finish(id, Ok(accepted));
        let outcome = prepare_type(
            &graph,
            TypeRequest::new(source.file(), &alias.ty, alias.span),
            &mut types,
            &nominals,
            &mut records,
            &mut |file, value| evaluate(&graph, file, value),
        )
        .unwrap();
        let TypePreparation::Ready(pointer) = outcome else {
            panic!()
        };
        let jai_types::TypeKind::Pointer(owner) = types.kind(pointer).unwrap() else {
            panic!()
        };
        let owner = *owner;
        let shape = types.record_definition(owner).unwrap();
        assert!(matches!(
            types.kind(shape.fields[0]).unwrap(),
            jai_types::TypeKind::FixedArray { count: 8, .. }
        ));
        assert_eq!(shape.fields[1], pointer);
        assert_eq!(
            records.key_for_type(owner).unwrap().arguments[0],
            BakedValue::Value(jai_ir::ConstantValue {
                ty,
                kind: jai_ir::ConstantKind::Int(jai_types::Integer::wrapping(
                    jai_types::IntegerType::S64,
                    8
                ))
            })
        );
        assert_eq!(records.records().count(), 1);
        assert!(records.modifiers.next().is_none());
    }

    #[test]
    fn concrete_modified_default_pattern_uses_the_original_recipe_and_final_type() {
        let path = std::path::Path::new("/jai-modified-default-pattern/main.jai");
        let mut overlay = SourceOverlay::new();
        overlay
            .insert(
                path,
                b"Buffer::struct(N:int=3) #modify{N+=1;return true;} {values:[N]int;} read::(value:Buffer())->int{return value.values.count;}".to_vec(),
            )
            .unwrap();
        let graph =
            ModuleGraph::load_with_provider(path, GraphOptions::default(), &overlay).unwrap();
        let mut types = TypeRegistry::new();
        let nominals = Nominals::reserve(&graph, &mut types).unwrap();
        let mut records = RecordSpecializations::default();
        let source = &graph.declarations()[1];
        let syntax::FileDeclarationKind::Procedure(procedure) = &source.syntax().kind else {
            panic!()
        };
        let syntax::ParameterBinding::RequiredType(annotation) = &procedure.parameters[0].binding
        else {
            panic!()
        };
        let request = || FormalPatternRequest {
            file: source.file(),
            syntax: annotation,
            span: procedure.parameters[0].span,
            substitution: None,
        };
        let pending = formal_pattern(
            &graph,
            request(),
            &mut types,
            &nominals,
            &mut records,
            &mut |file, value| evaluate(&graph, file, value),
        )
        .unwrap_err();
        assert!(pending.message.contains("#modify"), "{pending:?}");
        let (id, intent) = records.modifiers.next().unwrap();
        let mut accepted = intent.initial;
        let binding = &mut accepted.constants[0];
        let BakedValue::Value(original) = &binding.value else {
            panic!()
        };
        let jai_ir::ConstantKind::Int(original_value) = &original.kind else {
            panic!()
        };
        assert_eq!(original_value.value(), 3);
        binding.value = BakedValue::Value(jai_ir::ConstantValue {
            ty: original.ty,
            kind: jai_ir::ConstantKind::Int(jai_types::Integer::wrapping(
                jai_types::IntegerType::S64,
                4,
            )),
        });
        // Only the accepted-result boundary is exercised here; this test does
        // not execute the original source modifier.
        records.modifiers.finish(id, Ok(accepted));
        let pattern = formal_pattern(
            &graph,
            request(),
            &mut types,
            &nominals,
            &mut records,
            &mut |file, value| evaluate(&graph, file, value),
        )
        .unwrap();
        let crate::overloads::TypePattern::Concrete(owner) = pattern else {
            panic!("modified default annotation must use its canonical type")
        };
        let shape = types.record_definition(owner).unwrap();
        assert!(matches!(
            types.kind(shape.fields[0]).unwrap(),
            jai_types::TypeKind::FixedArray { count: 4, .. }
        ));
        assert!(records.record(owner).unwrap().defaults.is_empty());
        assert!(records.modifiers.next().is_none());
        assert_eq!(records.records().count(), 1);
    }
}
