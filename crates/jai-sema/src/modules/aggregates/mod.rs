//! Typed aggregate construction and scalar field bridges share the checked IR.
use super::*;
use jai_types::{FieldId, TypeKind};
mod instance_members;
mod layout;
mod promoted_literals;
pub(crate) use promoted_literals::concrete_literal;
pub(crate) mod types;
mod variants;
pub(crate) use types::{EnumConstant, Nominals};
mod defaults;
pub(crate) mod parameterized;
pub(super) use defaults::Defaults;

impl Resolver<'_> {
    pub(crate) fn typed_value(
        &self,
        value: ValueExpr,
        ty: TypeId,
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        Ok(
            match self
                .types
                .kind(ty)
                .map_err(|error| Diagnostic::new(span, error.to_string()))?
            {
                TypeKind::Integer(integer) => {
                    Expr::Int(IntExpr::new(*integer, IntExprKind::Value(Box::new(value))))
                }
                TypeKind::Float(float) => Expr::Float(FloatExpr::new(
                    *float,
                    FloatExprKind::Value(Box::new(value)),
                )),
                TypeKind::Bool => Expr::Bool(BoolExpr::Value(Box::new(value))),
                TypeKind::Pointer(_) => Expr::Pointer {
                    ty,
                    value,
                },
                TypeKind::Enum(_) => {
                    let representation = self
                        .types
                        .enum_definition(ty)
                        .map_err(|error| Diagnostic::new(span, error.to_string()))?
                        .representation;
                    Expr::Enum {
                        ty,
                        flags: self.enum_is_flags(ty),
                        representation,
                        value,
                    }
                }
                TypeKind::Type
                | TypeKind::Procedure(_)
                | TypeKind::Distinct(_)
                | TypeKind::Record(_)
                | TypeKind::Any(_)
                | TypeKind::String
                | TypeKind::FixedArray {
                    ..
                }
                | TypeKind::Slice(_)
                | TypeKind::DynamicArray(_) => Expr::Typed {
                    ty,
                    value,
                },
                _ => {
                    return Err(Diagnostic::new(
                        span,
                        "runtime value type is not implemented",
                    ));
                }
            },
        )
    }
    pub(crate) fn expression_type(&self, value: &Expr, span: Span) -> Result<TypeId, Diagnostic> {
        Ok(match value {
            Expr::Code(_) => self.types.code_type(),
            Expr::Type(_) => self.types.meta_type(),
            value @ (Expr::Float(_) | Expr::WeakFloat(_)) => {
                self.types.float(value.default_float_type())
            }
            value @ Expr::WeakConditional(_) if value.has_float() => {
                self.types.float(value.default_float_type())
            }
            Expr::Int(value) => self.types.scalar(ScalarType::Int(value.ty())),
            Expr::Bool(_) => self.types.scalar(ScalarType::Bool),
            Expr::Literal(_) | Expr::WeakConditional(_) => {
                self.types.scalar(ScalarType::Int(IntegerType::S64))
            }
            Expr::Typed {
                ty, ..
            }
            | Expr::Enum {
                ty, ..
            }
            | Expr::Pointer {
                ty, ..
            } => *ty,
            Expr::Null => {
                return Err(Diagnostic::new(
                    span,
                    "null requires a pointer type context",
                ));
            }
            Expr::Void(_)
            | Expr::IndirectVoid {
                ..
            } => {
                return Err(Diagnostic::new(span, "void call cannot supply a value"));
            }
        })
    }
    pub(crate) fn coerce_value(
        &self,
        value: Expr,
        ty: TypeId,
        span: Span,
    ) -> Result<ValueExpr, Diagnostic> {
        let value = self.implicit_variant_base(value, ty, span)?;
        let value = self.implicit_field_value(value, ty, span)?;
        match self
            .types
            .kind(ty)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?
        {
            TypeKind::Integer(integer) => Ok(ValueExpr::Int(value.int_as(*integer, span)?)),
            TypeKind::Float(float) => Ok(ValueExpr::Float(value.float_as(*float, span)?)),
            TypeKind::Bool => Ok(ValueExpr::Bool(value.bool(span)?)),
            TypeKind::Type => match value {
                Expr::Typed {
                    ty: actual,
                    value,
                } if actual == ty => Ok(value),
                Expr::Null => Ok(ValueExpr::Zero(ty)),
                _ => Err(Diagnostic::new(
                    span,
                    "runtime Type value requires a canonical descriptor identity",
                )),
            },
            TypeKind::Pointer(pointee) => match value {
                Expr::Null => Ok(ValueExpr::Zero(ty)),
                Expr::Pointer {
                    ty: actual,
                    value,
                } if actual == ty => Ok(value),
                Expr::Pointer {
                    value, ..
                } if *pointee == self.types.void() => Ok(ValueExpr::PointerCast {
                    value: Box::new(value),
                    ty,
                    mode: CastMode::Checked,
                }),
                Expr::Pointer {
                    ty: actual,
                    value,
                } => self
                    .reflection_pointer_coercion(value, actual, ty, span)?
                    .ok_or_else(|| {
                        Diagnostic::new(span, "pointer value has a different pointee type")
                    }),
                _ => Err(Diagnostic::new(
                    span,
                    "pointer value has a different pointee type",
                )),
            },
            TypeKind::String
            | TypeKind::FixedArray {
                ..
            }
            | TypeKind::Slice(_)
            | TypeKind::DynamicArray(_) => self.sequence_coercion(value, ty, span),
            TypeKind::Distinct(_) => self.variant_coercion(value, ty, span),
            TypeKind::Procedure(_) => match value {
                Expr::Null => Ok(ValueExpr::Zero(ty)),
                Expr::Typed {
                    ty: actual,
                    value,
                } if actual == ty => Ok(value),
                _ => Err(Diagnostic::new(
                    span,
                    "procedure value has a different canonical signature",
                )),
            },
            TypeKind::Record(_) | TypeKind::Any(_) => match value {
                Expr::Typed {
                    ty: actual,
                    value,
                } if actual == ty => Ok(value),
                _ => Err(Diagnostic::new(
                    span,
                    "record value has a different nominal type",
                )),
            },
            TypeKind::Enum(_) => match value {
                Expr::Enum {
                    ty: actual,
                    value,
                    ..
                } if actual == ty => Ok(value),
                Expr::Literal(0) if self.enum_is_flags(ty) => {
                    let representation = self
                        .types
                        .enum_definition(ty)
                        .map_err(|error| Diagnostic::new(span, error.to_string()))?
                        .representation;
                    Ok(ValueExpr::Enum {
                        ty,
                        value: IntegerValue::wrapping(representation, 0),
                    })
                }
                _ => Err(Diagnostic::new(
                    span,
                    "enum value has a different nominal type",
                )),
            },
            _ => Err(Diagnostic::new(
                span,
                "value coercion is not implemented for this type",
            )),
        }
    }
    pub(crate) fn expr_expected(
        &mut self,
        expression: &syntax::Expression,
        ty: TypeId,
    ) -> Result<Expr, Diagnostic> {
        if let syntax::ExpressionKind::String(bytes) = &expression.kind
            && matches!(
                self.types.kind(ty),
                Ok(TypeKind::Pointer(element))
                    if *element == self.types.scalar(ScalarType::Int(IntegerType::U8))
            )
        {
            return self.c_string_literal(bytes, ty, expression.span);
        }
        if let syntax::ExpressionKind::CompileTime(body) = &expression.kind {
            return self.resolve_compile_time_expected(body, expression.span, ty);
        }
        if let syntax::ExpressionKind::AnonymousProcedure(source) = &expression.kind {
            return self.anonymous_procedure_value(source, expression.span, Some(ty));
        }
        if let syntax::ExpressionKind::ShortLambda(source) = &expression.kind {
            return self.short_lambda_value(source, expression.span, Some(ty), None);
        }
        if matches!(self.types.kind(ty), Ok(TypeKind::Procedure(_))) {
            let path = match &expression.kind {
                syntax::ExpressionKind::Name(name) => Some(syntax::NamePath {
                    root: *name,
                    members: vec![],
                }),
                syntax::ExpressionKind::QualifiedName(path) => Some(path.clone()),
                _ => None,
            };
            if let Some(path) = path
                && let Some(value) = self.named_short_lambda_value(&path, ty, expression.span)?
            {
                return Ok(value);
            }
        }
        if let syntax::ExpressionKind::InferredCast {
            mode,
            value,
        } = &expression.kind
        {
            return self.cast_expression(value, ty, *mode, expression.span);
        }
        if crate::inferred_casts::needs_cast_context(expression) {
            match &expression.kind {
                syntax::ExpressionKind::Conditional(source)
                    if matches!(
                        self.types.kind(ty),
                        Ok(TypeKind::Integer(_) | TypeKind::Float(_) | TypeKind::Bool)
                    ) =>
                {
                    return self.contextual_scalar_conditional(source, ty, expression.span);
                }
                syntax::ExpressionKind::Binary(operation, left, right)
                    if matches!(Operator::from(*operation), Operator::Integer(_))
                        && matches!(
                            self.types.kind(ty),
                            Ok(TypeKind::Integer(_)
                                | TypeKind::Float(_)
                                | TypeKind::Enum(_)
                                | TypeKind::Distinct(_))
                        ) =>
                {
                    let left = self.expr_expected(left, ty)?;
                    let right = self.expr_expected(right, ty)?;
                    return self.binary(*operation, left, right, expression.span);
                }
                _ => {}
            }
        }
        if matches!(self.types.kind(ty), Ok(TypeKind::Any(_))) {
            return self.any_expected(expression, ty);
        }
        if let syntax::ExpressionKind::InferredMember(name) = &expression.kind {
            return self.inferred_enum_member(ty, *name, expression.span);
        }
        if Self::contextual_enum_operand(expression)
            && matches!(self.types.kind(ty), Ok(TypeKind::Enum(_)))
        {
            return self.resolve_contextual_enum_operand(expression, ty);
        }
        if let syntax::ExpressionKind::Conditional(conditional) = &expression.kind
            && matches!(
                self.types.kind(ty),
                Ok(TypeKind::Type
                    | TypeKind::Record(_)
                    | TypeKind::Enum(_)
                    | TypeKind::Distinct(_)
                    | TypeKind::Procedure(_)
                    | TypeKind::String
                    | TypeKind::FixedArray { .. }
                    | TypeKind::Slice(_)
                    | TypeKind::DynamicArray(_))
            )
        {
            return self.value_conditional(conditional, ty, expression.span);
        }
        if let syntax::ExpressionKind::Conditional(conditional) = &expression.kind
            && matches!(self.types.kind(ty), Ok(TypeKind::Pointer(_)))
        {
            return self.pointer_conditional(conditional, Some(ty), expression.span);
        }
        if matches!(expression.kind, syntax::ExpressionKind::Null)
            && matches!(self.types.kind(ty), Ok(TypeKind::Pointer(_)))
        {
            return Ok(Expr::Pointer {
                ty,
                value: ValueExpr::Zero(ty),
            });
        }
        if let syntax::ExpressionKind::ArrayLiteral(literal) = &expression.kind {
            return self.array_literal(literal, Some(ty), expression.span);
        }
        if let syntax::ExpressionKind::StructLiteral(literal) = &expression.kind {
            let value =
                self.record_literal(literal, literal.ty.is_none().then_some(ty), expression.span)?;
            return self.implicit_field_value(value, ty, expression.span);
        }
        if let syntax::ExpressionKind::PositionalStructLiteral(literal) = &expression.kind {
            let value = self.positional_record_literal(
                literal,
                literal.ty.is_none().then_some(ty),
                expression.span,
            )?;
            return self.implicit_field_value(value, ty, expression.span);
        }
        let value = self.expr(expression)?;
        let value = if ty == self.types.meta_type() {
            self.runtime_type_expression(value, expression.span)?
        } else {
            value
        };
        let value = self.implicit_field_pointer(value, ty, expression.span)?;
        self.implicit_field_value(value, ty, expression.span)
    }
    pub(crate) fn positional_record_literal(
        &mut self,
        literal: &syntax::PositionalStructLiteral,
        expected: Option<TypeId>,
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        let ty = match &literal.ty {
            Some(path) => self.local_type_name(path, span)?,
            None => expected.ok_or_else(|| {
                Diagnostic::new(
                    span,
                    "positional record literal requires an explicit or contextual type",
                )
            })?,
        };
        if expected.is_some_and(|expected| expected != ty) {
            return Err(Diagnostic::new(
                span,
                "record literal has a different nominal type",
            ));
        }
        if matches!(
            self.types.kind(ty),
            Ok(TypeKind::String | TypeKind::Slice(_) | TypeKind::DynamicArray(_))
        ) {
            return self.positional_sequence_literal(literal, ty, span);
        }
        let record = self.record_metadata(ty, span)?;
        if record.kind != jai_types::RecordKind::Struct {
            return Err(Diagnostic::new(
                span,
                "positional union literals require an explicit alternative",
            ));
        }
        if literal.values.len() > record.fields.len() {
            return Err(Diagnostic::new(
                span,
                "positional record literal has too many values",
            ));
        }
        self.collect_record_field_default_jobs()?;
        self.prepared_positional_record_literal(literal, ty, span)
    }

    pub(crate) fn record_literal(
        &mut self,
        literal: &syntax::StructLiteral,
        expected: Option<TypeId>,
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        if let Some(ty) = expected.filter(|ty| {
            literal.ty.is_none()
                && matches!(
                    self.types.kind(*ty),
                    Ok(TypeKind::String | TypeKind::Slice(_) | TypeKind::DynamicArray(_))
                )
        }) {
            return self.sequence_literal(literal, ty, span);
        }
        let ty = match &literal.ty {
            Some(path) => self.local_type_name(path, span)?,
            None => expected.ok_or_else(|| {
                Diagnostic::new(
                    span,
                    "record literal requires an explicit or contextual type",
                )
            })?,
        };
        self.collect_record_field_default_jobs()?;
        if expected.is_some_and(|expected| expected != ty) {
            return Err(Diagnostic::new(
                span,
                "record literal has a different nominal type",
            ));
        }
        if matches!(
            self.types.kind(ty),
            Ok(TypeKind::String | TypeKind::Slice(_) | TypeKind::DynamicArray(_))
        ) {
            return self.sequence_literal(literal, ty, span);
        }
        if matches!(self.types.kind(ty), Ok(TypeKind::Any(_))) {
            return self.any_literal(literal, ty, span);
        }
        self.promoted_record_literal(literal, ty, span)
    }

    pub(crate) fn member_value(
        &mut self,
        base: Expr,
        member: Symbol,
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        if let Expr::Type(ty) = base {
            if let Some(value) = self.context_constant_value(ty, member, span) {
                return value;
            }
            let binding = self
                .type_namespace_member(ty, member, span)?
                .ok_or_else(|| {
                    Diagnostic::new(
                        span,
                        format!(
                            "type declaration has no namespace member '{}'",
                            self.symbols.name(member)
                        ),
                    )
                })?;
            return self.binding_expression(binding, span);
        }
        if let Expr::Pointer {
            ty,
            value,
        } = base
        {
            let owner = match self.types.kind(ty) {
                Ok(TypeKind::Pointer(owner)) => *owner,
                _ => {
                    return Err(Diagnostic::new(
                        span,
                        "record receiver pointer type is invalid",
                    ));
                }
            };
            if matches!(self.types.kind(owner), Ok(TypeKind::Record(_)))
                && self.optional_field_path(owner, member, span)?.is_none()
                && let Some(binding) = self.instance_namespace_member(owner, member, span)?
            {
                return self.instance_namespace_value(value, binding, span);
            }
            let place = self
                .places
                .dereference(value, self.types)
                .map_err(|error| Diagnostic::new(span, error.to_string()))?;
            let base = self.typed_value(ValueExpr::Load(place), place.ty(), span)?;
            return self.member_value(base, member, span);
        }
        let ty = self.expression_type(&base, span)?;
        if let Some(value) = self.context_constant_value(ty, member, span) {
            return value;
        }
        if matches!(
            self.types.kind(ty),
            Ok(TypeKind::String
                | TypeKind::FixedArray { .. }
                | TypeKind::Slice(_)
                | TypeKind::DynamicArray(_))
        ) {
            return self.sequence_member(base, member, span);
        }
        if let Some(field) = self.any_field(ty, member, span)? {
            let value = self.coerce_value(base, ty, span)?;
            let field_type = self
                .types
                .field_type(field)
                .map_err(|error| Diagnostic::new(span, error.to_string()))?;
            let value = match self.boxed_value_place(&value, span)? {
                Some(place) => ValueExpr::Load(
                    self.places
                        .field(place, field, self.types)
                        .map_err(|error| Diagnostic::new(span, error.to_string()))?,
                ),
                None => ValueExpr::Field {
                    base: Box::new(value),
                    field,
                    ty: field_type,
                },
            };
            return self.typed_value(value, field_type, span);
        }
        let fields = if let Some(field) = self.context_field(ty, member, span) {
            field?
        } else {
            match self.reflection_field_path(ty, member, span)? {
                Some(fields) => fields,
                None => match self.optional_field_path(ty, member, span)? {
                    Some(fields) => fields,
                    None => {
                        if let Some(binding) = self.instance_namespace_member(ty, member, span)? {
                            let receiver = self.coerce_value(base, ty, span)?;
                            return self.instance_namespace_value(receiver, binding, span);
                        }
                        return Err(Diagnostic::new(span, "unknown record member"));
                    }
                },
            }
        };
        let mut value = self.coerce_value(base, ty, span)?;
        let mut ty = ty;
        for field in fields {
            ty = self
                .types
                .field_type(field)
                .map_err(|error| Diagnostic::new(span, error.to_string()))?;
            value = match value {
                ValueExpr::Load(base) => ValueExpr::Load(
                    self.places
                        .field(base, field, self.types)
                        .map_err(|error| Diagnostic::new(span, error.to_string()))?,
                ),
                value => ValueExpr::Field {
                    base: Box::new(value),
                    field,
                    ty,
                },
            };
        }
        self.typed_value(value, ty, span)
    }
    pub(crate) fn binding_expression(
        &mut self,
        binding: Binding,
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        if let Binding::Storage(storage) = binding {
            self.check_local_storage_capture(storage, span)?;
        }
        match binding {
            Binding::CompilerInput {
                binding,
                ty,
            } => self.typed_value(
                ValueExpr::Bound {
                    binding,
                    ty,
                },
                ty,
                span,
            ),
            Binding::Discarded(_) => {
                Err(Diagnostic::new(span, "#discard parameter cannot be read"))
            }
            Binding::LambdaPreview(crate::short_lambdas::PreviewBinding::RuntimeCapture) => Err(
                Diagnostic::new(span, "short lambda cannot capture runtime local storage"),
            ),
            Binding::LambdaPreview(crate::short_lambdas::PreviewBinding::Parameter(_)) => {
                Err(Diagnostic::new(
                    span,
                    "lambda candidate type fact cannot supply a runtime value",
                ))
            }
            Binding::Macro(_) => Err(Diagnostic::new(
                span,
                "expanded procedures are source templates and cannot supply runtime values",
            )),
            Binding::Namespace(_) => Err(Diagnostic::new(
                span,
                "module namespaces cannot supply runtime values",
            )),
            Binding::Imported(binding) => {
                let value = self.imported_binding_value(binding, span)?;
                self.binding_expression(value, span)
            }
            Binding::Library(_) => Err(Diagnostic::new(
                span,
                "foreign libraries are compiler declarations and cannot supply runtime values",
            )),
            Binding::Code(id) => Ok(Expr::Code(id)),
            Binding::TypedConstant(id) => {
                let value = self
                    .meta
                    .constant(id)
                    .ok_or_else(|| {
                        Diagnostic::new(
                            span,
                            "typed constant does not belong to this semantic context",
                        )
                    })?
                    .clone();
                let ty = value.ty;
                self.typed_value(value.into_expression(), ty, span)
            }
            Binding::Type(ty) => Ok(Expr::Type(ty)),
            Binding::Procedure {
                procedure,
                ty,
            } => {
                let value = self.typed_value(
                    ValueExpr::ProcedureValue {
                        procedure,
                        ty,
                    },
                    ty,
                    span,
                )?;
                self.warn_deprecated_procedure(procedure, span)?;
                Ok(value)
            }
            Binding::Storage(Storage::Int(place)) => Ok(Expr::Int(IntExpr::load(place))),
            Binding::Storage(Storage::Bool(place)) => Ok(Expr::Bool(BoolExpr::Load(place))),
            Binding::Storage(Storage::Value(place)) => {
                self.typed_value(ValueExpr::Load(place), place.ty(), span)
            }
            Binding::Constant(value) => Ok(Self::constant(value)),
            Binding::Enum(constant) => Ok(Expr::Enum {
                ty: constant.ty,
                flags: self.enum_is_flags(constant.ty),
                representation: constant.value.ty(),
                value: ValueExpr::Enum {
                    ty: constant.ty,
                    value: constant.value,
                },
            }),
        }
    }
    pub(crate) fn path_expression(
        &mut self,
        path: &NamePath,
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        if let Some((binding, members)) = self.lexical_imported_value_root(path, span)? {
            let mut value = self.binding_expression(binding, span)?;
            for member in members {
                value = self.member_value(value, member, span)?;
            }
            return Ok(value);
        }
        if let Some(binding) = self.namespace_binding(path, span)? {
            return self.binding_expression(binding, span);
        }
        if let Some(enumeration) = self.local_enum_member(path, span)? {
            return self.binding_expression(Binding::Enum(enumeration), span);
        }
        if let Some(binding) = self.resolve_local_name(path.root, span)? {
            self.reject_lexical_capture(path.root, span)?;
            let mut value = self.binding_expression(binding, span)?;
            for member in &path.members {
                value = self.member_value(value, *member, span)?;
            }
            return Ok(value);
        }
        if let Some(binding) = self
            .scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(&path.root))
            .cloned()
        {
            let mut value = self.binding_expression(binding, span)?;
            for member in &path.members {
                value = self.member_value(value, *member, span)?;
            }
            return Ok(value);
        }
        if let Some(scope) = self.graph_scope {
            if let Some((bytes, members)) = scope.parameter_string(path) {
                let mut value = self.string_literal(&bytes, span)?;
                for member in members {
                    value = self.member_value(value, member, span)?;
                }
                return Ok(value);
            }
            if let Some(baked) = scope.baked_value(&NamePath {
                root: path.root,
                members: vec![],
            }) {
                use crate::polymorphism::BakedValue;
                let mut value = match baked {
                    BakedValue::Value(value) => {
                        let ty = value.ty;
                        self.typed_value(value.into_expression(), ty, span)
                    }
                    BakedValue::Float(value) => Ok(Expr::Float(FloatExpr::constant(value))),
                    BakedValue::String(bytes) => {
                        let ty = self.types.string();
                        self.typed_value(
                            jai_ir::ConstantValue {
                                ty,
                                kind: jai_ir::ConstantKind::StringBytes(bytes.into_vec()),
                            }
                            .into_expression(),
                            ty,
                            span,
                        )
                    }
                    BakedValue::Type(ty) => Ok(Expr::Type(ty)),
                    BakedValue::Code(id) => Ok(Expr::Code(id)),
                }?;
                for member in &path.members {
                    value = self.member_value(value, *member, span)?;
                }
                return Ok(value);
            }
            if let Some(context) = self.compile_time
                && let Some(id) = scope.pending_constant(path, context.deferred)
            {
                context.pending_constants.borrow_mut().push(id);
                return Err(Diagnostic::new(
                    span,
                    "constant declaration is pending typed compile-time evaluation",
                ));
            }
            let (binding, members) = match scope.value_root(path, span) {
                Ok(value) => value,
                Err(error) => {
                    if let Ok(ty) = scope.type_name(path, span) {
                        return Ok(Expr::Type(ty));
                    }
                    if scope.signature(path, span).is_ok() {
                        return self.procedure_value(path, span);
                    }
                    if let Some(ty) = self.reflection_type_name(path, span)? {
                        return Ok(Expr::Type(ty));
                    }
                    if let Some(value) = self.builtin_type_expression(path, span)? {
                        return Ok(value);
                    }
                    return Err(error);
                }
            };
            let mut value = self.binding_expression(binding, span)?;
            for member in members {
                value = self.member_value(value, member, span)?;
            }
            return Ok(value);
        }
        match self.lookup_path(path, span) {
            Ok(binding) => self.binding_expression(binding, span),
            Err(error) => {
                if let Some(value) = self.builtin_type_expression(path, span)? {
                    return Ok(value);
                }
                self.procedure_value(path, span).or(Err(error))
            }
        }
    }
    pub(crate) fn member_place(
        &mut self,
        base: Place,
        member: Symbol,
        span: Span,
    ) -> Result<Place, Diagnostic> {
        if matches!(self.types.kind(base.ty()), Ok(TypeKind::Pointer(_))) {
            let place = self
                .places
                .dereference(ValueExpr::Load(base), self.types)
                .map_err(|error| Diagnostic::new(span, error.to_string()))?;
            return self.member_place(place, member, span);
        }
        if matches!(
            self.types.kind(base.ty()),
            Ok(TypeKind::String | TypeKind::Slice(_) | TypeKind::DynamicArray(_))
        ) {
            return self.sequence_member_place(base, member, span);
        }
        let fields = if let Some(field) = self.any_field(base.ty(), member, span)? {
            vec![field]
        } else if let Some(field) = self.context_field(base.ty(), member, span) {
            field?
        } else {
            match self.reflection_field_path(base.ty(), member, span)? {
                Some(fields) => fields,
                None => self.field_path(base.ty(), member, span)?,
            }
        };
        let mut place = base;
        for field in fields {
            place = self
                .places
                .field(place, field, self.types)
                .map_err(|error| Diagnostic::new(span, error.to_string()))?;
        }
        Ok(place)
    }
}

