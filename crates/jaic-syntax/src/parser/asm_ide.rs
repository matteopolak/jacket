//! Tolerant `#asm` syntax for editors.
//!
//! While an instruction is being typed the block usually does not parse, so the strict parser
//! (`parser/asm.rs`) cannot say where the statements and operands are. [`asm_blocks`] finds
//! every block from the editor token stream (comments and strings are never mistaken for
//! code), splits its body into `;` statements and the statements into operands, and keeps
//! going over unfinished input: an unterminated `#asm {` has a body that runs to the brace
//! closing it, or to the end of the text.
//!
//! The shapes are syntactic only; which mnemonics, classes and registers exist is the
//! instruction catalog's business (`sema::asm_catalog`).
use crate::lexer::{self, P, Tok, Token};
use crate::source::{FileId, Span};

const FILE: FileId = FileId(0);

/// One `#asm` block.
#[derive(Clone, Debug)]
pub struct AsmBlockSyntax {
    /// The `#asm` token.
    pub directive: Span,
    /// The feature modifier names written between `#asm` and `{`.
    pub features: Vec<Span>,
    /// Where the header ends: the `{`, else the next token (or the end of the text).
    pub header_end: u32,
    /// The `{`, if there is one.
    pub open: Option<Span>,
    /// Between the `{` and the closing `}` (or the end of the text when unterminated).
    pub body: Span,
    /// The closing `}` is there.
    pub closed: bool,
    /// Default vector width in bytes the feature modifiers give: 16, 32 or 64.
    pub width: u64,
    /// The `;`-separated statements of the body, the last one possibly unfinished or empty.
    pub statements: Vec<AsmStmtSyntax>,
}

/// Where a byte offset is relative to an `#asm` block.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AsmPlace {
    /// Between `#asm` and `{`: a feature modifier, starting at the given offset, is typed.
    Features { word_start: u32 },
    /// Inside the body.
    Body,
}

impl AsmBlockSyntax {
    /// The whole block, `#asm` to the closing brace (or to what there is of it).
    pub fn span(&self) -> Span {
        let end = if self.closed {
            self.body.end + 1
        } else {
            self.body.end.max(self.header_end)
        };
        Span {
            end,
            ..self.directive
        }
    }

    pub fn place(&self, byte: usize) -> Option<AsmPlace> {
        let byte = byte as u32;
        if byte <= self.directive.start {
            return None;
        }
        if byte > self.directive.end && byte <= self.header_end {
            let word_start = self
                .features
                .iter()
                .find(|f| f.start < byte && byte <= f.end)
                .map_or(byte, |f| f.start);
            return Some(AsmPlace::Features {
                word_start,
            });
        }
        (self.open.is_some() && self.body.start <= byte && byte <= self.body.end)
            .then_some(AsmPlace::Body)
    }

    /// The statement holding `byte` (`start <= byte <= end`).
    pub fn statement_at(&self, byte: usize) -> Option<&AsmStmtSyntax> {
        let byte = byte as u32;
        self.statements
            .iter()
            .find(|s| s.span.start <= byte && byte <= s.span.end)
    }

    /// Vector width in bytes of an instruction with the given size suffix (`x`, `y`, `z`) in
    /// this block.
    pub fn width_for(&self, text: &str, suffix: Option<Span>) -> u64 {
        match suffix.and_then(|s| text.get(s.start as usize..s.end as usize)) {
            Some("x") => 16,
            Some("y") => 32,
            Some("z") => 64,
            _ => self.width,
        }
    }
}

/// One statement of a body, without its `;`.
#[derive(Clone, Debug)]
pub struct AsmStmtSyntax {
    pub span: Span,
    pub kind: AsmStmtKind,
}

#[derive(Clone, Debug)]
pub enum AsmStmtKind {
    /// Nothing but whitespace and comments.
    Empty,
    /// `name: class`, `name:`, optionally pinned.
    Decl {
        decl: AsmDeclSyntax,
        /// The `===` of a pin.
        pin: Option<Span>,
    },
    /// `name === reg`: pins an existing variable.
    Pin,
    /// `mnemonic.suffix operand, operand`.
    Inst(AsmInstSyntax),
    /// Anything else.
    Other,
}

#[derive(Clone, Debug)]
pub struct AsmInstSyntax {
    pub mnemonic: Span,
    /// After `.`: the size suffix, empty when only the dot is written.
    pub suffix: Option<Span>,
    /// Start of the operand list (after the whitespace following the mnemonic); `None` while
    /// the mnemonic is still the last thing written.
    pub operands_start: Option<u32>,
    /// Split at commas outside `[...]`/`(...)`; at least one when `operands_start` is set.
    pub operands: Vec<AsmOperandSyntax>,
}

