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
mod binary;
#[cfg(test)]
mod body_failure_tests;
pub mod body_probe;
mod callable;
pub(crate) mod calls;
pub mod code_export;
mod compile_time;
mod consteval;
mod convert;
mod debug_info;
mod decl_count_hints;
mod decls;
mod dependencies;
mod diagnostics;
mod driver;
mod enum_hints;
mod enums;
mod epochs;
mod expr;
mod flow;
mod format_check;
mod host_import;
mod host_run;
mod host_twin;
mod import_trace;
mod interfaces;
mod interpolate;
mod lambda;
pub(crate) mod lower;
mod meta_state;
mod modify;
mod module_rank;
mod modules;
mod name_caches;
mod namespace_hints;
mod namespaces;
mod output;
pub mod perf;
mod private_names;
mod proc_state;
pub mod procs;
pub(crate) mod reachability;
mod runtime_config;
mod runtime_info;
pub mod scope;
mod source_state;
mod stdlib_index;
mod stmt;
pub(crate) mod structs;
mod suggestions;
mod trap_report;
mod type_state;
mod typeinfo;
pub(crate) mod using;
mod wide;

use crate::ast;
use crate::fxhash::{HashMap, HashSet};
use crate::intern::Sym;
use crate::ir;
pub use crate::lang_items::PreludeType;
use crate::source::{Diagnostic, DiagnosticKind, FileId, SourcePath, Span};
use crate::types::{TypeId, Types};
/// The `#asm` instruction table, for editors (`jailsp`).
pub use asm::catalog as asm_catalog;
pub use code_export::Modification;
pub use compile_time::CompileTime;
pub use diagnostics::Diagnostics;
pub use driver::ProgramSource;
pub use epochs::Epochs;
pub use import_trace::{FailedAssert, ImportEdge, ImportTrace};
pub use meta_state::MetaState;
pub use module_rank::{ModuleRanking, Rank, count_importers};
pub use modules::{
    BootstrapModule, FileSystem, NativeFs, VirtualFs, content_stamp, find_module_in, import_entry,
};
pub use name_caches::NameCaches;
pub use namespaces::Namespaces;
pub use output::Output;
pub use proc_state::ProcState;
pub use scope::{EntityId, ScopeId};
pub use source_state::SourceState;
use std::path::PathBuf;
use std::rc::Rc;
pub use type_state::TypeState;
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

/// Platform on which this compiler binary executes, independent of output options.
fn compiler_host() -> (TargetOs, TargetCpu) {
    #[cfg(not(any(
        target_os = "windows",
        target_os = "macos",
        target_os = "linux",
        target_arch = "wasm32"
    )))]
    compile_error!("compiler host OS has no supported Operating_System_Tag mapping");
    #[cfg(not(any(
        target_arch = "x86_64",
        target_arch = "aarch64",
        target_arch = "wasm32"
    )))]
    compile_error!("compiler host architecture has no supported CPU_Tag mapping");
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
    (os, cpu)
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

pub const DEFAULT_TEMPORARY_STORAGE_SIZE: i64 = 32768;

/// Byte capacity reserved by `Build_Options` for target-specific machine settings.
pub const MACHINE_OPTIONS_SIZE: usize = 128;

/// The machine compile-time code runs on, as opposed to the platform the program is built for
/// (`Options::os`, `Options::cpu`, `Options::long_double`). A cross build keeps both: the
/// target compiler describes the program being built, and the host twin (`host_twin.rs`) is made
/// from this description, so everything compile-time code sees (`OS`, `CPU`, type layouts, the
/// libraries it loads) comes from here and never from the target.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HostPlatform {
    pub os: TargetOs,
    pub cpu: TargetCpu,
    /// The host C compiler's `long double` when wider than `float64`; see `long_double_for`.
    pub long_double: Option<crate::wide_float::WideFloat>,
}

impl HostPlatform {
    /// The platform this compiler binary executes on.
    pub fn native() -> Self {
        let (os, cpu) = compiler_host();
        Self {
            os,
            cpu,
            long_double: long_double_for(os, cpu, cfg!(target_env = "gnu")),
        }
    }
}

