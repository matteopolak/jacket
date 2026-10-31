//! Selected generic specializations and identities portable between compiler analyses.
use crate::ide::Recorder;
use crate::sema::scope::{EntityKind, EntityState, Resolved};
use crate::sema::{Compiler, ProcId, ScopeId, Value};
use crate::source::Span;
use crate::types::{TypeId, TypeKind};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug)]
pub struct IdeSpecialization {
    /// Resolved bindings, including nominal declaration and module-instance identities.
    pub identity: String,
    pub bindings: String,
    /// Disambiguates equal labels without using compiler-local type numbers.
    pub detail: String,
}

#[derive(Clone, Debug)]
pub struct IdePolymorph {
    pub span: Span,
    pub name: String,
    pub instances: Vec<IdeSpecialization>,
}

impl Compiler {
    pub(crate) fn ide_freeze_polymorphs(&mut self) {
        if self.ed.facts().is_some() {
            let rows = self.ide_polymorph_summaries();
            self.ed.facts_mut().unwrap().polymorph_summaries = Some(rows);
        }
    }

    fn ide_polymorph_detail(&self, id: ProcId) -> String {
        let mut origins = BTreeSet::new();
        if let Some(scope) = self.proc(id).bindings {
            for ids in self.scope(scope).names.values() {
                for &entity in ids {
                    if let EntityKind::Const {
                        value: Value::Type(t),
                        ..
                    } = self.entity(entity).kind
                    {
                        self.ide_type_origins(t, &mut origins);
                    }
                }
            }
        }
        for (name, value, _) in self
            .ns
            .modules
            .get(self.scope(self.proc(id).scope).module.0 as usize)
            .into_iter()
            .flat_map(|m| &m.params)
        {
            origins.insert(format!("{name} = {}", value.render(&self.ty.types, true)));
        }
        let bindings = self.ide_bindings_text(id);
        if origins.is_empty() {
            bindings
        } else {
            format!(
                "{bindings} [{}]",
                origins.into_iter().collect::<Vec<_>>().join("; ")
            )
        }
    }

    fn ide_type_origins(&self, t: TypeId, out: &mut BTreeSet<String>) {
        let span = match self.ty.types.kind(t) {
            TypeKind::Pointer(t)
            | TypeKind::Array {
                elem: t, ..
            } => {
                self.ide_type_origins(*t, out);
                return;
            }
            TypeKind::Struct(s) => Some(self.ty.types.struct_info(*s).span),
            TypeKind::Enum(_) | TypeKind::Distinct(_) => {
                self.ty.nominal_origins.get(&t).map(|(s, _)| *s)
            }
            _ => None,
        };
        if let Some(span) = span.filter(|s| *s != Span::NONE) {
            let source = self.src.sources.get(span.file);
            out.insert(format!("{}:{}", source.path, source.line_col(span.start).0));
        }
    }

    /// A selected call or procedure value, never a trial instantiation or sketch.
    pub(crate) fn ide_note_polymorph(&mut self, id: ProcId) {
        if self.ed.sketching() || self.proc(id).bindings.is_none() {
            return;
        }
        if let Some(facts) = self.ed.facts_mut() {
            facts.selected_polymorphs.insert(id);
        }
    }

    /// The bodies of the polymorphic procedures the program declares (not their instances).
    /// What they contain depends on the types a caller picks, so a branch no instance took
    /// is not dead.
    pub fn ide_polymorphic_bodies(&self) -> Vec<Span> {
        self.pr
            .procs
            .iter()
            .filter(|p| p.is_poly && p.bindings.is_none())
            .filter_map(|p| p.lit.body.as_ref().map(|body| body.span))
            .collect()
    }

    /// All declarations, including imported ones, with only their real specializations.
    pub fn ide_polymorph_summaries(&self) -> Vec<IdePolymorph> {
        if let Some(rows) = self.ed.facts().and_then(|f| f.polymorph_summaries.as_ref()) {
            return rows.clone();
        }
        let mut keys = Keys {
            c: self,
            active: BTreeSet::new(),
            types: BTreeMap::new(),
            binding_scopes: self.pr.procs.iter().filter_map(|p| p.bindings).collect(),
        };
        self.pr
            .procs
            .iter()
            .filter(|p| p.is_poly && p.bindings.is_none() && p.span != Span::NONE)
            .map(|p| {
                let mut instances: Vec<_> = p
                    .instances
                    .values()
                    .filter(|id| {
                        self.ed
                            .facts()
                            .is_some_and(|f| f.selected_polymorphs.contains(id))
                    })
                    .map(|&id| IdeSpecialization {
                        identity: keys.procedure(id),
                        bindings: self.ide_bindings_text(id),
                        detail: self.ide_polymorph_detail(id),
                    })
                    .collect();
                instances
                    .sort_by(|a, b| (&a.bindings, &a.identity).cmp(&(&b.bindings, &b.identity)));
                IdePolymorph {
                    span: p.span,
                    name: p.name.to_string(),
                    instances,
                }
            })
            .collect()
    }
}

