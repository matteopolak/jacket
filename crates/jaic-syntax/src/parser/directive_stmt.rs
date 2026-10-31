//! Directive statements: `#if`, `#import`, `#load`, `#run`, `#assert`, scope markers, ...
use super::stmt::stmt;
use super::{PResult, Parser};
use crate::ast::{
    AssertStmt, BinOp, CheckBits, Decl, DirectiveStmt, Expr, ExprKind, Ident, Import, ImportSource,
    InsertStmt, ModuleParametersStmt, ScopeKind, StaticIfStmt, Stmt, StmtKind, SwitchStmt, UnOp,
};
use crate::diagnostics::Code;
use crate::intern::Sym;
use crate::lexer::{P, Tok};
use crate::source::Span;
use std::rc::Rc;

/// Directive statements that take a plain list of expressions up to the `;`.
pub const GENERIC_DIRECTIVES: &[&str] = &[
    "library",
    "system_library",
    "foreign_library",
    "foreign_system_library",
    "program_export",
    "poke_name",
];

/// `#program_export "name" f :: () {}` sets the export flag on the procedure itself.
fn mark_program_export(decl: &mut Decl, name: Option<Rc<[u8]>>) {
    if let Some(Expr {
        kind: ExprKind::Proc(lit),
        ..
    }) = &mut decl.value
        && let Some(lit) = Rc::get_mut(lit)
        && let Some(header) = Rc::get_mut(&mut lit.header)
    {
        header.flags.program_export = Some(name);
    }
}

