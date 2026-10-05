//! Facts for editors. With `Compiler::ide` set, checking records what each identifier of the
//! user's files names and its type, and the extent of block and procedure scopes. The queries
//! below answer hover (`ide_hover`), completion (`ide_visible`, `ide_members`) and member
//! receivers (`ide_receiver`) from those facts and the compiler's scopes.
use super::lower::Operand;
use super::scope::{EntityId, EntityKind, EntityState, Found, Resolved, ScopeId, ScopeKind};
use super::value::ProcId;
use super::*;
use crate::types::{ArrayKind, TypeKind};

#[derive(Default)]
pub struct IdeFacts {
    /// Files whose path starts with one of these are recorded.
    roots: Vec<String>,
    wanted: HashMap<FileId, bool>,
    pub refs: Vec<IdeRef>,
    pub scopes: Vec<(Span, ScopeId)>,
    /// Entity the identifier being checked resolved to (set by `check_ident`).
    pub(super) last_entity: Option<EntityId>,
}

impl IdeFacts {
    pub fn new(roots: Vec<String>) -> Self {
        Self {
            roots,
            ..Self::default()
        }
    }
}

#[derive(Clone, Debug)]
pub struct IdeRef {
    pub span: Span,
    pub what: IdeWhat,
    pub ty: TypeId,
}