#[derive(Clone, Debug)]
pub struct AsmOperandSyntax {
    /// From the previous comma (or the operand list's start) to the next comma (or the end of
    /// the statement); not trimmed.
    pub span: Span,
    /// `name:` or `name: class` at the start of the operand.
    pub decl: Option<AsmDeclSyntax>,
    /// A `===` in the operand.
    pub pin: Option<Span>,
}

/// `name:` with an optional class.
#[derive(Clone, Copy, Debug)]
pub struct AsmDeclSyntax {
    pub name: Span,
    pub class: Option<Span>,
    /// The class is what is being typed: it is missing, or it ends where the text parsed
    /// ends (as `parse_asm_statement`'s end does at the cursor).
    pub class_open: bool,
}

/// Every `#asm` block of `text`, in order.
pub fn asm_blocks(text: &str) -> Vec<AsmBlockSyntax> {
    if !text.contains("#asm") {
        return Vec::new();
    }
    let (tokens, _) = lexer::lex_ide(FILE, text);
    let mut out = Vec::new();
    let mut i = 0;
    while i < tokens.len() {
        if is_asm(&tokens[i]) {
            let (block, next) = block_at(text, &tokens, i);
            out.push(block);
            i = next.max(i + 1);
        } else {
            i += 1;
        }
    }
    out
}

/// Parses `text[start..end]` as one statement (to the cursor, say): the editor completes
/// what is before the cursor, so it asks about a statement cut there.
pub fn parse_asm_statement(text: &str, start: usize, end: usize) -> AsmStmtSyntax {
    let (mut tokens, _) = lexer::lex_ide(FILE, text.get(start..end).unwrap_or(""));
    tokens.retain(|t| t.tok != Tok::Eof);
    for t in &mut tokens {
        t.span.start += start as u32;
        t.span.end += start as u32;
    }
    statement(&tokens, start as u32, end as u32)
}

/// The identifier characters around `byte`, as the lexer takes them: `(start, end)`.
pub fn word_range(text: &str, byte: usize) -> (usize, usize) {
    let b = text.as_bytes();
    let byte = byte.min(b.len());
    let mut start = byte;
    while start > 0 && lexer::is_ident_char(b[start - 1]) {
        start -= 1;
    }
    let mut end = byte;
    while end < b.len() && lexer::is_ident_char(b[end]) {
        end += 1;
    }
    (start, end)
}

fn is_asm(t: &Token) -> bool {
    matches!(&t.tok, Tok::Directive(d) if d.as_str() == "asm")
}

fn is(t: &Token, p: P) -> bool {
    t.tok == Tok::Punct(p)
}

fn opens_brace(t: &Token) -> bool {
    is(t, P::LBrace) || is(t, P::DotBrace)
}

/// The block whose `#asm` is token `at`, and the index after it.
fn block_at(text: &str, tokens: &[Token], at: usize) -> (AsmBlockSyntax, usize) {
    let directive = tokens[at].span;
    let mut features = Vec::new();
    let mut width = 16;
    let mut i = at + 1;
    let mut header_last = directive.end;
    while let Some(t) = tokens.get(i) {
        match t.tok {
            Tok::Ident(_) => {
                let name = text
                    .get(t.span.start as usize..t.span.end as usize)
                    .unwrap_or("");
                if name.starts_with("AVX512") {
                    width = 64;
                } else if name.starts_with("AVX") && width < 32 {
                    width = 32;
                }
                features.push(t.span);
            }
            Tok::Punct(P::Comma) => {}
            _ => break,
        }
        header_last = t.span.end;
        i += 1;
    }
    let text_end = text.len() as u32;
    let Some(open) = tokens.get(i).filter(|t| is(t, P::LBrace)) else {
        let header_end = tokens
            .get(i)
            .filter(|t| t.tok != Tok::Eof)
            .map_or(text_end, |t| t.span.start)
            .max(header_last);
        let block = AsmBlockSyntax {
            directive,
            features,
            header_end,
            open: None,
            body: Span::new(FILE, header_end as usize, header_end as usize),
            closed: false,
            width,
            statements: Vec::new(),
        };
        return (block, i);
    };
    let mut depth = 1;
    let mut close = None;
    let mut j = i + 1;
    while let Some(t) = tokens.get(j) {
        if opens_brace(t) {
            depth += 1;
        } else if is(t, P::RBrace) {
            depth -= 1;
            if depth == 0 {
                close = Some(j);
                break;
            }
        }
        j += 1;
    }
    let body_end = close.map_or(text_end, |c| tokens[c].span.start);
    let body_tokens = &tokens[i + 1..close.unwrap_or(tokens.len())];
    let body_tokens = match body_tokens.last() {
        Some(t) if t.tok == Tok::Eof => &body_tokens[..body_tokens.len() - 1],
        _ => body_tokens,
    };
    let mut statements = Vec::new();
    let mut from = open.span.end;
    let mut first = 0;
    for (n, t) in body_tokens.iter().enumerate() {
        if is(t, P::Semi) {
            statements.push(statement(&body_tokens[first..n], from, t.span.start));
            from = t.span.end;
            first = n + 1;
        }
    }
    statements.push(statement(&body_tokens[first..], from, body_end));
    let block = AsmBlockSyntax {
        directive,
        features,
        header_end: open.span.start,
        open: Some(open.span),
        body: Span::new(FILE, open.span.end as usize, body_end as usize),
        closed: close.is_some(),
        width,
        statements,
    };
    (block, close.map_or(tokens.len(), |c| c + 1))
}

