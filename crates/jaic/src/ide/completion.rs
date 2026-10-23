//! Names visible at a position, and the members of a receiver.
use super::facts::{IdeKind, IdeName, IdeReceiver, IdeRef, IdeWhat};
use crate::ast;
use crate::fxhash::HashSet;
use crate::intern::Sym;
use crate::sema::scope::{EntityId, EntityKind, EntityState, Found, Resolved, ScopeId, ScopeKind};
use crate::sema::{Compiler, ModuleId, scope, value};
use crate::source::FileId;
use crate::source::Span;
use crate::types::TypeId;
use crate::types::{ArrayKind, TypeKind};

impl Compiler {
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
                    modules.push((m, Some(self.scope(sid).imports[i].filter.clone())));
                }
            }
            for entry in self.scope(sid).usings.clone() {
                match entry {
                    scope::UsingEntry::Module(m, filter) => {
                        modules.push((m, filter.as_deref().cloned()));
                    }
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
                    scope::UsingEntry::Filtered(..) => {
                        for n in self.ide_using_members(&entry) {
                            if seen.insert(Sym::intern(&n.name)) {
                                out.push(n);
                            }
                        }
                    }
                }
            }
            if self.scope(sid).kind == ScopeKind::Module {
                modules.extend(
                    [self.ns.preload, self.ns.runtime_support]
                        .into_iter()
                        .flatten()
                        .map(|m| (m, None)),
                );
            }
            current = self.scope(sid).parent;
        }
        while let Some((m, filter)) = modules.pop() {
            let mut names = self.ide_module_exports(m, &mut HashSet::default());
            if let Some(filter) = filter {
                Self::ide_filter_names(&mut names, &filter);
            }
            for n in names {
                if seen.insert(Sym::intern(&n.name)) {
                    out.push(n);
                }
            }
        }
        out
    }

    /// Exported names, applying each edge filter before merging transitive names.
    /// Only the active path is guarded: another filtered path to the same module
    /// may expose additional names and must still be traversed.
    fn ide_module_exports(
        &mut self,
        m: ModuleId,
        visiting: &mut HashSet<ModuleId>,
    ) -> Vec<IdeName> {
        self.ide_module_names(m, visiting, false)
    }

    fn ide_module_names(
        &mut self,
        m: ModuleId,
        visiting: &mut HashSet<ModuleId>,
        qualified: bool,
    ) -> Vec<IdeName> {
        if !visiting.insert(m) {
            return Vec::new();
        }
        let scope = self.ns.modules[m.0 as usize].scope;
        let _ = self.expand_pending(scope);
        let mut ids: Vec<EntityId> = self
            .scope(scope)
            .names
            .values()
            .filter_map(|ids| {
                self.ide_pick(
                    ids.iter()
                        .copied()
                        .filter(|&e| self.entity(e).exported || qualified),
                )
            })
            .collect();
        ids.sort_by_key(|&e| self.entity(e).name.as_str());
        let mut names = Vec::new();
        for id in ids {
            let entity = self.entity(id);
            if entity.file_private {
                continue;
            }
            // Facades often export procedures through aliases. Syntax alone calls
            // these constants, even when their targets are overload sets. Resolve
            // only simple aliases through the usual lookup/visibility rules; do
            // not eagerly compile every unrelated declaration or procedure body.
            if matches!(entity.state, EntityState::Unresolved)
                && let EntityKind::Decl {
                    decl, ..
                } = &entity.kind
                && decl.kind == ast::DeclKind::Const
                && matches!(
                    decl.value.as_ref().map(|v| &v.kind),
                    Some(ast::ExprKind::Ident(_) | ast::ExprKind::Member(..))
                )
            {
                let _ = self.resolve_entity(id);
            }
            if let Some(name) = self.ide_entity_name(id) {
                names.push(name);
            }
        }
        let imports = if qualified {
            &self.ns.modules[m.0 as usize].member_using_imports
        } else {
            &self.ns.modules[m.0 as usize].exported_using_imports
        }
        .clone();
        for index in imports {
            if let Ok(Some(inner)) = self.import_module(scope, index) {
                let mut inherited = self.ide_module_exports(inner, visiting);
                Self::ide_filter_names(&mut inherited, &self.scope(scope).imports[index].filter);
                names.extend(inherited);
            }
        }
        for used in self.ns.modules[m.0 as usize].using_modules.clone() {
            if !qualified && !used.exported {
                continue;
            }
            let mut inherited = self.ide_module_exports(used.module, visiting);
            if let Some(filter) = &used.filter {
                Self::ide_filter_names(&mut inherited, filter);
            }
            names.extend(inherited);
        }
        let usings = if qualified {
            self.scope(scope).usings.clone()
        } else {
            self.ns.modules[m.0 as usize].exported_usings.clone()
        };
        for entry in usings {
            names.extend(self.ide_using_members_inner(&entry, visiting));
        }
        visiting.remove(&m);
        let mut seen = HashSet::default();
        names.retain(|n| seen.insert(n.name.clone()));
        names
    }

    fn ide_filter_names(names: &mut Vec<IdeName>, filter: &ast::UsingFilter) {
        match filter {
            ast::UsingFilter::Map(pairs) => {
                *names = pairs
                    .iter()
                    .filter_map(|(new, old)| {
                        let mut n = names.iter().find(|n| n.name == old.name.as_str())?.clone();
                        n.name = new.name.to_string();
                        Some(n)
                    })
                    .collect();
            }
            filter => names.retain(|n| !scope::filter_hides(filter, Sym::intern(&n.name))),
        }
    }

    fn ide_using_members(&mut self, entry: &scope::UsingEntry) -> Vec<IdeName> {
        self.ide_using_members_inner(entry, &mut HashSet::default())
    }

    pub(super) fn ide_filter_targets(
        &self,
        file: FileId,
        offset: u32,
    ) -> Option<(Span, Vec<EntityId>)> {
        self.ed
            .facts()?
            .filter_targets
            .iter()
            .filter(|(span, _)| span.file == file && span.start <= offset && offset <= span.end)
            .min_by_key(|(span, _)| span.end - span.start)
            .map(|(&span, ids)| (span, ids.clone()))
    }

    /// Static filters contain selector names, not ordinary expressions. Record an
    /// existing visible target without turning nonexistent selectors into errors
    /// (the language permits them) or inventing source declarations for builtins.
    pub(crate) fn ide_note_filter_names(
        &mut self,
        entry: &scope::UsingEntry,
        filter: &ast::UsingFilter,
    ) {
        if !self.ed.is_live() {
            return;
        }
        let (ast::UsingFilter::Only(names) | ast::UsingFilter::Except(names)) = filter else {
            return;
        };
        for &name in names {
            if self.ide_wants(name.span.file) {
                self.ide_note_filter_target(entry, name, &mut HashSet::default());
            }
        }
    }

    fn ide_note_filter_target(
        &mut self,
        entry: &scope::UsingEntry,
        name: ast::Ident,
        visiting: &mut HashSet<(ModuleId, Sym)>,
    ) {
        match entry {
            scope::UsingEntry::Module(module, filter) => {
                let target = match filter.as_deref() {
                    Some(filter) => crate::sema::using::filter_target(filter, name.name),
                    None => Some(name.name),
                };
                let Some(target) = target else {
                    return;
                };
                if !visiting.insert((*module, target)) {
                    return;
                }
                match self.module_member_lookup(*module, target) {
                    Ok(Found::Entities(ids)) if !ids.is_empty() => {
                        if let Some(ide) = self.ed.facts_mut() {
                            if ids.len() > 1 {
                                ide.filter_targets.insert(name.span, ids);
                            } else {
                                ide.refs.push(IdeRef {
                                    span: name.span,
                                    what: IdeWhat::Entity(ids[0]),
                                    ty: TypeId::COMPILE_TIME,
                                    decl: false,
                                });
                            }
                        }
                    }
                    Ok(Found::Using(entry, target)) => self.ide_note_filter_target(
                        &entry,
                        ast::Ident {
                            name: target,
                            span: name.span,
                        },
                        visiting,
                    ),
                    _ => {}
                }
            }
            scope::UsingEntry::Filtered(base, filter) => {
                if let Some(target) = crate::sema::using::filter_target(filter, name.name) {
                    self.ide_note_filter_target(
                        base,
                        ast::Ident {
                            name: target,
                            span: name.span,
                        },
                        visiting,
                    );
                }
            }
            scope::UsingEntry::Type(ty)
            | scope::UsingEntry::Place {
                ty, ..
            } => {
                if let (Some(decl), Some(ty)) = (
                    self.ide_member_decl(*ty, name.name),
                    self.ide_field_type(*ty, name.name),
                ) {
                    self.ide_note_named_field(name.span, name.name, decl, ty);
                }
            }
        }
    }

    fn ide_using_members_inner(
        &mut self,
        entry: &scope::UsingEntry,
        visiting: &mut HashSet<ModuleId>,
    ) -> Vec<IdeName> {
        match entry {
            scope::UsingEntry::Place {
                ty, ..
            } => self.ide_members(IdeReceiver::Value(*ty)),
            scope::UsingEntry::Type(ty) => self.ide_members(IdeReceiver::Type(*ty)),
            scope::UsingEntry::Module(m, filter) => {
                let mut names = self.ide_module_exports(*m, visiting);
                if let Some(filter) = filter {
                    Self::ide_filter_names(&mut names, filter);
                }
                names
            }
            scope::UsingEntry::Filtered(base, filter) => {
                let mut names = self.ide_using_members_inner(base, visiting);
                Self::ide_filter_names(&mut names, filter);
                names
            }
        }
    }

    /// What `names[0].names[1]...` is, looked up from `scope`.
    pub fn ide_receiver(&mut self, scope: ScopeId, names: &[Sym]) -> Option<IdeReceiver> {
        self.ide_receiver_at(scope, None, names)
    }

    /// [`Compiler::ide_receiver`] as seen from `at` (file, offset): a local declared further down
    /// its block does not hide the name it shadows until then.
    pub fn ide_receiver_at(
        &mut self,
        scope: ScopeId,
        at: Option<(FileId, u32)>,
        names: &[Sym],
    ) -> Option<IdeReceiver> {
        let (first, rest) = names.split_first()?;
        let mut from = scope;
        if let Some((file, offset)) = at {
            while self.scope(from).names.get(first).is_some_and(|ids| {
                !ids.is_empty()
                    && ids.iter().all(|&e| {
                        let e = self.entity(e);
                        matches!(e.kind, EntityKind::Local { .. })
                            && e.span.file == file
                            && e.span.start > offset
                    })
            }) && let Some(parent) = self.scope(from).parent
            {
                from = parent;
            }
        }
        let mut recv = match self.lookup_full(from, *first).ok()? {
            Found::Entities(ids) => self.ide_entity_receiver(*ids.first()?)?,
            Found::Using(entry, member) => {
                let ty = entry.member_type()?;
                IdeReceiver::Value(self.ide_field_type(ty, member)?)
            }
        };
        for name in rest {
            recv = match recv {
                IdeReceiver::Value(t) => IdeReceiver::Value(self.ide_field_type(t, *name)?),
                IdeReceiver::Type(t) => {
                    // A type declared in the struct (`Outer.Inner`) is the receiver; else
                    // `Enum.MEMBER.` or a struct constant: the value has the type itself.
                    let nested = self
                        .struct_scope_entities(self.ide_deref(t), *name)
                        .first()
                        .and_then(|&id| self.ide_entity_receiver(id));
                    match nested {
                        Some(r @ IdeReceiver::Type(_)) => r,
                        _ => IdeReceiver::Value(t),
                    }
                }
                IdeReceiver::Module(m) => match self.module_member_lookup(m, *name).ok()? {
                    Found::Entities(ids) => self.ide_entity_receiver(*ids.first()?)?,
                    Found::Using(entry, member) => {
                        IdeReceiver::Value(self.ide_field_type(entry.member_type()?, member)?)
                    }
                },
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
    pub(super) fn ide_deref(&self, mut t: TypeId) -> TypeId {
        for _ in 0..8 {
            t = match self.ty.types.kind(t) {
                TypeKind::Pointer(inner) => *inner,
                TypeKind::Distinct(d) => self.ty.types.distincts[d.0 as usize].base,
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
        match self.ty.types.kind(t) {
            TypeKind::Struct(s) => {
                for f in self.ty.types.struct_info(*s).fields.clone().iter() {
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
                "data" => Some(match self.ty.types.kind(t).clone() {
                    TypeKind::Array {
                        elem, ..
                    } => self.ty.types.pointer(elem),
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
            IdeReceiver::Module(m) => {
                return self.ide_module_names(m, &mut HashSet::default(), true);
            }
            IdeReceiver::Type(t) => {
                if let TypeKind::Enum(e) = self.ty.types.kind(t) {
                    let info = self.ty.types.enum_info(*e);
                    let strict = info.loose_of.map_or(*e, |t| match self.ty.types.kind(t) {
                        TypeKind::Enum(strict) => *strict,
                        _ => *e,
                    });
                    for (n, v) in &info.members {
                        out.push(IdeName {
                            name: n.to_string(),
                            kind: IdeKind::EnumMember,
                            detail: format!("{} = {v}", info.name),
                            ty: Some(t),
                            decl: self
                                .ed
                                .facts()
                                .and_then(|ide| ide.enum_decls.get(&(strict, *n)).copied())
                                .unwrap_or(Span::NONE),
                            call: None,
                        });
                    }
                }
                // Struct constants live in the struct's scope.
                let scopes: Vec<ScopeId> = (0..self.ns.scopes.len())
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
                match self.ty.types.kind(t).clone() {
                    TypeKind::Struct(s) => {
                        let fields = self.ty.types.struct_info(s).fields.clone();
                        for f in fields.iter() {
                            if let Some(n) = f.name {
                                out.push(IdeName {
                                    name: n.to_string(),
                                    kind: IdeKind::Field,
                                    detail: self.ty.types.name(f.ty),
                                    decl: f.span,
                                    call: None,
                                    ty: Some(f.ty),
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
                        let elem = self.ty.types.name(elem);
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

pub(super) fn field(name: &str, detail: &str) -> IdeName {
    IdeName {
        name: name.into(),
        kind: IdeKind::Field,
        detail: detail.into(),
        decl: Span::NONE,
        call: None,
        ty: None,
    }
}

/// First line of a snippet, bounded.
pub(super) fn clip(text: &str) -> String {
    let line = text.lines().next().unwrap_or("").trim();
    let line = line.trim_end_matches('{').trim_end();
    if line.len() > 160 {
        let end = line.floor_char_boundary(160);
        format!("{}…", &line[..end])
    } else {
        line.to_string()
    }
}
