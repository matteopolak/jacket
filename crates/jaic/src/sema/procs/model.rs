//! Procedure records: targets, parameters, signatures and the polymorphism helpers over headers.
use crate::diagnostics::Code;
use crate::sema::{
    HashMap, InstanceKeyValue, ProcId, ProcTypeParams, Rc, Result, ScopeId, Span, Sym, TypeId, ast,
    err, ir,
};

#[derive(Clone, Copy, Debug)]
pub enum ProcTarget {
    Func(ir::FuncId),
    Foreign(ir::ForeignId),
}

#[derive(Clone, Debug)]
pub struct ParamInfo {
    pub name: Option<Sym>,
    pub ty: TypeId,
    pub default: Option<ast::Expr>,
    pub callable: Option<Rc<ProcTypeParams>>,
    pub variadic: bool,
    pub using: bool,
    pub using_filter: Option<ast::UsingFilter>,
    /// `#discard`: callers pass no real value and the body may not name it.
    pub discard: bool,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct Signature {
    pub params: Vec<ParamInfo>,
    pub returns: Vec<TypeId>,
    pub return_callables: Vec<Option<Rc<ProcTypeParams>>>,
    pub return_names: Vec<Option<Sym>>,
    pub ty: TypeId,
    pub has_context: bool,
    pub c_call: bool,
    pub c_varargs: bool,
    /// Scope the parameter types were resolved in (holds polymorphic bindings).
    pub scope: ScopeId,
}

/// C ellipses consume the trailing ABI arguments. Baked parameters do not
/// participate in that ABI, unlike ordinary runtime parameters.
pub(in crate::sema) fn validate_c_variadic_header(header: &ast::ProcHeader) -> Result<()> {
    if !header.uses_c_abi() {
        return Ok(());
    }
    let mut runtime = header.params.iter().filter(|p| !p.baked).peekable();
    while let Some(param) = runtime.next() {
        if param.variadic && runtime.peek().is_some() {
            return err(
                Code::ProcCVariadicParameter,
                param.span,
                "a C variadic parameter must be the last runtime parameter",
            );
        }
    }
    Ok(())
}

pub struct ProcInfo {
    pub name: Sym,
    pub lit: Rc<ast::ProcLit>,
    /// Defining scope.
    pub scope: ScopeId,
    pub span: Span,
    pub is_poly: bool,
    /// Parameter types were checked for bare polymorphic structs (implicit polymorphism).
    pub poly_checked: bool,
    pub is_macro: bool,
    pub sig: Option<Rc<Signature>>,
    pub sig_resolving: bool,
    pub target: Option<ProcTarget>,
    pub body_state: BodyState,
    pub instances: HashMap<Vec<InstanceKeyValue>, ProcId>,
    /// For instances: scope with the polymorphic bindings (parent = defining scope).
    pub bindings: Option<ScopeId>,
    pub export: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BodyState {
    NotNeeded,
    Queued,
    Lowering,
    Done,
}

/// Does a type expression contain `$T` binders?
pub fn has_poly(expr: &ast::Expr) -> bool {
    use ast::ExprKind as E;
    match &expr.kind {
        E::PolyVar {
            ..
        }
        | E::PolyRestricted {
            ..
        } => true,
        E::Unary(_, e) => has_poly(e),
        E::ArrayType {
            size,
            elem,
        } => {
            has_poly(elem)
                || match size {
                    ast::ArraySize::Fixed(n) => has_poly(n),
                    _ => false,
                }
        }
        E::Call {
            callee,
            args,
            ..
        } => has_poly(callee) || args.iter().any(|a| has_poly(&a.value)),
        E::ProcType(h) => {
            h.params.iter().any(|p| p.ty.as_ref().is_some_and(has_poly))
                || h.returns
                    .iter()
                    .any(|r| r.ty.as_ref().is_some_and(has_poly))
        }
        E::TypeDirective {
            ty, ..
        } => has_poly(ty),
        _ => false,
    }
}

/// Names introduced by a polymorphic type pattern, in pattern order. Shared
/// source reflection binds these symbolically; instances bind concrete values.
pub(in crate::sema) fn for_poly_binders(expr: &ast::Expr, visit: &mut impl FnMut(Sym)) {
    use ast::ExprKind as E;
    match &expr.kind {
        E::PolyVar {
            name, ..
        }
        | E::PolyRestricted {
            name, ..
        } => visit(*name),
        E::Unary(_, e)
        | E::TypeDirective {
            ty: e, ..
        } => for_poly_binders(e, visit),
        E::ArrayType {
            size,
            elem,
        } => {
            if let ast::ArraySize::Fixed(n) = size {
                for_poly_binders(n, visit);
            }
            for_poly_binders(elem, visit);
        }
        E::Call {
            callee,
            args,
            ..
        } => {
            for_poly_binders(callee, visit);
            for arg in args {
                for_poly_binders(&arg.value, visit);
            }
        }
        E::ProcType(header) => {
            for ty in header
                .params
                .iter()
                .filter_map(|p| p.ty.as_ref())
                .chain(header.returns.iter().filter_map(|r| r.ty.as_ref()))
            {
                for_poly_binders(ty, visit);
            }
        }
        _ => {}
    }
}
