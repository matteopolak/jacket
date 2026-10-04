//! Declarations: names, types, values, modifiers (`using`, `#as`) and trailing flags.
use super::stmt::stmt;
use super::{PResult, Parser};
use crate::ast::{AstId, Decl, DeclKind, Foreign, ForeignName, Ident, Stmt, StmtKind, UsingFilter};
use crate::intern::Sym;
use crate::lexer::{P, Tok};
use std::rc::Rc;

/// Directives that may trail a declaration (`x: int #align 16;`).
struct DeclNames {
    names: Vec<Ident>,
    existing: Vec<bool>,
}

const DECL_FLAGS: &[&str] = &[
    "no_reset",
    "elsewhere",
    "deprecated",
    "program_export",
    "type_info_none",
];

impl Parser<'_> {
    // -- declarations -------------------------------------------------------

    /// At offset `n`: ``[`]name {, name} :``, `::` or `:=`, an `operator` declaration, or a
    /// mixed list such as `a=, b := f()` / `a:, b = f()`.
    pub(super) fn decl_ahead(&self, n: usize) -> bool {
        let mut i = n;
        if matches!(self.tok_at(i), Tok::Punct(P::Backtick)) {
            i += 1;
        }
        if self.kw_at(i) == Some("operator") && self.operator_end(i + 1).is_some() {
            return true;
        }
        let mut declared_marker = false;
        loop {
            if !matches!(self.tok_at(i), Tok::Ident(_)) {
                return false;
            }
            i += 1;
            match (self.tok_at(i), self.tok_at(i + 1)) {
                (Tok::Punct(P::Eq), Tok::Punct(P::Comma | P::ColonEq)) => i += 1,
                (Tok::Punct(P::Colon), Tok::Punct(P::Comma)) => {
                    declared_marker = true;
                    i += 1;
                }
                _ => {}
            }
            if !matches!(self.tok_at(i), Tok::Punct(P::Comma)) {
                return match self.tok_at(i) {
                    Tok::Punct(P::Colon | P::ColonColon | P::ColonEq) => true,
                    Tok::Punct(P::Eq) => declared_marker,
                    _ => false,
                };
            }
            i += 1;
        }
    }

    /// Offset after leading `using[,filter]` / `#as` modifiers if a declaration follows them.
    pub(super) fn decl_modifiers_end(&self, n: usize) -> Option<usize> {
        let mut i = n;
        loop {
            if self.kw_at(i) == Some("using") {
                i = self.using_filter_end(i + 1);
            } else if self.directive_at(i) == Some("as") {
                i += 1;
            } else {
                break;
            }
        }
        (i > n && self.decl_ahead(i)).then_some(i)
    }

    /// Offset after an optional `,only(..)` / `,except(..)` / `,map(..)` starting at offset `n`.
    fn using_filter_end(&self, n: usize) -> usize {
        if !(self.at_n(n, P::Comma) && matches!(self.kw_at(n + 1), Some("only" | "except" | "map")))
        {
            return n;
        }
        if self.at_n(n + 2, P::DotBracket) {
            return self.bracket_end(n + 2).unwrap_or(n);
        }
        if !self.at_n(n + 2, P::LParen) {
            return self.computed_filter_end(n + 2).unwrap_or(n);
        }
        match self
            .at_n(n + 2, P::LParen)
            .then(|| self.matching_paren(n + 2))
            .flatten()
        {
            Some(close) => close - self.pos + 1,
            None => n,
        }
    }

    /// For a filter written as a bare expression, the offset where the declaration after it begins.
    fn computed_filter_end(&self, from: usize) -> Option<usize> {
        let mut depth = 0usize;
        for i in from.. {
            match self.tok_at(i) {
                Tok::Punct(P::LParen | P::LBracket | P::LBrace | P::DotBracket | P::DotBrace) => {
                    depth += 1
                }
                Tok::Punct(P::RParen | P::RBracket | P::RBrace) => depth = depth.checked_sub(1)?,
                Tok::Punct(P::Semi) | Tok::Eof => return None,
                _ if depth == 0 && i > from && self.decl_ahead(i) => return Some(i),
                _ => {}
            }
        }
        None
    }

    /// `using,except(x) #as name: T`: consumes the modifiers, then the declaration.
    pub(super) fn parse_modified_decl(&mut self) -> PResult<Stmt> {
        let (mut using, mut as_, mut filter) = (false, false, UsingFilter::None);
        loop {
            if self.at_kw("using") {
                self.bump();
                using = true;
                filter = self.parse_using_filter()?;
            } else if self.at_directive("as") {
                self.bump();
                as_ = true;
            } else {
                break;
            }
        }
        let mut declaration = self.parse_decl(using, as_)?;
        match &mut declaration.kind {
            StmtKind::Decl(decl) if !matches!(filter, UsingFilter::None) => {
                if let Some(decl) = Rc::get_mut(decl) {
                    decl.using_filter = Some(filter);
                }
            }
            // `using X :: #import "M"` (with or without a filter) imports the names too.
            StmtKind::Import(import) if using => {
                if let Some(import) = Rc::get_mut(import) {
                    import.using = Some(filter);
                }
            }
            _ => {}
        }
        Ok(declaration)
    }

    /// For `operator <punct>... ::` starting at the first punctuation token, the offset of `::`.
    fn operator_end(&self, from: usize) -> Option<usize> {
        let mut i = from;
        while i < from + 4 {
            match self.tok_at(i) {
                Tok::Punct(P::ColonColon) if i > from => return Some(i),
                Tok::Punct(P::Colon | P::ColonEq | P::ColonColon | P::Comma | P::Semi | P::Dot) => {
                    return None;
                }
                Tok::Punct(_) => i += 1,
                _ => return None,
            }
        }
        None
    }

    /// Declaration without its terminator. The cursor is at the first name (or backtick).
    pub(super) fn parse_decl(&mut self, using: bool, as_: bool) -> PResult<Stmt> {
        let start = self.span();
        let backtick = self.eat(P::Backtick);
        let DeclNames {
            names,
            existing,
        } = self.parse_decl_names()?;
        let mut decl = Decl {
            id: AstId::fresh(),
            names,
            kind: DeclKind::Var,
            ty: None,
            value: None,
            extra_values: Vec::new(),
            existing,
            foreign: None,
            union_tag: None,
            using_filter: None,
            using,
            as_,
            backtick,
            align: None,
            flags: Vec::new(),
            notes: Vec::new(),
            span: start,
        };
        match self.tok() {
            Tok::Punct(P::ColonColon) => {
                self.bump();
                decl.kind = DeclKind::Const;
                if self.at_directive("import") {
                    return self.parse_import(decl.names.first().copied(), start);
                }
                self.parse_decl_values(&mut decl)?;
            }
            Tok::Punct(P::ColonEq) => {
                self.bump();
                self.parse_decl_values(&mut decl)?;
            }
            Tok::Punct(P::Eq) if !decl.existing.is_empty() => {
                self.bump();
                self.parse_decl_values(&mut decl)?;
            }
            _ => self.parse_typed_decl_rest(&mut decl)?,
        }
        self.parse_decl_suffix(&mut decl)?;
        decl.span = start.to(self.prev_span());
        let span = decl.span;
        Ok(stmt(StmtKind::Decl(Rc::new(decl)), span))
    }

    /// Declared names, with the assigned-to-existing mask for mixed lists.
    fn parse_decl_names(&mut self) -> PResult<DeclNames> {
        if self.at_kw("operator") && self.operator_end(1).is_some() {
            let start = self.bump();
            let mut text = String::new();
            while !self.at(P::ColonColon) {
                if let Tok::Punct(p) = self.tok() {
                    text.push_str(p.text());
                }
                self.bump();
            }
            let name = Ident {
                name: Sym::intern(&format!("operator{text}")),
                span: start.to(self.prev_span()),
            };
            self.pending_operator = Some(text.into());
            return Ok(DeclNames {
                names: vec![name],
                existing: Vec::new(),
            });
        }
        let (mut names, mut existing) = (Vec::new(), Vec::new());
        let (mut any_assigned, mut any_declared_marker) = (false, false);
        loop {
            names.push(self.ident("as declaration name")?);
            let assigned = matches!(self.tok(), Tok::Punct(P::Eq))
                && matches!(self.tok_at(1), Tok::Punct(P::Comma | P::ColonEq));
            let declared = matches!(self.tok(), Tok::Punct(P::Colon))
                && matches!(self.tok_at(1), Tok::Punct(P::Comma));
            if assigned || declared {
                self.bump();
            }
            any_assigned |= assigned;
            any_declared_marker |= declared;
            // In `a:, b = f()` the unmarked names are the existing variables.
            existing.push((assigned, declared));
            if !self.eat(P::Comma) {
                break;
            }
        }
        let existing = if any_declared_marker {
            existing
                .into_iter()
                .map(|(_, declared)| !declared)
                .collect()
        } else if any_assigned {
            existing.into_iter().map(|(assigned, _)| assigned).collect()
        } else {
            Vec::new()
        };
        Ok(DeclNames {
            names,
            existing,
        })
    }

    /// After the names: `: T`, `: T = v`, `: T : v`, `: = v`.
    fn parse_typed_decl_rest(&mut self, decl: &mut Decl) -> PResult<()> {
        self.expect(P::Colon, "after the declaration name")?;
        if !matches!(self.tok(), Tok::Punct(P::Eq | P::Colon)) {
            decl.ty = Some(self.parse_param_type()?);
        }
        self.parse_decl_suffix(decl)?;
        if self.eat(P::Eq) {
            self.parse_decl_values(decl)?;
        } else if self.eat(P::Colon) {
            decl.kind = DeclKind::Const;
            self.parse_decl_values(decl)?;
        }
        Ok(())
    }

    /// One value, or several for `a, b := 1, 2;`.
    fn parse_decl_values(&mut self, decl: &mut Decl) -> PResult<()> {
        decl.value = Some(self.parse_expr()?);
        while decl.names.len() > 1 && self.eat(P::Comma) {
            decl.extra_values.push(self.parse_expr()?);
        }
        Ok(())
    }

    /// Trailing `#align N`, flags such as `#no_reset`, and `@notes`.
    fn parse_decl_suffix(&mut self, decl: &mut Decl) -> PResult<()> {
        loop {
            match self.tok() {
                Tok::Note(_) => decl.notes.extend(self.parse_notes()),
                // A directive on the next line starts the next statement.
                Tok::Directive(_) if self.newline_before() => return Ok(()),
                Tok::Directive(name) if name.as_str() == "align" => {
                    self.bump();
                    decl.align = Some(self.parse_unary()?);
                }
                Tok::Directive(name) if name.as_str() == "elsewhere" => {
                    decl.flags.push(Ident {
                        name: *name,
                        span: self.span(),
                    });
                    self.bump();
                    decl.foreign = Some(self.parse_elsewhere_library()?.unwrap_or(Foreign {
                        library: None,
                        name: ForeignName::Default,
                    }));
                }
                Tok::Directive(name) if DECL_FLAGS.contains(&name.as_str()) => {
                    decl.flags.push(Ident {
                        name: *name,
                        span: self.span(),
                    });
                    self.bump();
                }
                _ => return Ok(()),
            }
        }
    }

    /// Optional `lib ["symbol"]` after `#elsewhere`.
    pub(super) fn parse_elsewhere_library(&mut self) -> PResult<Option<Foreign>> {
        if !matches!(self.tok(), Tok::Ident(_)) || self.newline_before() {
            return Ok(None);
        }
        let library = Some(self.ident("after '#elsewhere'")?);
        let name = self.parse_foreign_symbol();
        Ok(Some(Foreign {
            library,
            name,
        }))
    }

    /// Optional string literal naming the foreign symbol.
    pub(super) fn parse_foreign_symbol(&mut self) -> ForeignName {
        match self.tok() {
            Tok::Str(s) => {
                let name = String::from_utf8_lossy(s).into_owned();
                self.bump();
                ForeignName::Named(name.into())
            }
            _ => ForeignName::Default,
        }
    }
}
