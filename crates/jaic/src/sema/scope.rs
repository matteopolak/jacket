//! Scopes, entities and name lookup.
use super::*;
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
    Place {
        ty: TypeId,
        entity: EntityId,
    },
    /// Constants of a type (struct constants, enum members).
    Type(TypeId),
    Module(ModuleId),
}

pub struct Scope {
    pub kind: ScopeKind,
    pub parent: Option<ScopeId>,
    pub module: ModuleId,
    pub file: Option<FileId>,
    pub names: HashMap<Sym, Vec<EntityId>>,
    pub imports: Vec<ImportEntry>,
    pub pending: Vec<Pending>,
    pub usings: Vec<UsingEntry>,
    /// Procedure whose body this scope belongs to (locals are only visible inside it).
    pub proc_depth: u32,
    /// The procedure whose parameters and body a `Proc` scope holds (for `#this`).
    pub proc: Option<ProcId>,
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
    /// `#placeholder` name awaiting a later definition.
    Placeholder,
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

/// A procedure-valued declaration can join an overload set.
fn decl_is_proc(decl: &ast::Decl) -> bool {
    decl.kind == ast::DeclKind::Const
        && matches!(
            decl.value.as_ref().map(|v| &v.kind),
            Some(ast::ExprKind::Proc(_) | ast::ExprKind::Lambda { .. })
        )
}

/// A condition made of names, literals and operators only.
fn plain_condition(expr: &ast::Expr) -> bool {
    use ast::ExprKind as E;
    match &expr.kind {
        E::Ident(_) | E::Int(_) | E::Bool(_) | E::Str(_) | E::InferredMember(_) | E::Null => true,
        E::Member(base, _) => plain_condition(base),
        E::Unary(_, a) => plain_condition(a),
        E::Binary(_, a, b) => plain_condition(a) && plain_condition(b),
        _ => false,
    }
}

impl Compiler {
    pub fn new_scope(
        &mut self,
        kind: ScopeKind,
        parent: Option<ScopeId>,
        module: ModuleId,
        file: Option<FileId>,
    ) -> ScopeId {
        let proc_depth = parent.map_or(0, |p| self.scopes[p.0 as usize].proc_depth)
            + u32::from(kind == ScopeKind::Proc);
        self.scopes.push(Scope {
            kind,
            parent,
            module,
            file,
            names: HashMap::default(),
            imports: Vec::new(),
            pending: Vec::new(),
            usings: Vec::new(),
            proc_depth,
            proc: None,
        });
        ScopeId(self.scopes.len() as u32 - 1)
    }
    pub fn scope(&self, id: ScopeId) -> &Scope {
        &self.scopes[id.0 as usize]
    }
    pub fn scope_mut(&mut self, id: ScopeId) -> &mut Scope {
        &mut self.scopes[id.0 as usize]
    }
    pub fn entity(&self, id: EntityId) -> &Entity {
        &self.entities[id.0 as usize]
    }
    pub fn entity_mut(&mut self, id: EntityId) -> &mut Entity {
        &mut self.entities[id.0 as usize]
    }

    pub fn add_entity(
        &mut self,
        scope: ScopeId,
        name: Sym,
        span: Span,
        kind: EntityKind,
        exported: bool,
    ) -> EntityId {
        let id = EntityId(self.entities.len() as u32);
        self.entities.push(Entity {
            name,
            span,
            scope,
            home: scope,
            exported,
            kind,
            state: EntityState::Unresolved,
            file_private: false,
            untyped_const: false,
        });
        self.scope_mut(scope)
            .names
            .entry(name)
            .or_default()
            .push(id);
        if self.ide.is_some() {
            self.ide_note_entity(id);
        }
        id
    }

    /// Declare a constant whose value is already known.
    pub fn add_const(
        &mut self,
        scope: ScopeId,
        name: Sym,
        span: Span,
        value: Value,
        ty: TypeId,
    ) -> EntityId {
        let id = self.add_entity(
            scope,
            name,
            span,
            EntityKind::Const {
                value: value.clone(),
                ty,
            },
            true,
        );
        self.entity_mut(id).state = EntityState::Done(Resolved::Const {
            value,
            ty,
        });
        id
    }

