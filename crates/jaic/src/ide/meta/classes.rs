//! Declared types, token classes and reference targets.
use super::{IdeClass, Target};
use crate::ast;
use crate::ide::{IdeRef, IdeWhat};
use crate::sema::scope::{EntityId, EntityKind, EntityState, Resolved};
use crate::sema::{Compiler, Value, scope};
use crate::source::FileId;
use crate::source::Span;
use crate::types::{TypeId, TypeKind, Types};

impl Compiler {
    /// Inferred types of declarations `x := value` in `file`: (name span, type), spelled in
    /// full so an edit can write them into the code.
    pub fn ide_declared_types(&self, file: FileId) -> Vec<(Span, String)> {
        self.declared_types(file, Types::full_name)
    }

    /// [`Self::ide_declared_types`] spelled to label the code, as inlay hints do: a type whose
    /// name would be cut short is named by its polymorphic struct alone.
    pub fn ide_declared_type_labels(&self, file: FileId) -> Vec<(Span, String)> {
        self.declared_types(file, Types::label)
    }

    fn declared_types(
        &self,
        file: FileId,
        spell: fn(&Types, TypeId) -> String,
    ) -> Vec<(Span, String)> {
        let Some(ide) = self.ed.facts() else {
            return Vec::new();
        };
        let mut out: Vec<(Span, String)> = Vec::new();
        let mut conflicting = Vec::new();
        for r in ide.refs.iter().filter(|r| r.decl && r.span.file == file) {
            let IdeWhat::Entity(e) = r.what else {
                continue;
            };
            let EntityKind::Local {
                ty, ..
            } = self.entity(e).kind
            else {
                continue;
            };
            let name = spell(&self.ty.types, ty);
            match out.iter().find(|(s, _)| *s == r.span) {
                Some((_, n)) if *n != name => conflicting.push(r.span),
                Some(_) => {}
                None => out.push((r.span, name)),
            }
        }
        out.retain(|(s, _)| !conflicting.contains(s));
        out
    }

    fn ide_class_of_entity(&self, id: EntityId) -> Option<IdeClass> {
        let e = self.entity(id);
        Some(match &e.kind {
            EntityKind::Local {
                ..
            } => IdeClass::Variable,
            EntityKind::Const {
                value: Value::Type(_),
                ..
            } => IdeClass::Type,
            EntityKind::Const {
                ..
            }
            | EntityKind::EnumMember {
                ..
            } => IdeClass::Constant,
            EntityKind::Import(_) => IdeClass::Module,
            EntityKind::Builtin(b) => match b {
                scope::Builtin::Type(_) => IdeClass::Type,
                scope::Builtin::Proc(_) => IdeClass::Function,
                scope::Builtin::TargetConstant(_) => IdeClass::Constant,
            },
            EntityKind::Placeholder | EntityKind::UnboundGenericParameter => return None,
            EntityKind::Decl {
                decl, ..
            } => match &e.state {
                EntityState::Done(
                    Resolved::Proc(p)
                    | Resolved::Const {
                        value: Value::Proc(p),
                        ..
                    },
                ) => {
                    if self.proc(*p).is_macro {
                        IdeClass::Macro
                    } else {
                        IdeClass::Function
                    }
                }
                EntityState::Done(Resolved::ProcSet(_)) => IdeClass::Function,
                EntityState::Done(Resolved::Global {
                    ..
                }) => IdeClass::Variable,
                EntityState::Done(
                    Resolved::Const {
                        value: Value::Type(_),
                        ..
                    }
                    | Resolved::PolyStruct(_),
                ) => IdeClass::Type,
                EntityState::Done(Resolved::Const {
                    ..
                }) => IdeClass::Constant,
                EntityState::Done(Resolved::Module(_) | Resolved::Library(_)) => IdeClass::Module,
                _ => {
                    use crate::ast::ExprKind as E;
                    match (decl.kind, decl.value.as_ref().map(|v| &v.kind)) {
                        (_, Some(E::Proc(lit))) if lit.header.flags.expand() => IdeClass::Macro,
                        (_, Some(E::Proc(_))) => IdeClass::Function,
                        (_, Some(E::Struct(_) | E::Enum(_) | E::ProcType(_))) => IdeClass::Type,
                        (ast::DeclKind::Var, _) => IdeClass::Variable,
                        _ => IdeClass::Constant,
                    }
                }
            },
        })
    }

