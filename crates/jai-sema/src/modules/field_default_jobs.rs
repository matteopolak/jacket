//! Retain typed record initializer recipes until their real providers are ready.
use super::*;
use crate::local_declarations::FieldMetadata;
use crate::polymorphism::Substitution;
use aggregates::parameterized::{RecordDefaultOverride, RecordSpecializations};
use jai_ir::ConstantValue as TypedConstant;
use jai_types::{FieldId, TypeKind};
use std::collections::{HashSet, VecDeque};

#[derive(Clone)]
pub(crate) struct FieldDefaultJob {
    pub field: FieldId,
    pub owner: TypeId,
    pub file: FileInstanceId,
    pub location: jai_source::SourceSpan,
    pub metadata: FieldMetadata,
    pub substitution: Substitution,
    pub overrides: Vec<RecordDefaultOverride>,
    pub fields: Vec<FieldId>,
}

impl FieldDefaultJob {
    fn is_no_write(&self) -> bool {
        self.metadata
            .syntax
            .initializer()
            .is_some_and(|expression| {
                matches!(expression.kind, syntax::ExpressionKind::Uninitialized)
            })
    }
    pub(crate) fn has_explicit_overlay(&self) -> bool {
        self.metadata.syntax.initializer().is_some() || !self.overrides.is_empty()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum FieldDefaultDependency {
    Field(FieldId),
    Vm(jai_vm::Dependency),
    Constant(DeclarationId),
}

pub(super) enum FieldDefaultOutcome {
    NoWrite,
    Ready(TypedConstant),
    Pending {
        dependencies: Vec<FieldDefaultDependency>,
        diagnostic: LocatedDiagnostic,
    },
    Failed(LocatedDiagnostic),
}

pub(crate) enum FieldDefaultReadiness<'a> {
    NoWrite(&'a FieldDefaultJob),
    Pending(&'a FieldDefaultJob),
    Ready(&'a TypedConstant),
    Failed(&'a LocatedDiagnostic),
}

enum JobState {
    NoWrite,
    Dormant,
    Queued,
    Running,
    Waiting {
        dependencies: Vec<FieldDefaultDependency>,
        diagnostic: LocatedDiagnostic,
    },
    Ready(TypedConstant),
    Failed(LocatedDiagnostic),
}

struct Entry {
    job: FieldDefaultJob,
    state: JobState,
}

#[derive(Default)]
pub(crate) struct FieldDefaultJobs {
    entries: HashMap<FieldId, Entry>,
    queue: VecDeque<FieldId>,
}

pub(super) struct FieldDefaultSources<'a, 'graph> {
    pub graph: &'graph ModuleGraph,
    pub nominals: &'a Nominals<'graph>,
    pub records: &'a RecordSpecializations,
    pub types: &'a TypeRegistry,
    pub deferred_constants: &'a HashSet<DeclarationId>,
}

pub(crate) struct PreparedFieldDefaults {
    pub ready: HashMap<FieldId, TypedConstant>,
    pub pending: HashSet<FieldId>,
}

impl Resolver<'_> {
    pub(crate) fn collect_record_field_default_jobs(&mut self) -> Result<(), Diagnostic> {
        let Some(scope) = self.graph_scope else {
            return Ok(());
        };
        let graph = scope.declarations.graph;
        let deferred = deferred_constants::classify(graph);
        self.meta
            .field_default_jobs
            .collect(FieldDefaultSources {
                graph,
                nominals: &scope.declarations.nominals,
                records: &self.meta.record_specializations,
                types: self.types,
                deferred_constants: self
                    .compile_time
                    .map_or(&deferred, |context| context.deferred),
            })
            .map_err(|error| Diagnostic::at_source(error.location, error.message))
    }
}

pub(super) fn execute(resolver: &mut Resolver<'_>, job: &FieldDefaultJob) -> FieldDefaultOutcome {
    if job.is_no_write() {
        return FieldDefaultOutcome::NoWrite;
    }
    if let Some(key) = resolver.meta.record_specializations.key_for_type(job.owner) {
        let graph = resolver
            .graph_scope
            .expect("source field job has a retained graph")
            .declarations
            .graph;
        let declaration = graph
            .declaration(key.template.0)
            .expect("field recipe origin exists");
        let FileDeclarationKind::Record(record) = &declaration.syntax().kind else {
            unreachable!("record specialization retains its actual declaration");
        };
        let captured = record
            .parameters
            .iter()
            .map(|parameter| {
                job.substitution
                    .constant(parameter.name)
                    .cloned()
                    .or_else(|| {
                        job.substitution
                            .ty(parameter.name)
                            .map(crate::polymorphism::BakedValue::Type)
                    })
            })
            .collect::<Option<Vec<_>>>();
        if captured.as_deref() != Some(key.arguments.as_ref()) {
            return FieldDefaultOutcome::Failed(LocatedDiagnostic {
                location: job.location,
                message: "record field recipe differs from its canonical specialization bindings"
                    .into(),
            });
        }
    }
    let result = resolver.with_record_source_environment(job.owner, job.location.span, |child| {
        let result = (|| {
            let mut value = match job.metadata.syntax.initializer() {
                Some(expression) => child.local_typed_constant(expression, job.metadata.ty)?,
                None => child.field_type_default_value(&job.metadata)?,
            };
            for override_ in &job.overrides {
                let target = *override_
                    .path
                    .last()
                    .expect("checked override has a field path");
                let expected = child
                    .types
                    .field_type(target)
                    .map_err(|error| Diagnostic::new(override_.span, error.to_string()))?;
                let replacement = child.local_typed_constant(&override_.value, expected)?;
                value = crate::record_default_overrides::replace_constant(
                    value,
                    &override_.path[1..],
                    replacement,
                    child.types,
                    override_.span,
                )?;
            }
            Ok(value)
        })();
        let mut dependencies = vec![];
        if let Some(context) = child.compile_time {
            dependencies.extend(
                context
                    .pending_field_defaults
                    .borrow()
                    .iter()
                    .copied()
                    .map(FieldDefaultDependency::Field),
            );
            dependencies.extend(
                context
                    .pending
                    .borrow()
                    .iter()
                    .cloned()
                    .map(FieldDefaultDependency::Vm),
            );
            dependencies.extend(
                context
                    .pending_constants
                    .borrow()
                    .iter()
                    .copied()
                    .map(FieldDefaultDependency::Constant),
            );
        }
        Ok(match result {
            Ok(value) => FieldDefaultOutcome::Ready(value),
            Err(error) => {
                let diagnostic = LocatedDiagnostic::new(job.location.source, error);
                if dependencies.is_empty() {
                    FieldDefaultOutcome::Failed(diagnostic)
                } else {
                    FieldDefaultOutcome::Pending {
                        dependencies,
                        diagnostic,
                    }
                }
            }
        })
    });
    result.unwrap_or_else(|error| {
        FieldDefaultOutcome::Failed(LocatedDiagnostic::new(job.location.source, error))
    })
}

pub(super) fn sweep(resolver: &mut Resolver<'_>) -> Result<(), LocatedDiagnostic> {
    let scope = resolver
        .graph_scope
        .expect("source defaults use a retained file scope");
    let graph = scope.declarations.graph;
    resolver
        .meta
        .field_default_jobs
        .collect(FieldDefaultSources {
            graph,
            nominals: &scope.declarations.nominals,
            records: &resolver.meta.record_specializations,
            types: resolver.types,
            deferred_constants: resolver
                .compile_time
                .expect("field jobs require a real provider")
                .deferred,
        })?;
    resolver.meta.field_default_jobs.retry_waiting();
    while let Some(job) = resolver.meta.field_default_jobs.next() {
        let preparation = if resolver
            .meta
            .record_specializations
            .record(job.owner)
            .is_none()
        {
            let declaration = scope.declarations.nominals.records[&job.owner].declaration;
            aggregates::parameterized::materialize_static_record(
                graph,
                declaration,
                resolver.types,
                &scope.declarations.nominals,
                &mut resolver.meta.record_specializations,
                &mut |file, expression| {
                    jai_eval::evaluate_paths(expression, |path, span| {
                        match scope.code_file(file).value(path, span)? {
                            Binding::Constant(value) => Ok(value),
                            Binding::Enum(value) => Ok(ConstantValue::Int(value.value)),
                            _ => Err(Diagnostic::new(
                                span,
                                "record declaration requires a scalar constant",
                            )),
                        }
                    })
                    .map_err(|error| located(graph, file, error))
                },
            )
        } else {
            Ok(())
        };
        let outcome = match preparation {
            Ok(()) => execute(resolver, &job),
            Err(error) => FieldDefaultOutcome::Failed(error),
        };
        resolver
            .meta
            .field_default_jobs
            .finish(job.field, outcome, resolver.types);
    }
    let mut fields = resolver.meta.field_default_jobs.active_fields();
    fields.sort_by_key(|field| (field.record().index(), field.index()));
    if let Some(field) = fields.first().copied() {
        match resolver
            .meta
            .field_default_jobs
            .readiness(field)
            .expect("blocked field has a recipe")
        {
            FieldDefaultReadiness::Failed(error) => return Err(error.clone()),
            FieldDefaultReadiness::Pending(job) => {
                let diagnostic = resolver
                    .meta
                    .field_default_jobs
                    .waiting()
                    .find_map(|(candidate, _, error)| (candidate == field).then(|| error.clone()))
                    .unwrap_or_else(|| LocatedDiagnostic {
                        location: job.location,
                        message: "record field initializer job is pending".into(),
                    });
                return Err(diagnostic);
            }
            FieldDefaultReadiness::Ready(_) | FieldDefaultReadiness::NoWrite(_) => {
                unreachable!("completed fields are not active")
            }
        }
    }
    Ok(())
}

impl FieldDefaultJobs {
    /// Add newly materialized owners without changing an existing recipe's identity.
    pub(super) fn collect(
        &mut self,
        sources: FieldDefaultSources<'_, '_>,
    ) -> Result<(), LocatedDiagnostic> {
        let mut fields = HashMap::new();
        for (&owner, record) in &sources.nominals.records {
            for field in &record.fields {
                fields.insert(
                    field.id,
                    FieldDefaultJob {
                        field: field.id,
                        owner,
                        file: record.file,
                        location: field_location(sources.graph, record.file, field.syntax.span),
                        metadata: FieldMetadata {
                            name: Some(field.name),
                            id: field.id,
                            ty: field.ty,
                            syntax: field.syntax.clone().into(),
                        },
                        substitution: Substitution::default(),
                        overrides: sources.records.default_overrides(field.id).to_vec(),
                        fields: vec![],
                    },
                );
            }
        }
        for (owner, record) in sources.records.records() {
            for field in &record.shape.fields {
                fields.insert(
                    field.id,
                    FieldDefaultJob {
                        field: field.id,
                        owner,
                        file: record.file,
                        location: field_location(sources.graph, record.file, field.syntax.span()),
                        metadata: field.clone(),
                        substitution: record.substitution.clone(),
                        overrides: sources.records.default_overrides(field.id).to_vec(),
                        fields: vec![],
                    },
                );
            }
        }
        let mut deferred = fields
            .values()
            .filter(|job| {
                job.metadata
                    .syntax
                    .initializer()
                    .is_some_and(|expression| needs_provider(&sources, job.file, expression))
                    || job
                        .overrides
                        .iter()
                        .any(|override_| needs_provider(&sources, job.file, &override_.value))
            })
            .map(|job| job.field)
            .collect::<HashSet<_>>();
        let owners = fields_by_owner(&fields);
        for job in fields.values_mut() {
            job.fields =
                dependent_fields(sources.types, &owners, job.metadata.ty).map_err(|error| {
                    located(
                        sources.graph,
                        job.file,
                        Diagnostic::new(job.location.span, error.to_string()),
                    )
                })?;
        }
        loop {
            let before = deferred.len();
            for job in fields.values_mut() {
                if job.fields.iter().any(|field| deferred.contains(field)) {
                    deferred.insert(job.field);
                }
            }
            if deferred.len() == before {
                break;
            }
        }
        let mut pending = fields
            .into_values()
            .filter(|job| job.is_no_write() || deferred.contains(&job.field))
            .collect::<Vec<_>>();
        pending.sort_by_key(|job| (job.owner.index(), job.field.index()));
        for job in pending {
            if let std::collections::hash_map::Entry::Vacant(entry) = self.entries.entry(job.field)
            {
                let state = if job.is_no_write() {
                    JobState::NoWrite
                } else if job.has_explicit_overlay() {
                    self.queue.push_back(job.field);
                    JobState::Queued
                } else {
                    JobState::Dormant
                };
                entry.insert(Entry {
                    job,
                    state,
                });
            }
        }
        Ok(())
    }

    pub(super) fn blocked_fields(&self) -> HashSet<FieldId> {
        self.entries
            .iter()
            .filter_map(|(&field, entry)| {
                (!matches!(entry.state, JobState::Ready(_))).then_some(field)
            })
            .collect()
    }

    pub(crate) fn readiness(&self, field: FieldId) -> Option<FieldDefaultReadiness<'_>> {
        self.entries.get(&field).map(|entry| match &entry.state {
            JobState::NoWrite => FieldDefaultReadiness::NoWrite(&entry.job),
            JobState::Ready(value) => FieldDefaultReadiness::Ready(value),
            JobState::Failed(error) => FieldDefaultReadiness::Failed(error),
            _ => FieldDefaultReadiness::Pending(&entry.job),
        })
    }

    pub(super) fn request(&mut self, field: FieldId) {
        if let Some(entry) = self.entries.get_mut(&field)
            && matches!(entry.state, JobState::Dormant)
        {
            entry.state = JobState::Queued;
            self.queue.push_back(field);
        }
    }

    pub(super) fn requested_count(&self) -> usize {
        self.entries
            .values()
            .filter(|entry| !matches!(entry.state, JobState::Dormant))
            .count()
    }

    fn active_fields(&self) -> Vec<FieldId> {
        self.entries
            .iter()
            .filter_map(|(&field, entry)| {
                (!matches!(
                    entry.state,
                    JobState::Dormant | JobState::Ready(_) | JobState::NoWrite
                ))
                .then_some(field)
            })
            .collect()
    }

    pub(super) fn pending_fields(&self) -> HashSet<FieldId> {
        self.active_fields().into_iter().collect()
    }

    pub(super) fn prerequisite_procedures(&self) -> HashSet<jai_ir::ProcedureId> {
        self.waiting()
            .flat_map(|(_, dependencies, _)| dependencies)
            .filter_map(|dependency| match dependency {
                FieldDefaultDependency::Vm(jai_vm::Dependency::Procedure(id)) => Some(*id),
                _ => None,
            })
            .collect()
    }

    pub(super) fn next(&mut self) -> Option<FieldDefaultJob> {
        let field = self.queue.pop_front()?;
        let entry = self
            .entries
            .get_mut(&field)
            .expect("queued field has a source recipe");
        assert!(matches!(entry.state, JobState::Queued));
        entry.state = JobState::Running;
        Some(entry.job.clone())
    }

    pub(super) fn finish(
        &mut self,
        field: FieldId,
        outcome: FieldDefaultOutcome,
        types: &TypeRegistry,
    ) {
        let entry = self
            .entries
            .get_mut(&field)
            .expect("field recipe was reserved");
        assert!(matches!(entry.state, JobState::Running));
        entry.state = match outcome {
            FieldDefaultOutcome::NoWrite => {
                if entry.job.is_no_write()
                    && types
                        .validate_field(entry.job.owner, field)
                        .is_ok_and(|ty| ty == entry.job.metadata.ty)
                {
                    JobState::NoWrite
                } else {
                    JobState::Failed(LocatedDiagnostic {
                        location: entry.job.location,
                        message: "no-write field recipe differs from its canonical field or source initializer".into(),
                    })
                }
            }
            FieldDefaultOutcome::Ready(value) => {
                let checked = types
                    .validate_field(entry.job.owner, field)
                    .is_ok_and(|ty| ty == entry.job.metadata.ty && ty == value.ty)
                    && crate::constant_limits::cells(&value).is_some();
                if checked {
                    JobState::Ready(value)
                } else {
                    JobState::Failed(LocatedDiagnostic {
                        location: entry.job.location,
                        message: "record field default differs from its canonical field type or exceeds the constant budget".into(),
                    })
                }
            }
            FieldDefaultOutcome::Pending {
                dependencies,
                diagnostic,
            } => JobState::Waiting {
                dependencies,
                diagnostic,
            },
            FieldDefaultOutcome::Failed(error) => JobState::Failed(error),
        };
    }

    pub(super) fn retry_waiting(&mut self) {
        let mut fields = self
            .entries
            .iter()
            .filter_map(|(&field, entry)| {
                matches!(entry.state, JobState::Waiting { .. }).then_some(field)
            })
            .collect::<Vec<_>>();
        fields.sort_by_key(|field| (field.record().index(), field.index()));
        for field in fields {
            self.entries.get_mut(&field).unwrap().state = JobState::Queued;
            self.queue.push_back(field);
        }
    }

    pub(super) fn ready(&self) -> impl Iterator<Item = (FieldId, &TypedConstant)> {
        self.entries
            .iter()
            .filter_map(|(&field, entry)| match &entry.state {
                JobState::Ready(value) => Some((field, value)),
                _ => None,
            })
    }

    pub(super) fn waiting(
        &self,
    ) -> impl Iterator<Item = (FieldId, &[FieldDefaultDependency], &LocatedDiagnostic)> {
        self.entries
            .iter()
            .filter_map(|(&field, entry)| match &entry.state {
                JobState::Waiting {
                    dependencies,
                    diagnostic,
                } => Some((field, dependencies.as_slice(), diagnostic)),
                _ => None,
            })
    }
}

fn field_location(graph: &ModuleGraph, file: FileInstanceId, span: Span) -> jai_source::SourceSpan {
    jai_source::SourceSpan {
        source: graph
            .file(file)
            .expect("record field source exists")
            .source(),
        span,
    }
}

fn needs_provider(
    sources: &FieldDefaultSources<'_, '_>,
    file: FileInstanceId,
    expression: &syntax::Expression,
) -> bool {
    let mut needs = false;
    deferred_constants::visit(expression, |expression| {
        use syntax::ExpressionKind as E;
        if matches!(
            expression.kind,
            E::CompileTime(_) | E::ShortLambda(_) | E::AnonymousProcedure(_) | E::BakeArguments(_)
        ) {
            needs = true;
        }
        let path = match &expression.kind {
            E::Name(name) => Some(path(*name)),
            E::QualifiedName(path) => Some(path.clone()),
            _ => None,
        };
        if let Some(path) = path
            && let Ok(jai_modules::Binding::Declaration(id)) = sources.graph.lookup(file, &path)
            && sources.deferred_constants.contains(&id)
        {
            needs = true;
        }
    });
    needs
}

pub(crate) fn contains_typed_leaf(expression: &syntax::Expression) -> bool {
    let mut found = false;
    deferred_constants::visit(expression, |expression| {
        found |= matches!(
            expression.kind,
            syntax::ExpressionKind::CompileTime(_)
                | syntax::ExpressionKind::ShortLambda(_)
                | syntax::ExpressionKind::AnonymousProcedure(_)
                | syntax::ExpressionKind::BakeArguments(_)
        );
    });
    found
}

fn fields_by_owner(fields: &HashMap<FieldId, FieldDefaultJob>) -> HashMap<TypeId, Vec<FieldId>> {
    let mut owners: HashMap<_, Vec<_>> = HashMap::new();
    for job in fields.values() {
        owners.entry(job.owner).or_default().push(job.field);
    }
    owners
}

fn dependent_fields(
    types: &TypeRegistry,
    owners: &HashMap<TypeId, Vec<FieldId>>,
    ty: TypeId,
) -> Result<Vec<FieldId>, jai_types::TypeError> {
    let mut pending = vec![ty];
    let mut visited = HashSet::new();
    let mut fields = vec![];
    while let Some(ty) = pending.pop() {
        if !visited.insert(ty) {
            continue;
        }
        match types.kind(ty)? {
            TypeKind::Record(_) | TypeKind::Any(_) => {
                fields.extend(owners.get(&ty).into_iter().flatten().copied())
            }
            TypeKind::FixedArray {
                element,
                count,
            } if *count > 0 => pending.push(*element),
            _ => {}
        }
    }
    Ok(fields)
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_modules::{GraphOptions, SourceOverlay};

    #[test]
    fn no_write_recipe_is_completed_without_a_typed_value_or_pending_job() {
        let path = std::path::Path::new("/jai-no-write-field/main.jai");
        let mut source = SourceOverlay::new();
        source
            .insert(path, b"Record::struct{skipped:int=---;}".to_vec())
            .unwrap();
        let graph =
            ModuleGraph::load_with_provider(path, GraphOptions::default(), &source).unwrap();
        let mut types = TypeRegistry::new();
        let mut nominals = Nominals::reserve(&graph, &mut types).unwrap();
        let mut records = RecordSpecializations::default();
        nominals
            .define_records_with_specializations(
                &graph,
                &mut types,
                &mut records,
                &mut |file, expression| {
                    Err(located(
                        &graph,
                        file,
                        Diagnostic::new(expression.span, "no-write must not be evaluated"),
                    ))
                },
            )
            .unwrap();
        let deferred = HashSet::new();
        let mut jobs = FieldDefaultJobs::default();
        let sources = || FieldDefaultSources {
            graph: &graph,
            nominals: &nominals,
            records: &records,
            types: &types,
            deferred_constants: &deferred,
        };
        jobs.collect(sources()).unwrap();
        let field = nominals.records.values().next().unwrap().fields[0].id;
        let Some(FieldDefaultReadiness::NoWrite(recipe)) = jobs.readiness(field) else {
            panic!("completed source no-write recipe");
        };
        assert_eq!(recipe.field, field);
        assert!(recipe.is_no_write());
        assert!(jobs.blocked_fields().contains(&field));
        assert!(jobs.pending_fields().is_empty());
        assert!(jobs.ready().next().is_none());
        jobs.request(field);
        assert!(jobs.next().is_none());
        jobs.collect(sources()).unwrap();
        assert!(matches!(
            jobs.readiness(field),
            Some(FieldDefaultReadiness::NoWrite(_))
        ));
    }

    fn fixture() -> ModuleGraph {
        let path = std::path::Path::new("/jai-field-default-jobs/main.jai");
        let mut source = SourceOverlay::new();
        source.insert(path, b"Base::struct{value:int=#run seed();} Container::struct{base:Base;pointer:*Base;empty:[0]Base;plain:int=3;} seed::()->int{return 21;}".to_vec()).unwrap();
        ModuleGraph::load_with_provider(path, GraphOptions::default(), &source).unwrap()
    }

    #[test]
    fn genuine_field_recipes_include_transitive_value_defaults_and_reuse_ids() {
        let graph = fixture();
        let mut types = TypeRegistry::new();
        let mut nominals = Nominals::reserve(&graph, &mut types).unwrap();
        let mut records = RecordSpecializations::default();
        nominals
            .define_records_with_specializations(
                &graph,
                &mut types,
                &mut records,
                &mut |file, expression| {
                    jai_eval::evaluate_paths(expression, |_, span| {
                        Err(Diagnostic::new(span, "unexpected lookup"))
                    })
                    .map_err(|error| located(&graph, file, error))
                },
            )
            .unwrap();
        let deferred = HashSet::new();
        let sources = || FieldDefaultSources {
            graph: &graph,
            nominals: &nominals,
            records: &records,
            types: &types,
            deferred_constants: &deferred,
        };
        let mut jobs = FieldDefaultJobs::default();
        jobs.collect(sources()).unwrap();
        let blocked = jobs.blocked_fields();
        assert_eq!(blocked.len(), 2);
        jobs.collect(sources()).unwrap();
        assert_eq!(jobs.blocked_fields(), blocked);
        let first = jobs.next().unwrap();
        assert!(jobs.next().is_none());
        let second_field = *blocked.iter().find(|&&field| field != first.field).unwrap();
        let Some(FieldDefaultReadiness::Pending(recipe)) = jobs.readiness(second_field) else {
            panic!("implicit aggregate retains its pending source recipe");
        };
        assert!(!recipe.has_explicit_overlay());
        jobs.request(second_field);
        jobs.request(second_field);
        let second = jobs.next().unwrap();
        assert!(jobs.next().is_none());
        assert_eq!(first.file, graph.declarations()[0].file());
        assert!(matches!(
            first.metadata.syntax.initializer().unwrap().kind,
            syntax::ExpressionKind::CompileTime(_)
        ));
        assert_eq!(second.fields, vec![first.field]);
        assert!(matches!(
            jobs.readiness(first.field),
            Some(FieldDefaultReadiness::Pending(_))
        ));
        jobs.finish(
            first.field,
            FieldDefaultOutcome::Pending {
                dependencies: vec![
                    FieldDefaultDependency::Field(second.field),
                    FieldDefaultDependency::Vm(jai_vm::Dependency::Procedure(
                        jai_ir::ProcedureId::new(9),
                    )),
                ],
                diagnostic: LocatedDiagnostic {
                    location: first.location,
                    message: "not ready".into(),
                },
            },
            &types,
        );
        assert_eq!(
            jobs.waiting().next().unwrap().1,
            &[
                FieldDefaultDependency::Field(second.field),
                FieldDefaultDependency::Vm(jai_vm::Dependency::Procedure(
                    jai_ir::ProcedureId::new(9)
                )),
            ]
        );
        assert_eq!(
            jobs.prerequisite_procedures(),
            HashSet::from([jai_ir::ProcedureId::new(9)])
        );
        jobs.retry_waiting();
        assert_eq!(jobs.next().unwrap().field, first.field);
        jobs.finish(
            first.field,
            FieldDefaultOutcome::Ready(TypedConstant {
                ty: first.metadata.ty,
                kind: jai_ir::ConstantKind::Int(jai_types::Integer::wrapping(
                    jai_types::IntegerType::S64,
                    21,
                )),
            }),
            &types,
        );
        assert_eq!(jobs.ready().count(), 1);
        assert!(jobs.prerequisite_procedures().is_empty());
        assert!(!jobs.blocked_fields().contains(&first.field));
        assert!(matches!(
            jobs.readiness(second.field),
            Some(FieldDefaultReadiness::Pending(_))
        ));
        jobs.collect(sources()).unwrap();
        assert!(jobs.next().is_none());
    }

    #[test]
    fn failed_or_incompatible_field_recipes_never_publish_an_initializer() {
        let graph = fixture();
        let mut types = TypeRegistry::new();
        let mut nominals = Nominals::reserve(&graph, &mut types).unwrap();
        let mut records = RecordSpecializations::default();
        nominals
            .define_records_with_specializations(
                &graph,
                &mut types,
                &mut records,
                &mut |file, expression| {
                    jai_eval::evaluate_paths(expression, |_, span| {
                        Err(Diagnostic::new(span, "unexpected lookup"))
                    })
                    .map_err(|error| located(&graph, file, error))
                },
            )
            .unwrap();
        let mut jobs = FieldDefaultJobs::default();
        jobs.collect(FieldDefaultSources {
            graph: &graph,
            nominals: &nominals,
            records: &records,
            types: &types,
            deferred_constants: &HashSet::new(),
        })
        .unwrap();
        let first = jobs.next().unwrap();
        jobs.finish(
            first.field,
            FieldDefaultOutcome::Ready(TypedConstant {
                ty: types.scalar(ScalarType::Bool),
                kind: jai_ir::ConstantKind::Bool(true),
            }),
            &types,
        );
        let Some(FieldDefaultReadiness::Failed(error)) = jobs.readiness(first.field) else {
            panic!()
        };
        assert_eq!(error.location, first.location);
        assert_eq!(jobs.ready().count(), 0);
        let second_field = *jobs
            .blocked_fields()
            .iter()
            .find(|&&field| field != first.field)
            .unwrap();
        jobs.request(second_field);
        let second = jobs.next().unwrap();
        let error = LocatedDiagnostic {
            location: second.location,
            message: "source default rejected".into(),
        };
        jobs.finish(
            second.field,
            FieldDefaultOutcome::Failed(error.clone()),
            &types,
        );
        let Some(FieldDefaultReadiness::Failed(actual)) = jobs.readiness(second.field) else {
            panic!()
        };
        assert_eq!(actual, &error);
        assert_eq!(jobs.ready().count(), 0);
    }
}
