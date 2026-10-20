//! Module parameters and program parameters: declaring them, binding arguments and checking their contracts.
use crate::sema::scope::EntityKind;
use crate::sema::{
    Compiler, Diagnostic, EntityId, ModuleId, Rc, Result, ScopeId, Span, Sym, TypeId, Value, ast,
    err, ir, lower, procs, scope,
};
use std::path::Path;

/// A module-owned header contract, evaluated in its declaring file namespace.
/// Registration must not consume a default while an importing type is forming.
pub struct ParameterContract {
    pub(super) entity: EntityId,
    pub(super) pattern: ast::Expr,
    pub(super) scope: ScopeId,
    pub(super) state: ParameterContractState,
}

pub(super) enum ParameterContractState {
    Waiting,
    Checking,
    Checked,
    Failed(Box<Diagnostic>),
}

/// Marks an import argument `.Member` whose type comes from the module parameter.
pub(super) const INFERRED_PARAM: &str = "\0inferred.";

impl Compiler {
    /// Record the named parameters of an import so a program parameter among them reaches
    /// the module's shared instance before the import itself is resolved.
    pub(super) fn note_program_params(&mut self, file_scope: ScopeId, import: &ast::Import) {
        if Some(self.scope(file_scope).module) != self.ns.main_module {
            return;
        }
        let Some(start) = import.program_params_start else {
            return;
        };
        let arguments = &import.params[start..];
        let ast::ImportSource::Module(name) = &import.source else {
            return;
        };
        if arguments.iter().all(|a| a.name.is_none()) {
            return;
        }
        let file = self.scope_file(file_scope);
        let Some(entry) = self.find_module(name, &self.file_dir(file)) else {
            return;
        };
        let path = self.src.canon(&entry);
        for arg in arguments {
            if let Some(n) = arg.name {
                self.ns.program_param_settings.push((
                    path.clone(),
                    n.name,
                    arg.value.clone(),
                    file_scope,
                ));
            }
        }
        let loaded: Vec<ModuleId> = self
            .ns
            .module_cache
            .iter()
            .filter(|((p, _), _)| *p == path)
            .map(|(_, &id)| id)
            .collect();
        for id in loaded {
            self.apply_program_params(id);
        }
    }

    /// Give a module instance's not-yet-used program parameters the values imports set.
    pub(super) fn apply_program_params(&mut self, module: ModuleId) {
        let Some(path) = self.ns.modules[module.0 as usize].path.clone() else {
            return;
        };
        let path = self.src.canon(&path);
        for (p, name, value, scope) in self.ns.program_param_settings.clone() {
            if p != path
                || !self.ns.modules[module.0 as usize]
                    .program_params
                    .contains(&name)
            {
                continue;
            }
            let entity = self.ns.modules[module.0 as usize]
                .param_entities
                .iter()
                .copied()
                .find(|&e| self.entity(e).name == name);
            let Some(entity) = entity else {
                continue;
            };
            if !matches!(self.entity(entity).state, scope::EntityState::Unresolved) {
                continue;
            }
            // The value is evaluated where the import wrote it; a written parameter type
            // would not resolve there, so typed parameters wait for the import itself.
            if let EntityKind::Decl {
                decl, ..
            } = &mut self.entity_mut(entity).kind
                && decl.ty.is_none()
            {
                Rc::make_mut(decl).value = Some(value);
                self.entity_mut(entity).home = scope;
            }
        }
    }

