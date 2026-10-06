//! Expressions: precedence climbing, prefix/postfix operators and primaries.
use super::{PResult, Parser};
use crate::ast::{Arg, BinOp, CallHint, CallHintFlag, CastFlags, Expr, ExprKind, Ident, UnOp};
use crate::intern::Sym;
use crate::lexer::{P, Tok};
use crate::source::Span;

pub(super) fn mk(kind: ExprKind, span: Span) -> Expr {
    Expr {
        kind,
        span,
    }
}

/// Binary operator precedence, lowest first; every level is left associative.
///
/// This is Jai's table, not C's: the bitwise and shift operators share one level that binds
/// tighter than `*` (`1 << 2 + 3` is 7, `1 | 2 & 4` is 0), and `%` sits between `*`/`/`
/// and `+`/`-` (`10 % 3 * 2` is 4, `n - i % n` takes the remainder first). Prefix operators
/// bind tighter still; a prefix `cast`/`xx` sits at `CAST_PREC`. The evidence for each
/// relation is in docs/language/operators.md.
fn binary_op(p: P) -> Option<(u8, BinOp)> {
    Some(match p {
        P::OrOr => (1, BinOp::Or),
        P::AndAnd => (2, BinOp::And),
        P::EqEq => (3, BinOp::Eq),
        P::Ne => (3, BinOp::Ne),
        P::Lt => (4, BinOp::Lt),
        P::Le => (4, BinOp::Le),
        P::Gt => (4, BinOp::Gt),
        P::Ge => (4, BinOp::Ge),
        P::Plus => (5, BinOp::Add),
        P::Minus => (5, BinOp::Sub),
        P::Percent => (6, BinOp::Rem),
        P::Star => (7, BinOp::Mul),
        P::Slash => (7, BinOp::Div),
        P::Amp => (BITWISE_PREC, BinOp::BitAnd),
        P::Pipe => (BITWISE_PREC, BinOp::BitOr),
        P::Caret => (BITWISE_PREC, BinOp::BitXor),
        P::Shl => (BITWISE_PREC, BinOp::Shl),
        P::Shr => (BITWISE_PREC, BinOp::Shr),
        P::Rotl => (BITWISE_PREC, BinOp::Rotl),
        P::Rotr => (BITWISE_PREC, BinOp::Rotr),
        _ => return None,
    })
}

/// The level of a prefix `cast(T)` / `xx`: its value takes the operators that bind tighter
/// (the bitwise and shift level), and the cast's result is the operand of everything looser.
const CAST_PREC: u8 = 8;
const BITWISE_PREC: u8 = CAST_PREC + 1;

