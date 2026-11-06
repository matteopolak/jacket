use super::{CmpOp, Expr, Ty, comparison_constant};

pub(super) const MAX_NODES: usize = 128;
pub(super) const MAX_DEPTH: usize = 16;

#[derive(Clone, Copy)]
pub(super) struct Shape {
    pub nodes: usize,
    pub depth: usize,
}

impl Shape {
    fn parent(self, other: Option<Self>) -> Option<Self> {
        let nodes = self
            .nodes
            .checked_add(other.map_or(0, |s| s.nodes))?
            .checked_add(1)?;
        let depth = self
            .depth
            .max(other.map_or(0, |s| s.depth))
            .checked_add(1)?;
        (nodes <= MAX_NODES && depth <= MAX_DEPTH).then_some(Self {
            nodes,
            depth,
        })
    }
}

impl Expr {
    /// Check structural cost before allocating or copying a composed tree.
    pub(super) fn shape(&self) -> Option<Shape> {
        match self {
            Expr::Compare(a, ..) | Expr::Not(a) => a.shape()?.parent(None),
            Expr::And(a, b) | Expr::Or(a, b) => a.shape()?.parent(Some(b.shape()?)),
            _ => Some(Shape {
                nodes: 1,
                depth: 1,
            }),
        }
    }

    pub(super) fn not(source: Option<&Expr>) -> Self {
        let source = source.unwrap_or(&Expr::Unknown);
        if source == &Expr::Unknown || source.shape().and_then(|s| s.parent(None)).is_none() {
            return Expr::Unknown;
        }
        Expr::Not(Box::new(source.clone()))
    }

    pub(super) fn compare(source: Option<&Expr>, op: CmpOp, constant: u64, ty: Ty) -> Self {
        let source = source.unwrap_or(&Expr::Unknown);
        if comparison_constant(ty, op, constant).is_none()
            || source == &Expr::Unknown
            || source.shape().and_then(|s| s.parent(None)).is_none()
        {
            return Expr::Unknown;
        }
        Expr::Compare(Box::new(source.clone()), op, constant, ty)
    }

    pub(super) fn and(left: Option<&Expr>, right: Option<&Expr>) -> Self {
        Self::binary(left, right, true)
    }

    pub(super) fn or(left: Option<&Expr>, right: Option<&Expr>) -> Self {
        Self::binary(left, right, false)
    }

    fn binary(left: Option<&Expr>, right: Option<&Expr>, and: bool) -> Self {
        let left = left.unwrap_or(&Expr::Unknown);
        let right = right.unwrap_or(&Expr::Unknown);
        if left == &Expr::Unknown && right == &Expr::Unknown {
            return Expr::Unknown;
        }
        let fits = left
            .shape()
            .and_then(|l| l.parent(Some(right.shape()?)))
            .is_some();
        if !fits {
            return Expr::Unknown;
        }
        // Both operands remain borrowed until the complete parent cost fits.
        if and {
            Expr::And(Box::new(left.clone()), Box::new(right.clone()))
        } else {
            Expr::Or(Box::new(left.clone()), Box::new(right.clone()))
        }
    }
}
