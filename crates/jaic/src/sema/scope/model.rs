//! Scopes, entities and the kinds of things a name can resolve to.
use crate::sema::*;
use crate::types::TypeId;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct ScopeId(pub u32);

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct EntityId(pub u32);

/// Result of a full name lookup.
pub enum Found {
    Entities(Vec<EntityId>),
    /// The name is a member reached through a `using` entry.
    Using(UsingEntry, Sym),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScopeKind {
    Root,
    Module,
    File,
    Struct(TypeId),
    /// Polymorphic struct parameter scope (before instantiation).
    StructParams,
    Proc,
    Block,
    /// Macro expansion body: lookups for backticked names go to the caller.
    Macro,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PendingState {
    Waiting,
    Expanding,
    Done,
}

/// A conditional or generated top-level item whose declarations are not yet known.
pub struct Pending {
    pub stmt: ast::Stmt,
    pub exported: bool,
    /// File scope used for `#scope_file` declarations inside the item.
    pub file_scope: ScopeId,
    pub state: PendingState,
}

#[derive(Clone)]
pub struct ImportEntry {
    pub import: Rc<ast::Import>,
    pub module: Option<ModuleId>,
    pub loading: bool,
    /// The file that wrote the import (for relative paths and parameter evaluation).
    pub from_scope: ScopeId,
    /// `using,only(..)` / `using,except(..)` on a named import that also brings its names in.
    pub filter: ast::UsingFilter,
}

/// `using x;` inside a body/struct: names are looked up as members of `value`.
#[derive(Clone)]
pub enum UsingEntry {
    /// Members of a struct value at a place (address held in the current function).
    Place { ty: TypeId, entity: EntityId },
    /// Constants of a type (struct constants, enum members).
    Type(TypeId),
    /// A module's names, minus those an `only`/`except` filter hides.
    Module(ModuleId, Option<Rc<ast::UsingFilter>>),
    /// A resolved aggregate name filter; the target retains its original storage.
    Filtered(Box<UsingEntry>, Rc<ast::UsingFilter>),
}

impl UsingEntry {
    pub fn member_type(&self) -> Option<TypeId> {
        match self {
            Self::Place {
                ty, ..
            }
            | Self::Type(ty) => Some(*ty),
            Self::Filtered(entry, _) => entry.member_type(),
            Self::Module(..) => None,
        }
    }
}

pub struct Scope {
    pub kind: ScopeKind,
    pub parent: Option<ScopeId>,
    /// A Code insertion's synthetic lookup scope declares into this lexical
    /// scope. Its parent still supplies captured-name lookup; real blocks have
    /// no such link and retain their own declaration boundary.
    pub inserted_into: Option<ScopeId>,
    pub module: ModuleId,
    pub file: Option<FileId>,
    pub names: HashMap<Sym, Vec<EntityId>>,
    pub imports: Vec<ImportEntry>,
    pub pending: Vec<Pending>,
    /// How many leading `pending` items are `Done`: a done item never changes again, so
    /// scans start after them (every name lookup into the scope asks).
    pub pending_done: usize,
    pub usings: Vec<UsingEntry>,
    /// Procedure whose body this scope belongs to (locals are only visible inside it).
    pub proc_depth: u32,
    /// The procedure whose parameters and body a `Proc` scope holds (for `#this`).
    pub proc: Option<ProcId>,
    /// Scopes consulted for names nothing else binds (code made by `compiler_get_code`
    /// falls back on the scopes its nodes were written in).
    pub fallbacks: Vec<ScopeId>,
}

#[derive(Clone, Debug)]
pub enum Builtin {
    Type(TypeId),
    /// `size_of`, `type_of`, ... handled specially by the call checker.
    Proc(BuiltinProc),
    /// Compiler-defined constants that depend on Preload (`OS`, `CPU`, ...).
    TargetConstant(Sym),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuiltinProc {
    SizeOf,
    TypeOf,
    TypeInfo,
    InitializerOf,
    IsConstant,
    AlignOf,
    OffsetOf,
    IsValueType,
    CodeOf,
}

#[derive(Clone, Debug)]
pub enum EntityKind {
    /// Global or struct-level constant/variable from source.
    Decl {
        decl: Rc<ast::Decl>,
        index: usize,
    },
    /// `Name :: #import "X"`.
    Import(Rc<ast::Import>),
    Builtin(Builtin),
    /// A runtime local living at `addr` in the procedure being lowered.
    Local {
        ty: TypeId,
        addr: ir::Val,
        depth: u32,
    },
    /// A known constant (polymorphic bindings, module parameters, enum members...).
    Const {
        value: Value,
        ty: TypeId,
    },
    EnumMember {
        ty: TypeId,
        initializer: crate::sema::enums::Initializer,
    },
    /// `#placeholder` name awaiting a later definition.
    Placeholder,
    /// A generic declaration parameter before an instance supplies its value.
    UnboundGenericParameter,
}

#[derive(Clone, Debug)]
pub enum Resolved {
    Const {
        value: Value,
        ty: TypeId,
    },
    Proc(ProcId),
    /// An alias of an overload set (`dot :: dot_product;`).
    ProcSet(Vec<ProcId>),
    Global {
        storage: ir::Storage,
        ty: TypeId,
    },
    Module(ModuleId),
    Library(value::LibraryId),
    PolyStruct(value::PolyStructId),
}

#[derive(Clone, Debug)]
pub enum EntityState {
    Unresolved,
    Resolving,
    Done(Resolved),
    /// Its compile-time code ran and failed (an assertion, a trap). It is not run again: the
    /// run may have had effects, and a retry would repeat them (and see state the failed run
    /// left behind, such as `context.handling_assertion_failure`).
    Failed(Box<Diagnostic>),
}

pub struct Entity {
    pub name: Sym,
    pub span: Span,
    pub scope: ScopeId,
    /// Scope declarations are resolved in (the declaring file's scope for module-level names).
    pub home: ScopeId,
    pub exported: bool,
    pub kind: EntityKind,
    pub state: EntityState,
    /// Declared after `#scope_file` in a module file.
    pub file_private: bool,
    /// A constant declared without a type whose value is an untyped literal (`X :: 5`):
    /// uses of it convert like the literal would.
    pub untyped_const: bool,
}
