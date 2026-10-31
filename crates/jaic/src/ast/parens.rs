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

    /// The expression reads as one unit: a name, a literal, a member path, or a call of one.
    /// Another expression takes it as an operand without parentheses around it.
    pub fn is_atomic(&self) -> bool {
        fn path(e: &Expr) -> bool {
            match &e.kind {
                ExprKind::Ident(_) => true,
                ExprKind::Member(base, _) => !base.parenthesized() && path(base),
                _ => false,
            }
        }
        match &self.kind {
            ExprKind::Int(..)
            | ExprKind::Float(..)
            | ExprKind::Str(_)
            | ExprKind::Bool(_)
            | ExprKind::Null
            | ExprKind::Context => true,
            ExprKind::Call {
                callee, ..
            } => !callee.parenthesized() && path(callee),
            _ => path(self),
        }
    }

    /// [`Expr::extent`] without the parentheses written around the expression itself.
    pub fn body_extent(&self) -> Span {
        walk::body_extent(walk::Node::Expr(self))
    }
}
