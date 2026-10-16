//! Lexical call headers are not part of structural procedure type identity.
use super::lower::{FnCtx, Operand};
use super::scope::{Builtin, BuiltinProc, EntityKind, EntityState, Found, Resolved};
use super::*;
use crate::ast::ExprKind as E;

impl Compiler {
    /// Names/defaults of the actual selected declaration. This never reads
    /// runtime storage and never searches another declaration with an equal type.
    pub fn callable_entity_parameters(
        &mut self,
        entity: EntityId,
    ) -> Result<Option<Rc<ProcTypeParams>>> {
        self.callable_entity_parameters_in(entity, &mut HashSet::default())
    }

    fn callable_entity_parameters_in(
        &mut self,
        entity: EntityId,
        visiting: &mut HashSet<EntityId>,
    ) -> Result<Option<Rc<ProcTypeParams>>> {
        if let Some(info) = self.callable_entities.get(&entity) {
            return Ok(Some(info.clone()));
        }
        if !visiting.insert(entity) {
            return Ok(None);
        }
        let source = self.entity(entity);
        let scope = source.home;
        if let EntityKind::Decl {
            decl, ..
        } = &source.kind
            && decl.ty.is_none()
        {
            let procedure = match &source.state {
                EntityState::Done(
                    Resolved::Proc(procedure)
                    | Resolved::Const {
                        value: Value::Proc(procedure),
                        ..
                    },
                ) => Some(*procedure),
                EntityState::Done(Resolved::ProcSet(procedures)) if procedures.len() == 1 => {
                    Some(procedures[0])
                }
                _ => None,
            };
            if let Some(procedure) = procedure {
                visiting.remove(&entity);
                return self.callable_procedure_parameters(procedure).map(Some);
            }
        }
        let result = match source.kind.clone() {
            EntityKind::Decl {
                decl, ..
            } => self.callable_expression_parameters_in(
                scope,
                decl.ty.as_ref().or(decl.value.as_ref()),
                visiting,
            ),
            EntityKind::Const {
                value: Value::Proc(procedure),
                ..
            } => self.callable_procedure_parameters(procedure).map(Some),
            _ => Ok(None),
        };
        visiting.remove(&entity);
        result
    }

    pub(super) fn callable_expression_parameters(
        &mut self,
        scope: ScopeId,
        expression: Option<&ast::Expr>,
    ) -> Result<Option<Rc<ProcTypeParams>>> {
        self.callable_expression_parameters_in(scope, expression, &mut HashSet::default())
    }

    fn callable_expression_parameters_in(
        &mut self,
        scope: ScopeId,
        expression: Option<&ast::Expr>,
        visiting: &mut HashSet<EntityId>,
    ) -> Result<Option<Rc<ProcTypeParams>>> {
        let Some(expression) = expression else {
            return Ok(None);
        };
        let header = match &expression.kind {
            E::ProcType(header) => Some(header),
            E::Proc(procedure) => Some(&procedure.header),
            _ => None,
        };
        if let Some(header) = header {
            let c_call = header.flags.c_call || header.flags.cpp_method || header.foreign.is_some();
            let parameters: Vec<_> = header
                .params
                .iter()
                .filter(|p| !p.baked && !(c_call && p.variadic))
                .collect();
            let mut returns = Vec::new();
            for result in &header.returns {
                returns.push(self.callable_expression_parameters_in(
                    scope,
                    result.ty.as_ref().or(result.default.as_ref()),
                    visiting,
                )?);
            }
            return Ok(Some(Rc::new(ProcTypeParams {
                names: parameters.iter().map(|p| p.name.map(|n| n.name)).collect(),
                defaults: parameters.iter().map(|p| p.default.clone()).collect(),
                scope,
                returns,
            })));
        }
        if let Some(entity) = self.callable_source_entity(scope, expression)? {
            return self.callable_entity_parameters_in(entity, visiting);
        }
        match &expression.kind {
            E::TypeDirective {
                modifier: ast::TypeModifier::Plain,
                ty,
            } => return self.callable_expression_parameters_in(scope, Some(ty), visiting),
            E::Unary(ast::UnOp::Star | ast::UnOp::Deref, inner) => {
                return self.callable_expression_parameters_in(scope, Some(inner), visiting);
            }
            E::Member(base, member) => {
                if let Some(ty) = self.callable_source_type(scope, base)? {
                    return self.callable_member_parameters(
                        self.types.pointee(ty).unwrap_or(ty),
                        member.name,
                        member.span,
                    );
                }
            }
            E::Ident(name) => {
                if let Found::Using(entry, member) = self.lookup_full(scope, *name)?
                    && let Some(ty) = entry.member_type()
                {
                    return self.callable_member_parameters(
                        self.types.pointee(ty).unwrap_or(ty),
                        member,
                        expression.span,
                    );
                }
            }
            _ => {}
        }
        if let E::Call {
            callee,
            args,
            ..
        } = &expression.kind
            && args.len() == 1
            && let Some(entity) = self.callable_source_entity(scope, callee)?
            && matches!(
                self.entity(entity).kind,
                EntityKind::Builtin(Builtin::Proc(BuiltinProc::TypeOf))
            )
        {
            return self.callable_expression_parameters_in(scope, Some(&args[0].value), visiting);
        }
        if let E::Call {
            callee, ..
        } = &expression.kind
            && let Some(parameters) =
                self.callable_expression_parameters_in(scope, Some(callee), visiting)?
            && parameters.returns.len() == 1
        {
            return Ok(parameters.returns[0].clone());
        }
        Ok(None)
    }