impl Parser<'_> {
    pub(super) fn parse_expr(&mut self) -> PResult<Expr> {
        self.parse_binary(0)
    }

    /// Precedence climbing: an operand followed by the operators that bind tighter than
    /// `min_prec`.
    fn parse_binary(&mut self, min_prec: u8) -> PResult<Expr> {
        let lhs = self.parse_unary()?;
        self.parse_binary_after(lhs, min_prec)
    }

    /// Continues `parse_binary` after its first operand.
    fn parse_binary_after(&mut self, mut lhs: Expr, min_prec: u8) -> PResult<Expr> {
        while let Some((prec, op)) = self.peek_binary_op() {
            if prec <= min_prec {
                break;
            }
            self.bump();
            let rhs = self.parse_binary(prec)?;
            let span = lhs.span.to(rhs.span);
            lhs = mk(ExprKind::Binary(op, Box::new(lhs), Box::new(rhs)), span);
        }
        Ok(lhs)
    }

    /// The value of a prefix `cast(T)` or `xx`: a unary operand plus any bitwise and shift
    /// operators after it (`cast(float) (x >> 16) & 0xFF` casts the masked integer), but
    /// not `*`, `+`, comparisons or logical operators, which apply to the cast's result
    /// (`cast(s64) p - cast(s64) q` subtracts two integers).
    fn parse_cast_value(&mut self) -> PResult<Expr> {
        let first = self.parse_unary()?;
        self.parse_binary_after(first, CAST_PREC)
    }

    fn peek_binary_op(&self) -> Option<(u8, BinOp)> {
        let Tok::Punct(p) = self.tok() else {
            return None;
        };
        // `if x == {` opens a switch rather than continuing the expression.
        if *p == P::EqEq && self.switch_body_follows(1) {
            return None;
        }
        // After `}` on the previous line, `*p = 1;` and `-x;` start new statements.
        if self.continues_after_block() && matches!(p, P::Star | P::Minus | P::Plus | P::Shl) {
            return None;
        }
        binary_op(*p)
    }

    /// True at `{` or `#complete {` starting at offset `n`.
    pub(super) fn switch_body_follows(&self, n: usize) -> bool {
        self.at_n(n, P::LBrace)
            || (self.directive_at(n) == Some("complete") && self.at_n(n + 1, P::LBrace))
    }

    /// A line starting right after a `}` begins a new statement, not an operator continuation.
    fn continues_after_block(&self) -> bool {
        self.newline_before() && self.pos > 0 && self.at_prev(P::RBrace)
    }
    pub(super) fn at_prev(&self, p: P) -> bool {
        self.pos > 0 && matches!(&self.toks[self.pos - 1].tok, Tok::Punct(q) if *q == p)
    }

    // -- unary ------------------------------------------------------------

    pub(super) fn parse_unary(&mut self) -> PResult<Expr> {
        self.nested(Self::parse_unary_inner)
    }

    fn parse_unary_inner(&mut self) -> PResult<Expr> {
        let start = self.span();
        let op = match self.tok() {
            Tok::Punct(P::Minus) => UnOp::Neg,
            Tok::Punct(P::Plus) => UnOp::Plus,
            Tok::Punct(P::Bang) => UnOp::Not,
            Tok::Punct(P::Tilde) => UnOp::BitNot,
            Tok::Punct(P::Star) => UnOp::Star,
            Tok::Punct(P::Shl) => UnOp::Deref,
            Tok::Punct(P::Backtick) => return self.parse_backtick(),
            Tok::Punct(P::LParen) if self.at_n(1, P::DotStar) && self.at_n(2, P::RParen) => {
                // `(.*) value` is a prefix dereference.
                self.bump();
                self.bump();
                UnOp::Deref
            }
            Tok::Ident(name) => match name.as_str() {
                "cast" => return self.parse_cast(),
                "xx" if self.xx_is_cast() => return self.parse_xx(),
                "inline" | "no_inline" => return self.parse_inline(),
                _ => return self.parse_postfix(true),
            },
            _ => return self.parse_postfix(true),
        };
        self.bump();
        let operand = self.parse_unary()?;
        let span = start.to(operand.span);
        Ok(mk(ExprKind::Unary(op, Box::new(operand)), span))
    }

    fn parse_backtick(&mut self) -> PResult<Expr> {
        let start = self.bump();
        let inner = self.parse_primary()?;
        let span = start.to(inner.span);
        let expr = mk(ExprKind::Backtick(Box::new(inner)), span);
        self.postfix_loop(expr, true)
    }

    fn parse_cast(&mut self) -> PResult<Expr> {
        let start = self.bump();
        let flags = self.parse_cast_flags();
        self.expect(P::LParen, "after 'cast'")?;
        let ty = self.parse_expr()?;
        // Newer syntax: `cast(T, value)`.
        if self.eat(P::Comma) {
            let value = self.parse_expr()?;
            let end = self.expect(P::RParen, "after the value in 'cast'")?;
            let span = start.to(end);
            let cast = mk(
                ExprKind::Cast {
                    ty: Some(Box::new(ty)),
                    value: Box::new(value),
                    flags,
                },
                span,
            );
            return self.postfix_loop(cast, true);
        }
        self.expect(P::RParen, "after the type in 'cast'")?;
        let value = self.parse_cast_value()?;
        let span = start.to(value.span);
        Ok(mk(
            ExprKind::Cast {
                ty: Some(Box::new(ty)),
                value: Box::new(value),
                flags,
            },
            span,
        ))
    }

    /// `xx` is the auto-cast operator unless it is used as a plain name (`xx, y`).
    fn xx_is_cast(&self) -> bool {
        !matches!(
            self.tok_at(1),
            Tok::Punct(
                P::Comma | P::Semi | P::RParen | P::Eq | P::Colon | P::ColonEq | P::ColonColon
            )
        ) || matches!(
            self.kw_at(2),
            Some("no_check" | "trunc" | "truncate" | "force")
        )
    }

    fn parse_xx(&mut self) -> PResult<Expr> {
        let start = self.bump();
        let flags = self.parse_cast_flags();
        let value = self.parse_cast_value()?;
        let span = start.to(value.span);
        Ok(mk(
            ExprKind::Cast {
                ty: None,
                value: Box::new(value),
                flags,
            },
            span,
        ))
    }

    fn parse_cast_flags(&mut self) -> CastFlags {
        let mut flags = CastFlags::default();
        while self.at(P::Comma)
            && matches!(
                self.kw_at(1),
                Some("no_check" | "trunc" | "truncate" | "force")
            )
        {
            self.bump();
            match self.kw() {
                Some("no_check") => flags.no_check = true,
                Some("force") => flags.force = true,
                _ => flags.truncate = true,
            }
            self.bump();
        }
        flags
    }

    /// `inline f(x)` / `no_inline f(x)` call hints, and `inline (a: int) {..}` procedure flags.
    fn parse_inline(&mut self) -> PResult<Expr> {
        let hint = if self.at_kw("inline") {
            CallHint::Inline
        } else {
            CallHint::NoInline
        };
        if self.at_n(1, P::LParen) && self.paren_starts_header(1) || self.at_n(1, P::Arrow) {
            self.bump();
            let flag = if hint == CallHint::Inline {
                CallHintFlag::Inline
            } else {
                CallHintFlag::NoInline
            };
            let proc = self.parse_proc_expr(flag)?;
            return self.postfix_loop(proc, true);
        }
        let start = self.bump();
        let mut operand = self.parse_unary()?;
        if let ExprKind::Call {
            hint: slot, ..
        } = &mut operand.kind
        {
            *slot = hint;
        }
        operand.span = start.to(operand.span);
        Ok(operand)
    }

    // -- postfix ----------------------------------------------------------

    /// Primary expression followed by member access, calls, indexing and literals.
    /// With `literals == false`, `.{` / `.[` are not consumed (array element types).
    pub(super) fn parse_postfix(&mut self, literals: bool) -> PResult<Expr> {
        let primary = self.parse_primary()?;
        self.postfix_loop(primary, literals)
    }

    pub(super) fn postfix_loop(&mut self, mut expr: Expr, literals: bool) -> PResult<Expr> {
        loop {
            let start = expr.span;
            match self.tok() {
                Tok::Punct(P::Dot | P::DotStar) if self.continues_after_block() => return Ok(expr),
                Tok::Punct(P::Dot) if self.at_n(1, P::LParen) => {
                    // `value.(T)` is `cast(T) value`.
                    self.bump();
                    self.bump();
                    let ty = self.parse_expr()?;
                    // `value.(u32, trunc)`: cast flags after the type.
                    let mut flags = CastFlags::default();
                    while self.eat(P::Comma) {
                        match self.ident("as a cast flag")?.name.as_str() {
                            "no_check" => flags.no_check = true,
                            "force" => flags.force = true,
                            _ => flags.truncate = true,
                        }
                    }
                    let end = self.expect(P::RParen, "after the type in '.(T)'")?;
                    let kind = ExprKind::Cast {
                        ty: Some(Box::new(ty)),
                        value: Box::new(expr),
                        flags,
                    };
                    expr = mk(kind, start.to(end));
                }
                Tok::Punct(P::Dot) => {
                    self.bump();
                    let member = if self.operator_name_follows() {
                        self.parse_operator_name()?
                    } else {
                        self.ident("after '.'")?
                    };
                    expr = mk(
                        ExprKind::Member(Box::new(expr), member),
                        start.to(member.span),
                    );
                }
                Tok::Punct(P::DotStar) => {
                    let end = self.bump();
                    expr = mk(ExprKind::Unary(UnOp::Deref, Box::new(expr)), start.to(end));
                }
                Tok::Punct(P::DotBrace) if literals => {
                    self.bump();
                    let fields = self.parse_args(P::RBrace, "in struct literal")?;
                    let end = self.prev_span();
                    expr = mk(
                        ExprKind::StructLit {
                            ty: Some(Box::new(expr)),
                            fields,
                        },
                        start.to(end),
                    );
                }
                Tok::Punct(P::DotBracket) if literals => {
                    self.bump();
                    let elems = self.parse_elements(P::RBracket, "in array literal")?;
                    let end = self.prev_span();
                    expr = mk(
                        ExprKind::ArrayLit {
                            ty: Some(Box::new(expr)),
                            elems,
                        },
                        start.to(end),
                    );
                }
                Tok::Punct(P::LBracket) if !self.continues_after_block() => {
                    self.bump();
                    let index = self.parse_expr()?;
                    let end = self.expect(P::RBracket, "after index expression")?;
                    expr = mk(
                        ExprKind::Index(Box::new(expr), Box::new(index)),
                        start.to(end),
                    );
                }
                Tok::Punct(P::LParen) if !self.continues_after_block() => {
                    self.bump();
                    let args = self.parse_args(P::RParen, "in argument list")?;
                    let end = self.prev_span();
                    expr = mk(
                        ExprKind::Call {
                            callee: Box::new(expr),
                            args,
                            hint: CallHint::None,
                        },
                        start.to(end),
                    );
                }
                _ => return Ok(expr),
            }
        }
    }

    /// Comma-separated call arguments / literal fields up to and including `close`.
    pub(super) fn parse_args(&mut self, close: P, context: &str) -> PResult<Vec<Arg>> {
        let saved = std::mem::replace(&mut self.in_list, true);
        let mut args = Vec::new();
        // `f(,, allocator = temp)`: context arguments with no ordinary arguments before them.
        let mut after_context_comma = self.at(P::Comma) && self.at_n(1, P::Comma);
        if after_context_comma {
            self.bump();
            self.bump();
        }
        while !self.at(close) {
            let mut arg = self.parse_arg()?;
            arg.context = after_context_comma;
            args.push(arg);
            if !self.eat(P::Comma) {
                break;
            }
            // `f(a, b,, allocator = temp)`: everything after `,,` modifies the context.
            after_context_comma |= self.eat(P::Comma);
        }
        self.in_list = saved;
        self.expect(close, context)?;
        Ok(args)
    }

    pub(super) fn parse_arg(&mut self) -> PResult<Arg> {
        let mut spread = self.eat(P::DotDot);
        let (mut name, mut target) = (None, None);
        if !spread && let Some(simple) = self.named_arg_ahead() {
            if simple {
                name = Some(self.ident("as argument name")?);
            } else {
                target = Some(self.parse_postfix(true)?);
            }
            self.bump();
            spread = self.eat(P::DotDot);
        }
        let value = self.parse_expr()?;
        Ok(Arg {
            name,
            target,
            context: false,
            spread,
            value,
        })
    }

    /// At `name =` (`Some(true)`) or `a.b[i] =` (`Some(false)`).
    fn named_arg_ahead(&self) -> Option<bool> {
        if !matches!(self.tok_at(0), Tok::Ident(_)) {
            return None;
        }
        let mut i = 1;
        loop {
            match (self.tok_at(i), self.tok_at(i + 1)) {
                (Tok::Punct(P::Dot), Tok::Ident(_)) => i += 2,
                (Tok::Punct(P::LBracket), _) => i = self.bracket_end(i)?,
                (Tok::Punct(P::Eq), _) => return Some(i == 1),
                _ => return None,
            }
        }
    }

    /// Offset just past the `]` matching the `[` at offset `n`.
    pub(super) fn bracket_end(&self, n: usize) -> Option<usize> {
        let mut depth = 0usize;
        for i in n.. {
            match self.tok_at(i) {
                Tok::Punct(P::LBracket | P::DotBracket) => depth += 1,
                Tok::Punct(P::RBracket) => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(i + 1);
                    }
                }
                Tok::Eof => return None,
                _ => {}
            }
        }
        None
    }

    /// Comma-separated plain expressions up to and including `close`.
    pub(super) fn parse_elements(&mut self, close: P, context: &str) -> PResult<Vec<Expr>> {
        let saved = std::mem::replace(&mut self.in_list, true);
        let mut elems = Vec::new();
        while !self.at(close) {
            elems.push(self.parse_expr()?);
            if !self.eat(P::Comma) {
                break;
            }
        }
        self.in_list = saved;
        self.expect(close, context)?;
        Ok(elems)
    }

    // -- primary ----------------------------------------------------------

    pub(super) fn parse_primary(&mut self) -> PResult<Expr> {
        let span = self.span();
        match self.tok().clone() {
            Tok::Int(v) => {
                self.bump();
                Ok(mk(ExprKind::Int(v), span))
            }
            Tok::Float(v) => {
                self.bump();
                Ok(mk(ExprKind::Float(v), span))
            }
            Tok::Str(s) => {
                self.bump();
                Ok(mk(ExprKind::Str(s), span))
            }
            Tok::Ident(name) => self.parse_ident_primary(name),
            Tok::Directive(name) => self.parse_directive_expr(name.as_str()),
            Tok::Punct(p) => self.parse_punct_primary(p),
            Tok::Note(_) | Tok::Eof => Err(self.expected("expression", "")),
        }
    }

    fn parse_ident_primary(&mut self, name: Sym) -> PResult<Expr> {
        let span = self.span();
        match name.as_str() {
            "true" | "false" => {
                self.bump();
                Ok(mk(ExprKind::Bool(name.as_str() == "true"), span))
            }
            "null" => {
                self.bump();
                Ok(mk(ExprKind::Null, span))
            }
            "context" => {
                self.bump();
                Ok(mk(ExprKind::Context, span))
            }
            "struct" | "union" => self.parse_struct_expr(),
            "enum" | "enum_flags" => self.parse_enum_expr(),
            "ifx" => self.parse_ifx(false),
            "operator" if self.operator_name_follows() => {
                let name = self.parse_operator_name()?;
                Ok(mk(ExprKind::Ident(name.name), name.span))
            }
            _ if self.at_n(1, P::FatArrow) => self.parse_lambda(),
            _ => {
                self.bump();
                Ok(mk(ExprKind::Ident(name), span))
            }
        }
    }

    fn parse_punct_primary(&mut self, p: P) -> PResult<Expr> {
        let span = self.span();
        match p {
            P::LParen if self.paren_starts_header(0) => self.parse_proc_expr(Default::default()),
            P::Arrow => self.parse_proc_expr(Default::default()),
            P::LParen => {
                self.bump();
                let inner = self.parse_expr()?;
                self.expect(P::RParen, "to close the parenthesized expression")?;
                Ok(inner)
            }
            P::LBracket => self.parse_array_type(),
            P::DotBrace => {
                self.bump();
                let fields = self.parse_args(P::RBrace, "in struct literal")?;
                Ok(mk(
                    ExprKind::StructLit {
                        ty: None,
                        fields,
                    },
                    span.to(self.prev_span()),
                ))
            }
            P::DotBracket => {
                self.bump();
                let elems = self.parse_elements(P::RBracket, "in array literal")?;
                Ok(mk(
                    ExprKind::ArrayLit {
                        ty: None,
                        elems,
                    },
                    span.to(self.prev_span()),
                ))
            }
            P::Dot => {
                self.bump();
                let member = self.ident("after '.'")?;
                Ok(mk(ExprKind::InferredMember(member), span.to(member.span)))
            }
            P::LBrace if self.brace_is_struct_literal() => {
                self.bump();
                let fields = self.parse_args(P::RBrace, "in struct literal")?;
                Ok(mk(
                    ExprKind::StructLit {
                        ty: None,
                        fields,
                    },
                    span.to(self.prev_span()),
                ))
            }
            P::LBrace => {
                let block = self.parse_block()?;
                let span = block.span;
                Ok(mk(ExprKind::Block(block), span))
            }
            P::Uninit => {
                self.bump();
                Ok(mk(ExprKind::Uninit, span))
            }
            P::Dollar | P::DollarDollar => self.parse_poly_var(),
            _ => Err(self.expected("expression", "")),
        }
    }

    /// At `{` in expression position: `{ a = 1, b = 2 }` is a struct literal, a block has `;`s.
    fn brace_is_struct_literal(&self) -> bool {
        if self.at_n(1, P::RBrace) {
            return true;
        }
        if !(matches!(self.tok_at(1), Tok::Ident(_)) && self.at_n(2, P::Eq)) {
            return false;
        }
        let mut depth = 0usize;
        for i in 0.. {
            match self.tok_at(i) {
                Tok::Punct(P::LBrace | P::DotBrace | P::LParen | P::LBracket | P::DotBracket) => {
                    depth += 1
                }
                Tok::Punct(P::RBrace | P::RParen | P::RBracket) => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                Tok::Punct(P::Semi) if depth == 1 => return false,
                Tok::Eof => return false,
                _ => {}
            }
        }
        true
    }

    /// At `operator` followed by an operator symbol (`Basic.operator-`, `operator==`).
    pub(super) fn operator_name_follows(&self) -> bool {
        self.at_kw("operator")
            && matches!(
                self.tok_at(1),
                Tok::Punct(
                    P::Plus
                        | P::Minus
                        | P::Star
                        | P::Slash
                        | P::Percent
                        | P::EqEq
                        | P::Ne
                        | P::Lt
                        | P::Le
                        | P::Gt
                        | P::Ge
                        | P::Amp
                        | P::Pipe
                        | P::Caret
                        | P::Shl
                        | P::Shr
                        | P::LBracket
                )
            )
    }

    /// Consumes `operator` and its symbol(s) as one identifier named e.g. `operator[]=`.
    pub(super) fn parse_operator_name(&mut self) -> PResult<Ident> {
        let start = self.bump();
        let mut text = String::from("operator");
        if let Tok::Punct(symbol) = *self.tok() {
            text.push_str(symbol.text());
            self.bump();
            if symbol == P::LBracket {
                self.expect(P::RBracket, "in operator name")?;
                text.push(']');
                if self.eat(P::Eq) {
                    text.push('=');
                }
            }
        }
        Ok(Ident {
            name: Sym::intern(&text),
            span: start.to(self.prev_span()),
        })
    }

    /// `[N] T`, `[] T`, `[..] T`. The element type does not take `.{`/`.[` literals,
    /// so `[2]int.[1, 2]` is an array literal of type `[2]int`.
    fn parse_array_type(&mut self) -> PResult<Expr> {
        use crate::ast::ArraySize;
        let start = self.bump();
        let size = if self.eat(P::RBracket) {
            ArraySize::View
        } else if self.at(P::DotDot) && self.at_n(1, P::RBracket) {
            self.bump();
            self.bump();
            ArraySize::Resizable
        } else {
            let n = self.parse_expr()?;
            self.expect(P::RBracket, "after array size")?;
            ArraySize::Fixed(Box::new(n))
        };
        let elem = self.parse_element_type()?;
        let span = start.to(elem.span);
        Ok(mk(
            ExprKind::ArrayType {
                size,
                elem: Box::new(elem),
            },
            span,
        ))
    }

    fn parse_element_type(&mut self) -> PResult<Expr> {
        self.nested(Self::parse_element_type_inner)
    }

    fn parse_element_type_inner(&mut self) -> PResult<Expr> {
        if self.at(P::Star) {
            let start = self.bump();
            let inner = self.parse_element_type()?;
            let span = start.to(inner.span);
            return Ok(mk(ExprKind::Unary(UnOp::Star, Box::new(inner)), span));
        }
        self.parse_postfix(false)
    }

    /// `$T`, `$$x`, `$T/Interface`, `$T/interface Interface`.
    fn parse_poly_var(&mut self) -> PResult<Expr> {
        let start = self.span();
        let baked = self.at(P::DollarDollar);
        self.bump();
        let name = self.ident("after '$'")?;
        let mut end = name.span;
        if self.at(P::Slash) {
            self.bump();
            let interface = self.at_kw("interface") && matches!(self.tok_at(1), Tok::Ident(_));
            if interface {
                self.bump();
            }
            let restriction = self.parse_postfix(true)?;
            end = restriction.span;
            let kind = ExprKind::PolyRestricted {
                name: name.name,
                restriction: Box::new(restriction),
                interface,
            };
            return Ok(mk(kind, start.to(end)));
        }
        Ok(mk(
            ExprKind::PolyVar {
                name: name.name,
                baked,
            },
            start.to(end),
        ))
    }

    pub(super) fn parse_ifx(&mut self, is_static: bool) -> PResult<Expr> {
        let start = self.bump();
        let cond = self.parse_expr()?;
        let ends_here = matches!(
            self.tok(),
            Tok::Punct(P::Semi | P::Comma | P::RParen | P::RBracket | P::RBrace) | Tok::Eof
        );
        let then_value = if self.at_kw("else") || ends_here {
            None
        } else {
            self.eat_kw("then");
            Some(Box::new(self.parse_branch_value()?))
        };
        // `#ifx c then a; else b;` (seen in older code): the `;` before `else` is dropped.
        if is_static && matches!(self.tok(), Tok::Punct(P::Semi)) && self.kw_at(1) == Some("else") {
            self.bump();
        }
        let else_value = if self.eat_kw("else") {
            Some(Box::new(self.parse_branch_value()?))
        } else {
            None
        };
        let span = start.to(self.prev_span());
        Ok(mk(
            ExprKind::Ifx {
                cond: Box::new(cond),
                then_value,
                else_value,
                is_static,
            },
            span,
        ))
    }

    fn parse_branch_value(&mut self) -> PResult<Expr> {
        if self.at(P::LBrace) {
            let block = self.parse_block()?;
            let span = block.span;
            return Ok(mk(ExprKind::Block(block), span));
        }
        self.parse_expr()
    }
}