#[derive(Clone, Debug)]
pub enum IdeWhat {
    Entity(EntityId),
    Member(Sym),
    Type(TypeId),
    Procs(Vec<ProcId>),
    Module(ModuleId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IdeKind {
    Variable,
    Constant,
    Function,
    Type,
    Module,
    Field,
    EnumMember,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IdeName {
    pub name: String,
    pub kind: IdeKind,
    pub detail: String,
}

/// What `a.b.` refers to, for member completion.
#[derive(Clone, Copy, Debug)]
pub enum IdeReceiver {
    Value(TypeId),
    Type(TypeId),
    Module(ModuleId),
}

impl Compiler {
    fn ide_wants(&mut self, file: FileId) -> bool {
        let Some(ide) = self.ide.as_mut() else {
            return false;
        };
        if let Some(&w) = ide.wanted.get(&file) {
            return w;
        }
        let path = &self.sources.get(file).path;
        let w = ide.roots.iter().any(|r| path.starts_with(r.as_str()));
        ide.wanted.insert(file, w);
        w
    }

    pub(super) fn ide_note_expr(&mut self, expr: &ast::Expr, op: &Operand) {
        let (span, what) = match &expr.kind {
            ast::ExprKind::Ident(_) => {
                let entity = self.ide.as_mut().and_then(|i| i.last_entity.take());
                let what = match op {
                    Operand::Type(t) => IdeWhat::Type(*t),
                    Operand::Procs(p) => IdeWhat::Procs(p.clone()),
                    Operand::Module(m) => IdeWhat::Module(*m),
                    _ => match entity {
                        Some(e) => IdeWhat::Entity(e),
                        None => return,
                    },
                };
                (expr.span, what)
            }
            ast::ExprKind::Member(_, name) | ast::ExprKind::InferredMember(name) => {
                (name.span, IdeWhat::Member(name.name))
            }
            _ => return,
        };
        if !self.ide_wants(span.file) {
            return;
        }
        let ty = op.ty();
        if let Some(ide) = self.ide.as_mut() {
            ide.refs.push(IdeRef {
                span,
                what,
                ty,
            });
        }
    }

    /// A declaration: hovering its name shows it.
    pub(super) fn ide_note_entity(&mut self, id: EntityId) {
        let e = self.entity(id);
        let (span, name) = (e.span, e.name);
        if span.end <= span.start || !self.ide_wants(span.file) {
            return;
        }
        // Entity spans may cover the whole declaration: narrow to the name.
        let text = self.sources.snippet(span);
        let Some(at) = text.find(name.as_str()) else {
            return;
        };
        let start = span.start + at as u32;
        let span = Span {
            file: span.file,
            start,
            end: start + name.as_str().len() as u32,
        };
        let ty = match &self.entity(id).kind {
            EntityKind::Local {
                ty, ..
            }
            | EntityKind::Const {
                ty, ..
            } => *ty,
            _ => TypeId::COMPILE_TIME,
        };
        if let Some(ide) = self.ide.as_mut() {
            ide.refs.push(IdeRef {
                span,
                what: IdeWhat::Entity(id),
                ty,
            });
        }
    }

    /// `scope` covers `span` of the source (a block, a procedure body).
    pub(super) fn ide_scope_span(&mut self, scope: ScopeId, span: Span) {
        if span.end > span.start
            && self.ide_wants(span.file)
            && let Some(ide) = self.ide.as_mut()
        {
            ide.scopes.push((span, scope));
        }
    }

    /// After compiling for an editor: check every procedure body and top-level declaration of
    /// the recorded files, not just what the program reaches. Failures are ignored.
    pub fn ide_check_all(&mut self) {
        if self.ide.is_none() {
            return;
        }
        for i in 0..self.entities.len() {
            let id = EntityId(i as u32);
            let e = self.entity(id);
            if matches!(e.kind, EntityKind::Decl { .. })
                && matches!(e.state, EntityState::Unresolved)
                && self.ide_wants(e.span.file)
            {
                let _ = self.resolve_entity(id);
            }
        }
        let mut i = 0;
        while i < self.procs.len() {
            let id = ProcId(i as u32);
            i += 1;
            let p = self.proc(id);
            let span = p.span;
            if p.is_poly
                || p.is_macro
                || p.lit.body.is_none()
                || p.body_state != super::procs::BodyState::NotNeeded
                || !self.ide_wants(span.file)
            {
                continue;
            }
            let _ = self.proc_func(id, span);
        }
        let _ = self.drain_bodies_lenient();
    }

    /// The innermost recorded scope at `offset` of `file` (else the file's own scope).
    pub fn ide_scope_at(&self, file: FileId, offset: u32) -> Option<ScopeId> {
        let inner = self.ide.as_ref().and_then(|ide| {
            ide.scopes
                .iter()
                .filter(|(s, _)| s.file == file && s.start <= offset && offset <= s.end)
                .min_by_key(|(s, _)| s.end - s.start)
                .map(|(_, id)| *id)
        });
        inner.or_else(|| {
            (0..self.scopes.len())
                .map(|i| ScopeId(i as u32))
                .find(|&s| {
                    self.scope(s).kind == ScopeKind::File && self.scope(s).file == Some(file)
                })
        })
    }

    pub fn ide_hover(&mut self, file: FileId, offset: u32) -> Option<(Span, String)> {
        let r = self
            .ide
            .as_ref()?
            .refs
            .iter()
            .filter(|r| r.span.file == file && r.span.start <= offset && offset <= r.span.end)
            .min_by_key(|r| r.span.end - r.span.start)?
            .clone();
        let name = self.sources.snippet(r.span).to_string();
        let text = match &r.what {
            IdeWhat::Entity(e) => self.ide_entity_hover(*e, Some(r.ty)),
            IdeWhat::Member(_) => format!("{name}: {}", self.types.name(r.ty)),
            IdeWhat::Type(t) => self.ide_type_hover(&name, *t),
            // An overload set may include aliases (`print :: print_to_builder;`): show each
            // under the name used.
            IdeWhat::Procs(ps) => ps
                .iter()
                .map(|&p| {
                    let header = self.sources.snippet(self.proc(p).lit.header.span);
                    format!("{name} :: {}", clip(header))
                })
                .collect::<Vec<_>>()
                .join("\n"),
            IdeWhat::Module(m) => {
                format!("{name} :: #import \"{}\"", self.modules[m.0 as usize].name)
            }
        };
        Some((r.span, text))
    }

    /// Declarations the identifier at `offset` names: an entity's declaration, each procedure
    /// of an overload set, a struct or enum. Spans are narrowed to the declared name when it
    /// starts the declaration (`name :: ...`).
    pub fn ide_definition(&mut self, file: FileId, offset: u32) -> Vec<Span> {
        let Some(r) = self.ide.as_ref().and_then(|ide| {
            ide.refs
                .iter()
                .filter(|r| r.span.file == file && r.span.start <= offset && offset <= r.span.end)
                .min_by_key(|r| r.span.end - r.span.start)
                .cloned()
        }) else {
            return Vec::new();
        };
        let name = self.sources.snippet(r.span).to_string();
        let spans: Vec<(Span, String)> = match &r.what {
            IdeWhat::Entity(e) => vec![(self.entity(*e).span, name)],
            // Each under its own name: an overload set can include aliases.
            IdeWhat::Procs(ps) => ps
                .iter()
                .map(|&p| (self.proc(p).span, self.proc(p).name.to_string()))
                .collect(),
            IdeWhat::Type(t) => match self.types.kind(*t) {
                TypeKind::Struct(s) => vec![(self.types.struct_info(*s).span, name)],
                _ => Vec::new(),
            },
            IdeWhat::Member(_) | IdeWhat::Module(_) => Vec::new(),
        };
        spans
            .into_iter()
            .filter(|(s, _)| s.end >= s.start)
            .map(|(s, name)| {
                let text = self.sources.snippet(s);
                if let Some(at) = text.find(name.as_str())
                    && text[..at].trim().is_empty()
                {
                    return Span {
                        file: s.file,
                        start: s.start + at as u32,
                        end: s.start + (at + name.len()) as u32,
                    };
                }
                // A procedure's span starts at its literal: find `name :` earlier on the line.
                let file = &self.sources.get(s.file).text;
                let line = file[..s.start as usize].rfind('\n').map_or(0, |n| n + 1);
                let found = file[line..s.start as usize]
                    .rmatch_indices(name.as_str())
                    .find(|&(at, _)| {
                        let rest = &file[line + at + name.len()..];
                        rest.trim_start().starts_with(':')
                            && !file[..line + at]
                                .ends_with(|c: char| c == '_' || c.is_alphanumeric())
                    });
                match found {
                    Some((at, _)) => Span {
                        file: s.file,
                        start: (line + at) as u32,
                        end: (line + at + name.len()) as u32,
                    },
                    None => Span {
                        end: s.start,
                        ..s
                    },
                }
            })
            .collect()
    }

    fn ide_proc_header(&self, p: ProcId) -> String {
        let info = self.proc(p);
        let header = self.sources.snippet(info.lit.header.span);
        format!("{} :: {}", info.name, clip(header))
    }

    fn ide_type_hover(&self, name: &str, t: TypeId) -> String {
        match self.types.kind(t) {
            TypeKind::Struct(s) => {
                let info = self.types.struct_info(*s);
                let fields: Vec<String> = info
                    .fields
                    .iter()
                    .filter_map(|f| Some(format!("{}: {};", f.name?, self.types.name(f.ty))))
                    .take(16)
                    .collect();
                let keyword = if info.is_union {
                    "union"
                } else {
                    "struct"
                };
                format!("{name} :: {keyword} {{ {} }}", fields.join(" "))
            }
            TypeKind::Enum(e) => {
                let info = self.types.enum_info(*e);
                let members: Vec<String> = info
                    .members
                    .iter()
                    .take(16)
                    .map(|(n, _)| format!("{n};"))
                    .collect();
                let keyword = if info.is_flags {
                    "enum_flags"
                } else {
                    "enum"
                };
                format!(
                    "{name} :: {keyword} {} {{ {} }}",
                    self.types.name(info.base),
                    members.join(" ")
                )
            }
            _ => {
                let full = self.types.name(t);
                if full == name {
                    format!("{name} :: Type")
                } else {
                    format!("{name} :: {full}")
                }
            }
        }
    }

    fn ide_entity_hover(&mut self, id: EntityId, ty: Option<TypeId>) -> String {
        let name = self.entity(id).name;
        let resolved = match self.entity(id).kind.clone() {
            EntityKind::Local {
                ty, ..
            } => return format!("{name}: {}", self.types.name(ty)),
            EntityKind::Const {
                value: value::Value::Type(t),
                ..
            } => return self.ide_type_hover(name.as_str(), t),
            EntityKind::Const {
                ty, ..
            } => return format!("{name} :: {}", self.types.name(ty)),
            EntityKind::Placeholder => return format!("{name} :: #placeholder"),
            EntityKind::Builtin(_) => {
                return format!(
                    "{name}: {}",
                    self.types.name(ty.unwrap_or(TypeId::COMPILE_TIME))
                );
            }
            _ => self.resolve_entity(id).ok(),
        };
        match resolved {
            Some(Resolved::Const {
                value: value::Value::Type(t),
                ..
            }) => self.ide_type_hover(name.as_str(), t),
            Some(Resolved::Const {
                value: value::Value::Proc(p),
                ..
            })
            | Some(Resolved::Proc(p)) => self.ide_proc_header(p),
            Some(Resolved::ProcSet(ps)) => ps
                .iter()
                .map(|&p| self.ide_proc_header(p))
                .collect::<Vec<_>>()
                .join("\n"),
            Some(Resolved::Const {
                ty,
                value,
            }) => format!(
                "{name} :: {} = {}",
                self.types.name(ty),
                value.render(&self.types)
            ),
            Some(Resolved::Global {
                ty, ..
            }) => format!("{name}: {}", self.types.name(ty)),
            Some(Resolved::Module(m)) => {
                format!("{name} :: #import \"{}\"", self.modules[m.0 as usize].name)
            }
            Some(Resolved::Library(_)) => format!("{name} :: #library"),
            Some(Resolved::PolyStruct(_)) => format!("{name} :: struct (polymorphic)"),
            None => format!(
                "{name}: {}",
                self.types.name(ty.unwrap_or(TypeId::COMPILE_TIME))
            ),
        }
    }

    /// The declaration to describe a name by: a procedure if the name has overloads (an alias
    /// like `print :: print_to_builder;` may come first).
    fn ide_pick(&self, ids: impl Iterator<Item = EntityId>) -> Option<EntityId> {
        let ids: Vec<EntityId> = ids.collect();
        let is_proc = |&e: &EntityId| match &self.entity(e).kind {
            EntityKind::Decl {
                decl, ..
            } => matches!(
                decl.value.as_ref().map(|v| &v.kind),
                Some(ast::ExprKind::Proc(_))
            ),
            _ => false,
        };
        ids.iter()
            .copied()
            .find(is_proc)
            .or_else(|| ids.first().copied())
    }

    /// Kind and one-line detail of an entity for a completion list. Declarations not resolved
    /// yet are classified by their syntax (resolving every export of a module would compile it).
    fn ide_entity_name(&self, id: EntityId) -> Option<IdeName> {
        let e = self.entity(id);
        let name = e.name.as_str();
        if name.is_empty() || name.starts_with(['\0', '`']) || name.contains(' ') {
            return None;
        }
        let (kind, detail) = match &e.kind {
            EntityKind::Local {
                ty, ..
            } => (IdeKind::Variable, self.types.name(*ty)),
            EntityKind::Const {
                value: value::Value::Type(_),
                ..
            } => (IdeKind::Type, "Type".into()),
            EntityKind::Const {
                ty, ..
            } => (IdeKind::Constant, self.types.name(*ty)),
            EntityKind::Placeholder => return None,
            EntityKind::Import(_) => (IdeKind::Module, "module".into()),
            EntityKind::Builtin(b) => match b {
                scope::Builtin::Type(_) => (IdeKind::Type, "builtin type".into()),
                scope::Builtin::Proc(_) => (IdeKind::Function, "builtin".into()),
                scope::Builtin::TargetConstant(_) => (IdeKind::Constant, "builtin constant".into()),
            },
            EntityKind::Decl {
                decl, ..
            } => match &e.state {
                EntityState::Done(Resolved::Proc(p)) => (
                    IdeKind::Function,
                    clip(self.sources.snippet(self.proc(*p).lit.header.span)),
                ),
                EntityState::Done(Resolved::Global {
                    ty, ..
                }) => (IdeKind::Variable, self.types.name(*ty)),
                EntityState::Done(Resolved::Const {
                    value: value::Value::Type(_),
                    ..
                }) => (IdeKind::Type, "Type".into()),
                EntityState::Done(Resolved::Const {
                    ty, ..
                }) => (IdeKind::Constant, self.types.name(*ty)),
                EntityState::Done(Resolved::Module(_)) => (IdeKind::Module, "module".into()),
                _ => {
                    use ast::ExprKind as E;
                    let kind = match (decl.kind, decl.value.as_ref().map(|v| &v.kind)) {
                        (_, Some(E::Proc(_))) => IdeKind::Function,
                        (_, Some(E::Struct(_) | E::Enum(_) | E::ProcType(_))) => IdeKind::Type,
                        (ast::DeclKind::Var, _) => IdeKind::Variable,
                        _ => IdeKind::Constant,
                    };
                    let detail = match decl.value.as_ref().map(|v| &v.kind) {
                        Some(E::Proc(lit)) => clip(self.sources.snippet(lit.header.span)),
                        _ => clip(self.sources.snippet(e.span)),
                    };
                    (kind, detail)
                }
            },
        };
        Some(IdeName {
            name: name.to_string(),
            kind,
            detail,
        })
    }

    /// Names visible at `offset` of `file` from `scope`: locals declared before the offset,
    /// enclosing declarations, imported modules' exports, Preload.
    pub fn ide_visible(&mut self, scope: ScopeId, file: FileId, offset: u32) -> Vec<IdeName> {
        let mut out: Vec<IdeName> = Vec::new();
        let mut seen: HashSet<Sym> = HashSet::default();
        let depth = self.scope(scope).proc_depth;
        let mut modules = Vec::new();
        let mut current = Some(scope);
        while let Some(sid) = current {
            let _ = self.expand_pending(sid);
            let mut names: Vec<(Sym, EntityId)> = self
                .scope(sid)
                .names
                .iter()
                .filter_map(|(n, ids)| Some((*n, self.ide_pick(ids.iter().copied())?)))
                .collect();
            names.sort_by_key(|(n, _)| n.as_str());
            for (name, id) in names {
                let e = self.entity(id);
                if let EntityKind::Local {
                    depth: d, ..
                } = e.kind
                {
                    // Locals of enclosing procedures are out of reach; later ones not declared yet.
                    if d != depth || (e.span.file == file && e.span.start > offset) {
                        continue;
                    }
                }
                if seen.contains(&name) {
                    continue;
                }
                if let Some(n) = self.ide_entity_name(id) {
                    seen.insert(name);
                    out.push(n);
                }
            }
            let imports = self.scope(sid).imports.len();
            for i in 0..imports {
                if let Ok(Some(m)) = self.import_module(sid, i) {
                    modules.push(m);
                }
            }
            for entry in self.scope(sid).usings.clone() {
                match entry {
                    scope::UsingEntry::Module(m) => modules.push(m),
                    scope::UsingEntry::Place {
                        ty, ..
                    } => {
                        for n in self.ide_members(IdeReceiver::Value(ty)) {
                            if seen.insert(Sym::intern(&n.name)) {
                                out.push(n);
                            }
                        }
                    }
                    scope::UsingEntry::Type(ty) => {
                        for n in self.ide_members(IdeReceiver::Type(ty)) {
                            if seen.insert(Sym::intern(&n.name)) {
                                out.push(n);
                            }
                        }
                    }
                }
            }
            if self.scope(sid).kind == ScopeKind::Module {
                modules.extend([self.preload, self.runtime_support].into_iter().flatten());
            }
            current = self.scope(sid).parent;
        }
        let mut done = HashSet::default();
        while let Some(m) = modules.pop() {
            if !done.insert(m) {
                continue;
            }
            for n in self.ide_module_exports(m, &mut modules) {
                if seen.insert(Sym::intern(&n.name)) {
                    out.push(n);
                }
            }
        }
        out
    }

    /// Exported names of a module; modules it re-exports are pushed to `more`.
    fn ide_module_exports(&mut self, m: ModuleId, more: &mut Vec<ModuleId>) -> Vec<IdeName> {
        let scope = self.modules[m.0 as usize].scope;
        let _ = self.expand_pending(scope);
        let mut ids: Vec<EntityId> = self
            .scope(scope)
            .names
            .values()
            .filter_map(|ids| {
                self.ide_pick(ids.iter().copied().filter(|&e| self.entity(e).exported))
            })
            .collect();
        ids.sort_by_key(|&e| self.entity(e).name.as_str());
        for index in self.modules[m.0 as usize].exported_using_imports.clone() {
            if let Ok(Some(inner)) = self.import_module(scope, index) {
                more.push(inner);
            }
        }
        ids.into_iter()
            .filter(|&e| !self.entity(e).file_private)
            .filter_map(|e| self.ide_entity_name(e))
            .collect()
    }

    /// What `names[0].names[1]...` is, looked up from `scope`.
    pub fn ide_receiver(&mut self, scope: ScopeId, names: &[Sym]) -> Option<IdeReceiver> {
        let (first, rest) = names.split_first()?;
        let mut recv = match self.lookup_full(scope, *first).ok()? {
            Found::Entities(ids) => self.ide_entity_receiver(*ids.first()?)?,
            Found::Using(entry, member) => {
                let ty = match entry {
                    scope::UsingEntry::Place {
                        ty, ..
                    }
                    | scope::UsingEntry::Type(ty) => ty,
                    scope::UsingEntry::Module(_) => return None,
                };
                IdeReceiver::Value(self.ide_field_type(ty, member)?)
            }
        };
        for name in rest {
            recv = match recv {
                IdeReceiver::Value(t) => IdeReceiver::Value(self.ide_field_type(t, *name)?),
                IdeReceiver::Type(t) => {
                    // `Enum.MEMBER.` or a struct constant: the value has the type itself.
                    let _ = name;
                    IdeReceiver::Value(t)
                }
                IdeReceiver::Module(m) => {
                    let ids = self.module_exports(m, *name).ok()?;
                    self.ide_entity_receiver(*ids.first()?)?
                }
            };
        }
        Some(recv)
    }

    fn ide_entity_receiver(&mut self, id: EntityId) -> Option<IdeReceiver> {
        Some(match self.entity(id).kind.clone() {
            EntityKind::Local {
                ty, ..
            } => IdeReceiver::Value(ty),
            EntityKind::Const {
                value: value::Value::Type(t),
                ..
            } => IdeReceiver::Type(t),
            EntityKind::Const {
                ty, ..
            } => IdeReceiver::Value(ty),
            EntityKind::Builtin(scope::Builtin::Type(t)) => IdeReceiver::Type(t),
            _ => match self.resolve_entity(id).ok()? {
                Resolved::Const {
                    value: value::Value::Type(t),
                    ..
                } => IdeReceiver::Type(t),
                Resolved::Const {
                    ty, ..
                }
                | Resolved::Global {
                    ty, ..
                } => IdeReceiver::Value(ty),
                Resolved::Module(m) => IdeReceiver::Module(m),
                _ => return None,
            },
        })
    }

    /// Strip pointers and distinct wrappers: what `.` reaches.
    fn ide_deref(&self, mut t: TypeId) -> TypeId {
        for _ in 0..8 {
            t = match self.types.kind(t) {
                TypeKind::Pointer(inner) => *inner,
                TypeKind::Distinct(d) => self.types.distincts[d.0 as usize].base,
                _ => return t,
            };
        }
        t
    }

    fn ide_field_type(&mut self, t: TypeId, name: Sym) -> Option<TypeId> {
        self.ide_members(IdeReceiver::Value(t))
            .into_iter()
            .find(|n| n.name == name.as_str())
            .and_then(|_| self.ide_member_type(t, name))
    }

    fn ide_member_type(&mut self, t: TypeId, name: Sym) -> Option<TypeId> {
        let t = self.ide_deref(t);
        match self.types.kind(t) {
            TypeKind::Struct(s) => {
                for f in self.types.struct_info(*s).fields.clone() {
                    if f.name == Some(name) {
                        return Some(f.ty);
                    }
                    if f.using
                        && let Some(inner) = self.ide_member_type(f.ty, name)
                    {
                        return Some(inner);
                    }
                }
                None
            }
            TypeKind::String
            | TypeKind::Array {
                ..
            } => match name.as_str() {
                "count" | "allocated" => Some(TypeId::S64),
                "data" => Some(match self.types.kind(t).clone() {
                    TypeKind::Array {
                        elem, ..
                    } => self.types.pointer(elem),
                    _ => TypeId::U8_PTR,
                }),
                _ => None,
            },
            TypeKind::Any => match name.as_str() {
                "value_pointer" => Some(TypeId::VOID_PTR),
                _ => None,
            },
            _ => None,
        }
    }

    /// Members reachable with `.` on a value or type.
    pub fn ide_members(&mut self, recv: IdeReceiver) -> Vec<IdeName> {
        let mut out = Vec::new();
        match recv {
            IdeReceiver::Module(m) => return self.ide_module_exports(m, &mut Vec::new()),
            IdeReceiver::Type(t) => {
                if let TypeKind::Enum(e) = self.types.kind(t) {
                    let info = self.types.enum_info(*e);
                    for (n, v) in &info.members {
                        out.push(IdeName {
                            name: n.to_string(),
                            kind: IdeKind::EnumMember,
                            detail: format!("{} = {v}", info.name),
                        });
                    }
                }
                // Struct constants live in the struct's scope.
                let scopes: Vec<ScopeId> = (0..self.scopes.len())
                    .map(|i| ScopeId(i as u32))
                    .filter(|&s| self.scope(s).kind == ScopeKind::Struct(t))
                    .collect();
                for s in scopes {
                    let ids: Vec<EntityId> = self
                        .scope(s)
                        .names
                        .values()
                        .filter_map(|v| v.first().copied())
                        .collect();
                    for id in ids {
                        if let EntityKind::Decl {
                            decl, ..
                        } = &self.entity(id).kind
                            && decl.kind == ast::DeclKind::Const
                            && let Some(n) = self.ide_entity_name(id)
                        {
                            out.push(n);
                        }
                    }
                }
            }
            IdeReceiver::Value(t) => {
                let t = self.ide_deref(t);
                match self.types.kind(t).clone() {
                    TypeKind::Struct(s) => {
                        let fields = self.types.struct_info(s).fields.clone();
                        for f in fields {
                            if let Some(n) = f.name {
                                out.push(IdeName {
                                    name: n.to_string(),
                                    kind: IdeKind::Field,
                                    detail: self.types.name(f.ty),
                                });
                            }
                            if f.using {
                                out.extend(self.ide_members(IdeReceiver::Value(f.ty)));
                            }
                        }
                    }
                    TypeKind::String => {
                        for (n, d) in [("count", "s64"), ("data", "*u8")] {
                            out.push(field(n, d));
                        }
                    }
                    TypeKind::Array {
                        elem,
                        kind,
                    } => {
                        let elem = self.types.name(elem);
                        out.push(field("count", "s64"));
                        out.push(field("data", &format!("*{elem}")));
                        if kind == ArrayKind::Resizable {
                            out.push(field("allocated", "s64"));
                            out.push(field("allocator", "Allocator"));
                        }
                    }
                    TypeKind::Any => {
                        out.push(field("type", "*Type_Info"));
                        out.push(field("value_pointer", "*void"));
                    }
                    TypeKind::Enum(_) => return self.ide_members(IdeReceiver::Type(t)),
                    _ => {}
                }
            }
        }
        out
    }
}

fn field(name: &str, detail: &str) -> IdeName {
    IdeName {
        name: name.into(),
        kind: IdeKind::Field,
        detail: detail.into(),
    }
}

/// First line of a snippet, bounded.
fn clip(text: &str) -> String {
    let line = text.lines().next().unwrap_or("").trim();
    let line = line.trim_end_matches('{').trim_end();
    if line.len() > 160 {
        let end = line.floor_char_boundary(160);
        format!("{}…", &line[..end])
    } else {
        line.to_string()
    }
}
