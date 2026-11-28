//! Procedure records: targets, parameters, signatures and the polymorphism helpers over headers.
use crate::diagnostics::Code;
use crate::sema::{
    Arc, HashMap, InstanceKeyValue, ProcId, ProcTypeParams, Result, ScopeId, Span, Sym, TypeId,
    ast, err, ir,
};

/// What a procedure becomes in the program.
#[derive(Clone, Copy, Debug)]
pub enum ProcTarget {
    Func(ir::FuncId),
    Foreign(ir::ForeignId),
}

/// A declared parameter of a procedure.
#[derive(Clone, Debug)]
pub struct ParamInfo {
    pub name: Option<Sym>,
    pub ty: TypeId,
    pub default: Option<ast::Expr>,
    /// Names and defaults of the parameter's procedure type, when it has them.
    pub callable: Option<Arc<ProcTypeParams>>,
    pub variadic: bool,
    pub using: bool,
    pub using_filter: Option<ast::UsingFilter>,
    /// `#discard`: callers pass no real value and the body may not name it.
    pub discard: bool,
    pub span: Span,
}

/// The resolved signature of a procedure.
#[derive(Clone, Debug)]
pub struct Signature {
    pub params: Vec<ParamInfo>,
    pub returns: Vec<TypeId>,
    /// Parallel to `returns`: the parameter names and defaults of a returned procedure type, if it has them.
    pub return_callables: Vec<Option<Arc<ProcTypeParams>>>,
    pub return_names: Vec<Option<Sym>>,
    /// The procedure's type.
    pub ty: TypeId,
    /// Whether the procedure receives the implicit `context`: false for `#c_call`, `#no_context` and intrinsics.
    pub has_context: bool,
    /// Whether the procedure uses the C ABI.
    pub c_call: bool,
    /// Whether a C-ABI procedure ends in a variadic parameter; that parameter is left out of `ty`'s `params`.
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

/// A procedure the compiler knows about: its literal, its resolved signature, its target and the state of its body. A polymorphic procedure keeps its instances here.
pub struct ProcInfo {
    pub name: Sym,
    pub lit: Arc<ast::ProcLit>,
    /// Defining scope.
    pub scope: ScopeId,
    pub span: Span,
    pub is_poly: bool,
    /// Parameter types were checked for bare polymorphic structs (implicit polymorphism).
    pub poly_checked: bool,
    pub is_macro: bool,
    /// `None` until the signature is resolved.
    pub sig: Option<Arc<Signature>>,
    /// Set while the signature is being resolved, so that a signature that depends on itself is reported instead of recursing.
    pub sig_resolving: bool,
    /// The function or foreign symbol of the procedure, once it has one.
    pub target: Option<ProcTarget>,
    pub body_state: BodyState,
    /// Instances of a polymorphic procedure, by the values bound to its compile-time parameters.
    pub instances: HashMap<Vec<InstanceKeyValue>, ProcId>,
    /// For instances: scope with the polymorphic bindings (parent = defining scope).
    pub bindings: Option<ScopeId>,
    /// The symbol of a `#program_export` procedure: the name given, or the procedure's own.
    pub export: Option<String>,
}

/// How far a procedure's body has got.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BodyState {
    NotNeeded,
    /// Needed by a call but left unchecked: a library procedure no answer comes from
    /// (`Options::defer_library_bodies`). `lower_body` checks it when something asks.
    Deferred,
    Queued,
    Lowering,
    Done,
}

impl ProcInfo {
    /// Whether the body is still to be checked, now or when something asks for it.
    pub fn needs_body(&self) -> bool {
        matches!(self.body_state, BodyState::Queued | BodyState::Deferred)
    }
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