fn pin_in(tokens: &[Token]) -> Option<Span> {
    tokens
        .windows(2)
        .find(|w| is(&w[0], P::EqEq) && is(&w[1], P::Eq) && w[0].span.end == w[1].span.start)
        .map(|w| w[0].span.to(w[1].span))
}

/// `name:` (and a class) at the start of `tokens`; `end` is where the parsed text ends.
fn decl_in(tokens: &[Token], end: u32) -> Option<AsmDeclSyntax> {
    let [name, colon, rest @ ..] = tokens else {
        return None;
    };
    if !matches!(name.tok, Tok::Ident(_)) || !is(colon, P::Colon) {
        return None;
    }
    let class = rest
        .first()
        .filter(|t| matches!(t.tok, Tok::Ident(_)))
        .map(|t| t.span);
    Some(AsmDeclSyntax {
        name: name.span,
        class,
        class_open: class.map_or(rest.is_empty(), |c| c.end == end),
    })
}

/// The statement made of `tokens`, which lie in `start..end`.
fn statement(tokens: &[Token], start: u32, end: u32) -> AsmStmtSyntax {
    let span = Span::new(FILE, start as usize, end as usize);
    let kind = match tokens {
        [] => AsmStmtKind::Empty,
        [first, rest @ ..] if matches!(first.tok, Tok::Ident(_)) => {
            if let Some(decl) = decl_in(tokens, end) {
                AsmStmtKind::Decl {
                    decl,
                    pin: pin_in(&tokens[2..]),
                }
            } else if pin_in(&rest[..rest.len().min(2)]).is_some() {
                AsmStmtKind::Pin
            } else {
                instruction(first, rest, end)
            }
        }
        _ => AsmStmtKind::Other,
    };
    AsmStmtSyntax {
        span,
        kind,
    }
}

fn instruction(mnemonic: &Token, rest: &[Token], end: u32) -> AsmStmtKind {
    let mut used = 0;
    let mut stop = mnemonic.span.end;
    let mut suffix = None;
    match rest.first().map(|t| &t.tok) {
        Some(Tok::Punct(P::Dot)) => {
            let dot = &rest[0];
            stop = dot.span.end;
            used = 1;
            suffix = Some(Span::new(FILE, stop as usize, stop as usize));
            if let Some(t) = rest.get(1)
                && t.span.start == dot.span.end
                && matches!(t.tok, Tok::Ident(_) | Tok::Int(_))
            {
                suffix = Some(t.span);
                stop = t.span.end;
                used = 2;
            }
        }
        Some(Tok::Punct(P::Question)) if rest[0].span.start == stop => {
            // `?T`: a size written as a type, up to the next whitespace.
            while let Some(t) = rest.get(used)
                && t.span.start == stop
            {
                stop = t.span.end;
                used += 1;
            }
        }
        _ => {}
    }
    let after = &rest[used..];
    let operands_start = match after.first() {
        Some(t) if t.span.start == stop => return AsmStmtKind::Other,
        Some(t) => Some(t.span.start),
        None => (end > stop).then_some(end),
    };
    let operands = operands_start.map_or_else(Vec::new, |s| operands(after, s, end));
    AsmStmtKind::Inst(AsmInstSyntax {
        mnemonic: mnemonic.span,
        suffix,
        operands_start,
        operands,
    })
}

