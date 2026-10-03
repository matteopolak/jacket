//! Keep the genuine binding queues and execution state across suspension.
use super::*;

pub(super) struct BodyJob {
    pub(super) declaration: DeclarationId,
    pub(super) file: FileInstanceId,
    pub(super) signature: Signature,
    pub(super) substitution: Option<crate::polymorphism::Substitution>,
    pub(super) specialization: Option<crate::polymorphism::SpecializationId>,
    pub(super) syntax: Option<syntax::Procedure>,
    pub(super) modifier: Option<crate::polymorphism::SpecializationKey>,
}

pub(in crate::modules) struct Worklist<'graph> {
    pub(super) insertion_owners: HashMap<jai_modules::InsertionRequestId, ProcedureId>,
    pub(super) file_abi: HashMap<ProcedureId, jai_vm::file_abi::FileAbiProcedure>,
    pub(super) process_abi: HashMap<ProcedureId, jai_vm::process_abi::ProcessAbiProcedure>,
    pub(super) heap_abi: HashMap<ProcedureId, jai_vm::heap_abi::HeapAbiProcedure>,
    pub(super) code_constants: std::collections::HashSet<DeclarationId>,
    pub(super) record_callable_aliases: std::collections::HashSet<DeclarationId>,
    pub(super) signatures: HashMap<ProcedureId, TypeId>,
    pub(super) foreign: std::collections::HashSet<ProcedureId>,
    pub(super) cache: Cache,
    pub(super) ready: HashMap<ProcedureId, Procedure>,
    pub(super) pending: Vec<BodyJob>,
    pub(super) constants: Vec<&'graph jai_modules::Declaration>,
    pub(super) constant_owners: HashMap<DeclarationId, ProcedureId>,
    pub(super) runs: Vec<(&'graph jai_modules::FileRun, ProcedureId)>,
    pub(super) completed_runs: usize,
    pub(super) isolated_caches: HashMap<ProcedureId, Cache>,
    pub(super) completed_alignments: usize,
    pub(super) file_guards: Vec<file_conditions::Guard<'graph>>,
    pub(super) completed_file_guards: usize,
    pub(super) alignment_owners: HashMap<DeclarationId, ProcedureId>,
    pub(super) method_file: FileInstanceId,
    pub(super) method_owner: ProcedureId,
    pub(super) methods_pending: bool,
    pub(super) procedure_defaults: super::super::procedure_default_jobs::Jobs,
    pub(super) header_prerequisites: header_prerequisites::HeaderPrerequisites,
}

