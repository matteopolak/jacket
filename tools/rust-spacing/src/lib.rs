//! Inserts the blank lines rustfmt never adds, using `syn` spans so only real statement and
//! item boundaries are touched.
//!
//! The pass only inserts: it never removes or reflows code, a blank line that exists stays, and
//! running it twice gives the same text. A comment directly above a statement belongs to it, so
//! the blank line goes above the comment.
//!
//! Inside every block, between two statements A and B, a blank line goes in when:
//!
//! - A or B is a multi-line `if`, `match`, `loop`, `while` or `for` (a guard block);
//! - A or B is a multi-line item;
//! - A is a `let` and B is not (a run of `let`s ends); except that a block's last single-line
//!   statement stays attached to a lone `let` (`let n = f(); n + 1`);
//! - B is the last statement, a `return`, `break` or `continue`, of a block of three or more.
//!
//! Between two items of a file, `mod`, `impl`, `trait` or `extern` block, a blank line goes in
//! when either is multi-line (attributes count) or one is an import (`use`, `extern crate`,
//! `mod name;`) and the other is not. Two imports in a row are left to rustfmt's import grouping.
//!
//! Statements inside macro invocations are token trees and are left alone.

use proc_macro2::Span;
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Expr, File, ImplItem, Item, Stmt, TraitItem};

/// Returns `source` with the missing blank lines added.
///
/// ```
/// let out = rust_spacing::space("fn f() {\n    let a = 1;\n    run(a);\n}\n").unwrap();
/// assert_eq!(out, "fn f() {\n    let a = 1;\n\n    run(a);\n}\n");
/// ```
///
/// # Errors
///
/// When `source` is not a Rust file.
pub fn space(source: &str) -> syn::Result<String> {
    let file = syn::parse_file(source)?;
    let lines: Vec<&str> = source.split('\n').collect();
    let mut pass = Pass {
        lines: &lines,
        before: Vec::new(),
    };

    pass.visit_file(&file);
    pass.before.sort_unstable();
    pass.before.dedup();

    let mut out = String::with_capacity(source.len() + pass.before.len());
    let mut next = pass.before.iter().copied().peekable();

    for (index, line) in lines.iter().enumerate() {
        if next.next_if_eq(&index).is_some() {
            out.push('\n');
        }

        out.push_str(line);

        if index + 1 < lines.len() {
            out.push('\n');
        }
    }

    Ok(out)
}

/// What a statement or item is, as far as spacing is concerned.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Guard,
    Let,
    Flow,
    Tail,
    Import,
    Other,
}

/// One statement or item: its kind and where it sits in the file.
struct Node {
    kind: Kind,
    /// Whether it is a multi-line item, which is set apart on both sides.
    big_item: bool,
    /// Whether it spans more than one line.
    multi: bool,
    /// 0-based first and last line, and the column of the first character.
    first: usize,
    last: usize,
    column: usize,
}

impl Node {
    fn new(kind: Kind, item: bool, span: Span) -> Self {
        Self::with_head(kind, item, span, span)
    }

    /// Like `new`, but whether the node is multi-line is judged from `head`, where its
    /// declaration proper starts (after its attributes and doc comments).
    fn with_head(kind: Kind, item: bool, span: Span, head: Span) -> Self {
        let (start, end) = (span.start(), span.end());
        let multi = end.line > head.start().line;

        Node {
            kind,
            big_item: item && multi,
            multi,
            first: start.line - 1,
            last: end.line - 1,
            column: start.column,
        }
    }
}

struct Pass<'a> {
    lines: &'a [&'a str],
    /// 0-based lines a blank line goes in front of.
    before: Vec<usize>,
}

impl Pass<'_> {
    /// Marks a blank line between `prev` and `next` when the text has none there.
    fn separate(&mut self, prev: &Node, next: &Node) {
        let Some(line) = self.insertion_line(prev, next) else {
            return;
        };

        self.before.push(line);
    }

    /// The line to put a blank line in front of: the first of the comments attached to `next`.
    /// `None` when `next` shares its line, or a blank line is already there.
    fn insertion_line(&self, prev: &Node, next: &Node) -> Option<usize> {
        if prev.last >= next.first {
            return None;
        }

        let before_next: &str = self.lines.get(next.first)?;

        if !before_next
            .chars()
            .take(next.column)
            .all(char::is_whitespace)
        {
            return None;
        }

        let mut line = next.first;

        while line > prev.last + 1 {
            match self.comment_start(line - 1, prev.last) {
                Some(start) => line = start,
                None => break,
            }
        }

        let above = self.lines.get(line.checked_sub(1)?)?;

        (!above.trim().is_empty()).then_some(line)
    }

    /// If `line` ends a comment that stands alone on its lines, the first line of that comment.
    fn comment_start(&self, line: usize, floor: usize) -> Option<usize> {
        let text = self.lines.get(line)?.trim();

        if text.starts_with("//") {
            return Some(line);
        }

        if !text.ends_with("*/") {
            return None;
        }

        let mut at = line;

        loop {
            if at <= floor {
                return None;
            }

            let text = self.lines.get(at)?.trim();

            if text.contains("/*") {
                return text.starts_with("/*").then_some(at);
            }

            at -= 1;
        }
    }

    fn block(&mut self, stmts: &[Stmt]) {
        let nodes: Vec<Node> = stmts.iter().map(node_of_stmt).collect();
        let mut run = 0;

        for (index, pair) in nodes.windows(2).enumerate() {
            let (prev, next) = (&pair[0], &pair[1]);

            run = if prev.kind == Kind::Let {
                run + 1
            } else {
                0
            };

            let last = index + 2 == nodes.len();
            let short_end = last && !next.multi && matches!(next.kind, Kind::Tail | Kind::Flow);
            let set_apart = prev.kind == Kind::Guard
                || next.kind == Kind::Guard
                || prev.big_item
                || next.big_item
                || (prev.kind == Kind::Let && next.kind != Kind::Let && !(short_end && run == 1))
                || (last && next.kind == Kind::Flow && nodes.len() >= 3);

            let imports = prev.kind == Kind::Import && next.kind == Kind::Import;

            if set_apart && !imports {
                self.separate(prev, next);
            }
        }
    }

    fn items(&mut self, nodes: &[Node]) {
        for pair in nodes.windows(2) {
            let (prev, next) = (&pair[0], &pair[1]);

            let imports = prev.kind == Kind::Import && next.kind == Kind::Import;

            if !imports
                && (prev.multi
                    || next.multi
                    || (prev.kind == Kind::Import) != (next.kind == Kind::Import))
            {
                self.separate(prev, next);
            }
        }
    }
}

