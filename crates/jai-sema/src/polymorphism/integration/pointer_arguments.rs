//! Describe existing pointer operators without lowering or evaluating either operand.
use crate::overloads::{ArgumentInfo, ArgumentType};
use jai_source::{Diagnostic, Span};
use jai_syntax::BinaryOp;
use jai_types::{IntegerType, ScalarType, TypeId, TypeKind, TypeView};

pub(super) fn describe(
    types: &dyn TypeView,
    operation: BinaryOp,
    lhs: &ArgumentInfo,
    rhs: &ArgumentInfo,
    span: Span,
) -> Option<Result<ArgumentInfo, Diagnostic>> {
    let pointer = |value: &ArgumentInfo| match value.ty {
        ArgumentType::Known(ty) if matches!(types.kind(ty), Ok(TypeKind::Pointer(_))) => Some(ty),
        _ => None,
    };
    let left = pointer(lhs);
    let right = pointer(rhs);
    if left.is_none() && right.is_none() {
        return None;
    }
    let result = match operation {
        BinaryOp::Subtract if left.is_some() && right.is_some() => {
            if left == right {
                Ok(types.scalar(ScalarType::Int(IntegerType::S64)))
            } else {
                Err(Diagnostic::new(
                    span,
                    "pointer difference requires matching pointee types",
                ))
            }
        }
        BinaryOp::Add | BinaryOp::Subtract => match (left, right) {
            (Some(pointer), None) => offset(types, rhs, pointer, span),
            (None, Some(pointer)) if operation == BinaryOp::Add => {
                offset(types, lhs, pointer, span)
            }
            _ => Err(Diagnostic::new(
                span,
                "pointer arithmetic supports pointer plus or minus an integer",
            )),
        },
        BinaryOp::Equal | BinaryOp::NotEqual => {
            let compatible = match (left, right) {
                (Some(left), Some(right)) => left == right,
                (Some(_), None) => matches!(rhs.ty, ArgumentType::Null),
                (None, Some(_)) => matches!(lhs.ty, ArgumentType::Null),
                _ => false,
            };
            if compatible {
                Ok(types.scalar(ScalarType::Bool))
            } else {
                Err(Diagnostic::new(
                    span,
                    "pointer equality requires matching pointer types or null",
                ))
            }
        }
        BinaryOp::LogicalAnd | BinaryOp::LogicalOr => Ok(types.scalar(ScalarType::Bool)),
        _ => Err(Diagnostic::new(
            span,
            "pointer arithmetic supports pointer plus or minus an integer",
        )),
    };
    Some(result.map(ArgumentInfo::typed))
}

fn offset(
    types: &dyn TypeView,
    value: &ArgumentInfo,
    pointer: TypeId,
    span: Span,
) -> Result<TypeId, Diagnostic> {
    let preserves_range = match value.ty {
        ArgumentType::Known(ty) => matches!(
            types.kind(ty),
            Ok(TypeKind::Integer(integer)) if IntegerType::S64.contains(*integer)
        ),
        ArgumentType::WeakInteger {
            minimum,
            maximum,
        } => minimum >= IntegerType::S64.min() && maximum <= IntegerType::S64.max(),
        _ => false,
    };
    if preserves_range {
        Ok(pointer)
    } else {
        Err(Diagnostic::new(
            span,
            "implicit integer conversion does not preserve the source type's entire range",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_types::TypeRegistry;

    fn result(
        types: &dyn TypeView,
        operation: BinaryOp,
        lhs: &ArgumentInfo,
        rhs: &ArgumentInfo,
    ) -> Result<TypeId, Diagnostic> {
        let result = describe(types, operation, lhs, rhs, Span::default()).unwrap()?;
        let ArgumentType::Known(ty) = result.ty else {
            panic!("pointer operator has a concrete result type");
        };
        assert!(result.constant.is_none());
        Ok(ty)
    }

    #[test]
    fn offsets_keep_the_pointer_type_and_operand_order_without_constant_execution() {
        let mut types = TypeRegistry::new();
        let pointer = types
            .pointer(types.scalar(ScalarType::Int(IntegerType::U8)))
            .unwrap();
        let ptr = ArgumentInfo::typed(pointer);
        let strong = ArgumentInfo::typed(types.scalar(ScalarType::Int(IntegerType::S32)));
        let weak = ArgumentInfo {
            ty: ArgumentType::WeakInteger {
                minimum: -2,
                maximum: 10,
            },
            constant: None,
        };
        for (operation, left, right) in [
            (BinaryOp::Add, &ptr, &strong),
            (BinaryOp::Subtract, &ptr, &weak),
            (BinaryOp::Add, &weak, &ptr),
        ] {
            assert_eq!(result(&types, operation, left, right).unwrap(), pointer);
        }
        assert!(result(&types, BinaryOp::Subtract, &weak, &ptr).is_err());
        let void_pointer = types.pointer(types.void()).unwrap();
        assert_eq!(
            result(
                &types,
                BinaryOp::Add,
                &ArgumentInfo::typed(void_pointer),
                &strong,
            )
            .unwrap(),
            void_pointer
        );
    }

    #[test]
    fn pointer_difference_and_null_comparison_retain_exact_type_rules() {
        let mut types = TypeRegistry::new();
        let first = types
            .pointer(types.scalar(ScalarType::Int(IntegerType::U8)))
            .unwrap();
        let second = types
            .pointer(types.scalar(ScalarType::Int(IntegerType::S32)))
            .unwrap();
        let a = ArgumentInfo::typed(first);
        let b = ArgumentInfo::typed(second);
        assert_eq!(
            result(&types, BinaryOp::Subtract, &a, &a).unwrap(),
            types.scalar(ScalarType::Int(IntegerType::S64))
        );
        assert!(result(&types, BinaryOp::Subtract, &a, &b).is_err());
        assert!(result(&types, BinaryOp::Equal, &a, &b).is_err());
        let null = ArgumentInfo {
            ty: ArgumentType::Null,
            constant: None,
        };
        assert_eq!(
            result(&types, BinaryOp::NotEqual, &a, &null).unwrap(),
            types.scalar(ScalarType::Bool)
        );
    }

    #[test]
    fn offsets_reject_noninteger_and_nonpreserving_ranges() {
        let mut types = TypeRegistry::new();
        let pointer = types
            .pointer(types.scalar(ScalarType::Int(IntegerType::U8)))
            .unwrap();
        let ptr = ArgumentInfo::typed(pointer);
        for invalid in [
            ArgumentInfo::typed(types.scalar(ScalarType::Int(IntegerType::U64))),
            ArgumentInfo::typed(types.scalar(ScalarType::Bool)),
            ArgumentInfo {
                ty: ArgumentType::WeakInteger {
                    minimum: 0,
                    maximum: i128::from(u64::MAX),
                },
                constant: None,
            },
        ] {
            assert!(result(&types, BinaryOp::Add, &ptr, &invalid).is_err());
        }
        assert!(result(&types, BinaryOp::Add, &ptr, &ptr).is_err());
        let ordinary = ArgumentInfo::typed(types.scalar(ScalarType::Int(IntegerType::S32)));
        assert!(describe(&types, BinaryOp::Add, &ordinary, &ordinary, Span::default()).is_none());
    }
}
