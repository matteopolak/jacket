//! Closed, source-backed SIMD assembly syntax; machine constraints belong to checking.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SimdFeature {
    Avx,
    Avx2,
    Unsupported(Symbol),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SimdFeatureRequirement {
    pub feature: SimdFeature,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SimdOpcode {
    Movups,
    Addps,
    Movdqu,
    Paddb,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SimdWidth {
    X,
    Y,
}

#[derive(Clone, Debug)]
pub struct SimdBlock {
    pub span: Span,
    pub features: Vec<SimdFeatureRequirement>,
    pub statements: Vec<SimdStatement>,
}

#[derive(Clone, Debug)]
pub enum SimdStatement {
    Unsupported { span: Span },
    DebugTrap { span: Span },
    Interrupt { vector: u8, span: Span },
    RegisterDeclaration { name: Symbol, span: Span },
    Instruction(SimdInstruction),
}

#[derive(Clone, Debug)]
pub struct SimdInstruction {
    pub opcode: SimdOpcode,
    pub width: SimdWidth,
    pub operands: Vec<SimdOperand>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct SimdOperand {
    pub kind: SimdOperandKind,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum SimdOperandKind {
    Register { name: Symbol, introduce: bool },
    Memory(Expression),
}

impl Parser<'_> {
    pub(super) fn simd_block(&mut self) -> Result<SimdBlock, Diagnostic> {
        let start = self.token().span.start;
        self.at += 1; // #asm has already been recognized by statement parsing.
        let mut features = Vec::new();
        if !self.is(Punct::OpenBrace) {
            loop {
                let span = self.token().span;
                let feature = match (self.token().kind, self.text()) {
                    (Kind::Ident, "AVX") => SimdFeature::Avx,
                    (Kind::Ident, "AVX2") => SimdFeature::Avx2,
                    (Kind::Ident, _) => SimdFeature::Unsupported(self.name()?),
                    _ => return Err(self.error("#asm feature list requires identifiers")),
                };
                if !matches!(feature, SimdFeature::Unsupported(_)) {
                    self.at += 1;
                }
                features.push(SimdFeatureRequirement {
                    feature,
                    span,
                });
                if !self.take(Punct::Comma) {
                    break;
                }
            }
        }
        self.need(Punct::OpenBrace)?;
        let mut statements = Vec::new();
        while !self.take(Punct::CloseBrace) {
            if self.token().kind == Kind::Eof {
                return Err(self.error("unterminated #asm block"));
            }
            statements.push(self.simd_statement()?);
        }
        Ok(SimdBlock {
            span: Span::new(start, self.tokens[self.at - 1].span.end),
            features,
            statements,
        })
    }

    fn simd_statement(&mut self) -> Result<SimdStatement, Diagnostic> {
        let start = self.token().span.start;
        if self.token().kind == Kind::Ident
            && self.text() == "int3"
            && !self.named_prefix(Punct::Colon)
        {
            self.at += 1;
            self.need(Punct::Semicolon)?;
            return Ok(SimdStatement::DebugTrap {
                span: Span::new(start, self.tokens[self.at - 1].span.end),
            });
        }
        if self.token().kind == Kind::Ident
            && self.text() == "int"
            && !self.named_prefix(Punct::Colon)
        {
            self.at += 1;
            if self.token().kind != Kind::Number {
                return Err(self.error("#asm interrupt requires a u8 integer literal"));
            }
            let vector = numeric_literals::integer(self.text())
                .ok()
                .and_then(|value| u8::try_from(value).ok())
                .ok_or_else(|| self.error("#asm interrupt requires a u8 integer literal"))?;
            self.at += 1;
            self.need(Punct::Semicolon)?;
            return Ok(SimdStatement::Interrupt {
                vector,
                span: Span::new(start, self.tokens[self.at - 1].span.end),
            });
        }
        if self.named_prefix(Punct::Colon) {
            let ty = self.tokens.get(self.at + 2).copied();
            if !ty.is_some_and(|token| {
                token.kind == Kind::Ident && token.span.text(self.source) == "vec"
            }) {
                return self.unsupported_simd_statement();
            }
            let name = self.name()?;
            self.need(Punct::Colon)?;
            self.at += 1;
            self.need(Punct::Semicolon)?;
            return Ok(SimdStatement::RegisterDeclaration {
                name,
                span: Span::new(start, self.tokens[self.at - 1].span.end),
            });
        }
        let opcode = match (self.token().kind, self.text()) {
            (Kind::Ident, "movups") => SimdOpcode::Movups,
            (Kind::Ident, "addps") => SimdOpcode::Addps,
            (Kind::Ident, "movdqu") => SimdOpcode::Movdqu,
            (Kind::Ident, "paddb") => SimdOpcode::Paddb,
            _ => return self.unsupported_simd_statement(),
        };
        self.at += 1;
        self.need(Punct::Dot)?;
        let width = match (self.token().kind, self.text()) {
            (Kind::Ident, "x") => SimdWidth::X,
            (Kind::Ident, "y") => SimdWidth::Y,
            _ => return Err(self.error("unsupported #asm width modifier; expected x or y")),
        };
        self.at += 1;
        let mut operands = vec![self.simd_operand()?];
        while self.take(Punct::Comma) {
            operands.push(self.simd_operand()?);
        }
        self.need(Punct::Semicolon)?;
        Ok(SimdStatement::Instruction(SimdInstruction {
            opcode,
            width,
            operands,
            span: Span::new(start, self.tokens[self.at - 1].span.end),
        }))
    }

    fn unsupported_simd_statement(&mut self) -> Result<SimdStatement, Diagnostic> {
        let start = self.token().span.start;
        let mut closing = Vec::new();
        loop {
            match self.token().kind {
                Kind::Eof => return Err(self.error("unterminated unsupported #asm statement")),
                Kind::Punctuation(Punct::Semicolon) if closing.is_empty() => {
                    self.at += 1;
                    return Ok(SimdStatement::Unsupported {
                        span: Span::new(start, self.tokens[self.at - 1].span.end),
                    });
                }
                Kind::Punctuation(Punct::OpenParen) => closing.push(Punct::CloseParen),
                Kind::Punctuation(Punct::OpenBracket | Punct::ArrayLiteral) => {
                    closing.push(Punct::CloseBracket)
                }
                Kind::Punctuation(Punct::OpenBrace | Punct::StructLiteral) => {
                    closing.push(Punct::CloseBrace)
                }
                Kind::Punctuation(
                    punctuation @ (Punct::CloseParen | Punct::CloseBracket | Punct::CloseBrace),
                ) if closing.pop() != Some(punctuation) => {
                    return Err(self.error("unbalanced or unterminated unsupported #asm statement; expected ';' after a balanced statement"));
                }
                _ => {}
            }
            self.at += 1;
        }
    }

    fn simd_operand(&mut self) -> Result<SimdOperand, Diagnostic> {
        let start = self.token().span.start;
        let kind = if self.take(Punct::OpenBracket) {
            let address = self.expression(0)?;
            self.need(Punct::CloseBracket)?;
            SimdOperandKind::Memory(address)
        } else if self.token().kind == Kind::Ident {
            let name = self.name()?;
            let introduce = self.take(Punct::Colon);
            SimdOperandKind::Register {
                name,
                introduce,
            }
        } else {
            return Err(self.error("expected #asm register or bracketed memory operand"));
        };
        Ok(SimdOperand {
            kind,
            span: Span::new(start, self.tokens[self.at - 1].span.end),
        })
    }
}
