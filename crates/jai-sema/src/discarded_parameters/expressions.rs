//! Validate builtin expression domains while keeping discarded sources unevaluated.
use super::*;
use crate::overloads::{ArgumentInfo, ArgumentType};
use jai_types::TypeKind;

impl Resolver<'_> {
    pub(crate) fn validate_discarded_expression(
        &mut self,
        expression: &syntax::Expression,
    ) -> Result<(), Diagnostic> {
        self.validate_discarded_expression_inner(expression, 0)
    }
    fn validate_discarded_expression_inner(
        &mut self,
        expression: &syntax::Expression,
        depth: usize,
    ) -> Result<(), Diagnostic> {
        use syntax::ExpressionKind as E;
        if depth >= crate::constant_limits::MAX_CONSTANT_DEPTH {
            return Err(Diagnostic::new(
                expression.span,
                "discarded expression exceeds source depth",
            ));
        }
        let span = expression.span;
        match &expression.kind {
            E::CompileTime(syntax::CompileTimeRun {
                body:
                    syntax::CompileTimeBody::Procedure {
                        result,
                        body,
                    },
                ..
            }) => {
                self.check_discarded_run_body(result, body, span)?;
            }
            E::Binary(operation, left, right) => {
                self.validate_discarded_expression_inner(left, depth + 1)?;
                self.validate_discarded_expression_inner(right, depth + 1)?;
                if let Some(result) = self.describe_operator_expression(expression) {
                    result?;
                    return Ok(());
                }
                let left = self.describe_argument(left)?;
                let right = self.describe_argument(right)?;
                self.validate_discarded_binary(*operation, &left, &right, span)?;
            }
            E::Unary(operation, value) => {
                self.validate_discarded_expression_inner(value, depth + 1)?;
                if let Some(result) = self.describe_operator_expression(expression) {
                    result?;
                    return Ok(());
                }
                let value = self.describe_argument(value)?;
                if *operation == UnaryOp::LogicalNot {
                    self.check_discarded_condition(&value, span)?;
                } else {
                    let ty = self.argument_type(&value, span)?;
                    if !(matches!(self.types.kind(ty), Ok(TypeKind::Integer(_)))
                        || (*operation != UnaryOp::Complement
                            && matches!(self.types.kind(ty), Ok(TypeKind::Float(_))))
                        || (*operation == UnaryOp::Complement && self.enum_is_flags(ty)))
                    {
                        return Err(Diagnostic::new(
                            span,
                            "unary operator is not defined for this source type",
                        ));
                    }
                }
            }
            E::Cast(mode, target, value) => {
                self.validate_discarded_expression_inner(value, depth + 1)?;
                let info = self.describe_argument(value)?;
                let cast = ArgumentInfo::contextual_cast(*mode, info);
                overloads::contextual_conversion(
                    self.types,
                    self,
                    self.types.scalar(*target),
                    &cast.ty,
                    span,
                )?;
            }
            E::TypeCast {
                mode,
                ty,
                value,
            } => {
                self.validate_discarded_expression_inner(value, depth + 1)?;
                let target = self.preview_annotation(ty, span)?;
                let info = self.describe_argument(value)?;
                let cast = ArgumentInfo::contextual_cast(*mode, info);
                overloads::contextual_conversion(self.types, self, target, &cast.ty, span)?;
            }
            E::Conditional(value) => {
                self.validate_discarded_expression_inner(&value.condition, depth + 1)?;
                let condition = self.describe_argument(&value.condition)?;
                self.check_discarded_condition(&condition, value.condition.span)?;
                if let Some(value) = value.explicit_then() {
                    self.validate_discarded_expression_inner(value, depth + 1)?;
                }
                if let Some(value) = &value.else_value {
                    self.validate_discarded_expression_inner(value, depth + 1)?;
                }
            }
            E::Index {
                base,
                index,
            } => {
                self.validate_discarded_expression_inner(base, depth + 1)?;
                self.validate_discarded_expression_inner(index, depth + 1)?;
                if let Some(result) = self.describe_operator_expression(expression) {
                    result?;
                    return Ok(());
                }
                let info = self.describe_argument(index)?;
                let ty = self.argument_type(&info, index.span)?;
                if !matches!(self.types.kind(ty), Ok(TypeKind::Integer(_))) {
                    return Err(Diagnostic::new(
                        index.span,
                        "index requires an integer expression",
                    ));
                }
            }
            E::Call(_, args) | E::QualifiedCall(_, args) => {
                for arg in args {
                    self.validate_discarded_expression_inner(&arg.value, depth + 1)?;
                }
            }
            E::IndirectCall {
                callee,
                args,
            } => {
                self.validate_discarded_expression_inner(callee, depth + 1)?;
                for arg in args {
                    self.validate_discarded_expression_inner(&arg.value, depth + 1)?;
                }
            }
            E::ContextCall {
                callee,
                args,
                overrides,
            } => {
                self.validate_discarded_expression_inner(callee, depth + 1)?;
                for argument in args.iter().chain(overrides) {
                    self.validate_discarded_expression_inner(&argument.value, depth + 1)?;
                }
            }
            E::CompileTime(syntax::CompileTimeRun {
                body: syntax::CompileTimeBody::Expression(value),
                ..
            })
            | E::CallHint {
                call: value, ..
            }
            | E::InferredCast {
                value, ..
            }
            | E::Dereference(value)
            | E::Member {
                base: value, ..
            } => {
                self.validate_discarded_expression_inner(value, depth + 1)?;
            }
            E::AddressOf(value) => {
                self.validate_discarded_expression_inner(value, depth + 1)?;
                if let Some(result) = self.describe_operator_expression(expression) {
                    result?;
                    return Ok(());
                }
                let info = self.describe_argument(value)?;
                if !matches!(
                    info.constant,
                    Some(overloads::ConstantArgument::Value(
                        crate::polymorphism::BakedValue::Type(_)
                    ))
                ) {
                    self.check_discarded_addressable(value)?;
                }
            }
            E::ArrayLiteral(value) => {
                for element in &value.elements {
                    self.validate_discarded_expression_inner(element, depth + 1)?;
                }
            }
            E::StructLiteral(value) => {
                for field in &value.fields {
                    for index in crate::modules::aggregates::promoted_literals::target_expressions::index_expressions(&field.target)? {
                        self.validate_discarded_expression_inner(index, depth + 1)?;
                    }
                    self.validate_discarded_expression_inner(&field.value, depth + 1)?;
                }
            }
            E::PositionalStructLiteral(value) => {
                for element in &value.values {
                    self.validate_discarded_expression_inner(element, depth + 1)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
    pub(super) fn check_discarded_addressable(
        &mut self,
        source: &syntax::Expression,
    ) -> Result<TypeId, Diagnostic> {
        use syntax::ExpressionKind as E;
        match &source.kind {
            E::Name(name)
            | E::QualifiedName(syntax::NamePath {
                root: name,
                members: _,
            }) => {
                let path = match &source.kind {
                    E::QualifiedName(path) => path.clone(),
                    _ => syntax::NamePath {
                        root: *name,
                        members: vec![],
                    },
                };
                match self.lookup_path(&path, source.span)? {
                    Binding::Storage(storage) => {
                        self.reject_iteration_write(storage.place(), source.span)?;
                        Ok(storage.place().ty())
                    }
                    Binding::LambdaPreview(crate::short_lambdas::PreviewBinding::Parameter(ty)) => {
                        Ok(ty)
                    }
                    Binding::Discarded(_) => Err(Diagnostic::new(
                        source.span,
                        "#discard parameter cannot be read",
                    )),
                    _ => Err(Diagnostic::new(source.span, "address requires storage")),
                }
            }
            E::Dereference(pointer) => {
                let info = self.describe_argument(pointer)?;
                let ty = self.argument_type(&info, pointer.span)?;
                match self.types.kind(ty) {
                    Ok(TypeKind::Pointer(element)) if *element != self.types.void() => Ok(*element),
                    _ => Err(Diagnostic::new(
                        source.span,
                        "dereference requires a sized pointee type",
                    )),
                }
            }
            E::Member {
                base,
                member,
            } => {
                let info = self.describe_argument(base)?;
                let mut ty = self.argument_type(&info, base.span)?;
                if let Ok(TypeKind::Pointer(element)) = self.types.kind(ty) {
                    ty = *element;
                } else {
                    self.check_discarded_addressable(base)?;
                }
                self.record_metadata(ty, source.span)?
                    .fields
                    .iter()
                    .find(|field| field.name == Some(*member))
                    .map(|field| field.ty)
                    .ok_or_else(|| Diagnostic::new(source.span, "unknown addressable record field"))
            }
            E::Index {
                base,
                index,
            } => {
                let info = self.describe_argument(base)?;
                let ty = self.argument_type(&info, base.span)?;
                let element = match self.types.kind(ty) {
                    Ok(TypeKind::Pointer(element)) if *element != self.types.void() => *element,
                    Ok(
                        TypeKind::FixedArray {
                            element, ..
                        }
                        | TypeKind::Slice(element)
                        | TypeKind::DynamicArray(element),
                    ) => {
                        let element = *element;
                        self.check_discarded_addressable(base)?;
                        element
                    }
                    Ok(TypeKind::String) => {
                        self.check_discarded_addressable(base)?;
                        self.types.scalar(ScalarType::Int(IntegerType::U8))
                    }
                    _ => {
                        return Err(Diagnostic::new(
                            source.span,
                            "index requires addressable sequence or sized pointer",
                        ));
                    }
                };
                let info = self.describe_argument(index)?;
                let ty = self.argument_type(&info, index.span)?;
                if !matches!(self.types.kind(ty), Ok(TypeKind::Integer(_))) {
                    return Err(Diagnostic::new(
                        index.span,
                        "index requires an integer expression",
                    ));
                }
                Ok(element)
            }
            _ => Err(Diagnostic::new(source.span, "address requires storage")),
        }
    }
    pub(crate) fn check_discarded_condition(
        &self,
        value: &ArgumentInfo,
        span: Span,
    ) -> Result<(), Diagnostic> {
        if matches!(value.ty, ArgumentType::Null) {
            return Ok(());
        }
        let ty = self.argument_type(value, span)?;
        if matches!(
            self.types.kind(ty),
            Ok(TypeKind::Bool | TypeKind::Integer(_) | TypeKind::Float(_) | TypeKind::Pointer(_))
        ) || self.enum_is_flags(ty)
        {
            Ok(())
        } else {
            Err(Diagnostic::new(
                span,
                "source type cannot supply a runtime condition",
            ))
        }
    }
    fn validate_discarded_binary(
        &mut self,
        operation: BinaryOp,
        left: &ArgumentInfo,
        right: &ArgumentInfo,
        span: Span,
    ) -> Result<(), Diagnostic> {
        if matches!(operation, BinaryOp::LogicalAnd | BinaryOp::LogicalOr) {
            self.check_discarded_condition(left, span)?;
            return self.check_discarded_condition(right, span);
        }
        let equality = matches!(operation, BinaryOp::Equal | BinaryOp::NotEqual);
        let left_ty = self.argument_type(left, span).ok();
        let right_ty = self.argument_type(right, span).ok();
        let kind = |ty: Option<TypeId>| ty.and_then(|ty| self.types.kind(ty).ok());
        let left_kind = kind(left_ty);
        let right_kind = kind(right_ty);
        if matches!(left_kind, Some(TypeKind::Enum(_)))
            || matches!(right_kind, Some(TypeKind::Enum(_)))
        {
            // The shared pure enum descriptor checks representation and flags domains.
            return Ok(());
        }
        if equality
            && ((matches!(left_kind, Some(TypeKind::Bool)) && left_ty == right_ty)
                || (matches!(left_kind, Some(TypeKind::String | TypeKind::Type))
                    && left_ty == right_ty)
                || (matches!(
                    left_kind,
                    Some(TypeKind::Procedure(_) | TypeKind::Pointer(_))
                ) && (left_ty == right_ty || matches!(right.ty, ArgumentType::Null)))
                || (matches!(
                    right_kind,
                    Some(TypeKind::Procedure(_) | TypeKind::Pointer(_))
                ) && matches!(left.ty, ArgumentType::Null))
                || (matches!(left.ty, ArgumentType::Null)
                    && matches!(right.ty, ArgumentType::Null)))
        {
            return Ok(());
        }
        if matches!(operation, BinaryOp::Add | BinaryOp::Subtract) {
            let left_pointer = matches!(left_kind, Some(TypeKind::Pointer(_)));
            let right_pointer = matches!(right_kind, Some(TypeKind::Pointer(_)));
            if (left_pointer && matches!(right_kind, Some(TypeKind::Integer(_))))
                || (operation == BinaryOp::Add
                    && right_pointer
                    && matches!(left_kind, Some(TypeKind::Integer(_))))
                || (operation == BinaryOp::Subtract && left_pointer && left_ty == right_ty)
            {
                return Ok(());
            }
        }
        let numeric = |kind: Option<&TypeKind>| {
            matches!(
                kind,
                Some(TypeKind::Integer(_) | TypeKind::Float(_) | TypeKind::Distinct(_))
            )
        };
        if !numeric(left_kind) || !numeric(right_kind) {
            return Err(Diagnostic::new(
                span,
                "binary operator is not defined for these source types",
            ));
        }
        Ok(())
    }
}