#[derive(Clone)]
pub struct Options {
    pub os: TargetOs,
    pub cpu: TargetCpu,
    /// Where compile-time code runs. Always the native platform outside tests, which replace it
    /// to exercise a host that differs from the target on a machine that does not.
    pub host: HostPlatform,
    /// Directories searched by `#import "Name"`, in order.
    pub import_paths: Vec<PathBuf>,
    /// Bootstrap entry loaded into every program (see `crate::PRELUDE_ENTRIES`).
    pub preload: Option<PathBuf>,
    /// Configured module root, independent of an explicitly selected prelude file.
    pub preload_stdlib: Option<PathBuf>,
    /// Load `Runtime_Support` (entry point, context initialization) for programs.
    pub runtime_support: bool,
    pub temporary_storage_size: i64,
    /// Emit array bounds checks (`Build_Options.array_bounds_check != .OFF`).
    pub array_bounds_check: bool,
    /// `Build_Options.null_pointer_check`: native code checks pointers before loads and stores.
    pub null_pointer_check: bool,
    /// `Build_Options.arithmetic_overflow_check`: 0 = off, 1 = nonfatal, 2 = fatal.
    pub arithmetic_overflow_check: u8,
    /// `Build_Options.cast_bounds_check`: integer casts whose value does not fit the target.
    /// 0 = off, 1 = nonfatal, 2 = fatal.
    pub cast_bounds_check: u8,
    /// Maintain `context.stack_trace` while the program runs (`Build_Options.stack_trace`).
    pub stack_trace: bool,
    /// Record variables, scopes and types for native debug information
    /// (`Build_Options.emit_debug_info != .NONE`; only `jaic` turns it on).
    pub debug_info: bool,
    /// The target C compiler's `long double` when it is wider than `float64` (`None`: it is
    /// `float64`). Decides what `#jaic_type long_double` names; see `long_double_for`.
    pub long_double: Option<crate::wide_float::WideFloat>,
    /// `Build_Options.dead_code_elimination`.
    pub dead_code: DeadCode,
    /// `Build_Options.entry_point_name`: the procedure the program starts in (`main`).
    pub entry_point: String,
    /// `Build_Options.runtime_support_definitions`: whether `Runtime_Support` defines the
    /// system entry point and the runtime initialization.
    pub runtime_entry_point: bool,
    pub runtime_initialization: bool,
    /// `Build_Options.backtrace_on_crash`: `Runtime_Support` installs its crash handler.
    pub backtrace_on_crash: bool,
}

