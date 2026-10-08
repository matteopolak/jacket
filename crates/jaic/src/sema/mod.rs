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
mod debug_info;
mod decls;
mod driver;
mod expr;
pub mod ide;
pub mod ide_meta;
mod lambda;
mod lower;
mod modify;
mod modules;
pub mod procs;
mod runtime_info;
pub mod scope;
mod stmt;
mod structs;
mod suggestions;
mod trap_report;
mod typeinfo;
mod wide;

use crate::ast;
use crate::fxhash::{HashMap, HashSet};
use crate::intern::Sym;
use crate::ir;
use crate::source::{Diagnostic, DiagnosticKind, FileId, SourceMap, Span};
use crate::types::{TypeId, Types};
/// The `#asm` instruction table, for editors (`jailsp`).
pub use asm::catalog as asm_catalog;
pub use code_export::ModifiedStmt;
pub use driver::ProgramSource;
pub use modules::{FileSystem, NativeFs, VirtualFs, find_module_in, import_entry};
pub use scope::{EntityId, ScopeId};
use std::path::PathBuf;
use std::rc::Rc;
pub use suggestions::{COMMON_MODULES, COMPILER_INTERNAL_MODULES};
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

/// `Build_Options.dead_code_elimination`: which declarations that nothing the program reaches
/// still get type-checked. Only checking changes; compiled output keeps just what is reachable.
/// See `docs/language/dead-code-elimination.md`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DeadCode {
    /// `.NONE` (`-no_dce`): every declaration and every non-polymorphic body, modules included.
    None,
    /// `.ALL`: only what the program reaches (struct declarations are still laid out).
    All,
    /// `.MODULES_ONLY` (the default): everything declared in the program's own files, and what
    /// that reaches in modules.
    #[default]
    ModulesOnly,
}

impl DeadCode {
    /// The mode named by a `dead_code_elimination` enum member.
    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "NONE" => Self::None,
            "ALL" => Self::All,
            "MODULES_ONLY" => Self::ModulesOnly,
            _ => return None,
        })
    }
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
    /// `Build_Options.cast_bounds_check`: integer casts whose value does not fit the target.
    /// 0 = off, 1 = nonfatal, 2 = fatal.
    pub cast_bounds_check: u8,
    /// Maintain `context.stack_trace` while the program runs (`Build_Options.stack_trace`).
    pub stack_trace: bool,
    /// Record variables, scopes and types for native debug information
    /// (`Build_Options.emit_debug_info != .NONE`; only `jaic build` turns it on).
    pub debug_info: bool,
    /// The target C compiler's `long double` when it is wider than `float64` (`None`: it is
    /// `float64`). Decides what `#jaic_type long_double` names; see `long_double_for`.
    pub long_double: Option<crate::wide_float::WideFloat>,
    /// `Build_Options.dead_code_elimination`.
    pub dead_code: DeadCode,
    /// `Build_Options.entry_point_name`: the procedure the program starts in (`main`).
    pub entry_point: String,
    /// `Build_Options.runtime_support_definitions`: whether Runtime_Support defines the
    /// system entry point and the runtime initialization.
    pub runtime_entry_point: bool,
    pub runtime_initialization: bool,
    /// `Build_Options.backtrace_on_crash`: Runtime_Support installs its crash handler.
    pub backtrace_on_crash: bool,
}

/// The C `long double` of a target, when wider than `float64`: x87 extended on x86-64 System V
/// and MinGW (`windows_gnu`), binary128 on Linux AArch64 and wasm32, plain `double` on Apple
/// arm64 and with the Microsoft toolchain.
pub fn long_double_for(
    os: TargetOs,
    cpu: TargetCpu,
    windows_gnu: bool,
) -> Option<crate::wide_float::WideFloat> {
    use crate::wide_float::WideFloat;
    match (os, cpu) {
        (TargetOs::Windows, TargetCpu::X64) if windows_gnu => Some(WideFloat::X87),
        (TargetOs::Windows, _) => None,
        (TargetOs::MacOS, TargetCpu::Arm64) => None,
        (_, TargetCpu::X64) => Some(WideFloat::X87),
        (TargetOs::Linux, TargetCpu::Arm64) | (_, TargetCpu::Wasm) => Some(WideFloat::Binary128),
        (_, TargetCpu::Arm64) => None,
    }
}

#[cfg(test)]
mod long_double_tests {
    use super::{TargetCpu, TargetOs, long_double_for};
    use crate::wide_float::WideFloat;

