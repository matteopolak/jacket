//! Resolve names and construct a typed program before code generation.
mod calls;
mod cases;
mod cleanup;
mod declarations;
mod loops;
mod modules;
use jai_eval::operators::Operator;
pub use jai_eval::operators::{Equality, IntOp, Relation};
use jai_eval::{Integer as IntegerValue, Value as ConstantValue};
use jai_source::{Diagnostic, Span, Symbol, Symbols};
use jai_syntax::{
    self as syntax, BinaryOp, CastMode, IntegerType, ReturnType, ScalarType, UnaryOp,
};
pub use modules::{Library, resolve_graph, resolve_library};
use std::collections::HashMap;

mod ir;
pub use ir::*;
use jai_types::{CallingConvention, ContextMode, ProcedureType, TypeId, TypeRegistry};

#[derive(Clone, Copy, Debug)]
enum Binding {
    Storage(Storage),
    Constant(ConstantValue),
}
struct ParameterSignature {
    name: Symbol,
    ty: ScalarType,
    default: Option<ConstantValue>,
}
struct Signature {
    ty: TypeId,
    id: ProcedureId,
    parameters: Vec<ParameterSignature>,
    result: ReturnType,
}
enum Expr {
    Literal(i128),
    WeakConditional(Box<Conditional<Expr>>),
    Int(IntExpr),
    Bool(BoolExpr),
    Void(Call),
}
impl Expr {
    fn int(self, span: Span) -> Result<IntExpr, Diagnostic> {
        match self {
            Self::Int(e) => Ok(e),
            Self::Literal(n) => Self::Literal(n).int_as(IntegerType::S64, span),
            Self::WeakConditional(e) => Self::WeakConditional(e).int_as(IntegerType::S64, span),
            _ => Err(Diagnostic::new(span, "expected int value")),
        }
    }
    fn int_as(self, ty: IntegerType, span: Span) -> Result<IntExpr, Diagnostic> {
        match self {
            Self::WeakConditional(e) => Ok(IntExpr::new(
                ty,
                IntExprKind::Conditional(Box::new(Conditional {
                    condition: e.condition,
                    then_value: e.then_value.int_as(ty, span)?,
                    else_value: e.else_value.int_as(ty, span)?,
                })),
            )),
            Self::Literal(n) => IntegerValue::checked(ty, n)
                .map(IntExpr::constant)
                .ok_or_else(|| {
                    Diagnostic::new(span, "integer constant is out of range for its target type")
                }),
            Self::Int(e) if e.ty() == ty => Ok(e),
            Self::Int(e) if ty.contains(e.ty()) => Ok(IntExpr::new(
                ty,
                IntExprKind::Cast(CastMode::Unchecked, Box::new(e)),
            )),
            _ => Err(Diagnostic::new(
                span,
                "implicit integer conversion does not preserve the source type's entire range",
            )),
        }
    }
    fn bool(self, span: Span) -> Result<BoolExpr, Diagnostic> {
        match self {
            Self::Bool(e) => Ok(e),
            _ => Err(Diagnostic::new(span, "expected bool value")),
        }
    }
    fn value(self, span: Span) -> Result<ValueExpr, Diagnostic> {
        match self {
            Self::Int(e) => Ok(ValueExpr::Int(e)),
            Self::Literal(n) => Self::Literal(n).int(span).map(ValueExpr::Int),
            Self::WeakConditional(e) => Self::WeakConditional(e).int(span).map(ValueExpr::Int),
            Self::Bool(e) => Ok(ValueExpr::Bool(e)),
            Self::Void(_) => Err(Diagnostic::new(span, "void call cannot supply a value")),
        }
    }
    fn condition(self, span: Span) -> Result<BoolExpr, Diagnostic> {
        match self {
            Self::Bool(e) => Ok(e),
            Self::Int(e) => Ok(BoolExpr::FromInt(Box::new(e))),
            Self::Literal(n) => Ok(BoolExpr::Constant(n != 0)),
            Self::WeakConditional(e) => Ok(BoolExpr::Conditional(Box::new(Conditional {
                condition: e.condition,
                then_value: e.then_value.condition(span)?,
                else_value: e.else_value.condition(span)?,
            }))),
            Self::Void(_) => Err(Diagnostic::new(span, "void call cannot supply a condition")),
        }
    }
    fn weak_integer(&self) -> bool {
        matches!(self, Self::Literal(_) | Self::WeakConditional(_))
    }
    fn cast_integer(
        self,
        ty: IntegerType,
        mode: CastMode,
        span: Span,
    ) -> Result<IntExpr, Diagnostic> {
        Ok(match self {
            Self::Literal(n) => match mode {
                CastMode::Unchecked => IntExpr::constant(IntegerValue::wrapping(ty, n)),
                CastMode::Checked => IntegerValue::checked(ty, n)
                    .map(IntExpr::constant)
                    .unwrap_or_else(|| IntExpr::new(ty, IntExprKind::InvalidCheckedCast)),
            },
            Self::WeakConditional(e) => IntExpr::new(
                ty,
                IntExprKind::Conditional(Box::new(Conditional {
                    condition: e.condition,
                    then_value: e.then_value.cast_integer(ty, mode, span)?,
                    else_value: e.else_value.cast_integer(ty, mode, span)?,
                })),
            ),
            Self::Int(e) => IntExpr::new(ty, IntExprKind::Cast(mode, Box::new(e))),
            Self::Bool(e) => IntExpr::new(ty, IntExprKind::FromBool(Box::new(e))),
            Self::Void(_) => {
                return Err(Diagnostic::new(
                    span,
                    "void call cannot be cast to an integer",
                ));
            }
        })
    }
    fn integer_type(&self, span: Span) -> Result<Option<IntegerType>, Diagnostic> {
        match self {
            Self::Int(e) => Ok(Some(e.ty())),
            Self::Literal(_) | Self::WeakConditional(_) => Ok(None),
            _ => Err(Diagnostic::new(span, "expected integer operands")),
        }
    }
}