/// The C `long double` of a target, when wider than `float64`: x87 extended on x86-64 System V
/// and MinGW (`windows_gnu`), binary128 on Linux `AArch64` and wasm32, plain `double` on Apple
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
    /// Standard-library root for bootstrap layouts and explicitly supplied preload files.
    pub fn stdlib_root(&self) -> Option<&std::path::Path> {
        self.preload_stdlib.as_deref().or_else(|| {
            self.preload
                .as_deref()
                .and_then(crate::stdlib_root_from_preload)
        })
    }

    /// Configure bootstrap selection without deriving module paths from a custom file.
    pub fn set_stdlib(&mut self, fs: &dyn FileSystem, stdlib: &std::path::Path) {
        self.preload = Some(crate::preload_path(fs, stdlib));
        self.preload_stdlib = Some(stdlib.to_path_buf());
    }

    /// Host options for compiling the program whose main file is `main`, the way every tool
    /// sets them up: the module search path in the one order ([`crate::project::import_paths`]),
    /// the standard library `inputs` names and its prelude. The target stays the host's; a tool
    /// that checks for another selects it with `crate::target::select_os_cpu`.
    pub fn for_program(
        fs: &dyn FileSystem,
        main: &std::path::Path,
        inputs: &crate::project::ProgramInputs,
    ) -> Self {
        let mut options = Self::host();
        let main_dir = main
            .parent()
            .map(std::path::Path::to_path_buf)
            .unwrap_or_default();
        options.import_paths = crate::project::import_paths(
            &main_dir,
            &inputs.configured,
            &inputs.extra,
            &inputs.stdlib,
        );
        options.set_stdlib(fs, &inputs.stdlib);
        options
    }

    /// Compile for an already planned program: its import folders and its library. A prelude
    /// the caller already chose is kept.
    pub fn use_program(&mut self, fs: &dyn FileSystem, program: &crate::project::Program) {
        self.import_paths.clone_from(&program.import_dirs);
        if self.preload.is_none() {
            self.set_stdlib(fs, &program.stdlib);
        }
    }

    /// Does the program run somewhere other than where compile-time code does?
    pub fn is_cross_build(&self) -> bool {
        (self.os, self.cpu) != (self.host.os, self.host.cpu)
    }

    pub fn host() -> Self {
        let host = HostPlatform::native();
        let (os, cpu) = (host.os, host.cpu);
        Self {
            os,
            cpu,
            host,
            import_paths: Vec::new(),
            preload: None,
            preload_stdlib: None,
            runtime_support: true,
            temporary_storage_size: DEFAULT_TEMPORARY_STORAGE_SIZE,
            array_bounds_check: true,
            null_pointer_check: true,
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
    pub path: SourcePath,
    pub module: ModuleId,
    pub scope: ScopeId,
}

/// See `Module::using_modules`.
#[derive(Clone)]
pub struct ModuleUsing {
    pub module: ModuleId,
    pub filter: Option<Rc<ast::UsingFilter>>,
    pub exported: bool,
}

pub struct Module {
    /// Entry declarations are still being registered (recursive imports may see this instance).
    pub loading: bool,
    pub argument_span: Span,
    /// Source-list provenance for validating an import before binding its header.
    pub argument_imports: Vec<Rc<ast::Import>>,
    pub arguments_validated: bool,
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
    /// Header constraints wait for declaration registration and program bindings.
    pub parameter_contracts: Vec<modules::ParameterContract>,
    /// Exported top-level `using global;`: the global's members are visible to importers.
    pub exported_usings: Vec<scope::UsingEntry>,
    /// Import entries (indexes into the module scope's `imports`) of exported
    /// `using Name :: #import "M";` declarations: M's names are members of this module too.
    pub exported_using_imports: Vec<usize>,
    /// Module-scope using imports, also reachable through qualified access.
    pub member_using_imports: Vec<usize>,
    /// Top-level `using M;` naming a module (`I :: #import "I"; using I;`): M's names are
    /// members of this module (`This.name`), filtered as written. Exported ones (at export
    /// scope) also reach importers of this module; `#scope_module` ones only `This.name`.
    pub using_modules: Vec<ModuleUsing>,
    pub files: Vec<FileId>,
}

/// Most instances one polymorphic procedure or struct may have. Polymorphic recursion
/// (`f :: (x: $T) { f(*x); }`, `P :: struct (T: Type) { p: P(*T); }`) would otherwise create
/// instances until the compiler hangs or overflows its stack.
pub const MAX_INSTANCES: usize = 2000;

/// Contents of a constant view's data global: alignment, bytes and relocations.
pub type PackKey = (u64, Vec<u8>, Vec<(u64, ir::RelocTarget, i64)>);

/// Canonical type and value of a compile-time binding used to key polymorphic instances.
/// Identical bits of different types must not share an instance. Constant view keys also
/// retain the globals they point at rather than only their descriptor bytes.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct InstanceKeyValue {
    ty: TypeId,
    value: Value,
}

fn instance_key_value(types: &Types, value: &Value, ty: TypeId) -> InstanceKeyValue {
    let view = matches!(
        types.kind(ty),
        crate::types::TypeKind::Array {
            kind: crate::types::ArrayKind::View,
            ..
        }
    );
    let value = match value {
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
    };
    InstanceKeyValue {
        ty,
        value,
    }
}

fn too_many_instances(name: Sym) -> String {
    format!(
        "`{name}` has more than {MAX_INSTANCES} polymorphic instances (does it instantiate itself \
         with ever new arguments?)"
    )
}

/// A module a metaprogram supplied for an import that found none (`provide_import`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProvidedImport {
    /// The importing module and the wanted name, as in the `FAILED_IMPORT` message.
    pub host: String,
    pub target: String,
    pub kind: ProvidedKind,
    pub value: String,
}

/// `Provided_Import_Type`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProvidedKind {
    /// Another module's name.
    ShortName,
    PathToFile,
    PathToDirectory,
    FullText,
}

pub struct Compiler {
    pub ct: CompileTime,
    pub options: Options,
    pub src: SourceState,
    pub ty: TypeState,
    pub out: Output,
    pub ns: Namespaces,
    pub pr: ProcState,
    pub meta: MetaState,
    pub epochs: Epochs,
    pub caches: NameCaches,
    /// Debug builds check `namespace_generation` against a full count of scope contents.
    #[cfg(debug_assertions)]
    namespace_check: std::cell::Cell<(usize, u64)>,
    pub diag: Diagnostics,
    /// The editor analysis this compile records for, if any (`crate::ide`).
    pub ed: crate::ide::Editor,
}

