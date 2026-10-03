//! Owned lexical declarations retain nominal identity across body retries.
use super::*;
use crate::modules::aggregates::parameterized::NominalAnnotationContext;
use jai_modules::FileInstanceId;
use jai_source::SourceId;
use jai_types::{FieldId, Integer, RecordKind, TypeKind};
use std::collections::HashSet;
use std::sync::Arc;

mod applications;
mod callback_body_checks;
mod capture_exports;
mod compiler_retention;
mod constant_results;
mod constants;
mod default_overrides;
mod enums;
mod external_data;
pub(crate) use external_data::ExternalGlobals;
mod field_sources;
mod generic_procedures;
pub(crate) use field_sources::{FieldSource, FieldSourceRef};
mod imports;
mod libraries;
mod metadata;
mod methods;
pub(crate) use methods::RecordMethodPrerequisites;
mod namespaces;
mod operators;
mod origins;
mod procedure_sources;
mod procedures;
pub(crate) use procedure_sources::ProcedureSourceIdentity;
mod anonymous_preview;
mod anonymous_procedures;
mod ready_bindings;
mod record_conditions;
mod records;
mod signatures;
mod source_bodies;
mod source_headers;
use source_bodies::SourceProcedureDefinition;
pub(crate) use source_headers::CheckedSourceHeader;
use source_headers::SourceHeaderPhase;
mod sources;
use records::RecordSource;
use signatures::CallableSource;
pub(crate) use signatures::{HeaderReadiness, MethodPhase};
#[cfg(test)]
mod tests;
mod types;

