//! Replay origins contain source facts and substitutions, never arena numbers.
use super::*;

#[derive(Clone)]
pub(super) struct LocalTypeOrigin {
    source: Option<SourceId>,
    path: Vec<u8>,
    owner: Vec<u8>,
    scope: usize,
    span: Span,
    name: Option<Symbol>,
    substitution: Option<crate::polymorphism::Substitution>,
    files: Vec<FileInstanceId>,
}

impl LocalDeclarationRegistry {
    pub(crate) fn debug_record_fields(&self, ty: TypeId) -> Option<&[FieldMetadata]> {
        self.records.get(&ty).map(|record| record.fields.as_slice())
    }
    pub(crate) fn debug_type_origins(
        &self,
    ) -> impl Iterator<Item = (TypeId, jai_source::SourceSpan, Option<Symbol>)> {
        self.origins.iter().filter_map(|(&ty, origin)| {
            origin.source.map(|source| {
                (
                    ty,
                    jai_source::SourceSpan {
                        source,
                        span: origin.span,
                    },
                    origin.name,
                )
            })
        })
    }
    pub(crate) fn stable_procedure_origin(&self, id: ProcedureId) -> Option<&[u8]> {
        self.body_origins.get(&id).map(Vec::as_slice)
    }

    pub(crate) fn procedure_origin_files(&self, id: ProcedureId) -> Option<&[FileInstanceId]> {
        self.body_origin_files.get(&id).map(Vec::as_slice)
    }

    pub(crate) fn type_origin_files(&self, ty: TypeId) -> Option<&[FileInstanceId]> {
        self.origins.get(&ty).map(|origin| origin.files.as_slice())
    }

    pub(crate) fn procedure_origin_substitution(
        &self,
        id: ProcedureId,
    ) -> Option<&crate::polymorphism::Substitution> {
        self.procedure_substitutions.get(&id)
    }
    pub(crate) fn stable_type_origin(&self, ty: TypeId, symbols: &Symbols) -> Option<Vec<u8>> {
        let origin = self.origins.get(&ty)?;
        fn token(output: &mut Vec<u8>, value: &[u8]) {
            output.extend_from_slice(&(value.len() as u64).to_le_bytes());
            output.extend_from_slice(value);
        }
        let mut bytes = vec![];
        token(&mut bytes, &origin.path);
        token(&mut bytes, &origin.owner);
        bytes.extend_from_slice(&(origin.span.start as u64).to_le_bytes());
        bytes.extend_from_slice(&(origin.span.end as u64).to_le_bytes());
        bytes.extend_from_slice(&(origin.scope as u64).to_le_bytes());
        token(
            &mut bytes,
            origin
                .name
                .map(|name| symbols.name(name).as_bytes())
                .unwrap_or(b"<anonymous>"),
        );
        Some(bytes)
    }

    pub(crate) fn type_origin_substitution(
        &self,
        ty: TypeId,
    ) -> Option<&crate::polymorphism::Substitution> {
        self.origins.get(&ty)?.substitution.as_ref()
    }
}