fn node_of_stmt(stmt: &Stmt) -> Node {
    let span = stmt.span();

    match stmt {
        Stmt::Local(_) => Node::new(Kind::Let, false, span),
        Stmt::Item(Item::Use(_)) => Node::new(Kind::Import, true, span),
        Stmt::Item(_) => Node::new(Kind::Other, true, span),
        Stmt::Macro(_) => Node::new(Kind::Other, false, span),
        Stmt::Expr(expr, semi) => {
            let kind = match expr {
                Expr::If(_)
                | Expr::Match(_)
                | Expr::Loop(_)
                | Expr::While(_)
                | Expr::ForLoop(_) => Kind::Guard,
                Expr::Return(_) | Expr::Break(_) | Expr::Continue(_) => Kind::Flow,
                _ if semi.is_none() => Kind::Tail,
                _ => Kind::Other,
            };
            let node = Node::new(kind, false, span);

            // Only a multi-line guard is set apart; a one-line `if` is an ordinary statement.
            if kind == Kind::Guard && !node.multi {
                return Node {
                    kind: Kind::Other,
                    ..node
                };
            }

            node
        }
    }
}

/// The keyword (`const`, `static`, `type`) of a declaration whose attributes should not count
/// towards its height.
fn plain_head(item: ItemRef<'_>) -> Option<Span> {
    match item {
        ItemRef::Item(Item::Const(item)) => Some(item.const_token.span),
        ItemRef::Item(Item::Static(item)) => Some(item.static_token.span),
        ItemRef::Item(Item::Type(item)) => Some(item.type_token.span),
        ItemRef::Impl(ImplItem::Const(item)) => Some(item.const_token.span),
        ItemRef::Impl(ImplItem::Type(item)) => Some(item.type_token.span),
        ItemRef::Trait(TraitItem::Const(item)) => Some(item.const_token.span),
        ItemRef::Trait(TraitItem::Type(item)) => Some(item.type_token.span),
        _ => None,
    }
}

#[derive(Clone, Copy)]
enum ItemRef<'a> {
    Item(&'a Item),
    Impl(&'a ImplItem),
    Trait(&'a TraitItem),
}

fn node_of_member(item: ItemRef<'_>, span: Span) -> Node {
    match plain_head(item) {
        Some(head) => Node::with_head(Kind::Other, true, span, head),
        None => Node::new(Kind::Other, true, span),
    }
}

fn node_of_item(item: &Item) -> Node {
    let import = matches!(item, Item::Use(_) | Item::ExternCrate(_))
        || matches!(item, Item::Mod(module) if module.content.is_none());
    let kind = if import {
        Kind::Import
    } else {
        Kind::Other
    };

    match plain_head(ItemRef::Item(item)) {
        Some(head) => Node::with_head(kind, true, item.span(), head),
        None => Node::new(kind, true, item.span()),
    }
}

impl<'ast> Visit<'ast> for Pass<'_> {
    fn visit_file(&mut self, file: &'ast File) {
        let nodes: Vec<Node> = file.items.iter().map(node_of_item).collect();

        self.items(&nodes);
        visit::visit_file(self, file);
    }

    fn visit_item_mod(&mut self, module: &'ast syn::ItemMod) {
        if let Some((_, items)) = &module.content {
            let nodes: Vec<Node> = items.iter().map(node_of_item).collect();

            self.items(&nodes);
        }

        visit::visit_item_mod(self, module);
    }

    fn visit_item_impl(&mut self, item: &'ast syn::ItemImpl) {
        let nodes: Vec<Node> = item
            .items
            .iter()
            .map(|i| node_of_member(ItemRef::Impl(i), i.span()))
            .collect();

        self.items(&nodes);
        visit::visit_item_impl(self, item);
    }

    fn visit_item_trait(&mut self, item: &'ast syn::ItemTrait) {
        let nodes: Vec<Node> = item
            .items
            .iter()
            .map(|i| node_of_member(ItemRef::Trait(i), i.span()))
            .collect();

        self.items(&nodes);
        visit::visit_item_trait(self, item);
    }

    fn visit_item_foreign_mod(&mut self, item: &'ast syn::ItemForeignMod) {
        let nodes: Vec<Node> = item
            .items
            .iter()
            .map(|inner| Node::new(Kind::Other, true, inner.span()))
            .collect();

        self.items(&nodes);
        visit::visit_item_foreign_mod(self, item);
    }

    fn visit_block(&mut self, block: &'ast syn::Block) {
        self.block(&block.stmts);
        visit::visit_block(self, block);
    }
}

#[cfg(test)]
mod tests;
