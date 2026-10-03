//! Atomic source declaration registration after semantic borrows have retired.
use super::*;
use crate::declaration_insertions::*;
use std::sync::Arc;

impl GraphDiscovery<'_> {
    pub fn insertion_requests(&self) -> &[DeclarationInsertionRequest] {
        &self.builder.insertion_requests.requests
    }
    pub fn pending_insertion_requests(&self) -> impl Iterator<Item = &DeclarationInsertionRequest> {
        self.insertion_requests()
            .iter()
            .filter(|request| request.publication.is_none())
    }
    /// Stage source transport without proving declaration registration.
    /// Effectful Code producers must use `admit_insertion` before committing
    /// effects and stage its sealed payload through `prepare_insertion_admitted`.
    pub fn prepare_insertion(
        &mut self,
        request: InsertionRequestId,
        code: DeclarationInsertionCode,
    ) -> Result<InsertionTransaction, InsertionResponseError> {
        if self.has_failed() {
            return Err(InsertionResponseError::FailedDiscovery);
        }
        let source = self.builder.insertion_requests.request(request)?;
        self.builder
            .graph
            .validate_insertion_code(source.file, &code)?;
        self.builder
            .insertion_requests
            .stage(request, Arc::new(code))
    }
    pub fn cancel_insertion(
        &mut self,
        transaction: InsertionTransaction,
    ) -> Result<(), InsertionResponseError> {
        self.builder.insertion_requests.cancel(transaction)
    }
    pub fn commit_insertion(
        &mut self,
        transaction: InsertionTransaction,
    ) -> Result<&DeclarationInsertionPublication, InsertionPublicationError> {
        if self.has_failed() {
            return Err(InsertionPublicationError::Response(
                InsertionResponseError::FailedDiscovery,
            ));
        }
        let code = self
            .builder
            .insertion_requests
            .staged(transaction)
            .map_err(InsertionPublicationError::Response)?;
        if transaction
            .admitted_revision()
            .is_some_and(|revision| revision != self.insertion_revision)
        {
            self.builder
                .insertion_requests
                .cancel(transaction)
                .expect("stale frontier retains its live transaction");
            return Err(InsertionPublicationError::Response(
                InsertionResponseError::StaleAdmission,
            ));
        }
        let request = self
            .builder
            .insertion_requests
            .request(transaction.request())
            .map_err(InsertionPublicationError::Response)?
            .clone();
        self.invalidate_insertion_admissions();
        match self.builder.append_insertion(&request, code) {
            Ok(publication) => {
                self.builder
                    .insertion_requests
                    .publish(transaction, publication)
                    .expect("validated live insertion receipt");
                Ok(&self.builder.graph.insertion_publications[publication])
            }
            Err(error) => {
                self.builder
                    .insertion_requests
                    .cancel(transaction)
                    .expect("failed append retains the same live receipt");
                Err(InsertionPublicationError::Graph(error))
            }
        }
    }
}