impl Resolver<'_> {
    pub(super) fn ensure_local_body_origin(&mut self) -> Vec<u8> {
        let owner = self.lexical_owner();
        if let Some(origin) = self.meta.local_declarations.body_origins.get(&owner) {
            return origin.clone();
        }
        let source_path = self
            .graph_scope
            .and_then(|scope| {
                self.debug
                    .source()
                    .or_else(|| self.compile_time.map(|context| context.source))
                    .and_then(|source| scope.source_path_for(source))
            })
            .or_else(|| self.graph_scope.map(|scope| scope.source_path()));
        let path = source_path
            .map(|path| path.as_os_str().as_encoded_bytes())
            .unwrap_or(b"<scalar-module>");
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&(path.len() as u64).to_le_bytes());
        bytes.extend_from_slice(path);
        bytes.extend_from_slice(&(self.span.start as u64).to_le_bytes());
        bytes.extend_from_slice(&(self.span.end as u64).to_le_bytes());
        self.meta
            .local_declarations
            .body_origins
            .insert(owner, bytes.clone());
        let files = self
            .graph_scope
            .map_or_else(Vec::new, |scope| vec![scope.code_origin().0]);
        self.meta
            .local_declarations
            .body_origin_files
            .insert(owner, files);
        if let Some(substitution) = self.graph_scope.and_then(|scope| scope.substitution) {
            self.meta
                .local_declarations
                .procedure_substitutions
                .insert(owner, substitution.clone());
        }
        bytes
    }

    pub(super) fn remember_local_procedure_origin(
        &mut self,
        id: LocalDeclarationId,
        procedure: ProcedureId,
        span: Span,
    ) {
        if self
            .meta
            .local_declarations
            .body_origins
            .contains_key(&procedure)
        {
            return;
        }
        let mut bytes = match id.scope.owner {
            LexicalScopeOwner::Procedure(_) => self.ensure_local_body_origin(),
            LexicalScopeOwner::Record(ty) => self
                .meta
                .local_declarations
                .stable_type_origin(ty, self.symbols)
                .unwrap_or_default(),
        };
        let files = self.local_origin_files(id);
        let path = self
            .graph_scope
            .map(|scope| {
                self.debug
                    .source()
                    .and_then(|source| scope.source_path_for(source))
                    .unwrap_or_else(|| scope.source_path())
                    .as_os_str()
                    .as_encoded_bytes()
            })
            .unwrap_or(b"<scalar-module>");
        bytes.extend_from_slice(&(path.len() as u64).to_le_bytes());
        bytes.extend_from_slice(path);
        bytes.extend_from_slice(&(span.start as u64).to_le_bytes());
        bytes.extend_from_slice(&(span.end as u64).to_le_bytes());
        bytes.extend_from_slice(&(id.scope.ordinal as u64).to_le_bytes());
        self.meta
            .local_declarations
            .body_origins
            .insert(procedure, bytes);
        self.meta
            .local_declarations
            .body_origin_files
            .insert(procedure, files);
        if let Some(substitution) = self.graph_scope.and_then(|scope| scope.substitution) {
            self.meta
                .local_declarations
                .procedure_substitutions
                .insert(procedure, substitution.clone());
        }
    }

    pub(super) fn remember_local_type_origin(
        &mut self,
        id: LocalDeclarationId,
        name: Option<Symbol>,
        ty: TypeId,
        span: Span,
    ) {
        let owner = match id.scope.owner {
            LexicalScopeOwner::Procedure(_) => self.ensure_local_body_origin(),
            LexicalScopeOwner::Record(record) => self
                .meta
                .local_declarations
                .stable_type_origin(record, self.symbols)
                .unwrap_or_default(),
        };
        let files = self.local_origin_files(id);
        let origin = LocalTypeOrigin {
            // Current-scope insertion keeps the caller's lexical scope token,
            // while the declaration range belongs to the captured code source.
            source: self
                .debug
                .source()
                .or(id.defining_source())
                .or_else(|| self.graph_scope.map(|scope| scope.source())),
            owner,
            path: self
                .graph_scope
                .map(|scope| {
                    self.debug
                        .source()
                        .and_then(|source| scope.source_path_for(source))
                        .unwrap_or_else(|| scope.source_path())
                        .as_os_str()
                        .as_encoded_bytes()
                        .to_vec()
                })
                .unwrap_or_else(|| b"<scalar-module>".to_vec()),
            scope: id.scope.ordinal,
            span,
            name,
            substitution: self
                .graph_scope
                .and_then(|scope| scope.substitution)
                .cloned(),
            files,
        };
        self.meta
            .local_declarations
            .origins
            .entry(ty)
            .or_insert(origin);
    }

    fn local_origin_files(&self, id: LocalDeclarationId) -> Vec<FileInstanceId> {
        let mut files = match id.scope.owner {
            LexicalScopeOwner::Procedure(owner) => self
                .meta
                .local_declarations
                .procedure_origin_files(owner)
                .unwrap_or_default()
                .to_vec(),
            LexicalScopeOwner::Record(owner) => self
                .meta
                .local_declarations
                .type_origin_files(owner)
                .unwrap_or_default()
                .to_vec(),
        };
        for file in id
            .defining_file()
            .into_iter()
            .chain(self.graph_scope.map(|scope| scope.code_origin().0))
        {
            if !files.contains(&file) {
                files.push(file);
            }
        }
        files
    }
}
