//! Immutable configured-source receipts for virtual POSIX control declarations.
//! This module grants no native loading or executable registration.
use super::*;
use jai_types::{Architecture, BuildTarget, ByteOrder, LayoutPolicy, OperatingSystem};
use jai_vm::process_abi::{
    ProcessAbiNominals, ProcessAbiOperation, ProcessAbiProcedure, ProcessAuthority,
    ProcessSocketTypes,
};
use std::{path::PathBuf, sync::Arc};

#[derive(Clone, Debug)]
struct SourceReceipt {
    file: FileInstanceId,
    path: PathBuf,
    text: Arc<str>,
}
#[derive(Clone, Debug)]
struct SocketReceipts {
    entry: SourceReceipt,
    generated: SourceReceipt,
}
/// Explicit trusted embedding selection, retained against the same graph and target.
/// Source strings cannot create this context or select another canonical library.
#[derive(Clone, Debug)]
pub struct ProcessAbiBindingContext {
    unit: jai_source::UnitId,
    target: BuildTarget,
    entry: SourceReceipt,
    base: SourceReceipt,
    stdio: Option<SourceReceipt>,
    socket: Option<SocketReceipts>,
}
impl ProcessAbiBindingContext {
    pub fn from_graph(
        graph: &ModuleGraph,
        import_dirs: &[PathBuf],
        target: BuildTarget,
    ) -> Option<Self> {
        if target.layout != LayoutPolicy::lp64() || target.byte_order != ByteOrder::Little {
            return None;
        }
        let (bindings, generated) = match (&target.operating_system, &target.architecture) {
            (OperatingSystem::MacOS, Architecture::Arm64) => ("macos/arm64", "generated_macos.jai"),
            (OperatingSystem::MacOS, Architecture::X86_64) => ("macos/x64", "generated_macos.jai"),
            (OperatingSystem::Linux, Architecture::Arm64 | Architecture::X86_64) => {
                ("linux", "generated_linux.jai")
            }
            _ => return None,
        };
        for directory in import_dirs {
            let Some(entry) = selected_entry(graph, directory.join("POSIX/module.jai")) else {
                continue;
            };
            let Some(base) = selected_file(
                graph,
                &entry,
                directory
                    .join("POSIX/bindings")
                    .join(bindings)
                    .join("base.jai"),
            ) else {
                continue;
            };
            let stdio = selected_file(
                graph,
                &entry,
                directory
                    .join("POSIX/bindings")
                    .join(bindings)
                    .join("stdio.jai"),
            );
            let socket =
                selected_entry(graph, directory.join("Socket/module.jai")).and_then(|entry| {
                    selected_file(graph, &entry, directory.join("Socket").join(generated)).map(
                        |generated| SocketReceipts {
                            entry,
                            generated,
                        },
                    )
                });
            return Some(Self {
                unit: graph.unit(),
                target,
                entry,
                base,
                stdio,
                socket,
            });
        }
        None
    }
    pub fn target(&self) -> &BuildTarget {
        &self.target
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
fn selected_entry(graph: &ModuleGraph, path: PathBuf) -> Option<SourceReceipt> {
    let path = path.canonicalize().ok()?;
    graph
        .modules()
        .iter()
        .filter_map(|module| receipt(graph, module.entry()))
        .find(|source| source.path == path)
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
        .find(|source| source.path == path)
}
fn matches_receipt(graph: &ModuleGraph, source: &SourceReceipt) -> bool {
    receipt(graph, source.file)
        .is_some_and(|actual| actual.path == source.path && Arc::ptr_eq(&actual.text, &source.text))
}
fn nominal(
    graph: &ModuleGraph,
    declarations: &ScopedDeclarations<'_>,
    source: &SourceReceipt,
    name: &str,
) -> Option<TypeId> {
    graph
        .declarations()
        .iter()
        .find(|decl| decl.file() == source.file && graph.symbols().name(decl.name()) == name)
        .and_then(|decl| declarations.nominals.declarations.get(&decl.id()))
        .copied()
}
fn operation(name: &str) -> Option<ProcessAbiOperation> {
    Some(match name {
        "fcntl" => ProcessAbiOperation::Fcntl,
        "fork" => ProcessAbiOperation::Fork,
        "pipe" => ProcessAbiOperation::Pipe,
        "close" => ProcessAbiOperation::Close,
        "read" => ProcessAbiOperation::Read,
        "write" => ProcessAbiOperation::Write,
        "dup2" => ProcessAbiOperation::Dup2,
        "waitpid" => ProcessAbiOperation::WaitPid,
        "execvp" => ProcessAbiOperation::ExecVp,
        "_exit" => ProcessAbiOperation::Exit,
        "__errno_location" => ProcessAbiOperation::ErrnoLocation,
        "getpid" => ProcessAbiOperation::GetPid,
        "getppid" => ProcessAbiOperation::GetParentPid,
        "socketpair" => ProcessAbiOperation::SocketPair,
        "sendmsg" => ProcessAbiOperation::SendMsg,
        "recvmsg" => ProcessAbiOperation::RecvMsg,
        "shutdown" => ProcessAbiOperation::Shutdown,
        _ => return None,
    })
}
pub(super) fn bind(
    graph: &ModuleGraph,
    types: &TypeRegistry,
    declarations: &ScopedDeclarations<'_>,
    context: Option<&ProcessAbiBindingContext>,
    target: Option<&BuildTarget>,
) -> Result<HashMap<ProcedureId, ProcessAbiProcedure>, LocatedDiagnostic> {
    bind_available(graph, types, declarations, context, target, false)
}

pub(super) fn bind_ready(
    graph: &ModuleGraph,
    types: &TypeRegistry,
    declarations: &ScopedDeclarations<'_>,
    context: Option<&ProcessAbiBindingContext>,
    target: Option<&BuildTarget>,
) -> Result<HashMap<ProcedureId, ProcessAbiProcedure>, LocatedDiagnostic> {
    bind_available(graph, types, declarations, context, target, true)
}

fn bind_available(
    graph: &ModuleGraph,
    types: &TypeRegistry,
    declarations: &ScopedDeclarations<'_>,
    context: Option<&ProcessAbiBindingContext>,
    target: Option<&BuildTarget>,
    partial: bool,
) -> Result<HashMap<ProcedureId, ProcessAbiProcedure>, LocatedDiagnostic> {
    let Some(context) = context else {
        return Ok(HashMap::new());
    };
    let site = graph.locate(context.entry.file, Span::default()).unwrap();
    let error = |message: String| graph.diagnostic(site, message);
    let mut selected = vec![&context.entry, &context.base];
    selected.extend(context.stdio.iter());
    if let Some(socket) = &context.socket {
        selected.extend([&socket.entry, &socket.generated]);
    }
    if graph.unit() != context.unit
        || target != Some(&context.target)
        || selected
            .iter()
            .any(|source| !matches_receipt(graph, source))
    {
        return Err(error(
            "process source receipt differs from this graph or selected target".into(),
        ));
    }
    if partial {
        // A source reservation is not yet a checked nominal. Preserve the
        // original receipt and withhold capability until that fact publishes.
        let pending = |source: &SourceReceipt, name: &str| {
            graph.declarations().iter().any(|declaration| {
                declaration.file() == source.file
                    && graph.symbols().name(declaration.name()) == name
                    && !declarations
                        .nominals
                        .declarations
                        .contains_key(&declaration.id())
            })
        };
        if pending(&context.entry, "OS_Error_Code")
            || context.socket.as_ref().is_some_and(|socket| {
                ["SOCK", "IPPROTO", "MSG", "SHUT", "msghdr", "cmsghdr"]
                    .into_iter()
                    .any(|name| pending(&socket.generated, name))
                    || pending(&context.base, "iovec")
            })
        {
            return Ok(HashMap::new());
        }
    }
    let error_code = nominal(graph, declarations, &context.entry, "OS_Error_Code")
        .ok_or_else(|| error("selected POSIX entry has no checked nominal OS_Error_Code".into()))?;
    let socket = context
        .socket
        .as_ref()
        .map(|socket| {
            let find = |name| {
                nominal(graph, declarations, &socket.generated, name).ok_or_else(|| {
                    error(format!(
                        "selected Socket source has no checked nominal {name}"
                    ))
                })
            };
            Ok::<_, LocatedDiagnostic>(ProcessSocketTypes {
                socket_kind: find("SOCK")?,
                protocol: find("IPPROTO")?,
                message_flags: find("MSG")?,
                shutdown_kind: find("SHUT")?,
                message_header: find("msghdr")?,
                control_header: find("cmsghdr")?,
                io_vector: nominal(graph, declarations, &context.base, "iovec").ok_or_else(
                    || error("selected POSIX base has no checked nominal iovec".into()),
                )?,
            })
        })
        .transpose()?;
    let nominals = ProcessAbiNominals {
        error_code,
        socket,
    };
    let mut headers = vec![&context.base];
    headers.extend(context.stdio.iter());
    if let Some(socket) = &context.socket {
        headers.push(&socket.generated);
    }
    let mut bindings = HashMap::new();
    for source in headers {
        for declaration in graph
            .declarations()
            .iter()
            .filter(|decl| decl.file() == source.file)
        {
            let FileDeclarationKind::ProcedurePrototype(prototype) = &declaration.syntax().kind
            else {
                continue;
            };
            let Some(operation) = operation(graph.symbols().name(prototype.name)) else {
                continue;
            };
            if partial && !declarations.signatures.contains_key(&declaration.id()) {
                continue;
            }
            let signature = declarations
                .signatures
                .get(&declaration.id())
                .ok_or_else(|| {
                    error("selected process declaration has no checked signature".into())
                })?;
            let origin = foreign_libraries::origin(graph, declaration.file(), prototype)?;
            let PrototypeOrigin::Foreign {
                library: Some(library),
                ..
            } = &origin
            else {
                return Err(error(
                    "selected process declaration has no canonical foreign library".into(),
                ));
            };
            let jai_ir::ForeignLibraryId::File(id) = library.id else {
                return Err(error(
                    "selected process library is not a defining file declaration".into(),
                ));
            };
            if graph
                .declaration(id)
                .is_none_or(|decl| decl.file() != source.file)
            {
                return Err(error(
                    "process library lies outside its selected source receipt".into(),
                ));
            }
            let authority = match ProcessAuthority::from_verified_source(
                context.target.clone(),
                library.clone(),
                nominals,
                [(signature.id, operation, signature.ty)],
                types,
            ) {
                Ok(authority) => authority,
                Err(jai_vm::process_abi::ProcessAbiError::Type(
                    jai_types::TypeError::Incomplete(_),
                )) if partial => continue,
                Err(failure) => return Err(error(failure.to_string())),
            };
            let prototype = ProcedurePrototype {
                id: signature.id,
                signature: signature.ty,
                origin,
            };
            let binding = authority
                .bind(&prototype, &context.target, types)
                .map_err(|failure| error(failure.to_string()))?;
            if bindings.insert(prototype.id, binding).is_some() {
                return Err(error(
                    "duplicate process capability procedure identity".into(),
                ));
            }
        }
    }
    Ok(bindings)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let root = std::env::temp_dir().join(format!(
                "jai-process-receipt-{}-{:?}",
                std::process::id(),
                jai_vm::host_effects::HostRequestKey::allocate()
            ));
            fs::create_dir_all(root.join("modules/POSIX/bindings/macos/arm64")).unwrap();
            fs::write(
                root.join("main.jai"),
                "#import \"POSIX\"; main::()->s32 #no_context{return 0;}\n",
            )
            .unwrap();
            fs::write(
                root.join("modules/POSIX/module.jai"),
                "OS_Error_Code :: #type,isa s32; #load \"bindings/macos/arm64/base.jai\";\n",
            )
            .unwrap();
            fs::write(
                root.join("modules/POSIX/bindings/macos/arm64/base.jai"),
                "fork::()->s32 #foreign libc; libc::#system_library \"libc\";\n",
            )
            .unwrap();
            Self(root)
        }
        fn graph(&self) -> ModuleGraph {
            ModuleGraph::load(
                &self.0.join("main.jai"),
                jai_modules::GraphOptions {
                    import_dirs: vec![self.0.join("modules")],
                },
            )
            .unwrap()
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    fn target() -> BuildTarget {
        BuildTarget {
            operating_system: OperatingSystem::MacOS,
            architecture: Architecture::Arm64,
            layout: LayoutPolicy::lp64(),
            byte_order: ByteOrder::Little,
        }
    }
    #[test]
    fn configured_receipts_match_only_the_original_loaded_graph() {
        let fixture = Fixture::new();
        let graph = fixture.graph();
        let context =
            ProcessAbiBindingContext::from_graph(&graph, &[fixture.0.join("modules")], target())
                .unwrap();
        assert!(matches_receipt(&graph, &context.entry));
        assert!(matches_receipt(&graph, &context.base));
        assert!(context.socket.is_none());
        let module = graph
            .modules()
            .iter()
            .find(|module| module.entry() == context.entry.file)
            .unwrap();
        assert!(module.files().contains(&context.base.file));
        let other = fixture.graph();
        assert!(!matches_receipt(&other, &context.entry));
        assert!(!matches_receipt(&other, &context.base));
    }
    #[test]
    fn configured_receipts_reject_unselected_target_and_import_roots() {
        let fixture = Fixture::new();
        let graph = fixture.graph();
        assert!(ProcessAbiBindingContext::from_graph(&graph, &[], target()).is_none());
        let mut unsupported = target();
        unsupported.byte_order = ByteOrder::Big;
        assert!(
            ProcessAbiBindingContext::from_graph(&graph, &[fixture.0.join("modules")], unsupported)
                .is_none()
        );
    }
    fn declarations<'graph>(
        graph: &'graph ModuleGraph,
        types: &mut TypeRegistry,
    ) -> ScopedDeclarations<'graph> {
        let callable_aliases = crate::polymorphism::integration::callable_aliases(graph);
        let source_procedures =
            procedure_headers::identities::reserve(graph, &callable_aliases).unwrap();
        let count = source_procedures.len();
        ScopedDeclarations {
            graph,
            context: None,
            values: HashMap::new(),
            signatures: HashMap::new(),
            callable_aliases,
            source_procedures,
            source_lookup: Default::default(),
            generics: std::cell::RefCell::new(
                crate::polymorphism::integration::GenericContext::new(count),
            ),
            nominals: Nominals::reserve(graph, types).unwrap(),
            defaults: HashMap::new(),
        }
    }

