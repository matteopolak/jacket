//! Stage accepted record bindings before their source-dependent layout is ready.
//!
//! This helper is deliberately unregistered until source discovery has a
//! record-specific job. Its values belong to the current semantic registry;
//! transporting them to a graph requires a separate checked source codec.
use super::{RecordSpecializationKey, RecordSpecializations, syntax};
use crate::polymorphism::BakedValue;
use crate::{TypeId, TypeRegistry};
use jai_modules::{FileInstanceId, ModuleGraph};
use jai_source::{DeclarationId, LocatedDiagnostic, SourceSpan, Symbol};

/// The reservation is produced only after the source binder has normalized
/// accepted modifier output. It does not imply that the record body is ready.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct AcceptedRecordSourceJob {
    owner: TypeId,
    file: FileInstanceId,
    declaration: SourceSpan,
    record: SourceSpan,
    key: RecordSpecializationKey,
    bindings: Box<[(Symbol, BakedValue)]>,
}

impl AcceptedRecordSourceJob {
    pub(super) fn from_reservation(
        graph: &ModuleGraph,
        types: &TypeRegistry,
        records: &RecordSpecializations,
        owner: TypeId,
        demand: SourceSpan,
    ) -> Result<Self, LocatedDiagnostic> {
        let fail = |message: &str| LocatedDiagnostic {
            location: demand,
            message: message.into(),
        };
        if !matches!(types.kind(owner), Ok(jai_types::TypeKind::Record(_))) {
            return Err(fail(
                "a source record job requires an actual record reservation",
            ));
        }
        let key = records
            .key_for_type(owner)
            .ok_or_else(|| fail("a source record job requires its canonical specialization key"))?;
        let declaration = graph
            .declaration(key.template.0)
            .ok_or_else(|| fail("a source record job requires its original graph declaration"))?;
        let syntax::FileDeclarationKind::Record(record) = &declaration.syntax().kind else {
            return Err(fail(
                "a source record job does not identify a record template",
            ));
        };
        if record.parameters.is_empty() || record.parameters.len() != key.arguments.len() {
            return Err(fail(
                "a source record job differs from its original formal list",
            ));
        }
        if let Some(ready) = records.record(owner)
            && (ready.nested || ready.origin != Some(key.template))
        {
            return Err(fail(
                "a source record job differs from its materialized origin",
            ));
        }
        if let Ok(shape) = types.record_definition(owner)
            && shape.kind != record.kind
        {
            return Err(fail(
                "a source record job differs from its original record kind",
            ));
        }
        let file = graph
            .file(declaration.file())
            .ok_or_else(|| fail("a source record job requires its retained defining file"))?;
        if file.source() != declaration.location().source {
            return Err(fail("a source record job has a mismatched defining source"));
        }
        let bindings = record
            .parameters
            .iter()
            .zip(key.arguments.iter())
            .map(|(parameter, value)| (parameter.name, value.clone()))
            .collect();
        Ok(Self {
            owner,
            file: declaration.file(),
            // Keep the original full declaration extent independently of the
            // record's own syntax span. Neither is reconstructed from fields.
            declaration: declaration.location(),
            record: SourceSpan {
                source: file.source(),
                span: record.span,
            },
            key: key.clone(),
            bindings,
        })
    }

    pub(super) fn owner(&self) -> TypeId {
        self.owner
    }

    pub(super) fn template(&self) -> DeclarationId {
        self.key.template.0
    }

    pub(super) fn file(&self) -> FileInstanceId {
        self.file
    }

    pub(super) fn declaration(&self) -> SourceSpan {
        self.declaration
    }

    pub(super) fn record(&self) -> SourceSpan {
        self.record
    }

    pub(super) fn final_key(&self) -> &RecordSpecializationKey {
        &self.key
    }