pub(crate) fn scalar_constant(
    ty: TypeId,
    value: ConstantValue,
    types: &dyn jai_types::TypeView,
    span: Span,
) -> Result<jai_ir::ConstantValue, Diagnostic> {
    let kind = match value {
        ConstantValue::Int(value) => jai_ir::ConstantKind::Int(value),
        ConstantValue::Bool(value) => jai_ir::ConstantKind::Bool(value),
        ConstantValue::Float(value) => jai_ir::ConstantKind::Float(value),
        ConstantValue::WeakFloat(value) => {
            let TypeKind::Float(target) = types
                .kind(ty)
                .map_err(|error| Diagnostic::new(span, error.to_string()))?
            else {
                return Err(Diagnostic::new(
                    span,
                    "decimal constant requires a float type",
                ));
            };
            jai_ir::ConstantKind::Float(value.round(*target, span)?)
        }
        ConstantValue::Literal(value) => {
            let TypeKind::Integer(target) = types
                .kind(ty)
                .map_err(|error| Diagnostic::new(span, error.to_string()))?
            else {
                return Err(Diagnostic::new(
                    span,
                    "integer constant requires an integer type",
                ));
            };
            let ConstantValue::Int(value) =
                ConstantValue::Literal(value).coerce(ScalarType::Int(*target), span)?
            else {
                return Err(Diagnostic::new(
                    span,
                    "integer constant could not be materialized",
                ));
            };
            jai_ir::ConstantKind::Int(value)
        }
    };
    Ok(jai_ir::ConstantValue {
        ty,
        kind,
    })
}

