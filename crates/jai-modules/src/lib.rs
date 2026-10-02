//! Independently scoped source/module graph; no executable lowering occurs here.
mod loader;
use jai_source::{
    DeclarationId, Diagnostic, LocatedDiagnostic, ModuleId, ScopeId, SourceId, SourceMap,
    SourceSpan, Symbol, Symbols, UnitId,
};
use jai_syntax::{FileDeclaration, FileDeclarationKind, NamePath, ParsedFile, Visibility};
use loader::Builder;
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Binding {
    Declaration(DeclarationId),
    Module(ModuleId),
}
#[derive(Debug)]
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
            FileDeclarationKind::Procedure(p) => p.name,
            FileDeclarationKind::Global(g) => g.declaration.name(),
            FileDeclarationKind::Constant(c) => c.name,
            FileDeclarationKind::Record(r) => r.name,
            FileDeclarationKind::Enum(e) => e.name,
        }
    }
}
#[derive(Debug)]
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
#[derive(Debug)]
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
    Io {
        path: PathBuf,
        cause: std::io::Error,
    },
    Decode {
        path: PathBuf,
        diagnostic: Diagnostic,
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
            Self::Io { path, cause } => write!(f, "{}: {cause}", path.display()),
            Self::Decode { path, diagnostic } => write!(f, "{}: {diagnostic}", path.display()),
            Self::Located { rendered, .. } => f.write_str(rendered),
            Self::Cycle { rendered, .. } => f.write_str(rendered),
        }
    }
}
impl std::error::Error for GraphError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { cause, .. } => Some(cause),
            Self::Decode { diagnostic, .. } => Some(diagnostic),
            Self::Located { diagnostic, .. } => Some(diagnostic),
            Self::Cycle { .. } => None,
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
}
#[derive(Debug)]
pub struct ModuleGraph {
    unit: UnitId,
    root: ModuleId,
    sources: SourceMap,
    symbols: Symbols,
    files: Vec<FileInstance>,
    modules: Vec<ModuleInstance>,
    declarations: Vec<Declaration>,
    imports: Vec<ImportEdge>,
    loads: Vec<LoadEdge>,
}
impl ModuleGraph {
    pub fn load(path: &Path, options: GraphOptions) -> Result<Self, GraphError> {
        Builder::new(options).build(path)
    }
    pub fn unit(&self) -> UnitId {
        self.unit
    }
    pub fn root(&self) -> ModuleId {
        self.root
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
    pub fn loads(&self) -> &[LoadEdge] {
        &self.loads
    }
    pub fn declarations(&self) -> &[Declaration] {
        &self.declarations
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
    pub fn lookup(&self, file: FileInstanceId, path: &NamePath) -> Result<Binding, LookupError> {
        let file = self.file(file).ok_or(LookupError::InvalidFile)?;
        let module = &self.modules[file.module.index()];
        let mut binding = file
            .private
            .get(&path.root)
            .or_else(|| module.bindings.get(&path.root))
            .copied()
            .ok_or(LookupError::UnknownName(path.root))?;
        let mut previous = path.root;
        for &name in &path.members {
            let Binding::Module(id) = binding else {
                return Err(LookupError::NotNamespace(previous));
            };
            let module = &self.modules[id.index()];
            binding = module.exports.get(&name).copied().ok_or_else(|| {
                if module.bindings.contains_key(&name) {
                    LookupError::PrivateMember { module: id, name }
                } else {
                    LookupError::UnknownMember { module: id, name }
                }
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