    pub(super) fn bindings(&self) -> &[(Symbol, BakedValue)] {
        &self.bindings
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::aggregates::parameterized::{
        RecordTemplateId, TypePreparation, TypeRequest, prepare_type,
    };
    use crate::modules::aggregates::types::Nominals;
    use jai_modules::{GraphOptions, SourceOverlay};
    use jai_types::{Integer, IntegerType, RecordKind};

    #[test]
    fn accepted_final_bindings_can_be_retained_before_the_shape_is_ready() {
        let path = std::path::Path::new("/jai-record-source-job/main.jai");
        let mut overlay = SourceOverlay::new();
        overlay
            .insert(
                path,
                b"Buffer::struct(N:int=3) #modify{N+=1;return true;} {values:[N]int;missing:NotYetImported;} Alias::#type Buffer();".to_vec(),
            )
            .unwrap();
        let graph =
            ModuleGraph::load_with_provider(path, GraphOptions::default(), &overlay).unwrap();
        let mut types = TypeRegistry::new();
        let nominals = Nominals::reserve(&graph, &mut types).unwrap();
        let mut records = RecordSpecializations::default();
        let declaration = &graph.declarations()[0];
        let source = &graph.declarations()[1];
        let syntax::FileDeclarationKind::TypeAlias(alias) = &source.syntax().kind else {
            panic!("original alias");
        };
        let mut evaluate = |_, expression: &syntax::Expression| {
            jai_eval::evaluate_paths(expression, |_, span| {
                Err(crate::Diagnostic::new(span, "unexpected source dependency"))
            })
            .map_err(|error| crate::modules::located(&graph, source.file(), error))
        };
        let outcome = prepare_type(
            &graph,
            TypeRequest::new(source.file(), &alias.ty, alias.span),
            &mut types,
            &nominals,
            &mut records,
            &mut evaluate,
        )
        .unwrap();
        let TypePreparation::Pending(PendingType::RecordModifier(pending)) = outcome else {
            panic!("the original modifier recipe must remain pending");
        };
        let (id, intent) = records.modifiers.next().unwrap();
        assert_eq!(id, pending.id);
        assert_eq!(intent.key.arguments[0].as_integer().unwrap().value(), 3);
        let name = intent.record.parameters[0].name;
        let final_value = BakedValue::integer(Integer::wrapping(IntegerType::S64, 4), &types);
        let mut accepted = intent.initial;
        accepted.constants[0].value = final_value.clone();
        // Exercise the accepted-result handoff, rather than claim source VM
        // execution of this modifier in a metadata-only test.
        records.modifiers.finish(id, Ok(accepted));
        let error = prepare_type(
            &graph,
            TypeRequest::new(source.file(), &alias.ty, alias.span),
            &mut types,
            &nominals,
            &mut records,
            &mut evaluate,
        )
        .err()
        .expect("the field still needs its actual source dependency");
        assert!(error.message.contains("NotYetImported"), "{error:?}");
        let final_key = RecordSpecializationKey {
            template: RecordTemplateId(declaration.id()),
            arguments: vec![final_value.clone()].into_boxed_slice(),
        };
        // Retrying the canonical reservation reuses the actual identity that
        // the failed body attempt reserved. It must not allocate a second one.
        let boundary = types.reserve_record(RecordKind::Struct);
        let owner = match records.reserve(final_key.clone(), RecordKind::Struct, &mut types) {
            super::super::Reservation::Existing(owner)
            | super::super::Reservation::Resolve(owner) => owner,
        };
        let next = types.reserve_record(RecordKind::Struct);
        assert_eq!(next.index(), boundary.index() + 1);
        assert!(types.record_definition(owner).is_err());
        assert!(records.record(owner).is_none());
        let job = AcceptedRecordSourceJob::from_reservation(
            &graph,
            &types,
            &records,
            owner,
            source.location(),
        )
        .unwrap();
        assert_eq!(job.owner(), owner);
        assert_eq!(job.template(), declaration.id());
        assert_eq!(job.file(), declaration.file());
        assert_eq!(job.declaration(), declaration.location());
        assert_eq!(job.final_key(), &final_key);
        assert_eq!(job.bindings(), &[(name, final_value)]);
        assert!(job.declaration().span.end >= job.record().span.end);
        assert!(records.modifiers.next().is_none());
    }
}
