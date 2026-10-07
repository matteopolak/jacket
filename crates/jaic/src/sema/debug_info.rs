//! Debug information for native builds: what the LLVM backend needs to describe
//! variables, lexical scopes and types to a debugger (`docs/native/debug-info.md`).
//!
//! Recording is off unless `Options::debug_info` is set (by `jaic build`), so the
//! interpreter-only paths pay nothing. Procedures get an `ir::FuncDebug` side table;
//! the referenced types are converted to `ir::DebugType`s once, when the program is
//! about to be written (`collect_debug_types`).
use super::lower::FnCtx;
use super::scope::{EntityKind, ScopeId, ScopeKind};
use super::{Compiler, ProcId};
use crate::intern::Sym;
use crate::ir::{self, DebugField, DebugType, DebugTypeKind};
use crate::source::Span;
use crate::types::{ArrayKind, LayoutState, TypeId, TypeKind};

impl Compiler {
    /// Start the debug side table of a procedure body being lowered.
    pub(super) fn begin_func_debug(&mut self, f: &mut FnCtx, id: ProcId, span: Span) {
        if !self.options.debug_info {
            return;
        }
        let (line, _) = self.sources.get(span.file).line_col(span.start);
        let name = self.proc(id).name.to_string();
        f.b.func.debug = Some(Box::new(ir::FuncDebug {
            name,
            file: span.file.0,
            line,
            scopes: vec![ir::DebugScope {
                parent: 0,
                line,
                col: 0,
            }],
            vars: Vec::new(),
        }));
    }

    /// The debug scope index of sema scope `scope` in the procedure being lowered,
    /// creating lexical scopes for it and its enclosing blocks as needed. Block and macro
    /// scopes become lexical blocks; the procedure scope and anything outside it is 0.
    pub(super) fn debug_scope(&self, f: &mut FnCtx, scope: ScopeId, line: u32, col: u32) -> u32 {
        let Some(debug) = f.b.func.debug.as_mut() else {
            return 0;
        };
        // Walk out to a scope that already has an index, collecting the new ones.
        let mut chain = Vec::new();
        let mut cur = Some(scope);
        let mut parent = 0;
        while let Some(s) = cur {
            if let Some(&i) = f.debug_scopes.get(&s) {
                parent = i;
                break;
            }
            let info = self.scope(s);
            if !matches!(info.kind, ScopeKind::Block | ScopeKind::Macro) {
                break;
            }
            chain.push(s);
            cur = info.parent;
        }
        for s in chain.into_iter().rev() {
            debug.scopes.push(ir::DebugScope {
                parent,
                line,
                col,
            });
            parent = debug.scopes.len() as u32 - 1;
            f.debug_scopes.insert(s, parent);
        }
        parent
    }

