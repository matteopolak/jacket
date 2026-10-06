//! Procedure headers, parameters, return lists, flags and lambdas.
use super::expr::mk;
use super::{PResult, Parser};
use crate::ast::{
    AstId, CallHintFlag, Expr, ExprKind, Foreign, Param, ProcFlags, ProcHeader, ProcLit, Return,
    UsingFilter,
};
use crate::lexer::{P, Tok};
use std::rc::Rc;

/// Directives that can follow a header's closing parenthesis.
const HEADER_DIRECTIVES: &[&str] = &[
    "c_call",
    "no_context",
    "expand",
    "compiler",
    "intrinsic",
    "elsewhere",
    "foreign",
    "symmetric",
    "cpp_method",
    "cpp_return_type_is_non_pod",
    "runtime_support",
    "no_debug",
    "no_abc",
    "no_aoc",
    "deprecated",
    "program_export",
    "modify",
    "type_info_none",
    "no_call",
];

/// Directives that are kept by name in `ProcFlags::other`.
const OTHER_FLAGS: &[&str] = &["dump", "entry_point", "no_alias", "compile_time"];

fn new_header(start: crate::source::Span, inline: CallHintFlag) -> ProcHeader {
    ProcHeader {
        id: AstId::fresh(),
        params: Vec::new(),
        returns: Vec::new(),
        flags: ProcFlags {
            inline,
            ..ProcFlags::default()
        },
        foreign: None,
        modify: None,
        notes: Vec::new(),
        span: start,
        operator: None,
    }
}