/// Names and default values of a procedure type's parameters (defaults evaluate in `scope`).
#[derive(Debug)]
pub struct ProcTypeParams {
    pub names: Vec<Option<Sym>>,
    pub defaults: Vec<Option<ast::Expr>>,
    pub scope: ScopeId,
    /// Lexical headers on declared procedure-valued results.
    pub returns: Vec<Option<Rc<ProcTypeParams>>>,
}

impl Compiler {
    /// Give compile-time code access to the `Compiler` module's workspaces.
    pub fn attach_workspaces(&mut self, workspaces: crate::build::SharedWorkspaces) {
        let usage = workspaces.borrow().typechecked_use.clone();
        self.src.sources.share_typechecked_use(usage);
        self.ct.interp.workspaces = Some(workspaces);
    }

    /// Let go of what only compile-time code uses: the workspaces (with the records and
    /// syntax trees handed to the metaprogram) and the exporter's state. An editor keeps a
    /// compiler to answer questions long after its metaprograms ran; afterwards compile-time
    /// code that reaches for the `Compiler` module traps.
    pub fn release_build_state(&mut self) {
        self.ct.interp.workspaces = None;
        self.meta.export = code_export::ExportState::default();
    }

    /// Close the interpreter for good and drop what only running or emitting the program
    /// needs: the bodies of every procedure lowered so far (keeping their signatures) and the
    /// globals' initial contents (keeping their ids, sizes and names). An editor's compile
    /// answers from its facts once it is checked, and these are tens of MB for a large
    /// program. Code checked later may still lower, but no compile-time code runs again.
    pub fn release_lowered_code(&mut self) {
        // Specialization identities may include constant images that are about to go away.
        self.ide_freeze_polymorphs();
        self.out.release_lowered();
        self.ct.interp.release_code();
    }

    pub fn new(options: Options, fs: Rc<dyn FileSystem>) -> Self {
        let cross_target = options.is_cross_build();
        let mut c = Compiler {
            pr: ProcState::default(),
            caches: NameCaches::default(),
            ns: Namespaces::default(),
            ty: TypeState::default(),
            src: SourceState::new(fs),
            out: Output::new(cross_target),
            meta: MetaState::default(),
            epochs: Epochs::default(),
            diag: Diagnostics::default(),
            options,
            ed: crate::ide::Editor::default(),
            #[cfg(debug_assertions)]
            namespace_check: std::cell::Cell::new((0, 0)),
            ct: CompileTime::default(),
        };
        c.ns.root_scope = c.new_scope(scope::ScopeKind::Root, None, ModuleId(u32::MAX), None);
        c.declare_builtins();
        c
    }

    /// A new `Code` value. The interpreter keeps the body too (`compiler_get_nodes` exports
    /// it from compile-time code).
    pub fn add_code(&mut self, body: Rc<ast::CodeBody>, scope: ScopeId) -> value::CodeId {
        self.adopt_made_codes();
        // Constant evaluation uses synthetic procedure scopes to hide runtime
        // locals. Quoted code belongs to the lexical scope where it was written,
        // so generated declarations can bind that scope's placeholders.
        let mut scope = scope;
        while let Some(parent) = self.scope(scope).parent {
            if self.ns.thunk_scopes.get(&parent) != Some(&scope) {
                break;
            }
            scope = parent;
        }
        let id = value::CodeId(self.pr.codes.len() as u32);
        self.ct.interp.codes.push(body.clone());
        self.pr.codes.push(body);
        self.pr.code_scopes.push(scope);
        id
    }

    /// Take over the codes compile-time code made (`compiler_get_code`), registering their
    /// text as a source so diagnostics can point into it.
    pub fn adopt_made_codes(&mut self) {
        while self.pr.codes.len() < self.ct.interp.codes.len() {
            let id = self.pr.codes.len();
            let made = self.ct.interp.made_codes.iter().find(|m| m.id == id);
            let from = made.map(|m| m.scope_from);
            let source = made.map(|m| m.source.clone());
            let unscoped = from.is_none_or(|f| {
                matches!(&*self.pr.codes[f], ast::CodeBody::Expr(e) if matches!(e.kind, ast::ExprKind::Null))
            });
            if unscoped {
                // Names the insertion site lacks come from where the nodes were written.
                let fallbacks = (self.ct.interp.nodes_codes.iter().rev())
                    .filter_map(|&c| self.pr.code_scopes.get(c).copied())
                    .take(32)
                    .collect();
                self.pr.unscoped_codes.insert(id, fallbacks);
            }
            let scope = from
                .and_then(|f| self.pr.code_scopes.get(f).copied())
                .unwrap_or(self.ns.root_scope);
            if let Some(source) = source {
                let file = self.src.sources.add(
                    SourcePath::virtual_source(format!("compiler_get_code {id}")),
                    source,
                );
                ast::relocate::relocate_code(Rc::make_mut(&mut self.ct.interp.codes[id]), file);
            }
            let body = self.ct.interp.codes[id].clone();
            self.pr.codes.push(body);
            self.pr.code_scopes.push(scope);
        }
    }