    /// `sizeof(long double)` per `clang -target <triple>`: 16 (x87) for x86_64-linux-gnu,
    /// x86_64-apple-darwin and x86_64-w64-windows-gnu; 16 (binary128) for aarch64-linux-gnu and
    /// wasm32; 8 for arm64-apple-darwin and every Windows MSVC or arm64 triple
    /// (aarch64-pc-windows-msvc and aarch64-pc-windows-gnu included).
    #[test]
    fn formats_follow_the_c_compiler() {
        use TargetCpu::*;
        use TargetOs::*;
        assert_eq!(long_double_for(Linux, X64, true), Some(WideFloat::X87));
        assert_eq!(long_double_for(MacOS, X64, false), Some(WideFloat::X87));
        assert_eq!(long_double_for(Windows, X64, true), Some(WideFloat::X87));
        assert_eq!(long_double_for(Windows, X64, false), None);
        assert_eq!(
            long_double_for(Linux, Arm64, true),
            Some(WideFloat::Binary128)
        );
        assert_eq!(long_double_for(MacOS, Arm64, false), None);
        assert_eq!(long_double_for(Windows, Arm64, false), None);
        assert_eq!(long_double_for(Windows, Arm64, true), None);
    }
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
            cast_bounds_check: 2,
            stack_trace: true,
            debug_info: false,
            long_double: long_double_for(os, cpu, cfg!(target_env = "gnu")),
            dead_code: DeadCode::default(),
            entry_point: "main".into(),
            runtime_entry_point: true,
            runtime_initialization: true,
            backtrace_on_crash: false,
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

/// Most instances one polymorphic procedure or struct may have. Polymorphic recursion
/// (`f :: (x: $T) { f(*x); }`, `P :: struct (T: Type) { p: P(*T); }`) would otherwise create
/// instances until the compiler hangs or overflows its stack.
pub const MAX_INSTANCES: usize = 2000;

/// Contents of a constant view's data global: alignment, bytes and relocations.
pub type PackKey = (u64, Vec<u8>, Vec<(u64, ir::RelocTarget, i64)>);

/// The form of a binding's value that keys polymorphic instances. `Value` equality looks at
/// an aggregate's bytes only, which for a constant view (`$args: ..T`) is just its count, so
/// the key also names the globals the view points at.
fn instance_key_value(types: &Types, value: &Value, ty: TypeId) -> Value {
    let view = matches!(
        types.kind(ty),
        crate::types::TypeKind::Array {
            kind: crate::types::ArrayKind::View,
            ..
        }
    );
    match value {
        Value::Bytes(agg) if view && !agg.relocs.is_empty() => {
            let mut bytes = agg.bytes.clone();
            for r in &agg.relocs {
                let (kind, id) = match r.target {
                    ir::RelocTarget::Global(g) => (0u8, g.0),
                    ir::RelocTarget::Func(f) => (1, f.0),
                    ir::RelocTarget::Foreign(f) => (2, f.0),
                };
                bytes.extend_from_slice(&r.offset.to_le_bytes());
                bytes.push(kind);
                bytes.extend_from_slice(&id.to_le_bytes());
                bytes.extend_from_slice(&r.addend.to_le_bytes());
            }
            Value::Bytes(Rc::new(value::Aggregate {
                bytes,
                relocs: Vec::new(),
            }))
        }
        other => other.clone(),
    }
}

fn too_many_instances(name: Sym) -> String {
    format!(
        "`{name}` has more than {MAX_INSTANCES} polymorphic instances (does it instantiate itself \
         with ever new arguments?)"
    )
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
    /// `runtime_support_check_failed` was looked up (`note_check_handler`).
    pub check_handler_resolved: bool,
    pub main_module: Option<ModuleId>,
    /// `#add_context` declarations with their declaring scope, in load order.
    pub add_contexts: Vec<(Rc<ast::Decl>, ScopeId)>,
    /// `#poke_name Module name;` directives: (module expression, name, declaring file scope).
    pub pokes: Vec<(ast::Expr, Sym, ScopeId)>,
    /// Types of the fields of a struct body laid out so far, by struct scope.
    pub field_types: HashMap<ScopeId, Vec<(Sym, TypeId)>>,
    pub context_type: Option<TypeId>,
    pub top_level_runs: Vec<(ast::Expr, ScopeId)>,
    pub asserts: Vec<(ast::Expr, Option<ast::Expr>, Vec<ast::Expr>, ScopeId)>,
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
    /// Queued bodies whose last lenient attempt failed at `parked_epoch`: skipped as a group by
    /// `drain_bodies_lenient` until the epoch moves, instead of being looked at on every call.
    parked_bodies: Vec<ProcId>,
    /// Bodies being lowered right now (nested through compile-time runs).
    pub(super) lowering_depth: u32,
    /// Values of `a, b :: expr;` declarations, by declaration and scope.
    multi_consts: HashMap<(ast::AstId, ScopeId), Vec<lower::Operand>>,
    /// Codes made by `code_of(procedure)`: their procedure, lowered before a typed export
    /// so its locals have types.
    pub(super) code_procs: HashMap<usize, ProcId>,
    parked_epoch: u64,
    pub interp: crate::interp::Interp,
    /// Type_Info globals per type.
    pub type_infos: HashMap<TypeId, ir::GlobalId>,
    /// Type_Info globals whose descriptor could not be built yet (`type_info_global`).
    failed_type_infos: HashMap<TypeId, ir::GlobalId>,
    /// `compiler_set_type_info_flags` bits per struct (`Type_Info_Flags`).
    pub type_info_flags: HashMap<TypeId, u32>,
    /// String literal globals, deduplicated.
    pub strings: HashMap<Rc<[u8]>, ir::GlobalId>,
    /// The data of constant views made from baked variadic arguments, deduplicated by
    /// contents so that equal argument lists share one global (and one instance).
    pub packs: HashMap<PackKey, ir::GlobalId>,
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
    /// Modules whose re-exports a `module_declarations` call is searching.
    pub reexport_visiting: Vec<ModuleId>,
    /// Types of local variable declarations, by declaration (first name), once lowered.
    /// Types of local declarations, by declaration and name index (`a, b := f()`).
    pub local_decl_types: HashMap<(ast::AstId, usize), TypeId>,
    pub anonymous_types: HashMap<(ast::AstId, ScopeId), TypeId>,
    /// Scope each `Code` value was written in (parallel to `codes`).
    pub code_scopes: Vec<ScopeId>,
    /// Codes `compiler_get_code` made without a scope to copy: their names resolve where
    /// they are inserted.
    pub unscoped_codes: HashMap<usize, Vec<ScopeId>>,
    /// Notes after a procedure declaration's body (`f :: () { ... } @thread`).
    pub proc_decl_notes: HashMap<ProcId, Vec<ast::Note>>,
    /// Calls that received their `#caller_location` by default, by that location (file,
    /// line, column): the span of their first argument. A failed `assert` names its call's
    /// location, so the report shows this as the condition.
    pub first_arguments: HashMap<(u32, u32, u32), Span>,
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
    /// Compile-time runs that ended in a trap so far.
    pub ct_traps: u64,
    /// The stand-in `Type_Info_Struct` of each polymorphic struct, by its literal.
    pub generic_struct_infos: HashMap<ast::AstId, ir::GlobalId>,
    /// Declarations the driver resolves even when unreferenced: `#program_export`
    /// procedures and `link_always` libraries.
    pub export_entities: Vec<EntityId>,
    /// Locals declared by `#asm` register declarations (`x: gpr`).
    pub asm_regs: HashMap<EntityId, asm::AsmReg>,
    /// Spin lock word shared by `lock_cmpxchg8b`/`lock_cmpxchg16b` in `#asm` blocks.
    pub asm_pair_lock: Option<ir::GlobalId>,
    /// The AES S-boxes used by `aesenc` & co. in `#asm` blocks.
    pub asm_aes_tables: Option<ir::GlobalId>,
    /// Syntax trees and types already handed to metaprograms as records.
    pub export: code_export::ExportState,
    /// Scopes with top-level items put back to waiting because they failed while
    /// procedure bodies were mid-lowering (retried by `expand_all`).
    pub deferred_pending: Vec<ScopeId>,
    /// Module and file scopes that may still have top-level items to expand or imports to load.
    /// `expand_all` visits only these (in scope order) and drops the ones it finishes, so each
    /// round a metaprogram drives costs what is new rather than everything declared so far.
    pub unsettled: std::collections::BTreeSet<ScopeId>,
    /// Entities below this index have had their named imports (`X :: #import`) resolved.
    pub named_imports_done: usize,
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
    /// How deeply each `#insert`ed string file is nested in other inserted strings (absent: a
    /// real file). Bounds code that inserts itself (`X :: "#insert X;"`).
    insert_depth: HashMap<FileId, u32>,
    /// Interpreter blocks already copied into globals while freezing one compile-time value,
    /// by `(address, bytes)`: pointer cycles (`n.next = n`) refer back instead of recursing.
    frozen: HashMap<(u64, u64), ir::GlobalId>,
    /// Editor facts recorded while checking (`ide.rs`); `None` outside the language server.
    pub ide: Option<Box<ide::IdeFacts>>,
    /// Set by `finish_program`: pending items no longer wait for placeholders.
    pub placeholders_final: bool,
    /// Set while the plain `#if` pass expands an item: lookups then leave other pending items
    /// alone, so evaluating `OS` cannot run a sibling `using x: T` or `#insert` early.
    pub lookup_without_expansion: bool,
    /// `#no_reset` globals with their types: compiled output starts from the values that
    /// compile-time code left in them (`prepare_compiled_output`).
    pub no_reset_globals: Vec<(ir::GlobalId, TypeId)>,
    /// How much code existed when `finish_program` began checking what the program does not
    /// reach: compiled output drops what was made after it and is still unreferenced.
    pub(super) unreferenced_from: Option<driver::ProgramMark>,
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
            file_by_path: HashMap::default(),
            modules: Vec::new(),
            module_cache: HashMap::default(),
            scopes: Vec::new(),
            entities: Vec::new(),
            procs: Vec::new(),
            poly_structs: Vec::new(),
            codes: Vec::new(),
            libraries: Vec::new(),
            root_scope: ScopeId(0),
            preload: None,
            runtime_support: None,
            check_handler_resolved: false,
            main_module: None,
            add_contexts: Vec::new(),
            pokes: Vec::new(),
            field_types: HashMap::default(),
            context_type: None,
            top_level_runs: Vec::new(),
            asserts: Vec::new(),
            runs_done: 0,
            asserts_done: 0,
            workspace: crate::build::TOP_LEVEL_WORKSPACE,
            runtime_info: None,
            added_sources: 0,
            lenient_failures: HashMap::default(),
            default_scopes: HashMap::default(),
            auto_bake_variants: HashMap::default(),
            hoisted_imports: HashSet::default(),
            body_queue: Vec::new(),
            parked_bodies: Vec::new(),
            lowering_depth: 0,
            multi_consts: HashMap::default(),
            code_procs: HashMap::default(),
            parked_epoch: 0,
            interp: crate::interp::Interp::default(),
            type_infos: HashMap::default(),
            failed_type_infos: HashMap::default(),
            type_info_flags: HashMap::default(),
            strings: HashMap::default(),
            packs: HashMap::default(),
            output: Vec::new(),
            warnings: Vec::new(),
            struct_asts: HashMap::default(),
            entry_point: None,
            exports: Vec::new(),
            anonymous_procs: HashMap::default(),
            calls_in_flight: Vec::new(),
            local_consts: HashMap::default(),
            discard_params: HashSet::default(),
            last_call_must: None,
            const_macro_params: HashMap::default(),
            reexport_visiting: Vec::new(),
            local_decl_types: HashMap::default(),
            anonymous_types: HashMap::default(),
            code_scopes: Vec::new(),
            unscoped_codes: HashMap::default(),
            proc_decl_notes: HashMap::default(),
            first_arguments: HashMap::default(),
            member_aliases: HashMap::default(),
            default_images: HashMap::default(),
            default_globals: HashMap::default(),
            initializers: HashMap::default(),
            func_procs: HashMap::default(),
            thunk_scopes: HashMap::default(),
            ct_context: None,
            ct_traps: 0,
            generic_struct_infos: HashMap::default(),
            export_entities: Vec::new(),
            asm_regs: HashMap::default(),
            asm_pair_lock: None,
            asm_aes_tables: None,
            export: code_export::ExportState::default(),
            deferred_pending: Vec::new(),
            unsettled: std::collections::BTreeSet::new(),
            named_imports_done: 0,
            deferred_errors: Vec::new(),
            proc_type_params: HashMap::default(),
            program_param_settings: Vec::new(),
            retrying_pending: false,
            placeholder_misses: 0,
            in_progress_misses: 0,
            insert_depth: HashMap::default(),
            frozen: HashMap::default(),
            ide: None,
            placeholders_final: false,
            lookup_without_expansion: false,
            no_reset_globals: Vec::new(),
            unreferenced_from: None,
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
            let unscoped = from.is_none_or(|f| {
                matches!(&*self.codes[f], ast::CodeBody::Expr(e) if matches!(e.kind, ast::ExprKind::Null))
            });
            if unscoped {
                // Names the insertion site lacks come from where the nodes were written.
                let fallbacks = (self.interp.nodes_codes.iter().rev())
                    .filter_map(|&c| self.code_scopes.get(c).copied())
                    .take(32)
                    .collect();
                self.unscoped_codes.insert(id, fallbacks);
            }
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

    /// The scope an inserted `code` resolves its names in, inserted at `site`: its own, or
    /// for code `compiler_get_code` made without one, the site (falling back on the
    /// scopes its nodes came from).
    pub fn code_scope_at(&mut self, code: value::CodeId, site: ScopeId) -> ScopeId {
        match self.unscoped_codes.get(&(code.0 as usize)) {
            Some(fallbacks) if fallbacks.is_empty() => site,
            Some(fallbacks) => {
                let fallbacks = fallbacks.clone();
                let module = self.scope(site).module;
                let scope = self.new_scope(scope::ScopeKind::Block, Some(site), module, None);
                self.scopes[scope.0 as usize].fallbacks = fallbacks;
                scope
            }
            None => self.code_scopes[code.0 as usize],
        }
    }

    /// `d` as text for the user, with a "did you mean" help line where one applies.
    pub fn render(&self, d: &Diagnostic) -> String {
        match self.with_name_suggestion(d) {
            Some(d) => d.render(&self.sources),
            None => d.render(&self.sources),
        }
    }
}
