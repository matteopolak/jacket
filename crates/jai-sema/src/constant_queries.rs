//! Source constantness queries inspect checked facts without evaluating runtime operands.
use crate::overloads::ArgumentInfo;
use crate::{Expr, Resolver, ScalarConstant};
use jai_source::{Diagnostic, Span};
use jai_syntax::{self as syntax, ExpressionKind as E};

impl Resolver<'_> {
    fn is_constant_query(&self, path: &syntax::NamePath) -> bool {
        path.members.is_empty()
            && self.symbols.name(path.root) == "is_constant"
            && !self.local_name_present(path.root)
            && !self.globals.contains_key(&path.root)
            && !self.signatures.contains_key(&path.root)
            && !self
                .graph_scope
                .is_some_and(|scope| scope.has_source_name(path))
    }

    fn constant_query_operand<'a>(
        &self,
        args: &'a [syntax::CallArgument],
        span: Span,
    ) -> Result<&'a syntax::Expression, Diagnostic> {
        let [argument] = args else {
            return Err(Diagnostic::new(span, "is_constant requires one argument"));
        };
        if argument.name.is_some() || argument.spread {
            return Err(Diagnostic::new(
                span,
                "is_constant requires one positional argument",
            ));
        }
        Ok(&argument.value)
    }

    pub(crate) fn describe_constant_query_call(
        &mut self,
        path: &syntax::NamePath,
        args: &[syntax::CallArgument],
        span: Span,
    ) -> Result<Option<ArgumentInfo>, Diagnostic> {
        if !self.is_constant_query(path) {
            return Ok(None);
        }
        let source = self.constant_query_operand(args, span)?;
        if let E::CompileTime(run) = &source.kind {
            // A description proves its type only. The selected query still
            // executes the explicit directive through the shared #run engine.
            self.validate_discarded_expression(source)?;
            if let syntax::CompileTimeBody::Expression(value) = &run.body {
                self.source_is_constant(value)?;
            }
            return Ok(Some(ArgumentInfo::typed(
                self.types.scalar(jai_types::ScalarType::Bool),
            )));
        }
        let constant = self.source_is_constant(source)?;
        Ok(Some(ArgumentInfo::scalar_constant(
            ScalarConstant::Bool(constant),
            self.types,
        )))
    }

    pub(crate) fn constant_query_call(
        &mut self,
        path: &syntax::NamePath,
        args: &[syntax::CallArgument],
        span: Span,
    ) -> Result<Option<Expr>, Diagnostic> {
        if !self.is_constant_query(path) {
            return Ok(None);
        }
        let source = self.constant_query_operand(args, span)?;
        let constant = if let E::CompileTime(run) = &source.kind {
            self.execute_compile_time(run, source.span, None)?;
            true
        } else {
            self.source_is_constant(source)?
        };
        Ok(Some(Expr::Bool(jai_ir::BoolExpr::Constant(constant))))
    }

    pub(crate) fn optional_baking_needs_materialization(
        &mut self,
        source: &syntax::Expression,
    ) -> Result<bool, Diagnostic> {
        Ok(self.source_constantness(source, 0)? == Constantness::Deferred)
    }

    fn source_constantness(
        &mut self,
        source: &syntax::Expression,
        depth: usize,
    ) -> Result<Constantness, Diagnostic> {
        if depth >= crate::constant_limits::MAX_CONSTANT_DEPTH {
            return Err(Diagnostic::new(
                source.span,
                "constant query exceeds source depth",
            ));
        }
        if matches!(source.kind, E::CompileTime(_)) {
            return Ok(Constantness::Deferred);
        }
        if let E::Cast(jai_types::CastMode::Force(_), _, value)
        | E::InferredCast {
            mode: jai_types::CastMode::Force(_),
            value,
        }
        | E::TypeCast {
            mode: jai_types::CastMode::Force(_),
            value,
            ..
        } = &source.kind
        {
            return Ok(match self.source_constantness(value, depth + 1)? {
                Constantness::Runtime => Constantness::Runtime,
                Constantness::Constant | Constantness::Deferred => Constantness::Deferred,
            });
        }
        if self.describe_argument(source)?.is_compile_time_constant() {
            return Ok(Constantness::Constant);
        }
        let next = depth + 1;
        Ok(match &source.kind {
            E::Cast(_, _, value) | E::TypeCast { value, .. } => {
                match self.source_constantness(value, next)? {
                    Constantness::Runtime => Constantness::Runtime,
                    Constantness::Constant | Constantness::Deferred => Constantness::Deferred,
                }
            }
            E::Unary(_, value)
            | E::InferredCast { value, .. }
            | E::CallHint { call: value, .. } => self.source_constantness(value, next)?,
            E::Binary(_, left, right) => self
                .source_constantness(left, next)?
                .combine(self.source_constantness(right, next)?),
            E::Conditional(value) => {
                let mut result = self
                    .source_constantness(&value.condition, next)?
                    .combine(self.source_constantness(&value.then_value, next)?);
                if let Some(value) = &value.else_value {
                    result = result.combine(self.source_constantness(value, next)?);
                }
                result
            }
            E::ArrayLiteral(value) => {
                let mut result = Constantness::Constant;
                for element in &value.elements {
                    result = result.combine(self.source_constantness(element, next)?);
                }
                result
            }
            E::StructLiteral(value) => {
                let mut result = Constantness::Constant;
                for field in &value.fields {
                    result = result.combine(self.source_constantness(&field.value, next)?);
                }
                result
            }
            E::PositionalStructLiteral(value) => {
                let mut result = Constantness::Constant;
                for element in &value.values {
                    result = result.combine(self.source_constantness(element, next)?);
                }
                result
            }
            E::Call(name, arguments)
                if self.is_constant_query(&syntax::NamePath {
                    root: *name,
                    members: vec![],
                }) =>
            {
                let operand = self.constant_query_operand(arguments, source.span)?;
                if self.source_constantness(operand, next)? == Constantness::Deferred {
                    Constantness::Deferred
                } else {
                    Constantness::Constant
                }
            }
            _ => Constantness::Runtime,
        })
    }

    fn source_is_constant(&mut self, source: &syntax::Expression) -> Result<bool, Diagnostic> {
        match &source.kind {
            E::Call(name, args) => {
                let path = syntax::NamePath {
                    root: *name,
                    members: vec![],
                };
                if let Some(info) = self.describe_constant_query_call(&path, args, source.span)? {
                    return Ok(info.is_compile_time_constant());
                }
                self.describe_call_results(&path, args, source.span)?;
                Ok(false)
            }
            E::QualifiedCall(path, args) => {
                self.describe_call_results(path, args, source.span)?;
                Ok(false)
            }
            E::CallHint { call, .. } => self.source_is_constant(call),
            _ => Ok(self.describe_argument(source)?.is_compile_time_constant()),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Constantness {
    Constant,
    Deferred,
    Runtime,
}
impl Constantness {
    fn combine(self, other: Self) -> Self {
        match (self, other) {
            (Self::Runtime, _) | (_, Self::Runtime) => Self::Runtime,
            (Self::Deferred, _) | (_, Self::Deferred) => Self::Deferred,
            _ => Self::Constant,
        }
    }
}