impl Parser<'_> {
    // -- header detection -------------------------------------------------

    /// At `(` (offset `n`): does it start a procedure header rather than a parenthesized expression?
    pub(super) fn paren_starts_header(&self, n: usize) -> bool {
        let Some(close) = self.matching_paren(n) else {
            return false;
        };
        let after = &self.toks[(close + 1).min(self.toks.len() - 1)].tok;
        match after {
            Tok::Punct(P::Arrow | P::FatArrow) => return true,
            Tok::Directive(d) if HEADER_DIRECTIVES.contains(&d.as_str()) => return true,
            _ => {}
        }
        self.interior_looks_like_params(self.pos + n + 1, close)
    }

    /// Index of the `)` matching the `(` at offset `n`.
    pub(super) fn matching_paren(&self, n: usize) -> Option<usize> {
        let start = self.pos + n;
        let mut depth = 0usize;
        for (i, token) in self.toks.iter().enumerate().skip(start) {
            match &token.tok {
                Tok::Punct(P::LParen) => depth += 1,
                Tok::Punct(P::RParen) => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        return Some(i);
                    }
                }
                Tok::Eof => return None,
                _ => {}
            }
        }
        None
    }

    fn interior_looks_like_params(&self, from: usize, close: usize) -> bool {
        if from == close {
            return true;
        }
        let tok = |i: usize| &self.toks[i.min(close)].tok;
        match tok(from) {
            Tok::Punct(P::Dollar | P::DollarDollar | P::DotDot) => true,
            Tok::Directive(d) if d.as_str() == "discard" => true,
            Tok::Ident(name)
                if name.as_str() == "using" && matches!(tok(from + 1), Tok::Ident(_)) =>
            {
                true
            }
            // `(cast,no_check(T) x ...)` is an expression, not `(cast, ...)` parameters.
            Tok::Ident(name) if name.as_str() == "cast" => false,
            Tok::Ident(_)
                if matches!(
                    tok(from + 1),
                    Tok::Punct(P::Colon | P::ColonEq | P::ColonColon | P::Comma)
                ) =>
            {
                true
            }
            Tok::Directive(_) => false,
            _ => self.has_top_level_comma(from, close),
        }
    }

    pub(super) fn has_top_level_comma(&self, from: usize, close: usize) -> bool {
        let mut depth = 0usize;
        // The comma in a cast modifier (`-cast,no_check(int) x`, `xx,trunc v`) separates nothing.
        let cast_modifier = |i: usize| {
            let word = |j: usize| match &self.toks[j].tok {
                Tok::Ident(name) => Some(name.as_str()),
                _ => None,
            };
            i > 0
                && i + 1 < self.toks.len()
                && matches!(
                    word(i - 1),
                    Some("cast" | "xx" | "no_check" | "trunc" | "truncate" | "force")
                )
                && matches!(
                    word(i + 1),
                    Some("no_check" | "trunc" | "truncate" | "force")
                )
        };
        for (i, token) in self.toks[from..close].iter().enumerate() {
            match &token.tok {
                Tok::Punct(P::LParen | P::LBracket | P::LBrace | P::DotBrace | P::DotBracket) => {
                    depth += 1
                }
                Tok::Punct(P::RParen | P::RBracket | P::RBrace) => depth = depth.saturating_sub(1),
                Tok::Punct(P::Comma) if depth == 0 && !cast_modifier(from + i) => return true,
                _ => {}
            }
        }
        false
    }

    // -- procedure literals and types ---------------------------------------

    /// A procedure literal or type starting at `(` or `->`.
    pub(super) fn parse_proc_expr(&mut self, inline: CallHintFlag) -> PResult<Expr> {
        let start = self.span();
        let mut header = new_header(start, inline);
        header.operator = self.pending_operator.take();
        if self.eat(P::LParen) {
            header.params = self.parse_params(P::RParen)?;
        }
        if self.at(P::FatArrow) {
            return self.finish_lambda(header, start);
        }
        if self.eat(P::Arrow) {
            header.returns = self.parse_returns()?;
        }
        self.parse_proc_flags(&mut header)?;
        header.span = start.to(self.prev_span());
        let has_external_body = header.foreign.is_some()
            || header.flags.compiler
            || header.flags.intrinsic
            || header.flags.elsewhere.is_some();
        let header = Rc::new(header);
        if self.at(P::LBrace) {
            let body = self.parse_block()?;
            let span = start.to(body.span);
            return Ok(mk(
                ExprKind::Proc(Rc::new(ProcLit {
                    header,
                    body: Some(body),
                })),
                span,
            ));
        }
        let span = header.span;
        if has_external_body {
            return Ok(mk(
                ExprKind::Proc(Rc::new(ProcLit {
                    header,
                    body: None,
                })),
                span,
            ));
        }
        Ok(mk(ExprKind::ProcType(header), span))
    }

    /// `x => body` (the current token is the parameter name).
    pub(super) fn parse_lambda(&mut self) -> PResult<Expr> {
        let start = self.span();
        let name = self.ident("as lambda parameter")?;
        let mut header = new_header(start, CallHintFlag::None);
        header.params.push(Param {
            name: Some(name),
            baked: false,
            auto_bake: false,
            using: false,
            using_filter: None,
            discard: false,
            variadic: false,
            ty: None,
            default: None,
            notes: Vec::new(),
            span: name.span,
        });
        self.finish_lambda(header, start)
    }

    /// At `=>` after the parameters of a lambda.
    fn finish_lambda(
        &mut self,
        mut header: ProcHeader,
        start: crate::source::Span,
    ) -> PResult<Expr> {
        self.expect(P::FatArrow, "in lambda")?;
        // `(a, b) =>` parses like the unnamed parameter types of a procedure type;
        // a bare identifier there is the parameter's name.
        for p in &mut header.params {
            if p.name.is_none()
                && let Some(Expr {
                    kind: ExprKind::Ident(name),
                    span,
                }) = p.ty.take_if(|t| matches!(t.kind, ExprKind::Ident(_)))
            {
                p.name = Some(crate::ast::Ident {
                    name,
                    span,
                });
            }
        }
        let body = if self.at(P::LBrace) {
            let block = self.parse_block()?;
            let span = block.span;
            mk(ExprKind::Block(block), span)
        } else {
            self.parse_expr()?
        };
        let span = start.to(body.span);
        header.span = span;
        Ok(mk(
            ExprKind::Lambda {
                header: Rc::new(header),
                body: Box::new(body),
            },
            span,
        ))
    }

    // -- parameters ---------------------------------------------------------

    /// Parameters up to and including `close` (the opening token is already consumed).
    pub(super) fn parse_params(&mut self, close: P) -> PResult<Vec<Param>> {
        let saved = std::mem::replace(&mut self.in_list, true);
        let mut params = Vec::new();
        while !self.at(close) {
            self.parse_param_group(&mut params)?;
            if !(self.eat(P::Comma) || self.eat(P::Semi)) {
                break;
            }
        }
        self.in_list = saved;
        self.expect(close, "to end the parameter list")?;
        Ok(params)
    }

    /// A parameter or declaration type. A bare parenthesized list, as in `f: (*Vector3)`, is a procedure type
    /// returning nothing; parentheses around a type mean nothing here.
    pub(super) fn parse_param_type(&mut self) -> PResult<Expr> {
        if self.at(P::LParen) && !self.paren_starts_header(0) {
            let closes_type = self.matching_paren(0).is_some_and(|close| {
                matches!(
                    self.toks[close + 1].tok,
                    Tok::Punct(P::Comma | P::Semi | P::RParen | P::Eq)
                )
            });
            if closes_type {
                return self.parse_proc_expr(Default::default());
            }
        }
        self.parse_expr()
    }

    /// `a, b: T = v`, `$T: Type`, `using x: *X`, `args: ..Any` or a bare type (procedure types).
    fn parse_param_group(&mut self, out: &mut Vec<Param>) -> PResult<()> {
        let start = self.span();
        let (mut using, mut discard, mut using_filter) = (false, false, None);
        loop {
            if self.at_kw("using") && self.using_modifier_follows() {
                self.bump();
                using = true;
                using_filter = Some(self.parse_using_filter()?)
                    .filter(|filter| !matches!(filter, UsingFilter::None));
            } else if self.at_directive("discard") {
                self.bump();
                discard = true;
            } else {
                break;
            }
        }
        if !self.named_param_ahead() {
            let variadic = self.eat(P::DotDot);
            let ty = self.parse_param_type()?;
            let span = start.to(ty.span);
            out.push(Param {
                name: None,
                baked: false,
                auto_bake: false,
                using,
                using_filter,
                discard,
                variadic,
                ty: Some(ty),
                default: None,
                notes: Vec::new(),
                span,
            });
            return Ok(());
        }
        let mut names = Vec::new();
        loop {
            // `$$x` bakes only constant arguments (decided per call in sema).
            let baked = matches!(self.tok(), Tok::Punct(P::Dollar));
            let auto = matches!(self.tok(), Tok::Punct(P::DollarDollar));
            if baked || auto {
                self.bump();
            }
            names.push((self.ident("as parameter name")?, baked, auto));
            if !self.eat(P::Comma) {
                break;
            }
        }
        let (mut variadic, mut ty, mut default) = (false, None, None);
        if self.eat(P::ColonEq) {
            default = Some(self.parse_expr()?);
        } else {
            self.expect(P::Colon, "after the parameter name")?;
            variadic = self.eat(P::DotDot);
            if !matches!(
                self.tok(),
                Tok::Punct(P::Eq | P::Comma | P::Semi | P::RParen)
            ) {
                ty = Some(self.parse_param_type()?);
            }
            if self.eat(P::Eq) {
                default = Some(self.parse_expr()?);
            }
        }
        let notes = self.parse_notes();
        let end = self.prev_span();
        for (name, baked, auto_bake) in names {
            let (ty, default, notes) = (ty.clone(), default.clone(), notes.clone());
            out.push(Param {
                name: Some(name),
                baked,
                auto_bake,
                using,
                using_filter: using_filter.clone(),
                discard,
                variadic,
                ty,
                default,
                notes,
                span: start.to(end),
            });
        }
        Ok(())
    }

    /// At `using` that modifies a parameter, as opposed to a parameter named `using`.
    fn using_modifier_follows(&self) -> bool {
        match self.tok_at(1) {
            Tok::Punct(P::Colon | P::ColonEq) => false,
            Tok::Punct(P::Comma) => matches!(self.kw_at(2), Some("only" | "except" | "map")),
            _ => true,
        }
    }

    /// At `[$]name {, [$]name} :` or `:=`.
    pub(super) fn named_param_ahead(&self) -> bool {
        let mut i = 0;
        loop {
            if matches!(self.tok_at(i), Tok::Punct(P::Dollar | P::DollarDollar)) {
                i += 1;
            }
            if !matches!(self.tok_at(i), Tok::Ident(_)) {
                return false;
            }
            i += 1;
            if !matches!(self.tok_at(i), Tok::Punct(P::Comma)) {
                return matches!(self.tok_at(i), Tok::Punct(P::Colon | P::ColonEq));
            }
            i += 1;
        }
    }

    // -- return values ------------------------------------------------------

    /// After `->`: `int`, `int, bool #must`, `(x: int, y: float)`.
    fn parse_returns(&mut self) -> PResult<Vec<Return>> {
        if self.at(P::LParen) && self.return_list_in_parens() {
            self.bump();
            let mut returns = Vec::new();
            while !self.at(P::RParen) {
                returns.push(self.parse_return_value()?);
                if !(self.eat(P::Comma) || self.eat(P::Semi)) {
                    break;
                }
            }
            self.expect(P::RParen, "to end the return list")?;
            return Ok(returns);
        }
        let mut returns = vec![self.parse_return_value()?];
        // Inside argument and parameter lists a comma separates the arguments instead.
        while !self.in_list && self.at(P::Comma) {
            self.bump();
            returns.push(self.parse_return_value()?);
        }
        Ok(returns)
    }

    /// A parenthesized return list, unless the parentheses are a procedure type's parameters.
    fn return_list_in_parens(&self) -> bool {
        let Some(close) = self.matching_paren(0) else {
            return false;
        };
        !matches!(
            self.toks[(close + 1).min(self.toks.len() - 1)].tok,
            Tok::Punct(P::Arrow)
        )
    }

    fn parse_return_value(&mut self) -> PResult<Return> {
        let start = self.span();
        let name = if matches!(self.tok(), Tok::Ident(_))
            && matches!(self.tok_at(1), Tok::Punct(P::Colon | P::ColonEq))
        {
            Some(self.ident("as return name")?)
        } else {
            None
        };
        let (ty, default) = if name.is_some() && self.eat(P::ColonEq) {
            (None, Some(self.parse_expr()?))
        } else {
            if name.is_some() {
                self.bump();
            }
            let ty = self.parse_expr()?;
            (
                Some(ty),
                // Only named results take defaults; `(K) -> bool = null` in a
                // parameter list is the parameter's default instead.
                if name.is_some() && self.eat(P::Eq) {
                    Some(self.parse_expr()?)
                } else {
                    None
                },
            )
        };
        let must = self.at_directive("must");
        if must {
            self.bump();
        }
        Ok(Return {
            name,
            ty,
            default,
            must,
            span: start.to(self.prev_span()),
        })
    }

    /// `#modify { ... }` or the expression form `#modify check(T)`.
    pub(super) fn parse_modify_body(&mut self) -> PResult<crate::ast::Block> {
        if self.at(P::LBrace) {
            return self.parse_block();
        }
        let expr = self.parse_expr()?;
        let span = expr.span;
        let statement = super::stmt::stmt(crate::ast::StmtKind::Expr(expr), span);
        Ok(crate::ast::Block {
            stmts: vec![statement],
            span,
            no_abc: false,
            no_aoc: false,
        })
    }

    // -- flags --------------------------------------------------------------

    fn parse_proc_flags(&mut self, header: &mut ProcHeader) -> PResult<()> {
        loop {
            match self.tok() {
                Tok::Ident(name) => match name.as_str() {
                    "inline" => header.flags.inline = CallHintFlag::Inline,
                    "no_inline" => header.flags.inline = CallHintFlag::NoInline,
                    _ => return Ok(()),
                },
                Tok::Directive(name) => {
                    let name = *name;
                    if !self.apply_directive_flag(header, name.as_str())? {
                        return Ok(());
                    }
                    continue;
                }
                _ => return Ok(()),
            }
            self.bump();
        }
    }

    /// Consumes the directive at the cursor if it is a procedure flag. Returns false if it is not.
    fn apply_directive_flag(&mut self, header: &mut ProcHeader, name: &str) -> PResult<bool> {
        let span = self.span();
        let flags = &mut header.flags;
        match name {
            "c_call" => flags.c_call = true,
            "no_context" => flags.no_context = true,
            "expand" => flags.expand = true,
            "intrinsic" | "compiler" => {
                self.bump();
                if name == "intrinsic" {
                    header.flags.intrinsic = true;
                } else {
                    header.flags.compiler = true;
                }
                if let Tok::Str(s) = self.tok() {
                    header.flags.builtin_name =
                        Some(String::from_utf8_lossy(s).into_owned().into());
                    self.bump();
                }
                return Ok(true);
            }
            "symmetric" => flags.symmetric = true,
            "cpp_method" => flags.cpp_method = true,
            "cpp_return_type_is_non_pod" => flags.cpp_return_type_is_non_pod = true,
            "runtime_support" => flags.runtime_support = true,
            "no_debug" => flags.no_debug = true,
            "no_abc" => flags.no_abc = true,
            "no_aoc" => flags.no_aoc = true,
            "no_call" => flags.no_call = true,
            "type_info_none" => flags.type_info_none = true,
            "elsewhere" => {
                self.bump();
                let foreign = self.parse_elsewhere_library()?;
                header.flags.elsewhere = Some(foreign.as_ref().and_then(|foreign| foreign.library));
                header.foreign = header.foreign.take().or(foreign);
                return Ok(true);
            }
            "deprecated" | "program_export" => {
                self.bump();
                let text = match self.tok() {
                    Tok::Str(s) => {
                        let s = s.clone();
                        self.bump();
                        Some(s)
                    }
                    _ => None,
                };
                if name == "deprecated" {
                    header.flags.deprecated = Some(text);
                } else {
                    header.flags.program_export = Some(text);
                }
                return Ok(true);
            }
            "foreign" => {
                self.bump();
                let library = if matches!(self.tok(), Tok::Ident(_)) {
                    Some(self.ident("after `#foreign`")?)
                } else {
                    None
                };
                let name = self.parse_foreign_symbol();
                header.foreign = Some(Foreign {
                    library,
                    name,
                });
                return Ok(true);
            }
            "modify" => {
                self.bump();
                header.modify = Some(self.parse_modify_body()?);
                return Ok(true);
            }
            other if OTHER_FLAGS.contains(&other) => {
                let ident = crate::ast::Ident {
                    name: crate::intern::Sym::intern(other),
                    span,
                };
                header.flags.other.push(ident);
                self.bump();
                // `#dump` is followed by the body; nothing to consume here.
                return Ok(true);
            }
            _ => return Ok(false),
        }
        self.bump();
        Ok(true)
    }
}
