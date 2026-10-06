//! Recursive-descent parser producing `ast`.
//!
//! The parser works on the token vector from `lexer::lex`. Keywords are plain
//! identifiers, so every keyword check is made in a syntactic position where a
//! keyword is expected (`if`, `for`, `cast`, ...). Types are parsed as ordinary
//! expressions. Parsing stops at the first error; there is no recovery.
//!
//! Layout:
//! - `expr`      precedence climbing, unary/postfix operators, primaries
//! - `directive` `#directive` expressions (`#run`, `#code`, `#type`, ...)
//! - `asm`       `#asm` blocks (instructions, operands, register declarations)
//! - `procedure` procedure headers, parameters, return lists, lambdas
//! - `aggregate` struct/union/enum literals
//! - `stmt`      statements and control flow
//! - `decl`      declarations (names, types, values, `using` / `#as` modifiers, flags)
//! - `directive_stmt` statement-level directives (`#import`, `#load`, `#if`, `#run`, ...)
mod aggregate;
mod asm;
mod decl;
mod directive;
mod directive_stmt;
mod expr;
mod procedure;
mod stmt;
#[cfg(test)]
mod tests;

use crate::ast::{File, Ident, Note, Stmt};
use crate::lexer::{P, Tok, Token, lex};
use crate::source::{Diagnostic, FileId, Span};

pub(crate) type PResult<T> = Result<T, Diagnostic>;

/// Deepest nesting of expressions and statements the parser accepts. The parser and everything
/// after it recurse on the tree, so without a bound a file of a few kilobytes of `(` or `{`
/// overflows the stack. Real code stays far below this.
pub const MAX_NESTING: usize = 1000;

/// Parses one source file.
pub fn parse_file(file: FileId, text: &str) -> Result<File, Diagnostic> {
    let tokens = lex(file, text)?;
    let mut parser = Parser::new(text, tokens);
    let stmts = parser.parse_file_stmts()?;
    Ok(File {
        file,
        stmts,
    })
}

pub(crate) struct Parser<'a> {
    src: &'a str,
    toks: Vec<Token>,
    pos: usize,
    /// True inside argument and parameter lists, where a comma separates items
    /// instead of continuing a multi-value return list.
    in_list: bool,
    /// Operator text (`+`, `[]`, ...) for the procedure header parsed next.
    pending_operator: Option<std::rc::Rc<str>>,
    /// Token index just past a brace-terminated construct that ended with trailing flags
    /// (`struct {..} #no_padding`); such a statement needs no `;`.
    block_end: usize,
    /// Notes parsed in the middle of a construct, attached when it completes.
    pending_notes: Vec<Note>,
    /// Expressions and statements currently open (see `MAX_NESTING`).
    depth: usize,
}

impl<'a> Parser<'a> {
    pub(crate) fn new(src: &'a str, toks: Vec<Token>) -> Self {
        debug_assert!(matches!(toks.last().map(|t| &t.tok), Some(Tok::Eof)));
        Self {
            src,
            toks,
            pos: 0,
            in_list: false,
            pending_operator: None,
            block_end: usize::MAX,
            pending_notes: Vec::new(),
            depth: 0,
        }
    }

    /// Run `f` one nesting level deeper, failing past `MAX_NESTING`.
    fn nested<T>(&mut self, f: impl FnOnce(&mut Self) -> PResult<T>) -> PResult<T> {
        if self.depth >= MAX_NESTING {
            return Err(self.error(format!(
                "code is nested too deeply (more than {MAX_NESTING} levels)"
            )));
        }
        self.depth += 1;
        let result = f(self);
        self.depth -= 1;
        result
    }

    fn parse_file_stmts(&mut self) -> PResult<Vec<Stmt>> {
        let mut stmts = Vec::new();
        while !self.at_eof() {
            stmts.push(self.parse_stmt()?);
        }
        Ok(stmts)
    }

    // -- token access -----------------------------------------------------

