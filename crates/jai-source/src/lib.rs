//! Byte-based source locations shared by the compiler stages.
mod locations;
pub use locations::{
    DeclarationId, Identities, LocatedDiagnostic, ModuleId, ScopeId, SourceId, SourceMap,
    SourceRecord, SourceSpan, UnitId,
};
use std::collections::HashMap;
use std::fmt;

/// An interned spelling; declaration identity is represented separately.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Symbol(usize);

#[derive(Clone, Debug, Default)]
pub struct Symbols {
    names: Vec<String>,
    by_name: HashMap<String, Symbol>,
}
impl Symbols {
    pub fn intern(&mut self, name: &str) -> Symbol {
        if let Some(&symbol) = self.by_name.get(name) {
            return symbol;
        }
        let symbol = Symbol(self.names.len());
        self.names.push(name.to_owned());
        self.by_name.insert(name.to_owned(), symbol);
        symbol
    }
    pub fn find(&self, name: &str) -> Option<Symbol> {
        self.by_name.get(name).copied()
    }
    pub fn name(&self, symbol: Symbol) -> &str {
        &self.names[symbol.0]
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
    pub fn text(self, source: &str) -> &str {
        &source[self.start..self.end]
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub span: Span,
    pub message: String,
}

impl Diagnostic {
    pub fn new(span: Span, message: impl Into<String>) -> Self {
        Self {
            span,
            message: message.into(),
        }
    }
    pub fn render(&self, path: &str, source: &str) -> String {
        let at = self.span.start.min(source.len());
        let prefix = &source[..source.floor_char_boundary(at)];
        let line = prefix.bytes().filter(|&b| b == b'\n').count() + 1;
        let column = prefix.rsplit('\n').next().unwrap_or("").chars().count() + 1;
        format!("{path}:{line}:{column}: error: {}", self.message)
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for Diagnostic {}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unicode_columns_and_crlf() {
        let s = "é x\r\ny";
        assert_eq!(
            Diagnostic::new(Span::new(3, 4), "bad").render("a.jai", s),
            "a.jai:1:3: error: bad"
        );
        assert_eq!(
            Diagnostic::new(Span::new(6, 7), "bad").render("a.jai", s),
            "a.jai:2:1: error: bad"
        );
    }
}
