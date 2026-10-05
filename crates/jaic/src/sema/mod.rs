//! Semantic analysis.
//!
//! Resolution is demand-driven: declarations are entered into scopes without
//! being checked, and each entity, struct layout, procedure signature and body
//! is resolved the first time something needs it (recursively, with cycle
//! detection). Conditional top-level code (`#if`, `#insert`) is kept as
//! *pending* scope items that are expanded before a lookup can fall through to
//! an outer scope, so names they declare are never missed.
//!
//! Expression checking lowers directly to IR (`ir::Builder`); constants fold
//! into `Operand::Const`. Compile-time evaluation (`#run`, constant aggregates,
//! global initializers) lowers a thunk procedure and runs it in `interp`.
pub mod value;

mod asm;
mod bake;
mod calls;
pub mod code_export;
mod consteval;
mod convert;
mod decls;
mod driver;
mod expr;
pub mod ide;
mod lambda;
mod lower;
mod modify;
mod modules;
mod procs;
mod runtime_info;
mod scope;
mod stmt;
mod structs;
mod typeinfo;

use crate::ast;
use crate::intern::Sym;
use crate::ir;
use crate::source::{Diagnostic, FileId, SourceMap, Span};
use crate::types::{TypeId, Types};
pub use driver::ProgramSource;
pub use modules::{FileSystem, NativeFs, VirtualFs};
pub use scope::{EntityId, ScopeId};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::rc::Rc;
pub use value::{ModuleId, ProcId, Value};

pub type Result<T> = std::result::Result<T, Box<Diagnostic>>;
/// A named compile-time argument: a module parameter or a baked procedure argument.
pub type ConstArg = (Sym, Value, TypeId);

