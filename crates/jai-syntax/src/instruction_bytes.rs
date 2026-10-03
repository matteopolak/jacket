//! Literal instruction-byte syntax retained before source branch selection.
use super::*;

pub const MAX_INSTRUCTION_BYTES: usize = 4096;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstructionBytes {
    pub bytes: Box<[u8]>,
    pub span: Span,
}

impl Parser<'_> {
    pub(super) fn instruction_bytes(&mut self) -> Result<InstructionBytes, Diagnostic> {
        let start = self.token().span.start;
        self.at += 1; // The statement parser recognized #bytes.
        self.need(Punct::ArrayLiteral)?;
        let mut bytes = Vec::new();
        while !self.take(Punct::CloseBracket) {
            if bytes.len() == MAX_INSTRUCTION_BYTES {
                return Err(self.error("#bytes payload exceeds 4096 bytes"));
            }
            if self.token().kind != Kind::Number {
                return Err(self.error("#bytes requires u8 integer literals"));
            }
            let byte = numeric_literals::integer(self.text())
                .ok()
                .and_then(|value| u8::try_from(value).ok())
                .ok_or_else(|| self.error("#bytes requires u8 integer literals"))?;
            self.at += 1;
            bytes.push(byte);
            if !self.take(Punct::Comma) {
                self.need(Punct::CloseBracket)?;
                break;
            }
        }
        self.need(Punct::Semicolon)?;
        Ok(InstructionBytes {
            bytes: bytes.into_boxed_slice(),
            span: Span::new(start, self.tokens[self.at - 1].span.end),
        })
    }
}