    #[test]
    fn ready_binding_withholds_capability_until_actual_source_header_finishes() {
        let fixture = Fixture::new();
        let graph = fixture.graph();
        let target = target();
        let context = ProcessAbiBindingContext::from_graph(
            &graph,
            &[fixture.0.join("modules")],
            target.clone(),
        )
        .unwrap();
        let mut types = TypeRegistry::new();
        let mut declarations = declarations(&graph, &mut types);
        assert!(
            bind_ready(&graph, &types, &declarations, Some(&context), Some(&target))
                .unwrap()
                .is_empty()
        );
        assert!(bind(&graph, &types, &declarations, Some(&context), Some(&target)).is_err());
        let mut constants = Constants::new(&graph);
        let mut meta = crate::reflection::MetaContext::default();
        declarations
            .nominals
            .define_aliases_with_specializations(
                &graph,
                &mut types,
                &mut meta.record_specializations,
                &mut |file, expression| constants.evaluate_lazy(file, expression),
            )
            .unwrap();
        procedure_headers::register(
            &graph,
            &mut types,
            &mut declarations,
            &mut constants,
            &mut meta,
            procedure_headers::HeaderPhase::TypesOnly,
        )
        .unwrap();
        let ready =
            bind_ready(&graph, &types, &declarations, Some(&context), Some(&target)).unwrap();
        let strict = bind(&graph, &types, &declarations, Some(&context), Some(&target)).unwrap();
        assert_eq!(ready.len(), 1);
        assert_eq!(strict.len(), 1);
        let source = graph
            .declarations()
            .iter()
            .find(|source| graph.symbols().name(source.name()) == "fork")
            .unwrap();
        let id = declarations.signatures[&source.id()].id;
        assert_eq!(ready[&id].procedure(), id);
        assert_eq!(
            ready[&id].signature(),
            declarations.signatures[&source.id()].ty
        );
        assert_eq!(ready[&id].operation(), ProcessAbiOperation::Fork);
    }

    #[test]
    fn ready_binding_validates_original_graph_receipt_before_skipping_absent_headers() {
        let fixture = Fixture::new();
        let graph = fixture.graph();
        let target = target();
        let context = ProcessAbiBindingContext::from_graph(
            &graph,
            &[fixture.0.join("modules")],
            target.clone(),
        )
        .unwrap();
        let other = fixture.graph();
        let mut types = TypeRegistry::new();
        let declarations = declarations(&other, &mut types);
        let error =
            bind_ready(&other, &types, &declarations, Some(&context), Some(&target)).unwrap_err();
        assert!(error.message.contains("source receipt differs"));
    }
}