    /// How to color each recorded identifier of `file`. Declarations the checker has not needed
    /// yet are resolved first: an alias such as `Mod :: #import "Module";` is a module only once
    /// it is, so the color would otherwise depend on which requests were made before.
    pub fn ide_classes(&mut self, file: FileId) -> Vec<(Span, IdeClass)> {
        let pending: Vec<EntityId> = match self.ed.facts() {
            Some(ide) => ide
                .refs
                .iter()
                .filter(|r| r.span.file == file)
                .filter_map(|r| match r.what {
                    IdeWhat::Entity(e)
                        if matches!(self.entity(e).state, EntityState::Unresolved) =>
                    {
                        Some(e)
                    }
                    _ => None,
                })
                .collect(),
            None => return Vec::new(),
        };
        for id in pending {
            let _ = self.resolve_entity(id);
        }
        let Some(ide) = self.ed.facts() else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for r in ide.refs.iter().filter(|r| r.span.file == file) {
            let class = match &r.what {
                IdeWhat::Entity(e) => match self.ide_class_of_entity(*e) {
                    Some(c) => c,
                    None => continue,
                },
                IdeWhat::Procs(ps) => {
                    if !ps.is_empty() && ps.iter().all(|&p| self.proc(p).is_macro) {
                        IdeClass::Macro
                    } else {
                        IdeClass::Function
                    }
                }
                IdeWhat::Type(_) => IdeClass::Type,
                IdeWhat::Module(_) => IdeClass::Module,
                IdeWhat::Member(name) => match self.ty.types.kind(r.ty) {
                    TypeKind::Enum(en)
                        if self
                            .ty
                            .types
                            .enum_info(*en)
                            .members
                            .iter()
                            .any(|(m, _)| m == name) =>
                    {
                        IdeClass::EnumMember
                    }
                    _ => IdeClass::Field,
                },
            };
            out.push((r.span, class));
        }
        out
    }

    pub(super) fn ide_ref_at(&self, file: FileId, offset: u32) -> Option<IdeRef> {
        self.ed
            .facts()?
            .refs
            .iter()
            .filter(|r| r.span.file == file && r.span.start <= offset && offset <= r.span.end)
            .min_by_key(|r| r.span.end - r.span.start)
            .cloned()
    }

    pub(super) fn ide_targets(&self, r: &IdeRef) -> Vec<Target> {
        match &r.what {
            // A type written by a name: what the name declares (`T` bound to `int` is not every
            // `int`).
            IdeWhat::Type(_)
                if let Some(&e) = self.ed.facts().and_then(|i| i.type_names.get(&r.span)) =>
            {
                self.ide_entity_targets(e)
            }
            IdeWhat::Entity(e) => self.ide_entity_targets(*e),
            IdeWhat::Procs(ps) => ps.iter().map(|&p| Target::Proc(p)).collect(),
            IdeWhat::Type(t) => vec![Target::Type(*t)],
            IdeWhat::Module(m) => vec![Target::Module(*m)],
            IdeWhat::Member(_) => Vec::new(),
        }
    }

    /// Whether constant `e` holding type `t` is the declaration of a struct or enum
    /// (`Item :: struct`), rather than another name for a type (`T` bound to it, an alias).
    fn ide_declares_type(&self, e: EntityId, t: crate::types::TypeId) -> bool {
        // A built-in name (`int`) is declared nowhere: its uses are the type's.
        if self.ide_entity_name_span(e).is_none() {
            return true;
        }
        if self.ed.facts().is_some_and(|i| i.stand_ins.contains(&t)) {
            return false;
        }
        let name = self.entity(e).name;
        match self.ty.types.kind(t) {
            TypeKind::Struct(s) => self.ty.types.struct_info(*s).name == name,
            TypeKind::Enum(en) => self.ty.types.enum_info(*en).name == name,
            _ => false,
        }
    }

    fn ide_entity_targets(&self, e: EntityId) -> Vec<Target> {
        let declared = || {
            vec![Target::Declared(
                self.ide_entity_name_span(e).unwrap_or(self.entity(e).span),
                self.entity(e).name,
            )]
        };
        let e = &e;
        match &self.entity(*e).state {
            EntityState::Done(
                Resolved::Proc(p)
                | Resolved::Const {
                    value: Value::Proc(p),
                    ..
                },
            ) => vec![Target::Proc(*p)],
            EntityState::Done(Resolved::Const {
                value: Value::Type(t),
                ..
            }) if self.ide_declares_type(*e, *t) => vec![Target::Type(*t)],
            EntityState::Done(Resolved::Module(m)) => vec![Target::Module(*m)],
            _ => match &self.entity(*e).kind {
                EntityKind::Const {
                    value: Value::Type(t),
                    ..
                } if self.ide_declares_type(*e, *t) => vec![Target::Type(*t)],
                // A name an enum borrows with `using` is the member it was borrowed from.
                EntityKind::EnumMember {
                    initializer: crate::sema::enums::Initializer::Alias(original),
                    ..
                } => self.ide_entity_targets(*original),
                EntityKind::Local {
                    ..
                }
                | EntityKind::Const {
                    ..
                }
                | EntityKind::UnboundGenericParameter => declared(),
                _ => vec![Target::Entity(*e)],
            },
        }
    }
}
