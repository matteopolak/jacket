//! `#asm` blocks.
//!
//! The body is a list of `;`-terminated items in the `mnemonic operand, operand` form:
//!
//! ```text
//! #asm AVX2 {
//!     x: gpr === a;               // declaration (pinning is accepted and ignored)
//!     mov.q var, 17;              // `.q` / `.64` / `?T` size
//!     lea.q rax, [rdx + rax*4];   // memory operand: base + index*scale +/- disp
//!     mov tmp:, [p + 8];          // `tmp:` declares a register at its first use
//! }
//! ```
//!
//! The parser is mnemonic-agnostic; `sema::asm` owns the instruction table.
use super::expr::mk;
use super::{PResult, Parser};
use crate::ast::{
    AsmBlock, AsmDecl, AsmInst, AsmItem, AsmMem, AsmMemTerm, AsmOperand, AsmPin, AsmSize, Expr,
    ExprKind, Ident,
};
use crate::intern::Sym;
use crate::lexer::{P, Tok};
use std::rc::Rc;

impl Parser<'_> {
    /// `#asm FEATURE, ... { ... }`
    pub(super) fn parse_asm(&mut self) -> PResult<Expr> {
        let start = self.bump();
        let mut features = Vec::new();
        while !self.at(P::LBrace) {
            features.push(self.ident("as an '#asm' feature name or '{'")?);
            if !self.eat(P::Comma) {
                break;
            }
        }
        self.expect(P::LBrace, "after '#asm'")?;
        let mut items = Vec::new();
        loop {
            if self.eat(P::Semi) {
                continue;
            }
            if self.at(P::RBrace) {
                break;
            }
            if self.at_eof() {
                return Err(self.error("unterminated '#asm' block"));
            }
            items.push(self.parse_asm_item()?);
            if !self.at(P::RBrace) {
                self.expect(P::Semi, "after the '#asm' instruction")?;
            }
        }
        let end = self.bump();
        Ok(mk(
            ExprKind::Asm(Rc::new(AsmBlock {
                features,
                items,
            })),
            start.to(end),
        ))
    }

    /// At `===` (lexed as `==` followed by `=`) at offset `n`.
    fn at_asm_pin(&self, n: usize) -> bool {
        self.at_n(n, P::EqEq)
            && self.at_n(n + 1, P::Eq)
            && self.token_at(n).span.end == self.token_at(n + 1).span.start
    }

    fn parse_asm_item(&mut self) -> PResult<AsmItem> {
        if matches!(self.tok(), Tok::Ident(_)) && (self.at_n(1, P::Colon) || self.at_asm_pin(1)) {
            return Ok(AsmItem::Decl(self.parse_asm_decl()?));
        }
        let mnemonic = self.ident("as an instruction mnemonic")?;
        let size = self.parse_asm_size()?;
        let mut operands = Vec::new();
        if !self.at(P::Semi) && !self.at(P::RBrace) {
            loop {
                operands.push(self.parse_asm_operand()?);
                if !self.eat(P::Comma) {
                    break;
                }
            }
        }
        Ok(AsmItem::Inst(AsmInst {
            mnemonic,
            size,
            operands,
            span: mnemonic.span.to(self.prev_span()),
        }))
    }

    /// `.q` / `.64` / `?T` after a mnemonic.
    fn parse_asm_size(&mut self) -> PResult<Option<AsmSize>> {
        if self.eat(P::Dot) {
            let span = self.span();
            let name = match self.tok().clone() {
                Tok::Ident(name) => name,
                Tok::Int(bits) => Sym::intern(&bits.to_string()),
                _ => return Err(self.expected("operand size", "after '.'")),
            };
            self.bump();
            return Ok(Some(AsmSize::Suffix(Ident {
                name,
                span,
            })));
        }
        if self.eat(P::Question) {
            return Ok(Some(AsmSize::Dynamic(Box::new(self.parse_unary()?))));
        }
        Ok(None)
    }

    fn parse_asm_operand(&mut self) -> PResult<AsmOperand> {
        if self.at(P::LBracket) {
            return self.parse_asm_mem();
        }
        if matches!(self.tok(), Tok::Ident(_)) && self.at_n(1, P::Colon) {
            return Ok(AsmOperand::Decl(self.parse_asm_decl()?));
        }
        Ok(AsmOperand::Value(self.parse_unary()?))
    }

    /// `name`, `name:`, `name: class`, optionally followed by `=== reg`.
    fn parse_asm_decl(&mut self) -> PResult<AsmDecl> {
        let name = self.ident("as a register name")?;
        let colon = self.eat(P::Colon);
        let class = if colon && matches!(self.tok(), Tok::Ident(_)) {
            Some(self.ident("as a register class")?)
        } else {
            None
        };
        let pin = if self.at_asm_pin(0) {
            self.bump();
            self.bump();
            Some(match self.tok().clone() {
                Tok::Ident(name) => AsmPin::Name(Ident {
                    name,
                    span: self.bump(),
                }),
                Tok::Int(index) => {
                    self.bump();
                    AsmPin::Index(index)
                }
                _ => return Err(self.expected("register name or number", "after '==='")),
            })
        } else {
            None
        };
        Ok(AsmDecl {
            name,
            colon,
            class,
            pin,
        })
    }

    /// `[base + index*scale - disp]`: each term is a unary expression (`x`, `*x`, `10`, `(1 + 2)`).
    fn parse_asm_mem(&mut self) -> PResult<AsmOperand> {
        let start = self.bump();
        let mut terms = Vec::new();
        let mut negate = self.eat(P::Minus);
        loop {
            let value = self.parse_unary()?;
            let scale = if self.eat(P::Star) {
                Some(self.parse_unary()?)
            } else {
                None
            };
            terms.push(AsmMemTerm {
                negate,
                value,
                scale,
            });
            if self.eat(P::Plus) {
                negate = false;
            } else if self.eat(P::Minus) {
                negate = true;
            } else {
                break;
            }
        }
        let end = self.expect(P::RBracket, "to close the memory operand")?;
        Ok(AsmOperand::Mem(AsmMem {
            terms,
            span: start.to(end),
        }))
    }
}
