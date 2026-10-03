//! Independently scoped source/module graph; no executable lowering occurs here.
mod declaration_insertions;
mod deferred_cases;
pub use declaration_insertions::{
    DeclarationInsertionCode, DeclarationInsertionPublication, DeclarationInsertionRequest,
    InsertionAdmission, InsertionPublicationError, InsertionRequestId, InsertionResponseError,
    InsertionTransaction, SourceCaptureValue,
};
mod discovery;
pub use deferred_cases::{CaseRequestId, CaseSelectionError, DeferredCase, SourceCaseSelection};
mod loader;
pub use discovery::{
    ConditionRequestId, ConditionSelectionError, DeferredCondition, DeferredDependency,
    DiscoveryConditionContext, DiscoveryLexicalScope, DiscoveryStatus, GraphDiscovery,
};
mod bootstrap;
mod compiler_prelude;
pub use compiler_prelude::{COMPILER_PRELUDE_ENTRY, compiler_prelude_source};
mod parameter_requests;
mod params;
mod placeholder_bindings;
mod placeholders;
pub use parameter_requests::{
    DeferredParameter, ParameterRequestId, ParameterResponse, ParameterResponseError, ParameterTask,
};
pub use placeholders::{Placeholder, PlaceholderId};
mod operator_aliases;
mod operator_scopes;
mod scoped_imports;
mod source_arguments;
mod source_origins;
mod source_storage;
pub use source_origins::{SourceOriginAdmissionError, SourceOriginError};
pub use source_storage::GraphSyntaxStorage;
mod source_specializations;
mod storage_members;
pub use storage_members::{
    MAX_SOURCE_STORAGE_PATH_DEPTH, SourceStorageMember, SourceStorageMemberId,
};
mod using_requests;
pub use using_requests::{
    FileUsingDecision, FileUsingRequest, UsingAlias, UsingBinding, UsingDeclarationSource,
    UsingPlaceholder, UsingPublication, UsingRequestId, UsingResponseError, UsingStorageMember,
};
mod type_parameters;
pub use source_arguments::ModuleBoundArgument;
pub use source_specializations::{SourceDependencyTemplate, SourceSpecializationKey};
pub use type_parameters::{ModuleBuiltin, ModuleProcedureType, ModuleType, ModuleVariadic};
mod enum_masks;
mod enum_parameters;
mod source_provider;
pub use bootstrap::{
    BootstrapOptions, PreludeError, PreludeSource, RuntimeSupportOptions, RuntimeSupportParameters,
    RuntimeSupportSource,
};
use jai_source::{
    DeclarationId, Diagnostic, LocatedDiagnostic, ModuleId, ScopeId, SourceId, SourceMap,
    SourceSpan, Symbol, Symbols, UnitId,
};
use jai_syntax::{FileDeclaration, FileDeclarationKind, NamePath, ParsedFile, Visibility};
use loader::Builder;
pub use params::{BoundParameter, EnumParameter, ParameterId, ParameterValue};
pub use source_provider::{Filesystem, SourceOverlay, SourceProvider};
use std::{
    collections::HashMap,
    fmt,
    path::{Path, PathBuf},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FileInstanceId(usize);
impl FileInstanceId {
    pub fn index(self) -> usize {
        self.0
    }
}
/// Whether a branch was chosen by graph scalar evaluation or canonical semantics.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceConditionOrigin {
    Scalar,
    Semantic,
}
/// One immutable source branch decision in its original file and source span.
#[derive(Clone, Debug)]
pub struct SourceConditionSelection {
    file: FileInstanceId,
    location: SourceSpan,
    selected: bool,
    origin: SourceConditionOrigin,
    specialization: Option<SourceSpecializationKey>,
}
impl SourceConditionSelection {
    pub fn file(&self) -> FileInstanceId {
        self.file
    }
    pub fn location(&self) -> SourceSpan {
        self.location
    }
    pub fn selected(&self) -> bool {
        self.selected
    }
    pub fn origin(&self) -> SourceConditionOrigin {
        self.origin
    }
    pub fn specialization(&self) -> Option<&SourceSpecializationKey> {
        self.specialization.as_ref()
    }
}
#[derive(Clone, Debug)]
pub struct FileRun {
    pub file: FileInstanceId,
    pub syntax: jai_syntax::RunDirective,
}
#[derive(Clone, Debug)]
pub struct FileInsertion {
    pub file: FileInstanceId,
    pub directive: jai_syntax::InsertDirective,
    pub location: SourceSpan,
}
#[derive(Clone, Debug)]
pub struct ContextField {
    file: FileInstanceId,
    syntax: jai_syntax::ContextFieldDeclaration,
    location: SourceSpan,
}
impl ContextField {
    pub fn file(&self) -> FileInstanceId {
        self.file
    }
    pub fn syntax(&self) -> &jai_syntax::ContextFieldDeclaration {
        &self.syntax
    }
    pub fn location(&self) -> SourceSpan {
        self.location
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct OverloadSetId(usize);
impl OverloadSetId {
    pub fn index(self) -> usize {
        self.0
    }
}
/// Immutable declaration membership; exported and private sets stay independent.
#[derive(Clone, Debug)]
pub struct OverloadSet {
    id: OverloadSetId,
    declarations: Box<[DeclarationId]>,
}
impl OverloadSet {
    pub fn id(&self) -> OverloadSetId {
        self.id
    }
    pub fn declarations(&self) -> &[DeclarationId] {
        &self.declarations
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Binding {
    Declaration(DeclarationId),
    OverloadSet(OverloadSetId),
    Module(ModuleId),
    Parameter(ParameterId),
    StorageMember(SourceStorageMemberId),
    /// An original enum or record namespace member, rematerialized by semantics.
    SourceMember {
        declaration: DeclarationId,
        member: Symbol,
    },
}
#[derive(Clone, Debug)]
pub struct Declaration {
    id: DeclarationId,
    file: FileInstanceId,
    syntax: FileDeclaration,
}
impl Declaration {
    pub fn id(&self) -> DeclarationId {
        self.id
    }
    pub fn file(&self) -> FileInstanceId {
        self.file
    }
    pub fn syntax(&self) -> &FileDeclaration {
        &self.syntax
    }
    pub fn location(&self) -> SourceSpan {
        self.syntax.location
    }
    pub fn name(&self) -> Symbol {
        match &self.syntax.kind {
            FileDeclarationKind::Placeholder(marker) => marker.name,
            FileDeclarationKind::Library(library) => library.name,
            FileDeclarationKind::Procedure(p) => p.name,
            FileDeclarationKind::OperatorAlias(alias) => alias.name,
            FileDeclarationKind::ProcedurePrototype(p) => p.name,
            FileDeclarationKind::Global(g) => g.declaration.name(),
            FileDeclarationKind::Constant(c) => c.name,
            FileDeclarationKind::Record(r) => r.name,
            FileDeclarationKind::Enum(e) => e.name,
            FileDeclarationKind::TypeAlias(alias) => alias.name,
        }
    }
}
#[derive(Clone, Debug)]
pub struct FileInstance {
    id: FileInstanceId,
    module: ModuleId,
    source: SourceId,
    scope: ScopeId,
    private: HashMap<Symbol, Binding>,
    declarations: Vec<DeclarationId>,
    syntax: ParsedFile,
}
impl FileInstance {
    pub fn id(&self) -> FileInstanceId {
        self.id
    }
    pub fn module(&self) -> ModuleId {
        self.module
    }
    pub fn source(&self) -> SourceId {
        self.source
    }
    pub fn scope(&self) -> ScopeId {
        self.scope
    }
    pub fn private_bindings(&self) -> &HashMap<Symbol, Binding> {
        &self.private
    }
    pub fn declarations(&self) -> &[DeclarationId] {
        &self.declarations
    }
    pub fn syntax(&self) -> &ParsedFile {
        &self.syntax
    }
}
#[derive(Clone, Debug)]
pub struct ModuleInstance {
    id: ModuleId,
    scope: ScopeId,
    entry: Option<FileInstanceId>,
    files: Vec<FileInstanceId>,
    bindings: HashMap<Symbol, Binding>,
    exports: HashMap<Symbol, Binding>,
}
impl ModuleInstance {
    pub fn id(&self) -> ModuleId {
        self.id
    }
    pub fn scope(&self) -> ScopeId {
        self.scope
    }
    pub fn entry(&self) -> FileInstanceId {
        self.entry
            .expect("completed graph module has an entry file")
    }
    /// Partial discovery snapshots may contain an allocated module before its
    /// entry source has been published.
    pub fn discovered_entry(&self) -> Option<FileInstanceId> {
        self.entry
    }
    pub fn files(&self) -> &[FileInstanceId] {
        &self.files
    }
    pub fn bindings(&self) -> &HashMap<Symbol, Binding> {
        &self.bindings
    }
    pub fn exports(&self) -> &HashMap<Symbol, Binding> {
        &self.exports
    }
}
#[derive(Clone, Copy, Debug)]
pub struct ImportEdge {
    file: FileInstanceId,
    module: ModuleId,
    location: SourceSpan,
}
impl ImportEdge {
    pub fn file(&self) -> FileInstanceId {
        self.file
    }
    pub fn module(&self) -> ModuleId {
        self.module
    }
    pub fn location(&self) -> SourceSpan {
        self.location
    }
}
/// A source dependency whose exported names belong to a lexical body scope.
/// Loading its module does not add names to the importing file or module.
#[derive(Clone, Debug)]
pub struct ScopedImportEdge {
    file: FileInstanceId,
    module: ModuleId,
    location: SourceSpan,
    specialization: Option<SourceSpecializationKey>,
}
impl ScopedImportEdge {
    pub fn specialization(&self) -> Option<&SourceSpecializationKey> {
        self.specialization.as_ref()
    }
    pub fn file(&self) -> FileInstanceId {
        self.file
    }
    pub fn module(&self) -> ModuleId {
        self.module
    }
    pub fn location(&self) -> SourceSpan {
        self.location
    }
}
#[derive(Clone, Copy, Debug)]
pub struct LoadEdge {
    file: FileInstanceId,
    target: FileInstanceId,
    location: SourceSpan,
}
impl LoadEdge {
    pub fn file(&self) -> FileInstanceId {
        self.file
    }
    pub fn target(&self) -> FileInstanceId {
        self.target
    }
    pub fn location(&self) -> SourceSpan {
        self.location
    }
}
#[derive(Clone, Debug)]
pub struct GraphOptions {
    pub import_dirs: Vec<PathBuf>,
}
impl Default for GraphOptions {
    fn default() -> Self {
        Self {
            import_dirs: vec![PathBuf::from("modules")],
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DependencyKind {
    Load,
    Import,
}
#[derive(Debug)]
pub enum GraphError {
    InvalidUsingResponse(UsingResponseError),
    /// A failed session remains available for source diagnostics, but cannot
    /// publish a graph after an earlier hard discovery error.
    FailedDiscovery {
        rendered: String,
    },
    Prelude(PreludeError),
    Io {
        path: PathBuf,
        cause: std::io::Error,
    },
    Decode {
        path: PathBuf,
        diagnostic: Diagnostic,
    },
    Pending {
        diagnostic: LocatedDiagnostic,
        rendered: String,
    },
    Unsupported {
        diagnostic: LocatedDiagnostic,
        rendered: String,
    },
    Located {
        diagnostic: LocatedDiagnostic,
        rendered: String,
    },
    Cycle {
        kind: DependencyKind,
        path: PathBuf,
        location: Option<SourceSpan>,
        rendered: String,
    },
}
impl fmt::Display for GraphError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidUsingResponse(error) => fmt::Display::fmt(error, f),
            Self::FailedDiscovery {
                rendered,
            } => f.write_str(rendered),
            Self::Prelude(error) => fmt::Display::fmt(error, f),
            Self::Io {
                path,
                cause,
            } => write!(f, "{}: {cause}", path.display()),
            Self::Decode {
                path,
                diagnostic,
            } => write!(f, "{}: {diagnostic}", path.display()),
            Self::Located {
                rendered, ..
            }
            | Self::Pending {
                rendered, ..
            }
            | Self::Unsupported {
                rendered, ..
            } => f.write_str(rendered),
            Self::Cycle {
                rendered, ..
            } => f.write_str(rendered),
        }
    }
}
impl std::error::Error for GraphError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidUsingResponse(error) => Some(error),
            Self::FailedDiscovery {
                ..
            } => None,
            Self::Prelude(error) => Some(error),
            Self::Io {
                cause, ..
            } => Some(cause),
            Self::Decode {
                diagnostic, ..
            } => Some(diagnostic),
            Self::Located {
                diagnostic, ..
            }
            | Self::Pending {
                diagnostic, ..
            }
            | Self::Unsupported {
                diagnostic, ..
            } => Some(diagnostic),
            Self::Cycle {
                ..
            } => None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LookupError {
    InvalidFile,
    UnknownName(Symbol),
    NotNamespace(Symbol),
    UnknownMember { module: ModuleId, name: Symbol },
    PrivateMember { module: ModuleId, name: Symbol },
    UnfilledPlaceholder(PlaceholderId),
}
#[derive(Clone, Debug)]
pub struct ModuleGraph {
    unit: UnitId,
    target: Option<jai_types::BuildTarget>,
    root: ModuleId,
    prelude: Option<ModuleId>,
    runtime_support: Option<ModuleId>,
    sources: SourceMap,
    symbols: Symbols,
    files: Vec<FileInstance>,
    modules: Vec<ModuleInstance>,
    declarations: Vec<Declaration>,
    placeholders: placeholders::Placeholders,
    imports: Vec<ImportEdge>,
    scoped_imports: Vec<ScopedImportEdge>,
    source_conditions: Vec<SourceConditionSelection>,
    source_cases: Vec<SourceCaseSelection>,
    dependency_templates: Vec<SourceDependencyTemplate>,
    source_specializations: Vec<SourceSpecializationKey>,
    using_publications: Vec<UsingPublication>,
    storage_members: storage_members::StorageMembers,
    loads: Vec<LoadEdge>,
    runs: Vec<FileRun>,
    insertions: Vec<FileInsertion>,
    insertion_publications: Vec<DeclarationInsertionPublication>,
    context_fields: Vec<ContextField>,
    parameters: Vec<BoundParameter>,
    source_requests: HashMap<ModuleId, params::ModuleKey>,
    overload_sets: Vec<OverloadSet>,
}
impl ModuleGraph {
    /// Load actual Preload source into one shared fallback module before user files.
    /// Existing low-level load APIs deliberately omit implicit bootstrap.
    pub fn load_with_bootstrap(
        path: &Path,
        options: GraphOptions,
        prelude: PreludeSource,
        provider: &dyn SourceProvider,
        target: Option<jai_types::BuildTarget>,
    ) -> Result<Self, GraphError> {
        Builder::new_with_target(options, provider, target).build_with_bootstrap(path, prelude)
    }
    /// Bootstrap automatic source modules with explicit compiler runtime policy.
    pub fn load_with_bootstrap_options(
        path: &Path,
        options: GraphOptions,
        bootstrap: BootstrapOptions,
        provider: &dyn SourceProvider,
        target: Option<jai_types::BuildTarget>,
    ) -> Result<Self, GraphError> {
        Builder::new_with_target(options, provider, target)
            .build_with_bootstrap_options(path, bootstrap)
    }
    pub fn load(path: &Path, options: GraphOptions) -> Result<Self, GraphError> {
        Self::load_with_provider(path, options, &Filesystem)
    }
    pub fn load_with_provider(
        path: &Path,
        options: GraphOptions,
        provider: &dyn SourceProvider,
    ) -> Result<Self, GraphError> {
        Builder::new(options, provider).build(path)
    }
    /// Source selection uses supplied target facts and source-defined tag enums.
    pub fn load_with_target(
        path: &Path,
        options: GraphOptions,
        provider: &dyn SourceProvider,
        target: jai_types::BuildTarget,
    ) -> Result<Self, GraphError> {
        Builder::new_with_target(options, provider, Some(target)).build(path)
    }
    pub fn target(&self) -> Option<&jai_types::BuildTarget> {
        self.target.as_ref()
    }
    pub fn unit(&self) -> UnitId {
        self.unit
    }
    pub fn root(&self) -> ModuleId {
        self.root
    }
    /// Canonical shared Preload module; absence explicitly denotes no bootstrap.
    pub fn prelude(&self) -> Option<ModuleId> {
        self.prelude
    }
    pub fn runtime_support(&self) -> Option<ModuleId> {
        self.runtime_support
    }
    pub fn sources(&self) -> &SourceMap {
        &self.sources
    }
    pub fn symbols(&self) -> &Symbols {
        &self.symbols
    }
    pub fn files(&self) -> &[FileInstance] {
        &self.files
    }
    pub fn modules(&self) -> &[ModuleInstance] {
        &self.modules
    }
    pub fn imports(&self) -> &[ImportEdge] {
        &self.imports
    }
    pub fn scoped_imports(&self) -> &[ScopedImportEdge] {
        &self.scoped_imports
    }
    /// Completed source selections survive discovery so semantic resolution
    /// can consume the chosen branch without replaying semantic #run guards.
    pub fn source_condition_selections(&self) -> &[SourceConditionSelection] {
        &self.source_conditions
    }
    pub fn dependency_templates(&self) -> &[SourceDependencyTemplate] {
        &self.dependency_templates
    }
    pub fn source_specializations(&self) -> &[SourceSpecializationKey] {
        &self.source_specializations
    }
    pub fn selected_condition(&self, file: FileInstanceId, span: jai_source::Span) -> Option<bool> {
        self.selected_condition_for(file, span, None)
    }
    pub fn selected_condition_for(
        &self,
        file: FileInstanceId,
        span: jai_source::Span,
        specialization: Option<&SourceSpecializationKey>,
    ) -> Option<bool> {
        self.source_conditions
            .iter()
            .find(|selection| {
                selection.file == file
                    && selection.location.span == span
                    && selection.specialization.as_ref() == specialization
            })
            .map(|selection| selection.selected)
    }
    /// Only canonical typed decisions may bypass later guard validation.
    pub fn selected_semantic_condition(
        &self,
        file: FileInstanceId,
        span: jai_source::Span,
    ) -> Option<bool> {
        self.selected_semantic_condition_for(file, span, None)
    }
    pub fn selected_semantic_condition_for(
        &self,
        file: FileInstanceId,
        span: jai_source::Span,
        specialization: Option<&SourceSpecializationKey>,
    ) -> Option<bool> {
        self.source_conditions
            .iter()
            .find(|selection| {
                selection.file == file
                    && selection.location.span == span
                    && selection.specialization.as_ref() == specialization
                    && selection.origin == SourceConditionOrigin::Semantic
            })
            .map(|selection| selection.selected)
    }
    /// Resolve the retained import statement identity in its original file.
    pub fn scoped_import(&self, file: FileInstanceId, span: jai_source::Span) -> Option<ModuleId> {
        self.scoped_import_for(file, span, None)
    }
    pub fn scoped_import_for(
        &self,
        file: FileInstanceId,
        span: jai_source::Span,
        specialization: Option<&SourceSpecializationKey>,
    ) -> Option<ModuleId> {
        self.scoped_imports
            .iter()
            .find(|edge| {
                edge.file == file
                    && edge.location.span == span
                    && edge.specialization.as_ref() == specialization
            })
            .map(|edge| edge.module)
    }
    /// Traverse only exports, even when the importer is in the same program.
    pub fn lookup_module(
        &self,
        module: ModuleId,
        members: &[Symbol],
    ) -> Result<Binding, LookupError> {
        let mut binding = Binding::Module(module);
        let mut previous = None;
        for &name in members {
            let Binding::Module(module) = binding else {
                return Err(LookupError::NotNamespace(previous.unwrap_or(name)));
            };
            let module = self.module(module).ok_or(LookupError::InvalidFile)?;
            binding = module
                .exports
                .get(&name)
                .copied()
                .map(Ok)
                .or_else(|| self.placeholder_module_lookup(module.id, name))
                .unwrap_or_else(|| {
                    Err(if module.bindings.contains_key(&name) {
                        LookupError::PrivateMember {
                            module: module.id,
                            name,
                        }
                    } else {
                        LookupError::UnknownMember {
                            module: module.id,
                            name,
                        }
                    })
                })?;
            previous = Some(name);
        }
        Ok(binding)
    }
    pub fn loads(&self) -> &[LoadEdge] {
        &self.loads
    }
    /// Active file directives only; inactive #if branches never enqueue effects.
    pub fn runs(&self) -> &[FileRun] {
        &self.runs
    }
    pub fn insertions(&self) -> &[FileInsertion] {
        &self.insertions
    }
    pub fn context_fields(&self) -> &[ContextField] {
        &self.context_fields
    }
    pub fn parameters(&self) -> &[BoundParameter] {
        &self.parameters
    }
    pub fn parameter(&self, id: ParameterId) -> Option<&BoundParameter> {
        self.parameters.get(id.index())
    }
    pub fn declarations(&self) -> &[Declaration] {
        &self.declarations
    }
    pub fn overload_sets(&self) -> &[OverloadSet] {
        &self.overload_sets
    }
    pub fn overload_set(&self, id: OverloadSetId) -> Option<&OverloadSet> {
        self.overload_sets.get(id.0)
    }
    pub fn file(&self, id: FileInstanceId) -> Option<&FileInstance> {
        self.files.get(id.0)
    }
    pub fn module(&self, id: ModuleId) -> Option<&ModuleInstance> {
        self.modules.get(id.index())
    }
    pub fn declaration(&self, id: DeclarationId) -> Option<&Declaration> {
        self.declarations.get(id.index())
    }
    /// Locate an original child declaration without introducing a source name.
    /// Discarded using owners remain distinct through their file and source span.
    pub fn declaration_at(
        &self,
        file: FileInstanceId,
        location: SourceSpan,
    ) -> Option<&Declaration> {
        self.declarations
            .iter()
            .find(|declaration| declaration.file == file && declaration.location() == location)
    }
    pub(crate) fn canonical_binding_file(&self, mut file: FileInstanceId) -> FileInstanceId {
        while let Some(publication) = self.insertion_publication(file) {
            file = publication.destination;
        }
        file
    }
    pub fn lookup(&self, file: FileInstanceId, path: &NamePath) -> Result<Binding, LookupError> {
        let insertion_binding = self.insertion_root_binding(file, path.root);
        let file = self.file(file).ok_or(LookupError::InvalidFile)?;
        let module = &self.modules[file.module.index()];
        let mut binding = if let Some(binding) = insertion_binding {
            binding?
        } else {
            file.private
                .get(&path.root)
                .copied()
                .map(Ok)
                .or_else(|| {
                    self.placeholder_namespace_lookup(
                        placeholders::PlaceholderScope::File(self.canonical_binding_file(file.id)),
                        path.root,
                    )
                })
                .or_else(|| module.bindings.get(&path.root).copied().map(Ok))
                .or_else(|| {
                    self.placeholder_namespace_lookup(
                        placeholders::PlaceholderScope::Module(file.module),
                        path.root,
                    )
                })
                .or_else(|| {
                    self.prelude.and_then(|id| {
                        self.modules[id.index()]
                            .exports
                            .get(&path.root)
                            .copied()
                            .map(Ok)
                            .or_else(|| self.placeholder_module_lookup(id, path.root))
                    })
                })
                .or_else(|| {
                    self.runtime_support.and_then(|id| {
                        self.modules[id.index()]
                            .exports
                            .get(&path.root)
                            .copied()
                            .map(Ok)
                            .or_else(|| self.placeholder_module_lookup(id, path.root))
                    })
                })
                .unwrap_or(Err(LookupError::UnknownName(path.root)))?
        };
        let mut previous = path.root;
        for &name in &path.members {
            let Binding::Module(id) = binding else {
                return Err(LookupError::NotNamespace(previous));
            };
            let module = &self.modules[id.index()];
            binding = module
                .exports
                .get(&name)
                .copied()
                .map(Ok)
                .or_else(|| self.placeholder_module_lookup(id, name))
                .unwrap_or_else(|| {
                    Err(if module.bindings.contains_key(&name) {
                        LookupError::PrivateMember {
                            module: id,
                            name,
                        }
                    } else {
                        LookupError::UnknownMember {
                            module: id,
                            name,
                        }
                    })
                })?;
            previous = name;
        }
        Ok(binding)
    }
    pub fn locate(&self, file: FileInstanceId, span: jai_source::Span) -> Option<SourceSpan> {
        self.file(file).map(|file| SourceSpan {
            source: file.source,
            span,
        })
    }
    pub fn diagnostic(
        &self,
        location: SourceSpan,
        message: impl Into<String>,
    ) -> LocatedDiagnostic {
        LocatedDiagnostic {
            location,
            message: message.into(),
        }
    }
}
#[cfg(test)]
mod tests;

pub use jai_syntax::SourceMetadataError;