    /// The loaded instance of the module at `path` when `params` only set its program
    /// parameters; their values are applied to it. A parameter the module already used
    /// with a different value is an error.
    pub(super) fn program_instance(
        &mut self,
        path: &Path,
        params: &[(Sym, Value, TypeId)],
        span: Span,
        import: Option<&ast::Import>,
    ) -> Result<Option<ModuleId>> {
        let Some(id) = self
            .ns
            .module_cache
            .iter()
            .filter(|((p, _), _)| p == path)
            .map(|(_, &id)| id)
            .min_by_key(|id| id.0)
        else {
            return Ok(None);
        };
        let module = &self.ns.modules[id.0 as usize];
        if !params
            .iter()
            .all(|(n, ..)| module.program_params.contains(n))
        {
            return Ok(None);
        }
        if let Some(import) = import {
            self.validate_or_queue_import_groups(id, import)?;
        }
        self.ns.module_load_result(id)?;
        for (name, value, ty) in params {
            let entity = self.ns.modules[id.0 as usize]
                .param_entities
                .iter()
                .copied()
                .find(|&e| self.entity(e).name == *name);
            let Some(entity) = entity else {
                continue;
            };
            if let scope::EntityState::Done(resolved) = &self.entity(entity).state {
                let same =
                    matches!(resolved, scope::Resolved::Const { value: v, .. } if v == value);
                if !same {
                    return err(
                        span,
                        format!(
                            "module parameter `{name}` is set here after the module already used another value"
                        ),
                    );
                }
                continue;
            }
            // An import may have bound the value without resolving the entity
            // yet. Deferred contract checking must not let a later importer
            // overwrite that explicit binding merely because it is still lazy.
            if let EntityKind::Const {
                value: bound, ..
            } = &self.entity(entity).kind
            {
                if bound != value {
                    return err(
                        span,
                        format!(
                            "module parameter `{name}` is set here after the module already used another value"
                        ),
                    );
                }
                continue;
            }
            self.entity_mut(entity).kind = EntityKind::Const {
                value: value.clone(),
                ty: *ty,
            };
        }
        Ok(Some(id))
    }

    pub(super) fn declare_module_parameter(
        &mut self,
        scope: ScopeId,
        module: ModuleId,
        position: usize,
        param: &ast::Param,
        eval_scope: ScopeId,
    ) -> Result<()> {
        let Some(name) = param.name else {
            return err(param.span, "module parameter needs a name");
        };
        let provided = self.ns.modules[module.0 as usize]
            .params
            .iter()
            .find(|(n, ..)| *n == name.name || n.as_str() == format!("${position}"))
            .map(|(_, v, ty)| (v.clone(), *ty));
        let decl = Rc::new(ast::Decl {
            id: ast::AstId::fresh(),
            names: vec![name],
            kind: ast::DeclKind::Const,
            // `X: $I/interface T = v` takes its type from the value.
            ty: param
                .ty
                .clone()
                .filter(|t| !procs::has_poly(t))
                .or_else(|| literal_sized_type(param)),
            value: param.default.clone(),
            using: false,
            as_: false,
            backtick: false,
            notes: ast::Notes::default(),
            span: param.span,
            rare: None,
        });
        let id = self.add_entity(
            scope,
            name.name,
            name.span,
            EntityKind::Decl {
                decl,
                index: 0,
            },
            false,
        );
        // Parameter storage belongs to the module; its declaration expressions
        // retain the lexical file environment, just like ordinary declarations.
        self.entity_mut(id).home = eval_scope;
        self.ns.modules[module.0 as usize].param_entities.push(id);
        self.ns.modules[module.0 as usize].arguments_validated = false;
        if let Some((Value::String(text), _)) = &provided
            && let Some(member) = std::str::from_utf8(text)
                .ok()
                .and_then(|t| t.strip_prefix(INFERRED_PARAM))
        {
            // Typecheck `.Member` against the parameter's declared type.
            let EntityKind::Decl {
                decl, ..
            } = &mut self.entity_mut(id).kind
            else {
                unreachable!()
            };
            let decl = Rc::make_mut(decl);
            // `P := Kind.A` has no written type: use the default's (`type_of(Kind.A)`).
            if decl.ty.is_none()
                && let Some(default) = decl.value.take()
            {
                let span = default.span;
                decl.ty = Some(ast::Expr {
                    kind: ast::ExprKind::Call {
                        callee: Box::new(ast::Expr {
                            kind: ast::ExprKind::Ident(Sym::intern("type_of")),
                            span,
                        }),
                        args: vec![ast::Arg {
                            name: None,
                            target: None,
                            context: false,
                            spread: false,
                            value: default,
                        }],
                        hint: ast::CallHint::None,
                    },
                    span,
                });
            }
            decl.value = Some(ast::Expr {
                kind: ast::ExprKind::InferredMember(ast::Ident {
                    name: Sym::intern(member),
                    span: param.span,
                }),
                span: param.span,
            });
            return Ok(());
        }
        if let Some((value, _)) = &provided
            && let Some(wanted) = self.module_param_class(param, eval_scope)?
            && let Some(given) = ValueClass::of_value(value)
            && wanted != given
            && !(wanted == ValueClass::Float && given == ValueClass::Integer)
        {
            return err(
                param.span,
                format!(
                    "module parameter `{}` expects {wanted}, but the import gives {given}",
                    name.name
                ),
            );
        }
        if let Some((value, ty)) = provided {
            // Convert the original constant rather than reconstructing a literal AST:
            // reconstruction loses lexical precision, relocation identity and declared types.
            if let Some(target) = param.ty.as_ref().filter(|t| !procs::has_poly(t)) {
                let expected = self.eval_type(eval_scope, target)?;
                let actual = if ty == TypeId::VOID {
                    self.type_of_value(&value)
                } else {
                    ty
                };
                let mut f = self.thunk_ctx(ir::FuncOrigin::ConstInit, self.scope_file(eval_scope));
                let op = self.convert(
                    &mut f,
                    lower::Operand::Const {
                        ty: actual,
                        value,
                        untyped: ty == TypeId::VOID,
                    },
                    expected,
                    param.span,
                )?;
                let op = if op.is_const() {
                    op
                } else {
                    self.run_thunk(f, op, param.span)?
                };
                let Some(value) = op.const_value() else {
                    return err(
                        param.span,
                        "module parameter conversion did not produce a constant",
                    );
                };
                self.entity_mut(id).kind = EntityKind::Const {
                    value,
                    ty: expected,
                };
                return Ok(());
            }
            // A parameter without a written type is a constant like `NAME :: value`: given an
            // untyped number it stays untyped, so `MAX_FRAMES := 3` passes to a `u64`.
            let untyped = ty == TypeId::VOID
                && param.ty.is_none()
                && matches!(value, Value::Int(_) | Value::Float(_, _));
            let entity = self.entity_mut(id);
            entity.kind = EntityKind::Const {
                value,
                ty,
            };
            entity.untyped_const = untyped;
        }
        Ok(())
    }