impl Builder<'_> {
    pub(super) fn defer_insertion(
        &mut self,
        file: FileInstanceId,
        directive: &jai_syntax::InsertDirective,
        location: SourceSpan,
    ) {
        let inherited_visibility = self
            .graph
            .insertion_publication(file)
            .map(|publication| {
                self.insertion_requests
                    .request(publication.request)
                    .expect("published source insertion request")
                    .visibility
            })
            .unwrap_or(Visibility::Export);
        let visibility = insertion_visibility(
            self.graph.files[file.index()].syntax.items(),
            inherited_visibility,
            location,
        )
        .expect("insertion retains its original file item");
        let before = self.insertion_requests.requests.len();
        self.insertion_requests
            .retain(file, directive, visibility, location);
        if self.insertion_requests.requests.len() != before {
            self.graph.insertions.push(FileInsertion {
                file,
                directive: directive.clone(),
                location,
            });
        }
    }
    pub(super) fn append_insertion(
        &mut self,
        request: &DeclarationInsertionRequest,
        code: Arc<DeclarationInsertionCode>,
    ) -> Result<usize, GraphError> {
        let module = self.graph.files[request.file.index()].module;
        let file = FileInstanceId(self.graph.files.len());
        let publication = self.graph.insertion_publications.len();
        let declaration_count = self.graph.declarations.len();
        let overload_count = self.graph.overload_sets.len();
        let alias_count = self.callable_aliases.len();
        let identities = self.identities.clone();
        let placeholders = self.graph.placeholders.clone();
        // File-private bindings can propagate through nested expansion scopes.
        // Checkpoint every real destination before recursive binding starts.
        let mut destinations = vec![];
        let mut destination_names = vec![];
        let mut destination = request.file;
        loop {
            destinations.push((
                destination,
                self.graph.files[destination.index()].private.clone(),
            ));
            let Some(ancestor) = self.graph.insertion_publication(destination) else {
                break;
            };
            destination_names.push((ancestor.file, ancestor.own_names.clone()));
            destination = ancestor.destination;
        }
        let bindings = self.graph.modules[module.index()].bindings.clone();
        let exports = self.graph.modules[module.index()].exports.clone();
        self.graph.files.push(FileInstance {
            id: file,
            module,
            source: code.location.source,
            scope: self.identities.scope(),
            private: HashMap::new(),
            declarations: vec![],
            syntax: ParsedFile::from_items(code.location.source, code.items.clone()),
        });
        self.graph.modules[module.index()].files.push(file);
        self.graph
            .insertion_publications
            .push(DeclarationInsertionPublication {
                request: request.id,
                destination: request.file,
                file,
                location: request.location,
                code: Arc::clone(&code),
                declarations: vec![],
                scope: request.directive.scope,
                own_names: HashSet::new(),
            });
        if let Err(error) = self.register_declarations(file, &code.items) {
            self.graph.insertion_publications.truncate(publication);
            self.graph.files.truncate(file.index());
            self.graph.modules[module.index()].files.pop();
            self.graph.declarations.truncate(declaration_count);
            self.graph.overload_sets.truncate(overload_count);
            self.callable_aliases.truncate(alias_count);
            self.identities = identities;
            self.graph.placeholders = placeholders;
            for (destination, private) in destinations {
                self.graph.files[destination.index()].private = private;
            }
            for (file, own_names) in destination_names {
                self.graph
                    .insertion_publications
                    .iter_mut()
                    .find(|publication| publication.file == file)
                    .expect("retained ancestor expansion")
                    .own_names = own_names;
            }
            self.graph.modules[module.index()].bindings = bindings;
            self.graph.modules[module.index()].exports = exports;
            return Err(error);
        }
        self.initialized_files.insert(file);
        self.file_origins.insert(file, Some(request.location));
        let path = self
            .graph
            .sources
            .get(code.location.source)
            .expect("validated quote source")
            .path()
            .to_owned();
        self.pending.entry(module).or_default().extend(
            code.items
                .iter()
                .cloned()
                .map(|item| (file, path.clone(), item)),
        );
        if !self.pending_inserted_modules.contains(&module) {
            self.pending_inserted_modules.push(module);
        }
        self.completed_modules.remove(&module);
        Ok(publication)
    }
    pub(super) fn insertion_binding(
        &mut self,
        file: FileInstanceId,
        visibility: Visibility,
        name: Symbol,
        binding: Binding,
        location: SourceSpan,
    ) -> Result<(), GraphError> {
        let Some(publication) = self
            .graph
            .insertion_publications
            .iter_mut()
            .find(|publication| publication.file == file)
        else {
            return Ok(());
        };
        publication.own_names.insert(name);
        let destination = publication.destination;
        if visibility == Visibility::File {
            self.bind(destination, Visibility::File, name, binding, location, true)?;
        }
        Ok(())
    }
}

fn insertion_visibility(
    items: &[FileItem],
    mut visibility: Visibility,
    location: SourceSpan,
) -> Option<Visibility> {
    for item in items {
        match item {
            FileItem::Scope {
                visibility: next, ..
            } => visibility = *next,
            FileItem::Insert {
                location: candidate,
                ..
            } if *candidate == location => {
                return Some(visibility);
            }
            FileItem::Conditional {
                then_items,
                else_items,
                ..
            } => {
                if let Some(found) = insertion_visibility(then_items, visibility, location)
                    .or_else(|| insertion_visibility(else_items, visibility, location))
                {
                    return Some(found);
                }
            }
            FileItem::CompileTimeCases {
                cases, ..
            } => {
                for arm in &cases.arms {
                    if let Some(found) = insertion_visibility(&arm.body, visibility, location) {
                        return Some(found);
                    }
                }
                if let Some(default) = &cases.default
                    && let Some(found) = insertion_visibility(&default.body, visibility, location)
                {
                    return Some(found);
                }
            }
            FileItem::Parameters(parameters) => {
                if let Some(found) =
                    insertion_visibility(&parameters.declarations, visibility, location)
                {
                    return Some(found);
                }
            }
            _ => {}
        }
    }
    None
}