impl Parser<'_> {
    // -- directive statements -------------------------------------------------

    pub(super) fn parse_directive_stmt(&mut self, name: &str) -> PResult<Stmt> {
        let start = self.span();
        match name {
            "if" => self.parse_static_if(),
            "import" => {
                let import = self.parse_import(None, start)?;
                self.end_stmt("after `#import`")?;
                Ok(import)
            }
            "load" => self.parse_load(),
            "scope_file" => self.parse_scope(ScopeKind::File),
            "scope_export" => self.parse_scope(ScopeKind::Export),
            "scope_module" => self.parse_scope(ScopeKind::Module),
            "run" => self.parse_run_stmt(),
            "insert" => self.parse_insert_stmt(),
            "assert" => self.parse_assert(),
            "add_context" => self.parse_add_context(),
            "module_parameters" => self.parse_module_parameters(),
            "placeholder" => self.parse_placeholder(),
            "place" => self.parse_place(),
            "overlay" => self.parse_overlay(),
            "through" => {
                self.bump();
                self.end_stmt("after `#through`")?;
                Ok(stmt(StmtKind::Through, start))
            }
            "as" => self.parse_terminated_simple(),
            "no_abc" | "no_aoc" if self.at_n(1, P::LBrace) => {
                // `#no_abc { ... }` / `#no_aoc { ... }` turn bounds / overflow checks off in the block.
                self.bump();
                let body = self.parse_stmt()?;
                let checks_off = if name == "no_abc" {
                    CheckBits::NO_ABC
                } else {
                    CheckBits::NO_AOC
                };
                Ok(super::stmt::no_checks_if(body, checks_off))
            }
            "no_reset" | "program_export"
                if self.decl_ahead(1)
                    || (matches!(self.tok_at(1), Tok::Str(_)) && self.decl_ahead(2)) =>
            {
                self.parse_flagged_decl(name)
            }
            name if GENERIC_DIRECTIVES.contains(&name) => self.parse_generic_directive(),
            _ => self.parse_terminated_simple(),
        }
    }

    fn parse_static_if(&mut self) -> PResult<Stmt> {
        let start = self.bump();
        // `#if #complete x == { ... }`: only one case is ever compiled, so the completeness
        // promise is accepted without a check.
        if self.at_directive("complete") {
            self.bump();
        }
        let value = self.parse_expr()?;
        if self.at(P::EqEq) {
            self.bump();
            let header = start.to(self.span());
            let cases = self.parse_cases()?;
            return Ok(stmt(
                StmtKind::StaticSwitch(Box::new(SwitchStmt {
                    value,
                    cases,
                    complete: false,
                    header,
                })),
                start.to(self.prev_span()),
            ));
        }
        self.eat_kw("then");
        let file_level = self.platform.is_some() && self.depth == self.file_level_depth;
        let known = if file_level {
            self.fold_platform_test(&value)
        } else {
            None
        };
        let saved_level = self.file_level_depth;
        if file_level {
            self.file_level_depth += 1;
        }
        let then_branch = if known == Some(false) {
            self.skip_static_branch()?
        } else {
            self.parse_static_branch()?
        };
        let then_arm = start.to(self.prev_span());
        let else_start = self.span();
        let mut else_arm = None;
        let else_branch = if self.eat_kw("else") {
            let chained = self.at_directive("if");
            let mut branch = if known == Some(true) {
                self.skip_static_branch()?
            } else {
                self.parse_static_branch()?
            };
            // `else #if`: the `else` belongs with the inner `#if`'s first arm.
            if chained
                && let [inner] = branch.as_mut_slice()
                && let StmtKind::StaticIf(inner) = &mut inner.kind
            {
                inner.then_arm = else_start.to(inner.then_arm);
            }
            else_arm = Some(else_start.to(self.prev_span()));
            branch
        } else {
            Vec::new()
        };
        self.file_level_depth = saved_level;
        let value = match known {
            Some(decided) => Expr {
                kind: ExprKind::Bool(decided),
                span: value.span,
                parens: None,
            },
            None => value,
        };
        Ok(stmt(
            StmtKind::StaticIf(Box::new(StaticIfStmt {
                cond: value,
                then_branch,
                else_branch,
                then_arm,
                else_arm,
            })),
            start.to(self.prev_span()),
        ))
    }

    /// Whether `expr` is decided by the platform alone: `OS == .X`, `CPU != .X`, joined by `!`,
    /// `&&` and `||`. `None` for anything else, including a tag the prelude does not name,
    /// which type checking reports.
    fn fold_platform_test(&self, expr: &Expr) -> Option<bool> {
        let platform = self.platform?;
        match &expr.kind {
            ExprKind::Bool(value) => Some(*value),
            ExprKind::Unary(UnOp::Not, inner) => self.fold_platform_test(inner).map(|v| !v),
            ExprKind::Binary(BinOp::And, lhs, rhs) => {
                Some(self.fold_platform_test(lhs)? && self.fold_platform_test(rhs)?)
            }
            ExprKind::Binary(BinOp::Or, lhs, rhs) => {
                Some(self.fold_platform_test(lhs)? || self.fold_platform_test(rhs)?)
            }
            ExprKind::Binary(op @ (BinOp::Eq | BinOp::Ne), lhs, rhs) => {
                let (ExprKind::Ident(name), ExprKind::InferredMember(tag)) = (&lhs.kind, &rhs.kind)
                else {
                    return None;
                };
                platform.compare(name.as_str(), tag.name.as_str(), *op == BinOp::Eq)
            }
            _ => None,
        }
    }

    /// Steps over a `{ ... }` branch without building statements. A branch that is not a
    /// brace list is parsed (and kept) like any statement.
    fn skip_static_branch(&mut self) -> PResult<Vec<Stmt>> {
        if !self.at(P::LBrace) {
            return self.parse_static_branch();
        }
        let mut open = 0usize;
        loop {
            match self.tok() {
                Tok::Punct(P::LBrace | P::DotBrace) => open += 1,
                Tok::Punct(P::RBrace) => open -= 1,
                Tok::Eof => return Err(self.expected("`}`", "to end the `#if` body")),
                _ => {}
            }
            self.bump();
            if open == 0 {
                return Ok(Vec::new());
            }
        }
    }

    /// A `{ ... }` list (no new scope) or a single statement.
    fn parse_static_branch(&mut self) -> PResult<Vec<Stmt>> {
        if !self.at(P::LBrace) {
            let first = self.parse_stmt()?;
            // `#overlay(a)` only names the member that follows it, so that member belongs to
            // the branch too.
            if matches!(first.kind, StmtKind::Overlay(_)) && !self.at(P::RBrace) {
                let member = self.parse_stmt()?;
                return Ok(vec![first, member]);
            }
            return Ok(vec![first]);
        }
        let open = self.bump();
        let stmts = self.parse_stmts_until_close()?;
        self.ide_close_block(open, "to end the `#if` body")?;
        Ok(stmts)
    }

    /// `#import "Module"`, `#import,file "x.jai"`, with optional `(PARAM = value)`.
    /// The terminating `;` is left to the caller.
    pub(super) fn parse_import(&mut self, name: Option<Ident>, start: Span) -> PResult<Stmt> {
        self.bump();
        let mut flags = Vec::new();
        let mut kind = None;
        for flag in self.parse_directive_flags()? {
            match flag.name.name.as_str() {
                "file" | "dir" | "string" => kind = Some(flag.name.name.as_str()),
                _ => flags.push(flag.name),
            }
        }
        let (text, _) = self.string_lit("after `#import`")?;
        let text: Rc<str> = String::from_utf8_lossy(&text).into();
        let source = match kind {
            Some("file") => ImportSource::File(text),
            Some("dir") => ImportSource::Dir(text),
            Some("string") => ImportSource::String(text),
            _ => ImportSource::Module(text),
        };
        let mut params = Vec::new();
        let mut program_params_start = None;
        let mut groups = 0;
        while self.eat(P::LParen) {
            if groups == 1 {
                program_params_start = Some(params.len());
            }
            params.extend(self.parse_args(P::RParen, "in import parameters")?);
            groups += 1;
        }
        let span = start.to(self.prev_span());
        Ok(stmt(
            StmtKind::Import(Rc::new(Import {
                source,
                params,
                module_params_present: groups != 0,
                program_params_start,
                name,
                flags,
                using: None,
                span,
            })),
            span,
        ))
    }

    fn parse_load(&mut self) -> PResult<Stmt> {
        let start = self.bump();
        self.parse_directive_flags()?;
        let (path, span) = self.string_lit("after `#load`")?;
        self.end_stmt("after `#load`")?;
        let path: Rc<str> = String::from_utf8_lossy(&path).into();
        Ok(stmt(
            StmtKind::Load {
                path,
                span,
            },
            start.to(span),
        ))
    }

    fn parse_scope(&mut self, kind: ScopeKind) -> PResult<Stmt> {
        let span = self.bump();
        self.eat(P::Semi);
        Ok(stmt(StmtKind::Scope(kind), span))
    }

    fn parse_run_stmt(&mut self) -> PResult<Stmt> {
        let start = self.span();
        let run = self.parse_run_with(true)?;
        self.end_stmt("after `#run`")?;
        Ok(stmt(
            StmtKind::Run(Box::new(run)),
            start.to(self.prev_span()),
        ))
    }

    fn parse_insert_stmt(&mut self) -> PResult<Stmt> {
        let start = self.bump();
        let (flags, scope) = Self::split_insert_flags(self.parse_directive_flags()?);
        let replacements = self.parse_insert_replacements()?;
        let value = self.parse_expr()?;
        self.end_stmt("after `#insert`")?;
        Ok(stmt(
            StmtKind::Insert(Box::new(InsertStmt {
                value,
                flags,
                scope,
                replacements,
            })),
            start.to(self.prev_span()),
        ))
    }

    fn parse_assert(&mut self) -> PResult<Stmt> {
        let assert = self.parse_assert_core()?;
        // A condition that holds a braced body (`#assert #run -> bool { ... } "message"`) needs
        // no `;` after it.
        if std::mem::take(&mut self.assert_cond_has_body) {
            self.eat(P::Semi);
        } else {
            self.end_stmt("after `#assert`")?;
        }
        Ok(assert)
    }

    /// `#assert cond ["message"]` without the terminator.
    pub(super) fn parse_assert_core(&mut self) -> PResult<Stmt> {
        let start = self.bump();
        self.assert_cond_has_body = false;
        // `#assert,stallable(cond)`: checks run synchronously, so the flag changes nothing. Its
        // name is taken alone: the parentheses after it are the condition, not a flag argument.
        if self.at(P::Comma)
            && self.span().start == self.prev_span().end
            && self.kw_at(1) == Some("stallable")
            && self.token_at(1).span.start == self.span().end
        {
            self.bump();
            self.bump();
        }
        // `#assert(cond, "message")`: the call-like form, told apart by the comma inside the
        // parentheses. Deciding up front (rather than trying it and rewinding) keeps nested
        // `#assert(` from being parsed twice per level, which is exponential.
        if self.at(P::LParen)
            && self
                .matching_paren(0)
                .is_some_and(|close| self.has_top_level_comma(self.pos + 1, close))
        {
            self.bump();
            let cond = self.parse_expr()?;
            // A nested `#assert` in the condition may have taken the comma and message itself.
            let message = if self.eat(P::Comma) {
                Some(self.parse_expr()?)
            } else {
                None
            };
            // Arguments for the message's `%` slots, as in print.
            let mut args = Vec::new();
            while message.is_some() && self.eat(P::Comma) && !self.at(P::RParen) {
                args.push(self.parse_expr()?);
            }
            self.expect(P::RParen, "after `#assert` message")?;
            return Ok(stmt(
                StmtKind::Assert(Box::new(AssertStmt {
                    cond,
                    message,
                    args,
                })),
                start.to(self.prev_span()),
            ));
        }
        let cond_start = self.pos;
        let cond = self.parse_expr()?;
        self.assert_cond_has_body = self.toks[cond_start..self.pos]
            .iter()
            .any(|token| matches!(token.tok, Tok::Punct(P::RBrace)));
        if self.at(P::Comma) && matches!(self.tok_at(1), Tok::Str(_)) {
            self.bump();
        }
        let message = if matches!(self.tok(), Tok::Str(_)) {
            Some(self.parse_expr()?)
        } else {
            None
        };
        Ok(stmt(
            StmtKind::Assert(Box::new(AssertStmt {
                cond,
                message,
                args: Vec::new(),
            })),
            start.to(self.prev_span()),
        ))
    }

    fn parse_add_context(&mut self) -> PResult<Stmt> {
        let add_context = self.parse_add_context_core()?;
        self.end_stmt("after `#add_context`")?;
        Ok(add_context)
    }

    /// `#add_context [#as] [using] name: T [= v]` without the terminator.
    pub(super) fn parse_add_context_core(&mut self) -> PResult<Stmt> {
        let start = self.bump();
        let declaration = self.parse_simple_stmt()?;
        match declaration.kind {
            StmtKind::Decl(decl) => {
                Ok(stmt(StmtKind::AddContext(decl), start.to(self.prev_span())))
            }
            _ => Err(crate::diagnostics::Diagnostic::new(
                Code::ParseExpectedDeclaration,
                start,
                "expected a declaration after `#add_context`",
            )),
        }
    }

    /// `#module_parameters (A := 1) (B := 2);` or with a trailing block.
    fn parse_module_parameters(&mut self) -> PResult<Stmt> {
        let start = self.bump();
        let mut groups = Vec::new();
        while self.eat(P::LParen) {
            groups.push(self.parse_params(P::RParen)?);
        }
        let mut groups = groups.into_iter();
        let params = groups.next().unwrap_or_default();
        let runtime_params = groups.next().unwrap_or_default();
        let body = if self.at(P::LBrace) {
            Some(self.parse_block()?)
        } else {
            self.end_stmt("after '#module_parameters'")?;
            None
        };
        Ok(stmt(
            StmtKind::ModuleParameters(Box::new(ModuleParametersStmt {
                params,
                runtime_params,
                body,
            })),
            start.to(self.prev_span()),
        ))
    }

    fn parse_placeholder(&mut self) -> PResult<Stmt> {
        let start = self.bump();
        let mut names = vec![self.ident("after `#placeholder`")?];
        while self.eat(P::Comma) {
            names.push(self.ident("after `,`")?);
        }
        self.end_stmt("after `#placeholder`")?;
        Ok(stmt(
            StmtKind::Placeholder(names),
            start.to(self.prev_span()),
        ))
    }

    fn parse_place(&mut self) -> PResult<Stmt> {
        let start = self.bump();
        self.note_lenient(Code::PlaceDirective, start);
        let target = self.parse_expr()?;
        self.end_stmt("after `#place`")?;
        Ok(stmt(
            StmtKind::Place(Box::new(target)),
            start.to(self.prev_span()),
        ))
    }

    fn parse_overlay(&mut self) -> PResult<Stmt> {
        let start = self.bump();
        self.expect(P::LParen, "after `#overlay`")?;
        let target = self.parse_expr()?;
        let end = self.expect(P::RParen, "after the `#overlay` target")?;
        Ok(stmt(StmtKind::Overlay(Box::new(target)), start.to(end)))
    }

    /// `#no_reset x := 0;` / `#program_export f :: ...`: a flag in front of a declaration.
    fn parse_flagged_decl(&mut self, name: &str) -> PResult<Stmt> {
        let flag = Ident {
            name: Sym::intern(name),
            span: self.bump(),
        };
        let export_name = match self.tok() {
            Tok::Str(s) if name == "program_export" => {
                let s = s.clone();
                self.bump();
                Some(s)
            }
            _ => None,
        };
        let mut declaration = self.parse_terminated_simple()?;
        if let StmtKind::Decl(decl) = &mut declaration.kind
            && let Some(decl) = Rc::get_mut(decl)
        {
            decl.rare_mut().flags.push(flag);
            if name == "program_export" {
                mark_program_export(decl, export_name);
            }
        }
        Ok(declaration)
    }

    /// `#library "x";`, `#poke_name Module name;` and similar statements.
    fn parse_generic_directive(&mut self) -> PResult<Stmt> {
        let start = self.span();
        let Tok::Directive(sym) = self.tok().clone() else {
            return Err(self.expected("directive", ""));
        };
        self.bump();
        match sym.as_str() {
            "foreign_library" | "foreign_system_library" => {
                self.note_lenient(Code::ForeignLibrary, start);
            }
            "library" | "system_library" => self.note_lenient(Code::UnnamedLibrary, start),
            _ => {}
        }
        let name = Ident {
            name: sym,
            span: start,
        };
        let flags = self
            .parse_directive_flags()?
            .into_iter()
            .map(|flag| flag.name)
            .collect();
        let mut args = Vec::new();
        while !matches!(self.tok(), Tok::Punct(P::Semi) | Tok::Eof) && !self.at_prev(P::RBrace) {
            if let Some(operator) = self.try_operator_name() {
                args.push(operator);
            } else {
                args.push(self.parse_postfix(true)?);
            }
            self.eat(P::Comma);
        }
        self.end_stmt("after the directive")?;
        Ok(stmt(
            StmtKind::Directive(Box::new(DirectiveStmt {
                kind: crate::ast::DirectiveKind::of(sym.as_str()),
                name,
                flags,
                args,
            })),
            start.to(self.prev_span()),
        ))
    }

    /// `operator==` style names as a single identifier expression.
    fn try_operator_name(&mut self) -> Option<crate::ast::Expr> {
        if !(self.at_kw("operator")
            && matches!(self.tok_at(1), Tok::Punct(p) if !matches!(p, P::Semi | P::Comma)))
        {
            return None;
        }
        let start = self.bump();
        let mut text = String::from("operator");
        while let Tok::Punct(p) = self.tok() {
            if matches!(p, P::Semi | P::Comma) {
                break;
            }
            text.push_str(p.text());
            self.bump();
        }
        let span = start.to(self.prev_span());
        Some(super::expr::mk(
            crate::ast::ExprKind::Ident(Sym::intern(&text)),
            span,
        ))
    }
}