impl<'graph> Worklist<'graph> {
    /// A retained prefix may publish genuine headers and alignment requests.
    /// Refresh them under their existing IDs without replacing the VM cache or
    /// reseeding the shared auxiliary allocator.
    pub(super) fn refresh(
        &mut self,
        session: &mut BindSession<'_, 'graph, '_>,
    ) -> Result<(), LocatedDiagnostic> {
        let partial = matches!(session.mode, BindingMode::Types(_));
        let bind_file = if partial {
            super::super::file_abi_bindings::bind_ready
        } else {
            super::super::file_abi_bindings::bind
        };
        let bind_heap = if partial {
            super::super::file_abi_bindings::bind_heap_ready
        } else {
            super::super::file_abi_bindings::bind_heap
        };
        let bind_process = if partial {
            super::super::process_abi_bindings::bind_ready
        } else {
            super::super::process_abi_bindings::bind
        };
        for (&declaration, signature) in &session.declarations.signatures {
            self.signatures.insert(signature.id, signature.ty);
            if !matches!(
                session
                    .graph
                    .declaration(declaration)
                    .unwrap()
                    .syntax()
                    .kind,
                FileDeclarationKind::Procedure(_)
            ) || self.ready.contains_key(&signature.id)
                || self
                    .pending
                    .iter()
                    .any(|job| job.signature.id == signature.id)
            {
                continue;
            }
            self.pending.push(BodyJob {
                declaration,
                file: session.graph.declaration(declaration).unwrap().file(),
                signature: signature.clone(),
                substitution: None,
                specialization: None,
                syntax: None,
                modifier: None,
            });
        }
        self.foreign.extend(session.declarations.signatures.iter().filter_map(|(id, signature)| {
            matches!(&session.graph.declaration(*id)?.syntax().kind,
                FileDeclarationKind::ProcedurePrototype(prototype) if matches!(prototype.binding, syntax::PrototypeBinding::Foreign(_)))
                .then_some(signature.id)
        }));
        self.record_callable_aliases
            .extend(super::super::record_method_headers::aliases(
                session.graph,
                &session.declarations.nominals,
                session.types,
            ));
        let contextual = super::super::deferred_constants::contextual_lambdas(session.graph);
        for source in session.graph.declarations() {
            if !(session.deferred.contains(&source.id())
                || canonical_constant(source, session.declarations, session.types))
                || contextual.contains(&source.id())
                || session.declarations.values.contains_key(&source.id())
                || self.constant_owners.contains_key(&source.id())
            {
                continue;
            }
            let owner = session
                .declarations
                .generics
                .borrow_mut()
                .reserve_local_procedure()
                .map_err(|error| located(session.graph, source.file(), error))?;
            self.constant_owners.insert(source.id(), owner);
            self.constants.push(source);
        }
        self.file_abi = bind_file(
            session.graph,
            session.types,
            session.declarations,
            session.options.file_abi.as_ref(),
            session.options.target.as_ref(),
        )?;
        self.heap_abi = bind_heap(
            session.graph,
            session.types,
            session.declarations,
            session.options.file_abi.as_ref(),
            session.options.target.as_ref(),
        )?;
        self.process_abi = bind_process(
            session.graph,
            session.types,
            session.declarations,
            session.options.process_abi.as_ref(),
            session.options.target.as_ref(),
        )?;
        self.pending.sort_by_key(|job| job.signature.id.index());
        for job in session.alignment_jobs.iter() {
            if self.alignment_owners.contains_key(&job.declaration) {
                continue;
            }
            let owner = session
                .declarations
                .generics
                .borrow_mut()
                .reserve_local_procedure()
                .map_err(|error| located(session.graph, job.file, error))?;
            self.alignment_owners.insert(job.declaration, owner);
        }
        Ok(())
    }

