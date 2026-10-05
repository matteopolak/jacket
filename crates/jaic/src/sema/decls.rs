//! Resolving global declarations: constants, globals, types, libraries.
use super::lower::Operand;
use super::scope::{Builtin, EntityKind, EntityState, Resolved};
use super::value::LibraryId;
use super::*;
use crate::types::{DistinctInfo, TypeKind};

pub struct LibraryInfo {
    pub name: String,
    pub system: bool,
    pub ir: usize,
}

impl Compiler {
    /// Resolve an entity to its value/storage, checking it on first use.
    pub fn resolve_entity(&mut self, id: EntityId) -> Result<Resolved> {
        match &self.entity(id).state {
            EntityState::Done(r) => return Ok(r.clone()),
            EntityState::Resolving => {
                self.in_progress_misses += 1;
                let e = self.entity(id);
                return err(
                    e.span,
                    format!("circular dependency while resolving '{}'", e.name),
                );
            }
            EntityState::Unresolved => {}
        }
        self.entity_mut(id).state = EntityState::Resolving;
        let result = self.resolve_entity_inner(id);
        match &result {
            Ok(r) => self.entity_mut(id).state = EntityState::Done(r.clone()),
            Err(_) => self.entity_mut(id).state = EntityState::Unresolved,
        }
        result
    }

    fn resolve_entity_inner(&mut self, id: EntityId) -> Result<Resolved> {
        let entity = self.entity(id);
        let (name, span, scope) = (entity.name, entity.span, entity.home);
        match entity.kind.clone() {
            EntityKind::Const {
                value,
                ty,
            } => {
                if ty == TypeId::VOID {
                    // Module parameter provided by an import: type from its value.
                    let ty = self.type_of_value(&value);
                    return Ok(Resolved::Const {
                        value,
                        ty,
                    });
                }
                Ok(Resolved::Const {
                    value,
                    ty,
                })
            }
            EntityKind::Local {
                ..
            } => err(
                span,
                format!("internal: local '{name}' resolved as a declaration"),
            ),
            EntityKind::Placeholder => {
                // Top-level items that needed it wait for the metaprogram to define it.
                self.placeholder_misses += 1;
                err(
                    span,
                    format!("'{name}' is a #placeholder that was never defined"),
                )
            }
            EntityKind::Import(import) => {
                Ok(Resolved::Module(self.resolve_import(scope, &import)?))
            }
            EntityKind::Builtin(Builtin::Type(t)) => Ok(Resolved::Const {
                value: Value::Type(t),
                ty: TypeId::TYPE,
            }),
            EntityKind::Builtin(Builtin::Proc(_)) => {
                unreachable!("builtin procs are handled by lookup")
            }
            EntityKind::Builtin(Builtin::TargetConstant(sym)) => self.target_constant(sym, span),
            EntityKind::Decl {
                decl,
                index,
            } => match decl.kind {
                ast::DeclKind::Const => self.resolve_const_decl(id, scope, name, &decl, index),
                ast::DeclKind::Var => self.resolve_global_var(scope, name, &decl),
            },
        }
    }

    fn target_constant(&mut self, sym: Sym, span: Span) -> Result<Resolved> {
        let (enum_name, member) = match sym.as_str() {
            "OS" | "BUILD_OS" => (
                "Operating_System_Tag",
                match self.options.os {
                    TargetOs::Windows => "WINDOWS",
                    TargetOs::Linux => "LINUX",
                    TargetOs::MacOS => "MACOS",
                    TargetOs::Wasm => "WASM",
                },
            ),
            "CPU" | "BUILD_CPU" => (
                "CPU_Tag",
                match self.options.cpu {
                    TargetCpu::X64 => "X64",
                    TargetCpu::Arm64 => "ARM64",
                    TargetCpu::Wasm => "CUSTOM",
                },
            ),
            "TEMPORARY_STORAGE_SIZE" => {
                return Ok(Resolved::Const {
                    value: Value::Int(self.options.temporary_storage_size as i128),
                    ty: TypeId::S64,
                });
            }
            "LANGUAGE_VERSION_MAJOR" => {
                return Ok(Resolved::Const {
                    value: Value::Int(0),
                    ty: TypeId::S64,
                });
            }
            "LANGUAGE_VERSION_MINOR" => {
                return Ok(Resolved::Const {
                    value: Value::Int(2),
                    ty: TypeId::S64,
                });
            }
            _ => unreachable!(),
        };
        let Some(preload) = self.preload else {
            return err(span, format!("'{sym}' requires Preload"));
        };
        let ids = self.module_exports(preload, Sym::intern(enum_name))?;
        let Some(&enum_entity) = ids.first() else {
            return err(span, format!("Preload does not define {enum_name}"));
        };
        let Resolved::Const {
            value: Value::Type(ty),
            ..
        } = self.resolve_entity(enum_entity)?
        else {
            return err(span, format!("{enum_name} is not a type"));
        };
        let TypeKind::Enum(e) = self.types.kind(ty).clone() else {
            return err(span, format!("{enum_name} is not an enum"));
        };
        let info = self.types.enum_info(e);
        let Some(&(_, value)) = info.members.iter().find(|(n, _)| n.as_str() == member) else {
            return err(span, format!("{enum_name} has no member {member}"));
        };
        Ok(Resolved::Const {
            value: Value::Int(value),
            ty,
        })
    }

