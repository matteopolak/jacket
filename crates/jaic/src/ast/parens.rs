//! Parentheses around an expression: where they are, and the source range an expression covers
//! once they are counted.
use super::{Expr, ExprKind, walk};
use crate::source::Span;

/// How far the outermost parentheses around an expression reach past its span, in bytes. Two
/// small distances rather than a [`Span`], so an expression stays small; a gap wider than
/// 65535 bytes (a huge comment inside the parentheses) is cut short.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Parens {
    before: u16,
    after: std::num::NonZeroU16,
}

impl Parens {
    /// The parentheses that run from `open` to `close` around the expression spanning `inner`.
    pub fn new(open: Span, close: Span, inner: Span) -> Self {
        let gap = |a: u32, b: u32| a.saturating_sub(b).min(u32::from(u16::MAX)) as u16;
        Self {
            before: gap(inner.start, open.start),
            after: std::num::NonZeroU16::new(gap(close.end, inner.end).max(1)).unwrap(),
        }
    }
}

impl Expr {
    /// The expression was written inside parentheses.
    pub fn parenthesized(&self) -> bool {
        self.parens.is_some()
    }

    /// The span with the parentheses written around it.
    pub fn outer_span(&self) -> Span {
        match self.parens {
            Some(p) => Span {
                start: self.span.start - u32::from(p.before).min(self.span.start),
                end: self.span.end + u32::from(p.after.get()),
                ..self.span
            },
            None => self.span,
        }
    }

    /// The full source range of the expression: the parentheses around it, and those around
    /// its first and last operands that its span leaves out (`(a + b) * c` spans `a + b) * c`).
    pub fn extent(&self) -> Span {
        walk::extent(walk::Node::Expr(self))
    }

    /// The expression reads as one unit: a name, a literal, or a member path, call or index
    /// built on one. A prefix or postfix operator, or another expression, takes it as an
    /// operand without parentheses around it. A parenthesized link in the chain (`(a).b`) is
    /// not atomic.
    pub fn is_atomic(&self) -> bool {
        self.reads_as_unit(false)
    }

    /// The whole written expression ([`Expr::extent`], parentheses included) reads as one
    /// unit: [`Expr::is_atomic`], but a parenthesized expression, or a parenthesized link in
    /// the chain (`(a).b`), counts, since the text a rewrite replaces covers those parentheses.
    pub fn is_atomic_written(&self) -> bool {
        self.reads_as_unit(true)
    }

    fn reads_as_unit(&self, parens_count: bool) -> bool {
        if parens_count && self.parenthesized() {
            return true;
        }
        match &self.kind {
            ExprKind::Int(..)
            | ExprKind::Float(..)
            | ExprKind::Str(_)
            | ExprKind::Bool(_)
            | ExprKind::Null
            | ExprKind::Context
            | ExprKind::Ident(_) => true,
            ExprKind::Member(base, _)
            | ExprKind::Index(base, _)
            | ExprKind::Call {
                callee: base, ..
            } => (parens_count || !base.parenthesized()) && base.reads_as_unit(parens_count),
            _ => false,
        }
    }

    /// [`Expr::extent`] without the parentheses written around the expression itself.
    pub fn body_extent(&self) -> Span {
        walk::body_extent(walk::Node::Expr(self))
    }
}