    pub(in crate::modules) fn cancel(
        &self,
        effects: &dyn EffectService,
    ) -> Result<(), jai_vm::Error> {
        let mut first_error = self.cache.cancel(effects).err();
        for cache in self.isolated_caches.values() {
            if let Err(error) = cache.cancel(effects) {
                first_error.get_or_insert(error);
            }
        }
        match first_error {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }

    pub(super) fn new(
        session: &mut BindSession<'_, 'graph, '_>,
    ) -> Result<Self, LocatedDiagnostic> {
        let partial = matches!(session.mode, BindingMode::Types(_));
        let bind_file = if partial {
            super::super::file_abi_bindings::bind_ready
        } else {
            super::super::file_abi_bindings::bind
        };
        let bind_heap = if partial {
            super::super::file_abi_bindings::bind_heap_ready
        } else {
            super::super::file_abi_bindings::bind_heap
        };
        let bind_process = if partial {
            super::super::process_abi_bindings::bind_ready
        } else {
            super::super::process_abi_bindings::bind
        };
        let graph = session.graph;
        let types = &mut *session.types;
        let declarations = &mut *session.declarations;
        let options = session.options;
        let deferred = session.deferred;
        let alignment_jobs = &mut *session.alignment_jobs;
        let discovery = session.discovery.as_ref();
        let file_abi = bind_file(
            graph,
            types,
            declarations,
            options.file_abi.as_ref(),
            options.target.as_ref(),
        )?;
        let heap_abi = bind_heap(
            graph,
            types,
            declarations,
            options.file_abi.as_ref(),
            options.target.as_ref(),
        )?;
        let process_abi = bind_process(
            graph,
            types,
            declarations,
            options.process_abi.as_ref(),
            options.target.as_ref(),
        )?;
        let code_constants = super::super::deferred_constants::code_constants(graph);
        let contextual_lambdas = super::super::deferred_constants::contextual_lambdas(graph);
        let record_callable_aliases =
            super::super::record_method_headers::aliases(graph, &declarations.nominals, types);
        let signatures: HashMap<_, _> = declarations
            .signatures
            .values()
            .map(|signature| (signature.id, signature.ty))
            .collect();
        let foreign = declarations
            .signatures
            .iter()
            .filter_map(
                |(id, signature)| match &graph.declaration(*id)?.syntax().kind {
                    FileDeclarationKind::ProcedurePrototype(prototype)
                        if matches!(prototype.binding, syntax::PrototypeBinding::Foreign(_)) =>
                    {
                        Some(signature.id)
                    }
                    _ => None,
                },
            )
            .collect();
        let cache = Cache::default();
        let ready = HashMap::new();
        let mut pending: Vec<_> = declarations
            .signatures
            .iter()
            .filter(|(id, _)| {
                matches!(
                    graph.declaration(**id).unwrap().syntax().kind,
                    FileDeclarationKind::Procedure(_)
                )
            })
            .map(|(&declaration, signature)| BodyJob {
                declaration,
                file: graph.declaration(declaration).unwrap().file(),
                signature: signature.clone(),
                substitution: None,
                specialization: None,
                syntax: None,
                modifier: None,
            })
            .collect();
        pending.sort_by_key(|job| job.signature.id.index());
        let constants: Vec<_> = graph
            .declarations()
            .iter()
            .filter(|declaration| {
                (deferred.contains(&declaration.id())
                    || canonical_constant(declaration, declarations, types))
                    && !contextual_lambdas.contains(&declaration.id())
            })
            .collect();
        let mut constant_owners = HashMap::new();
        for declaration in &constants {
            let owner = declarations
                .generics
                .borrow_mut()
                .reserve_local_procedure()
                .map_err(|error| located(graph, declaration.file(), error))?;
            constant_owners.insert(declaration.id(), owner);
        }
        let mut file_owners = Vec::new();
        for request in graph.runs() {
            file_owners.push(
                declarations
                    .generics
                    .borrow_mut()
                    .reserve_local_procedure()
                    .map_err(|error| located(graph, request.file, error))?,
            );
        }
        let runs: Vec<_> = graph.runs().iter().zip(file_owners).collect();
        let completed_runs = 0;
        let isolated_caches: HashMap<ProcedureId, Cache> = HashMap::new();
        let completed_alignments = 0;
        let file_guards = if discovery.is_none() {
            file_conditions::collect(graph)
        } else {
            vec![]
        };
        let completed_file_guards = 0;
        let mut alignment_owners = HashMap::new();
        for job in alignment_jobs.iter() {
            let owner = declarations
                .generics
                .borrow_mut()
                .reserve_local_procedure()
                .map_err(|error| located(graph, job.file, error))?;
            alignment_owners.insert(job.declaration, owner);
        }
        let method_file = graph.module(graph.root()).unwrap().entry();
        let method_owner = declarations
            .generics
            .borrow_mut()
            .reserve_local_procedure()
            .map_err(|error| located(graph, method_file, error))?;
        let methods_pending = true;
        let mut insertion_owners = HashMap::new();
        if let Some(jobs) = discovery.and_then(|jobs| jobs.insertions.as_ref()) {
            for request in jobs.requests() {
                if request.publication.is_some() {
                    continue;
                }
                let owner = declarations
                    .generics
                    .borrow_mut()
                    .reserve_local_procedure()
                    .map_err(|error| located(graph, request.file, error))?;
                insertion_owners.insert(request.id, owner);
            }
        }
        Ok(Self {
            insertion_owners,
            file_abi,
            heap_abi,
            process_abi,
            code_constants,
            record_callable_aliases,
            signatures,
            foreign,
            cache,
            ready,
            pending,
            constants,
            constant_owners,
            runs,
            completed_runs,
            isolated_caches,
            completed_alignments,
            file_guards,
            completed_file_guards,
            alignment_owners,
            method_file,
            method_owner,
            methods_pending,
            procedure_defaults: Default::default(),
            header_prerequisites: Default::default(),
        })
    }
}

pub(super) fn canonical_constant(
    source: &jai_modules::Declaration,
    declarations: &ScopedDeclarations<'_>,
    types: &TypeRegistry,
) -> bool {
    matches!(&source.syntax().kind, FileDeclarationKind::Constant(constant) if constant.ty.is_some())
        && !declarations
            .nominals
            .is_type_alias(declarations.graph, source.id())
        && declarations
            .nominals
            .value_types
            .get(&source.id())
            .is_some_and(|&ty| {
                matches!(
                    types.kind(ty),
                    Ok(TypeKind::Distinct(_) | TypeKind::Enum(_))
                )
            })
}