    /// Validate registered header contracts only after their declarations and
    /// program settings exist. A reentrant expansion does not restart the same
    /// contract; resolving a genuinely cyclic parameter still fails normally.
    pub(super) fn validate_parameter_contracts(&mut self, module: ModuleId) -> Result<()> {
        if self.ns.modules[module.0 as usize].loading {
            return Ok(());
        }
        self.apply_program_params(module);
        let mut index = 0;
        while index < self.ns.modules[module.0 as usize].parameter_contracts.len() {
            let contract = &self.ns.modules[module.0 as usize].parameter_contracts[index];
            match &contract.state {
                ParameterContractState::Checked | ParameterContractState::Checking => {
                    index += 1;
                    continue;
                }
                ParameterContractState::Failed(error) => return Err(error.clone()),
                ParameterContractState::Waiting => {}
            }
            let (entity, pattern, eval_scope) =
                (contract.entity, contract.pattern.clone(), contract.scope);
            self.ns.modules[module.0 as usize].parameter_contracts[index].state =
                ParameterContractState::Checking;
            let result = (|| {
                let scope::Resolved::Const {
                    value,
                    ty,
                } = self.resolve_entity(entity)?
                else {
                    return err(pattern.span, "a module parameter must bind a constant");
                };
                let represented = if let Value::Type(t) = value {
                    Some(t)
                } else {
                    None
                };
                let actual = if ty == TypeId::VOID {
                    self.type_of_value(&value)
                } else {
                    ty
                };
                let mut bindings = Vec::new();
                self.match_pattern_value(&pattern, actual, &mut bindings, eval_scope, represented)?;
                let scope = self.ns.modules[module.0 as usize].scope;
                for (name, value, ty) in bindings {
                    self.add_const(scope, name, pattern.span, value, ty);
                }
                Ok(())
            })();
            let contract = &mut self.ns.modules[module.0 as usize].parameter_contracts[index];
            match result {
                // Resolving the parameter reached an import that forwards this very parameter as
                // an argument: the import is being resolved, so check the contract again later.
                Err(error)
                    if error
                        .message
                        .starts_with("circular dependency while resolving") =>
                {
                    contract.state = ParameterContractState::Waiting;
                }
                Ok(()) => contract.state = ParameterContractState::Checked,
                Err(error) => {
                    contract.state = ParameterContractState::Failed(error.clone());
                    return Err(error);
                }
            }
            index += 1;
        }
        Ok(())
    }
}

