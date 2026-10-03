//! Declaration defaults cast only after their declaration supplies a type.
use super::*;

impl Defaults<'_, '_> {
    pub(super) fn inferred_cast_constant(
        &mut self,
        file: FileInstanceId,
        value: &Expression,
        target: TypeId,
        mode: jai_types::CastMode,
        span: Span,
    ) -> Result<TypedConstant, LocatedDiagnostic> {
        if matches!(mode, jai_types::CastMode::Force(_)) {
            return Err(located(
                self.graph,
                file,
                Diagnostic::new(
                    span,
                    "storage cast declaration defaults require target-layout VM constant materialization",
                ),
            ));
        }
        let kind = self
            .types
            .kind(target)
            .map_err(|error| located(self.graph, file, Diagnostic::new(span, error.to_string())))?
            .clone();
        if let ExpressionKind::InferredCast { mode, value } = &value.kind {
            return self.inferred_cast_constant(file, value, target, *mode, span);
        }
        if matches!(kind, TypeKind::Pointer(_))
            && matches!(
                value.kind,
                ExpressionKind::Integer(_)
                    | ExpressionKind::Character(_)
                    | ExpressionKind::Unary(_, _)
                    | ExpressionKind::Cast(_, _, _)
                    | ExpressionKind::TypeCast { .. }
                    | ExpressionKind::Binary(_, _, _)
            )
        {
            return self.native_pointer_constant(file, value, target, mode, span);
        }
        if matches!(kind, TypeKind::Distinct(_)) {
            let base = self
                .types
                .distinct_definition(target)
                .map_err(|error| {
                    located(self.graph, file, Diagnostic::new(span, error.to_string()))
                })?
                .representation;
            let value = self.inferred_cast_constant(file, value, base, mode, span)?;
            return Ok(TypedConstant {
                ty: target,
                kind: ConstantKind::Distinct(Box::new(value)),
            });
        }
        if mode == jai_types::CastMode::Truncate
            && !matches!(
                kind,
                TypeKind::Integer(_) | TypeKind::Enum(_) | TypeKind::Pointer(_)
            )
        {
            return Err(located(
                self.graph,
                file,
                Diagnostic::new(
                    span,
                    "trunc cast is supported only for integer and pointer representations",
                ),
            ));
        }
        let scalar = match kind {
            TypeKind::Integer(integer) => Some(ScalarType::Int(integer)),
            TypeKind::Bool => Some(ScalarType::Bool),
            TypeKind::Enum(_) => Some(ScalarType::Int(
                self.types.enum_definition(target).unwrap().representation,
            )),
            _ => None,
        };
        if let Some(scalar) = scalar {
            let expression = Expression {
                span,
                kind: ExpressionKind::Cast(mode, scalar, Box::new(value.clone())),
            };
            let value = self.scalar(file, &expression)?;
            let kind = match (kind, value) {
                (TypeKind::Integer(_), ConstantValue::Int(value)) => ConstantKind::Int(value),
                (TypeKind::Bool, ConstantValue::Bool(value)) => ConstantKind::Bool(value),
                (TypeKind::Enum(_), ConstantValue::Int(value)) => ConstantKind::Enum(value),
                _ => {
                    return Err(located(
                        self.graph,
                        file,
                        Diagnostic::new(span, "contextual cast produced a different scalar type"),
                    ));
                }
            };
            return Ok(TypedConstant { ty: target, kind });
        }
        if let TypeKind::Float(float) = kind {
            let expression = Expression {
                span,
                kind: ExpressionKind::TypeCast {
                    mode,
                    ty: syntax::TypeSyntax::Builtin(syntax::BuiltinType::Float(float)),
                    value: Box::new(value.clone()),
                },
            };
            return self.expression(file, &expression, target);
        }
        self.expression(file, value, target)
    }
}
