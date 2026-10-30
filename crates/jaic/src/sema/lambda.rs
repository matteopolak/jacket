//! Lambda expressions: `(x) => x + 1`, `x => { ... }`.
//!
//! A lambda is desugared into an ordinary procedure literal (see [`lambda_lit`]).
//! Untyped parameters become polymorphic (`$__lambda_x`), so a lambda bound with `::`
//! behaves like a polymorphic procedure. Where the expected procedure type is known
//! (a typed parameter or declaration) the lambda is instantiated directly from it.
use super::lower::{FnCtx, Operand};
use super::procs::ParamInfo;
use super::scope::{EntityKind, Resolved, ScopeKind};
use super::{
    Compiler, Diagnostic, ProcId, Rc, Result, ScopeId, Span, Sym, TypeId, Value, ast, err, ir,
};
use crate::ast::ExprKind as E;
use crate::diagnostics::Code;
use crate::types::TypeKind;

/// Name of the constant holding the result type given to a lambda instance.
pub const LAMBDA_RETURN: &str = "__lambda_return";

/// Prefix of the polymorphic variables standing for untyped lambda parameter types.
const PARAM_PREFIX: &str = "__lambda_";

/// Desugar a lambda: untyped parameters get a polymorphic type, an expression
/// body becomes `return <expr>;`.
pub fn lambda_lit(header: &ast::ProcHeader, body: &ast::Expr) -> Rc<ast::ProcLit> {
    let mut header = header.clone();
    for (i, p) in header.params.iter_mut().enumerate() {
        if p.ty.is_none() && p.default.is_none() {
            let name = p.name.map_or_else(|| i.to_string(), |n| n.name.to_string());
            p.ty = Some(ast::Expr {
                kind: E::PolyVar {
                    name: Sym::intern(&format!("{PARAM_PREFIX}{name}")),
                    baked: false,
                },
                span: p.span,
                parenthesized: false,
            });
        }
    }
    let block = if let E::Block(block) = &body.kind {
        header.flags.lambda = ast::LambdaKind::Block;
        (**block).clone()
    } else {
        header.flags.lambda = ast::LambdaKind::Expr;
        ast::Block {
            stmts: vec![ast::Stmt {
                kind: ast::StmtKind::Return {
                    values: vec![ast::Arg {
                        name: None,
                        target: None,
                        context: false,
                        spread: false,
                        value: body.clone(),
                    }],
                    backtick: false,
                },
                span: body.span,
                notes: crate::ast::Notes::default(),
            }],
            span: body.span,
            checks_off: crate::ast::CheckBits::NONE,
        }
    };
    Rc::new(ast::ProcLit {
        header: Rc::new(header),
        body: Some(block),
    })
}

impl Compiler {
    /// The (possibly polymorphic) procedure for a lambda written in `scope`.
    pub fn lambda_proc(
        &mut self,
        scope: ScopeId,
        header: &Rc<ast::ProcHeader>,
        body: &ast::Expr,
        span: Span,
    ) -> ProcId {
        if let Some(&p) = self.pr.anonymous_procs.get(&(header.id, scope)) {
            return p;
        }
        let name = Sym::intern("lambda");
        let p = self.new_proc(name, lambda_lit(header, body), scope, span);
        self.pr.anonymous_procs.insert((header.id, scope), p);
        p
    }

    /// A lambda expression. With an expected procedure type the lambda is
    /// instantiated for it; otherwise the polymorphic procedure is returned.
    pub fn check_lambda(
        &mut self,
        scope: ScopeId,
        header: &Rc<ast::ProcHeader>,
        body: &ast::Expr,
        expected: Option<TypeId>,
        span: Span,
    ) -> Result<Operand> {
        let proc = self.lambda_proc(scope, header, body, span);
        let Some(expected) = expected else {
            return Ok(Operand::Procs(vec![proc]));
        };
        let TypeKind::Proc(pt) = self.ty.types.kind(expected).clone() else {
            return Ok(Operand::Procs(vec![proc]));
        };
        if pt.params.len() != header.params.len() {
            return Err(self.lambda_arity_error(header.params.len(), expected, span));
        }
        let ret = pt.returns.first().copied().unwrap_or(TypeId::VOID);
        let inst = self.lambda_instance(proc, &pt.params, Some(ret), span)?;
        Ok(Operand::Procs(vec![inst]))
    }

    /// Instantiate a lambda for the given parameter types (and result type, if known).
    fn lambda_instance(
        &mut self,
        proc: ProcId,
        params: &[TypeId],
        ret: Option<TypeId>,
        span: Span,
    ) -> Result<ProcId> {
        let header = self.proc(proc).lit.header.clone();
        let mut bindings = Vec::new();
        for (p, &ty) in header.params.iter().zip(params) {
            if let Some(ast::Expr {
                kind: E::PolyVar {
                    name, ..
                },
                ..
            }) = &p.ty
                && name.as_str().starts_with(PARAM_PREFIX)
            {
                bindings.push((*name, Value::Type(ty), TypeId::TYPE));
            }
        }
        if let Some(ret) = ret {
            bindings.push((Sym::intern(LAMBDA_RETURN), Value::Type(ret), TypeId::TYPE));
        }
        self.instantiate(proc, bindings, span)
    }