    /// The scope an inserted `code` resolves its names in, inserted at `site`: its own, or
    /// for code `compiler_get_code` made without one, the site (falling back on the
    /// scopes its nodes came from).
    pub fn code_scope_at(&mut self, code: value::CodeId, site: ScopeId) -> ScopeId {
        match self.pr.unscoped_codes.get(&(code.0 as usize)) {
            Some(fallbacks) if fallbacks.is_empty() => site,
            Some(fallbacks) => {
                let fallbacks = fallbacks.clone();
                let scope = self.new_hidden_scope(site);
                self.ns.scopes[scope.0 as usize].fallbacks = fallbacks;
                scope
            }
            None => self.pr.code_scopes[code.0 as usize],
        }
    }

    /// Record a warning, once per place and message (polymorphic instances check the same
    /// source again).
    pub fn warn(&mut self, d: Diagnostic) {
        if !self
            .diag
            .warnings
            .iter()
            .any(|w| w.span == d.span && w.message == d.message)
        {
            self.diag.warnings.push(d);
        }
    }

    /// `d` as text for the user, with a "did you mean" help line where one applies.
    pub fn render(&self, d: &Diagnostic) -> String {
        match self.with_name_suggestion(d) {
            Some(d) => d.render(&self.src.sources),
            None => d.render(&self.src.sources),
        }
    }
}

#[cfg(test)]
mod state_groups {
    use std::path::Path;

    /// The groups of `Compiler` state and the file each is declared in.
    const GROUPS: [(&str, &str); 10] = [
        ("CompileTime", "compile_time.rs"),
        ("SourceState", "source_state.rs"),
        ("TypeState", "type_state.rs"),
        ("Namespaces", "namespaces.rs"),
        ("NameCaches", "name_caches.rs"),
        ("ProcState", "proc_state.rs"),
        ("Output", "output.rs"),
        ("MetaState", "meta_state.rs"),
        ("Diagnostics", "diagnostics.rs"),
        ("Epochs", "epochs.rs"),
    ];

    /// Whether `line` contains `pat` not preceded by an identifier character.
    fn names(line: &str, pat: &str) -> bool {
        line.match_indices(pat).any(|(i, _)| {
            !line[..i]
                .chars()
                .next_back()
                .is_some_and(|c| c.is_alphanumeric() || c == '_')
        })
    }

    fn visit(dir: &Path, found: &mut Vec<String>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                visit(&path, found);
                continue;
            }
            if path.extension().is_none_or(|e| e != "rs") {
                continue;
            }
            let rel = path.strip_prefix(env!("CARGO_MANIFEST_DIR")).unwrap();
            let text = std::fs::read_to_string(&path).unwrap();
            for (group, own) in GROUPS {
                if rel.ends_with("src/sema/mod.rs")
                    || rel.ends_with(Path::new("src/sema").join(own))
                {
                    continue;
                }
                for (n, line) in text.lines().enumerate() {
                    let line = line.trim_start();
                    let builds = names(line, &format!("{group}::new("))
                        || names(line, &format!("{group}::default()"))
                        || (names(line, &format!("{group} {{"))
                            && !line.starts_with("impl")
                            && !line.contains("struct ")
                            && !line.contains("enum "));
                    if builds {
                        found.push(format!("{}:{}: {line}", rel.display(), n + 1));
                    }
                }
            }
        }
    }

    /// Only `Compiler::new` builds the state groups, so each starts in one known state and a
    /// reset is a method on its group rather than a fresh value made somewhere else.
    #[test]
    fn only_compiler_new_builds_state_groups() {
        let mut found = Vec::new();
        visit(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
            &mut found,
        );
        assert!(
            found.is_empty(),
            "state groups built outside sema/mod.rs:\n{}",
            found.join("\n")
        );
    }
}
