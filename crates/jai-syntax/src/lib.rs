//! Parsing converts raw tokens into domain operators, names and signatures.
mod modules;
mod numeric_literals;
mod types;
use jai_lexer::{Directive, Keyword, Kind, Punct, Token, lex};
use jai_source::{Diagnostic, Span, Symbol, Symbols};
pub use modules::*;
pub use types::*;

pub use jai_types::{CastMode, IntegerType, ReturnType, ScalarType};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnaryOp {
    Positive,
    Negate,
    LogicalNot,
    Complement,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinaryOp {
    LogicalOr,
    LogicalAnd,
    BitOr,
    BitXor,
    BitAnd,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    ShiftLeft,
    ShiftRight,
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
}
impl BinaryOp {
    fn compound(p: Punct) -> Option<Self> {
        Some(match p {
            Punct::AddAssign => Self::Add,
            Punct::SubAssign => Self::Subtract,
            Punct::MulAssign => Self::Multiply,
            Punct::DivAssign => Self::Divide,
            Punct::RemAssign => Self::Remainder,
            Punct::AndAssign => Self::BitAnd,
            Punct::OrAssign => Self::BitOr,
            Punct::XorAssign => Self::BitXor,
            Punct::ShiftLeftAssign => Self::ShiftLeft,
            Punct::ShiftRightAssign => Self::ShiftRight,
            Punct::LogicalAndAssign => Self::LogicalAnd,
            Punct::LogicalOrAssign => Self::LogicalOr,
            _ => return None,
        })
    }
    fn parse(p: Punct) -> Option<(Self, u8)> {
        Some(match p {
            Punct::LogicalOr => (Self::LogicalOr, 1),
            Punct::LogicalAnd => (Self::LogicalAnd, 3),
            Punct::Or => (Self::BitOr, 5),
            Punct::Xor => (Self::BitXor, 7),
            Punct::And => (Self::BitAnd, 9),
            Punct::Equal => (Self::Equal, 11),
            Punct::NotEqual => (Self::NotEqual, 11),
            Punct::Less => (Self::Less, 13),
            Punct::LessEqual => (Self::LessEqual, 13),
            Punct::Greater => (Self::Greater, 13),
            Punct::GreaterEqual => (Self::GreaterEqual, 13),
            Punct::ShiftLeft => (Self::ShiftLeft, 15),
            Punct::ShiftRight => (Self::ShiftRight, 15),
            Punct::Add => (Self::Add, 17),
            Punct::Sub => (Self::Subtract, 17),
            Punct::Mul => (Self::Multiply, 19),
            Punct::Div => (Self::Divide, 19),
            Punct::Rem => (Self::Remainder, 19),
            _ => return None,
        })
    }
}
#[derive(Clone, Debug)]
pub struct Module {
    procedures: Vec<Procedure>,
    symbols: Symbols,
    constants: Vec<ConstantDeclaration>,
    globals: Vec<GlobalDeclaration>,
}
impl Module {
    pub fn procedures(&self) -> &[Procedure] {
        &self.procedures
    }
    pub fn constants(&self) -> &[ConstantDeclaration] {
        &self.constants
    }
    pub fn globals(&self) -> &[GlobalDeclaration] {
        &self.globals
    }
    pub fn symbols(&self) -> &Symbols {
        &self.symbols
    }
}
#[derive(Clone, Debug)]
pub struct Parameter {
    pub name: Symbol,
    pub binding: ParameterBinding,
}
#[derive(Clone, Debug)]
pub enum ParameterBinding {
    Required(ScalarType),
    Defaulted {
        ty: Option<ScalarType>,
        expression: Expression,
    },
}
#[derive(Clone, Debug)]
pub struct CallArgument {
    pub name: Option<Symbol>,
    pub value: Expression,
}
#[derive(Clone, Debug)]
pub struct Procedure {
    pub name: Symbol,
    pub parameters: Vec<Parameter>,
    pub body: Vec<Statement>,
    pub span: Span,
    pub return_type: ReturnType,
}
#[derive(Clone, Debug)]
pub struct GlobalDeclaration {
    pub declaration: Declaration,
    pub span: Span,
}
#[derive(Clone, Debug)]
pub struct ConstantDeclaration {
    pub name: Symbol,
    pub ty: Option<ScalarType>,
    pub initializer: Expression,
    pub span: Span,
}
#[derive(Clone, Debug)]
pub enum Declaration {
    Inferred {
        name: Symbol,
        initializer: Expression,
    },
    Explicit {
        name: Symbol,
        ty: ScalarType,
        initializer: Option<Expression>,
    },
    UnresolvedExplicit {
        name: Symbol,
        ty: TypeSyntax,
        initializer: Option<Expression>,
    },
}
impl Declaration {
    pub fn name(&self) -> Symbol {
        match self {
            Self::Inferred { name, .. }
            | Self::Explicit { name, .. }
            | Self::UnresolvedExplicit { name, .. } => *name,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Forward,
    Reverse,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JumpKind {
    Break,
    Continue,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoopTarget {
    Innermost,
    Named(Symbol),
}
#[derive(Clone, Debug)]
pub enum WhileCondition {
    Expression(Expression),
    Binding {
        name: Symbol,
        initializer: Expression,
    },
}
#[derive(Clone, Debug)]
pub struct RangeLoop {
    pub iterator: Symbol,
    pub start: Expression,
    pub end: Expression,
    pub direction: Direction,
    pub body: Vec<Statement>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaseOperator {
    Equal,
    NotEqual,
}
#[derive(Clone, Debug)]
pub struct CaseStatement {
    pub value: Expression,
    pub operator: CaseOperator,
    pub arms: Vec<(Expression, Vec<Statement>, bool)>,
    pub complete: bool,
    pub default: Option<Vec<Statement>>,
}
#[derive(Clone, Debug)]
pub enum Statement {
    Declare(Declaration),
    Constant(ConstantDeclaration),
    Assign(Symbol, Expression),
    Update(Symbol, BinaryOp, Expression),
    Return(Option<Expression>),
    Expression(Expression),
    If(Expression, Vec<Statement>, Vec<Statement>),
    Cases(CaseStatement),
    While(WhileCondition, Vec<Statement>),
    Range(RangeLoop),
    Jump {
        kind: JumpKind,
        target: LoopTarget,
        span: Span,
    },
    Block(Vec<Statement>),
    Defer(Vec<Statement>),
}
#[derive(Clone, Debug)]
pub struct Expression {
    pub kind: ExpressionKind,
    pub span: Span,
}
#[derive(Clone, Debug)]
pub enum ExpressionKind {
    Integer(i128),
    Bool(bool),
    Name(Symbol),
    QualifiedName(NamePath),
    Call(Symbol, Vec<CallArgument>),
    QualifiedCall(NamePath, Vec<CallArgument>),
    StructLiteral(StructLiteral),
    Member {
        base: Box<Expression>,
        member: Symbol,
    },
    Unary(UnaryOp, Box<Expression>),
    Cast(CastMode, ScalarType, Box<Expression>),
    Binary(BinaryOp, Box<Expression>, Box<Expression>),
    Conditional(ConditionalExpression),
}
#[derive(Clone, Debug)]
pub struct ConditionalExpression {
    pub condition: Box<Expression>,
    pub then_value: Box<Expression>,
    pub else_value: Option<Box<Expression>>,
}

pub fn parse(source: &str) -> Result<Module, Diagnostic> {
    let mut parser = Parser {
        source,
        tokens: lex(source)?,
        at: 0,
        symbols: Symbols::default(),
        allow_qualified: false,
    };
    let mut procedures = Vec::new();
    let mut constants = Vec::new();
    let mut globals = Vec::new();
    while parser.token().kind != Kind::Eof {
        if parser.starts_procedure() {
            procedures.push(parser.procedure()?);
        } else {
            let span = parser.token().span;
            let name = parser.name()?;
            match parser.data_declaration(name, span)? {
                Statement::Declare(declaration) => {
                    globals.push(GlobalDeclaration { declaration, span })
                }
                Statement::Constant(declaration) => constants.push(declaration),
                _ => unreachable!("data declaration always produces a declaration"),
            }
        }
    }
    Ok(Module {
        procedures,
        constants,
        globals,
        symbols: parser.symbols,
    })
}
struct Parser<'a> {
    source: &'a str,
    tokens: Vec<Token>,
    at: usize,
    symbols: Symbols,
    allow_qualified: bool,
}
impl Parser<'_> {
    fn token(&self) -> Token {
        self.tokens[self.at]
    }
    fn text(&self) -> &str {
        self.token().span.text(self.source)
    }
    fn is(&self, p: Punct) -> bool {
        self.token().kind == Kind::Punctuation(p)
    }
    fn take(&mut self, p: Punct) -> bool {
        if self.is(p) {
            self.at += 1;
            true
        } else {
            false
        }
    }
    fn keyword(&mut self, k: Keyword) -> bool {
        if self.token().kind == Kind::Keyword(k) {
            self.at += 1;
            true
        } else {
            false
        }
    }
    fn error(&self, msg: impl Into<String>) -> Diagnostic {
        Diagnostic::new(self.token().span, msg)
    }
    fn need(&mut self, p: Punct) -> Result<(), Diagnostic> {
        if self.take(p) {
            Ok(())
        } else {
            Err(self.error(format!(
                "expected '{}', found '{}'",
                p.spelling(),
                self.text()
            )))
        }
    }
    fn name(&mut self) -> Result<Symbol, Diagnostic> {
        if self.token().kind != Kind::Ident {
            return Err(self.error("expected identifier"));
        }
        let name = self.token().span.text(self.source);
        let symbol = self.symbols.intern(name);
        self.at += 1;
        Ok(symbol)
    }
    fn scalar_type(&mut self) -> Result<ScalarType, Diagnostic> {
        let ty = match BuiltinType::from_spelling(self.text()) {
            Some(BuiltinType::Scalar(ty)) => ty,
            _ => return Err(self.error("expected a supported scalar type")),
        };
        if self.token().kind != Kind::Ident {
            return Err(self.error("expected type"));
        }
        self.at += 1;
        Ok(ty)
    }
    fn starts_procedure(&self) -> bool {
        if self.token().kind != Kind::Ident
            || self
                .tokens
                .get(self.at + 1)
                .is_none_or(|t| t.kind != Kind::Punctuation(Punct::Constant))
            || self
                .tokens
                .get(self.at + 2)
                .is_none_or(|t| t.kind != Kind::Punctuation(Punct::OpenParen))
        {
            return false;
        }
        let mut depth = 0;
        for (offset, token) in self.tokens[self.at + 2..].iter().enumerate() {
            match token.kind {
                Kind::Punctuation(Punct::OpenParen) => depth += 1,
                Kind::Punctuation(Punct::CloseParen) => {
                    depth -= 1;
                    if depth == 0 {
                        return self.tokens.get(self.at + offset + 3).is_some_and(|next| {
                            matches!(
                                next.kind,
                                Kind::Punctuation(Punct::OpenBrace | Punct::Arrow)
                            )
                        });
                    }
                }
                Kind::Eof => return false,
                _ => {}
            }
        }
        false
    }
    fn data_declaration(&mut self, name: Symbol, span: Span) -> Result<Statement, Diagnostic> {
        let statement = if self.take(Punct::Constant) {
            Statement::Constant(ConstantDeclaration {
                name,
                span,
                ty: None,
                initializer: self.expression(0)?,
            })
        } else if self.take(Punct::Infer) {
            Statement::Declare(Declaration::Inferred {
                name,
                initializer: self.expression(0)?,
            })
        } else {
            self.need(Punct::Colon)?;
            let ty = if self.allow_qualified {
                self.type_syntax()?
            } else {
                TypeSyntax::Builtin(BuiltinType::Scalar(self.scalar_type()?))
            };
            if self.take(Punct::Colon) {
                let Some(ty) = ty.as_scalar() else {
                    return Err(self.error("non-scalar constant annotations are not implemented"));
                };
                Statement::Constant(ConstantDeclaration {
                    name,
                    span,
                    ty: Some(ty),
                    initializer: self.expression(0)?,
                })
            } else {
                let initializer = if self.take(Punct::Assign) {
                    Some(self.expression(0)?)
                } else {
                    None
                };
                Statement::Declare(if let Some(ty) = ty.as_scalar() {
                    Declaration::Explicit {
                        name,
                        ty,
                        initializer,
                    }
                } else {
                    Declaration::UnresolvedExplicit {
                        name,
                        ty,
                        initializer,
                    }
                })
            }
        };
        self.need(Punct::Semicolon)?;
        Ok(statement)
    }
    fn procedure(&mut self) -> Result<Procedure, Diagnostic> {
        let span = self.token().span;
        let name = self.name()?;
        self.need(Punct::Constant)?;
        self.need(Punct::OpenParen)?;
        let mut parameters = Vec::new();
        if !self.take(Punct::CloseParen) {
            loop {
                let name = self.name()?;
                let binding = if self.take(Punct::Infer) {
                    ParameterBinding::Defaulted {
                        ty: None,
                        expression: self.expression(0)?,
                    }
                } else {
                    self.need(Punct::Colon)?;
                    let ty = self.scalar_type()?;
                    if self.take(Punct::Assign) {
                        ParameterBinding::Defaulted {
                            ty: Some(ty),
                            expression: self.expression(0)?,
                        }
                    } else {
                        ParameterBinding::Required(ty)
                    }
                };
                parameters.push(Parameter { name, binding });
                if self.take(Punct::CloseParen) {
                    break;
                }
                self.need(Punct::Comma)?;
            }
        }
        let return_type = if self.take(Punct::Arrow) {
            ReturnType::Value(self.scalar_type()?)
        } else {
            ReturnType::Void
        };
        Ok(Procedure {
            name,
            parameters,
            body: self.block()?,
            span,
            return_type,
        })
    }
    fn block(&mut self) -> Result<Vec<Statement>, Diagnostic> {
        self.need(Punct::OpenBrace)?;
        let mut out = Vec::new();
        while !self.take(Punct::CloseBrace) {
            if self.token().kind == Kind::Eof {
                return Err(self.error("unterminated block"));
            }
            out.push(self.statement()?);
        }
        Ok(out)
    }
    fn body(&mut self) -> Result<Vec<Statement>, Diagnostic> {
        if self.is(Punct::OpenBrace) {
            self.block()
        } else {
            self.keyword(Keyword::Then);
            Ok(vec![self.statement()?])
        }
    }
    fn named_prefix(&self, punctuation: Punct) -> bool {
        self.token().kind == Kind::Ident
            && self
                .tokens
                .get(self.at + 1)
                .is_some_and(|t| t.kind == Kind::Punctuation(punctuation))
    }
    fn statement(&mut self) -> Result<Statement, Diagnostic> {
        if self.is(Punct::OpenBrace) {
            return Ok(Statement::Block(self.block()?));
        }
        if self.keyword(Keyword::Defer) {
            return Ok(Statement::Defer(self.body()?));
        }
        if self.keyword(Keyword::Return) {
            let expr = if self.is(Punct::Semicolon) {
                None
            } else {
                Some(self.expression(0)?)
            };
            self.need(Punct::Semicolon)?;
            return Ok(Statement::Return(expr));
        }
        if self.keyword(Keyword::If) {
            let complete = self.token().kind == Kind::Directive(Directive::Complete);
            if complete {
                self.at += 1;
            }
            let cond = self.expression(0)?;
            if self.is(Punct::OpenBrace) {
                let operator = match self.tokens[self.at - 1].kind {
                    Kind::Punctuation(Punct::Equal) => Some(CaseOperator::Equal),
                    Kind::Punctuation(Punct::NotEqual) => Some(CaseOperator::NotEqual),
                    _ => None,
                };
                if let Some(operator) = operator {
                    return self.case_statement(cond, operator, complete);
                }
            }
            if complete {
                return Err(self.error("#complete requires if-case"));
            }
            let yes = self.body()?;
            let no = if self.keyword(Keyword::Else) {
                self.body()?
            } else {
                Vec::new()
            };
            return Ok(Statement::If(cond, yes, no));
        }
        if self.keyword(Keyword::While) {
            let condition = if self.named_prefix(Punct::Infer) {
                let name = self.name()?;
                self.need(Punct::Infer)?;
                WhileCondition::Binding {
                    name,
                    initializer: self.expression(0)?,
                }
            } else {
                WhileCondition::Expression(self.expression(0)?)
            };
            return Ok(Statement::While(condition, self.body()?));
        }
        if self.keyword(Keyword::For) {
            let direction = if self.take(Punct::Less) {
                Direction::Reverse
            } else {
                Direction::Forward
            };
            let iterator = if self.named_prefix(Punct::Colon) {
                let name = self.name()?;
                self.need(Punct::Colon)?;
                name
            } else {
                self.symbols.intern("it")
            };
            let start = self.expression(0)?;
            self.need(Punct::Range)?;
            let end = self.expression(0)?;
            return Ok(Statement::Range(RangeLoop {
                iterator,
                start,
                end,
                direction,
                body: self.body()?,
            }));
        }
        let jump = match self.token().kind {
            Kind::Keyword(Keyword::Break) => Some(JumpKind::Break),
            Kind::Keyword(Keyword::Continue) => Some(JumpKind::Continue),
            _ => None,
        };
        if let Some(kind) = jump {
            let span = self.token().span;
            self.at += 1;
            let target = if self.token().kind == Kind::Ident {
                LoopTarget::Named(self.name()?)
            } else {
                LoopTarget::Innermost
            };
            self.need(Punct::Semicolon)?;
            return Ok(Statement::Jump { kind, target, span });
        }
        if self.token().kind == Kind::Ident
            && matches!(self.tokens[self.at + 1].kind, Kind::Punctuation(p)
                if matches!(p, Punct::Infer | Punct::Colon | Punct::Constant | Punct::Assign) || BinaryOp::compound(p).is_some())
        {
            let span = self.token().span;
            let name = self.name()?;
            if let Kind::Punctuation(p) = self.token().kind
                && let Some(op) = BinaryOp::compound(p)
            {
                self.at += 1;
                let value = self.expression(0)?;
                self.need(Punct::Semicolon)?;
                return Ok(Statement::Update(name, op, value));
            }
            if self.take(Punct::Assign) {
                let v = self.expression(0)?;
                self.need(Punct::Semicolon)?;
                return Ok(Statement::Assign(name, v));
            }
            return self.data_declaration(name, span);
        }
        let expr = self.expression(0)?;
        self.need(Punct::Semicolon)?;
        Ok(Statement::Expression(expr))
    }
    fn case_statement(
        &mut self,
        value: Expression,
        operator: CaseOperator,
        complete: bool,
    ) -> Result<Statement, Diagnostic> {
        self.need(Punct::OpenBrace)?;
        let mut arms = Vec::new();
        let mut default = None;
        while !self.take(Punct::CloseBrace) {
            if !self.keyword(Keyword::Case) {
                return Err(self.error("expected case label"));
            }
            if default.is_some() {
                return Err(self.error("default case must be last"));
            }
            let label = if self.is(Punct::Semicolon) {
                None
            } else {
                Some(self.expression(0)?)
            };
            self.need(Punct::Semicolon)?;
            let mut body = Vec::new();
            let mut through = false;
            while !self.is(Punct::CloseBrace) && self.token().kind != Kind::Keyword(Keyword::Case) {
                if self.token().kind == Kind::Eof {
                    return Err(self.error("unterminated case block"));
                }
                if self.token().kind == Kind::Directive(Directive::Through) {
                    self.at += 1;
                    self.need(Punct::Semicolon)?;
                    through = true;
                    if !self.is(Punct::CloseBrace)
                        && self.token().kind != Kind::Keyword(Keyword::Case)
                    {
                        return Err(self.error("#through must be the last case statement"));
                    }
                    break;
                }
                body.push(self.statement()?);
            }
            if let Some(label) = label {
                arms.push((label, body, through));
            } else {
                if through {
                    return Err(self.error("default case cannot #through"));
                }
                default = Some(body);
            }
        }
        Ok(Statement::Cases(CaseStatement {
            value,
            operator,
            arms,
            default,
            complete,
        }))
    }
    fn expression(&mut self, minimum: u8) -> Result<Expression, Diagnostic> {
        let token = self.token();
        let span = token.span;
        let unary = match token.kind {
            Kind::Punctuation(Punct::Sub) => Some(UnaryOp::Negate),
            Kind::Punctuation(Punct::Add) => Some(UnaryOp::Positive),
            Kind::Punctuation(Punct::Not) => Some(UnaryOp::LogicalNot),
            Kind::Punctuation(Punct::Complement) => Some(UnaryOp::Complement),
            _ => None,
        };
        let mut lhs = if self.take(Punct::OpenParen) {
            let e = self.expression(0)?;
            self.need(Punct::CloseParen)?;
            e
        } else if self.is(Punct::StructLiteral) {
            if !self.allow_qualified {
                return Err(self.error("struct literals require aggregate type resolution"));
            }
            self.struct_literal(None, span.start)?
        } else if self.keyword(Keyword::Ifx) {
            let condition = Box::new(self.expression(0)?);
            if self.is(Punct::OpenBrace) {
                return Err(self.error("ifx case expressions are not implemented"));
            }
            self.keyword(Keyword::Then);
            if self.token().kind == Kind::Keyword(Keyword::Else) {
                return Err(self.error("implicit then values in ifx are not implemented yet"));
            }
            let then_value = Box::new(self.expression(0)?);
            let else_value = if self.keyword(Keyword::Else) {
                Some(Box::new(self.expression(0)?))
            } else {
                None
            };
            let end = else_value.as_ref().unwrap_or(&then_value).span.end;
            Expression {
                span: Span::new(span.start, end),
                kind: ExpressionKind::Conditional(ConditionalExpression {
                    condition,
                    then_value,
                    else_value,
                }),
            }
        } else if self.keyword(Keyword::Cast) {
            let mode = if self.take(Punct::Comma) {
                if self.token().kind != Kind::Ident || self.text() != "no_check" {
                    return Err(self.error("expected no_check cast modifier"));
                }
                self.at += 1;
                CastMode::Unchecked
            } else {
                CastMode::Checked
            };
            self.need(Punct::OpenParen)?;
            let ty = self.scalar_type()?;
            self.need(Punct::CloseParen)?;
            let value = self.expression(21)?;
            Expression {
                span: Span::new(span.start, value.span.end),
                kind: ExpressionKind::Cast(mode, ty, Box::new(value)),
            }
        } else if let Some(op) = unary {
            self.at += 1;
            let rhs = self.expression(21)?;
            Expression {
                span: Span::new(span.start, rhs.span.end),
                kind: ExpressionKind::Unary(op, Box::new(rhs)),
            }
        } else if token.kind == Kind::Number {
            let value = numeric_literals::integer(self.text())
                .map_err(|error| Diagnostic::new(span, error.to_string()))?;
            self.at += 1;
            Expression {
                span,
                kind: ExpressionKind::Integer(i128::from(value)),
            }
        } else if self.keyword(Keyword::True) {
            Expression {
                span,
                kind: ExpressionKind::Bool(true),
            }
        } else if self.keyword(Keyword::False) {
            Expression {
                span,
                kind: ExpressionKind::Bool(false),
            }
        } else if token.kind == Kind::Ident {
            Expression {
                span,
                kind: ExpressionKind::Name(self.name()?),
            }
        } else {
            return Err(self.error("expected expression; this syntax is not implemented yet"));
        };
        loop {
            if self.is(Punct::StructLiteral) && minimum <= 23 {
                if !self.allow_qualified {
                    return Err(self.error("struct literals require aggregate type resolution"));
                }
                let ty = match lhs.kind {
                    ExpressionKind::Name(root) => NamePath {
                        root,
                        members: Vec::new(),
                    },
                    ExpressionKind::QualifiedName(path) => path,
                    _ => return Err(self.error("struct literal type must be a named type")),
                };
                lhs = self.struct_literal(Some(ty), lhs.span.start)?;
                continue;
            }
            if self.is(Punct::Dot) && minimum <= 23 {
                if !self.allow_qualified {
                    return Err(
                        self.error("qualified module references require module scope resolution")
                    );
                }
                self.at += 1;
                let member = self.name()?;
                let start = lhs.span.start;
                let kind = match lhs.kind {
                    ExpressionKind::Name(root) => ExpressionKind::QualifiedName(NamePath {
                        root,
                        members: vec![member],
                    }),
                    ExpressionKind::QualifiedName(mut path) => {
                        path.members.push(member);
                        ExpressionKind::QualifiedName(path)
                    }
                    _ => ExpressionKind::Member {
                        base: Box::new(lhs),
                        member,
                    },
                };
                lhs = Expression {
                    span: Span::new(start, self.tokens[self.at - 1].span.end),
                    kind,
                };
                continue;
            }
            if self.is(Punct::OpenParen) && minimum <= 23 {
                let callee = match lhs.kind {
                    ExpressionKind::Name(name) => NamePath {
                        root: name,
                        members: vec![],
                    },
                    ExpressionKind::QualifiedName(path) => path,
                    _ => return Err(self.error("indirect procedure calls are not implemented")),
                };
                self.at += 1;
                let mut args = Vec::new();
                if !self.take(Punct::CloseParen) {
                    loop {
                        let name = if self.named_prefix(Punct::Assign) {
                            let name = self.name()?;
                            self.need(Punct::Assign)?;
                            Some(name)
                        } else {
                            None
                        };
                        args.push(CallArgument {
                            name,
                            value: self.expression(0)?,
                        });
                        if self.take(Punct::CloseParen) {
                            break;
                        }
                        self.need(Punct::Comma)?;
                    }
                }
                lhs = Expression {
                    span: Span::new(lhs.span.start, self.tokens[self.at - 1].span.end),
                    kind: if callee.members.is_empty() {
                        ExpressionKind::Call(callee.root, args)
                    } else {
                        ExpressionKind::QualifiedCall(callee, args)
                    },
                };
                continue;
            }
            let Kind::Punctuation(punctuation) = self.token().kind else {
                break;
            };
            let Some((op, precedence)) = BinaryOp::parse(punctuation) else {
                break;
            };
            if precedence < minimum {
                break;
            }
            self.at += 1;
            if self.is(Punct::OpenBrace) && matches!(op, BinaryOp::Equal | BinaryOp::NotEqual) {
                break;
            }
            let rhs = self.expression(precedence + 1)?;
            lhs = Expression {
                span: Span::new(lhs.span.start, rhs.span.end),
                kind: ExpressionKind::Binary(op, Box::new(lhs), Box::new(rhs)),
            };
        }
        Ok(lhs)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn recursive_procedure() {
        let m = parse("f :: (n: int) -> int { if n <= 1 return 1; return n*f(n-1); }").unwrap();
        assert_eq!(m.symbols.name(m.procedures[0].parameters[0].name), "n");
        assert_eq!(m.procedures[0].body.len(), 2);
    }
    #[test]
    fn precedence() {
        let m = parse("main :: () -> int { return 1 + 2 * 3; }").unwrap();
        let Statement::Return(Some(e)) = &m.procedures[0].body[0] else {
            panic!()
        };
        let ExpressionKind::Binary(BinaryOp::Add, _, rhs) = &e.kind else {
            panic!()
        };
        assert!(matches!(
            rhs.kind,
            ExpressionKind::Binary(BinaryOp::Multiply, _, _)
        ));
    }
    #[test]
    fn unsupported_is_not_success() {
        assert!(parse("#import \"Basic\";").is_err());
        assert!(parse("main :: () { x: float; }").is_err());
    }
    #[test]
    fn missing_delimiter() {
        assert!(parse("main :: () { return 1;").is_err());
    }
    #[test]
    fn loop_syntax_has_domain_tags() {
        let module =
            parse("main :: () { for < i: 1..4 { continue i; } while value := true break value; }")
                .unwrap();
        let Statement::Range(range) = &module.procedures[0].body[0] else {
            panic!("expected range")
        };
        assert_eq!(range.direction, Direction::Reverse);
        assert!(
            matches!(&range.body[0], Statement::Jump { kind: JumpKind::Continue, target: LoopTarget::Named(name), .. } if *name == range.iterator)
        );
        assert!(matches!(
            &module.procedures[0].body[1],
            Statement::While(WhileCondition::Binding { .. }, _)
        ));
        for invalid in [
            "main :: () { for 1.. {} }",
            "main :: () { for 1 {} }",
            "main :: () { break 1; }",
            "main :: () { for #v2 < 1..3 {} }",
        ] {
            assert!(parse(invalid).is_err(), "{invalid}");
        }
    }
    #[test]
    fn distinguish_parenthesized_constants_from_procedures() {
        let module = parse("N :: (1 + 2) * 14; flag : bool : true; count : int; main :: ()->int { LOCAL :: 2; return N; }").unwrap();
        assert_eq!(module.constants().len(), 2);
        assert_eq!(module.globals().len(), 1);
        assert_eq!(module.procedures().len(), 1);
        assert!(matches!(
            module.procedures()[0].body[0],
            Statement::Constant(_)
        ));
        assert!(parse("N :: (1 + 2; main :: () {}").is_err());
    }
    #[test]
    fn reserved_words_are_not_names() {
        assert!(parse("if :: () {}").is_err());
    }
    #[test]
    fn conditional_expressions_bind_else_to_the_nearest_ifx() {
        let module =
            parse("main :: ()->int { return ifx true then ifx false 1 else 2 else 3; }").unwrap();
        let Statement::Return(Some(e)) = &module.procedures()[0].body[0] else {
            panic!()
        };
        let ExpressionKind::Conditional(outer) = &e.kind else {
            panic!()
        };
        assert!(matches!(
            outer.else_value.as_ref().unwrap().kind,
            ExpressionKind::Integer(3)
        ));
        let ExpressionKind::Conditional(inner) = &outer.then_value.kind else {
            panic!()
        };
        assert!(matches!(
            inner.else_value.as_ref().unwrap().kind,
            ExpressionKind::Integer(2)
        ));
        assert!(parse("main :: ()->int { return ifx true then 1; }").is_ok());
        for source in [
            "main :: ()->int { return ifx true then else 1; }",
            "main :: ()->int { return ifx true else 1; }",
            "main :: ()->int { return ifx true then 1 else; }",
        ] {
            assert!(parse(source).is_err(), "{source}");
        }
    }
}

#[cfg(test)]
mod case_parser_tests {
    #[test]
    fn rejects_invalid_case_structure_and_expression_cases() {
        for source in [
            "main :: () { if 1 == { case; case 1; } }",
            "main :: () { if 1 == { case 1; #through; n := 1; case; } }",
            "main :: () { if 1 == { case; #through; } }",
            "main :: () { if 1 == { n := 1; } }",
            "main :: () { if 1 == { case 1;",
            "main :: () { if #complete true { } }",
            "main :: ()->int { return ifx 1 == { case 1; 2; case; 3; }; }",
        ] {
            assert!(super::parse(source).is_err(), "{source}");
        }
    }
}