    pub fn type_of_value(&mut self, value: &Value) -> TypeId {
        match value {
            Value::Int(_) => TypeId::S64,
            Value::Float(_) => TypeId::F32,
            Value::Bool(_) => TypeId::BOOL,
            Value::String(_) => TypeId::STRING,
            Value::Type(_) => TypeId::TYPE,
            Value::Null => TypeId::NULL,
            Value::Proc(p) => self
                .proc_type(*p, Span::default())
                .unwrap_or(TypeId::VOID_PTR),
            Value::Code(_) => TypeId::CODE,
            Value::Bytes(_) | Value::Void => TypeId::VOID,
        }
    }

    fn resolve_const_decl(
        &mut self,
        id: EntityId,
        scope: ScopeId,
        name: Sym,
        decl: &Rc<ast::Decl>,
        index: usize,
    ) -> Result<Resolved> {
        let Some(value) = &decl.value else {
            return err(decl.span, "constant declaration needs a value");
        };
        let multiple = decl.names.len() > 1;
        match &value.kind {
            // `a, b :: #run f();` takes one value each (below).
            _ if multiple => {}
            ast::ExprKind::Proc(lit) => {
                let proc = self.new_proc(name, lit.clone(), scope, value.span);
                if !decl.notes.is_empty() {
                    self.proc_decl_notes.insert(proc, decl.notes.clone());
                }
                return Ok(Resolved::Proc(proc));
            }
            ast::ExprKind::Lambda {
                header,
                body,
            } => {
                let lit = lambda::lambda_lit(header, body);
                return Ok(Resolved::Proc(self.new_proc(name, lit, scope, value.span)));
            }
            ast::ExprKind::ProcType(header)
                if header
                    .flags
                    .other
                    .iter()
                    .any(|f| f.name.as_str() == "entry_point") =>
            {
                let lit = Rc::new(ast::ProcLit {
                    header: header.clone(),
                    body: None,
                });
                return Ok(Resolved::Proc(self.new_proc(name, lit, scope, value.span)));
            }
            ast::ExprKind::Struct(lit) => {
                if !lit.params.is_empty() {
                    return Ok(Resolved::PolyStruct(self.new_poly_struct(
                        name,
                        lit.clone(),
                        scope,
                    )));
                }
                let ty = self.new_struct_type(name, lit.clone(), scope, Vec::new(), None);
                return Ok(Resolved::Const {
                    value: Value::Type(ty),
                    ty: TypeId::TYPE,
                });
            }
            ast::ExprKind::Enum(lit) => {
                let ty = self.new_enum_type(name, lit, scope)?;
                return Ok(Resolved::Const {
                    value: Value::Type(ty),
                    ty: TypeId::TYPE,
                });
            }
            ast::ExprKind::TypeDirective {
                modifier,
                ty,
            } if *modifier != ast::TypeModifier::Plain => {
                let base = self.eval_type(scope, ty)?;
                let isa = *modifier == ast::TypeModifier::Isa;
                let ty = self.types.new_distinct(DistinctInfo {
                    name,
                    base,
                    isa,
                });
                return Ok(Resolved::Const {
                    value: Value::Type(ty),
                    ty: TypeId::TYPE,
                });
            }
            ast::ExprKind::UnknownDirective {
                name: directive,
                flags,
                operand,
            } => {
                let d = directive.name.as_str();
                if matches!(
                    d,
                    "system_library" | "library" | "foreign_library" | "foreign_system_library"
                ) {
                    let Some(operand) = operand else {
                        return err(value.span, "library directive needs a name");
                    };
                    let lib_name = match &operand.kind {
                        ast::ExprKind::Str(s) => String::from_utf8_lossy(s).into_owned(),
                        _ => return err(operand.span, "library name must be a string literal"),
                    };
                    let flag = |f: &str| flags.iter().any(|x| x.name.as_str() == f);
                    let system = d.contains("system") || flag("system");
                    let link_always = flag("link_always");
                    let base_dir = self.file_dir(self.scope_file(scope)).display().to_string();
                    let ir = self.program.libraries.len();
                    self.program.libraries.push(ir::Library {
                        name: lib_name.clone(),
                        system,
                        link_always,
                        base_dir,
                    });
                    self.libraries.push(LibraryInfo {
                        name: lib_name,
                        system,
                        ir,
                    });
                    return Ok(Resolved::Library(LibraryId(
                        self.libraries.len() as u32 - 1,
                    )));
                }
            }
            _ => {}
        }
        let expected = match &decl.ty {
            Some(t) => Some(self.eval_type(scope, t)?),
            None => None,
        };
        let op = if multiple {
            // Evaluated once for all the names.
            // Per scope: a macro body's declaration is evaluated per expansion.
            let values = match self.multi_consts.get(&(decl.id, scope)) {
                Some(values) => values.clone(),
                None => {
                    let values = self.eval_const_all(scope, value)?;
                    self.multi_consts.insert((decl.id, scope), values.clone());
                    values
                }
            };
            match values.get(index) {
                Some(op) => op.clone(),
                None => {
                    return err(
                        decl.span,
                        format!("{} names but {} values", decl.names.len(), values.len()),
                    );
                }
            }
        } else {
            self.eval_const(scope, value, expected)?
        };
        match op {
            Operand::Type(t) => Ok(Resolved::Const {
                value: Value::Type(t),
                ty: TypeId::TYPE,
            }),
            Operand::Const {
                ty,
                value,
                untyped,
            } => {
                let ty = match expected {
                    Some(t) => t,
                    None if untyped => {
                        self.entity_mut(id).untyped_const = true;
                        self.default_untyped(ty, &value)
                    }
                    None => ty,
                };
                let value = if self.types.is_float(ty) {
                    match value {
                        Value::Int(i) => Value::Float(i as f64),
                        v => v,
                    }
                } else {
                    value
                };
                Ok(Resolved::Const {
                    value,
                    ty,
                })
            }
            Operand::Procs(procs) if procs.len() == 1 => Ok(Resolved::Proc(procs[0])),
            Operand::Procs(procs) => Ok(Resolved::ProcSet(procs)),
            Operand::Module(m) => Ok(Resolved::Module(m)),
            Operand::PolyStruct(p) => Ok(Resolved::PolyStruct(p)),
            Operand::Library(l) => Ok(Resolved::Library(l)),
            _ => err(
                value.span,
                "constant declaration requires a compile-time value; use #run to compute it",
            ),
        }
    }