pub fn resolve(module: &syntax::Module) -> Result<Program, Diagnostic> {
    declarations::check_top_level_names(module)?;
    let mut types = TypeRegistry::new();
    let (globals, global_bindings) = declarations::resolve_globals(module, &types)?;
    let mut signatures = HashMap::new();
    for (index, p) in module.procedures().iter().enumerate() {
        let parameters = calls::parameters(p, &global_bindings)?;
        let results = match p.return_type {
            ReturnType::Void => vec![],
            ReturnType::Value(ty) => vec![types.scalar(ty)],
        };
        let ty = types
            .procedure(ProcedureType {
                parameters: parameters.iter().map(|p| types.scalar(p.ty)).collect(),
                results: results.into_boxed_slice(),
                convention: CallingConvention::Jai,
                context: ContextMode::None,
            })
            .map_err(|error| Diagnostic::new(p.span, error.to_string()))?;
        let signature = Signature {
            ty,
            id: ProcedureId(index),
            parameters,
            result: p.return_type,
        };
        if signatures.insert(p.name, signature).is_some() {
            return Err(Diagnostic::new(
                p.span,
                format!("duplicate procedure '{}'", module.symbols().name(p.name)),
            ));
        }
    }
    let main = module
        .symbols()
        .find("main")
        .and_then(|s| signatures.get(&s))
        .ok_or_else(|| Diagnostic::new(Span::default(), "no main procedure"))?;
    if !main.parameters.is_empty() {
        return Err(Diagnostic::new(
            Span::default(),
            "main cannot take parameters in this compiler stage",
        ));
    }
    let entry = match main.result {
        ReturnType::Void => EntryPoint::Void(main.id),
        ReturnType::Value(ScalarType::Int(IntegerType::S64)) => EntryPoint::Int(main.id),
        ReturnType::Value(_) => {
            return Err(Diagnostic::new(
                Span::default(),
                "main must return int or void",
            ));
        }
    };
    let mut procedures = Vec::new();
    for (index, p) in module.procedures().iter().enumerate() {
        let mut r = Resolver {
            graph_scope: None,
            procedure: ProcedureId(index),
            types: &types,
            signatures: &signatures,
            symbols: module.symbols(),
            globals: &global_bindings,
            scopes: vec![HashMap::new()],
            locals: Vec::new(),
            span: p.span,
            result: p.return_type,
            loops: Vec::new(),
            next_loop: 0,
            cleanups: Vec::new(),
            deferred_scopes: Vec::new(),
            cleanup_context: None,
        };
        let mut parameters = Vec::new();
        for param in &signatures[&p.name].parameters {
            parameters.push(r.declare(param.name, param.ty)?);
        }
        let body = r.block(&p.body, false)?;
        if p.return_type != ReturnType::Void && body.flow != Flow::Terminates {
            return Err(Diagnostic::new(
                p.span,
                "value-returning procedure may reach its end",
            ));
        }
        procedures.push(Procedure {
            id: ProcedureId(index),
            parameters,
            locals: r.locals,
            cleanups: r.cleanups,
            signature: signatures[&p.name].ty,
            body,
        });
    }
    Ok(Program {
        procedures,
        entry,
        globals,
        types: types
            .freeze()
            .map_err(|error| Diagnostic::new(Span::default(), error.to_string()))?,
    })
}
struct LoopBinding {
    id: LoopId,
    name: Option<Symbol>,
    cleanup_depth: usize,
}
#[derive(Clone, Copy)]
struct CleanupContext {
    loop_depth: usize,
}
struct Resolver<'a> {
    graph_scope: Option<modules::FileScope<'a>>,
    procedure: ProcedureId,
    types: &'a TypeRegistry,
    signatures: &'a HashMap<Symbol, Signature>,
    symbols: &'a Symbols,
    scopes: Vec<HashMap<Symbol, Binding>>,
    globals: &'a HashMap<Symbol, Binding>,
    locals: Vec<Local>,
    span: Span,
    result: ReturnType,
    loops: Vec<LoopBinding>,
    next_loop: usize,
    cleanups: Vec<Block>,
    deferred_scopes: Vec<Vec<CleanupId>>,
    cleanup_context: Option<CleanupContext>,
}
impl Resolver<'_> {
    fn error(&self, text: impl Into<String>) -> Diagnostic {
        Diagnostic::new(self.span, text)
    }
    fn lookup(&self, name: Symbol) -> Result<Binding, Diagnostic> {
        self.lookup_path(
            &syntax::NamePath {
                root: name,
                members: Vec::new(),
            },
            self.span,
        )
    }
    fn lookup_path(&self, path: &syntax::NamePath, span: Span) -> Result<Binding, Diagnostic> {
        if let Some(value) = self
            .scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(&path.root))
            .copied()
        {
            if path.members.is_empty() {
                return Ok(value);
            }
            return Err(Diagnostic::new(span, "scalar value is not a namespace"));
        }
        if let Some(scope) = self.graph_scope {
            return scope.value(path, span);
        }
        if !path.members.is_empty() {
            return Err(Diagnostic::new(
                span,
                "qualified name requires a module scope",
            ));
        }
        self.globals.get(&path.root).copied().ok_or_else(|| {
            Diagnostic::new(
                span,
                format!("unknown variable '{}'", self.symbols.name(path.root)),
            )
        })
    }
    fn storage(&self, name: Symbol) -> Result<Storage, Diagnostic> {
        match self.lookup(name)? {
            Binding::Storage(storage) => Ok(storage),
            Binding::Constant(_) => Err(self.error("cannot assign to a constant")),
        }
    }
    fn bind_name(&mut self, name: Symbol, binding: Binding) -> Result<(), Diagnostic> {
        let scope = self.scopes.last_mut().expect("resolver always has a scope");
        if scope.contains_key(&name) {
            return Err(self.error(format!("duplicate variable '{}'", self.symbols.name(name))));
        }
        scope.insert(name, binding);
        Ok(())
    }
    fn bind(&mut self, name: Symbol, local: Local) -> Result<(), Diagnostic> {
        self.bind_name(name, Binding::Storage(Storage::local(local, self.types)))?;
        self.locals.push(local);
        Ok(())
    }
    fn allocate(&mut self, ty: ScalarType) -> Local {
        let local = Local::new(self.procedure, self.locals.len(), ty, self.types);
        self.locals.push(local);
        local
    }
    fn declare_int(&mut self, name: Symbol, ty: IntegerType) -> Result<IntLocal, Diagnostic> {
        let local = self.declare(name, ScalarType::Int(ty))?;
        Ok(local.integer(self.types).expect("integer declaration"))
    }
    fn declare_bool(&mut self, name: Symbol) -> Result<BoolLocal, Diagnostic> {
        let local = self.declare(name, ScalarType::Bool)?;
        Ok(local.boolean(self.types).expect("boolean declaration"))
    }
    fn declare(&mut self, name: Symbol, ty: ScalarType) -> Result<Local, Diagnostic> {
        let local = Local::new(self.procedure, self.locals.len(), ty, self.types);
        self.bind(name, local)?;
        Ok(local)
    }
    fn store(&self, local: Storage, value: Expr) -> Result<Statement, Diagnostic> {
        Ok(match local {
            Storage::Int(id) => Statement::StoreInt(id, value.int_as(id.ty(), self.span)?),
            Storage::Bool(id) => Statement::StoreBool(id, value.bool(self.span)?),
        })
    }
    fn block(
        &mut self,
        statements: &[syntax::Statement],
        scoped: bool,
    ) -> Result<Block, Diagnostic> {
        if scoped {
            self.scopes.push(HashMap::new());
        }
        self.bind_block_constants(statements)?;
        self.deferred_scopes.push(Vec::new());
        let mut out = Vec::new();
        let mut flow = Flow::FallsThrough;
        for statement in statements {
            if matches!(statement, syntax::Statement::Constant(_)) {
                continue;
            }
            if flow == Flow::Terminates {
                return Err(self.error("unreachable statement"));
            }
            if let syntax::Statement::Defer(body) = statement {
                self.register_defer(body)?;
                continue;
            }
            let s = self.statement(statement)?;
            flow = match &s {
                Statement::Exit(_) => Flow::Terminates,
                Statement::If(_, yes, no)
                    if yes.flow == Flow::Terminates && no.flow == Flow::Terminates =>
                {
                    Flow::Terminates
                }
                Statement::Cases(c) => c.flow,
                Statement::Block(b) => b.flow,
                _ => Flow::FallsThrough,
            };
            out.push(s);
        }
        let pending = self
            .deferred_scopes
            .pop()
            .expect("each block has a cleanup scope");
        if flow == Flow::FallsThrough {
            out.extend(pending.into_iter().rev().map(Statement::Cleanup));
        }
        if scoped {
            self.scopes.pop();
        }
        Ok(Block {
            statements: out,
            flow,
        })
    }
    fn statement(&mut self, statement: &syntax::Statement) -> Result<Statement, Diagnostic> {
        Ok(match statement {
            syntax::Statement::Declare(decl) => {
                let (name, ty, value) = match decl {
                    syntax::Declaration::UnresolvedExplicit { .. } => {
                        return Err(self.error("local aggregate types are not implemented"));
                    }
                    syntax::Declaration::Inferred { name, initializer } => {
                        let value = self.expr(initializer)?.value(initializer.span)?;
                        let (ty, value) = match value {
                            ValueExpr::Int(e) => (ScalarType::Int(e.ty()), Expr::Int(e)),
                            ValueExpr::Bool(e) => (ScalarType::Bool, Expr::Bool(e)),
                        };
                        (*name, ty, value)
                    }
                    syntax::Declaration::Explicit {
                        name,
                        ty,
                        initializer,
                    } => {
                        let value = match initializer {
                            Some(e) => self.expr(e)?,
                            None => match ty {
                                ScalarType::Int(ty) => {
                                    Expr::Int(IntExpr::constant(IntegerValue::wrapping(*ty, 0)))
                                }
                                ScalarType::Bool => Expr::Bool(BoolExpr::Constant(false)),
                            },
                        };
                        (*name, *ty, value)
                    }
                };
                let local = self.declare(name, ty)?;
                self.store(Storage::local(local, self.types), value)?
            }
            syntax::Statement::Assign(name, e) => {
                let local = self.storage(*name)?;
                let value = self.expr(e)?;
                self.store(local, value)?
            }
            syntax::Statement::Update(name, op, e) => {
                let local = self.storage(*name)?;
                let lhs = match local {
                    Storage::Int(id) => Expr::Int(IntExpr::load(id)),
                    Storage::Bool(id) => Expr::Bool(BoolExpr::Load(id)),
                };
                let value = self.binary(*op, lhs, self.expr(e)?, e.span)?;
                self.store(local, value)?
            }
            syntax::Statement::Return(e) => self.resolve_return(e.as_ref())?,
            syntax::Statement::Defer(_) | syntax::Statement::Constant(_) => {
                unreachable!("declarations are handled by block resolution")
            }
            syntax::Statement::Expression(e) => match self.expr(e)? {
                Expr::Literal(n) => Statement::DiscardInt(Expr::Literal(n).int(e.span)?),
                Expr::WeakConditional(value) => {
                    Statement::DiscardInt(Expr::WeakConditional(value).int(e.span)?)
                }
                Expr::Int(e) => Statement::DiscardInt(e),
                Expr::Bool(e) => Statement::DiscardBool(e),
                Expr::Void(c) => Statement::CallVoid(c),
            },
            syntax::Statement::If(cond, yes, no) => Statement::If(
                self.expr(cond)?.condition(cond.span)?,
                self.block(yes, true)?,
                self.block(no, true)?,
            ),
            syntax::Statement::Cases(c) => self.resolve_cases(c)?,
            syntax::Statement::While(condition, body) => self.resolve_while(condition, body)?,
            syntax::Statement::Range(range) => self.resolve_range(range)?,
            syntax::Statement::Jump { kind, target, span } => {
                self.resolve_jump(*kind, *target, *span)?
            }
            syntax::Statement::Block(body) => Statement::Block(self.block(body, true)?),
        })
    }
    fn expr(&self, expr: &syntax::Expression) -> Result<Expr, Diagnostic> {
        let span = expr.span;
        Ok(match &expr.kind {
            syntax::ExpressionKind::Integer(n) => Expr::Literal(*n),
            syntax::ExpressionKind::Bool(b) => Expr::Bool(BoolExpr::Constant(*b)),
            syntax::ExpressionKind::Name(name) => match self.lookup_path(
                &syntax::NamePath {
                    root: *name,
                    members: Vec::new(),
                },
                span,
            )? {
                Binding::Storage(Storage::Int(id)) => Expr::Int(IntExpr::load(id)),
                Binding::Storage(Storage::Bool(id)) => Expr::Bool(BoolExpr::Load(id)),
                Binding::Constant(value) => Self::constant(value),
            },
            syntax::ExpressionKind::Call(name, args) => self.resolve_call(*name, args, span)?,
            syntax::ExpressionKind::QualifiedName(path) => match self.lookup_path(path, span)? {
                Binding::Storage(Storage::Int(id)) => Expr::Int(IntExpr::load(id)),
                Binding::Storage(Storage::Bool(id)) => Expr::Bool(BoolExpr::Load(id)),
                Binding::Constant(value) => Self::constant(value),
            },
            syntax::ExpressionKind::QualifiedCall(path, args) => {
                self.resolve_call_path(path, args, span)?
            }
            syntax::ExpressionKind::StructLiteral(_) | syntax::ExpressionKind::Member { .. } => {
                return Err(Diagnostic::new(
                    span,
                    "aggregate expression lowering is not implemented",
                ));
            }
            syntax::ExpressionKind::Conditional(e) => {
                let condition = self.expr(&e.condition)?.condition(e.condition.span)?;
                let yes = self.expr(&e.then_value)?;
                let no = e.else_value.as_ref().map(|e| self.expr(e)).transpose()?;
                match yes {
                    Expr::Bool(yes) => Expr::Bool(BoolExpr::Conditional(Box::new(Conditional {
                        condition,
                        then_value: yes,
                        else_value: match no {
                            Some(e) => e.bool(span)?,
                            None => BoolExpr::Constant(false),
                        },
                    }))),
                    Expr::Void(_) => {
                        return Err(Diagnostic::new(
                            span,
                            "void call cannot supply an ifx result",
                        ));
                    }
                    yes => {
                        let no = no.unwrap_or(Expr::Literal(0));
                        if yes.weak_integer() && no.weak_integer() {
                            return Ok(Expr::WeakConditional(Box::new(Conditional {
                                condition,
                                then_value: yes,
                                else_value: no,
                            })));
                        }
                        let (yes, no) = Self::integer_pair(yes, no, span)?;
                        Expr::Int(IntExpr::new(
                            yes.ty(),
                            IntExprKind::Conditional(Box::new(Conditional {
                                condition,
                                then_value: yes,
                                else_value: no,
                            })),
                        ))
                    }
                }
            }
            syntax::ExpressionKind::Unary(op, e) => {
                let value = self.expr(e)?;
                match (op, value) {
                    (UnaryOp::LogicalNot, e) => {
                        Expr::Bool(BoolExpr::Not(Box::new(e.condition(span)?)))
                    }
                    (UnaryOp::Positive, Expr::Literal(n)) => Expr::Literal(n),
                    (UnaryOp::Positive, e) => Expr::Int(e.int(span)?),
                    (UnaryOp::Negate, Expr::Literal(n)) => Expr::Literal(
                        n.checked_neg()
                            .ok_or_else(|| Diagnostic::new(span, "integer literal overflow"))?,
                    ),
                    (UnaryOp::Complement, Expr::Literal(n)) => Expr::Literal(!n),
                    (op, e) => {
                        let e = e.int(span)?;
                        let ty = e.ty();
                        Expr::Int(IntExpr::new(
                            ty,
                            match op {
                                UnaryOp::Negate => IntExprKind::Negate(Box::new(e)),
                                UnaryOp::Complement => IntExprKind::Complement(Box::new(e)),
                                _ => unreachable!(),
                            },
                        ))
                    }
                }
            }
            syntax::ExpressionKind::Cast(mode, ty, e) => {
                let value = self.expr(e)?;
                match ty {
                    ScalarType::Bool => Expr::Bool(value.condition(span)?),
                    ScalarType::Int(ty) => Expr::Int(value.cast_integer(*ty, *mode, span)?),
                }
            }
            syntax::ExpressionKind::Binary(op, a, b) => {
                self.binary(*op, self.expr(a)?, self.expr(b)?, span)?
            }
        })
    }
    fn constant(value: ConstantValue) -> Expr {
        match value {
            ConstantValue::Literal(n) => Expr::Literal(n),
            ConstantValue::Int(n) => Expr::Int(IntExpr::constant(n)),
            ConstantValue::Bool(b) => Expr::Bool(BoolExpr::Constant(b)),
        }
    }
    fn integer_pair(a: Expr, b: Expr, span: Span) -> Result<(IntExpr, IntExpr), Diagnostic> {
        let ty = match (a.integer_type(span)?, b.integer_type(span)?) {
            (None, None) => IntegerType::S64,
            (Some(ty), None) | (None, Some(ty)) => ty,
            (Some(a), Some(b)) => a.common(b).ok_or_else(|| {
                Diagnostic::new(span, "integer operands have incompatible ranges")
            })?,
        };
        Ok((a.int_as(ty, span)?, b.int_as(ty, span)?))
    }
    fn binary(&self, op: BinaryOp, a: Expr, b: Expr, span: Span) -> Result<Expr, Diagnostic> {
        if let (Expr::Literal(a), Expr::Literal(b)) = (&a, &b)
            && let Ok(value) = jai_eval::binary_literals(op, *a, *b, span)
        {
            return Ok(Self::constant(value));
        }
        Ok(match Operator::from(op) {
            Operator::Integer(op) => {
                let (a, b) = Self::integer_pair(a, b, span)?;
                Expr::Int(IntExpr::new(
                    a.ty(),
                    IntExprKind::Binary(op, Box::new(a), Box::new(b)),
                ))
            }
            Operator::Relation(op) => {
                let (a, b) = Self::integer_pair(a, b, span)?;
                Expr::Bool(BoolExpr::CompareInts(op, Box::new(a), Box::new(b)))
            }
            Operator::Equality(op) => match (a, b) {
                (Expr::Bool(a), Expr::Bool(b)) => {
                    Expr::Bool(BoolExpr::CompareBools(op, Box::new(a), Box::new(b)))
                }
                (a, b) => {
                    let (a, b) = Self::integer_pair(a, b, span)?;
                    Expr::Bool(BoolExpr::CompareInts(
                        match op {
                            Equality::Equal => Relation::Equal,
                            Equality::NotEqual => Relation::NotEqual,
                        },
                        Box::new(a),
                        Box::new(b),
                    ))
                }
            },
            Operator::And => Expr::Bool(BoolExpr::And(
                Box::new(a.condition(span)?),
                Box::new(b.condition(span)?),
            )),
            Operator::Or => Expr::Bool(BoolExpr::Or(
                Box::new(a.condition(span)?),
                Box::new(b.condition(span)?),
            )),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn check(source: &str) -> Result<Program, Diagnostic> {
        resolve(&syntax::parse(source).unwrap())
    }
    #[test]
    fn conditional_results_require_matching_non_void_types() {
        for source in [
            "main :: ()->int { return ifx true then 1 else false; }",
            "main :: ()->bool { return ifx false then false else 1; }",
            "f :: () {} main :: ()->int { return ifx true then f() else 1; }",
            "f :: () {} main :: ()->int { return ifx false then 1 else f(); }",
            "f :: () {} main :: ()->int { return ifx f() then 1 else 2; }",
            "main :: ()->int { return ifx true then 1 else missing; }",
            "N :: ifx true then 1 else M; M :: N; main :: ()->int { return N; }",
        ] {
            assert!(check(source).is_err(), "{source}");
        }
        assert!(
            check("N :: ifx true then M else 1 / 0; M :: 42; main :: ()->int { return N; }")
                .is_ok()
        );
    }
    #[test]
    fn integer_conversions_reject_range_loss_and_oversized_literals() {
        for source in [
            "main :: () { n:u8 = 256; }",
            "main :: () { n:s8 = 128; }",
            "main :: () { n:u64 = -1; }",
            "main :: () { n := 18446744073709551615; }",
            "main :: () { a:u16=42; b:u8=a; }",
            "main :: () { a:s8=42; b:u64=a; }",
            "main :: () { a:u64=42; b:s64=a; }",
            "main :: () { a:s8=1; b:u8=1; c:=a+b; }",
            "f :: (n:u8) {} main :: () { f(256); }",
            "f :: ()->u8 { n:u16=1; return n; } main :: () {}",
            "N : u8 : 256; main :: () {}",
            "n:u8 = 256; main :: () {}",
            "main :: () { n := +true; }",
        ] {
            assert!(check(source).is_err(), "{source}");
        }
        assert!(check("N :: 255; main :: () { n:u8=N; large:u64=18446744073709551615; }").is_ok());
    }
    #[test]
    fn reject_unresolved_names_and_arity() {
        for src in [
            "main :: () { x = 1; }",
            "main :: () { missing(); }",
            "f :: (x:int) {} main :: () { f(); }",
        ] {
            assert!(check(src).is_err(), "{src}");
        }
    }
    #[test]
    fn preserve_scalar_types() {
        for src in [
            "main :: () { x: int = true; }",
            "main :: () { x: bool = 1; }",
            "main :: ()->int { return true; }",
            "f :: (b:bool) {} main :: () { f(1); }",
            "main :: () { x := true + false; }",
        ] {
            assert!(check(src).is_err(), "{src}");
        }
    }
    #[test]
    fn void_values_cannot_escape() {
        assert!(check("f :: () {} main :: () { f(); }").is_ok());
        assert!(check("f :: () {} main :: () { x := f(); }").is_err());
    }
    #[test]
    fn return_flow_and_scope() {
        for src in [
            "main :: ()->int {}",
            "main :: () { return; x := 1; }",
            "main :: () { { x := 1; } x = 2; }",
            "main :: () { x := 1; x := 2; }",
        ] {
            assert!(check(src).is_err(), "{src}");
        }
        assert!(check("main :: ()->int { if true return 1; else return 2; }").is_ok());
    }
    #[test]
    fn jumps_require_active_loop_targets_and_reachable_code() {
        for source in [
            "main :: () { break; }",
            "main :: () { continue; }",
            "main :: () { x := 1; while true { break x; } }",
            "main :: () { for i: 1..3 {} break i; }",
            "main :: () { while true { break; x := 1; } }",
            "main :: () { while true { if true continue; else break; x := 1; } }",
            "main :: ()->int { while true { break; } }",
        ] {
            assert!(check(source).is_err(), "{source}");
        }
    }
    #[test]
    fn iterator_and_while_bindings_are_scoped_and_typed() {
        for source in [
            "main :: () { for true..4 {} }",
            "main :: () { for 1..false {} }",
            "main :: () { for i: 1..3 {} i = 4; }",
            "main :: () { while value := true {} value = false; }",
            "main :: () { while value := true { value = 1; } }",
            "f :: () {} main :: () { while value := f() {} }",
        ] {
            assert!(check(source).is_err(), "{source}");
        }
        assert!(check("main :: () { for i: 1..3 { for i: 1..3 break i; continue i; } }").is_ok());
    }
    #[test]
    fn cleanup_cannot_escape_to_its_enclosing_procedure_or_loop() {
        for source in [
            "main :: () { defer return; }",
            "main :: () { for i: 1..3 { defer break i; } }",
            "main :: () { while true { defer continue; } }",
            "main :: () { defer x = 1; x := 0; }",
            "main :: () { defer { x := 1; } x = 2; }",
            "main :: () { return; defer {} }",
        ] {
            assert!(check(source).is_err(), "{source}");
        }
        assert!(check("main :: () { defer { for i: 1..3 { if i == 2 break i; } } }").is_ok());
    }
    #[test]
    fn constants_are_immutable_and_dependencies_must_be_acyclic() {
        for source in [
            "A :: B; B :: A; main :: () {}",
            "A :: A; main :: () {}",
            "A :: 1; main :: () { A = 2; }",
            "main :: () { A :: 1; A += 2; }",
            "main :: (x:int) { A :: x; }",
            "main :: () { x := 1; { A :: x; } }",
            "A :: missing; main :: () {}",
            "A :: 1 / 0; main :: () {}",
            "A : bool : 1; main :: () {}",
            "A :: 1; A := 2; main :: () {}",
            "main :: () { A :: 1; A :: 2; }",
        ] {
            assert!(check(source).is_err(), "{source}");
        }
    }
    #[test]
    fn global_initializers_and_scalar_calls_are_checked() {
        for source in [
            "x : bool = 1; main :: () {}",
            "x : int = true; main :: () {}",
            "x := y; y := 3; main :: () {}",
            "f :: ()->int { return 1; } x := f(); main :: () {}",
            "main :: () {} main := 0;",
            "f :: () {} main :: () { f := 2; f(); }",
        ] {
            assert!(check(source).is_err(), "{source}");
        }
    }
    #[test]
    fn deep_constant_dependency_chains_use_a_worklist() {
        use std::fmt::Write;
        let mut source = String::new();
        for n in 0..20_000 {
            writeln!(source, "C{n} :: C{} + 1;", n + 1).unwrap();
        }
        source.push_str("C20000 :: 0; main :: ()->int { return C0; }");
        assert!(check(&source).is_ok());
    }
    #[test]
    fn entry_point_and_parameter_invariants() {
        for src in [
            "f :: () {}",
            "main :: (x:int) {}",
            "main :: ()->bool { return true; }",
            "f :: (x:int,x:bool) {} main :: () {}",
        ] {
            assert!(check(src).is_err(), "{src}");
        }
    }
}
