use super::*;

impl<A> Visitor<'_, A> {
    pub(super) fn argument<E>(&mut self, argument: &CallArgument, depth: usize) -> Result<E>
    where
        A: FnMut(usize, usize) -> std::result::Result<(), E>,
    {
        self.node(depth)?;
        self.expression(&argument.value, depth + 1)
    }

    pub(super) fn expression<E>(&mut self, source: &Expression, depth: usize) -> Result<E>
    where
        A: FnMut(usize, usize) -> std::result::Result<(), E>,
    {
        self.node(depth)?;
        match &source.kind {
            ExpressionKind::Integer(_)
            | ExpressionKind::Character(_)
            | ExpressionKind::Null
            | ExpressionKind::CompileTimePredicate
            | ExpressionKind::CallerLocation
            | ExpressionKind::SourceLocation
            | ExpressionKind::SourceFile
            | ExpressionKind::SourceFilepath
            | ExpressionKind::SourceLine
            | ExpressionKind::Uninitialized
            | ExpressionKind::Bool(_)
            | ExpressionKind::Name(_)
            | ExpressionKind::Context
            | ExpressionKind::InferredMember(_)
            | ExpressionKind::CompileVariable(_) => Ok(()),
            ExpressionKind::Float(literal) => match literal {
                FloatLiteral::Decimal(value) => {
                    self.allocation::<u8, E>(value.retained_spelling_capacity())
                }
                FloatLiteral::Bits32(_) | FloatLiteral::Bits64(_) => Ok(()),
            },
            ExpressionKind::String(bytes) => self.allocation::<u8, E>(bytes.capacity()),
            ExpressionKind::HereString(value) => {
                self.allocation::<u8, E>(value.bytes.capacity())?;
                self.allocation::<HereStringModifier, E>(value.modifiers.capacity())
            }
            ExpressionKind::BakeArguments(source) => {
                self.boxed(source.as_ref(), depth, |this, source, depth| {
                    this.boxed(source.callee.as_ref(), depth, Self::expression)?;
                    this.sequence(&source.arguments, depth, Self::argument)
                })
            }
            ExpressionKind::Type(ty) => self.ty(ty, depth + 1),
            ExpressionKind::CompileTime(run) => self.run_body(&run.body, depth + 1),
            ExpressionKind::ShortLambda(source) => {
                self.boxed(source.as_ref(), depth, Self::short_lambda)
            }
            ExpressionKind::AnonymousProcedure(source) => {
                self.boxed(source.as_ref(), depth, Self::source_procedure)
            }
            ExpressionKind::Code(body) => self.code(body, depth + 1),
            ExpressionKind::Insert(directive) => {
                self.boxed(directive.as_ref(), depth, Self::insert)
            }
            ExpressionKind::QualifiedName(path) => self.path(path, depth + 1),
            ExpressionKind::Call(_, arguments) => self.sequence(arguments, depth, Self::argument),
            ExpressionKind::QualifiedCall(path, arguments) => {
                self.path(path, depth + 1)?;
                self.sequence(arguments, depth, Self::argument)
            }
            ExpressionKind::IndirectCall {
                callee,
                args,
            } => {
                self.boxed(callee.as_ref(), depth, Self::expression)?;
                self.sequence(args, depth, Self::argument)
            }
            ExpressionKind::ContextCall {
                callee,
                args,
                overrides,
            } => {
                self.boxed(callee.as_ref(), depth, Self::expression)?;
                self.sequence(args, depth, Self::argument)?;
                self.sequence(overrides, depth, Self::argument)
            }
            ExpressionKind::CallHint {
                call: value, ..
            }
            | ExpressionKind::AddressOf(value)
            | ExpressionKind::Dereference(value)
            | ExpressionKind::Member {
                base: value, ..
            }
            | ExpressionKind::Unary(_, value)
            | ExpressionKind::Cast(_, _, value)
            | ExpressionKind::InferredCast {
                value, ..
            }
            | ExpressionKind::TypeQuery {
                value, ..
            } => self.boxed(value.as_ref(), depth, Self::expression),
            ExpressionKind::Index {
                base,
                index,
            }
            | ExpressionKind::Binary(_, base, index) => {
                self.boxed(base.as_ref(), depth, Self::expression)?;
                self.boxed(index.as_ref(), depth, Self::expression)
            }
            ExpressionKind::TypeCast {
                ty,
                value,
                ..
            } => {
                self.ty(ty, depth + 1)?;
                self.boxed(value.as_ref(), depth, Self::expression)
            }
            ExpressionKind::ArrayLiteral(literal) => {
                if let Some(ty) = &literal.element_type {
                    self.ty(ty, depth + 1)?;
                }
                self.sequence(&literal.elements, depth, Self::expression)
            }
            ExpressionKind::StructLiteral(literal) => {
                if let Some(ty) = &literal.ty {
                    self.ty(ty, depth + 1)?;
                }
                self.sequence(&literal.fields, depth, |this, field, depth| {
                    this.node(depth)?;
                    this.place(&field.target, depth + 1)?;
                    this.expression(&field.value, depth + 1)
                })
            }
            ExpressionKind::PositionalStructLiteral(literal) => {
                if let Some(ty) = &literal.ty {
                    self.ty(ty, depth + 1)?;
                }
                self.sequence(&literal.values, depth, Self::expression)
            }
            ExpressionKind::Conditional(source) => {
                self.boxed(source.condition.as_ref(), depth, Self::expression)?;
                if let Some(value) = source.explicit_then() {
                    self.boxed(value, depth, Self::expression)?;
                }
                if let Some(value) = &source.else_value {
                    self.boxed(value.as_ref(), depth, Self::expression)?;
                }
                Ok(())
            }
        }
    }

    fn short_lambda<E>(&mut self, source: &ShortLambda, depth: usize) -> Result<E>
    where
        A: FnMut(usize, usize) -> std::result::Result<(), E>,
    {
        self.node(depth)?;
        self.sequence(&source.parameters, depth, |this, parameter, depth| {
            this.node(depth)?;
            if let Some(ty) = &parameter.ty {
                this.ty(ty, depth + 1)?;
            }
            Ok(())
        })?;
        match &source.body.kind {
            ShortLambdaBodyKind::Expression(value) => self.expression(value, depth + 1),
            ShortLambdaBodyKind::Block(body) => self.sequence(body, depth, Self::statement),
        }
    }

    pub(super) fn run_body<E>(&mut self, source: &CompileTimeBody, depth: usize) -> Result<E>
    where
        A: FnMut(usize, usize) -> std::result::Result<(), E>,
    {
        self.node(depth)?;
        match source {
            CompileTimeBody::Expression(value) => {
                self.boxed(value.as_ref(), depth, Self::expression)
            }
            CompileTimeBody::Block(body) => self.sequence(body, depth, Self::statement),
            CompileTimeBody::Procedure {
                result,
                body,
            } => {
                self.ty(result, depth + 1)?;
                self.sequence(body, depth, Self::statement)
            }
        }
    }

    pub(super) fn code<E>(&mut self, source: &CodeBody, depth: usize) -> Result<E>
    where
        A: FnMut(usize, usize) -> std::result::Result<(), E>,
    {
        self.node(depth)?;
        match source {
            CodeBody::Null => Ok(()),
            CodeBody::Expression(value) => self.boxed(value.as_ref(), depth, Self::expression),
            CodeBody::Statement(value) => self.boxed(value.as_ref(), depth, Self::statement),
            CodeBody::Block(body) => self.sequence(body, depth, Self::statement),
        }
    }

    pub(super) fn insert<E>(&mut self, source: &InsertDirective, depth: usize) -> Result<E>
    where
        A: FnMut(usize, usize) -> std::result::Result<(), E>,
    {
        self.node(depth)?;
        self.expression(&source.value, depth + 1)?;
        self.sequence(&source.replacements, depth, |this, replacement, depth| {
            this.node(depth)?;
            match &replacement.body {
                LoopControlReplacementBody::Code(code) => this.code(code, depth + 1),
                LoopControlReplacementBody::Assert {
                    condition,
                    message,
                    ..
                } => {
                    this.boxed(condition.as_ref(), depth, Self::expression)?;
                    if let Some(message) = message {
                        this.boxed(message.as_ref(), depth, Self::expression)?;
                    }
                    Ok(())
                }
            }
        })
    }
}
