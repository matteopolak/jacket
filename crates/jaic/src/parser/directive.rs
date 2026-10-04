//! `#directive` expressions.
use super::expr::mk;
use super::{PResult, Parser};
use crate::ast::{Block, CodeBody, Expr, ExprKind, Ident, RunBody, StmtKind, TypeModifier};
use crate::lexer::{P, Tok};
use std::rc::Rc;

/// `,name` or `,name(expr)` after a directive, e.g. `#run,stallable` or `#insert,scope(x)`.
pub(super) struct DirectiveFlag {
    pub name: Ident,
    pub arg: Option<Expr>,
}

impl Parser<'_> {
    pub(super) fn parse_directive_expr(&mut self, name: &str) -> PResult<Expr> {
        let start = self.span();
        let simple = |kind: ExprKind| Some(kind);
        let kind = match name {
            "caller_location" => simple(ExprKind::CallerLocation),
            "caller_code" => simple(ExprKind::CallerCode),
            "file" => simple(ExprKind::File),
            "line" => simple(ExprKind::Line),
            "filepath" => simple(ExprKind::Filepath),
            "this" => simple(ExprKind::This),
            "compile_time" => simple(ExprKind::CompileTime),
            _ => None,
        };
        if let Some(kind) = kind {
            self.bump();
            return Ok(mk(kind, start));
        }
        match name {
            "run" => self.parse_run(),
            "code" => self.parse_code(),
            "insert" => self.parse_insert_expr(),
            "assert" => self.parse_assert_expr(),
            "type" => self.parse_type_directive(),
            "ifx" => self.parse_ifx(true),
            "char" => self.parse_char(),
            "location" => self.parse_optional_operand(start, ExprKind::Location),
            "procedure_name" => self.parse_optional_operand(start, ExprKind::ProcedureName),
            "bake_arguments" => self.parse_bake(false),
            "bake_constants" => self.parse_bake(true),
            "procedure_of_call" => self.parse_operand(start, ExprKind::ProcedureOfCall),
            "bytes" => self.parse_operand(start, ExprKind::Bytes),
            "exists" => self.parse_exists(),
            "asm" => self.parse_asm(),
            _ => self.parse_unknown_directive(),
        }
    }

    /// Flags written directly after a directive, with no space before the comma.
    pub(super) fn parse_directive_flags(&mut self) -> PResult<Vec<DirectiveFlag>> {
        let mut flags = Vec::new();
        // `#run,stallable`: no space on either side of the comma (`#code,null, loc := ...` has an argument).
        while self.at(P::Comma)
            && self.span().start == self.prev_span().end
            && matches!(self.tok_at(1), Tok::Ident(_))
            && (self.token_at(1).span.start == self.span().end || self.at_known_flag())
        {
            self.bump();
            let name = self.ident("as a directive flag")?;
            let arg = if self.at(P::LParen) && self.span().start == self.prev_span().end {
                self.bump();
                let arg = if self.at(P::RParen) {
                    None
                } else {
                    Some(self.parse_expr()?)
                };
                self.expect(P::RParen, "after the directive flag argument")?;
                arg
            } else {
                None
            };
            flags.push(DirectiveFlag {
                name,
                arg,
            });
        }
        Ok(flags)
    }

    /// The identifier after the comma is a flag name that may be spaced out (`#type, distinct u64`).
    fn at_known_flag(&self) -> bool {
        matches!(
            self.kw_at(1),
            Some("distinct" | "isa" | "file" | "dir" | "string" | "stallable" | "scope")
        )
    }

    fn parse_run(&mut self) -> PResult<Expr> {
        self.parse_run_with(false)
    }

    /// `#run`; at statement level (`allow_stmt`) the operand may also be an assignment or declaration.
    pub(super) fn parse_run_with(&mut self, allow_stmt: bool) -> PResult<Expr> {
        let start = self.bump();
        let flags = self
            .parse_directive_flags()?
            .into_iter()
            .map(|f| f.name)
            .collect();
        let body = if self.at(P::LBrace) {
            RunBody::Block(self.parse_block()?)
        } else if self.at(P::Arrow) {
            // `#run -> T { ... }`: run an anonymous procedure and use its result.
            let proc = self.parse_proc_expr(Default::default())?;
            let span = proc.span;
            RunBody::Expr(mk(
                ExprKind::Call {
                    callee: Box::new(proc),
                    args: Vec::new(),
                    hint: crate::ast::CallHint::None,
                },
                span,
            ))
        } else if allow_stmt {
            match self.parse_simple_stmt()? {
                crate::ast::Stmt {
                    kind: StmtKind::Expr(expr),
                    ..
                } => RunBody::Expr(expr),
                other => {
                    let span = other.span;
                    RunBody::Block(Block {
                        stmts: vec![other],
                        span,
                        no_abc: false,
                        no_aoc: false,
                    })
                }
            }
        } else {
            RunBody::Expr(self.parse_expr()?)
        };
        let end = self.prev_span();
        Ok(mk(
            ExprKind::Run {
                body: Rc::new(body),
                flags,
            },
            start.to(end),
        ))
    }

    fn parse_code(&mut self) -> PResult<Expr> {
        let start = self.bump();
        let flags = self.parse_directive_flags()?;
        let body = if flags.iter().any(|flag| flag.name.name.as_str() == "null") {
            // `#code,null`: the empty code value.
            CodeBody::Expr(mk(ExprKind::Null, start))
        } else if self.at(P::LBrace) {
            CodeBody::Block(self.parse_block()?)
        } else if self.at_directive("add_context") {
            let statement = self.parse_add_context_core()?;
            let span = statement.span;
            CodeBody::Block(Block {
                stmts: vec![statement],
                span,
                no_abc: false,
                no_aoc: false,
            })
        } else {
            // `#code a := 1` and `#code x = x + 1` are statements without a terminator.
            let stmt = self.parse_simple_stmt()?;
            match stmt.kind {
                StmtKind::Expr(expr) => CodeBody::Expr(expr),
                _ => {
                    let span = stmt.span;
                    CodeBody::Block(Block {
                        stmts: vec![stmt],
                        span,
                        no_abc: false,
                        no_aoc: false,
                    })
                }
            }
        };
        let end = self.prev_span();
        Ok(mk(ExprKind::Code(Rc::new(body)), start.to(end)))
    }

    fn parse_insert_expr(&mut self) -> PResult<Expr> {
        let start = self.bump();
        let (flags, scope) = Self::split_insert_flags(self.parse_directive_flags()?);
        let replacements = self.parse_insert_replacements()?;
        let value = self.parse_expr()?;
        let span = start.to(value.span);
        let kind = ExprKind::Insert {
            value: Box::new(value),
            flags,
            scope: scope.map(Box::new),
            replacements,
        };
        Ok(mk(kind, span))
    }

    /// `#insert (remove = {..}, break = break x) code`: replacements for `#insert`ed identifiers.
    pub(super) fn parse_insert_replacements(&mut self) -> PResult<Vec<crate::ast::Arg>> {
        if !(self.at(P::LParen) && matches!(self.tok_at(1), Tok::Ident(_)) && self.at_n(2, P::Eq)) {
            return Ok(Vec::new());
        }
        self.bump();
        let mut replacements = Vec::new();
        while !self.at(P::RParen) {
            let name = self.ident("as replacement name")?;
            self.expect(P::Eq, "after the replacement name")?;
            let value = self.parse_replacement_value()?;
            replacements.push(crate::ast::Arg {
                name: Some(name),
                target: None,
                context: false,
                spread: false,
                value,
            });
            if !self.eat(P::Comma) {
                break;
            }
        }
        self.expect(P::RParen, "to end the '#insert' replacements")?;
        Ok(replacements)
    }

    /// A replacement is an expression, a block, or a jump statement such as `break outer`.
    fn parse_replacement_value(&mut self) -> PResult<Expr> {
        let jump = match self.kw() {
            Some("break") => Some(StmtKind::Break as fn(Option<Ident>) -> StmtKind),
            Some("continue") => Some(StmtKind::Continue as fn(Option<Ident>) -> StmtKind),
            _ => None,
        };
        let Some(make) = jump else {
            return self.parse_expr();
        };
        let start = self.bump();
        let label = if matches!(self.tok(), Tok::Ident(_)) {
            Some(self.ident("as label")?)
        } else {
            None
        };
        let span = start.to(self.prev_span());
        Ok(mk(
            ExprKind::Block(Block {
                stmts: vec![super::stmt::stmt(make(label), span)],
                span,
                no_abc: false,
                no_aoc: false,
            }),
            span,
        ))
    }

    /// `#assert cond "message"` in expression position becomes a block holding the assert statement.
    fn parse_assert_expr(&mut self) -> PResult<Expr> {
        let assert = self.parse_assert_core()?;
        let span = assert.span;
        Ok(mk(
            ExprKind::Block(Block {
                stmts: vec![assert],
                span,
                no_abc: false,
                no_aoc: false,
            }),
            span,
        ))
    }

    /// Separates plain flags from the `scope(expr)` argument.
    pub(super) fn split_insert_flags(flags: Vec<DirectiveFlag>) -> (Vec<Ident>, Option<Expr>) {
        let mut names = Vec::new();
        let mut scope = None;
        for flag in flags {
            match flag.arg {
                Some(arg) if flag.name.name.as_str() == "scope" => scope = Some(arg),
                _ => names.push(flag.name),
            }
        }
        (names, scope)
    }

    fn parse_type_directive(&mut self) -> PResult<Expr> {
        let start = self.bump();
        let mut modifier = TypeModifier::Plain;
        for flag in self.parse_directive_flags()? {
            match flag.name.name.as_str() {
                "distinct" => modifier = TypeModifier::Distinct,
                "isa" => modifier = TypeModifier::Isa,
                _ => {}
            }
        }
        // `#type (...)` is always a procedure type: `#type (*T)` takes one `*T` argument.
        let ty = if self.at(P::LParen) {
            self.parse_proc_expr(crate::ast::CallHintFlag::None)?
        } else {
            self.parse_unary()?
        };
        let span = start.to(ty.span);
        Ok(mk(
            ExprKind::TypeDirective {
                modifier,
                ty: Box::new(ty),
            },
            span,
        ))
    }

    fn parse_char(&mut self) -> PResult<Expr> {
        let start = self.bump();
        let (bytes, span) = self.string_lit("after '#char'")?;
        let value = match std::str::from_utf8(&bytes) {
            Ok(text) => text.chars().next().map_or(0, |c| c as u32),
            Err(_) => bytes.first().copied().unwrap_or(0) as u32,
        };
        Ok(mk(ExprKind::Char(value), start.to(span)))
    }

    /// `#directive` or `#directive(expr)`.
    fn parse_optional_operand(
        &mut self,
        start: crate::source::Span,
        ctor: fn(Option<Box<Expr>>) -> ExprKind,
    ) -> PResult<Expr> {
        self.bump();
        let mut end = start;
        let mut operand = None;
        if self.at(P::LParen) && self.span().start == self.prev_span().end {
            self.bump();
            if !self.at(P::RParen) {
                operand = Some(Box::new(self.parse_expr()?));
            }
            end = self.expect(P::RParen, "to close the directive operand")?;
        }
        Ok(mk(ctor(operand), start.to(end)))
    }

    /// `#directive operand` where the operand is a unary expression.
    fn parse_operand(
        &mut self,
        start: crate::source::Span,
        ctor: fn(Box<Expr>) -> ExprKind,
    ) -> PResult<Expr> {
        self.bump();
        let operand = self.parse_unary()?;
        let span = start.to(operand.span);
        Ok(mk(ctor(Box::new(operand)), span))
    }

    fn parse_exists(&mut self) -> PResult<Expr> {
        let start = self.bump();
        self.expect(P::LParen, "after '#exists'")?;
        let operand = self.parse_expr()?;
        let end = self.expect(P::RParen, "after the '#exists' operand")?;
        Ok(mk(ExprKind::Exists(Box::new(operand)), start.to(end)))
    }

    /// `#bake_arguments f(a = 1)` / `#bake_constants f(T = int)`.
    fn parse_bake(&mut self, constants: bool) -> PResult<Expr> {
        let start = self.bump();
        let operand = self.parse_unary()?;
        let span = start.to(operand.span);
        match operand.kind {
            ExprKind::Call {
                callee,
                args,
                ..
            } => Ok(mk(
                ExprKind::Bake {
                    callee,
                    args,
                    constants,
                },
                span,
            )),
            _ => Err(crate::source::Diagnostic::error(
                operand.span,
                "expected a call after the bake directive",
            )),
        }
    }

    /// At `,flag,flag "string"`: flags that belong to the directive rather than to an argument list.
    fn flags_before_string_ahead(&self) -> bool {
        let mut i = 0;
        while self.at_n(i, P::Comma) && matches!(self.tok_at(i + 1), Tok::Ident(_)) {
            i += 2;
        }
        i > 0 && matches!(self.tok_at(i), Tok::Str(_))
    }

    /// Directives the parser has no dedicated node for, with an optional string operand
    /// (`#library "x"`, `#system_library "x"`, `#foreign_library "x"`).
    fn parse_unknown_directive(&mut self) -> PResult<Expr> {
        let start = self.span();
        let Tok::Directive(sym) = self.tok().clone() else {
            return Err(self.expected("directive", ""));
        };
        self.bump();
        let name = Ident {
            name: sym,
            span: start,
        };
        let flags = if self.flags_before_string_ahead() {
            self.parse_directive_flags()?
                .into_iter()
                .map(|f| f.name)
                .collect()
        } else {
            Vec::new()
        };
        let operand = if matches!(self.tok(), Tok::Str(_)) {
            Some(Box::new(self.parse_primary()?))
        } else {
            None
        };
        let end = self.prev_span();
        Ok(mk(
            ExprKind::UnknownDirective {
                name,
                flags,
                operand,
            },
            start.to(end),
        ))
    }
}
