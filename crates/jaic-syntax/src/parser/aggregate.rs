//! `struct`, `union` and `enum` literals.
use super::expr::mk;
use super::{PResult, Parser};
use crate::ast::{
    AstId, EnumItem, EnumLit, EnumMember, Expr, ExprKind, StructBits, StructFlags, StructKind,
    StructLit, UsingStmt,
};
use crate::lexer::{P, Tok};
use std::rc::Rc;

impl Parser<'_> {
    /// `struct (T: Type) #flags { ... }` / `union { ... }`.
    pub(super) fn parse_struct_expr(&mut self) -> PResult<Expr> {
        let start = self.span();
        let kind = if self.at_kw("union") {
            StructKind::Union
        } else {
            StructKind::Struct
        };
        self.bump();
        let params = if self.eat(P::LParen) {
            self.parse_params(P::RParen)?
        } else {
            Vec::new()
        };
        let mut notes = self.parse_notes();
        let tag = self.parse_union_tag()?;
        notes.append(&mut self.pending_notes);
        let mut flags = StructFlags::default();
        let mut modify = self.parse_struct_flags(&mut flags)?;
        self.expect(P::LBrace, "to start the body")?;
        let outer = std::mem::replace(&mut self.in_aggregate_body, true);
        let body = self.parse_stmts_until_close();
        self.in_aggregate_body = outer;
        let body = body?;
        let end = self.ide_close_block(start, "to end the body")?;
        modify = self.parse_struct_flags(&mut flags)?.or(modify);
        self.block_end = self.pos;
        let lit = StructLit {
            id: AstId::fresh(),
            kind,
            tag,
            params,
            body,
            flags,
            modify,
            notes,
            span: start.to(end),
        };
        Ok(mk(ExprKind::Struct(Rc::new(lit)), start.to(end)))
    }

    /// `union tag : Type {`, `union tag : Type = default {` or `union tag := default {`: the tag
    /// member of a tagged union.
    fn parse_union_tag(&mut self) -> PResult<Option<crate::ast::UnionTag>> {
        if !(matches!(self.tok(), Tok::Ident(_))
            && (self.at_n(1, P::Colon) || self.at_n(1, P::ColonEq)))
        {
            return Ok(None);
        }
        let name = self.ident("as union tag name")?;
        let ty = if self.eat(P::ColonEq) {
            None
        } else {
            self.bump();
            Some(Box::new(self.parse_expr()?))
        };
        let value = if ty.is_none() || self.eat(P::Eq) {
            Some(Box::new(self.parse_expr()?))
        } else {
            None
        };
        // Notes may follow the tag type: `union kind : Kind @Serialize(1) {`.
        let notes = self.parse_notes();
        self.pending_notes.extend(notes);
        Ok(Some(crate::ast::UnionTag {
            name,
            ty,
            value,
        }))
    }

    /// Struct directives, accepted before the body and after its closing brace.
    /// Returns the `#modify` block if there is one.
    fn parse_struct_flags(
        &mut self,
        flags: &mut StructFlags,
    ) -> PResult<Option<crate::ast::Block>> {
        let mut modify = None;
        while let Some(name) = self.directive() {
            if let Some(bit) = StructBits::from_directive(name) {
                flags.set(bit);
                self.bump();
                continue;
            }
            match name {
                "align" => {
                    let start = self.bump();
                    flags.align = Some(self.parse_unary()?);
                    self.note_lenient(
                        crate::diagnostics::Code::StructAlign,
                        start.to(self.prev_span()),
                    );
                }
                "modify" => {
                    self.bump();
                    modify = Some(self.parse_modify_body()?);
                }
                _ => break,
            }
        }
        Ok(modify)
    }

    /// `enum u8 #specified { A; B :: 4; }` / `enum_flags { ... }`.
    pub(super) fn parse_enum_expr(&mut self) -> PResult<Expr> {
        let start = self.span();
        let flags_enum = self.at_kw("enum_flags");
        self.bump();
        let base = if matches!(
            self.tok(),
            Tok::Punct(P::LBrace) | Tok::Directive(_) | Tok::Note(_)
        ) {
            None
        } else {
            Some(self.parse_expr()?)
        };
        // Directives and notes before the body, in any order: `enum u8 #specified @Wire {`.
        let (mut specified, mut complete) = (false, false);
        let mut notes = Vec::new();
        loop {
            notes.append(&mut self.parse_notes());
            match self.directive() {
                Some("specified") => specified = true,
                Some("complete") => complete = true,
                _ => break,
            }
            self.bump();
        }
        self.expect(P::LBrace, "to start the enum body")?;
        let items = self.parse_enum_items()?;
        let end = self.ide_close_block(start, "to end the enum body")?;
        let lit = EnumLit {
            id: AstId::fresh(),
            flags_enum,
            base,
            items,
            specified,
            complete,
            notes,
            span: start.to(end),
        };
        Ok(mk(ExprKind::Enum(Rc::new(lit)), start.to(end)))
    }

    /// Enum items up to (not including) the closing brace.
    fn parse_enum_items(&mut self) -> PResult<Vec<EnumItem>> {
        let open = self.prev_span();
        let mut items = Vec::new();
        while !self.at(P::RBrace) {
            if self.recovery.is_some() && (self.at_eof() || self.ide_block_boundary(open)) {
                break;
            }
            if self.eat(P::Semi) {
                continue;
            }
            let start = self.pos;
            let item = if self.at_directive("insert") {
                self.bump();
                self.parse_expr().map(EnumItem::Insert)
            } else if self.at_kw("using") {
                self.bump();
                let item = self.parse_using_filter().and_then(|filter| {
                    let value = self.parse_expr()?;
                    Ok(EnumItem::Using(Box::new(UsingStmt {
                        value,
                        filter,
                    })))
                });
                self.eat(P::Semi);
                item
            } else if self.at_directive("if") {
                self.parse_enum_if()
            } else {
                self.parse_enum_member().map(EnumItem::Member)
            };
            match item {
                Ok(item) => items.push(item),
                Err(error) => {
                    let Some(errors) = self.recovery.as_mut() else {
                        return Err(error);
                    };
                    errors.push(crate::diagnostics::Diagnostic::new(
                        error.code,
                        self.toks[start].span,
                        error.message,
                    ));
                    self.ide_synchronize(start, true);
                }
            }
        }
        Ok(items)
    }

    fn parse_enum_member(&mut self) -> PResult<EnumMember> {
        let name = self.ident("as enum member name")?;
        // `A :: 1`, `A = 1`, and `A : : 1` (the constant form with the empty type spelled out).
        let spaced = self.at(P::Colon) && self.at_n(1, P::Colon);
        if spaced {
            self.bump();
        }
        let value = if self.eat(P::ColonColon) || self.eat(P::Eq) || (spaced && self.eat(P::Colon))
        {
            Some(self.parse_expr()?)
        } else {
            None
        };
        let mut notes = self.parse_notes();
        if !(self.eat(P::Semi) || self.eat(P::Comma) || self.at(P::RBrace)) {
            return Err(self.expected("`;`", "after the enum member"));
        }
        notes.extend(self.parse_notes());
        Ok(EnumMember {
            name,
            value,
            notes,
        })
    }

    fn parse_enum_if(&mut self) -> PResult<EnumItem> {
        self.bump();
        let cond = self.parse_expr()?;
        self.eat_kw("then");
        let then_items = self.parse_enum_branch()?;
        let mut else_items = Vec::new();
        if self.eat_kw("else") {
            else_items = if self.at_directive("if") {
                vec![self.parse_enum_if()?]
            } else {
                self.parse_enum_branch()?
            };
        }
        Ok(EnumItem::If {
            cond,
            then_items,
            else_items,
        })
    }

    fn parse_enum_branch(&mut self) -> PResult<Vec<EnumItem>> {
        let open = self.expect(P::LBrace, "to start the `#if` body")?;
        let items = self.parse_enum_items()?;
        self.ide_close_block(open, "to end the `#if` body")?;
        Ok(items)
    }
}