impl Resolver<'_> {
    pub(crate) fn expression_place(
        &mut self,
        expression: &syntax::Expression,
    ) -> Result<Place, Diagnostic> {
        match &expression.kind {
            syntax::ExpressionKind::Context => self.context_place(expression.span),
            syntax::ExpressionKind::Dereference(pointer) => {
                self.dereference_place(pointer, expression.span)
            }
            syntax::ExpressionKind::Index {
                base,
                index,
            } => self.index_place(base, index, expression.span),
            syntax::ExpressionKind::Name(name) => Ok(self.storage(*name)?.place()),
            syntax::ExpressionKind::QualifiedName(path) => self.path_place(path, expression.span),
            syntax::ExpressionKind::Member {
                base,
                member,
            } => {
                let base = self.expression_place(base)?;
                self.member_place(base, *member, expression.span)
            }
            _ => Err(Diagnostic::new(
                expression.span,
                "assignment target does not denote mutable storage",
            )),
        }
    }
    pub(crate) fn path_place(&mut self, path: &NamePath, span: Span) -> Result<Place, Diagnostic> {
        let (binding, members) =
            if let Some(value) = self.lexical_imported_value_root(path, span)? {
                value
            } else if let Some(binding) = self
                .scopes
                .iter()
                .rev()
                .find_map(|scope| scope.get(&path.root))
                .cloned()
            {
                (binding, path.members.clone())
            } else if let Some(scope) = self.graph_scope {
                scope.value_root(path, span)?
            } else {
                (self.lookup_path(path, span)?, Vec::new())
            };
        let binding = if let Binding::Imported(binding) = binding {
            self.imported_binding_value(binding, span)?
        } else {
            binding
        };
        let Binding::Storage(storage) = binding else {
            return Err(Diagnostic::new(span, "cannot assign to a constant"));
        };
        let mut place = storage.place();
        for member in members {
            place = self.member_place(place, member, span)?;
        }
        Ok(place)
    }
    pub(crate) fn resolve_place(
        &mut self,
        target: &syntax::PlaceSyntax,
    ) -> Result<Place, Diagnostic> {
        match &target.kind {
            syntax::PlaceKind::Insert(directive) => self.insert_place(directive),
            syntax::PlaceKind::Dereference(pointer) => self.dereference_place(pointer, target.span),
            syntax::PlaceKind::Index {
                base,
                index,
            } => self.index_place(base, index, target.span),
            syntax::PlaceKind::Name(name) => Ok(self.storage(*name)?.place()),
            syntax::PlaceKind::Qualified(path) => self.path_place(path, target.span),
            syntax::PlaceKind::Member {
                base,
                member,
            } => {
                let base = self.expression_place(base)?;
                self.member_place(base, *member, target.span)
            }
        }
    }
}

#[cfg(test)]
mod scalar_constant_tests;