pub fn err<T>(span: Span, message: impl Into<String>) -> Result<T> {
    Err(Box::new(Diagnostic::error(span, message)))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetOs {
    Windows,
    Linux,
    MacOS,
    Wasm,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetCpu {
    X64,
    Arm64,
    Wasm,
}

#[derive(Clone)]
pub struct Options {
    pub os: TargetOs,
    pub cpu: TargetCpu,
    /// Directories searched by `#import "Name"`, in order.
    pub import_paths: Vec<PathBuf>,
    /// Path of the bootstrap module (loaded into every program).
    pub preload: Option<PathBuf>,
    /// Load Runtime_Support (entry point, context initialization) for programs.
    pub runtime_support: bool,
    pub temporary_storage_size: i64,
    /// Emit array bounds checks (`Build_Options.array_bounds_check != .OFF`).
    pub array_bounds_check: bool,
    /// `Build_Options.arithmetic_overflow_check`: 0 = off, 1 = nonfatal, 2 = fatal.
    pub arithmetic_overflow_check: u8,
    /// Maintain `context.stack_trace` while the program runs (`Build_Options.stack_trace`).
    pub stack_trace: bool,
}

impl Options {
    pub fn host() -> Self {
        let os = if cfg!(target_os = "windows") {
            TargetOs::Windows
        } else if cfg!(target_os = "macos") {
            TargetOs::MacOS
        } else if cfg!(target_arch = "wasm32") {
            TargetOs::Wasm
        } else {
            TargetOs::Linux
        };
        let cpu = if cfg!(target_arch = "aarch64") {
            TargetCpu::Arm64
        } else if cfg!(target_arch = "wasm32") {
            TargetCpu::Wasm
        } else {
            TargetCpu::X64
        };
        Self {
            os,
            cpu,
            import_paths: Vec::new(),
            preload: None,
            runtime_support: true,
            temporary_storage_size: 32768,
            array_bounds_check: true,
            arithmetic_overflow_check: 0,
            stack_trace: true,
        }
    }
}

/// A loaded source file and its scope.
pub struct FileInfo {
    pub id: FileId,
    pub path: PathBuf,
    pub module: ModuleId,
    pub scope: ScopeId,
}

pub struct Module {
    pub name: String,
    pub path: Option<PathBuf>,
    /// Names visible module-wide (`#scope_export` and `#scope_module`).
    pub scope: ScopeId,
    pub params: Vec<(Sym, Value, TypeId)>,
    /// Entities of `#module_parameters` (readable as `Module.NAME`).
    pub param_entities: Vec<scope::EntityId>,
    /// Names of the second `#module_parameters` list: one value for the whole program,
    /// so an import that only sets these joins the existing instance.
    pub program_params: Vec<Sym>,
    /// Exported top-level `using global;`: the global's members are visible to importers.
    pub exported_usings: Vec<scope::UsingEntry>,
    /// Import entries (indexes into the module scope's `imports`) of exported
    /// `using Name :: #import "M";` declarations: M's names are members of this module too.
    pub exported_using_imports: Vec<usize>,
    pub files: Vec<FileId>,
}

pub struct Compiler {
    pub options: Options,
    pub fs: Rc<dyn FileSystem>,
    pub sources: SourceMap,
    pub types: Types,
    pub program: ir::Program,
    pub files: Vec<FileInfo>,
    file_by_path: HashMap<(PathBuf, ModuleId), usize>,
    pub modules: Vec<Module>,
    module_cache: HashMap<(PathBuf, Vec<ConstArg>), ModuleId>,
    pub scopes: Vec<scope::Scope>,
    pub entities: Vec<scope::Entity>,
    pub procs: Vec<procs::ProcInfo>,
    pub poly_structs: Vec<structs::PolyStruct>,
    pub codes: Vec<Rc<ast::CodeBody>>,
    pub libraries: Vec<decls::LibraryInfo>,
    pub root_scope: ScopeId,
    pub preload: Option<ModuleId>,
    pub runtime_support: Option<ModuleId>,
    pub main_module: Option<ModuleId>,
    /// `#add_context` declarations with their declaring scope, in load order.
    pub add_contexts: Vec<(Rc<ast::Decl>, ScopeId)>,
    /// `#poke_name Module name;` directives: (module expression, name, declaring file scope).
    pub pokes: Vec<(ast::Expr, Sym, ScopeId)>,
    /// Types of the fields of a struct body laid out so far, by struct scope.
    pub field_types: HashMap<ScopeId, Vec<(Sym, TypeId)>>,
    pub context_type: Option<TypeId>,
    pub top_level_runs: Vec<(ast::Expr, ScopeId)>,
    pub asserts: Vec<(ast::Expr, Option<ast::Expr>, ScopeId)>,
    /// How many of `top_level_runs`/`asserts` already executed.
    runs_done: usize,
    asserts_done: usize,
    /// `__runtime_info`, when the program references it.
    runtime_info: Option<ir::GlobalId>,
    /// Id of the workspace this compiler builds (1: the top-level program).
    pub workspace: i64,
    /// Sources added so far (labels of added strings).
    added_sources: usize,
    /// Bodies the lenient drain failed to lower, with the `lower_epoch` of the failure.
    lenient_failures: HashMap<ProcId, (u64, Box<Diagnostic>)>,
    /// Scopes holding the bindings that a polymorphic parameter's default is evaluated in.
    default_scopes: HashMap<(ScopeId, Vec<Value>), ScopeId>,
    /// `$$x` variants: (procedure, which auto-bake params are baked) -> procedure.
    auto_bake_variants: HashMap<(ProcId, Vec<calls::AutoBake>), ProcId>,
    /// Body `#import`s already added to their file scope (by statement span).
    hoisted_imports: HashSet<Span>,
    /// Procedures that need their bodies lowered.
    pub body_queue: Vec<ProcId>,
    pub interp: crate::interp::Interp,
    /// Type_Info globals per type.
    pub type_infos: HashMap<TypeId, ir::GlobalId>,
    /// `compiler_set_type_info_flags` bits per struct (`Type_Info_Flags`).
    pub type_info_flags: HashMap<TypeId, u32>,
    /// String literal globals, deduplicated.
    pub strings: HashMap<Rc<[u8]>, ir::GlobalId>,
    pub output: Vec<u8>,
    pub warnings: Vec<Diagnostic>,
    /// Struct declaration AST per struct (for layout).
    pub struct_asts: HashMap<crate::types::StructId, structs::StructSource>,
    pub entry_point: Option<ProcId>,
    pub exports: Vec<ProcId>,
    /// Procedure literals and anonymous types, per (AST node, scope).
    pub anonymous_procs: HashMap<(ast::AstId, ScopeId), ProcId>,
    /// Macro parameters whose argument was a compile-time constant: compile-time code in
    /// the macro (`#insert -> string { ... }`) reads them even though they are locals.
    /// User-written calls being emitted, innermost last: `#caller_code` evaluates to the
    /// innermost one. Only calls to procedures that use `#caller_code` are recorded.
    pub calls_in_flight: Vec<(Rc<ast::Expr>, ScopeId)>,
    pub local_consts: HashMap<EntityId, (Value, TypeId)>,
    /// `#discard` parameters: naming one in the body is an error.
    pub discard_params: HashSet<EntityId>,
    /// The most recently emitted call that has `#must` results: call span, procedure name, and
    /// which results are `#must`.
    pub last_call_must: Option<(Span, Sym, Vec<bool>)>,
    /// Macro parameters bound to a constant argument the macro never writes: reading them
    /// gives the constant (`#if n <= 1` in a recursive macro; `message.count` of a constant
    /// string fits a `u64` parameter).
    pub const_macro_params: HashMap<EntityId, (Value, TypeId)>,
    /// Modules whose re-exports a `module_exports` call is searching.
    pub reexport_visiting: Vec<ModuleId>,
    /// Types of local variable declarations, by declaration (first name), once lowered.
    pub local_decl_types: HashMap<ast::AstId, TypeId>,
    pub anonymous_types: HashMap<(ast::AstId, ScopeId), TypeId>,
    /// Scope each `Code` value was written in (parallel to `codes`).
    pub code_scopes: Vec<ScopeId>,
    /// `using,only(...) field.path;` aliases declared in struct bodies.
    pub member_aliases: HashMap<crate::types::StructId, Vec<structs::MemberAlias>>,
    pub default_images: HashMap<TypeId, Option<Rc<value::Aggregate>>>,
    pub default_globals: HashMap<TypeId, ir::GlobalId>,
    pub initializers: HashMap<TypeId, ir::FuncId>,
    /// Interpreted function -> source procedure (reading back procedure values).
    pub func_procs: HashMap<ir::FuncId, ProcId>,
    pub thunk_scopes: HashMap<ScopeId, ScopeId>,
    /// Interpreter address of the compile-time `#Context`.
    pub ct_context: Option<u64>,
    /// Declarations the driver resolves even when unreferenced: `#program_export`
    /// procedures and `link_always` libraries.
    pub export_entities: Vec<EntityId>,
    /// Locals declared by `#asm` register declarations (`x: gpr`).
    pub asm_regs: HashMap<EntityId, asm::AsmReg>,
    /// Syntax trees and types already handed to metaprograms as records.
    pub export: code_export::ExportState,
    /// Scopes with top-level items put back to waiting because they failed while
    /// procedure bodies were mid-lowering (retried by `expand_all`).
    pub deferred_pending: Vec<ScopeId>,
    /// Why each deferred item failed; shown with a later error if it is never retried.
    pub deferred_errors: Vec<Box<Diagnostic>>,
    /// Parameter names and defaults written in procedure types (`(s: string, start := 0) -> s64`),
    /// by type, for calls through procedure values. Types ignore them: the latest header with
    /// defaults wins, and a procedure used as a value fills in a type no header described.
    pub proc_type_params: HashMap<TypeId, Rc<ProcTypeParams>>,
    /// Program parameters set by imports (`#import "Basic"()(MEMORY_DEBUGGER = DEBUG)`), by
    /// module entry path: (name, value expression, scope it is evaluated in). Applied to
    /// the module's instance as soon as both exist, before anything reads the parameter.
    pub program_param_settings: Vec<(PathBuf, Sym, ast::Expr, ScopeId)>,
    /// Set while `expand_all` retries deferred items: failures are then final.
    pub retrying_pending: bool,
    /// Lookups that reached an undefined `#placeholder`: a pending top-level item whose
    /// expansion did so waits (a metaprogram may define it later) until `finish_program`.
    pub placeholder_misses: u64,
    /// Lookups that reached something still being computed (a signature, entity or layout in
    /// progress): a failure that did so may succeed once that finishes, so it is not memoized.
    pub in_progress_misses: u64,
    /// Editor facts recorded while checking (`ide.rs`); `None` outside the language server.
    pub ide: Option<Box<ide::IdeFacts>>,
    /// Set by `finish_program`: pending items no longer wait for placeholders.
    pub placeholders_final: bool,
}

/// Names and default values of a procedure type's parameters (defaults evaluate in `scope`).
#[derive(Debug)]
pub struct ProcTypeParams {
    pub names: Vec<Option<Sym>>,
    pub defaults: Vec<Option<ast::Expr>>,
    pub scope: ScopeId,
}

impl Compiler {
    /// Give compile-time code access to the `Compiler` module's workspaces.
    pub fn attach_workspaces(&mut self, workspaces: crate::build::SharedWorkspaces) {
        self.interp.workspaces = Some(workspaces);
    }

    pub fn new(options: Options, fs: Rc<dyn FileSystem>) -> Self {
        let mut c = Compiler {
            options,
            fs,
            sources: SourceMap::default(),
            types: Types::new(),
            program: ir::Program::default(),
            files: Vec::new(),
            file_by_path: HashMap::new(),
            modules: Vec::new(),
            module_cache: HashMap::new(),
            scopes: Vec::new(),
            entities: Vec::new(),
            procs: Vec::new(),
            poly_structs: Vec::new(),
            codes: Vec::new(),
            libraries: Vec::new(),
            root_scope: ScopeId(0),
            preload: None,
            runtime_support: None,
            main_module: None,
            add_contexts: Vec::new(),
            pokes: Vec::new(),
            field_types: HashMap::new(),
            context_type: None,
            top_level_runs: Vec::new(),
            asserts: Vec::new(),
            runs_done: 0,
            asserts_done: 0,
            workspace: crate::build::TOP_LEVEL_WORKSPACE,
            runtime_info: None,
            added_sources: 0,
            lenient_failures: HashMap::new(),
            default_scopes: HashMap::new(),
            auto_bake_variants: HashMap::new(),
            hoisted_imports: HashSet::new(),
            body_queue: Vec::new(),
            interp: crate::interp::Interp::default(),
            type_infos: HashMap::new(),
            type_info_flags: HashMap::new(),
            strings: HashMap::new(),
            output: Vec::new(),
            warnings: Vec::new(),
            struct_asts: HashMap::new(),
            entry_point: None,
            exports: Vec::new(),
            anonymous_procs: HashMap::new(),
            calls_in_flight: Vec::new(),
            local_consts: HashMap::new(),
            discard_params: HashSet::new(),
            last_call_must: None,
            const_macro_params: HashMap::new(),
            reexport_visiting: Vec::new(),
            local_decl_types: HashMap::new(),
            anonymous_types: HashMap::new(),
            code_scopes: Vec::new(),
            member_aliases: HashMap::new(),
            default_images: HashMap::new(),
            default_globals: HashMap::new(),
            initializers: HashMap::new(),
            func_procs: HashMap::new(),
            thunk_scopes: HashMap::new(),
            ct_context: None,
            export_entities: Vec::new(),
            asm_regs: HashMap::new(),
            export: code_export::ExportState::default(),
            deferred_pending: Vec::new(),
            deferred_errors: Vec::new(),
            proc_type_params: HashMap::new(),
            program_param_settings: Vec::new(),
            retrying_pending: false,
            placeholder_misses: 0,
            in_progress_misses: 0,
            ide: None,
            placeholders_final: false,
        };
        c.root_scope = c.new_scope(scope::ScopeKind::Root, None, ModuleId(u32::MAX), None);
        c.declare_builtins();
        c
    }

    /// A new `Code` value. The interpreter keeps the body and its text too
    /// (`compiler_get_nodes` exports it from compile-time code).
    pub fn add_code(&mut self, body: Rc<ast::CodeBody>, scope: ScopeId) -> value::CodeId {
        self.adopt_made_codes();
        let id = value::CodeId(self.codes.len() as u32);
        let span = match &*body {
            ast::CodeBody::Expr(e) => e.span,
            ast::CodeBody::Block(b) => b.span,
        };
        let text: Rc<str> = if (span.file.0 as usize) < self.sources.len() {
            self.sources.snippet(span).into()
        } else {
            "".into()
        };
        self.interp.codes.push((body.clone(), text));
        self.codes.push(body);
        self.code_scopes.push(scope);
        id
    }

    /// Take over the codes compile-time code made (`compiler_get_code`), parsing their
    /// text again as a registered source so diagnostics can point into it.
    pub fn adopt_made_codes(&mut self) {
        while self.codes.len() < self.interp.codes.len() {
            let id = self.codes.len();
            let text = self.interp.codes[id].1.clone();
            let from = self
                .interp
                .made_codes
                .iter()
                .find(|(made, _)| *made == id)
                .map(|&(_, from)| from);
            let scope = from
                .and_then(|f| self.code_scopes.get(f).copied())
                .unwrap_or(self.root_scope);
            let file = self.sources.add(
                format!("<compiler_get_code {id}>"),
                format!("__jaic_code :: #code {text};\n").into(),
            );
            let body = crate::build::parse_code_text(file, &text)
                .unwrap_or_else(|_| self.interp.codes[id].0.clone());
            self.interp.codes[id].0 = body.clone();
            self.codes.push(body);
            self.code_scopes.push(scope);
        }
    }

    pub fn render(&self, d: &Diagnostic) -> String {
        d.render(&self.sources)
    }
}