    pub fn scope_file(&self, mut scope: ScopeId) -> FileId {
        loop {
            let s = self.scope(scope);
            if let Some(f) = s.file {
                return f;
            }
            match s.parent {
                Some(p) => scope = p,
                None => return FileId(0),
            }
        }
    }

    fn resolve_global_var(
        &mut self,
        scope: ScopeId,
        name: Sym,
        decl: &Rc<ast::Decl>,
    ) -> Result<Resolved> {
        let span = decl.span;
        let declared = match &decl.ty {
            Some(t) => Some(self.eval_type(scope, t)?),
            None => None,
        };
        // A typed result of evaluating the initializer to find the type; reused so that
        // `x := #run f();` runs `f` once.
        let mut evaluated: Option<Value> = None;
        // Foreign data: `x: T #elsewhere lib;`
        let ty = match (&declared, &decl.value) {
            (Some(t), _) => *t,
            (None, Some(v)) => {
                let op = self.eval_const_or_run(scope, v, None)?;
                match &op {
                    Operand::Const {
                        ty,
                        value,
                        untyped,
                    } => {
                        if *untyped {
                            self.default_untyped(*ty, value)
                        } else {
                            evaluated = Some(value.clone());
                            *ty
                        }
                    }
                    Operand::Type(_) => TypeId::TYPE,
                    other => other.ty(),
                }
            }
            (None, None) => return err(span, "global needs a type or initializer"),
        };
        if let Some(foreign) = &decl.foreign {
            return self.foreign_global(scope, name, foreign, ty, span);
        }
        let size = self.size_of(ty, span)?;
        let align = self.align_of(ty, span)?;
        let mut align = align;
        if let Some(a) = &decl.align {
            let a = self.eval_int(scope, a)?;
            align = align.max(a as u64);
        }
        let global = self.program.add_global(ir::Global {
            name: name.to_string(),
            size,
            align,
            init: Vec::new(),
            relocs: Vec::new(),
            read_only: false,
            export: None,
        });
        self.debug_global(global, name, ty, span);
        if !decl.flags.iter().any(|f| f.name.as_str() == "no_reset") {
            self.program.reset_globals.push(global);
        }
        // Mark resolved before evaluating the initializer so self-references work.
        // The entity state is set by resolve_entity on return; initializers that
        // reference this global's address see it through `pending_globals`.
        let init = match &decl.value {
            Some(v) if matches!(v.kind, ast::ExprKind::Uninit) => None,
            Some(_) if evaluated.is_some() => {
                let size = self.size_of(ty, span)?;
                let mut agg = super::value::Aggregate {
                    bytes: vec![0; size as usize],
                    relocs: Vec::new(),
                };
                self.write_value(&mut agg, 0, evaluated.as_ref().unwrap(), ty, span)?;
                Some(Rc::new(agg))
            }
            Some(v) => Some(self.global_initializer(scope, v, ty)?),
            None => self.default_initializer(ty, span)?,
        };
        if let Some(agg) = init {
            let g = &mut self.program.globals[global.0 as usize];
            g.init = agg.bytes.clone();
            g.relocs = agg.relocs.clone();
        }
        Ok(Resolved::Global {
            storage: ir::Storage::Data(global),
            ty,
        })
    }

