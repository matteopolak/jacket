//! `struct`, `union` and `enum` literals.
use super::expr::mk;
use super::{PResult, Parser};
use crate::ast::{
    AstId, EnumItem, EnumLit, EnumMember, Expr, ExprKind, StructFlags, StructKind, StructLit,
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
        let body = self.parse_stmts_until_close()?;
        let end = self.expect(P::RBrace, "to end the body")?;
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
            match name {
                "type_info_none" => flags.type_info_none = true,
                "type_info_procedures_are_void_pointers" => {
                    flags.type_info_procedures_are_void_pointers = true
                }
                "type_info_no_size_complaint" => flags.type_info_no_size_complaint = true,
                "no_padding" => flags.no_padding = true,
                "align" => {
                    self.bump();
                    flags.align = Some(self.parse_unary()?);
                    continue;
                }
                "modify" => {
                    self.bump();
                    modify = Some(self.parse_modify_body()?);
                    continue;
                }
                _ => break,
            }
            self.bump();
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
        let end = self.expect(P::RBrace, "to end the enum body")?;
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
        let mut items = Vec::new();
        while !self.at(P::RBrace) {
            if self.eat(P::Semi) {
                continue;
            }
            if self.at_directive("insert") {
                self.bump();
                let value = self.parse_expr()?;
                items.push(EnumItem::Insert(value));
            } else if self.at_directive("if") {
                items.push(self.parse_enum_if()?);
            } else {
                items.push(EnumItem::Member(self.parse_enum_member()?));
            }
        }
        Ok(items)
    }

    fn parse_enum_member(&mut self) -> PResult<EnumMember> {
        let name = self.ident("as enum member name")?;
        let value = if self.eat(P::ColonColon) || self.eat(P::Eq) {
            Some(self.parse_expr()?)
        } else {
            None
        };
        let mut notes = self.parse_notes();
        if !(self.eat(P::Semi) || self.eat(P::Comma) || self.at(P::RBrace)) {
            return Err(self.expected("';'", "after the enum member"));
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
        self.expect(P::LBrace, "to start the '#if' body")?;
        let items = self.parse_enum_items()?;
        self.expect(P::RBrace, "to end the '#if' body")?;
        Ok(items)
    }
}