/// The kind of value a module parameter's literal check compares: the parameter's declared type
/// (or literal default) against the constant the import gives.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ValueClass {
    Bool,
    String,
    Integer,
    Float,
}

impl ValueClass {
    fn of_type(kind: &crate::types::TypeKind) -> Option<ValueClass> {
        use crate::types::TypeKind as K;
        match kind {
            K::Bool => Some(ValueClass::Bool),
            K::String => Some(ValueClass::String),
            K::Int {
                ..
            } => Some(ValueClass::Integer),
            K::Float {
                ..
            } => Some(ValueClass::Float),
            _ => None,
        }
    }

    fn of_literal(expr: &ast::Expr) -> Option<ValueClass> {
        use ast::ExprKind as E;
        let mut expr = expr;
        while let E::Unary(ast::UnOp::Neg | ast::UnOp::Plus, inner) = &expr.kind {
            expr = inner;
        }
        match expr.kind {
            E::Bool(_) => Some(ValueClass::Bool),
            E::Str(_) => Some(ValueClass::String),
            E::Int(_) => Some(ValueClass::Integer),
            E::Float(_) => Some(ValueClass::Float),
            _ => None,
        }
    }

    fn of_value(value: &Value) -> Option<ValueClass> {
        match value {
            Value::Bool(_) => Some(ValueClass::Bool),
            Value::String(_) => Some(ValueClass::String),
            Value::Int(_) => Some(ValueClass::Integer),
            Value::Float(_, _) => Some(ValueClass::Float),
            _ => None,
        }
    }
}

impl std::fmt::Display for ValueClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            ValueClass::Bool => "bool",
            ValueClass::String => "string",
            ValueClass::Integer => "integer",
            ValueClass::Float => "float",
        })
    }
}

impl Compiler {
    /// What a module parameter holds: its declared type, resolved in `scope` (so aliases and
    /// shadowing follow the program), else its literal default. Polymorphic, distinct, enum and
    /// other parameters are not checked here; the conversion after this reports those.
    fn module_param_class(
        &mut self,
        param: &ast::Param,
        scope: ScopeId,
    ) -> Result<Option<ValueClass>> {
        let Some(ty) = &param.ty else {
            return Ok(param.default.as_ref().and_then(ValueClass::of_literal));
        };
        if procs::has_poly(ty) {
            return Ok(None);
        }
        let ty = self.eval_type(scope, ty)?;
        Ok(ValueClass::of_type(self.ty.types.kind(ty)))
    }
}

/// `NAMES: [$N] string = .[...]`: with no import argument the parameter has the literal's
/// count, so its type is `[count] string`.
fn literal_sized_type(param: &ast::Param) -> Option<ast::Expr> {
    let (
        Some(ast::Expr {
            kind:
                ast::ExprKind::ArrayType {
                    size: ast::ArraySize::Fixed(size),
                    elem,
                },
            span,
        }),
        Some(ast::Expr {
            kind:
                ast::ExprKind::ArrayLit {
                    ty: None,
                    elems,
                },
            ..
        }),
    ) = (&param.ty, &param.default)
    else {
        return None;
    };
    if !matches!(size.kind, ast::ExprKind::PolyVar { .. }) || procs::has_poly(elem) {
        return None;
    }
    Some(ast::Expr {
        kind: ast::ExprKind::ArrayType {
            size: ast::ArraySize::Fixed(Box::new(ast::Expr {
                kind: ast::ExprKind::Int(ast::Int128::new(elems.len() as u128)),
                span: size.span,
            })),
            elem: elem.clone(),
        },
        span: *span,
    })
}
