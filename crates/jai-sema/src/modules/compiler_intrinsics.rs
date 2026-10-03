//! Bind explicitly marked source declarations once, before VM execution.
use super::*;
mod code_arguments;
pub(crate) mod schema;
pub(super) use code_arguments::lower_code_signature;
pub(crate) use code_arguments::{SourceCompilerSignature, source_candidate};
use jai_source::ModuleId;
use jai_types::{CallingConvention, ContextMode, TypeView};
use jai_vm::{CompilerIntrinsic, CompilerProcedure, WorkspaceId};
use schema::{build_options_projection, validate_enum, validate_location};
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompilerModuleOrigin {
    Application,
    Compiler,
    Preload,
    RuntimeSupport,
}

/// Module identities belong to this graph, never to a name at a call site.
#[derive(Clone, Debug, Default)]
pub struct CompilerModuleOrigins {
    modules: HashMap<ModuleId, CompilerModuleOrigin>,
}
impl CompilerModuleOrigins {
    pub fn register(&mut self, module: ModuleId, origin: CompilerModuleOrigin) {
        self.modules.insert(module, origin);
    }
    pub fn origin(&self, module: ModuleId) -> Option<CompilerModuleOrigin> {
        self.modules.get(&module).copied()
    }
    /// Match the loader's selected, canonical source entry against configured roots.
    pub fn from_graph(graph: &ModuleGraph, import_dirs: &[PathBuf]) -> Self {
        let mut origins = Self::default();
        origins.register(graph.root(), CompilerModuleOrigin::Application);
        for module in graph.modules() {
            let Some(file) = graph.file(module.entry()) else {
                continue;
            };
            let Some(source) = graph.sources().get(file.source()) else {
                continue;
            };
            for directory in import_dirs {
                for (name, origin) in [
                    ("Compiler", CompilerModuleOrigin::Compiler),
                    ("Preload", CompilerModuleOrigin::Preload),
                ] {
                    let candidates = [
                        directory.join(format!("{name}.jai")),
                        directory.join(name).join("module.jai"),
                    ];
                    if candidates.iter().any(|candidate| {
                        candidate
                            .canonicalize()
                            .is_ok_and(|path| path == source.path())
                    }) {
                        origins.register(module.id(), origin);
                    }
                }
            }
        }
        if let Some(module) = graph.prelude() {
            origins.register(module, CompilerModuleOrigin::Preload);
        }
        if let Some(module) = graph.runtime_support() {
            origins.register(module, CompilerModuleOrigin::RuntimeSupport);
        }
        origins
    }
}