    fn tok(&self) -> &Tok {
        self.tok_at(0)
    }
    fn tok_at(&self, n: usize) -> &Tok {
        &self.toks[(self.pos + n).min(self.toks.len() - 1)].tok
    }
    fn token_at(&self, n: usize) -> &Token {
        &self.toks[(self.pos + n).min(self.toks.len() - 1)]
    }
    fn span(&self) -> Span {
        self.token_at(0).span
    }
    fn prev_span(&self) -> Span {
        self.toks[self.pos.saturating_sub(1)].span
    }
    fn newline_before(&self) -> bool {
        self.token_at(0).newline_before
    }
    /// True if the previous token is a `#string` here-string, which ends a statement by itself.
    fn prev_is_here_string(&self) -> bool {
        let Some(prev) = self.pos.checked_sub(1).map(|i| &self.toks[i]) else {
            return false;
        };
        matches!(prev.tok, Tok::Str(_))
            && self.src.as_bytes().get(prev.span.start as usize) == Some(&b'#')
    }
    fn at_eof(&self) -> bool {
        matches!(self.tok(), Tok::Eof)
    }
    /// Consumes the current token and returns its span.
    fn bump(&mut self) -> Span {
        let span = self.span();
        if self.pos + 1 < self.toks.len() {
            self.pos += 1;
        }
        span
    }

    fn at(&self, p: P) -> bool {
        self.at_n(0, p)
    }
    fn at_n(&self, n: usize, p: P) -> bool {
        matches!(self.tok_at(n), Tok::Punct(q) if *q == p)
    }
    fn eat(&mut self, p: P) -> bool {
        let found = self.at(p);
        if found {
            self.bump();
        }
        found
    }
    fn expect(&mut self, p: P, context: &str) -> PResult<Span> {
        if self.at(p) {
            Ok(self.bump())
        } else {
            Err(self.expected(&format!("'{}'", p.text()), context))
        }
    }

    /// Text of the identifier at offset `n`, if the token is an identifier.
    fn kw_at(&self, n: usize) -> Option<&'static str> {
        match self.tok_at(n) {
            Tok::Ident(s) => Some(s.as_str()),
            _ => None,
        }
    }
    fn kw(&self) -> Option<&'static str> {
        self.kw_at(0)
    }
    fn at_kw(&self, kw: &str) -> bool {
        self.kw() == Some(kw)
    }
    fn eat_kw(&mut self, kw: &str) -> bool {
        let found = self.at_kw(kw);
        if found {
            self.bump();
        }
        found
    }
    /// Name of the directive at offset `n`, if the token is a directive.
    fn directive_at(&self, n: usize) -> Option<&'static str> {
        match self.tok_at(n) {
            Tok::Directive(s) => Some(s.as_str()),
            _ => None,
        }
    }
    fn directive(&self) -> Option<&'static str> {
        self.directive_at(0)
    }
    fn at_directive(&self, name: &str) -> bool {
        self.directive() == Some(name)
    }

    fn ident(&mut self, context: &str) -> PResult<Ident> {
        match self.tok() {
            Tok::Ident(name) => {
                let name = *name;
                let span = self.bump();
                Ok(Ident {
                    name,
                    span,
                })
            }
            _ => Err(self.expected("identifier", context)),
        }
    }

    /// Consumes a string literal token.
    fn string_lit(&mut self, context: &str) -> PResult<(std::rc::Rc<[u8]>, Span)> {
        match self.tok() {
            Tok::Str(s) => {
                let s = s.clone();
                Ok((s, self.bump()))
            }
            _ => Err(self.expected("string literal", context)),
        }
    }

    /// Consumes any trailing `@note` tokens.
    fn parse_notes(&mut self) -> Vec<Note> {
        let after_block = self.at_prev(P::RBrace);
        let mut notes = Vec::new();
        while let Tok::Note(text) = self.tok() {
            let text = text.clone();
            let span = self.bump();
            notes.push(Note {
                text,
                span,
            });
        }
        // `} @Note` still ends the statement with its block.
        if after_block && !notes.is_empty() {
            self.block_end = self.pos;
        }
        notes
    }

    // -- diagnostics ------------------------------------------------------

    fn error(&self, message: impl Into<String>) -> Diagnostic {
        Diagnostic::error(self.span(), message)
    }
    /// "expected X <context>, found Y" at the current token.
    fn expected(&self, what: &str, context: &str) -> Diagnostic {
        let context = if context.is_empty() {
            String::new()
        } else {
            format!(" {context}")
        };
        self.error(format!(
            "expected {what}{context}, found {}",
            describe(self.tok())
        ))
    }
}

/// Human-readable token description for diagnostics.
fn describe(tok: &Tok) -> String {
    match tok {
        Tok::Ident(s) => format!("'{s}'"),
        Tok::Directive(s) => format!("'#{s}'"),
        Tok::Note(s) => format!("'@{s}'"),
        Tok::Int(_) | Tok::Float(_) => "number".into(),
        Tok::Str(_) => "string literal".into(),
        Tok::Punct(p) => format!("'{}'", p.text()),
        Tok::Eof => "end of file".into(),
    }
}
