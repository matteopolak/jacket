//! Pending modifier recipes carry real source identity without reserving a type.
use super::*;
use jai_source::SourceSpan;
use std::collections::VecDeque;
type WaitSiteKey = (jai_source::SourceId, usize, usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct RecordModifierId(usize);

#[derive(Clone)]
pub(crate) struct RecordModifierIntent {
    pub(crate) key: RecordSpecializationKey,
    pub(crate) file: FileInstanceId,
    pub(crate) record: syntax::RecordDeclaration,
    pub(crate) initial: Substitution,
}

impl RecordModifierIntent {
    pub(crate) fn new(
        graph: &ModuleGraph,
        key: RecordSpecializationKey,
        initial: Substitution,
    ) -> Result<Self, LocatedDiagnostic> {
        let declaration = graph
            .declaration(key.template.0)
            .expect("modifier origin belongs to the retained graph");
        let syntax::FileDeclarationKind::Record(record) = &declaration.syntax().kind else {
            return Err(located(
                graph,
                declaration.file(),
                Diagnostic::new(
                    declaration.location().span,
                    "record modifier origin is not a record declaration",
                ),
            ));
        };
        if record.modify.is_none() {
            return Err(located(
                graph,
                declaration.file(),
                Diagnostic::new(
                    record.span,
                    "record modifier recipe requires actual modifier source",
                ),
            ));
        }
        let arguments = record
            .parameters
            .iter()
            .map(|parameter| {
                initial
                    .constant(parameter.name)
                    .cloned()
                    .or_else(|| initial.ty(parameter.name).map(BakedValue::Type))
            })
            .collect::<Option<Vec<_>>>();
        if arguments.as_deref() != Some(key.arguments.as_ref()) {
            return Err(located(
                graph,
                declaration.file(),
                Diagnostic::new(
                    record.span,
                    "record modifier key differs from its normalized formal bindings",
                ),
            ));
        }
        Ok(Self {
            key,
            file: declaration.file(),
            record: record.clone(),
            initial,
        })
    }
}

#[derive(Clone)]
pub(crate) enum RecordModifierReadiness {
    Pending(RecordModifierId),
    Ready(Substitution),
    Failed(LocatedDiagnostic),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    Queued,
    Waiting,
    Finished,
}

struct Entry {
    intent: RecordModifierIntent,
    state: State,
    result: Option<Result<Substitution, LocatedDiagnostic>>,
    wait_sites: HashMap<WaitSiteKey, SourceSpan>,
}

#[derive(Default)]
pub(crate) struct RecordModifierTable {
    ids: HashMap<RecordSpecializationKey, RecordModifierId>,
    entries: Vec<Entry>,
    queue: VecDeque<RecordModifierId>,
}

impl RecordModifierTable {
    pub(crate) fn completed_count(&self) -> usize {
        self.entries
            .iter()
            .filter(|entry| entry.result.is_some())
            .count()
    }
    pub(crate) fn queued_count(&self) -> usize {
        self.queue.len()
    }

    pub(crate) fn queued_location(&self) -> Option<SourceSpan> {
        let id = self.queue.front()?;
        self.wait_sites(*id)
            .min_by_key(|site| (site.source.index(), site.span.start, site.span.end))
    }
    pub(crate) fn prepare(
        &mut self,
        intent: RecordModifierIntent,
        site: SourceSpan,
    ) -> RecordModifierReadiness {
        let id = match self.ids.get(&intent.key) {
            Some(id) => *id,
            None => {
                let id = RecordModifierId(self.entries.len());
                self.ids.insert(intent.key.clone(), id);
                self.entries.push(Entry {
                    intent,
                    state: State::Queued,
                    result: None,
                    wait_sites: HashMap::new(),
                });
                self.queue.push_back(id);
                id
            }
        };
        self.entries[id.0]
            .wait_sites
            .insert((site.source, site.span.start, site.span.end), site);
        self.readiness(id)
    }

    pub(crate) fn readiness(&self, id: RecordModifierId) -> RecordModifierReadiness {
        match &self.entries[id.0].result {
            Some(Ok(substitution)) => RecordModifierReadiness::Ready(substitution.clone()),
            Some(Err(error)) => RecordModifierReadiness::Failed(error.clone()),
            None => RecordModifierReadiness::Pending(id),
        }
    }

    pub(crate) fn next(&mut self) -> Option<(RecordModifierId, RecordModifierIntent)> {
        let id = self.queue.pop_front()?;
        let entry = &mut self.entries[id.0];
        assert_eq!(entry.state, State::Queued);
        entry.state = State::Waiting;
        Some((id, entry.intent.clone()))
    }

    /// Requeue a genuine dependency retry while preserving the recipe identity.
    pub(crate) fn retry(&mut self, id: RecordModifierId) {
        let entry = &mut self.entries[id.0];
        if entry.state == State::Waiting {
            entry.state = State::Queued;
            self.queue.push_back(id);
        }
    }

    pub(crate) fn finish(
        &mut self,
        id: RecordModifierId,
        result: Result<Substitution, LocatedDiagnostic>,
    ) {
        let entry = &mut self.entries[id.0];
        assert_eq!(entry.state, State::Waiting);
        entry.result = Some(result);
        entry.state = State::Finished;
    }

    pub(crate) fn wait_sites(&self, id: RecordModifierId) -> impl Iterator<Item = SourceSpan> + '_ {
        self.entries[id.0].wait_sites.values().copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_modules::{GraphOptions, SourceOverlay};
    use jai_types::{Integer, IntegerType};

    fn fixture() -> (ModuleGraph, RecordModifierIntent, SourceSpan) {
        let path = std::path::Path::new("/jai-record-modifier-intents/main.jai");
        let mut overlay = SourceOverlay::new();
        overlay
            .insert(
                path,
                b"Buffer::struct(N:int) #modify{if N<4 N=4;return true;} {values:[N]int;}".to_vec(),
            )
            .unwrap();
        let graph =
            ModuleGraph::load_with_provider(path, GraphOptions::default(), &overlay).unwrap();
        let declaration = &graph.declarations()[0];
        let syntax::FileDeclarationKind::Record(record) = &declaration.syntax().kind else {
            panic!()
        };
        let types = TypeRegistry::new();
        let value = BakedValue::Value(jai_ir::ConstantValue {
            ty: types.scalar(jai_types::ScalarType::Int(IntegerType::S64)),
            kind: jai_ir::ConstantKind::Int(Integer::wrapping(IntegerType::S64, 1)),
        });
        let mut initial = Substitution::default();
        initial.bind_constant(record.parameters[0].name, value.clone());
        let key = RecordSpecializationKey {
            template: RecordTemplateId(declaration.id()),
            arguments: vec![value].into_boxed_slice(),
        };
        let site = declaration.location();
        let intent = RecordModifierIntent::new(&graph, key, initial).unwrap();
        (graph, intent, site)
    }

    #[test]
    fn pending_recipes_deduplicate_and_retry_without_type_reservation() {
        let (_graph, intent, site) = fixture();
        let mut table = RecordModifierTable::default();
        let mut types = TypeRegistry::new();
        let first_nominal = types.reserve_record(jai_types::RecordKind::Struct);
        let RecordModifierReadiness::Pending(id) = table.prepare(intent.clone(), site) else {
            panic!()
        };
        assert!(
            matches!(table.prepare(intent.clone(), site), RecordModifierReadiness::Pending(same) if same==id)
        );
        let (job, source) = table.next().unwrap();
        assert_eq!(job, id);
        assert_eq!(source.file, intent.file);
        assert_eq!(
            source.record.modify.as_ref().unwrap().span,
            intent.record.modify.as_ref().unwrap().span
        );
        assert!(table.next().is_none());
        table.retry(id);
        table.retry(id);
        assert_eq!(table.next().unwrap().0, id);
        assert!(table.next().is_none());
        assert_eq!(table.wait_sites(id).collect::<Vec<_>>(), vec![site]);
        table.finish(id, Ok(intent.initial.clone()));
        assert!(
            matches!(table.prepare(intent.clone(), site), RecordModifierReadiness::Ready(value) if value==intent.initial)
        );
        assert!(table.next().is_none());
        let second_nominal = types.reserve_record(jai_types::RecordKind::Struct);
        assert_eq!(second_nominal.index(), first_nominal.index() + 1);
    }

    #[test]
    fn failed_recipes_publish_no_substitution_and_keep_the_source_error() {
        let (graph, intent, site) = fixture();
        let mut table = RecordModifierTable::default();
        let RecordModifierReadiness::Pending(id) = table.prepare(intent.clone(), site) else {
            panic!()
        };
        table.next().unwrap();
        let error = located(
            &graph,
            intent.file,
            Diagnostic::new(
                intent.record.modify.as_ref().unwrap().span,
                "recipe rejected",
            ),
        );
        table.finish(id, Err(error.clone()));
        let RecordModifierReadiness::Failed(actual) = table.prepare(intent, site) else {
            panic!()
        };
        assert_eq!(actual.location, error.location);
        assert_eq!(actual.message, error.message);
        assert!(table.next().is_none());
    }
}