#[derive(Clone, Debug)]
pub struct CompilerBindingContext {
    pub current_workspace: WorkspaceId,
    pub origins: CompilerModuleOrigins,
}
impl CompilerBindingContext {
    pub fn from_graph(
        graph: &ModuleGraph,
        import_dirs: &[PathBuf],
        current_workspace: WorkspaceId,
    ) -> Self {
        Self {
            current_workspace,
            origins: CompilerModuleOrigins::from_graph(graph, import_dirs),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SourceIntrinsic {
    BeginIntercept,
    EndIntercept,
    WaitForMessage,
    VersionInfo,
    RuntimeInfo,
    WorkspaceName,
    DestroyWorkspace,
    SetWorkspaceStatus,
    CreateWorkspace,
    CurrentWorkspace,
    AddString,
    AddFile,
    Report,
    SetBuildOptions,
    GetBuildOptions,
    WriteString,
    WriteStrings,
    DebugBreak,
}
impl SourceIntrinsic {
    fn parse(name: &str) -> Option<Self> {
        Some(match name {
            "compiler_begin_intercept" => Self::BeginIntercept,
            "compiler_end_intercept" => Self::EndIntercept,
            "compiler_wait_for_message" => Self::WaitForMessage,
            "compiler_get_version_info" => Self::VersionInfo,
            "get_runtime_info" => Self::RuntimeInfo,
            "get_name" => Self::WorkspaceName,
            "compiler_destroy_workspace" => Self::DestroyWorkspace,
            "compiler_set_workspace_status" => Self::SetWorkspaceStatus,
            "compiler_create_workspace" => Self::CreateWorkspace,
            "get_current_workspace" => Self::CurrentWorkspace,
            "add_build_string" => Self::AddString,
            "add_build_file" => Self::AddFile,
            "compiler_report" => Self::Report,
            "set_build_options" => Self::SetBuildOptions,
            "get_build_options" => Self::GetBuildOptions,
            "write_string" => Self::WriteString,
            "write_strings" => Self::WriteStrings,
            "compile_time_debug_break" => Self::DebugBreak,
            _ => return None,
        })
    }
}

pub(super) fn bind(
    graph: &ModuleGraph,
    types: &TypeRegistry,
    declarations: &ScopedDeclarations<'_>,
    context: Option<&CompilerBindingContext>,
    meta: &crate::reflection::MetaContext,
) -> Result<HashMap<ProcedureId, CompilerProcedure>, LocatedDiagnostic> {
    bind_available(graph, types, declarations, context, meta, false)
}

pub(super) fn bind_ready(
    graph: &ModuleGraph,
    types: &TypeRegistry,
    declarations: &ScopedDeclarations<'_>,
    context: Option<&CompilerBindingContext>,
    meta: &crate::reflection::MetaContext,
) -> Result<HashMap<ProcedureId, CompilerProcedure>, LocatedDiagnostic> {
    bind_available(graph, types, declarations, context, meta, true)
}

fn bind_available(
    graph: &ModuleGraph,
    types: &TypeRegistry,
    declarations: &ScopedDeclarations<'_>,
    context: Option<&CompilerBindingContext>,
    meta: &crate::reflection::MetaContext,
    partial: bool,
) -> Result<HashMap<ProcedureId, CompilerProcedure>, LocatedDiagnostic> {
    let mut bindings = HashMap::new();
    for declaration in graph.declarations() {
        let (declared_name, parameters, mark) = match &declaration.syntax().kind {
            FileDeclarationKind::ProcedurePrototype(prototype) => match &prototype.binding {
                syntax::PrototypeBinding::Compiler(mark) => {
                    (prototype.name, &prototype.parameters, mark)
                }
                _ => continue,
            },
            FileDeclarationKind::Procedure(procedure) => {
                let Some(mark) = &procedure.compiler else {
                    continue;
                };
                (procedure.name, &procedure.parameters, mark)
            }
            _ => continue,
        };
        if partial && !declarations.signatures.contains_key(&declaration.id()) {
            continue;
        }
        let name = mark
            .tag
            .as_deref()
            .unwrap_or_else(|| graph.symbols().name(declared_name));
        let source = SourceIntrinsic::parse(name).ok_or_else(|| {
            graph.diagnostic(
                declaration.location(),
                format!("unsupported #compiler intrinsic `{name}`"),
            )
        })?;
        let signature = declarations
            .signatures
            .get(&declaration.id())
            .ok_or_else(|| {
                graph.diagnostic(
                    declaration.location(),
                    "#compiler declaration has no checked signature",
                )
            })?;
        let checked = types
            .procedure_definition(signature.ty)
            .map_err(|error| graph.diagnostic(declaration.location(), error.to_string()))?;
        let expected_context = if matches!(
            source,
            SourceIntrinsic::WriteString
                | SourceIntrinsic::WriteStrings
                | SourceIntrinsic::DebugBreak
        ) {
            ContextMode::None
        } else {
            ContextMode::Implicit
        };
        if checked.convention != CallingConvention::Jai
            || checked.context != expected_context
            || parameters.iter().any(|parameter| {
                parameter.using
                    || parameter.baking != syntax::ParameterBaking::None
                    || parameter.evaluation != syntax::ParameterEvaluation::Evaluate
                    || (parameter.variadic && source != SourceIntrinsic::WriteStrings)
            })
        {
            return Err(graph.diagnostic(
                declaration.location(),
                "#compiler declaration calling convention, context, or parameter binding differs from its source catalog",
            ));
        }
        let Some(context) = context else {
            return Err(graph.diagnostic(
                declaration.location(),
                "#compiler declaration requires a compiler workspace binding context",
            ));
        };
        let module = graph
            .file(declaration.file())
            .expect("declaration belongs to graph")
            .module();
        let Some(origin) = context.origins.origin(module) else {
            return Err(graph.diagnostic(declaration.location(), "#compiler declaration does not belong to the application or a selected Compiler/Preload module"));
        };
        if origin == CompilerModuleOrigin::Preload && source != SourceIntrinsic::CurrentWorkspace {
            return Err(graph.diagnostic(
                declaration.location(),
                "this #compiler intrinsic is not part of the Preload API",
            ));
        }
        if meta.compiler_source_signatures.contains_key(&signature.id)
            && source != SourceIntrinsic::AddString
        {
            return Err(graph.diagnostic(
                declaration.location(),
                "compile-only Code parameter is supported only by add_build_string",
            ));
        }
        if origin == CompilerModuleOrigin::RuntimeSupport
            && !matches!(
                source,
                SourceIntrinsic::WriteString
                    | SourceIntrinsic::WriteStrings
                    | SourceIntrinsic::DebugBreak
            )
        {
            return Err(graph.diagnostic(
                declaration.location(),
                "this #compiler intrinsic is not part of the Runtime_Support API",
            ));
        }
        let intrinsic =
            bind_source_signature(source, checked, types, graph, declarations, context, meta)
                .map_err(|message| {
                    graph.diagnostic(
                        declaration.location(),
                        format!("#compiler `{name}`: {message}"),
                    )
                })?;
        bindings.insert(
            signature.id,
            CompilerProcedure {
                signature: signature.ty,
                intrinsic,
            },
        );
    }
    Ok(bindings)
}

fn bind_source_signature(
    source: SourceIntrinsic,
    signature: &ProcedureType,
    types: &TypeRegistry,
    graph: &ModuleGraph,
    declarations: &ScopedDeclarations<'_>,
    context: &CompilerBindingContext,
    meta: &crate::reflection::MetaContext,
) -> Result<CompilerIntrinsic, String> {
    let string = |ty| matches!(types.kind(ty), Ok(TypeKind::String));
    let workspace = |ty| matches!(types.kind(ty), Ok(TypeKind::Integer(IntegerType::S64)));
    match (
        source,
        signature.parameters.as_ref(),
        signature.results.as_ref(),
    ) {
        (SourceIntrinsic::BeginIntercept, [id, flags], []) if workspace(*id) => {
            schema::messages::intercept_flags(*flags, graph, declarations, types, context, meta)?;
            return Ok(CompilerIntrinsic::SourceBeginIntercept {
                current_workspace: context.current_workspace,
            });
        }
        (SourceIntrinsic::WaitForMessage, [], [pointer]) => {
            let TypeKind::Pointer(message) =
                types.kind(*pointer).map_err(|error| error.to_string())?
            else {
                return Err("requires () -> *Message".into());
            };
            return Ok(CompilerIntrinsic::SourceWaitForMessage {
                schema: schema::messages::projection(
                    *message,
                    graph,
                    declarations,
                    types,
                    context,
                    meta,
                )?,
            });
        }
        (SourceIntrinsic::VersionInfo, [pointer], [result]) if string(*result) => {
            let TypeKind::Pointer(record) = types.kind(*pointer).map_err(|e| e.to_string())? else {
                return Err("requires (version_info_return: *Version_Info) -> string".into());
            };
            schema::validate_version(*record, graph, declarations, types, context)?;
            return Ok(CompilerIntrinsic::SourceVersionInfo { record: *record });
        }
        (SourceIntrinsic::RuntimeInfo, [id], [result]) if workspace(*id) => {
            return Ok(CompilerIntrinsic::SourceRuntimeInfo {
                current_workspace: context.current_workspace,
                schema: schema::runtime_info::projection(
                    *result,
                    graph,
                    declarations,
                    types,
                    context,
                    meta,
                )?,
            });
        }
        (SourceIntrinsic::SetWorkspaceStatus, [status, id], []) if workspace(*id) => {
            validate_enum(
                *status,
                "Workspace_Status",
                &["OK", "FAILED"],
                graph,
                declarations,
                types,
                context,
            )?;
            return Ok(CompilerIntrinsic::SourceSetWorkspaceStatus {
                current_workspace: context.current_workspace,
            });
        }
        (SourceIntrinsic::AddString | SourceIntrinsic::AddFile, [text, id, location], [])
            if string(*text) && workspace(*id) =>
        {
            validate_location(*location, graph, declarations, types, context)?;
            return Ok(if source == SourceIntrinsic::AddString {
                CompilerIntrinsic::SourceAddStringAt {
                    current_workspace: context.current_workspace,
                }
            } else {
                CompilerIntrinsic::SourceAddFileAt {
                    current_workspace: context.current_workspace,
                }
            });
        }
        (SourceIntrinsic::Report, [text, location, mode], []) if string(*text) => {
            validate_location(*location, graph, declarations, types, context)?;
            validate_enum(
                *mode,
                "Report",
                &["ERROR", "ERROR_CONTINUABLE", "WARNING", "INFO"],
                graph,
                declarations,
                types,
                context,
            )?;
            return Ok(CompilerIntrinsic::SourceReportAt);
        }
        (SourceIntrinsic::SetBuildOptions, [options, id, location], []) if workspace(*id) => {
            validate_location(*location, graph, declarations, types, context)?;
            let projection =
                build_options_projection(*options, graph, declarations, types, context, meta)?;
            return Ok(CompilerIntrinsic::SourceSetBuildOptionsAt {
                current_workspace: context.current_workspace,
                projection,
            });
        }
        (SourceIntrinsic::SetBuildOptions, [options, id], []) if workspace(*id) => {
            let projection =
                build_options_projection(*options, graph, declarations, types, context, meta)?;
            return Ok(CompilerIntrinsic::SourceSetBuildOptions {
                current_workspace: context.current_workspace,
                projection,
            });
        }
        (SourceIntrinsic::GetBuildOptions, [id], [options]) if workspace(*id) => {
            let projection =
                build_options_projection(*options, graph, declarations, types, context, meta)?;
            return Ok(CompilerIntrinsic::SourceGetBuildOptions {
                current_workspace: context.current_workspace,
                projection,
                result: *options,
            });
        }
        _ => {}
    }
    bind_signature(source, signature, types, context.current_workspace).map_err(str::to_owned)
}

fn bind_signature(
    source: SourceIntrinsic,
    signature: &ProcedureType,
    types: &dyn TypeView,
    current_workspace: WorkspaceId,
) -> Result<CompilerIntrinsic, &'static str> {
    let is_string = |ty| matches!(types.kind(ty), Ok(TypeKind::String));
    let is_workspace = |ty| matches!(types.kind(ty), Ok(TypeKind::Integer(IntegerType::S64)));
    let parameters = signature.parameters.as_ref();
    let results = signature.results.as_ref();
    match source {
        SourceIntrinsic::EndIntercept
            if matches!(parameters, [id] if is_workspace(*id)) && results.is_empty() =>
        {
            Ok(CompilerIntrinsic::SourceEndIntercept { current_workspace })
        }
        SourceIntrinsic::BeginIntercept => Err(
            "requires (w: s64, flags: Intercept_Flags) -> void with the exact source u32 flags enum",
        ),
        SourceIntrinsic::EndIntercept => Err("requires (w: s64) -> void"),
        SourceIntrinsic::WaitForMessage => {
            Err("requires () -> *Message with the exact source event schemas")
        }
        SourceIntrinsic::WorkspaceName
            if matches!(parameters, [workspace] if is_workspace(*workspace))
                && matches!(results, [name] if is_string(*name)) =>
        {
            Ok(CompilerIntrinsic::SourceGetWorkspaceName { current_workspace })
        }
        SourceIntrinsic::DestroyWorkspace
            if matches!(parameters, [workspace] if is_workspace(*workspace))
                && results.is_empty() =>
        {
            Ok(CompilerIntrinsic::SourceDestroyWorkspace { current_workspace })
        }
        SourceIntrinsic::WriteString
            if matches!(parameters, [text, error] if is_string(*text) && matches!(types.kind(*error), Ok(TypeKind::Bool)))
                && results.is_empty()
                && signature.variadic == jai_types::Variadic::None =>
        {
            Ok(CompilerIntrinsic::SourceWriteString)
        }
        SourceIntrinsic::WriteStrings
            if matches!(parameters, [pack, error] if matches!(types.kind(*pack), Ok(TypeKind::Slice(element)) if is_string(*element)) && matches!(types.kind(*error), Ok(TypeKind::Bool)))
                && results.is_empty()
                && matches!(signature.variadic, jai_types::Variadic::Jai { parameter: 0, element } if is_string(element)) =>
        {
            Ok(CompilerIntrinsic::SourceWriteStrings)
        }
        SourceIntrinsic::DebugBreak
            if parameters.is_empty()
                && results.is_empty()
                && signature.variadic == jai_types::Variadic::None =>
        {
            Ok(CompilerIntrinsic::SourceDebugBreak)
        }
        SourceIntrinsic::WriteString
        | SourceIntrinsic::WriteStrings
        | SourceIntrinsic::DebugBreak => {
            Err("requires the exact Runtime_Support compiler signature")
        }
        SourceIntrinsic::CreateWorkspace
            if matches!(parameters, [name] if is_string(*name))
                && matches!(results, [workspace] if is_workspace(*workspace)) =>
        {
            Ok(CompilerIntrinsic::SourceCreateWorkspace)
        }
        SourceIntrinsic::CurrentWorkspace
            if parameters.is_empty()
                && matches!(results, [workspace] if is_workspace(*workspace)) =>
        {
            Ok(CompilerIntrinsic::SourceCurrentWorkspace { current_workspace })
        }
        SourceIntrinsic::AddString | SourceIntrinsic::AddFile
            if matches!(parameters, [text, workspace] if is_string(*text) && is_workspace(*workspace))
                && results.is_empty() =>
        {
            Ok(if source == SourceIntrinsic::AddString {
                CompilerIntrinsic::SourceAddString { current_workspace }
            } else {
                CompilerIntrinsic::SourceAddFile { current_workspace }
            })
        }
        SourceIntrinsic::AddString => Err(
            "requires (data: string, w: s64) -> void or the checked Code/location source signature",
        ),
        SourceIntrinsic::AddFile => {
            Err("requires (filename: string, w: s64[, loc: Source_Code_Location]) -> void")
        }
        SourceIntrinsic::SetBuildOptions => Err(
            "requires (options: Build_Options, w: s64[, loc: Source_Code_Location]) -> void with a supported checked options schema",
        ),
        SourceIntrinsic::GetBuildOptions => {
            Err("requires (w: s64) -> Build_Options with a supported checked options schema")
        }
        SourceIntrinsic::Report
            if matches!(parameters, [text] if is_string(*text)) && results.is_empty() =>
        {
            Ok(CompilerIntrinsic::SourceReport {
                level: jai_vm::MessageLevel::Error,
            })
        }
        SourceIntrinsic::Report => Err(
            "requires (message: string) -> void or (message: string, loc: Source_Code_Location, mode: Report) -> void",
        ),
        SourceIntrinsic::SetWorkspaceStatus => Err(
            "requires (status: Workspace_Status, w: s64) -> void with the exact source u8 OK/FAILED enum",
        ),
        SourceIntrinsic::DestroyWorkspace => Err("requires (w: s64) -> void"),
        SourceIntrinsic::WorkspaceName => Err("requires (w: s64) -> string"),
        SourceIntrinsic::VersionInfo => Err(
            "requires (version_info_return: *Version_Info) -> string with the exact source s32 major/minor/micro struct",
        ),
        SourceIntrinsic::RuntimeInfo => Err(
            "requires (w: s64) -> Runtime_Info with the selected source runtime/global-data schemas and adopted Type_Info identity",
        ),
        SourceIntrinsic::CreateWorkspace => Err("requires (name: string) -> s64"),
        SourceIntrinsic::CurrentWorkspace => Err("requires () -> s64"),
    }
}

#[cfg(test)]
mod tests;