    pub(super) fn declare_builtins(&mut self) {
        let root = self.root_scope;
        let types: &[(&str, TypeId)] = &[
            ("void", TypeId::VOID),
            ("bool", TypeId::BOOL),
            ("s8", TypeId::S8),
            ("s16", TypeId::S16),
            ("s32", TypeId::S32),
            ("s64", TypeId::S64),
            ("int", TypeId::S64),
            ("u8", TypeId::U8),
            ("u16", TypeId::U16),
            ("u32", TypeId::U32),
            ("u64", TypeId::U64),
            ("float", TypeId::F32),
            ("float32", TypeId::F32),
            ("float64", TypeId::F64),
            ("string", TypeId::STRING),
            ("Type", TypeId::TYPE),
            ("Any", TypeId::ANY),
            ("Code", TypeId::CODE),
            // `#asm` register parameters of macros bind to the caller's operand expression.
            ("__reg", TypeId::CODE),
        ];
        for &(name, ty) in types {
            self.add_const(
                root,
                Sym::intern(name),
                Span::default(),
                Value::Type(ty),
                TypeId::TYPE,
            );
        }
        let procs: &[(&str, BuiltinProc)] = &[
            ("size_of", BuiltinProc::SizeOf),
            ("type_of", BuiltinProc::TypeOf),
            ("type_info", BuiltinProc::TypeInfo),
            ("initializer_of", BuiltinProc::InitializerOf),
            ("is_constant", BuiltinProc::IsConstant),
            ("align_of", BuiltinProc::AlignOf),
            ("offset_of", BuiltinProc::OffsetOf),
        ];
        for &(name, p) in procs {
            let id = self.add_entity(
                root,
                Sym::intern(name),
                Span::default(),
                EntityKind::Builtin(Builtin::Proc(p)),
                true,
            );
            let _ = id;
        }
        for name in [
            "OS",
            "CPU",
            "BUILD_OS",
            "BUILD_CPU",
            "TEMPORARY_STORAGE_SIZE",
            "LANGUAGE_VERSION_MAJOR",
            "LANGUAGE_VERSION_MINOR",
        ] {
            self.add_entity(
                root,
                Sym::intern(name),
                Span::default(),
                EntityKind::Builtin(Builtin::TargetConstant(Sym::intern(name))),
                true,
            );
        }
    }

    pub fn entity_is_overloadable(&self, id: EntityId) -> bool {
        match &self.entity(id).kind {
            EntityKind::Decl {
                decl, ..
            } => decl_is_proc(decl),
            EntityKind::Const {
                value: Value::Proc(_),
                ..
            } => true,
            _ => false,
        }
    }

    /// Like `entity_is_overloadable`, but `name :: other_proc;` aliases count too
    /// (resolved on demand; an alias that cannot be resolved yet shadows).
    fn overloadable_or_alias(&mut self, id: EntityId) -> bool {
        if self.entity_is_overloadable(id) {
            return true;
        }
        let EntityKind::Decl {
            decl, ..
        } = &self.entity(id).kind
        else {
            return false;
        };
        let alias = decl.kind == ast::DeclKind::Const
            && decl.ty.is_none()
            && matches!(
                decl.value.as_ref().map(|v| &v.kind),
                Some(ast::ExprKind::Ident(_) | ast::ExprKind::Member(..))
            );
        alias
            && matches!(
                self.resolve_entity(id),
                Ok(Resolved::Proc(_)
                    | Resolved::ProcSet(_)
                    | Resolved::Const {
                        value: Value::Proc(_),
                        ..
                    })
            )
    }

    /// Expand pending conditional items of a scope (each at most once).
    pub fn expand_pending(&mut self, scope: ScopeId) -> Result<()> {
        let mut i = 0;
        while i < self.scope(scope).pending.len() {
            // Items expand in source order: a re-entrant expansion (from inside an
            // item's own condition or lookup) must not run later items early.
            if self.scope(scope).pending[i].state == PendingState::Expanding {
                return Ok(());
            }
            if self.scope(scope).pending[i].state == PendingState::Waiting {
                self.scope_mut(scope).pending[i].state = PendingState::Expanding;
                let stmt = self.scope(scope).pending[i].stmt.clone();
                let exported = self.scope(scope).pending[i].exported;
                let file_scope = self.scope(scope).pending[i].file_scope;
                // Compile-time code run from inside a body that is being lowered
                // (a lookup reached this scope) cannot call procedures that are
                // themselves mid-lowering. Put the item back and retry it later.
                let reentrant = !self.retrying_pending
                    && self
                        .procs
                        .iter()
                        .any(|p| p.body_state == super::procs::BodyState::Lowering);
                let misses = self.placeholder_misses;
                let result = self.expand_pending_item(scope, file_scope, &stmt, exported);
                if result.is_err() && self.waits_for_placeholder(misses) {
                    self.scope_mut(scope).pending[i].state = PendingState::Waiting;
                    return Ok(());
                }
                if reentrant && let Err(e) = result {
                    self.scope_mut(scope).pending[i].state = PendingState::Waiting;
                    self.deferred_pending.push(scope);
                    self.deferred_errors.push(e);
                    return Ok(());
                }
                self.scope_mut(scope).pending[i].state = PendingState::Done;
                result?;
            }
            i += 1;
        }
        Ok(())
    }