/// Joins already-built keys into one. Each part is length-prefixed rather than quoted with `{:?}`:
/// quoting a string that holds quoted strings doubles its backslashes at every level, so a key
/// for a type nested n pointers deep (`*****T`) would be 2^n bytes long.
fn join<S: AsRef<str>>(parts: impl IntoIterator<Item = S>) -> String {
    let mut out = String::from("(");
    for part in parts {
        let part = part.as_ref();
        out.push_str(&part.len().to_string());
        out.push(':');
        out.push_str(part);
        out.push(';');
    }
    out.push(')');
    out
}

/// `join` of an optional key: none and empty differ.
fn join_opt(part: Option<&str>) -> String {
    part.map_or_else(|| "none".to_string(), |p| join([p]))
}

struct Keys<'a> {
    c: &'a Compiler,
    active: BTreeSet<String>,
    types: BTreeMap<TypeId, String>,
    binding_scopes: BTreeSet<ScopeId>,
}

impl Keys<'_> {
    fn location(&self, span: Span) -> String {
        if span == Span::NONE {
            return "builtin".into();
        }
        let source = self.c.src.sources.get(span.file);
        let path = source.path.as_file().map(|p| self.c.src.fs.canonical(p));
        // Generated sources have no globally meaningful path. Their source text prevents
        // equally numbered added strings in different workspaces being conflated.
        join([
            format!("{path:?}"),
            join_opt(path_is_generated(&source.path).then_some(source.text.as_ref())),
            span.start.to_string(),
            span.end.to_string(),
        ])
    }

    fn scope(&mut self, scope: ScopeId) -> String {
        let tag = format!("scope:{}", scope.0);
        if !self.active.insert(tag.clone()) {
            return "recursive-scope".into();
        }
        let module = self.c.ns.modules.get(self.c.scope(scope).module.0 as usize);
        let parameters: Vec<_> = module
            .into_iter()
            .flat_map(|m| &m.params)
            .map(|(n, v, t)| join([n.as_str().to_string(), self.typed(v, *t)]))
            .collect();
        let path = module
            .and_then(|m| m.path.as_ref())
            .map(|p| self.c.src.fs.canonical(p));
        let mut enclosing = Vec::new();
        let mut at = Some(scope);
        while let Some(sid) = at {
            if self.binding_scopes.contains(&sid) {
                enclosing.push(self.bindings(sid));
            }
            at = self.c.scope(sid).parent;
        }
        self.active.remove(&tag);
        let generated_origin = module
            .filter(|_| path.is_none())
            .map(|m| self.location(m.argument_span));
        join([
            format!("{path:?}"),
            join_opt(generated_origin.as_deref()),
            join(parameters),
            join(enclosing),
        ])
    }

    fn bindings(&mut self, scope: ScopeId) -> String {
        let c = self.c;
        let mut values: Vec<_> = c
            .scope(scope)
            .names
            .iter()
            .flat_map(|(n, ids)| {
                ids.iter().filter_map(move |id| {
                    if let EntityKind::Const {
                        value,
                        ty,
                    } = &c.entity(*id).kind
                    {
                        Some((n.as_str(), value, *ty))
                    } else {
                        None
                    }
                })
            })
            .collect();
        values.sort_by_key(|(n, ..)| *n);
        join(
            values
                .into_iter()
                .map(|(n, v, t)| join([n.to_string(), self.typed(v, t)]))
                .collect::<Vec<_>>(),
        )
    }

    fn procedure(&mut self, id: ProcId) -> String {
        let p = self.c.proc(id);
        let tag = format!("proc:{}", id.0);
        if !self.active.insert(tag.clone()) {
            return "recursive-procedure".into();
        }
        let bindings = p.bindings.map(|s| self.bindings(s));
        let key = join([
            self.location(p.span),
            p.name.as_str().to_string(),
            self.scope(p.scope),
            join_opt(bindings.as_deref()),
            format!("{:?}", self.c.options.os),
            format!("{:?}", self.c.options.cpu),
        ]);
        self.active.remove(&tag);
        key
    }

    fn ty(&mut self, ty: TypeId) -> String {
        if let Some(key) = self.types.get(&ty) {
            return key.clone();
        }
        let tag = format!("type:{}", ty.0);
        if !self.active.insert(tag.clone()) {
            return "recursive-type".into();
        }
        let key = match self.c.ty.types.kind(ty) {
            TypeKind::Pointer(t) => format!("ptr:{}", self.ty(*t)),
            TypeKind::Array {
                elem,
                kind,
            } => format!("array:{:?}:{}", kind, self.ty(*elem)),
            TypeKind::Proc(p) => format!(
                "proc:{}",
                join([
                    join(p.params.iter().map(|t| self.ty(*t)).collect::<Vec<_>>()),
                    join(p.returns.iter().map(|t| self.ty(*t)).collect::<Vec<_>>()),
                    format!(
                        "{:?}",
                        (
                            p.variadic,
                            p.c_varargs,
                            p.c_call,
                            p.no_context,
                            p.non_pod_return
                        )
                    ),
                ])
            ),
            TypeKind::Struct(s) => {
                let info = self.c.ty.types.struct_info(*s);
                let source = self.c.ty.struct_asts.get(s);
                let args = source.map(|s| self.bindings(s.scope));
                format!(
                    "struct:{}",
                    join([
                        self.location(info.span),
                        join_opt(source.map(|s| self.scope(s.scope)).as_deref()),
                        join_opt(args.as_deref()),
                    ])
                )
            }
            TypeKind::Enum(_) | TypeKind::Distinct(_) => {
                let origin = self
                    .c
                    .ty
                    .nominal_origins
                    .get(&ty)
                    .map(|(span, scope)| join([self.location(*span), self.scope(*scope)]));
                // Compiler-defined distincts have stable names rather than source locations.
                format!(
                    "nominal:{}",
                    join([
                        format!("{:?}", self.c.ty.types.name(ty)),
                        join_opt(origin.as_deref()),
                    ])
                )
            }
            TypeKind::PolyStruct {
                id, ..
            } => {
                let p = &self.c.ty.poly_structs[id.0 as usize];
                format!(
                    "family:{}",
                    join([self.location(p.lit.span), self.scope(p.scope)])
                )
            }
            primitive => format!("{primitive:?}"),
        };
        self.active.remove(&tag);
        self.types.insert(ty, key.clone());
        key
    }

    fn typed(&mut self, value: &Value, ty: TypeId) -> String {
        join([self.ty(ty), self.value(value)])
    }

    fn value(&mut self, value: &Value) -> String {
        match value {
            Value::Type(t) => format!("type:{}", self.ty(*t)),
            Value::Proc(p) => self.procedure(*p),
            Value::Code(id) => {
                let span = match self.c.pr.codes[id.0 as usize].as_ref() {
                    crate::ast::CodeBody::Expr(e) => e.span,
                    crate::ast::CodeBody::Stmt(s) => s.span,
                    crate::ast::CodeBody::Block(b) => b.span,
                };
                format!("code:{}", self.location(span))
            }
            Value::Bytes(a) => format!(
                "bytes:{}",
                join([
                    format!("{:?}", a.bytes),
                    join(
                        a.relocs
                            .iter()
                            .map(|r| {
                                join([
                                    r.offset.to_string(),
                                    self.relocation(r.target),
                                    r.addend.to_string(),
                                ])
                            })
                            .collect::<Vec<_>>(),
                    ),
                ])
            ),
            scalar => format!("{scalar:?}"),
        }
    }

    fn relocation(&mut self, target: crate::ir::RelocTarget) -> String {
        use crate::ir::{RelocTarget, Storage};
        match target {
            RelocTarget::Func(f) => {
                let proc = self.c.pr.procs.iter().position(|p| {
                    matches!(p.target, Some(crate::sema::procs::ProcTarget::Func(id)) if id == f)
                });
                match proc {
                    Some(i) => self.procedure(ProcId(i as u32)),
                    None => format!(
                        "generated-func:{}",
                        join([format!("{:?}", self.c.out.program.func_names[f.0 as usize])])
                    ),
                }
            }
            RelocTarget::Foreign(f) => {
                let foreign = &self.c.out.program.foreigns[f.0 as usize];
                format!(
                    "foreign:{}",
                    join([
                        format!("{:?}", foreign.symbol),
                        format!(
                            "{:?}",
                            foreign
                                .library
                                .map(|i| &self.c.out.program.libraries[i].name)
                        ),
                    ])
                )
            }
            RelocTarget::Global(g) => {
                if let Some((&ty, _)) = self.c.ty.infos.iter().find(|(_, id)| **id == g) {
                    return format!("info:{}", self.ty(ty));
                }
                if let Some(entity) = self.c.ns.entities.iter().find(|e| {
                    matches!(e.state, EntityState::Done(Resolved::Global { storage: Storage::Data(id), .. }) if id == g)
                }) {
                    return format!(
                        "global:{}",
                        join([self.location(entity.span), self.scope(entity.scope)])
                    );
                }
                let tag = format!("global:{}", g.0);
                if !self.active.insert(tag.clone()) {
                    return "recursive-global".into();
                }
                let global = &self.c.out.program.globals[g.0 as usize];
                let relocs = global
                    .relocs
                    .iter()
                    .map(|r| {
                        join([
                            r.offset.to_string(),
                            self.relocation(r.target),
                            r.addend.to_string(),
                        ])
                    })
                    .collect::<Vec<_>>();
                let key = format!(
                    "image:{}",
                    join([
                        format!("{:?}", global.name),
                        format!("{:?}", global.init),
                        join(relocs),
                    ])
                );
                self.active.remove(&tag);
                key
            }
        }
    }
}

fn path_is_generated(path: &crate::source::SourcePath) -> bool {
    path.as_file().is_none()
}
