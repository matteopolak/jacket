//! Text that describes procedures, types and entities.
use super::completion::clip;
use super::facts::{IdeCall, IdeKind, IdeName};
use crate::ast;
use crate::intern::Sym;
use crate::sema::scope::{EntityId, EntityKind, EntityState, Resolved};
use crate::sema::value::ProcId;
use crate::sema::{Compiler, scope, value};
use crate::types::TypeId;
use crate::types::TypeKind;

/// How many fields a written-out type shows.
const HOVER_FIELDS: usize = 32;

impl Compiler {
    fn ide_proc_header(&self, name: Sym, p: ProcId) -> String {
        format!("{name} :: {}", clip(&self.ide_header_text(p)))
    }

    /// A procedure's header as written, less the parameters `#bake_arguments` fixed.
    pub(crate) fn ide_header_text(&self, p: ProcId) -> String {
        let header = &self.proc(p).lit.header;
        let mut text = self.ide_snippet(header.span).to_string();
        // A `#modify` block is code, not signature: the header names it without its body.
        if let Some(modify) = &header.modify {
            let start = (modify.span.start.saturating_sub(header.span.start)) as usize;
            let end = (modify.span.end.saturating_sub(header.span.start)) as usize;
            if start < end
                && end <= text.len()
                && text.is_char_boundary(start)
                && text.is_char_boundary(end)
            {
                let rest = text[end..].to_string();
                text.truncate(start);
                text = format!("{}{rest}", text.trim_end());
            }
        }
        let text = text.as_str();
        if !header.baked {
            return text.to_string();
        }
        // The parameter list is the first parenthesized group; the rest follows it.
        let mut depth = 0;
        let close = text.char_indices().find_map(|(i, c)| {
            match c {
                '(' => depth += 1,
                ')' if depth == 1 => return Some(i),
                ')' => depth -= 1,
                _ => {}
            }
            None
        });
        let Some(close) = close else {
            return text.to_string();
        };
        let open = text.find('(').unwrap_or(0);
        let params: Vec<&str> = header
            .params
            .iter()
            .map(|param| self.ide_snippet(param.span).trim())
            .collect();
        format!(
            "{}({}){}",
            &text[..open],
            params.join(", "),
            &text[close + 1..]
        )
    }

    /// A value's type for hover text: a struct (or union) is written out as its fields, each
    /// under the doc comment of its declaration, since the name alone may say little (an
    /// instance whose long string argument was cut short, a body `#insert` wrote).
    pub(super) fn ide_value_type(&mut self, t: TypeId) -> String {
        let short = self.ty.types.name(t);
        let TypeKind::Struct(s) = *self.ty.types.kind(t) else {
            return short;
        };
        self.ide_laid_out(t);
        let info = self.ty.types.struct_info(s);
        // A cut name says less than the fields under it: the polymorphic struct's name heads them.
        let head = if short == self.ty.types.full_name(t) {
            short
        } else {
            self.ty
                .types
                .struct_info(info.poly_parent.unwrap_or(s))
                .name
                .to_string()
        };
        let fields = info.fields.clone();
        if fields.iter().all(|f| f.name.is_none()) {
            return head;
        }
        let mut text = format!("{head} {{\n");
        for f in fields.iter().take(HOVER_FIELDS) {
            let Some(name) = f.name else {
                continue;
            };
            let doc = self
                .src
                .sources
                .try_get(f.span.file)
                .and_then(|file| crate::docs::declaration(&file.text, f.span.start as usize));
            for line in doc.iter().flat_map(|d| d.lines()) {
                text.push_str(&format!("    /// {line}\n"));
            }
            text.push_str(&format!("    {name}: {};\n", self.ty.types.name(f.ty)));
        }
        if fields.len() > HOVER_FIELDS {
            text.push_str("    ...\n");
        }
        text.push('}');
        text
    }