fn operands(tokens: &[Token], start: u32, end: u32) -> Vec<AsmOperandSyntax> {
    let mut out = Vec::new();
    let (mut from, mut first, mut depth) = (start, 0, 0i32);
    let push = |tokens: &[Token], from: u32, to: u32, out: &mut Vec<AsmOperandSyntax>| {
        out.push(AsmOperandSyntax {
            span: Span::new(FILE, from as usize, to as usize),
            decl: decl_in(tokens, to),
            pin: pin_in(tokens),
        });
    };
    for (n, t) in tokens.iter().enumerate() {
        match t.tok {
            Tok::Punct(P::LBracket | P::LParen) => depth += 1,
            Tok::Punct(P::RBracket | P::RParen) => depth -= 1,
            Tok::Punct(P::Comma) if depth <= 0 => {
                push(&tokens[first..n], from, t.span.start, &mut out);
                from = t.span.end;
                first = n + 1;
            }
            _ => {}
        }
    }
    push(&tokens[first..], from, end, &mut out);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text_of(text: &str, span: Span) -> &str {
        &text[span.start as usize..span.end as usize]
    }

    #[test]
    fn statements_and_operands_are_split() {
        let text = "x := 1; #asm AVX2 {\n  t: gpr === a; mov.q tmp:, [p + 8]; // c;\n  vaddps.y a, b, c\n} y := 2;";
        let blocks = asm_blocks(text);
        assert_eq!(blocks.len(), 1);
        let b = &blocks[0];
        assert!(b.closed);
        assert_eq!(b.width, 32);
        let whole = &text[text.find("#asm").unwrap()..=text.find("} y").unwrap()];
        assert_eq!(text_of(text, b.span()), whole);
        let kinds: Vec<_> = b.statements.iter().map(|s| &s.kind).collect();
        assert_eq!(kinds.len(), 3);
        let AsmStmtKind::Decl {
            decl,
            pin: Some(_),
        } = kinds[0]
        else {
            panic!("{:?}", kinds[0])
        };
        assert_eq!(text_of(text, decl.name), "t");
        let AsmStmtKind::Inst(mov) = kinds[1] else {
            panic!("{:?}", kinds[1])
        };
        assert_eq!(text_of(text, mov.suffix.unwrap()), "q");
        assert_eq!(mov.operands.len(), 2);
        assert!(mov.operands[0].decl.is_some());
        assert_eq!(text_of(text, mov.operands[1].span).trim(), "[p + 8]");
        let AsmStmtKind::Inst(add) = kinds[2] else {
            panic!("{:?}", kinds[2])
        };
        assert_eq!(add.operands.len(), 3);
    }

    #[test]
    fn strings_and_comments_hide_the_directive() {
        assert!(asm_blocks("// #asm { x\n\"#asm {\" /* #asm { */").is_empty());
    }

    #[test]
    fn an_unterminated_body_runs_to_the_end() {
        let text = "f :: () {\n #asm {\n mov.q x, 1;\n ad";
        let blocks = asm_blocks(text);
        assert_eq!(blocks.len(), 1);
        assert!(!blocks[0].closed);
        assert_eq!(blocks[0].body.end as usize, text.len());
        assert_eq!(blocks[0].statements.len(), 2);
        let last = &blocks[0].statements[1];
        assert!(matches!(&last.kind, AsmStmtKind::Inst(i) if i.operands_start.is_none()));
    }

    #[test]
    fn a_cut_statement_reports_what_is_being_typed() {
        let text = "#asm { mov.q x, ";
        let stmt = parse_asm_statement(text, 7, text.len());
        let AsmStmtKind::Inst(inst) = stmt.kind else {
            panic!()
        };
        assert_eq!(inst.operands.len(), 2);
        assert_eq!(inst.operands[1].span.end as usize, text.len());
        let text = "#asm { t: g";
        let stmt = parse_asm_statement(text, 7, text.len());
        assert!(matches!(stmt.kind, AsmStmtKind::Decl { decl, .. } if decl.class_open));
    }

    #[test]
    fn header_only_blocks_report_the_header() {
        let blocks = asm_blocks("#asm AVX, AV");
        assert_eq!(blocks[0].features.len(), 2);
        assert!(blocks[0].open.is_none());
        assert_eq!(
            blocks[0].place(12),
            Some(AsmPlace::Features {
                word_start: 10
            })
        );
    }
}