    /// Expand a scope's `#if` items whose conditions are plain constant expressions (no
    /// calls, no `#run`), ahead of items that run compile-time code. `expand_all` does
    /// this for every scope first, so a module's `#if FLAG #load "x.jai"` (and the
    /// `#add_context` in it) is in before any `#run` lays out the Context.
    pub fn expand_plain_ifs(&mut self, scope: ScopeId) -> Result<()> {
        for i in 0..self.scope(scope).pending.len() {
            let item = &self.scope(scope).pending[i];
            if item.state != PendingState::Waiting {
                continue;
            }
            let ast::StmtKind::StaticIf {
                cond, ..
            } = &item.stmt.kind
            else {
                continue;
            };
            if !plain_condition(cond) {
                continue;
            }
            self.scope_mut(scope).pending[i].state = PendingState::Expanding;
            let item = &self.scope(scope).pending[i];
            let (stmt, exported, file_scope) = (item.stmt.clone(), item.exported, item.file_scope);
            let misses = self.placeholder_misses;
            let result = self.expand_pending_item(scope, file_scope, &stmt, exported);
            if result.is_err() && self.waits_for_placeholder(misses) {
                self.scope_mut(scope).pending[i].state = PendingState::Waiting;
                continue;
            }
            self.scope_mut(scope).pending[i].state = PendingState::Done;
            result?;
        }
        Ok(())
    }

    /// Did an item's failed expansion reach an undefined `#placeholder` (since `before`)?
    fn waits_for_placeholder(&self, before: u64) -> bool {
        !self.placeholders_final && self.placeholder_misses != before
    }

    /// Look up `name` starting at `scope`, ignoring `using` struct members.
    pub fn lookup(&mut self, scope: ScopeId, name: Sym) -> Result<Vec<EntityId>> {
        match self.lookup_full(scope, name)? {
            Found::Entities(ids) => Ok(ids),
            Found::Using(..) => Ok(Vec::new()),
        }
    }

    /// Full lookup. Procedures from several scopes merge into one overload
    /// set; any other binding shadows everything outside it. `using` entries
    /// of a scope are consulted after its own names.
    pub fn lookup_full(&mut self, scope: ScopeId, name: Sym) -> Result<Found> {
        let mut found: Vec<EntityId> = Vec::new();
        let mut current = Some(scope);
        while let Some(sid) = current {
            // A name this scope already binds to a non-procedure is final (another
            // declaration would be a redefinition): pending items need not run first.
            let settled = self.scope(sid).names.get(&name).is_some_and(|ids| {
                ids.iter().any(|&e| {
                    !matches!(self.entity(e).kind, EntityKind::Placeholder)
                        && !self.entity_is_overloadable(e)
                })
            });
            if !settled {
                self.expand_pending(sid)?;
            }
            if let Some(ids) = self.scope(sid).names.get(&name).cloned() {
                // A `#placeholder` gives way to the real declaration (possibly
                // added later by a metaprogram).
                let is_placeholder =
                    |e: &EntityId| matches!(self.entity(*e).kind, EntityKind::Placeholder);
                let defined = ids.iter().any(|e| !is_placeholder(e));
                let ids: Vec<EntityId> = ids
                    .into_iter()
                    .filter(|e| !is_placeholder(e) || (!defined && found.is_empty()))
                    .collect();
                if !ids.is_empty() && self.collect(&mut found, &ids) {
                    return Ok(Found::Entities(found));
                }
            }
            for entry in self.scope(sid).usings.clone() {
                match &entry {
                    UsingEntry::Module(m) => {
                        let ids = self.module_exports(*m, name)?;
                        if !ids.is_empty() && self.collect(&mut found, &ids) {
                            return Ok(Found::Entities(found));
                        }
                    }
                    UsingEntry::Place {
                        ty, ..
                    }
                    | UsingEntry::Type(ty) => {
                        if found.is_empty() && self.type_has_member(*ty, name)? {
                            return Ok(Found::Using(entry.clone(), name));
                        }
                    }
                }
            }
            let imports = self.scope(sid).imports.len();
            for i in 0..imports {
                if let Some(module) = self.import_module(sid, i)? {
                    let hidden = match &self.scope(sid).imports[i].filter {
                        ast::UsingFilter::Only(names) => names.iter().all(|n| n.name != name),
                        ast::UsingFilter::Except(names) => names.iter().any(|n| n.name == name),
                        _ => false,
                    };
                    if hidden {
                        continue;
                    }
                    let ids = self.module_exports(module, name)?;
                    if !ids.is_empty() && self.collect(&mut found, &ids) {
                        return Ok(Found::Entities(found));
                    }
                    // Members of a global the module exports with `using` (GL's `using gl;`).
                    if found.is_empty() {
                        for entry in self.modules[module.0 as usize].exported_usings.clone() {
                            if let UsingEntry::Place {
                                ty, ..
                            } = &entry
                                && self.type_has_member(*ty, name)?
                            {
                                return Ok(Found::Using(entry, name));
                            }
                        }
                    }
                }
            }
            // Preload and Runtime_Support are implicitly visible to every module.
            if self.scope(sid).kind == ScopeKind::Module {
                for implicit in [self.preload, self.runtime_support].into_iter().flatten() {
                    if self.scope(sid).module == implicit {
                        continue;
                    }
                    let ids = self.module_exports(implicit, name)?;
                    if !ids.is_empty() && self.collect(&mut found, &ids) {
                        return Ok(Found::Entities(found));
                    }
                }
            }
            current = self.scope(sid).parent;
        }
        if found.is_empty() {
            return self.lookup_sibling_file_imports(scope, name);
        }
        Ok(Found::Entities(found))
    }

