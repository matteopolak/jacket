//! Namespace constants retain aggregate shapes and checked callable identities.
use super::*;

impl<F> TypeResolver<'_, '_, F>
where
    F: FnMut(FileInstanceId, &syntax::Expression) -> Result<ScalarConstant, LocatedDiagnostic>,
{
    pub(super) fn constant_member(
        &mut self,
        file: FileInstanceId,
        constant: &syntax::ConstantDeclaration,
        scope: &Substitution,
    ) -> TypeResult<BakedValue> {
        if let Some(annotation) = &constant.ty {
            let expected = self.resolve(file, annotation, Some(scope), constant.span)?;
            return self.baked(file, &constant.initializer, expected, Some(scope));
        }
        if constant.ty.is_none() {
            if let Some(value) = expression_path(&constant.initializer).and_then(|path| {
                member_value(
                    self.graph,
                    file,
                    self.nominals,
                    self.records,
                    Some(scope),
                    &path,
                    constant.span,
                )
            }) {
                return Ok(value);
            }
            if matches!(
                constant.initializer.kind,
                syntax::ExpressionKind::StructLiteral(_)
                    | syntax::ExpressionKind::PositionalStructLiteral(_)
                    | syntax::ExpressionKind::ArrayLiteral(_)
                    | syntax::ExpressionKind::String(_)
                    | syntax::ExpressionKind::HereString(_)
                    | syntax::ExpressionKind::TypeCast { .. }
            ) {
                let expected = self.inferred_default_type(file, &constant.initializer, scope)?;
                return self.baked(file, &constant.initializer, expected, Some(scope));
            }
            if let Some(syntax) = type_expression(&constant.initializer) {
                match self.resolve(file, &syntax, Some(scope), constant.span) {
                    Ok(ty) => return Ok(BakedValue::Type(ty)),
                    Err(pending @ TypeFailure::Pending(_)) => return Err(pending),
                    Err(TypeFailure::Diagnostic(_)) => {}
                }
            }
        }
        let value = self.scalar(file, &constant.initializer, Some(scope))?;
        let ty = value.type_id(self.types);
        let kind = match value {
            ScalarConstant::Literal(value) => jai_ir::ConstantKind::Int(
                jai_types::Integer::checked(jai_types::IntegerType::S64, value).ok_or_else(
                    || {
                        failure(
                            self.graph,
                            file,
                            Diagnostic::new(
                                constant.span,
                                "member constant exceeds default integer range",
                            ),
                        )
                    },
                )?,
            ),
            ScalarConstant::Int(value) => jai_ir::ConstantKind::Int(value),
            ScalarConstant::Bool(value) => jai_ir::ConstantKind::Bool(value),
            ScalarConstant::Float(value) => jai_ir::ConstantKind::Float(value),
            ScalarConstant::WeakFloat(value) => jai_ir::ConstantKind::Float(
                value
                    .round(value.default_type(), constant.span)
                    .map_err(|error| failure(self.graph, file, error))?,
            ),
        };
        BakedValue::runtime(jai_ir::ConstantValue { ty, kind }, self.types).map_err(|error| {
            failure(
                self.graph,
                file,
                Diagnostic::new(constant.span, error.to_string()),
            )
        })
    }
}