    /// `x: T #elsewhere lib ["symbol"];`: a variable defined by a foreign
    /// library (or the process). `__runtime_info` without a library is the
    /// compiler's own runtime information.
    fn foreign_global(
        &mut self,
        scope: ScopeId,
        name: Sym,
        foreign: &ast::Foreign,
        ty: TypeId,
        span: Span,
    ) -> Result<Resolved> {
        if foreign.library.is_none() && name.as_str() == "__runtime_info" {
            let global = self.runtime_info_global(span)?;
            return Ok(Resolved::Global {
                storage: ir::Storage::Data(global),
                ty,
            });
        }
        let library = match &foreign.library {
            Some(lib) => self.resolve_library(scope, lib)?,
            None => None,
        };
        let symbol = match &foreign.name {
            ast::ForeignName::Named(s) => s.to_string(),
            ast::ForeignName::Default => name.to_string(),
        };
        let foreign = self.program.add_foreign(ir::Foreign {
            symbol,
            library,
            sig: ir::Sig {
                params: Vec::new(),
                returns: Vec::new(),
                conv: ir::Conv::C,
                c_varargs: false,
                c_fixed: 0,
                c_abi: None,
            },
            is_data: true,
        });
        Ok(Resolved::Global {
            storage: ir::Storage::Foreign(foreign),
            ty,
        })
    }

    /// Evaluate a type expression.
    pub fn eval_type(&mut self, scope: ScopeId, expr: &ast::Expr) -> Result<TypeId> {
        let op = self.eval_const(scope, expr, Some(TypeId::TYPE))?;
        self.operand_as_type(op, expr.span)
    }

    pub fn operand_as_type(&mut self, op: Operand, span: Span) -> Result<TypeId> {
        match op {
            Operand::Type(t) => Ok(t),
            Operand::Const {
                value: Value::Type(t),
                ..
            } => Ok(t),
            Operand::Procs(p) if p.len() == 1 => {
                // A procedure name used as a type means its procedure type (e.g. `#type my_proc`).
                self.proc_type(p[0], span)
            }
            // A polymorphic struct whose parameters all have defaults is a type already.
            Operand::PolyStruct(ps)
                if self.poly_structs[ps.0 as usize]
                    .lit
                    .params
                    .iter()
                    .all(|p| p.default.is_some()) =>
            {
                self.instantiate_struct(ps, Vec::new(), span)
            }
            other => err(
                span,
                format!("expected a type, found {}", self.describe(&other)),
            ),
        }
    }

    pub fn describe(&self, op: &Operand) -> String {
        match op {
            Operand::Value {
                ty, ..
            }
            | Operand::Place {
                ty, ..
            } => format!("a value of type {}", self.types.name(*ty)),
            Operand::Const {
                ty,
                value,
                ..
            } => format!(
                "constant {} of type {}",
                value.render(&self.types),
                self.types.name(*ty)
            ),
            Operand::Type(t) => format!("type {}", self.types.name(*t)),
            Operand::Procs(_) => "a procedure".into(),
            Operand::PolyStruct(_) => "a polymorphic struct".into(),
            Operand::Module(_) => "a module".into(),
            Operand::Builtin(_) => "a builtin procedure".into(),
            Operand::Library(_) => "a library".into(),
            Operand::Multi(_) => "multiple values".into(),
            Operand::Void => "nothing (void)".into(),
        }
    }

