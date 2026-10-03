//! Discarded source arguments prove compatibility without producing expressions.
use super::*;
mod bodies;
mod expressions;

impl Resolver<'_> {
    pub(crate) fn check_discarded_c_variadic_argument(
        &mut self,
        expression: &syntax::Expression,
    ) -> Result<(), Diagnostic> {
        self.validate_discarded_expression(expression)?;
        let info = self.describe_argument(expression)?;
        let ty = self.argument_type(&info, expression.span)?;
        if matches!(
            self.types.kind(ty),
            Ok(jai_types::TypeKind::Integer(_)
                | jai_types::TypeKind::Float(_)
                | jai_types::TypeKind::Bool
                | jai_types::TypeKind::Pointer(_)
                | jai_types::TypeKind::Enum(_))
        ) {
            Ok(())
        } else {
            Err(Diagnostic::new(
                expression.span,
                "C variadic argument requires an ABI scalar or pointer",
            ))
        }
    }
    pub(crate) fn bind_discarded_parameter(
        &mut self,
        name: Symbol,
        ty: TypeId,
    ) -> Result<(), Diagnostic> {
        self.bind_name(name, Binding::Discarded(ty))
    }

    pub(crate) fn check_discarded_argument(
        &mut self,
        expression: &syntax::Expression,
        expected: TypeId,
    ) -> Result<(), Diagnostic> {
        self.validate_discarded_expression(expression)?;
        if let syntax::ExpressionKind::PositionalStructLiteral(literal) = &expression.kind {
            let actual = literal
                .ty
                .as_ref()
                .map(|path| self.local_type_name(path, expression.span))
                .transpose()?
                .unwrap_or(expected);
            let metadata = self.record_metadata(actual, expression.span)?;
            if literal.values.len() > metadata.fields.len()
                || (metadata.kind == jai_types::RecordKind::Union && literal.values.len() != 1)
            {
                return Err(Diagnostic::new(
                    expression.span,
                    "positional record literal does not match its checked fields",
                ));
            }
            for (value, field) in literal.values.iter().zip(&metadata.fields) {
                self.check_discarded_argument(value, field.ty)?;
            }
            overloads::contextual_conversion(
                self.types,
                self,
                expected,
                &overloads::ArgumentType::Known(actual),
                expression.span,
            )?;
            return Ok(());
        }
        if let syntax::ExpressionKind::ShortLambda(source) = &expression.kind {
            self.preview_short_lambda(source, expected, expression.span)?;
            return Ok(());
        }
        let path = match &expression.kind {
            syntax::ExpressionKind::Name(root) => Some(syntax::NamePath {
                root: *root,
                members: vec![],
            }),
            syntax::ExpressionKind::QualifiedName(path) => Some(path.clone()),
            _ => None,
        };
        if let Some(path) = path
            && self
                .describe_contextual_named_short_lambda(&path, &[expected], expression.span)?
                .is_some()
        {
            return Ok(());
        }
        let info = if matches!(expression.kind, syntax::ExpressionKind::CallerLocation) {
            overloads::ArgumentInfo::typed(self.caller_location_type(expression.span)?)
        } else {
            self.describe_argument(expression)?
        };
        overloads::contextual_conversion(self.types, self, expected, &info.ty, expression.span)?;
        Ok(())
    }
}