    /// The result type of a lambda procedure: the one given to the instance, void for
    /// a block body, or the type of the body expression. `None` for other procedures.
    pub fn lambda_return_type(
        &mut self,
        id: ProcId,
        scope: ScopeId,
        params: &[ParamInfo],
    ) -> Result<Option<TypeId>> {
        let lit = self.proc(id).lit.clone();
        let kind = lit.header.flags.lambda;
        if kind == ast::LambdaKind::None {
            return Ok(None);
        }
        if let Ok(ids) = self.lookup(scope, Sym::intern(LAMBDA_RETURN))
            && let [entity] = ids[..]
            && let Ok(Resolved::Const {
                value: Value::Type(t),
                ..
            }) = self.resolve_entity(entity)
        {
            return Ok(Some(t));
        }
        let Some(ast::StmtKind::Return {
            values, ..
        }) = lit
            .body
            .as_ref()
            .and_then(|b| b.stmts.first())
            .map(|s| &s.kind)
            .filter(|_| kind == ast::LambdaKind::Expr)
        else {
            return Ok(Some(TypeId::VOID));
        };
        // Check the body expression once against the parameter types to learn its type.
        let file = self.scope_file(scope);
        let mut scratch = FnCtx::new(
            "lambda".into(),
            ir::Sig {
                params: vec![ir::Ty::Ptr],
                returns: vec![],
                conv: ir::Conv::Jai,
                c_varargs: false,
                c_fixed: 0,
                c_abi: None,
            },
            file,
        );
        scratch.set_context(Some(scratch.b.param(0)));
        let module = self.scope(scope).module;
        let body_scope = self.new_scope(ScopeKind::Proc, Some(scope), module, None);
        for param in params {
            let Some(name) = param.name else {
                continue;
            };
            let size = self.size_of(param.ty, param.span)?;
            let align = self.align_of(param.ty, param.span)?;
            let addr = scratch.b.alloca(size.max(1), align);
            let depth = self.scope(body_scope).proc_depth;
            self.add_entity(
                body_scope,
                name,
                param.span,
                EntityKind::Local {
                    ty: param.ty,
                    addr,
                    depth,
                },
                false,
            );
        }
        let value = &values[0].value;
        let op = self.check_expr(&mut scratch, body_scope, value, None)?;
        Ok(Some(match op {
            Operand::Const {
                ty,
                value,
                untyped: true,
                ..
            } => self.ty.default_untyped(ty, &value),
            Operand::Procs(p) if p.len() == 1 && !self.proc(p[0]).is_poly => {
                self.proc_type(p[0], value.span)?
            }
            Operand::Void => TypeId::VOID,
            other => other.ty(),
        }))
    }

    /// Bindings a lambda argument contributes for a procedure type pattern such as
    /// `(T) -> $R`: the lambda is instantiated with the parameter types bound so far
    /// and its signature is matched against the pattern.
    pub fn infer_lambda_bindings(
        &mut self,
        pattern: &ast::Expr,
        arg: (ScopeId, &Rc<ast::ProcHeader>, &ast::Expr),
        def_scope: ScopeId,
        bindings: &mut Vec<(Sym, Value, TypeId)>,
        span: Span,
    ) -> Result<()> {
        let (arg_scope, header, body) = arg;
        let E::ProcType(pattern_header) = &pattern.kind else {
            return Ok(());
        };
        if pattern_header.params.len() != header.params.len() {
            return Err(Box::new(
                Diagnostic::new(
                    Code::LambdaSignatureMismatch,
                    span,
                    format!(
                        "lambda takes {}, but the parameter type expects {}",
                        plural(header.params.len(), "parameter"),
                        pattern_header.params.len()
                    ),
                )
                .with_note(
                    Span::NONE,
                    format!(
                        "the expected parameter type is `{}`",
                        self.src.sources.snippet(pattern.span).trim()
                    ),
                ),
            ));
        }
        let known = self.const_scope(def_scope, bindings.clone(), span);
        let mut params = Vec::new();
        for p in &pattern_header.params {
            let Some(t) = &p.ty else {
                return err(
                    Code::LambdaSignatureMismatch,
                    span,
                    "cannot infer the parameter types of this lambda",
                );
            };
            params.push(self.eval_type(known, t)?);
        }
        let ret = match pattern_header.returns.as_slice() {
            [] => Some(TypeId::VOID),
            [r] => r.ty.as_ref().and_then(|t| self.eval_type(known, t).ok()),
            _ => None,
        };
        let proc = self.lambda_proc(arg_scope, header, body, span);
        let inst = self.lambda_instance(proc, &params, ret, span)?;
        let ty = self.proc_type(inst, span)?;
        self.match_pattern(pattern, ty, bindings, def_scope)
    }
}

/// `1 parameter`, `2 parameters`.
fn plural(count: usize, noun: &str) -> String {
    format!(
        "{count} {noun}{}",
        if count == 1 {
            ""
        } else {
            "s"
        }
    )
}

#[expect(
    clippy::unnecessary_box_returns,
    reason = "the diagnostics are boxed because the error type carries them boxed; callers pass them through `Err`"
)]
impl Compiler {
    /// A lambda with `got` parameters where the procedure type `expected` takes another number:
    /// the signature it has to fit.
    pub(in crate::sema) fn lambda_arity_error(
        &self,
        got: usize,
        expected: TypeId,
        span: Span,
    ) -> Box<Diagnostic> {
        let wanted = match self.ty.types.kind(expected) {
            TypeKind::Proc(pt) => pt.params.len(),
            _ => 0,
        };
        let name = self.ty.types.name(expected);
        Box::new(
            Diagnostic::new(
                Code::LambdaSignatureMismatch,
                span,
                format!(
                    "lambda takes {}, but `{name}` expects {wanted}",
                    plural(got, "parameter")
                ),
            )
            .with_label(format!("takes {got}, expected {wanted}"))
            .with_note(Span::NONE, format!("the expected signature is `{name}`")),
        )
    }
}
