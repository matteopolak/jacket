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
    module_cache: HashMap<(PathBuf, Vec<(Sym, Value, TypeId)>), ModuleId>,
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
    /// Body `#import`s already added to their file scope (by statement span).
    hoisted_imports: HashSet<Span>,
    /// Procedures that need their bodies lowered.
    pub body_queue: Vec<ProcId>,
    pub interp: crate::interp::Interp,
    /// Type_Info globals per type.
    pub type_infos: HashMap<TypeId, ir::GlobalId>,
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
    /// Macro parameters bound to a constant string the macro never writes: `name.count`
    /// is a constant (`write(fd, message.data, message.count)` into a `u64`).
    pub const_string_params: HashMap<EntityId, i64>,
    pub anonymous_types: HashMap<(ast::AstId, ScopeId), TypeId>,
    /// Scope each `Code` value was written in (parallel to `codes`).
    pub code_scopes: Vec<ScopeId>,
    pub default_images: HashMap<TypeId, Option<Rc<value::Aggregate>>>,
    pub default_globals: HashMap<TypeId, ir::GlobalId>,
    pub initializers: HashMap<TypeId, ir::FuncId>,
    /// Interpreted function -> source procedure (reading back procedure values).
    pub func_procs: HashMap<ir::FuncId, ProcId>,
    pub thunk_scopes: HashMap<ScopeId, ScopeId>,
    /// Interpreter address of the compile-time `#Context`.
    pub ct_context: Option<u64>,
    /// Declarations of `#program_export` procedures (resolved by the driver).
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
            workspace: 1,
            runtime_info: None,
            added_sources: 0,
            hoisted_imports: HashSet::new(),
            body_queue: Vec::new(),
            interp: crate::interp::Interp::default(),
            type_infos: HashMap::new(),
            strings: HashMap::new(),
            output: Vec::new(),
            warnings: Vec::new(),
            struct_asts: HashMap::new(),
            entry_point: None,
            exports: Vec::new(),
            anonymous_procs: HashMap::new(),
            calls_in_flight: Vec::new(),
            local_consts: HashMap::new(),
            const_string_params: HashMap::new(),
            anonymous_types: HashMap::new(),
            code_scopes: Vec::new(),
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
        };
        c.root_scope = c.new_scope(scope::ScopeKind::Root, None, ModuleId(u32::MAX), None);
        c.declare_builtins();
        c
    }

    /// A new `Code` value. The interpreter keeps the body and its text too
    /// (`compiler_get_nodes` exports it from compile-time code).
    pub fn add_code(&mut self, body: Rc<ast::CodeBody>, scope: ScopeId) -> value::CodeId {
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

    pub fn render(&self, d: &Diagnostic) -> String {
        d.render(&self.sources)
    }
}