    pub fn eval_int(&mut self, scope: ScopeId, expr: &ast::Expr) -> Result<i128> {
        match self.eval_const(scope, expr, None)? {
            Operand::Const {
                value: Value::Int(v),
                ..
            } => Ok(v),
            Operand::Const {
                value: Value::Bool(v),
                ..
            } => Ok(v as i128),
            other => err(
                expr.span,
                format!(
                    "expected a constant integer, found {}",
                    self.describe(&other)
                ),
            ),
        }
    }

    pub fn eval_const_value(&mut self, scope: ScopeId, expr: &ast::Expr) -> Result<Value> {
        Ok(self.eval_const_typed(scope, expr)?.0)
    }

    /// Like `eval_const_value`, also giving the constant's type (`VOID` when it has none).
    pub fn eval_const_typed(
        &mut self,
        scope: ScopeId,
        expr: &ast::Expr,
    ) -> Result<(Value, TypeId)> {
        let op = self.eval_const_or_run(scope, expr, None)?;
        match op {
            Operand::Type(t) => Ok((Value::Type(t), TypeId::VOID)),
            Operand::Const {
                value,
                ty,
                ..
            } => Ok((value, ty)),
            Operand::Procs(p) if p.len() == 1 => Ok((Value::Proc(p[0]), TypeId::VOID)),
            other => err(
                expr.span,
                format!("expected a constant, found {}", self.describe(&other)),
            ),
        }
    }

    pub fn eval_static_condition(&mut self, scope: ScopeId, expr: &ast::Expr) -> Result<bool> {
        match self.eval_const(scope, expr, Some(TypeId::BOOL))? {
            Operand::Const {
                value: Value::Bool(b),
                ..
            } => Ok(b),
            Operand::Const {
                value: Value::Int(i),
                ..
            } => Ok(i != 0),
            Operand::Const {
                value: Value::Null,
                ..
            } => Ok(false),
            Operand::Const {
                value: Value::Proc(_) | Value::Type(_) | Value::String(_),
                ..
            }
            | Operand::Type(_)
            | Operand::Procs(_) => Ok(true),
            // A pointer-sized constant computed at compile time (e.g. a procedure address).
            Operand::Const {
                value: Value::Bytes(agg),
                ty,
                ..
            } if self.types.is_pointer(ty) || matches!(self.types.kind(ty), TypeKind::Proc(_)) => {
                Ok(!agg.relocs.is_empty() || agg.bytes.iter().any(|&b| b != 0))
            }
            other => err(
                expr.span,
                format!(
                    "#if condition must be a compile-time constant, found {}",
                    self.describe(&other)
                ),
            ),
        }
    }

    /// Evaluate an expression that must fold to a constant (no runtime code).
    pub fn eval_const(
        &mut self,
        scope: ScopeId,
        expr: &ast::Expr,
        expected: Option<TypeId>,
    ) -> Result<Operand> {
        let file = self.scope_file(scope);
        let mut f = self.thunk_ctx("const", file);
        // Runtime locals of the enclosing procedure are not visible to constants.
        let scope = self.thunk_scope(scope);
        let op = self.check_expr(&mut f, scope, expr, expected)?;
        match op {
            Operand::Value {
                ..
            }
            | Operand::Place {
                ..
            }
            | Operand::Multi(_) => {
                // Not foldable: run the thunk.
                self.run_thunk(f, op, expr.span)
            }
            other => Ok(other),
        }
    }

    /// Every value of a constant expression (`a, b :: #run f();`).
    fn eval_const_all(&mut self, scope: ScopeId, expr: &ast::Expr) -> Result<Vec<Operand>> {
        // `#run f()` keeps all of f's results, not only the first.
        let expr = match &expr.kind {
            ast::ExprKind::Run {
                body, ..
            } => match &**body {
                ast::RunBody::Expr(e) => e,
                ast::RunBody::Block(_) => expr,
            },
            _ => expr,
        };
        let file = self.scope_file(scope);
        let mut f = self.thunk_ctx("const", file);
        let scope = self.thunk_scope(scope);
        let op = self.check_expr(&mut f, scope, expr, None)?;
        match op {
            Operand::Value {
                ..
            }
            | Operand::Place {
                ..
            }
            | Operand::Multi(_) => self.run_thunk_all(f, op, expr.span),
            other => Ok(vec![other]),
        }
    }

    /// Like `eval_const`, but general expressions are always allowed (global initializers).
    pub fn eval_const_or_run(
        &mut self,
        scope: ScopeId,
        expr: &ast::Expr,
        expected: Option<TypeId>,
    ) -> Result<Operand> {
        self.eval_const(scope, expr, expected)
    }
}
