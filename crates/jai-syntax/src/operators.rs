//! Translate syntax operators to shared typed domain operators.
use crate::BinaryOp;
use jai_types::{Equality, IntOp, Relation};
impl From<crate::BinaryOp> for jai_types::Operator {
    fn from(op: BinaryOp) -> Self {
        match op {
            BinaryOp::Add => Self::Integer(IntOp::Add),
            BinaryOp::Subtract => Self::Integer(IntOp::Subtract),
            BinaryOp::Multiply => Self::Integer(IntOp::Multiply),
            BinaryOp::Divide => Self::Integer(IntOp::Divide),
            BinaryOp::Remainder => Self::Integer(IntOp::Remainder),
            BinaryOp::BitAnd => Self::Integer(IntOp::BitAnd),
            BinaryOp::BitOr => Self::Integer(IntOp::BitOr),
            BinaryOp::BitXor => Self::Integer(IntOp::BitXor),
            BinaryOp::ShiftLeft => Self::Integer(IntOp::ShiftLeft),
            BinaryOp::ShiftRight => Self::Integer(IntOp::ShiftRight),
            BinaryOp::Less => Self::Relation(Relation::Less),
            BinaryOp::LessEqual => Self::Relation(Relation::LessEqual),
            BinaryOp::Greater => Self::Relation(Relation::Greater),
            BinaryOp::GreaterEqual => Self::Relation(Relation::GreaterEqual),
            BinaryOp::Equal => Self::Equality(Equality::Equal),
            BinaryOp::NotEqual => Self::Equality(Equality::NotEqual),
            BinaryOp::LogicalAnd => Self::And,
            BinaryOp::LogicalOr => Self::Or,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use jai_source::SourceMap;

    #[test]
    fn modern_flags_tests_bind_bitwise_masks_before_comparisons_and_logical_operations() {
        let mut sources = SourceMap::default();
        let text = "main :: () { enabled := flags & .POINTER != 0 && mask | .OTHER == combined; }";
        let id = sources.insert("flag-precedence.jai".into(), text.into());
        let parsed = parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap();
        let FileItem::Declaration(FileDeclaration {
            kind: FileDeclarationKind::Procedure(procedure),
            ..
        }) = &parsed.items()[0]
        else {
            panic!()
        };
        let StatementKind::Declare(Declaration::Inferred {
            initializer, ..
        }) = &procedure.body[0].kind
        else {
            panic!()
        };
        let ExpressionKind::Binary(BinaryOp::LogicalAnd, left, right) = &initializer.kind else {
            panic!("expected logical combination")
        };
        assert!(
            matches!(&left.kind, ExpressionKind::Binary(BinaryOp::NotEqual, masked, _) if matches!(masked.kind, ExpressionKind::Binary(BinaryOp::BitAnd, _, _)))
        );
        assert!(
            matches!(&right.kind, ExpressionKind::Binary(BinaryOp::Equal, masked, _) if matches!(masked.kind, ExpressionKind::Binary(BinaryOp::BitOr, _, _)))
        );
    }
}
