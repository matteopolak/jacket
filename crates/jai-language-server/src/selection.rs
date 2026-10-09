//! Selection ranges: what "expand selection" grows through.
//!
//! From the token under the cursor outwards: the expressions around it, the statements, the
//! bodies between braces and with them, the declaration, and the whole file. A document that
//! does not parse still has its token and the whole file.
use crate::analysis::{Span, TokenKind};
use crate::refactor::{extent, stmt_extent, trim};
use crate::session::Session;
use crate::{Error, Position, SelectionRange};
use jaic::ast::{Block, ExprKind as E, Stmt, StmtKind as S};
use jaic::source::FileId;
use jailint::syntax::{Node, walk};

/// The ranges around `byte`, in no order.
fn around(text: &str, stmts: &[Stmt], byte: usize, out: &mut Vec<(usize, usize)>) {
    let mut add = |(a, b): (usize, usize)| {
        if a <= byte && byte <= b && b <= text.len() {
            out.push((a, b));
        }
    };
    let inner = |block: &Block, add: &mut dyn FnMut((usize, usize))| {
        let (a, b) = (block.span.start as usize, block.span.end as usize);
        add((a, b));
        if b > a + 1 && text.get(a..a + 1) == Some("{") && text.get(b - 1..b) == Some("}") {
            add(trim(text, a + 1, b - 1));
        }
    };
    let mut nested: Vec<&[Stmt]> = Vec::new();
    walk(stmts, &mut |node, _| {
        match node {
            Node::Stmt(s) => {
                add(stmt_extent(text, s));
                match &s.kind {
                    S::Block(b) => inner(b, &mut add),
                    S::Switch {
                        cases, ..
                    }
                    | S::StaticSwitch {
                        cases, ..
                    } => {
                        for c in cases {
                            add((c.span.start as usize, c.span.end as usize));
                        }
                    }
                    _ => {}
                }
            }
            Node::Expr(x) => {
                add(extent(text, x.span));
                match &x.kind {
                    E::Block(b) => inner(b, &mut add),
                    E::Proc(lit) => {
                        if let Some(body) = &lit.body {
                            inner(body, &mut add);
                            nested.push(&body.stmts);
                        }
                        return false;
                    }
                    E::Struct(st) => {
                        nested.push(&st.body);
                        return false;
                    }
                    _ => {}
                }
            }
        }
        true
    });
    for list in nested {
        around(text, list, byte, out);
    }
}

impl Session {
    /// The selection ranges around each of `positions`, innermost first.
    pub fn selection_ranges(
        &self,
        uri: &crate::DocumentUri,
        positions: &[Position],
    ) -> Result<Vec<SelectionRange>, Error> {
        let doc = self.document(uri)?;
        let text = &doc.text;
        let parsed = jaic::parser::parse_file(FileId(0), text).ok();
        let mut result = Vec::new();
        for &position in positions {
            let byte = doc.index.byte(text, position)?;
            let mut found: Vec<(usize, usize)> = Vec::new();
            // The token under the cursor (the later one when between two), and a string's text.
            let tokens = &self.analyses[uri].tokens;
            let token = tokens
                .iter()
                .rev()
                .find(|t| t.span.start <= byte && byte < t.span.end)
                .or_else(|| tokens.iter().find(|t| t.span.end == byte));
            if let Some(t) = token {
                found.push((t.span.start, t.span.end));
                if t.kind == TokenKind::String && t.span.end > t.span.start + 1 {
                    let (a, b) = (t.span.start + 1, t.span.end - 1);
                    if a <= byte && byte <= b {
                        found.push((a, b));
                    }
                }
            }
            if let Some(file) = &parsed {
                around(text, &file.stmts, byte, &mut found);
            }
            found.push((0, text.len()));
            // Smallest first, each range holding the one before it.
            found.sort_by_key(|&(a, b)| (b - a, a));
            found.dedup();
            let mut chain: Vec<(usize, usize)> = Vec::new();
            for range in found {
                if chain
                    .last()
                    .is_none_or(|l| range.0 <= l.0 && l.1 <= range.1 && range != *l)
                {
                    chain.push(range);
                }
            }
            let mut selection: Option<SelectionRange> = None;
            for (a, b) in chain.into_iter().rev() {
                let range = doc.index.range(text, Span::new(a, b))?;
                selection = Some(SelectionRange {
                    range,
                    parent: selection.map(Box::new),
                });
            }
            result.push(selection.expect("the whole file is always a range"));
        }
        Ok(result)
    }
}