/// Local scopes belong to one concrete source body, including a specialization.
/// They deliberately do not occupy the module graph's declaration arena.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum LexicalScopeOwner {
    Procedure(ProcedureId),
    Record(TypeId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct LexicalScopeId {
    owner: LexicalScopeOwner,
    file: Option<FileInstanceId>,
    source: Option<SourceId>,
    ordinal: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct LocalDeclarationId {
    scope: LexicalScopeId,
    start: usize,
    end: usize,
    ordinal: usize,
}
impl LocalDeclarationId {
    pub(crate) fn defining_file(self) -> Option<FileInstanceId> {
        self.scope.file
    }
    pub(crate) fn defining_source(self) -> Option<SourceId> {
        self.scope.source
    }
    pub(crate) fn source_span(self) -> Span {
        Span::new(self.start, self.end)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum LocalCallableSpecialization {
    Expected(TypeId),
    InferredParameters(TypeId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct LocalCallableId {
    declaration: LocalDeclarationId,
    specialization: LocalCallableSpecialization,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct GraphCallableId {
    declaration: jai_source::DeclarationId,
    specialization: LocalCallableSpecialization,
}

#[derive(Clone)]
enum DeclarationSyntax {
    Library(syntax::LibraryDeclaration),
    Record(syntax::RecordDeclaration),
    Enum(syntax::EnumDeclaration),
    Alias(syntax::TypeAliasDeclaration),
    Constant(syntax::ConstantDeclaration),
    ConstantResult(constant_results::ConstantResult),
    Procedure(syntax::Procedure),
    Prototype(syntax::ProcedurePrototype),
}
impl DeclarationSyntax {
    fn from_record_member(member: &syntax::RecordMember) -> Option<Self> {
        Some(match member {
            syntax::RecordMember::Constant(value) => Self::Constant(value.clone()),
            syntax::RecordMember::TypeAlias(value) => Self::Alias(value.clone()),
            syntax::RecordMember::Record(value) => Self::Record((**value).clone()),
            syntax::RecordMember::Enum(value) => Self::Enum(value.clone()),
            syntax::RecordMember::Procedure(value) => Self::Procedure((**value).clone()),
            syntax::RecordMember::ProcedurePrototype(value) => Self::Prototype(value.clone()),
            _ => return None,
        })
    }
    fn from_statement(statement: &syntax::Statement) -> Option<Self> {
        Some(match &statement.kind {
            syntax::StatementKind::Library(library) => Self::Library(library.clone()),
            syntax::StatementKind::Record(record) => Self::Record(record.clone()),
            syntax::StatementKind::Enum(enumeration) => Self::Enum(enumeration.clone()),
            syntax::StatementKind::TypeAlias(alias) => Self::Alias(alias.clone()),
            syntax::StatementKind::Constant(constant) => Self::Constant(constant.clone()),
            syntax::StatementKind::Procedure(procedure) => Self::Procedure((**procedure).clone()),
            syntax::StatementKind::ProcedurePrototype(prototype) => {
                Self::Prototype(prototype.clone())
            }
            syntax::StatementKind::UsingDeclaration { declaration, .. } => {
                return Self::from_statement(declaration);
            }
            _ => return None,
        })
    }
    fn name(&self) -> Symbol {
        match self {
            Self::Library(value) => value.name,
            Self::Record(value) => value.name,
            Self::Enum(value) => value.name,
            Self::Alias(value) => value.name,
            Self::Constant(value) => value.name,
            Self::ConstantResult(value) => value.name(),
            Self::Procedure(value) => value.name,
            Self::Prototype(value) => value.name,
        }
    }
    fn span(&self) -> Span {
        match self {
            Self::Library(value) => value.span,
            Self::Record(value) => value.span,
            Self::Enum(value) => value.span,
            Self::Alias(value) => value.span,
            Self::Constant(value) => value.span,
            Self::ConstantResult(value) => value.span(),
            Self::Procedure(value) => value.span,
            Self::Prototype(value) => value.span,
        }
    }
}

#[derive(Clone)]
struct Declaration {
    id: LocalDeclarationId,
    syntax: DeclarationSyntax,
    checks: crate::safety_checks::ActiveChecks,
    imports: imports::ScopedImportEnvironment,
    operator_imports: Vec<jai_source::ModuleId>,
    using: imports::UsingPrefixEnvironment,
}
#[derive(Clone)]
struct ScopeFrame {
    id: LexicalScopeId,
    declarations: HashMap<Symbol, Declaration>,
    operators: Vec<Declaration>,
    operator_imports: Vec<jai_source::ModuleId>,
    runtime: HashMap<Symbol, syntax::Declaration>,
    external: HashMap<Symbol, LocalDeclarationId>,
    runtime_symbols: HashSet<Symbol>,
    imports: imports::ScopedImportEnvironment,
    import_aliases: HashSet<Symbol>,
    using_bindings: HashMap<Symbol, Binding>,
    using_pending: HashSet<Symbol>,
    using_placeholders: HashMap<Symbol, jai_modules::PlaceholderId>,
    using_origins: HashMap<Symbol, Span>,
    using_operator_declarations: Vec<jai_source::DeclarationId>,
}
#[derive(Clone, Default)]
pub(crate) struct LocalScopes {
    frames: Vec<ScopeFrame>,
    next_scope: usize,
    body_owner: Option<ProcedureId>,
    annotation_owner: NominalAnnotationContext,
    active: HashSet<LocalDeclarationId>,
}

pub(crate) struct CleanupRuntimeShadows {
    frames: Vec<CleanupRuntimeFrame>,
}

struct CleanupRuntimeFrame {
    id: LexicalScopeId,
    runtime: HashMap<Symbol, syntax::Declaration>,
    symbols: HashSet<Symbol>,
}

impl LocalScopes {
    pub(crate) fn capture_cleanup_runtime(
        &mut self,
        bindings: &[HashMap<Symbol, Binding>],
    ) -> CleanupRuntimeShadows {
        let frames = self
            .frames
            .iter_mut()
            .zip(bindings)
            .map(|(frame, bindings)| {
                let runtime = frame
                    .runtime
                    .extract_if(|name, _| !bindings.contains_key(name))
                    .collect();
                let symbols = frame
                    .runtime_symbols
                    .extract_if(|name| !bindings.contains_key(name))
                    .collect();
                CleanupRuntimeFrame {
                    id: frame.id,
                    runtime,
                    symbols,
                }
            })
            .collect();
        CleanupRuntimeShadows { frames }
    }

    pub(crate) fn restore_cleanup_runtime(&mut self, shadows: CleanupRuntimeShadows) {
        for CleanupRuntimeFrame {
            id,
            runtime,
            symbols,
        } in shadows.frames
        {
            let frame = self
                .frames
                .iter_mut()
                .find(|frame| frame.id == id)
                .expect("cleanup capture preserves defining frames");
            frame.runtime.extend(runtime);
            frame.runtime_symbols.extend(symbols);
        }
    }

    pub(crate) fn for_procedure(&self, procedure: ProcedureId) -> Self {
        let mut scopes = self.clone();
        scopes.body_owner = Some(procedure);
        scopes.annotation_owner = NominalAnnotationContext::None;
        scopes
    }

    pub(crate) fn body_owner(&self) -> Option<ProcedureId> {
        self.body_owner
    }

    pub(crate) fn isolated_expansion(&self) -> Self {
        Self {
            frames: vec![],
            annotation_owner: self.annotation_owner,
            next_scope: self.next_scope,
            body_owner: self.body_owner,
            active: HashSet::new(),
        }
    }

    pub(crate) fn resume_after_expansion(&mut self, expanded: &Self) {
        self.next_scope = self.next_scope.max(expanded.next_scope);
    }

    pub(crate) fn capture_identity(&self) -> Vec<LexicalScopeId> {
        self.frames.iter().map(|frame| frame.id).collect()
    }
}

#[derive(Clone)]
pub(crate) struct FieldMetadata {
    pub(crate) name: Option<Symbol>,
    pub(crate) id: FieldId,
    pub(crate) ty: TypeId,
    pub(crate) syntax: FieldSource,
}
#[derive(Clone)]
pub(crate) struct RecordMetadata {
    pub(crate) name: Option<Symbol>,
    pub(crate) kind: RecordKind,
    pub(crate) fields: Vec<FieldMetadata>,
}
#[derive(Clone)]
pub(crate) struct SelectedLocalRecordSource {
    pub(crate) declaration: LocalDeclarationId,
    pub(crate) owner: TypeId,
    pub(crate) members: Arc<[syntax::RecordMember]>,
    environment: Arc<sources::SourceEnvironment>,
}
struct EnumMetadata {
    name: Option<Symbol>,
    flags: bool,
    members: HashMap<Symbol, Integer>,
    values: Vec<(Symbol, Integer)>,
}
#[derive(Default)]
struct Entry {
    nominal: Option<TypeId>,
    procedure: Option<ProcedureId>,
    binding: Option<Binding>,
    scope_watermark: usize,
}
#[derive(Default)]
pub(crate) struct LocalDeclarationRegistry {
    generic_procedures: generic_procedures::LocalGenericProcedures,
    callback_checks: callback_body_checks::CallbackBodyChecks,
    entries: HashMap<LocalDeclarationId, Entry>,
    records: HashMap<TypeId, RecordMetadata>,
    enumerations: HashMap<TypeId, EnumMetadata>,
    defaults: HashMap<FieldId, jai_ir::ConstantValue>,
    no_write_record_overrides: HashMap<TypeId, Box<[syntax::RecordMember]>>,
    record_overrides_ready: HashSet<TypeId>,
    active_record_overrides: HashSet<TypeId>,
    field_notes: HashMap<FieldId, Box<[Box<[u8]>]>>,
    record_reflection: HashMap<TypeId, jai_types::ReflectedRecordMetadata>,
    record_namespaces: HashMap<TypeId, HashMap<Symbol, Binding>>,
    namespace_scopes: HashMap<TypeId, LexicalScopeId>,
    namespace_sources: HashMap<TypeId, Arc<sources::SourceEnvironment>>,
    selected_record_sources: HashMap<TypeId, Arc<SelectedLocalRecordSource>>,
    constant_sources: HashMap<LocalDeclarationId, sources::SourceEnvironment>,
    alias_sources: HashMap<LocalDeclarationId, crate::procedure_values::contracts::ContractSyntax>,
    method_phases: HashMap<TypeId, MethodPhase>,
    method_body_demand: methods::MethodBodyDemand,
    active_method_records: HashSet<TypeId>,
    checked_method_records: HashSet<TypeId>,
    pending_local_method_records: HashSet<TypeId>,
    names: HashMap<TypeId, Symbol>,
    origins: HashMap<TypeId, origins::LocalTypeOrigin>,
    body_origins: HashMap<ProcedureId, Vec<u8>>,
    body_origin_files: HashMap<ProcedureId, Vec<FileInstanceId>>,
    procedure_substitutions: HashMap<ProcedureId, crate::polymorphism::Substitution>,
    signatures: HashMap<ProcedureId, Signature>,
    header_readiness: HashMap<ProcedureId, HeaderReadiness>,
    callable_declarations: HashMap<ProcedureId, LocalDeclarationId>,
    procedure_sources: HashMap<ProcedureId, ProcedureSourceIdentity>,
    procedures: HashMap<ProcedureId, Procedure>,
    anonymous_procedures: HashMap<LocalCallableId, ProcedureId>,
    graph_anonymous_procedures: HashMap<GraphCallableId, ProcedureId>,
    active_anonymous_procedures: HashSet<ProcedureId>,
    prototypes: HashMap<ProcedureId, ProcedurePrototype>,
    foreign_libraries: HashMap<ForeignLibraryId, ForeignLibrary>,
    library_ids: HashMap<LocalDeclarationId, ForeignLibraryId>,
    next_library: HashMap<ProcedureId, usize>,
    /// The legacy scalar resolver has no graph's shared procedure allocator.
    next_legacy_procedure: Option<usize>,
}
impl LocalDeclarationRegistry {
    pub(crate) fn has_foreign_library_owner(&self, procedure: ProcedureId) -> bool {
        self.foreign_libraries.keys().any(|id| {
            matches!(id,
            ForeignLibraryId::Local { procedure: owner, .. } if *owner == procedure)
        })
    }
    pub(crate) fn begin_anonymous_procedure(
        &mut self,
        procedure: ProcedureId,
        span: Span,
    ) -> Result<(), Diagnostic> {
        if !self.active_anonymous_procedures.insert(procedure) {
            return Err(Diagnostic::new(
                span,
                "recursive inferred lambda requires a declared procedure signature",
            ));
        }
        Ok(())
    }

    pub(crate) fn end_anonymous_procedure(&mut self, procedure: ProcedureId) {
        self.active_anonymous_procedures.remove(&procedure);
    }

    pub(crate) fn foreign_libraries(&self) -> Vec<ForeignLibrary> {
        let mut libraries: Vec<_> = self.foreign_libraries.values().cloned().collect();
        libraries.sort_by_key(|library| match library.id {
            ForeignLibraryId::File(id) => (0, id.index(), 0),
            ForeignLibraryId::Local { procedure, index } => (1, procedure.index(), index),
        });
        libraries
    }
    pub(crate) fn publish_generated(
        &mut self,
        signature: Signature,
        procedure: Procedure,
    ) -> Result<(), Diagnostic> {
        if signature.id != procedure.id || signature.ty != procedure.signature {
            return Err(Diagnostic::new(
                Span::default(),
                "generated procedure does not match its published signature",
            ));
        }
        self.signatures.insert(signature.id, signature);
        self.procedures.insert(procedure.id, procedure);
        Ok(())
    }
    pub(crate) fn signature(&self, id: ProcedureId) -> Option<&Signature> {
        self.signatures.get(&id)
    }
    pub(crate) fn signature_snapshot(&self) -> HashMap<ProcedureId, TypeId> {
        self.signatures
            .iter()
            .map(|(&id, signature)| (id, signature.ty))
            .collect()
    }
    pub(crate) fn ready_snapshot(&self) -> HashMap<ProcedureId, Procedure> {
        self.procedures.clone()
    }
    pub(crate) fn ready_procedure(&self, id: ProcedureId) -> Option<&Procedure> {
        self.procedures.get(&id)
    }
    pub(crate) fn semantic_ready_count(&self) -> usize {
        self.procedures
            .keys()
            .filter(|procedure| self.body_origins.contains_key(procedure))
            .count()
    }
    pub(crate) fn prototypes(&self) -> Vec<ProcedurePrototype> {
        let mut prototypes: Vec<_> = self.prototypes.values().cloned().collect();
        prototypes.sort_by_key(|prototype| prototype.id.index());
        prototypes
    }
    pub(crate) fn append_reflection_metadata(
        &self,
        metadata: &mut jai_types::ReflectionMetadata,
        symbols: &Symbols,
    ) {
        for (&ty, &name) in &self.names {
            metadata.name(ty, symbols.name(name).as_bytes());
        }
        for (&ty, record) in &self.records {
            if let Some(name) = record.name {
                metadata.name(ty, symbols.name(name).as_bytes());
            }
            if let Some(record) = self.record_reflection.get(&ty) {
                metadata.record(ty, record.clone());
            }
            for field in &record.fields {
                metadata.field(
                    field.id,
                    jai_types::ReflectedFieldMetadata {
                        name: field.name.map(|name| symbols.name(name).as_bytes().into()),
                        using: field.syntax.using(),
                    },
                );
                if let Some(notes) = self.field_notes.get(&field.id) {
                    metadata.field_notes(field.id, notes.clone());
                }
            }
        }
        for (&ty, enumeration) in &self.enumerations {
            if let Some(name) = enumeration.name {
                metadata.name(ty, symbols.name(name).as_bytes());
            }
            metadata.enumeration(
                ty,
                jai_types::ReflectedEnumMetadata {
                    members: enumeration
                        .values
                        .iter()
                        .map(|&(name, value)| jai_types::ReflectedEnumMember {
                            name: Some(symbols.name(name).as_bytes().into()),
                            value,
                        })
                        .collect(),
                    flags: enumeration.flags,
                },
            );
        }
    }
}

impl Resolver<'_> {
    pub(crate) fn reserve_graph_anonymous_procedure(
        &mut self,
        declaration: jai_source::DeclarationId,
        span: Span,
        specialization: LocalCallableSpecialization,
    ) -> Result<ProcedureId, Diagnostic> {
        let callable = GraphCallableId {
            declaration,
            specialization,
        };
        if let Some(&procedure) = self
            .meta
            .local_declarations
            .graph_anonymous_procedures
            .get(&callable)
        {
            return Ok(procedure);
        }
        let procedure = self.reserve_generated_procedure(span)?;
        let scope = self.graph_scope.ok_or_else(|| {
            Diagnostic::new(span, "module lambda requires its defining source scope")
        })?;
        let path = scope.source_path().as_os_str().as_encoded_bytes();
        let mut origin = Vec::new();
        origin.extend_from_slice(&(path.len() as u64).to_le_bytes());
        origin.extend_from_slice(path);
        origin.extend_from_slice(&(span.start as u64).to_le_bytes());
        origin.extend_from_slice(&(span.end as u64).to_le_bytes());
        self.meta
            .local_declarations
            .body_origins
            .insert(procedure, origin);
        self.meta
            .local_declarations
            .body_origin_files
            .insert(procedure, vec![scope.code_origin().0]);
        if let Some(substitution) = scope.substitution {
            self.meta
                .local_declarations
                .procedure_substitutions
                .insert(procedure, substitution.clone());
        }
        self.meta
            .local_declarations
            .graph_anonymous_procedures
            .insert(callable, procedure);
        Ok(procedure)
    }

    pub(crate) fn reserve_local_anonymous_procedure(
        &mut self,
        span: Span,
        specialization: LocalCallableSpecialization,
    ) -> Result<ProcedureId, Diagnostic> {
        while self.local_scopes.frames.len() < self.scopes.len() {
            self.push_local_scope();
        }
        let declaration = LocalDeclarationId {
            scope: self.local_scopes.frames.last().unwrap().id,
            start: span.start,
            end: span.end,
            ordinal: usize::MAX - 1,
        };
        let callable = LocalCallableId {
            declaration,
            specialization,
        };
        if let Some(&procedure) = self
            .meta
            .local_declarations
            .anonymous_procedures
            .get(&callable)
        {
            return Ok(procedure);
        }
        let procedure = self.reserve_generated_procedure(span)?;
        self.meta
            .local_declarations
            .anonymous_procedures
            .insert(callable, procedure);
        self.remember_local_procedure_origin(declaration, procedure, span);
        Ok(procedure)
    }

    pub(crate) fn reserve_generated_procedure(
        &mut self,
        span: Span,
    ) -> Result<ProcedureId, Diagnostic> {
        if let Some(scope) = self.graph_scope {
            return scope.reserve_local_procedure();
        }
        let next = self
            .meta
            .local_declarations
            .next_legacy_procedure
            .get_or_insert_with(|| {
                self.signatures
                    .values()
                    .map(|signature| signature.id.index())
                    .max()
                    .map_or(0, |index| index + 1)
            });
        let id = ProcedureId::new(*next);
        *next = next
            .checked_add(1)
            .ok_or_else(|| Diagnostic::new(span, "procedure identity space exhausted"))?;
        Ok(id)
    }

    fn lexical_owner(&self) -> ProcedureId {
        self.local_scopes.body_owner().unwrap_or_else(|| {
            self.compile_time
                .map_or(self.procedure, |context| context.owner)
        })
    }
    pub(crate) fn push_local_scope(&mut self) {
        self.ensure_local_body_origin();
        let id = LexicalScopeId {
            owner: LexicalScopeOwner::Procedure(self.lexical_owner()),
            file: self
                .graph_scope
                .map(|scope| scope.code_origin().0)
                .or_else(|| self.compile_time.map(|context| context.file)),
            source: self
                .debug
                .source()
                .or_else(|| self.graph_scope.map(|scope| scope.source())),
            ordinal: self.local_scopes.next_scope,
        };
        self.local_scopes.next_scope += 1;
        self.local_scopes.frames.push(ScopeFrame {
            id,
            declarations: HashMap::new(),
            operators: vec![],
            operator_imports: vec![],
            runtime: HashMap::new(),
            external: HashMap::new(),
            runtime_symbols: HashSet::new(),
            imports: imports::ScopedImportEnvironment::default(),
            import_aliases: HashSet::new(),
            using_bindings: HashMap::new(),
            using_pending: HashSet::new(),
            using_placeholders: HashMap::new(),
            using_origins: HashMap::new(),
            using_operator_declarations: vec![],
        });
    }
    pub(crate) fn pop_local_scope(&mut self) {
        self.local_scopes.frames.pop();
    }
    pub(crate) fn local_name_reserved(&self, name: Symbol) -> bool {
        self.local_scopes
            .frames
            .last()
            .and_then(|frame| frame.declarations.get(&name))
            .is_some_and(|declaration| !self.local_scopes.active.contains(&declaration.id))
    }
    pub(crate) fn local_name_present(&self, name: Symbol) -> bool {
        self.scopes.iter().any(|scope| scope.contains_key(&name))
            || self.local_scopes.frames.iter().any(|frame| {
                frame.declarations.contains_key(&name)
                    || frame.runtime.contains_key(&name)
                    || frame.runtime_symbols.contains(&name)
                    || frame.imports.contains_key(&name)
                    || frame.using_bindings.contains_key(&name)
                    || frame.using_pending.contains(&name)
                    || frame.using_placeholders.contains_key(&name)
            })
    }

    pub(crate) fn register_local_declarations(
        &mut self,
        statements: &[syntax::Statement],
    ) -> Result<(), Diagnostic> {
        while self.local_scopes.frames.len() < self.scopes.len() {
            self.push_local_scope();
        }
        let depth = self.local_scopes.frames.len() - 1;
        let mut names = HashSet::new();
        let mut imports = self.local_scopes.frames[depth].imports.clone();
        let mut operator_imports = self.local_scopes.frames[depth].operator_imports.clone();
        let mut using =
            imports::UsingPrefixEnvironment::from_frame(&self.local_scopes.frames[depth]);
        for (ordinal, statement) in statements.iter().enumerate() {
            if let syntax::StatementKind::Import(import) = &statement.kind {
                self.extend_scoped_imports(&mut imports, import)?;
                self.extend_operator_imports(&mut operator_imports, import)?;
                continue;
            }
            if let syntax::StatementKind::Using(directive) = &statement.kind {
                self.extend_checked_using_prefix(&mut using, directive)?;
                continue;
            }
            let declaration_statement = if let syntax::StatementKind::UsingDeclaration {
                declaration,
                ..
            } = &statement.kind
            {
                declaration.as_ref()
            } else {
                statement
            };
            if let syntax::StatementKind::Declare(declaration) = &declaration_statement.kind {
                self.local_scopes.frames[depth]
                    .runtime
                    .insert(declaration.name(), declaration.clone());
                if matches!(declaration, syntax::Declaration::External { .. }) {
                    let span = declaration_statement.span;
                    let scope = self.local_scopes.frames[depth].id;
                    self.local_scopes.frames[depth].external.insert(
                        declaration.name(),
                        LocalDeclarationId {
                            scope,
                            start: span.start,
                            end: span.end,
                            ordinal,
                        },
                    );
                }
            }
            let declarations = if let syntax::StatementKind::ConstantResults(group) =
                &declaration_statement.kind
            {
                constant_results::declarations(group, self.symbols)
            } else {
                DeclarationSyntax::from_statement(statement)
                    .into_iter()
                    .collect()
            };
            for syntax in declarations {
                if syntax.operator().is_some() {
                    self.register_owned_local_declaration(
                        depth,
                        syntax,
                        ordinal,
                        imports.clone(),
                        operator_imports.clone(),
                        using.clone(),
                    );
                    continue;
                }
                let name = syntax.name();
                let span = syntax.span();
                if !names.insert(name)
                    || self.scopes[depth].contains_key(&name)
                    || self.local_scopes.frames[depth]
                        .declarations
                        .contains_key(&name)
                {
                    return Err(Diagnostic::new(
                        span,
                        format!("duplicate local declaration '{}'", self.symbols.name(name)),
                    ));
                }
                self.register_owned_local_declaration(
                    depth,
                    syntax,
                    ordinal,
                    imports.clone(),
                    operator_imports.clone(),
                    using.clone(),
                );
            }
            if let Some(directive) = statement.using_declaration_directive() {
                self.extend_checked_using_prefix(&mut using, &directive)?;
            }
        }
        Ok(())
    }

    pub(crate) fn register_local_record_declarations(
        &mut self,
        members: &[syntax::RecordMember],
    ) -> Result<(), Diagnostic> {
        while self.local_scopes.frames.len() < self.scopes.len() {
            self.push_local_scope();
        }
        let depth = self.local_scopes.frames.len() - 1;
        let imports = self.local_scopes.frames[depth].imports.clone();
        let operator_imports = self.local_scopes.frames[depth].operator_imports.clone();
        let using = imports::UsingPrefixEnvironment::from_frame(&self.local_scopes.frames[depth]);
        for (ordinal, member) in members.iter().enumerate() {
            let Some(syntax) = DeclarationSyntax::from_record_member(member) else {
                continue;
            };
            if syntax.operator().is_some() {
                self.register_owned_local_declaration(
                    depth,
                    syntax,
                    ordinal,
                    imports.clone(),
                    operator_imports.clone(),
                    using.clone(),
                );
                continue;
            }
            let name = syntax.name();
            if let Some(previous) = self.local_scopes.frames[depth].declarations.get(&name) {
                if previous.syntax.span() == syntax.span() {
                    continue;
                }
                return Err(Diagnostic::new(
                    syntax.span(),
                    "duplicate record namespace declaration",
                ));
            }
            if self.scopes[depth].contains_key(&name) {
                return Err(Diagnostic::new(
                    syntax.span(),
                    "duplicate record namespace declaration",
                ));
            }
            self.register_owned_local_declaration(
                depth,
                syntax,
                ordinal,
                imports.clone(),
                operator_imports.clone(),
                using.clone(),
            );
        }
        Ok(())
    }

    pub(crate) fn declare_local_runtime_symbol(&mut self, name: Symbol) {
        while self.local_scopes.frames.len() < self.scopes.len() {
            self.push_local_scope();
        }
        self.local_scopes
            .frames
            .last_mut()
            .unwrap()
            .runtime_symbols
            .insert(name);
    }

    fn register_owned_local_declaration(
        &mut self,
        depth: usize,
        syntax: DeclarationSyntax,
        ordinal: usize,
        imports: imports::ScopedImportEnvironment,
        operator_imports: Vec<jai_source::ModuleId>,
        using: imports::UsingPrefixEnvironment,
    ) {
        let name = syntax.name();
        let span = syntax.span();
        let id = LocalDeclarationId {
            scope: self.local_scopes.frames[depth].id,
            start: span.start,
            end: span.end,
            ordinal,
        };
        let entry = self.meta.local_declarations.entries.entry(id).or_default();
        if entry.nominal.is_none() {
            entry.nominal = match &syntax {
                DeclarationSyntax::Record(record) => Some(self.types.reserve_record(record.kind)),
                DeclarationSyntax::Alias(syntax::TypeAliasDeclaration {
                    ty: syntax::TypeSyntax::Variant { kind, .. },
                    ..
                }) => Some(self.types.reserve_distinct(match kind {
                    syntax::TypeVariantKind::Distinct => jai_types::DistinctKind::Distinct,
                    syntax::TypeVariantKind::IsA => jai_types::DistinctKind::IsA,
                })),
                _ => None,
            };
        }
        if let Some(ty) = entry.nominal {
            self.meta.local_declarations.names.insert(ty, name);
            self.remember_local_type_origin(id, Some(name), ty, span);
        }
        let is_operator = syntax.operator().is_some();
        let declaration = Declaration {
            id,
            syntax,
            checks: self.checks,
            imports,
            operator_imports,
            using,
        };
        if is_operator {
            let operators = &mut self.local_scopes.frames[depth].operators;
            if !operators.iter().any(|previous| previous.id == id) {
                operators.push(declaration);
            }
        } else {
            self.local_scopes.frames[depth]
                .declarations
                .insert(name, declaration);
        }
    }

    pub(crate) fn resolve_registered_local_declarations(&mut self) -> Result<(), Diagnostic> {
        let depth = self.local_scopes.frames.len() - 1;
        let mut declarations: Vec<_> = self.local_scopes.frames[depth]
            .declarations
            .values()
            .cloned()
            .collect();
        declarations.extend(self.local_scopes.frames[depth].operators.iter().cloned());
        declarations.sort_by_key(|declaration| {
            let span = declaration.syntax.span();
            (span.start, span.end)
        });
        // Pending names are all installed before any initializer or annotation
        // resolves, so lookup follows the defining block rather than source order.
        for declaration in declarations {
            if matches!(&declaration.syntax, DeclarationSyntax::Procedure(procedure) if procedure.expands)
            {
                continue;
            }
            if matches!(&declaration.syntax, DeclarationSyntax::Procedure(procedure)
                if procedure.operator.is_some() && crate::polymorphism::is_polymorphic(procedure))
            {
                continue;
            }
            if let DeclarationSyntax::Constant(constant) = &declaration.syntax
                && (reflection::is_semantic_constant(&constant.initializer)
                    || matches!(
                        constant.initializer.kind,
                        syntax::ExpressionKind::Call(..)
                            | syntax::ExpressionKind::QualifiedCall(..)
                            | syntax::ExpressionKind::IndirectCall { .. }
                            | syntax::ExpressionKind::ContextCall { .. }
                            | syntax::ExpressionKind::CallHint { .. }
                            | syntax::ExpressionKind::ShortLambda(_)
                    ))
            {
                continue;
            }
            self.resolve_local_declaration(depth, &declaration)?;
        }
        self.validate_local_using_fields(self.span)
    }

    pub(crate) fn resolve_local_name(
        &mut self,
        name: Symbol,
        span: Span,
    ) -> Result<Option<Binding>, Diagnostic> {
        for depth in (0..self.scopes.len()).rev() {
            if let Some(binding) = self.scopes[depth].get(&name).cloned() {
                return Ok(Some(binding));
            }
            if let Some(declaration) = self
                .local_scopes
                .frames
                .get(depth)
                .and_then(|frame| frame.declarations.get(&name))
                .cloned()
            {
                return self
                    .resolve_local_declaration(depth, &declaration)
                    .map(Some);
            }
            if self.local_scopes.frames.get(depth).is_some_and(|frame| {
                frame.runtime.contains_key(&name) || frame.runtime_symbols.contains(&name)
            }) {
                self.reject_lexical_capture(name, span)?;
                return Err(Diagnostic::new(
                    span,
                    format!(
                        "runtime local '{}' is unavailable before its declaration",
                        self.symbols.name(name)
                    ),
                ));
            }
            if let Some(binding) = self
                .local_scopes
                .frames
                .get(depth)
                .and_then(|frame| frame.using_bindings.get(&name))
                .cloned()
            {
                return Ok(Some(binding));
            }
            if let Some(marker) = self
                .local_scopes
                .frames
                .get(depth)
                .and_then(|frame| frame.using_placeholders.get(&name))
                .copied()
            {
                let scope = self.graph_scope.ok_or_else(|| {
                    Diagnostic::new(span, "using placeholder requires its source graph")
                })?;
                return Ok(Some(Binding::Imported(
                    scope.imported_placeholder_binding(marker, span)?,
                )));
            }
            if self
                .local_scopes
                .frames
                .get(depth)
                .is_some_and(|frame| frame.using_pending.contains(&name))
            {
                return Err(Diagnostic::new(
                    span,
                    format!(
                        "using place alias '{}' is unavailable before its source statement",
                        self.symbols.name(name)
                    ),
                ));
            }
            if let Some(binding) = self
                .local_scopes
                .frames
                .get(depth)
                .and_then(|frame| frame.imports.get(&name))
                .copied()
            {
                return Ok(Some(match binding {
                    jai_modules::Binding::Module(module) => Binding::Namespace(module),
                    binding => Binding::Imported(binding),
                }));
            }
            if let Some(marker) = self
                .local_scopes
                .frames
                .get(depth)
                .and_then(|frame| frame.imports.placeholder(name))
            {
                let scope = self.graph_scope.ok_or_else(|| {
                    Diagnostic::new(span, "imported placeholder requires its source graph")
                })?;
                let binding = scope.imported_placeholder_binding(marker, span)?;
                return Ok(Some(match binding {
                    jai_modules::Binding::Module(module) => Binding::Namespace(module),
                    binding => Binding::Imported(binding),
                }));
            }
        }
        Ok(None)
    }

    pub(crate) fn is_runtime_local_name(&self, name: Symbol) -> bool {
        for depth in (0..self.scopes.len()).rev() {
            if let Some(binding) = self.scopes[depth].get(&name) {
                return matches!(binding, Binding::Storage(_));
            }
            if let Some(frame) = self.local_scopes.frames.get(depth) {
                if frame.declarations.contains_key(&name) {
                    return false;
                }
                if frame.runtime.contains_key(&name) || frame.runtime_symbols.contains(&name) {
                    return true;
                }
                if let Some(binding) = frame.using_bindings.get(&name) {
                    return matches!(binding, Binding::Storage(_));
                }
                if frame.using_pending.contains(&name) {
                    return true;
                }
                if frame.using_placeholders.contains_key(&name) {
                    return false;
                }
                if frame.imports.contains_key(&name) {
                    return false;
                }
            }
        }
        false
    }

    fn resolve_local_declaration(
        &mut self,
        depth: usize,
        declaration: &Declaration,
    ) -> Result<Binding, Diagnostic> {
        if let Some(binding) = self.meta.local_declarations.entries[&declaration.id]
            .binding
            .clone()
        {
            self.local_scopes.next_scope = self
                .local_scopes
                .next_scope
                .max(self.meta.local_declarations.entries[&declaration.id].scope_watermark);
            if declaration.syntax.operator().is_none() {
                self.scopes[depth].insert(declaration.syntax.name(), binding.clone());
            }
            return Ok(binding);
        }
        if !self.local_scopes.active.insert(declaration.id) {
            if let Some(ty) = self.meta.local_declarations.entries[&declaration.id].nominal {
                return Ok(Binding::Type(ty));
            }
            if let Some(procedure) = self.meta.local_declarations.entries[&declaration.id].procedure
                && let Some(signature) = self.meta.local_declarations.signatures.get(&procedure)
            {
                return Ok(Binding::Procedure {
                    procedure,
                    ty: signature.ty,
                });
            }
            return Err(Diagnostic::new(
                declaration.syntax.span(),
                "cyclic local declaration dependencies",
            ));
        }
        // Resolve aliases/defaults in the declaration's environment. Inner
        // shadows must never alter a forward outer declaration's meaning.
        let inner_bindings = self.scopes.split_off(depth + 1);
        let inner_frames = self.local_scopes.frames.split_off(depth + 1);
        let previous_checks = std::mem::replace(&mut self.checks, declaration.checks);
        let previous_imports = std::mem::replace(
            &mut self.local_scopes.frames[depth].imports,
            declaration.imports.clone(),
        );
        let previous_operator_imports = std::mem::replace(
            &mut self.local_scopes.frames[depth].operator_imports,
            declaration.operator_imports.clone(),
        );
        let previous_using =
            imports::UsingPrefixEnvironment::from_frame(&self.local_scopes.frames[depth]);
        declaration
            .using
            .clone()
            .restore(&mut self.local_scopes.frames[depth]);
        let defining_source = declaration.id.defining_source();
        let previous_source = self
            .debug
            .replace_source(defining_source.or(self.debug.source()));
        let result = match &declaration.syntax {
            DeclarationSyntax::Library(library) => {
                self.define_local_library(declaration.id, library)
            }
            DeclarationSyntax::Record(record) => {
                if let Some(modifier) = &record.modify {
                    Err(Diagnostic::new(
                        modifier.span,
                        "local record #modify requires checked modifier execution",
                    ))
                } else if !record.parameters.is_empty() {
                    Err(Diagnostic::new(
                        record.span,
                        "local record templates require a lexical specialization origin",
                    ))
                } else {
                    self.define_local_record_body(
                        RecordSource {
                            id: declaration.id,
                            name: Some(record.name),
                            kind: record.kind,
                            attributes: &record.attributes,
                            notes: &record.notes,
                            span: record.span,
                        },
                        &record.members,
                    )
                    .map(Binding::Type)
                }
            }
            DeclarationSyntax::Enum(enumeration) => self
                .define_local_enum(declaration.id, enumeration)
                .map(Binding::Type),
            DeclarationSyntax::Alias(alias) => self
                .define_local_alias(declaration.id, alias)
                .map(Binding::Type),
            DeclarationSyntax::Constant(constant) => self.local_constant_binding(constant),
            DeclarationSyntax::ConstantResult(result) => self.local_constant_result_binding(result),
            DeclarationSyntax::Procedure(procedure) => {
                self.define_local_procedure(declaration.id, procedure)
            }
            DeclarationSyntax::Prototype(prototype) => {
                self.define_local_prototype(declaration.id, prototype)
            }
        };
        let result = match defining_source {
            Some(source) => result.map_err(|error| error.with_fallback_source(source)),
            None => result,
        };
        self.debug.replace_source(previous_source);
        self.scopes.extend(inner_bindings);
        self.local_scopes.frames.extend(inner_frames);
        self.checks = previous_checks;
        self.local_scopes.frames[depth].imports = previous_imports;
        self.local_scopes.frames[depth].operator_imports = previous_operator_imports;
        previous_using.restore(&mut self.local_scopes.frames[depth]);
        self.local_scopes.active.remove(&declaration.id);
        self.meta
            .local_declarations
            .entries
            .get_mut(&declaration.id)
            .unwrap()
            .scope_watermark = self.local_scopes.next_scope;
        if let Ok(binding) = &result {
            let signature_only = matches!(
                declaration.syntax,
                DeclarationSyntax::Procedure(_) | DeclarationSyntax::Prototype(_)
            ) && self.local_method_phase(declaration.id)
                != MethodPhase::Bodies;
            if !signature_only {
                self.meta
                    .local_declarations
                    .entries
                    .get_mut(&declaration.id)
                    .unwrap()
                    .binding = Some(binding.clone());
            }
            if declaration.syntax.operator().is_none() {
                self.scopes[depth].insert(declaration.syntax.name(), binding.clone());
            }
        }
        result
    }

    pub(crate) fn local_type_name(
        &mut self,
        path: &syntax::NamePath,
        span: Span,
    ) -> Result<TypeId, Diagnostic> {
        if let Some(binding) = self.namespace_binding(path, span)? {
            return match binding {
                Binding::Type(ty) => Ok(ty),
                Binding::Discarded(_) => Err(Diagnostic::new(
                    span,
                    "#discard parameter cannot be used as a type",
                )),
                _ => Err(Diagnostic::new(
                    span,
                    "record namespace member does not denote a type",
                )),
            };
        }
        if let Some(binding) = self.resolve_local_name(path.root, span)? {
            return match binding {
                Binding::Type(ty) if path.members.is_empty() => Ok(ty),
                Binding::Discarded(_) => Err(Diagnostic::new(
                    span,
                    "#discard parameter cannot be used as a type",
                )),
                Binding::Imported(binding) if path.members.is_empty() => {
                    match self.imported_binding_value(binding, span)? {
                        Binding::Type(ty) => Ok(ty),
                        _ => Err(Diagnostic::new(
                            span,
                            "imported declaration does not denote a type",
                        )),
                    }
                }
                Binding::Type(_) => Err(Diagnostic::new(
                    span,
                    "local type does not contain a nested type namespace",
                )),
                _ => Err(Diagnostic::new(
                    span,
                    "local declaration does not denote a type",
                )),
            };
        }
        if let Some(scope) = self.graph_scope {
            if let Some(binding @ jai_modules::Binding::SourceMember { .. }) =
                scope.using_graph_binding(path)
            {
                return match self.imported_binding_value(binding, span)? {
                    Binding::Type(ty) => Ok(ty),
                    _ => Err(Diagnostic::new(
                        span,
                        "imported source member does not denote a type",
                    )),
                };
            }
            if let Ok(ty) = scope.type_name(path, span) {
                return Ok(ty);
            }
        }
        if path.members.is_empty() && self.symbols.name(path.root) == "Code" {
            return Ok(self.types.code_type());
        }
        if path.members.is_empty()
            && let Some(builtin) = syntax::BuiltinType::from_spelling(self.symbols.name(path.root))
        {
            return self.lexical_annotation(&syntax::TypeSyntax::Builtin(builtin), span);
        }
        if let Some(ty) = self.reflection_type_name(path, span)? {
            return Ok(ty);
        }
        self.graph_scope
            .ok_or_else(|| Diagnostic::new(span, "type name requires a graph type scope"))?
            .type_name(path, span)
    }

    fn define_local_alias(
        &mut self,
        id: LocalDeclarationId,
        alias: &syntax::TypeAliasDeclaration,
    ) -> Result<TypeId, Diagnostic> {
        let ty = if let syntax::TypeSyntax::Variant { base, .. } = &alias.ty {
            let base = self.lexical_annotation(base, alias.span)?;
            let ty = self.meta.local_declarations.entries[&id].nominal.unwrap();
            self.types
                .define_distinct(ty, base)
                .map_err(|error| Diagnostic::new(alias.span, error.to_string()))?;
            ty
        } else {
            self.lexical_annotation(&alias.ty, alias.span)?
        };
        let source = self.retained_callback_syntax(&alias.ty, alias.span)?;
        self.meta
            .local_declarations
            .alias_sources
            .insert(id, source);
        Ok(ty)
    }

    pub(crate) fn local_declared_type(
        &mut self,
        name: Symbol,
        span: Span,
    ) -> Result<Option<TypeId>, Diagnostic> {
        for depth in (0..self.scopes.len()).rev() {
            if let Some(binding) = self.scopes[depth].get(&name).cloned() {
                if let Binding::LambdaPreview(crate::short_lambdas::PreviewBinding::Parameter(ty)) =
                    binding
                {
                    return Ok(Some(ty));
                }
                let value = self.binding_expression(binding, span)?;
                return self.expression_type(&value, span).map(Some);
            }
            if let Some(declaration) = self
                .local_scopes
                .frames
                .get(depth)
                .and_then(|frame| frame.runtime.get(&name))
                .cloned()
            {
                return match declaration {
                    syntax::Declaration::Explicit { ty, .. } => Ok(Some(self.types.scalar(ty))),
                    syntax::Declaration::UnresolvedExplicit { ty, .. }
                    | syntax::Declaration::External { ty, .. } => {
                        let inner_bindings = self.scopes.split_off(depth + 1);
                        let inner_frames = self.local_scopes.frames.split_off(depth + 1);
                        let result = self.preview_annotation(&ty, span).map(Some);
                        self.scopes.extend(inner_bindings);
                        self.local_scopes.frames.extend(inner_frames);
                        result
                    }
                    syntax::Declaration::Inferred { .. } => Err(Diagnostic::new(
                        span,
                        "forward type_of on an inferred variable requires its initializer's checked type",
                    )),
                };
            }
        }
        Ok(None)
    }

    pub(crate) fn reject_lexical_capture(
        &self,
        name: Symbol,
        span: Span,
    ) -> Result<(), Diagnostic> {
        for depth in (0..self.scopes.len()).rev() {
            if self.scopes[depth].contains_key(&name) {
                return Ok(());
            }
            if let Some(frame) = self.local_scopes.frames.get(depth) {
                if frame.declarations.contains_key(&name) {
                    return Ok(());
                }
                if frame.runtime.contains_key(&name) || frame.runtime_symbols.contains(&name) {
                    if frame.id.owner != LexicalScopeOwner::Procedure(self.lexical_owner()) {
                        return Err(Diagnostic::new(
                            span,
                            format!(
                                "nested procedure cannot capture runtime local '{}'",
                                self.symbols.name(name)
                            ),
                        ));
                    }
                    return Ok(());
                }
            }
        }
        Ok(())
    }

    pub(crate) fn check_local_storage_capture(
        &self,
        storage: Storage,
        span: Span,
    ) -> Result<(), Diagnostic> {
        let mut place = storage.place();
        loop {
            place = match place.kind() {
                PlaceKind::Local(id) => {
                    if id.procedure() != self.procedure {
                        return Err(Diagnostic::new(
                            span,
                            "nested procedure cannot capture runtime local storage",
                        ));
                    }
                    return Ok(());
                }
                PlaceKind::Field(id) => {
                    self.places
                        .projection(id)
                        .map_err(|error| Diagnostic::new(span, error.to_string()))?
                        .base
                }
                PlaceKind::Index(id) => {
                    self.places
                        .index_projection(id)
                        .map_err(|error| Diagnostic::new(span, error.to_string()))?
                        .base
                }
                PlaceKind::SequenceField(id) => {
                    self.places
                        .sequence_projection(id)
                        .map_err(|error| Diagnostic::new(span, error.to_string()))?
                        .base
                }
                _ => return Ok(()),
            };
        }
    }
}
