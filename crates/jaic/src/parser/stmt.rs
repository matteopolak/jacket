//! Statements and control flow.
use std::rc::Rc;

use super::{PResult, Parser};
use crate::ast::{
    AssignOp, BinOp, Block, Case, Expr, For, ForOver, Ident, Stmt, StmtKind, UsingFilter,
};
use crate::lexer::{P, Tok};

pub(super) fn stmt(kind: StmtKind, span: crate::source::Span) -> Stmt {
    Stmt {
        kind,
        span,
        notes: Vec::new(),
    }
}

fn assign_op(p: P) -> Option<AssignOp> {
    Some(match p {
        P::Eq => AssignOp::Assign,
        P::AddAssign => AssignOp::Op(BinOp::Add),
        P::SubAssign => AssignOp::Op(BinOp::Sub),
        P::MulAssign => AssignOp::Op(BinOp::Mul),
        P::DivAssign => AssignOp::Op(BinOp::Div),
        P::RemAssign => AssignOp::Op(BinOp::Rem),
        P::AndAssign => AssignOp::Op(BinOp::BitAnd),
        P::OrAssign => AssignOp::Op(BinOp::BitOr),
        P::XorAssign => AssignOp::Op(BinOp::BitXor),
        P::ShlAssign => AssignOp::Op(BinOp::Shl),
        P::ShrAssign => AssignOp::Op(BinOp::Shr),
        P::RotlAssign => AssignOp::Op(BinOp::Rotl),
        P::RotrAssign => AssignOp::Op(BinOp::Rotr),
        P::AndAndAssign => AssignOp::Op(BinOp::And),
        P::OrOrAssign => AssignOp::Op(BinOp::Or),
        _ => return None,
    })
}

