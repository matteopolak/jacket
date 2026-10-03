//! Independent selected-source receipts for virtual stdio and heap adapters.
//! No receipt permits loading a native library or executing supplied native bytes.
use super::*;
#[path = "file_abi_bindings/heap.rs"]
mod heap;
pub(super) use heap::{bind_heap, bind_heap_ready};
use jai_types::{Architecture, BuildTarget, ByteOrder, OperatingSystem};
use jai_vm::file_abi::{FileAbiOperation, FileAbiProcedure, StdioAuthority};
use std::{path::PathBuf, sync::Arc};

#[derive(Clone, Debug)]
struct SourceReceipt {
    file: FileInstanceId,
    path: PathBuf,
    text: Arc<str>,
}
#[derive(Clone, Debug)]
struct StdioSourceReceipt {
    entry: SourceReceipt,
    declarations: SourceReceipt,
}
#[derive(Clone, Debug)]
struct AllocatorSourceReceipt {
    entry: SourceReceipt,
    roles: Vec<(DeclarationId, jai_vm::heap_abi::HeapAbiOperation)>,
}
/// Explicit embedding authority for independently selected source roles. A receipt
/// retains the actual graph unit, immutable source allocation and declaration IDs.
#[derive(Clone, Debug)]
pub struct FileAbiBindingContext {
    unit: jai_source::UnitId,
    target: BuildTarget,
    stdio: Option<StdioSourceReceipt>,
    allocator: Vec<AllocatorSourceReceipt>,
}
impl FileAbiBindingContext {
    pub fn from_graph(
        graph: &ModuleGraph,
        import_dirs: &[PathBuf],
        target: BuildTarget,
    ) -> Option<Self> {
        Self::selected_roles(graph, import_dirs, target, true)
    }
    /// Pure source resolution may select virtual heap leaves independently of
    /// file effects. This context never carries a stdio source role.
    pub fn allocator_from_graph(
        graph: &ModuleGraph,
        import_dirs: &[PathBuf],
        target: BuildTarget,
    ) -> Option<Self> {
        Self::selected_roles(graph, import_dirs, target, false)
    }
    fn selected_roles(
        graph: &ModuleGraph,
        import_dirs: &[PathBuf],
        target: BuildTarget,
        include_stdio: bool,
    ) -> Option<Self> {
        if target.layout != jai_types::LayoutPolicy::lp64()
            || target.byte_order != ByteOrder::Little
            || graph.target().is_some_and(|actual| actual != &target)
        {
            return None;
        }
        let relative = match (&target.operating_system, &target.architecture) {
            (OperatingSystem::MacOS, Architecture::Arm64) => "macos/arm64/stdio.jai",
            (OperatingSystem::MacOS, Architecture::X86_64) => "macos/x64/stdio.jai",
            (OperatingSystem::Linux, Architecture::Arm64 | Architecture::X86_64) => {
                "linux/stdio.jai"
            }
            _ => return None,
        };
        let mut stdio = None;
        let mut allocator = Vec::new();
        for directory in import_dirs {
            for entry in selected_entries(graph, directory.join("Default_Allocator/module.jai")) {
                if allocator
                    .iter()
                    .all(|source: &AllocatorSourceReceipt| source.entry.file != entry.file)
                {
                    allocator.push(AllocatorSourceReceipt::selected(graph, entry));
                }
            }
            if include_stdio && stdio.is_none() {
                stdio =
                    selected_entry(graph, directory.join("POSIX/module.jai")).and_then(|entry| {
                        selected_file(
                            graph,
                            &entry,
                            directory.join("POSIX/bindings").join(relative),
                        )
                        .map(|declarations| StdioSourceReceipt {
                            entry,
                            declarations,
                        })
                    });
            }
        }
        if stdio.is_none() && allocator.is_empty() {
            return None;
        }
        Some(Self {
            unit: graph.unit(),
            target,
            stdio,
            allocator,
        })
    }
    /// Whether this same graph includes an independently selected allocator role.
    pub fn includes_default_allocator(&self) -> bool {
        !self.allocator.is_empty()
    }
    pub fn target(&self) -> &BuildTarget {
        &self.target
    }
}
impl AllocatorSourceReceipt {
    fn selected(graph: &ModuleGraph, entry: SourceReceipt) -> Self {
        use jai_vm::heap_abi::HeapAbiOperation;
        let roles = graph
            .declarations()
            .iter()
            .filter(|declaration| declaration.file() == entry.file)
            .filter_map(|declaration| {
                let FileDeclarationKind::ProcedurePrototype(prototype) = &declaration.syntax().kind
                else {
                    return None;
                };
                let role = match graph.symbols().name(prototype.name) {
                    "c_malloc" => HeapAbiOperation::Malloc,
                    "c_realloc" => HeapAbiOperation::Realloc,
                    "c_free" => HeapAbiOperation::Free,
                    _ => return None,
                };
                Some((declaration.id(), role))
            })
            .collect();
        Self {
            entry,
            roles,
        }
    }
}
fn receipt(graph: &ModuleGraph, file: FileInstanceId) -> Option<SourceReceipt> {
    let source = graph.sources().get(graph.file(file)?.source())?;
    Some(SourceReceipt {
        file,
        path: source.path().to_owned(),
        text: source.shared_text(),
    })
}
fn selected_entries(graph: &ModuleGraph, path: PathBuf) -> Vec<SourceReceipt> {
    let Ok(path) = path.canonicalize() else {
        return Vec::new();
    };
    graph
        .modules()
        .iter()
        .filter_map(|module| receipt(graph, module.entry()))
        .filter(|receipt| receipt.path == path)
        .collect()
}
fn selected_entry(graph: &ModuleGraph, path: PathBuf) -> Option<SourceReceipt> {
    selected_entries(graph, path).into_iter().next()
}
fn selected_file(
    graph: &ModuleGraph,
    entry: &SourceReceipt,
    path: PathBuf,
) -> Option<SourceReceipt> {
    let path = path.canonicalize().ok()?;
    graph
        .modules()
        .iter()
        .filter(|module| module.entry() == entry.file)
        .flat_map(|module| module.files())
        .filter_map(|file| receipt(graph, *file))
        .find(|receipt| receipt.path == path)
}
fn matches_receipt(graph: &ModuleGraph, expected: &SourceReceipt) -> bool {
    receipt(graph, expected.file).is_some_and(|actual| {
        actual.path == expected.path && Arc::ptr_eq(&actual.text, &expected.text)
    })
}
fn matches_context(
    graph: &ModuleGraph,
    context: &FileAbiBindingContext,
    target: Option<&BuildTarget>,
) -> bool {
    graph.unit() == context.unit
        && target == Some(&context.target)
        && graph
            .target()
            .is_none_or(|actual| actual == &context.target)
        && context.stdio.as_ref().is_none_or(|stdio| {
            matches_receipt(graph, &stdio.entry) && matches_receipt(graph, &stdio.declarations)
        })
        && context
            .allocator
            .iter()
            .all(|source| matches_receipt(graph, &source.entry))
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum HeaderAvailability {
    Complete,
    Ready,
}
fn bind_stdio_headers(
    graph: &ModuleGraph,
    types: &TypeRegistry,
    declarations: &ScopedDeclarations<'_>,
    context: Option<&FileAbiBindingContext>,
    target: Option<&BuildTarget>,
    availability: HeaderAvailability,
) -> Result<HashMap<ProcedureId, FileAbiProcedure>, LocatedDiagnostic> {
    let Some(context) = context else {
        return Ok(HashMap::new());
    };
    let file = context
        .stdio
        .as_ref()
        .map(|stdio| stdio.declarations.file)
        .or_else(|| context.allocator.first().map(|source| source.entry.file));
    let site = file
        .and_then(|file| graph.locate(file, Span::default()))
        .unwrap_or_else(|| {
            graph
                .locate(graph.module(graph.root()).unwrap().entry(), Span::default())
                .unwrap()
        });
    let error = |message: String| graph.diagnostic(site, message);
    if !matches_context(graph, context, target) {
        return Err(error(
            "stdio source receipt differs from this graph or selected target".into(),
        ));
    }
    let Some(stdio) = &context.stdio else {
        return Ok(HashMap::new());
    };
    let selected: Vec<_> = graph
        .declarations()
        .iter()
        .filter(|declaration| declaration.file() == stdio.declarations.file)
        .collect();
    let file_declaration = selected
        .iter()
        .find(|declaration| graph.symbols().name(declaration.name()) == "FILE")
        .ok_or_else(|| error("selected stdio source has no original FILE declaration".into()))?;
    let file = declarations
        .nominals
        .declarations
        .get(&file_declaration.id())
        .copied();
    let Some(file) = file else {
        if availability == HeaderAvailability::Ready {
            return Ok(HashMap::new());
        }
        return Err(error(
            "selected stdio source has no resolved nominal FILE".into(),
        ));
    };
    let mut candidates = Vec::new();
    let mut library = None;
    for declaration in selected {
        let FileDeclarationKind::ProcedurePrototype(prototype) = &declaration.syntax().kind else {
            continue;
        };
        let operation = match graph.symbols().name(prototype.name) {
            "fopen" => FileAbiOperation::Open,
            "fread" => FileAbiOperation::Read,
            "fwrite" => FileAbiOperation::Write,
            "fseek" => FileAbiOperation::Seek,
            "ftello" | "ftello64" => FileAbiOperation::Tell,
            "feof" => FileAbiOperation::Eof,
            "fclose" => FileAbiOperation::Close,
            _ => continue,
        };
        let Some(signature) = declarations.signatures.get(&declaration.id()) else {
            if availability == HeaderAvailability::Ready {
                continue;
            }
            return Err(error(
                "selected stdio declaration has no checked signature".into(),
            ));
        };
        let origin = foreign_libraries::origin(graph, declaration.file(), prototype)?;
        let PrototypeOrigin::Foreign {
            library: Some(actual),
            ..
        } = &origin
        else {
            return Err(error(
                "selected stdio declaration has no canonical library".into(),
            ));
        };
        let jai_ir::ForeignLibraryId::File(library_id) = actual.id else {
            return Err(error(
                "stdio library must be a defining file declaration".into(),
            ));
        };
        if graph
            .declaration(library_id)
            .is_none_or(|decl| decl.file() != stdio.declarations.file)
        {
            return Err(error(
                "stdio library is outside the selected source receipt".into(),
            ));
        }
        if library.as_ref().is_some_and(|previous| previous != actual) {
            return Err(error(
                "selected stdio declarations use different libraries".into(),
            ));
        }
        library = Some(actual.clone());
        candidates.push((
            operation,
            ProcedurePrototype {
                id: signature.id,
                signature: signature.ty,
                origin,
            },
        ));
    }
    let Some(library) = library else {
        if availability == HeaderAvailability::Ready {
            return Ok(HashMap::new());
        }
        return Err(error(
            "selected stdio source has no supported foreign declarations".into(),
        ));
    };
    let authority = match StdioAuthority::from_verified_source(
        library,
        file,
        candidates
            .iter()
            .map(|(operation, prototype)| (prototype.id, *operation)),
        types,
    ) {
        Ok(authority) => authority,
        Err(jai_vm::file_abi::FileAbiError::Type(jai_types::TypeError::Incomplete(_)))
            if availability == HeaderAvailability::Ready =>
        {
            return Ok(HashMap::new());
        }
        Err(failure) => return Err(error(failure.to_string())),
    };
    candidates
        .iter()
        .map(|(_, prototype)| {
            authority
                .bind(prototype, types)
                .map(|binding| (prototype.id, binding))
                .map_err(|failure| error(failure.to_string()))
        })
        .collect()
}

pub(super) fn bind(
    graph: &ModuleGraph,
    types: &TypeRegistry,
    declarations: &ScopedDeclarations<'_>,
    context: Option<&FileAbiBindingContext>,
    target: Option<&BuildTarget>,
) -> Result<HashMap<ProcedureId, FileAbiProcedure>, LocatedDiagnostic> {
    bind_stdio_headers(
        graph,
        types,
        declarations,
        context,
        target,
        HeaderAvailability::Complete,
    )
}
/// Bind only already checked original headers while source preparation is pending.
pub(super) fn bind_ready(
    graph: &ModuleGraph,
    types: &TypeRegistry,
    declarations: &ScopedDeclarations<'_>,
    context: Option<&FileAbiBindingContext>,
    target: Option<&BuildTarget>,
) -> Result<HashMap<ProcedureId, FileAbiProcedure>, LocatedDiagnostic> {
    bind_stdio_headers(
        graph,
        types,
        declarations,
        context,
        target,
        HeaderAvailability::Ready,
    )
}
#[cfg(test)]
#[path = "file_abi_bindings/tests.rs"]
mod tests;