    /// A declaration's already-resolved static type, not its runtime contents.
    fn callable_source_type(
        &mut self,
        scope: ScopeId,
        expression: &ast::Expr,
    ) -> Result<Option<TypeId>> {
        if matches!(expression.kind, E::Context) {
            return Ok(self.context_type);
        }
        if let Some(entity) = self.callable_source_entity(scope, expression)? {
            if let EntityKind::Local {
                ty, ..
            } = self.entity(entity).kind
            {
                return Ok(Some(ty));
            }
            return Ok(match self.resolve_entity(entity)? {
                Resolved::Const {
                    value: Value::Type(ty),
                    ..
                } => Some(ty),
                Resolved::Const {
                    ty, ..
                }
                | Resolved::Global {
                    ty, ..
                } => Some(ty),
                _ => None,
            });
        }
        if let E::Member(base, member) = &expression.kind
            && let Some(ty) = self.callable_source_type(scope, base)?
        {
            return Ok(self
                .find_member(
                    self.types.pointee(ty).unwrap_or(ty),
                    member.name,
                    member.span,
                )?
                .map(|(_, ty)| ty));
        }
        Ok(None)
    }

    /// Resolve only namespace/declaration paths. No evaluation of arbitrary
    /// initializers, calls or foreign loads is used to discover a header.
    fn callable_source_entity(
        &mut self,
        scope: ScopeId,
        expression: &ast::Expr,
    ) -> Result<Option<EntityId>> {
        let found = match &expression.kind {
            E::Ident(name) => self.lookup_full(scope, *name)?,
            E::Member(base, member) => {
                let Some(entity) = self.callable_source_entity(scope, base)? else {
                    return Ok(None);
                };
                if matches!(self.entity(entity).kind, EntityKind::Local { .. }) {
                    return Ok(None);
                }
                match self.resolve_entity(entity)? {
                    Resolved::Module(module) => self.module_member_lookup(module, member.name)?,
                    Resolved::Const {
                        value: Value::Type(ty),
                        ..
                    } => {
                        Found::Entities(self.struct_constant(ty, member.name)?.unwrap_or_default())
                    }
                    _ => return Ok(None),
                }
            }
            _ => return Ok(None),
        };
        Ok(match found {
            Found::Entities(ids) if ids.len() == 1 => Some(ids[0]),
            _ => None,
        })
    }

    pub(super) fn callable_procedure_parameters(
        &mut self,
        procedure: ProcId,
    ) -> Result<Rc<ProcTypeParams>> {
        let signature = self.signature(procedure, self.proc(procedure).span)?;
        Ok(Rc::new(ProcTypeParams {
            names: signature
                .params
                .iter()
                .filter(|p| !(signature.c_varargs && p.variadic))
                .map(|p| p.name)
                .collect(),
            defaults: signature
                .params
                .iter()
                .filter(|p| !(signature.c_varargs && p.variadic))
                .map(|p| p.default.clone())
                .collect(),
            scope: signature.scope,
            returns: signature.return_callables.clone(),
        }))
    }

    pub(super) fn callable_operand_parameters(
        &mut self,
        _function: &FnCtx,
        operand: &Operand,
    ) -> Result<Option<Rc<ProcTypeParams>>> {
        match operand {
            Operand::Multi(values) if !values.is_empty() => {
                self.callable_operand_parameters(_function, &values[0])
            }
            Operand::Callable {
                parameters, ..
            } => Ok(Some(parameters.clone())),
            Operand::Procs(procedures) if procedures.len() == 1 => {
                self.callable_procedure_parameters(procedures[0]).map(Some)
            }
            Operand::Const {
                value: Value::Proc(procedure),
                ..
            } => self.callable_procedure_parameters(*procedure).map(Some),
            _ => Ok(None),
        }
    }
}