    pub(super) fn ide_type_hover(&mut self, name: &str, t: TypeId) -> String {
        match self.ty.types.kind(t) {
            TypeKind::Struct(s) => {
                // A polymorphic instance may not have its fields yet when nothing has used it.
                let s = *s;
                self.ide_laid_out(t);
                let info = self.ty.types.struct_info(s);
                let fields: Vec<String> = info
                    .fields
                    .iter()
                    .filter_map(|f| Some(format!("{}: {};", f.name?, self.ty.types.name(f.ty))))
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
                let info = self.ty.types.enum_info(*e);
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
                    self.ty.types.name(info.base),
                    members.join(" ")
                )
            }
            _ => {
                let full = self.ty.types.name(t);
                if full == name {
                    format!("{name} :: Type")
                } else {
                    format!("{name} :: {full}")
                }
            }
        }
    }

    pub(super) fn ide_entity_hover(&mut self, id: EntityId, ty: Option<TypeId>) -> String {
        let name = self.entity(id).name;
        let resolved = match self.entity(id).kind.clone() {
            EntityKind::Local {
                ty, ..
            } => return format!("{name}: {}", self.ide_value_type(ty)),
            EntityKind::Const {
                value: value::Value::Type(t),
                ..
            } => return self.ide_type_hover(name.as_str(), t),
            EntityKind::Const {
                ty, ..
            } => return format!("{name} :: {}", self.ide_value_type(ty)),
            EntityKind::Placeholder => return format!("{name} :: #placeholder"),
            EntityKind::Builtin(_) => {
                return format!(
                    "{name}: {}",
                    self.ty.types.name(ty.unwrap_or(TypeId::COMPILE_TIME))
                );
            }
            _ => self.resolve_entity(id).ok(),
        };
        match resolved {
            Some(Resolved::Const {
                value: value::Value::Type(t),
                ..
            }) => self.ide_type_hover(name.as_str(), t),
            Some(
                Resolved::Const {
                    value: value::Value::Proc(p),
                    ..
                }
                | Resolved::Proc(p),
            ) => self.ide_proc_header(name, p),
            Some(Resolved::ProcSet(ps)) => ps
                .iter()
                .map(|&p| self.ide_proc_header(self.proc(p).name, p))
                .collect::<Vec<_>>()
                .join("\n"),
            // A pointer constant holds an address from compile-time execution, different in every
            // session and meaningless to the reader: only `null` is shown.
            Some(Resolved::Const {
                ty,
                value,
            }) if self.ty.types.is_pointer(ty) && matches!(value, value::Value::Int(_)) => {
                let null = if matches!(value, value::Value::Int(0)) {
                    " = null"
                } else {
                    ""
                };
                format!("{name} :: {}{null}", self.ty.types.name(ty))
            }
            Some(Resolved::Const {
                ty,
                value,
            }) => format!(
                "{name} :: {} = {}",
                self.ty.types.name(ty),
                value.render(&self.ty.types, true)
            ),
            Some(Resolved::Global {
                ty, ..
            }) => format!("{name}: {}", self.ty.types.name(ty)),
            Some(Resolved::Module(m)) => {
                format!(
                    "{name} :: #import \"{}\"",
                    self.ns.modules[m.0 as usize].name
                )
            }
            Some(Resolved::Library(_)) => format!("{name} :: #library"),
            Some(Resolved::PolyStruct(_)) => format!("{name} :: struct (polymorphic)"),
            // A constant whose `#run` failed still has the type the run returns.
            None if let EntityKind::Decl {
                decl, ..
            } = &self.entity(id).kind
                && decl.kind == ast::DeclKind::Const
                && let Some(value) = decl.value.clone()
                && let home = self.entity(id).home
                && let Some(t) = self.failed_run_type(home, &value, 8) =>
            {
                format!("{name} :: {}", self.ty.types.name(t))
            }
            None => format!(
                "{name}: {}",
                self.ty.types.name(ty.unwrap_or(TypeId::COMPILE_TIME))
            ),
        }
    }

    /// The declaration to describe a name by: a procedure if the name has overloads (an alias
    /// like `print :: print_to_builder;` may come first).
    pub(super) fn ide_pick(&self, ids: impl Iterator<Item = EntityId>) -> Option<EntityId> {
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
    pub(super) fn ide_entity_name(&self, id: EntityId) -> Option<IdeName> {
        let e = self.entity(id);
        let name = e.name.as_str();
        if name.is_empty() || name.starts_with(['\0', '`']) || name.contains(' ') {
            return None;
        }
        let mut call = None;
        let mut ty = None;
        let (kind, detail) = match &e.kind {
            EntityKind::EnumMember {
                ty: t, ..
            } => {
                ty = Some(*t);
                (IdeKind::Constant, self.ty.types.name(*t))
            }
            EntityKind::Local {
                ty: t, ..
            } => {
                ty = Some(*t);
                (IdeKind::Variable, self.ty.types.name(*t))
            }
            EntityKind::Const {
                value: value::Value::Type(_),
                ..
            } => (IdeKind::Type, "Type".into()),
            EntityKind::Const {
                ty: t, ..
            } => {
                ty = Some(*t);
                (IdeKind::Constant, self.ty.types.name(*t))
            }
            EntityKind::Placeholder | EntityKind::UnboundGenericParameter => return None,
            EntityKind::Import(_) => (IdeKind::Module, "module".into()),
            EntityKind::Builtin(b) => match b {
                scope::Builtin::Type(_) => (IdeKind::Type, "builtin type".into()),
                scope::Builtin::Proc(_) => {
                    call = Some(IdeCall::unknown());
                    (IdeKind::Function, "builtin".into())
                }
                scope::Builtin::TargetConstant(_) => (IdeKind::Constant, "builtin constant".into()),
            },
            EntityKind::Decl {
                decl, ..
            } => match &e.state {
                EntityState::Done(
                    Resolved::Proc(p)
                    | Resolved::Const {
                        value: value::Value::Proc(p),
                        ..
                    },
                ) => {
                    call = Some(IdeCall::of_headers([&*self.proc(*p).lit.header]));
                    (IdeKind::Function, clip(&self.ide_header_text(*p)))
                }
                EntityState::Done(Resolved::ProcSet(procedures)) => {
                    call = Some(IdeCall::of_headers(
                        procedures.iter().map(|p| &*self.proc(*p).lit.header),
                    ));
                    (
                        IdeKind::Function,
                        procedures
                            .iter()
                            .map(|p| clip(&self.ide_header_text(*p)))
                            .collect::<Vec<_>>()
                            .join(" | "),
                    )
                }
                EntityState::Done(Resolved::Global {
                    ty: t, ..
                }) => {
                    ty = Some(*t);
                    (IdeKind::Variable, self.ty.types.name(*t))
                }
                EntityState::Done(Resolved::Const {
                    value: value::Value::Type(_),
                    ..
                }) => (IdeKind::Type, "Type".into()),
                EntityState::Done(Resolved::Const {
                    ty: t, ..
                }) => {
                    ty = Some(*t);
                    (IdeKind::Constant, self.ty.types.name(*t))
                }
                EntityState::Done(Resolved::Module(_)) => (IdeKind::Module, "module".into()),
                _ => {
                    use crate::ast::ExprKind as E;
                    let kind = match (decl.kind, decl.value.as_ref().map(|v| &v.kind)) {
                        (_, Some(E::Proc(_))) => IdeKind::Function,
                        (_, Some(E::Struct(_) | E::Enum(_) | E::ProcType(_))) => IdeKind::Type,
                        (ast::DeclKind::Var, _) => IdeKind::Variable,
                        _ => IdeKind::Constant,
                    };
                    if let Some(E::Proc(lit)) = decl.value.as_ref().map(|v| &v.kind) {
                        call = Some(IdeCall::of_headers([&*lit.header]));
                    }
                    let detail = match decl.value.as_ref().map(|v| &v.kind) {
                        Some(E::Proc(lit)) => clip(self.src.sources.snippet(lit.header.span)),
                        _ => clip(self.src.sources.snippet(e.span)),
                    };
                    (kind, detail)
                }
            },
        };
        // Overloads are separate declarations of one name; the call must suit them all.
        if kind == IdeKind::Function
            && let Some(ids) = self.scope(e.scope).names.get(&e.name)
            && ids.len() > 1
        {
            let headers: Vec<&ast::ProcHeader> = ids
                .iter()
                .filter_map(|&i| match &self.entity(i).kind {
                    EntityKind::Decl {
                        decl, ..
                    } => match decl.value.as_ref().map(|v| &v.kind) {
                        Some(ast::ExprKind::Proc(lit)) => Some(&*lit.header),
                        _ => None,
                    },
                    _ => None,
                })
                .collect();
            if headers.len() > 1 {
                call = Some(IdeCall::of_headers(headers));
            }
        }
        Some(IdeName {
            docs: None,
            name: name.to_string(),
            kind,
            detail,
            decl: e.span,
            call,
            ty,
        })
    }
}
