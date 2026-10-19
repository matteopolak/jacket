//! Declared types, token classes and reference targets.
use super::{IdeClass, Target};
use crate::sema::ide::{IdeRef, IdeWhat};
use crate::sema::scope::{EntityId, EntityKind, EntityState, Resolved};
use crate::sema::{Compiler, FileId, Span, Value, ast, scope};
use crate::types::TypeKind;

impl Compiler {
    /// Inferred types of declarations `x := value` in `file`: (name span, type).
    pub fn ide_declared_types(&self, file: FileId) -> Vec<(Span, String)> {
        let Some(ide) = self.ide.as_ref() else {
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
            let name = self.types.name(ty);
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
                    use ast::ExprKind as E;
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
        let pending: Vec<EntityId> = match self.ide.as_ref() {
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
        let Some(ide) = self.ide.as_ref() else {
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
                IdeWhat::Member(name) => match self.types.kind(r.ty) {
                    TypeKind::Enum(en)
                        if self
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
        self.ide
            .as_ref()?
            .refs
            .iter()
            .filter(|r| r.span.file == file && r.span.start <= offset && offset <= r.span.end)
            .min_by_key(|r| r.span.end - r.span.start)
            .cloned()
    }

    pub(super) fn ide_targets(&self, r: &IdeRef) -> Vec<Target> {
        match &r.what {
            IdeWhat::Entity(e) => match &self.entity(*e).state {
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
                }) => vec![Target::Type(*t)],
                EntityState::Done(Resolved::Module(m)) => vec![Target::Module(*m)],
                _ => match &self.entity(*e).kind {
                    EntityKind::Const {
                        value: Value::Type(t),
                        ..
                    } => vec![Target::Type(*t)],
                    EntityKind::Local {
                        ..
                    }
                    | EntityKind::Const {
                        ..
                    }
                    | EntityKind::UnboundGenericParameter => vec![Target::Declared(
                        self.ide_entity_name_span(*e)
                            .unwrap_or(self.entity(*e).span),
                        self.entity(*e).name,
                    )],
                    _ => vec![Target::Entity(*e)],
                },
            },
            IdeWhat::Procs(ps) => ps.iter().map(|&p| Target::Proc(p)).collect(),
            IdeWhat::Type(t) => vec![Target::Type(*t)],
            IdeWhat::Module(m) => vec![Target::Module(*m)],
            IdeWhat::Member(_) => Vec::new(),
        }
    }
}