    /// Last resort for an unknown name: modules imported under `#scope_file` by the other
    /// files of the same module (code such as focus relies on these being visible
    /// module-wide). Only reached when nothing else binds the name.
    fn lookup_sibling_file_imports(&mut self, scope: ScopeId, name: Sym) -> Result<Found> {
        let module = self.scope(scope).module;
        let file_scopes: Vec<ScopeId> = self
            .files
            .iter()
            .filter(|f| f.module == module)
            .map(|f| f.scope)
            .collect();
        let mut found = Vec::new();
        for fs in file_scopes {
            for i in 0..self.scope(fs).imports.len() {
                if !matches!(self.scope(fs).imports[i].filter, ast::UsingFilter::None) {
                    continue;
                }
                if let Some(m) = self.import_module(fs, i)? {
                    for id in self.module_exports(m, name)? {
                        if !found.contains(&id) {
                            found.push(id);
                        }
                    }
                }
            }
        }
        Ok(Found::Entities(found))
    }

    /// Merge `ids` into `found`; returns true when the lookup must stop
    /// (a non-procedure binding shadows everything outside it).
    fn collect(&mut self, found: &mut Vec<EntityId>, ids: &[EntityId]) -> bool {
        let overloadable = ids.iter().all(|&e| self.overloadable_or_alias(e));
        if found.is_empty() {
            found.extend_from_slice(ids);
            return !overloadable;
        }
        if overloadable {
            for &id in ids {
                if !found.contains(&id) {
                    found.push(id);
                }
            }
        }
        false
    }

    /// Exported names of a module (after expanding its pending items).
    pub fn module_exports(&mut self, module: ModuleId, name: Sym) -> Result<Vec<EntityId>> {
        let scope = self.modules[module.0 as usize].scope;
        self.expand_pending(scope)?;
        let own: Vec<EntityId> = self
            .scope(scope)
            .names
            .get(&name)
            .map(|ids| {
                ids.iter()
                    .copied()
                    .filter(|&e| self.entity(e).exported)
                    .collect()
            })
            .unwrap_or_default();
        if !own.is_empty() {
            return Ok(own);
        }
        // Names of modules this one re-exports with `using M :: #import "M";` (each module
        // once per lookup, so re-export cycles end).
        if self.reexport_visiting.contains(&module) {
            return Ok(own);
        }
        self.reexport_visiting.push(module);
        let mut result = Ok(own);
        for index in self.modules[module.0 as usize]
            .exported_using_imports
            .clone()
        {
            let ids = match self.import_module(scope, index) {
                Ok(Some(inner)) => self.module_exports(inner, name),
                Ok(None) => continue,
                Err(e) => Err(e),
            };
            if !matches!(&ids, Ok(ids) if ids.is_empty()) {
                result = ids;
                break;
            }
        }
        self.reexport_visiting.pop();
        result
    }

    /// Names visible inside a module from one of its files (export + module scope).
    pub fn module_member(&mut self, module: ModuleId, name: Sym) -> Result<Vec<EntityId>> {
        self.module_exports(module, name)
    }
}