    /// Record a named local variable (or parameter, `arg` > 0) living at `addr`.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn debug_var(
        &self,
        f: &mut FnCtx,
        scope: ScopeId,
        name: Sym,
        span: Span,
        ty: TypeId,
        addr: ir::Val,
        arg: u32,
    ) {
        if f.b.func.debug.is_none() || name.as_str().starts_with('\0') || name.as_str() == "_" {
            return;
        }
        let (line, col) = self.sources.get(span.file).line_col(span.start);
        let scope = if arg > 0 {
            0
        } else {
            self.debug_scope(f, scope, line, col)
        };
        if let Some(debug) = f.b.func.debug.as_mut() {
            debug.vars.push(ir::DebugVar {
                name: name.to_string(),
                ty: ty.0,
                addr,
                arg,
                scope,
                line,
                col,
            });
        }
    }

    /// `debug_var` for a `Local` entity kind (loop variables are declared as kinds).
    pub(super) fn debug_local_kind(
        &self,
        f: &mut FnCtx,
        scope: ScopeId,
        name: Sym,
        span: Span,
        kind: &EntityKind,
    ) {
        if let EntityKind::Local {
            ty,
            addr,
            ..
        } = kind
        {
            self.debug_var(f, scope, name, span, *ty, *addr, 0);
        }
    }

    /// Record a program global variable.
    pub(super) fn debug_global(&mut self, global: ir::GlobalId, name: Sym, ty: TypeId, span: Span) {
        if !self.options.debug_info {
            return;
        }
        let (line, _) = self.sources.get(span.file).line_col(span.start);
        self.program.debug_globals.push(ir::DebugGlobal {
            global,
            name: name.to_string(),
            ty: ty.0,
            file: span.file.0,
            line,
        });
    }

    /// Describe every type the recorded debug information names (idempotent), and make
    /// sure `Program::file_paths` names every source file.
    pub fn collect_debug_types(&mut self) {
        if self.program.file_paths.len() < self.sources.len() {
            self.program.file_paths = (0..self.sources.len())
                .map(|i| {
                    self.sources
                        .get(crate::source::FileId(i as u32))
                        .path
                        .clone()
                })
                .collect();
        }
        let mut pending: Vec<u32> = Vec::new();
        for func in self.program.funcs.iter().flatten() {
            if let Some(debug) = &func.debug {
                pending.extend(debug.vars.iter().map(|v| v.ty));
            }
        }
        pending.extend(self.program.debug_globals.iter().map(|g| g.ty));
        while let Some(key) = pending.pop() {
            if self.program.debug_types.contains_key(&key) {
                continue;
            }
            let ty = self.describe_debug_type(key);
            if let DebugTypeKind::Struct {
                fields, ..
            } = &ty.kind
            {
                pending.extend(fields.iter().map(|f| f.ty));
            }
            match ty.kind {
                DebugTypeKind::Pointer(t)
                | DebugTypeKind::Array {
                    elem: t, ..
                }
                | DebugTypeKind::Enum {
                    base: t, ..
                }
                | DebugTypeKind::Typedef(t) => pending.push(t),
                _ => {}
            }
            self.program.debug_types.insert(key, ty);
        }
    }

    fn describe_debug_type(&mut self, key: u32) -> DebugType {
        let simple = |name: &str, size: u64, kind: DebugTypeKind| DebugType {
            name: name.into(),
            size,
            align: size.max(1),
            kind,
        };
        if key == ir::DEBUG_CHAR {
            return simple("u8", 1, DebugTypeKind::Char);
        }
        if key == ir::DEBUG_CHAR_PTR {
            return simple("", 8, DebugTypeKind::Pointer(ir::DEBUG_CHAR));
        }
        let ty = TypeId(key);
        let name = self.types.name(ty);
        let kind = match self.types.kind(ty).clone() {
            TypeKind::Bool => DebugTypeKind::Bool,
            TypeKind::Int {
                signed, ..
            } => DebugTypeKind::Int {
                signed,
            },
            TypeKind::Float {
                ..
            }
            | TypeKind::WideFloat(_) => DebugTypeKind::Float,
            TypeKind::Pointer(t) => DebugTypeKind::Pointer(t.0),
            TypeKind::Proc(_) | TypeKind::Type | TypeKind::Code | TypeKind::Null => {
                DebugTypeKind::Typedef(TypeId::VOID_PTR.0)
            }
            TypeKind::String => {
                let fields = vec![
                    field("count", TypeId::S64.0, 0),
                    field("data", ir::DEBUG_CHAR_PTR, 8),
                ];
                DebugTypeKind::Struct {
                    fields,
                    union: false,
                }
            }
            TypeKind::Array {
                elem,
                kind: ArrayKind::Fixed(count),
            } => DebugTypeKind::Array {
                elem: elem.0,
                count,
            },
            TypeKind::Array {
                ..
            }
            | TypeKind::Any => {
                let members = self.builtin_members(ty, Span::NONE).unwrap_or_default();
                DebugTypeKind::Struct {
                    fields: members
                        .into_iter()
                        .map(|(n, t, offset)| field(n.as_str(), t.0, offset))
                        .collect(),
                    union: false,
                }
            }
            TypeKind::Struct(s) => {
                let info = self.types.struct_info(s);
                if info.layout != LayoutState::Done {
                    DebugTypeKind::Void
                } else {
                    let fields = info
                        .fields
                        .iter()
                        .filter_map(|f| Some(field(f.name?.as_str(), f.ty.0, f.offset)))
                        .collect();
                    DebugTypeKind::Struct {
                        fields,
                        union: info.is_union,
                    }
                }
            }
            TypeKind::Enum(e) => {
                let info = self.types.enum_info(e);
                DebugTypeKind::Enum {
                    base: info.base.0,
                    members: info
                        .members
                        .iter()
                        .map(|(n, v)| (n.to_string(), *v as i64))
                        .collect(),
                }
            }
            TypeKind::Distinct(d) => {
                DebugTypeKind::Typedef(self.types.distincts[d.0 as usize].base.0)
            }
            TypeKind::Void | TypeKind::CompileTimeOnly => DebugTypeKind::Void,
        };
        let sized = !matches!(kind, DebugTypeKind::Void);
        DebugType {
            name,
            size: if sized {
                self.types.size_of(ty)
            } else {
                0
            },
            align: if sized {
                self.types.align_of(ty)
            } else {
                1
            },
            kind,
        }
    }
}

fn field(name: &str, ty: u32, offset: u64) -> DebugField {
    DebugField {
        name: name.into(),
        ty,
        offset,
    }
}