impl Parser<'_> {
    pub(super) fn parse_block(&mut self) -> PResult<Block> {
        let start = self.expect(P::LBrace, "to start a block")?;
        let stmts = self.parse_stmts_until_close()?;
        let end = self.expect(P::RBrace, "to end the block")?;
        Ok(Block {
            stmts,
            span: start.to(end),
            no_abc: false,
            no_aoc: false,
        })
    }

    /// Statements up to (not including) the closing `}`.
    pub(super) fn parse_stmts_until_close(&mut self) -> PResult<Vec<Stmt>> {
        let saved = std::mem::replace(&mut self.in_list, false);
        let mut stmts = Vec::new();
        while !self.at(P::RBrace) {
            if self.at_eof() {
                return Err(self.expected("`}`", "to close the block"));
            }
            stmts.push(self.parse_stmt()?);
        }
        self.in_list = saved;
        Ok(stmts)
    }

    /// A statement, plus any `@notes` following it (on the same or later lines).
    pub(super) fn parse_stmt(&mut self) -> PResult<Stmt> {
        let mut statement = self.nested(Self::parse_stmt_inner)?;
        let notes = self.parse_notes();
        self.attach_notes(&mut statement, notes);
        Ok(statement)
    }

    fn parse_stmt_inner(&mut self) -> PResult<Stmt> {
        let start = self.span();
        match self.tok() {
            Tok::Punct(P::Semi) => {
                self.bump();
                Ok(stmt(StmtKind::Empty, start))
            }
            Tok::Punct(P::LBrace) => {
                let block = self.parse_block()?;
                let span = block.span;
                Ok(stmt(StmtKind::Block(block), span))
            }
            Tok::Punct(P::Backtick)
                if matches!(self.kw_at(1), Some("return" | "defer" | "push_context")) =>
            {
                self.bump();
                self.parse_keyword_stmt(true)
            }
            Tok::Directive(name) => self.parse_directive_stmt(name.as_str()),
            Tok::Ident(_) => self.parse_keyword_stmt(false),
            _ => self.parse_terminated_simple(),
        }
    }

    fn parse_keyword_stmt(&mut self, backtick: bool) -> PResult<Stmt> {
        match self.kw() {
            Some("if") => self.parse_if(),
            Some("ifx") => self.parse_ifx_stmt(),
            Some("while") => self.parse_while(),
            Some("for") => self.parse_for(),
            Some("break") => self.parse_jump(StmtKind::Break),
            Some("continue") => self.parse_jump(StmtKind::Continue),
            Some("remove") => self.parse_jump(StmtKind::Remove),
            Some("return") => self.parse_return(backtick),
            Some("defer") => self.parse_defer(backtick),
            Some("using") => self.parse_using(),
            Some("push_context") => self.parse_push_context(),
            Some("case") => Err(self.error("`case` outside of a switch")),
            _ => self.parse_terminated_simple(),
        }
    }

    // -- simple statements --------------------------------------------------

    /// Declaration, assignment or expression statement followed by `;`.
    pub(super) fn parse_terminated_simple(&mut self) -> PResult<Stmt> {
        let statement = self.parse_simple_stmt()?;
        self.end_stmt("after statement")?;
        Ok(statement)
    }

    /// Consumes the `;` ending a statement. Statements ending in `}` do not need one.
    pub(super) fn end_stmt(&mut self, context: &str) -> PResult<()> {
        // `x = #code case 1;` before `}`: the case took the `;` (and the statements after it).
        if self.eat(P::Semi) || self.ends_block() || (self.at_prev(P::Semi) && self.at(P::RBrace)) {
            return Ok(());
        }
        Err(self.expected("`;`", context))
    }

    /// True if the previous token closed a brace body or a here-string: no `;` is needed.
    pub(super) fn ends_block(&self) -> bool {
        self.at_prev(P::RBrace) || self.pos == self.block_end || self.prev_is_here_string()
    }

    pub(super) fn attach_notes(&self, statement: &mut Stmt, notes: Vec<crate::ast::Note>) {
        if notes.is_empty() {
            return;
        }
        match &mut statement.kind {
            StmtKind::Decl(decl) | StmtKind::AddContext(decl) => {
                if let Some(decl) = std::rc::Rc::get_mut(decl) {
                    decl.notes.extend(notes);
                    return;
                }
            }
            _ => {}
        }
        statement.notes.extend(notes);
    }

    /// A declaration, assignment or expression, without its terminator.
    pub(super) fn parse_simple_stmt(&mut self) -> PResult<Stmt> {
        if self.tagged_member_ahead() {
            return self.parse_tagged_member();
        }
        if self.decl_modifiers_end(0).is_some() {
            return self.parse_modified_decl();
        }
        if self.decl_ahead(0) {
            return self.parse_decl(false, false);
        }
        let start = self.span();
        let mut lhs = vec![self.parse_assign_target()?];
        while self.eat(P::Comma) {
            lhs.push(self.parse_assign_target()?);
        }
        let op = match self.tok() {
            Tok::Punct(p) => assign_op(*p),
            _ => None,
        };
        let Some(op) = op else {
            return match <[Expr; 1]>::try_from(lhs) {
                Ok([expr]) => {
                    let span = expr.span;
                    Ok(stmt(StmtKind::Expr(expr), span))
                }
                Err(_) => Err(self.expected("an assignment", "after the expression list")),
            };
        };
        self.bump();
        let mut rhs = vec![self.parse_expr()?];
        while self.eat(P::Comma) {
            rhs.push(self.parse_expr()?);
        }
        let span = start.to(self.prev_span());
        Ok(stmt(
            StmtKind::Assign {
                op,
                lhs,
                rhs,
            },
            span,
        ))
    }

    /// One target of an assignment list. `ok=, x.* = f();` marks `ok` as assigned (the form
    /// declarations use for existing names, `ok=, y := f();`); toml-jai writes it here too.
    fn parse_assign_target(&mut self) -> PResult<Expr> {
        if matches!(self.tok(), Tok::Ident(_)) && self.at_n(1, P::Eq) && self.at_n(2, P::Comma) {
            let name = self.ident("as assignment target")?;
            self.bump();
            return Ok(Expr {
                kind: crate::ast::ExprKind::Ident(name.name),
                span: name.span,
            });
        }
        self.parse_expr()
    }

    /// `TAG ,, member: T;` inside a tagged union: an expression (`.A`, `4`, `-12`, `u16`)
    /// followed by two commas before the statement ends.
    fn tagged_member_ahead(&self) -> bool {
        let mut depth = 0usize;
        for n in 0.. {
            match self.tok_at(n) {
                Tok::Eof => return false,
                Tok::Punct(P::LParen | P::LBracket) => depth += 1,
                Tok::Punct(P::RParen | P::RBracket) => match depth.checked_sub(1) {
                    Some(d) => depth = d,
                    None => return false,
                },
                Tok::Punct(P::Semi | P::LBrace | P::RBrace | P::Colon) if depth == 0 => {
                    return false;
                }
                Tok::Punct(P::Comma) if depth == 0 => return n > 0 && self.at_n(n + 1, P::Comma),
                _ => {}
            }
        }
        false
    }

    fn parse_tagged_member(&mut self) -> PResult<Stmt> {
        let tag = self.parse_expr()?;
        self.expect(P::Comma, "after the union tag")?;
        self.expect(P::Comma, "after the union tag")?;
        let mut member = self.parse_simple_stmt()?;
        if let StmtKind::Decl(decl) = &mut member.kind
            && let Some(decl) = std::rc::Rc::get_mut(decl)
        {
            decl.union_tag = Some(tag);
        }
        Ok(member)
    }

    // -- control flow -------------------------------------------------------

    fn parse_if(&mut self) -> PResult<Stmt> {
        let start = self.bump();
        let complete_first = self.at_directive("complete");
        if complete_first {
            self.bump();
        }
        let cond = self.parse_expr()?;
        if self.at(P::EqEq) {
            return self.parse_switch(start, cond, complete_first);
        }
        let mut flags = Vec::new();
        self.skip_check_flags(&mut flags);
        self.eat_kw("then");
        let then_branch = Box::new(no_checks_if(
            self.parse_stmt()?,
            has_flag(&flags, "no_abc"),
            has_flag(&flags, "no_aoc"),
        ));
        let else_branch = if self.eat_kw("else") {
            Some(Box::new(self.parse_stmt()?))
        } else {
            None
        };
        let span = start.to(self.prev_span());
        Ok(stmt(
            StmtKind::If {
                cond,
                then_branch,
                else_branch,
            },
            span,
        ))
    }

    /// `ifx c then a = 1 else a = 2;`: an `ifx` statement whose branches assign is an `if`
    /// (toml-jai). Anything else is parsed as an expression statement.
    fn parse_ifx_stmt(&mut self) -> PResult<Stmt> {
        let (pos, block_end, notes) = (self.pos, self.block_end, self.pending_notes.len());
        if let Ok(Some(statement)) = self.try_ifx_assignments() {
            return Ok(statement);
        }
        self.pos = pos;
        self.block_end = block_end;
        self.pending_notes.truncate(notes);
        self.parse_terminated_simple()
    }

    fn try_ifx_assignments(&mut self) -> PResult<Option<Stmt>> {
        let start = self.bump();
        let cond = self.parse_expr()?;
        if !self.eat_kw("then") {
            return Ok(None);
        }
        let then_branch = self.parse_simple_stmt()?;
        let else_branch = if self.eat_kw("else") {
            Some(self.parse_simple_stmt()?)
        } else {
            None
        };
        let assigns = |s: &Stmt| matches!(s.kind, StmtKind::Assign { .. });
        if !assigns(&then_branch) && !else_branch.as_ref().is_some_and(assigns) {
            return Ok(None);
        }
        self.end_stmt("after statement")?;
        let span = start.to(self.prev_span());
        Ok(Some(stmt(
            StmtKind::If {
                cond,
                then_branch: Box::new(then_branch),
                else_branch: else_branch.map(Box::new),
            },
            span,
        )))
    }

    /// At `==` of `if value == { case ...; }`.
    fn parse_switch(
        &mut self,
        start: crate::source::Span,
        value: Expr,
        complete_first: bool,
    ) -> PResult<Stmt> {
        self.bump();
        let complete = complete_first || self.at_directive("complete");
        if self.at_directive("complete") {
            self.bump();
        }
        let cases = self.parse_cases()?;
        let span = start.to(self.prev_span());
        Ok(stmt(
            StmtKind::Switch {
                value,
                cases,
                complete,
            },
            span,
        ))
    }

    /// `{ case a, b; ...; case; ... }` including both braces.
    pub(super) fn parse_cases(&mut self) -> PResult<Vec<Case>> {
        self.expect(P::LBrace, "to start the switch body")?;
        let mut cases = Vec::new();
        while !self.at(P::RBrace) {
            cases.push(self.parse_case()?);
        }
        self.expect(P::RBrace, "to end the switch body")?;
        Ok(cases)
    }

    pub(super) fn parse_case(&mut self) -> PResult<Case> {
        let start = self.span();
        if !self.eat_kw("case") {
            return Err(self.expected("`case`", "in switch body"));
        }
        let mut values = Vec::new();
        while !matches!(self.tok(), Tok::Punct(P::Semi | P::Colon | P::LBrace)) {
            values.push(self.parse_expr()?);
            if !self.eat(P::Comma) {
                break;
            }
        }
        if !(self.eat(P::Semi) || self.eat(P::Colon) || self.at(P::LBrace)) {
            return Err(self.expected("`;`", "after the case values"));
        }
        let mut body = Vec::new();
        let mut through = false;
        while !self.at(P::RBrace) && !self.at_kw("case") {
            if self.at_eof() {
                return Err(self.expected("`}`", "to end the switch body"));
            }
            if self.at_directive("through") {
                self.bump();
                self.end_stmt("after `#through`")?;
                through = true;
                break;
            }
            body.push(self.parse_stmt()?);
        }
        Ok(Case {
            values,
            body,
            through,
            span: start.to(self.prev_span()),
        })
    }

    fn parse_while(&mut self) -> PResult<Stmt> {
        let start = self.bump();
        let label = if self.at(P::Colon) && matches!(self.tok_at(1), Tok::Ident(_)) {
            self.bump();
            Some(self.ident("as loop name")?)
        } else {
            None
        };
        // `while name := cond` labels the loop for `break name`.
        let tick = usize::from(self.at(P::Backtick));
        let mut bind_label = false;
        let label = if label.is_none()
            && matches!(self.tok_at(tick), Tok::Ident(_))
            && self.at_n(tick + 1, P::ColonEq)
        {
            self.eat(P::Backtick);
            let name = self.ident("as loop name")?;
            self.bump();
            bind_label = true;
            Some(name)
        } else {
            label
        };
        let cond = self.parse_expr()?;
        let mut flags = Vec::new();
        self.skip_check_flags(&mut flags);
        self.eat_kw("then");
        let body = Box::new(no_checks_if(
            self.parse_stmt()?,
            has_flag(&flags, "no_abc"),
            has_flag(&flags, "no_aoc"),
        ));
        let span = start.to(self.prev_span());
        Ok(stmt(
            StmtKind::While {
                label,
                bind_label,
                cond,
                body,
            },
            span,
        ))
    }

    fn parse_for(&mut self) -> PResult<Stmt> {
        let start = self.bump();
        let mut iterator = None;
        let (mut flags, mut reverse, mut by_pointer) = (Vec::new(), false, false);
        let (mut pointer_if, mut reverse_if) = (None, None);
        loop {
            match self.tok() {
                // `for :name` picks a for_expansion; it may follow `<` / `*` (`for < :iter x: c`).
                Tok::Punct(P::Colon)
                    if iterator.is_none() && matches!(self.tok_at(1), Tok::Ident(_)) =>
                {
                    self.bump();
                    iterator = Some(self.ident("as iterator name")?);
                    continue;
                }
                Tok::Directive(name) => {
                    let ident = Ident {
                        name: *name,
                        span: self.span(),
                    };
                    flags.push(ident);
                }
                Tok::Punct(P::Lt) => reverse = true,
                Tok::Punct(P::Star) => by_pointer = true,
                // Conditional forms: `for *=cond <=cond it: items`.
                Tok::Punct(P::MulAssign) => {
                    self.bump();
                    pointer_if = Some(self.parse_unary()?);
                    self.eat(P::Comma);
                    continue;
                }
                Tok::Punct(P::Le) => {
                    self.bump();
                    reverse_if = Some(self.parse_unary()?);
                    self.eat(P::Comma);
                    continue;
                }
                _ => break,
            }
            self.bump();
        }
        let (mut it, mut index) = (None, None);
        let backtick_names = self.at(P::Backtick);
        if self.for_names_ahead() {
            self.eat(P::Backtick);
            it = Some(self.ident("as iterator name")?);
            if self.eat(P::Comma) {
                self.eat(P::Backtick);
                index = Some(self.ident("as index name")?);
            }
            self.bump();
        }
        let first = self.parse_expr()?;
        let over = if self.eat(P::DotDot) {
            ForOver::Range(first, self.parse_expr()?)
        } else {
            ForOver::Collection(first)
        };
        self.skip_check_flags(&mut flags);
        self.eat_kw("then");
        let body = Box::new(self.parse_stmt()?);
        let span = start.to(self.prev_span());
        let node = For {
            it,
            index,
            by_pointer,
            reverse,
            pointer_if,
            reverse_if,
            iterator,
            backtick_names,
            over,
            body,
            flags,
        };
        Ok(stmt(StmtKind::For(Box::new(node)), span))
    }

    /// At `name :` or `name, index :`.
    fn for_names_ahead(&self) -> bool {
        let skip_tick =
            |i: usize| i + usize::from(matches!(self.tok_at(i), Tok::Punct(P::Backtick)));
        let mut i = skip_tick(0);
        if !matches!(self.tok_at(i), Tok::Ident(_)) {
            return false;
        }
        i += 1;
        if matches!(self.tok_at(i), Tok::Punct(P::Comma)) {
            i = skip_tick(i + 1);
            if !matches!(self.tok_at(i), Tok::Ident(_)) {
                return false;
            }
            i += 1;
        }
        matches!(self.tok_at(i), Tok::Punct(P::Colon))
    }

    /// `#no_abc` / `#no_aoc` after a loop or `if` header (array-bounds / arithmetic-overflow checks).
    fn skip_check_flags(&mut self, flags: &mut Vec<Ident>) {
        while matches!(self.directive(), Some("no_abc" | "no_aoc")) {
            if let Tok::Directive(name) = self.tok() {
                flags.push(Ident {
                    name: *name,
                    span: self.span(),
                });
            }
            self.bump();
        }
    }

    /// `break`, `continue` and `remove`, each with an optional label.
    fn parse_jump(&mut self, make: fn(Option<Ident>) -> StmtKind) -> PResult<Stmt> {
        let start = self.bump();
        let label = if matches!(self.tok(), Tok::Ident(_)) {
            Some(self.ident("as label")?)
        } else {
            None
        };
        self.end_stmt("after the statement")?;
        Ok(stmt(make(label), start.to(self.prev_span())))
    }

    fn parse_return(&mut self, backtick: bool) -> PResult<Stmt> {
        let start = self.bump();
        let mut values = Vec::new();
        while !matches!(self.tok(), Tok::Punct(P::Semi | P::RBrace)) {
            values.push(self.parse_arg()?);
            if !self.eat(P::Comma) {
                break;
            }
        }
        self.end_stmt("after `return`")?;
        Ok(stmt(
            StmtKind::Return {
                values,
                backtick,
            },
            start.to(self.prev_span()),
        ))
    }

    fn parse_defer(&mut self, backtick: bool) -> PResult<Stmt> {
        let start = self.bump();
        let body = Box::new(self.parse_stmt()?);
        Ok(stmt(
            StmtKind::Defer {
                body,
                backtick,
            },
            start.to(self.prev_span()),
        ))
    }

    fn parse_push_context(&mut self) -> PResult<Stmt> {
        let start = self.bump();
        if self.at(P::LBrace) {
            // `push_context { ... }` re-pushes the current context.
            let context = super::expr::mk(crate::ast::ExprKind::Context, start);
            let body = Box::new(self.parse_stmt()?);
            return Ok(stmt(
                StmtKind::PushContext {
                    context,
                    body,
                },
                start.to(self.prev_span()),
            ));
        }
        if self.at(P::Comma) && self.kw_at(1) == Some("defer_pop") {
            self.bump();
            self.bump();
            let context = if self.at(P::Semi) {
                None
            } else {
                Some(self.parse_expr()?)
            };
            self.end_stmt("after 'push_context,defer_pop'")?;
            return Ok(stmt(
                StmtKind::PushContextDefer {
                    context,
                },
                start.to(self.prev_span()),
            ));
        }
        let context = self.parse_expr()?;
        let body = Box::new(self.parse_stmt()?);
        Ok(stmt(
            StmtKind::PushContext {
                context,
                body,
            },
            start.to(self.prev_span()),
        ))
    }

    /// `using x;`, `using,only(a, b) x;`, or `using x: T;` (a declaration).
    fn parse_using(&mut self) -> PResult<Stmt> {
        let start = self.span();
        if self.decl_modifiers_end(0).is_some() {
            return self.parse_terminated_simple();
        }
        self.bump();
        let filter = self.parse_using_filter()?;
        // `using,only(a, b) #import "M";` imports just those names.
        if self.at_directive("import") {
            let mut import = self.parse_import(None, start)?;
            self.end_stmt("after `#import`")?;
            if let StmtKind::Import(i) = &mut import.kind
                && let Some(i) = Rc::get_mut(i)
            {
                i.using = Some(filter);
            }
            return Ok(import);
        }
        let value = self.parse_expr()?;
        self.end_stmt("after `using`")?;
        Ok(stmt(
            StmtKind::Using {
                value,
                filter,
            },
            start.to(self.prev_span()),
        ))
    }

    pub(super) fn parse_using_filter(&mut self) -> PResult<UsingFilter> {
        if !(self.at(P::Comma) && matches!(self.kw_at(1), Some("only" | "except" | "map"))) {
            return Ok(UsingFilter::None);
        }
        self.bump();
        let kind = self.ident("as using filter")?;
        // `using,except .["x", "y"] name: T;`
        if !self.at(P::LParen) {
            return Ok(UsingFilter::Computed(self.parse_postfix(true)?));
        }
        self.bump();
        if self.at(P::DotBracket) {
            let list = self.parse_expr()?;
            self.expect(P::RParen, "to end the using filter")?;
            return Ok(UsingFilter::Computed(list));
        }
        let mut names = Vec::new();
        let mut pairs = Vec::new();
        while !self.at(P::RParen) {
            let name = self.ident("in using filter")?;
            if self.eat(P::Eq) {
                pairs.push((name, self.ident("in using filter")?));
            } else {
                names.push(name);
            }
            if !self.eat(P::Comma) {
                break;
            }
        }
        self.expect(P::RParen, "to end the using filter")?;
        Ok(match kind.name.as_str() {
            "only" => UsingFilter::Only(names),
            "except" => UsingFilter::Except(names),
            _ => UsingFilter::Map(pairs),
        })
    }
}

fn has_flag(flags: &[Ident], name: &str) -> bool {
    flags.iter().any(|f| f.name.as_str() == name)
}

/// `body` as a block with bounds checks (`no_abc`) and / or arithmetic overflow checks (`no_aoc`) off
/// (`#no_abc { }`, a flagged loop or `if`).
pub(super) fn no_checks_if(body: Stmt, no_abc: bool, no_aoc: bool) -> Stmt {
    if !no_abc && !no_aoc {
        return body;
    }
    match body.kind {
        StmtKind::Block(mut block) => {
            block.no_abc |= no_abc;
            block.no_aoc |= no_aoc;
            Stmt {
                kind: StmtKind::Block(block),
                ..body
            }
        }
        _ => {
            let span = body.span;
            stmt(
                StmtKind::Block(Block {
                    stmts: vec![body],
                    span,
                    no_abc,
                    no_aoc,
                }),
                span,
            )
        }
    }
}
