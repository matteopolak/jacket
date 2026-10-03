//! Describe call arguments without lowering IR or executing source #run code.
use crate::overloads::{
    Argument, ArgumentInfo, ArgumentType, ConstantArgument, LiteralField, Match, NominalView,
    RecordArgumentField,
};
use crate::polymorphism::BakedValue;
use crate::{Binding, Resolver, ScalarConstant, Signature};
use jai_ir::{ConstantKind, ConstantValue};
use jai_source::{Diagnostic, Span};
use jai_syntax::{self as syntax, Expression, ExpressionKind as E, NamePath, UnaryOp};
use jai_types::{FloatType, FloatValue, IntegerType, ScalarType, TypeId, TypeKind, TypeView};

#[path = "pointer_arguments.rs"]
mod pointer_arguments;

enum SelectedHeader {
    Concrete(Signature),
    Generic(Match),
}

fn simple_binding_receiver(binding: &Binding, info: &ArgumentInfo) -> bool {
    info.is_compile_time_constant()
        || matches!(binding, Binding::Storage(storage)
            if matches!(storage.place().kind(),
                jai_ir::PlaceKind::Local(_) | jai_ir::PlaceKind::Global(_)))
}

impl NominalView for Resolver<'_> {
    fn layout_policy(&self) -> Option<jai_types::LayoutPolicy> {
        self.target_layout
    }
    fn enum_flags(&self, ty: TypeId) -> bool {
        self.enum_is_flags(ty)
    }
    fn nominal_ancestor(
        &self,
        actual: TypeId,
        required: TypeId,
        span: Span,
    ) -> Result<bool, Diagnostic> {
        self.restricted_nominal_ancestor(actual, required, span)
    }
    fn interface_member(
        &self,
        actual: TypeId,
        name: jai_source::Symbol,
        span: Span,
    ) -> Result<Option<TypeId>, Diagnostic> {
        self.restricted_interface_member(actual, name, span)
    }
    fn symbol_name(&self, name: jai_source::Symbol) -> Option<&str> {
        Some(self.symbols.name(name))
    }
    fn specialization(
        &self,
        ty: TypeId,
    ) -> Option<(
        jai_source::DeclarationId,
        &crate::polymorphism::Substitution,
    )> {
        self.meta.record_specializations.specialization(ty)
    }
    fn default_argument(&self, ty: TypeId, name: jai_source::Symbol) -> Option<&BakedValue> {
        self.meta.record_specializations.default_argument(ty, name)
    }
    fn enum_member(&self, ty: TypeId, name: jai_source::Symbol) -> Option<jai_types::Integer> {
        self.meta
            .local_declarations
            .enum_member_value(ty, name)
            .or_else(|| {
                self.meta
                    .record_specializations
                    .member_enum(ty)?
                    .values
                    .iter()
                    .find(|(member, _)| *member == name)
                    .map(|(_, value)| *value)
            })
            .or_else(|| {
                self.graph_scope
                    .and_then(|scope| scope.enum_member_value(ty, name))
            })
    }
    fn implicit_conversion(
        &self,
        source: TypeId,
        target: TypeId,
        span: Span,
    ) -> Result<bool, Diagnostic> {
        Ok(self.has_implicit_field_conversion(source, target, span)?
            || self.reflection_pointer_convertible(source, target, span)?)
    }
    fn record_fields(&self, ty: TypeId, span: Span) -> Result<Vec<LiteralField>, Diagnostic> {
        if matches!(self.types.kind(ty), Ok(TypeKind::Any(_))) {
            return ["type", "value_pointer"]
                .iter()
                .enumerate()
                .map(|(index, name)| {
                    let field = self
                        .types
                        .field(ty, index)
                        .map_err(|error| Diagnostic::new(span, error.to_string()))?;
                    Ok(LiteralField {
                        name: self.symbols.find(name),
                        id: field.id,
                        ty: field.ty,
                    })
                })
                .collect();
        }
        Ok(self
            .record_metadata(ty, span)?
            .fields
            .iter()
            .map(|field| LiteralField {
                name: field.name,
                id: field.id,
                ty: field.ty,
            })
            .collect())
    }
    fn field_default(
        &self,
        field: jai_types::FieldId,
        span: Span,
    ) -> Result<ConstantValue, Diagnostic> {
        let owner = self
            .types
            .record_type(field.record())
            .map_err(|error| Diagnostic::new(span, error.to_string()))?;
        if matches!(self.types.kind(owner), Ok(TypeKind::Any(_))) {
            let ty = self
                .types
                .field_type(field)
                .map_err(|error| Diagnostic::new(span, error.to_string()))?;
            return Ok(ConstantValue {
                ty,
                kind: ConstantKind::Zero,
            });
        }
        self.field_default_value(field, span)
    }
}

impl Resolver<'_> {
    pub(crate) fn resolve_overloaded_call_binding_with_defaults(
        &mut self,
        path: &NamePath,
        args: &[syntax::CallArgument],
        span: Span,
    ) -> Option<Result<crate::runtime_defaults::DirectCallWithDefaults, Diagnostic>> {
        self.graph_scope?;
        Some((|| {
            let header = self.select_call_header(path, args, span)?;
            self.bind_selected_header_with_defaults(header, args, span)
        })())
    }

    fn declaration_match_header(
        &self,
        matched: Match,
        span: Span,
    ) -> Result<SelectedHeader, Diagnostic> {
        let scope = self
            .graph_scope
            .ok_or_else(|| Diagnostic::new(span, "declaration match requires a module scope"))?;
        Ok(match scope.concrete_signature(matched.declaration) {
            Some(signature) => SelectedHeader::Concrete(signature.clone()),
            None => SelectedHeader::Generic(matched),
        })
    }

    pub(crate) fn describe_declaration_match(
        &mut self,
        matched: Match,
        span: Span,
    ) -> Result<ArgumentInfo, Diagnostic> {
        let header = self.declaration_match_header(matched, span)?;
        self.describe_selected_header(header, span)
    }

    pub(crate) fn bind_declaration_match(
        &mut self,
        matched: Match,
        args: &[syntax::CallArgument],
        span: Span,
    ) -> Result<(Signature, jai_ir::Call), Diagnostic> {
        let header = self.declaration_match_header(matched, span)?;
        self.bind_selected_header(header, args, span)
    }

    pub(crate) fn materialize_declaration_match(
        &mut self,
        matched: Match,
        span: Span,
    ) -> Result<Signature, Diagnostic> {
        let header = self.declaration_match_header(matched, span)?;
        self.materialize_selected_header(header, span)
            .map(|(signature, _)| signature)
    }
    fn bind_selected_header(
        &mut self,
        header: SelectedHeader,
        args: &[syntax::CallArgument],
        span: Span,
    ) -> Result<(Signature, jai_ir::Call), Diagnostic> {
        self.bind_selected_header_with_defaults(header, args, span)
            .map(|(signature, call, _)| (signature, call))
    }

    fn bind_selected_header_with_defaults(
        &mut self,
        header: SelectedHeader,
        args: &[syntax::CallArgument],
        span: Span,
    ) -> Result<crate::runtime_defaults::DirectCallWithDefaults, Diagnostic> {
        let source_arguments = args;
        let (signature, matched) = self.materialize_selected_header(header, span)?;
        let runtime_arguments;
        let args = if let Some(matched) = matched {
            let candidate = self
                .graph_scope
                .expect("generic binding has a graph scope")
                .candidate(matched.declaration, self.types, span)?;
            runtime_arguments = matched
                .bindings
                .iter()
                .filter(|binding| {
                    !candidate.parameters[binding.parameter].is_baked(&matched.substitution)
                })
                .map(|binding| args[binding.argument].clone())
                .collect::<Vec<_>>();
            runtime_arguments.as_slice()
        } else {
            args
        };
        let (call, reads) = self.bind_signature_arguments_with_defaults(&signature, args, span)?;
        self.register_generic_callback_arguments(&call, source_arguments, span)?;
        Ok((signature, call, reads))
    }

    fn materialize_selected_header(
        &mut self,
        header: SelectedHeader,
        span: Span,
    ) -> Result<(Signature, Option<Match>), Diagnostic> {
        match header {
            SelectedHeader::Concrete(signature) => Ok((signature, None)),
            SelectedHeader::Generic(matched) => {
                let scope = self
                    .graph_scope
                    .expect("generic candidates require a graph scope");
                let signature = scope.specialize(
                    &matched,
                    self.types,
                    &mut self.meta.record_specializations,
                    self.target_layout,
                    span,
                )?;
                self.register_graph_deprecation(signature.id, matched.declaration, span)?;
                if let Some(context) = self.compile_time
                    && context.effect_mode == crate::compile_time::EffectsMode::Isolated
                {
                    context
                        .generics
                        .borrow_mut()
                        .mark_isolated_procedure(signature.id);
                }
                Ok((signature, Some(matched)))
            }
        }
    }
    fn select_call_header(
        &mut self,
        path: &NamePath,
        args: &[syntax::CallArgument],
        span: Span,
    ) -> Result<SelectedHeader, Diagnostic> {
        if let Some(signature) = self.ready_local_callable_signature(path, span)? {
            if let Some(source) = self.meta.compiler_source_signatures.get(&signature.id) {
                let candidate =
                    crate::modules::compiler_intrinsics::source_candidate(signature.id, source);
                let arguments = args
                    .iter()
                    .map(|argument| {
                        Ok(Argument {
                            name: argument.name,
                            spread: argument.spread,
                            info: self.describe_argument(&argument.value)?,
                            span: argument.value.span,
                        })
                    })
                    .collect::<Result<Vec<_>, Diagnostic>>()?;
                crate::overloads::match_candidate_with_nominals(
                    self.types, self, &candidate, &arguments, span,
                )?;
            } else {
                let procedure = self
                    .types
                    .procedure_definition(signature.ty)
                    .map_err(|error| Diagnostic::new(span, error.to_string()))?
                    .clone();
                let metadata =
                    crate::procedure_values::bindings::CallbackSignature::source(&signature);
                self.preview_callback_call_results(&procedure, Some(&metadata), args, span)?;
            }
            return Ok(SelectedHeader::Concrete(signature));
        }
        let Some(scope) = self.graph_scope else {
            if !path.members.is_empty() {
                return Err(Diagnostic::new(
                    span,
                    "qualified call requires a module scope",
                ));
            }
            return self
                .signatures
                .get(&path.root)
                .cloned()
                .map(SelectedHeader::Concrete)
                .ok_or_else(|| Diagnostic::new(span, "unknown procedure"));
        };
        let declarations = match self.lexical_graph_binding(path, span)? {
            Some(binding) => scope.imported_callable(binding, span)?,
            None => scope.callable_declarations(path, span)?,
        };
        self.select_declarations_header(&declarations, args, span)
    }

    fn select_declarations_header(
        &mut self,
        declarations: &[jai_source::DeclarationId],
        args: &[syntax::CallArgument],
        span: Span,
    ) -> Result<SelectedHeader, Diagnostic> {
        let scope = self.graph_scope.ok_or_else(|| {
            Diagnostic::new(span, "declaration candidates require a module scope")
        })?;
        let candidates = declarations
            .iter()
            .map(|&id| {
                if let Some(signature) = scope.concrete_signature(id)
                    && let Some(source) = self.meta.compiler_source_signatures.get(&signature.id)
                {
                    Ok(crate::modules::compiler_intrinsics::source_candidate(
                        id, source,
                    ))
                } else {
                    scope.candidate(id, self.types, span)
                }
            })
            .collect::<Result<Vec<_>, _>>()?;
        let arguments = args
            .iter()
            .map(|argument| {
                Ok(Argument {
                    name: argument.name,
                    spread: argument.spread,
                    info: match &argument.value.kind {
                        syntax::ExpressionKind::ShortLambda(_) => ArgumentInfo {
                            ty: ArgumentType::ContextualProcedure {
                                compatible_signatures: Box::new([]),
                            },
                            constant: None,
                        },
                        syntax::ExpressionKind::Name(name) => {
                            let path = NamePath {
                                root: *name,
                                members: vec![],
                            };
                            if self.named_short_lambda_source_present(&path, argument.value.span)? {
                                ArgumentInfo {
                                    ty: ArgumentType::ContextualProcedure {
                                        compatible_signatures: Box::new([]),
                                    },
                                    constant: None,
                                }
                            } else {
                                self.describe_argument(&argument.value)?
                            }
                        }
                        syntax::ExpressionKind::QualifiedName(path) => {
                            if self.named_short_lambda_source_present(path, argument.value.span)? {
                                ArgumentInfo {
                                    ty: ArgumentType::ContextualProcedure {
                                        compatible_signatures: Box::new([]),
                                    },
                                    constant: None,
                                }
                            } else {
                                self.describe_argument(&argument.value)?
                            }
                        }
                        _ => self.describe_argument(&argument.value)?,
                    },
                    span: argument.value.span,
                })
            })
            .collect::<Result<Vec<_>, Diagnostic>>()?;
        let mut matches = Vec::new();
        let mut rejections = Vec::new();
        let mut pending = None;
        for candidate in &candidates {
            let matched = self
                .describe_candidate_callbacks(candidate, args, &arguments, span)
                .and_then(|arguments| {
                    crate::overloads::match_candidate_with_nominals(
                        self.types, self, candidate, &arguments, span,
                    )
                    .and_then(|matched| {
                        self.refine_source_declaration_match(
                            candidate, args, &arguments, matched, span,
                        )
                    })
                });
            match matched {
                Ok(matched) => matches.push(matched),
                Err(diagnostic) => {
                    if self
                        .compile_time
                        .is_some_and(|context| !context.pending.borrow().is_empty())
                    {
                        pending.get_or_insert_with(|| diagnostic.clone());
                    }
                    rejections.push(crate::overloads::Rejection {
                        declaration: candidate.declaration,
                        diagnostic,
                    });
                }
            }
        }
        if let Some(diagnostic) = pending {
            return Err(diagnostic);
        }
        let matched = if matches.is_empty() {
            return Err(crate::overloads::SelectionError::NoMatch(rejections).diagnostic(span));
        } else {
            crate::overloads::select_matches(matches).map_err(|error| error.diagnostic(span))?
        };
        Ok(match scope.concrete_signature(matched.declaration) {
            Some(signature) => SelectedHeader::Concrete(signature.clone()),
            None => SelectedHeader::Generic(matched),
        })
    }

    pub(crate) fn describe_argument(
        &mut self,
        expression: &Expression,
    ) -> Result<ArgumentInfo, Diagnostic> {
        let span = expression.span;
        if let Some(result) = self.describe_operator_expression(expression) {
            return result;
        }
        Ok(match &expression.kind {
            E::Context => match self.source_context_binding(span)? {
                Some(binding) => self.describe_binding(binding, span)?,
                None => ArgumentInfo::typed(self.require_context(span)?.definition.record_type),
            },
            E::CompileTimePredicate => ArgumentInfo::typed(self.types.scalar(ScalarType::Bool)),
            E::SourceFile | E::SourceFilepath | E::SourceLine | E::SourceLocation => {
                let location = self.ast_source_location(span)?;
                let scope = self.graph_scope.ok_or_else(|| {
                    Diagnostic::at_source(location, "source directive requires a module scope")
                })?;
                let record = scope.source_record(location.source).ok_or_else(|| {
                    Diagnostic::at_source(location, "source origin record is not retained")
                })?;
                match &expression.kind {
                    E::SourceFile => {
                        let point = crate::source_locations::source_point(record, location)?;
                        ArgumentInfo::constant(
                            BakedValue::String(point.filename.as_bytes().into()),
                            self.types.string(),
                        )
                    }
                    E::SourceFilepath => {
                        let point = crate::source_locations::source_point(record, location)?;
                        let directory = point.directory.ok_or_else(|| {
                            Diagnostic::at_source(
                                location,
                                "#filepath requires a retained source directory",
                            )
                        })?;
                        ArgumentInfo::constant(
                            BakedValue::String(directory.as_bytes().into()),
                            self.types.string(),
                        )
                    }
                    E::SourceLine => {
                        let point = crate::source_locations::source_point(record, location)?;
                        let value =
                            jai_types::Integer::checked(IntegerType::S64, point.line as i128)
                                .ok_or_else(|| {
                                    Diagnostic::at_source(location, "source coordinate exceeds s64")
                                })?;
                        ArgumentInfo::scalar_constant(ScalarConstant::Int(value), self.types)
                    }
                    E::SourceLocation => {
                        let ty = self.caller_location_type(span)?;
                        scope.validate_caller_location_type(ty, self.types, span)?;
                        let value = crate::source_locations::location_constant(
                            record, location, ty, self.types,
                        )?;
                        let value = BakedValue::runtime(value, self.types)
                            .map_err(|error| Diagnostic::at_source(location, error.to_string()))?;
                        ArgumentInfo::constant(value, ty)
                    }
                    _ => unreachable!("source directive branch"),
                }
            }
            E::AnonymousProcedure(source) => {
                let header = self.preview_anonymous_procedure_argument(source)?;
                ArgumentInfo::typed(header.ty)
            }
            E::CallHint { call, .. } => return self.describe_argument(call),
            E::InferredCast { mode, value } => {
                let value = self.describe_argument(value)?;
                if matches!(value.ty, ArgumentType::Known(ty) if ty == self.types.meta_type()) {
                    self.ensure_runtime_type_storage(self.types.meta_type(), span)?;
                }
                ArgumentInfo::contextual_cast(*mode, value)
            }
            E::Integer(value) => ArgumentInfo::integer_literal(*value),
            E::Character(value) => ArgumentInfo::scalar_constant(
                ScalarConstant::Int(jai_types::Integer::wrapping(
                    IntegerType::U8,
                    i128::from(*value),
                )),
                self.types,
            ),
            E::Bool(value) => {
                ArgumentInfo::scalar_constant(ScalarConstant::Bool(*value), self.types)
            }
            E::Null => ArgumentInfo::null(),
            E::InferredMember(name) => ArgumentInfo::enum_member(*name),
            E::Float(syntax::FloatLiteral::Decimal(decimal)) => ArgumentInfo::decimal_literal(
                decimal.spelling().into(),
                false,
                jai_eval::floats::default_decimal_type(decimal),
            ),
            E::Float(syntax::FloatLiteral::Bits32(bits)) => ArgumentInfo::scalar_constant(
                ScalarConstant::Float(FloatValue::F32(*bits)),
                self.types,
            ),
            E::Float(syntax::FloatLiteral::Bits64(bits)) => ArgumentInfo::scalar_constant(
                ScalarConstant::Float(FloatValue::F64(*bits)),
                self.types,
            ),
            E::String(bytes) => {
                ArgumentInfo::string_literal(bytes.clone().into_boxed_slice(), self.types.string())
            }
            E::HereString(value) => ArgumentInfo::constant(
                BakedValue::String(value.bytes.clone().into_boxed_slice()),
                self.types.string(),
            ),
            E::Code(body) => {
                let crate::Expr::Code(id) = self.capture_code(body, span)? else {
                    unreachable!("code capture returns a Code identity");
                };
                ArgumentInfo::constant(BakedValue::Code(id), self.types.code_type())
            }
            E::Name(name) | E::CompileVariable(name) => self.describe_path(
                &NamePath {
                    root: *name,
                    members: vec![],
                },
                span,
            )?,
            E::QualifiedName(path) => self.describe_path(path, span)?,
            E::Type(ty) => {
                let ty = self.preview_annotation(ty, span)?;
                ArgumentInfo::constant(BakedValue::Type(ty), self.types.meta_type())
            }
            E::Call(name, args) => self.describe_call(
                &NamePath {
                    root: *name,
                    members: vec![],
                },
                args,
                span,
            )?,
            E::QualifiedCall(path, args) => self.describe_call(path, args, span)?,
            E::ContextCall {
                callee,
                args,
                overrides,
            } => {
                let results = self.preview_context_call_results(callee, args, overrides, span)?;
                match results.as_slice() {
                    [ty] => ArgumentInfo::typed(*ty),
                    _ => {
                        return Err(Diagnostic::new(
                            span,
                            "call argument requires one procedure result",
                        ));
                    }
                }
            }
            E::IndirectCall { callee, args } => {
                let info = self.describe_argument(callee)?;
                let ty = self.argument_type(&info, span)?;
                let signature = self
                    .types
                    .procedure_definition(ty)
                    .map_err(|error| Diagnostic::new(span, error.to_string()))?
                    .clone();
                let metadata = self
                    .preview_expected_callback_contract(callee, ty, span)?
                    .and_then(|contract| contract.callback().cloned());
                let results =
                    self.preview_callback_call_results(&signature, metadata.as_ref(), args, span)?;
                match results.as_slice() {
                    [ty] => ArgumentInfo::typed(*ty),
                    _ => {
                        return Err(Diagnostic::new(
                            span,
                            "call argument requires one procedure result",
                        ));
                    }
                }
            }
            E::CompileTime(syntax::CompileTimeRun {
                body: syntax::CompileTimeBody::Expression(value),
                ..
            }) => {
                let mut info = self.describe_argument(value)?;
                info.constant = None;
                info
            }
            E::CompileTime(syntax::CompileTimeRun {
                body: syntax::CompileTimeBody::Procedure { result, .. },
                ..
            }) => {
                let ty = self.preview_annotation(result, span)?;
                if matches!(self.types.kind(ty), Ok(TypeKind::Void)) {
                    return Err(Diagnostic::new(
                        span,
                        "void #run cannot supply a call argument",
                    ));
                }
                ArgumentInfo::typed(ty)
            }
            E::Unary(operation, value) => {
                let mut info = self.describe_argument(value)?;
                if let ArgumentType::Known(ty) = info.ty
                    && matches!(self.types.kind(ty), Ok(TypeKind::Enum(_)))
                {
                    if *operation != UnaryOp::Complement || !self.enum_is_flags(ty) {
                        return Err(Diagnostic::new(
                            span,
                            "enum unary operation requires complement on an enum_flags type",
                        ));
                    }
                    info.constant = None;
                    return Ok(info);
                }
                if matches!(
                    info.ty,
                    ArgumentType::WeakFloat { .. } | ArgumentType::WeakFloatExpression { .. }
                ) {
                    match operation {
                        UnaryOp::Positive => return Ok(info),
                        UnaryOp::Negate => {
                            if let ArgumentType::WeakFloat { negative, .. } = &mut info.ty {
                                *negative = !*negative;
                            }
                            info.constant = match info.constant {
                                Some(ConstantArgument::FloatLiteral { spelling, negative }) => {
                                    Some(ConstantArgument::FloatLiteral {
                                        spelling,
                                        negative: !negative,
                                    })
                                }
                                Some(ConstantArgument::FloatExpression { f32, f64 }) => {
                                    Some(ConstantArgument::FloatExpression {
                                        f32: f32.map(FloatValue::negate),
                                        f64: f64.map(FloatValue::negate),
                                    })
                                }
                                _ => None,
                            };
                            return Ok(info);
                        }
                        UnaryOp::LogicalNot => {
                            return Err(Diagnostic::new(
                                span,
                                "floating-point value has no implicit bool conversion",
                            ));
                        }
                        UnaryOp::Complement => {
                            return Err(Diagnostic::new(
                                span,
                                "floating-point complement is not defined",
                            ));
                        }
                    }
                }
                if let Ok(value) = self.describe_scalar_constant(expression) {
                    ArgumentInfo::scalar_constant(value, self.types)
                } else {
                    match operation {
                        UnaryOp::Positive => {}
                        UnaryOp::Negate => match &mut info.ty {
                            ArgumentType::WeakInteger { minimum, maximum } => {
                                let lower = maximum.checked_neg().ok_or_else(|| {
                                    Diagnostic::new(span, "integer literal negation overflow")
                                })?;
                                let upper = minimum.checked_neg().ok_or_else(|| {
                                    Diagnostic::new(span, "integer literal negation overflow")
                                })?;
                                *minimum = lower;
                                *maximum = upper;
                            }
                            ArgumentType::WeakFloat { negative, .. } => *negative = !*negative,
                            _ => {}
                        },
                        UnaryOp::LogicalNot => {
                            info.ty = ArgumentType::Known(self.types.scalar(ScalarType::Bool))
                        }
                        UnaryOp::Complement => {}
                    }
                    if matches!(info.ty, ArgumentType::WeakInteger { .. }) {
                        info.ty = ArgumentType::Known(
                            self.types.scalar(ScalarType::Int(IntegerType::S64)),
                        );
                    }
                    info.constant = None;
                    info
                }
            }
            E::Cast(mode, ty, value) => {
                let operand = self.describe_argument(value)?;
                crate::overloads::explicit_cast_argument(
                    self.types,
                    self,
                    self.types.scalar(*ty),
                    &operand,
                    *mode,
                    span,
                )?
            }
            E::TypeCast { mode, ty, value } => {
                let operand = self.describe_argument(value)?;
                let target = self.preview_annotation(ty, span)?;
                crate::overloads::explicit_cast_argument(
                    self.types, self, target, &operand, *mode, span,
                )?
            }
            E::Binary(operation, lhs, rhs) => {
                let lhs = self.describe_argument(lhs)?;
                let rhs = self.describe_argument(rhs)?;
                if let Some(result) =
                    pointer_arguments::describe(self.types, *operation, &lhs, &rhs, span)
                {
                    return result;
                }
                if let Some(result) = self.describe_enum_binary(*operation, &lhs, &rhs, span) {
                    return result;
                }
                if let (Some(lhs), Some(rhs)) = (lhs.scalar_value(), rhs.scalar_value()) {
                    let value = jai_eval::binary_values(
                        *operation,
                        lhs,
                        rhs,
                        self.checks.arithmetic_overflow,
                        span,
                    )?;
                    return Ok(ArgumentInfo::scalar_constant(value, self.types));
                }
                if self.is_float_argument(&lhs) || self.is_float_argument(&rhs) {
                    let info = self.common_float_argument(&lhs, &rhs, span)?;
                    if matches!(
                        operation,
                        syntax::BinaryOp::Equal
                            | syntax::BinaryOp::NotEqual
                            | syntax::BinaryOp::Less
                            | syntax::BinaryOp::LessEqual
                            | syntax::BinaryOp::Greater
                            | syntax::BinaryOp::GreaterEqual
                    ) {
                        return Ok(match self.describe_scalar_constant(expression) {
                            Ok(value) => ArgumentInfo::scalar_constant(value, self.types),
                            Err(_) => ArgumentInfo::typed(self.types.scalar(ScalarType::Bool)),
                        });
                    }
                    if !matches!(
                        operation,
                        syntax::BinaryOp::Add
                            | syntax::BinaryOp::Subtract
                            | syntax::BinaryOp::Multiply
                            | syntax::BinaryOp::Divide
                            | syntax::BinaryOp::Remainder
                    ) {
                        return Err(Diagnostic::new(
                            span,
                            "operator is not defined for floating-point arguments",
                        ));
                    }
                    return Ok(self.float_constant_description(info, expression));
                }
                match self.describe_scalar_constant(expression) {
                    Ok(value) => ArgumentInfo::scalar_constant(value, self.types),
                    Err(error) if lhs.constant.is_some() && rhs.constant.is_some() => {
                        return Err(error);
                    }
                    Err(_) => {
                        if matches!(
                            operation,
                            syntax::BinaryOp::Equal
                                | syntax::BinaryOp::NotEqual
                                | syntax::BinaryOp::Less
                                | syntax::BinaryOp::LessEqual
                                | syntax::BinaryOp::Greater
                                | syntax::BinaryOp::GreaterEqual
                                | syntax::BinaryOp::LogicalOr
                                | syntax::BinaryOp::LogicalAnd
                        ) {
                            ArgumentInfo::typed(self.types.scalar(ScalarType::Bool))
                        } else {
                            let mut info = self.common_argument(lhs, rhs, span)?;
                            if matches!(info.ty, ArgumentType::WeakInteger { .. }) {
                                info.ty = ArgumentType::Known(
                                    self.types.scalar(ScalarType::Int(IntegerType::S64)),
                                );
                            }
                            info
                        }
                    }
                }
            }
            E::Conditional(value) => {
                let _ = self.describe_argument(&value.condition)?;
                let lhs = self.describe_argument(&value.then_value)?;
                let rhs =
                    self.describe_argument(value.else_value.as_ref().ok_or_else(|| {
                        Diagnostic::new(span, "conditional argument requires an else value")
                    })?)?;
                if self.is_float_argument(&lhs) || self.is_float_argument(&rhs) {
                    let info = self.common_float_argument(&lhs, &rhs, span)?;
                    return Ok(self.float_constant_description(info, expression));
                }
                match self.describe_scalar_constant(expression) {
                    Ok(value) => ArgumentInfo::scalar_constant(value, self.types),
                    Err(error)
                        if lhs.constant.is_some()
                            && rhs.constant.is_some()
                            && self.describe_scalar_constant(&value.condition).is_ok() =>
                    {
                        return Err(error);
                    }
                    Err(_) => self.common_argument(lhs, rhs, span)?,
                }
            }
            E::AddressOf(value) => {
                let info = self.describe_argument(value)?;
                if let Some(ConstantArgument::Value(BakedValue::Type(pointee))) =
                    info.constant.as_ref()
                {
                    let ty = self
                        .types
                        .pointer(*pointee)
                        .map_err(|error| Diagnostic::new(span, error.to_string()))?;
                    return Ok(ArgumentInfo::constant(
                        BakedValue::Type(ty),
                        self.types.meta_type(),
                    ));
                }
                let ty = self.argument_type(&info, span)?;
                ArgumentInfo::typed(
                    self.types
                        .pointer(ty)
                        .map_err(|error| Diagnostic::new(span, error.to_string()))?,
                )
            }
            E::Dereference(value) => {
                let info = self.describe_argument(value)?;
                let ty = self.argument_type(&info, span)?;
                let TypeKind::Pointer(ty) = *self
                    .types
                    .kind(ty)
                    .map_err(|error| Diagnostic::new(span, error.to_string()))?
                else {
                    return Err(Diagnostic::new(span, "dereference requires a pointer"));
                };
                ArgumentInfo::typed(ty)
            }
            E::Index { base, index } => {
                let info = self.describe_argument(base)?;
                let _ = self.describe_argument(index)?;
                let ty = self.argument_type(&info, span)?;
                let element = match *self
                    .types
                    .kind(ty)
                    .map_err(|error| Diagnostic::new(span, error.to_string()))?
                {
                    TypeKind::FixedArray { element, .. }
                    | TypeKind::Slice(element)
                    | TypeKind::DynamicArray(element)
                    | TypeKind::Pointer(element) => element,
                    TypeKind::String => self.types.scalar(ScalarType::Int(IntegerType::U8)),
                    _ => {
                        return Err(Diagnostic::new(
                            span,
                            "index requires an array, slice, string or pointer",
                        ));
                    }
                };
                ArgumentInfo::typed(element)
            }
            E::Member { base, member } => {
                let info = self.describe_argument(base)?;
                let simple = self.simple_member_receiver(base, &info);
                self.describe_baked_member(info, *member, span, simple)?
            }
            E::StructLiteral(literal) => ArgumentInfo {
                ty: ArgumentType::RecordLiteral {
                    ty: literal
                        .ty
                        .as_ref()
                        .map(|path| self.local_type_name(path, span))
                        .transpose()?,
                    fields: literal
                        .fields
                        .iter()
                        .map(|field| {
                            Ok(RecordArgumentField {
                                name: field.name,
                                value: self.describe_argument(&field.value)?,
                                span: field.span,
                            })
                        })
                        .collect::<Result<_, Diagnostic>>()?,
                },
                constant: None,
            },
            E::PositionalStructLiteral(literal) if literal.ty.is_some() => {
                let scope = self
                    .graph_scope
                    .ok_or_else(|| Diagnostic::new(span, "record requires a graph scope"))?;
                ArgumentInfo::typed(scope.type_name(literal.ty.as_ref().unwrap(), span)?)
            }
            E::ArrayLiteral(literal) => {
                let explicit = literal
                    .element_type
                    .as_ref()
                    .map(|ty| self.preview_annotation(ty, span))
                    .transpose()?;
                let elements = literal
                    .elements
                    .iter()
                    .map(|value| self.describe_argument(value))
                    .collect::<Result<Vec<_>, _>>()?;
                let element = explicit.or_else(|| {
                    elements
                        .first()
                        .and_then(|value| self.argument_type(value, span).ok())
                });
                let default = element
                    .map(|element| {
                        self.types
                            .fixed_array(element, literal.elements.len() as u64)
                            .map_err(|error| Diagnostic::new(span, error.to_string()))
                    })
                    .transpose()?;
                ArgumentInfo {
                    ty: ArgumentType::ArrayLiteral {
                        explicit,
                        default,
                        elements,
                    },
                    constant: None,
                }
            }
            E::TypeQuery {
                query: syntax::TypeQueryKind::TypeOf,
                value,
            } => {
                let ty = self.queried_expression_type(value)?;
                ArgumentInfo::constant(BakedValue::Type(ty), self.types.meta_type())
            }
            E::TypeQuery {
                query: syntax::TypeQueryKind::SizeOf,
                value,
            } => {
                let ty = self.describe_type_value(value)?;
                let jai_types::ReflectionReadiness::Ready(size) =
                    jai_types::reflected_size(self.types, ty, self.target_layout)
                        .map_err(|error| Diagnostic::new(span, error.to_string()))?
                else {
                    return Err(Diagnostic::new(
                        span,
                        "size_of is waiting for target or nominal layout dependencies",
                    ));
                };
                let value = jai_types::Integer::checked(IntegerType::S64, i128::from(size))
                    .ok_or_else(|| {
                        Diagnostic::new(span, "target size cannot be represented by Jai int")
                    })?;
                ArgumentInfo::scalar_constant(ScalarConstant::Int(value), self.types)
            }
            E::TypeQuery {
                query: syntax::TypeQueryKind::TypeInfo,
                value,
            } => {
                let ty = self.describe_type_value(value)?;
                ArgumentInfo::typed(self.type_info_pointer_type(ty, span)?)
            }
            E::TypeQuery {
                query: syntax::TypeQueryKind::InitializerOf,
                value,
            } => {
                let ty = self.describe_type_value(value)?;
                let value = BakedValue::runtime(self.default_value(ty, span)?, self.types)
                    .map_err(|error| Diagnostic::new(span, error.to_string()))?;
                ArgumentInfo::constant(value, ty)
            }
            _ => {
                return Err(Diagnostic::new(
                    span,
                    "call argument requires a contextual type that overload matching cannot yet determine",
                ));
            }
        })
    }

    fn describe_scalar_constant(
        &self,
        expression: &Expression,
    ) -> Result<ScalarConstant, Diagnostic> {
        jai_eval::evaluate_paths(expression, |path, span| {
            match self.lookup_path(path, span)? {
                Binding::Constant(value) => Ok(value),
                _ => Err(Diagnostic::new(
                    span,
                    "argument is not a scalar compile-time constant",
                )),
            }
        })
    }
    fn describe_type_value(&mut self, expression: &Expression) -> Result<TypeId, Diagnostic> {
        match self.describe_argument(expression)?.constant {
            Some(ConstantArgument::Value(BakedValue::Type(ty))) => Ok(ty),
            _ => Err(Diagnostic::new(
                expression.span,
                "type query requires a type value; use type_of for a runtime expression",
            )),
        }
    }
    fn describe_path(&mut self, path: &NamePath, span: Span) -> Result<ArgumentInfo, Diagnostic> {
        if let Some(binding) = self.lexical_graph_binding(path, span)? {
            let binding = self.imported_binding_value(binding, span)?;
            return self.describe_binding(binding, span);
        }
        if let Some(binding) = self
            .scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(&path.root))
            .cloned()
        {
            let mut info = self.describe_binding(binding.clone(), span)?;
            let mut simple = simple_binding_receiver(&binding, &info);
            for &member in &path.members {
                info = self.describe_baked_member(info, member, span, simple)?;
                simple = info.is_compile_time_constant();
            }
            return Ok(info);
        }
        if let Some(scope) = self.graph_scope {
            if let Some(value) = scope.baked_value(&NamePath {
                root: path.root,
                members: vec![],
            }) {
                let ty = match &value {
                    BakedValue::Value(value) => value.ty,
                    BakedValue::Type(_) => self.types.meta_type(),
                    BakedValue::Float(value) => self.types.float(value.ty()),
                    BakedValue::String(_) => self.types.string(),
                    BakedValue::Code(_) => self.types.code_type(),
                };
                let mut info = ArgumentInfo::constant(value, ty);
                for &member in &path.members {
                    let simple = info.is_compile_time_constant();
                    info = self.describe_baked_member(info, member, span, simple)?;
                }
                return Ok(info);
            }
            if let Ok((binding, members)) = scope.value_root(path, span) {
                let mut info = self.describe_binding(binding.clone(), span)?;
                let mut simple = simple_binding_receiver(&binding, &info);
                for member in members {
                    info = self.describe_baked_member(info, member, span, simple)?;
                    simple = info.is_compile_time_constant();
                }
                return Ok(info);
            }
            if let Ok(ty) = scope.type_name(path, span) {
                return Ok(ArgumentInfo::constant(
                    BakedValue::Type(ty),
                    self.types.meta_type(),
                ));
            }
            if let Ok(signature) = scope.signature(path, span) {
                return self.describe_binding(
                    Binding::Procedure {
                        procedure: signature.id,
                        ty: signature.ty,
                    },
                    span,
                );
            }
        }
        if let Some(crate::Expr::Type(ty)) = self.builtin_type_expression(path, span)? {
            return Ok(ArgumentInfo::constant(
                BakedValue::Type(ty),
                self.types.meta_type(),
            ));
        }
        let binding = self.lookup_path(path, span)?;
        self.describe_binding(binding, span)
    }
    pub(crate) fn describe_binding(
        &self,
        binding: Binding,
        span: Span,
    ) -> Result<ArgumentInfo, Diagnostic> {
        Ok(match binding {
            Binding::CompilerInput { ty, .. } => ArgumentInfo::typed(ty),
            Binding::Discarded(_) => {
                return Err(Diagnostic::new(span, "a #discard parameter cannot be read"));
            }
            Binding::LambdaPreview(crate::short_lambdas::PreviewBinding::Parameter(ty)) => {
                ArgumentInfo::typed(ty)
            }
            Binding::LambdaPreview(crate::short_lambdas::PreviewBinding::RuntimeCapture) => {
                return Err(Diagnostic::new(
                    span,
                    "procedure cannot capture runtime local storage",
                ));
            }
            Binding::Macro(_) => {
                return Err(Diagnostic::new(
                    span,
                    "macro expansion is not a runtime procedure value",
                ));
            }
            Binding::Namespace(_) => {
                return Err(Diagnostic::new(
                    span,
                    "module namespace cannot supply a runtime or baked procedure argument",
                ));
            }
            Binding::Imported(binding) => {
                let scope = self.graph_scope.ok_or_else(|| {
                    Diagnostic::new(span, "imported argument requires its module graph")
                })?;
                let resolved = scope.imported_value(binding, span)?;
                if matches!(resolved, Binding::Imported(_)) {
                    let declarations = scope.imported_callable(binding, span)?;
                    let [declaration] = declarations.as_slice() else {
                        return Err(Diagnostic::new(
                            span,
                            "overloaded procedure argument requires a contextual signature",
                        ));
                    };
                    let signature = scope.concrete_signature(*declaration).ok_or_else(|| {
                        Diagnostic::new(
                            span,
                            "polymorphic procedure argument requires a contextual specialization",
                        )
                    })?;
                    self.describe_binding(
                        Binding::Procedure {
                            procedure: signature.id,
                            ty: signature.ty,
                        },
                        span,
                    )?
                } else {
                    self.describe_binding(resolved, span)?
                }
            }
            Binding::Library(_) => {
                return Err(Diagnostic::new(
                    span,
                    "foreign library metadata cannot supply a runtime or baked procedure argument",
                ));
            }
            Binding::Code(id) => {
                self.meta.codes.get(id).ok_or_else(|| {
                    Diagnostic::new(
                        span,
                        "captured Code value belongs to another semantic context",
                    )
                })?;
                ArgumentInfo::constant(BakedValue::Code(id), self.types.code_type())
            }
            Binding::Procedure { procedure, ty } => {
                let value = BakedValue::runtime(
                    ConstantValue {
                        ty,
                        kind: ConstantKind::Procedure(procedure),
                    },
                    self.types,
                )
                .map_err(|error| Diagnostic::new(span, error.to_string()))?;
                ArgumentInfo::constant(value, ty)
            }
            Binding::Constant(value) => ArgumentInfo::scalar_constant(value, self.types),
            Binding::Storage(storage) => ArgumentInfo::typed(storage.place().ty()),
            Binding::Enum(value) => ArgumentInfo::constant(
                BakedValue::Value(ConstantValue {
                    ty: value.ty,
                    kind: ConstantKind::Enum(value.value),
                }),
                value.ty,
            ),
            Binding::Type(ty) => {
                ArgumentInfo::constant(BakedValue::Type(ty), self.types.meta_type())
            }
            Binding::TypedConstant(id) => {
                let value = self.meta.constant(id).ok_or_else(|| {
                    Diagnostic::new(
                        span,
                        "typed constant does not belong to this semantic context",
                    )
                })?;
                ArgumentInfo::constant(BakedValue::Value(value.clone()), value.ty)
            }
        })
    }
    fn describe_member(
        &mut self,
        mut ty: TypeId,
        member: jai_source::Symbol,
        span: Span,
        simple: bool,
    ) -> Result<ArgumentInfo, Diagnostic> {
        if let Ok(TypeKind::Pointer(pointee)) = self.types.kind(ty) {
            ty = *pointee;
        }
        let element = match self
            .types
            .kind(ty)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?
        {
            TypeKind::String => Some(self.types.scalar(ScalarType::Int(IntegerType::U8))),
            TypeKind::FixedArray { element, .. }
            | TypeKind::Slice(element)
            | TypeKind::DynamicArray(element) => Some(*element),
            _ => None,
        };
        if let Some(element) = element {
            let member_ty = match self.symbols.name(member) {
                "count" => self.types.scalar(ScalarType::Int(IntegerType::S64)),
                "data" => self
                    .types
                    .pointer(element)
                    .map_err(|error| Diagnostic::new(span, error.to_string()))?,
                "allocated" if matches!(self.types.kind(ty), Ok(TypeKind::DynamicArray(_))) => {
                    self.types.scalar(ScalarType::Int(IntegerType::S64))
                }
                _ => return Err(Diagnostic::new(span, "unknown sequence member")),
            };
            return Ok(ArgumentInfo::typed(member_ty));
        }
        if let Some(field) = self.any_field(ty, member, span)? {
            return Ok(ArgumentInfo::typed(
                self.types
                    .field_type(field)
                    .map_err(|error| Diagnostic::new(span, error.to_string()))?,
            ));
        }
        let Some(fields) = self.optional_field_path(ty, member, span)? else {
            return self
                .describe_instance_namespace_member(ty, member, span, simple)?
                .ok_or_else(|| Diagnostic::new(span, "unknown record member"));
        };
        for field in fields {
            ty = self
                .types
                .field_type(field)
                .map_err(|error| Diagnostic::new(span, error.to_string()))?;
        }
        Ok(ArgumentInfo::typed(ty))
    }
    fn describe_baked_member(
        &mut self,
        info: ArgumentInfo,
        member: jai_source::Symbol,
        span: Span,
        simple: bool,
    ) -> Result<ArgumentInfo, Diagnostic> {
        if let Some(ConstantArgument::Value(BakedValue::Type(ty))) = &info.constant {
            return self
                .describe_instance_namespace_member(*ty, member, span, true)?
                .ok_or_else(|| Diagnostic::new(span, "type declaration has no namespace member"));
        }
        if let Some(ConstantArgument::Value(BakedValue::Value(mut value))) = info.constant.clone()
            && matches!(
                self.types.kind(value.ty),
                Ok(TypeKind::Record(_) | TypeKind::Any(_))
            )
        {
            let Some(path) = self.optional_field_path(value.ty, member, span)? else {
                return self
                    .describe_instance_namespace_member(value.ty, member, span, true)?
                    .ok_or_else(|| Diagnostic::new(span, "unknown record member"));
            };
            for field in path {
                let ty = self
                    .types
                    .validate_field(value.ty, field)
                    .map_err(|error| Diagnostic::new(span, error.to_string()))?;
                value = match value.kind {
                    ConstantKind::Record(mut values) if field.index() < values.len() => {
                        values.remove(field.index())
                    }
                    ConstantKind::Union {
                        field: active,
                        value,
                    } if active == field => *value,
                    ConstantKind::Zero => ConstantValue {
                        ty,
                        kind: ConstantKind::Zero,
                    },
                    _ => {
                        return Err(Diagnostic::new(
                            span,
                            "baked record field does not identify an initialized value",
                        ));
                    }
                };
                if value.ty != ty {
                    return Err(Diagnostic::new(
                        span,
                        "baked record field has a different checked type",
                    ));
                }
            }
            let ty = value.ty;
            let value = BakedValue::runtime(value, self.types)
                .map_err(|error| Diagnostic::new(span, error.to_string()))?;
            return Ok(ArgumentInfo::constant(value, ty));
        }
        let ty = self.argument_type(&info, span)?;
        if matches!(
            info.ty,
            ArgumentType::RecordLiteral { .. } | ArgumentType::ArrayLiteral { .. }
        ) {
            crate::overloads::contextual_conversion(self.types, self, ty, &info.ty, span)?;
        }
        self.describe_member(ty, member, span, simple)
    }

    fn describe_instance_namespace_member(
        &self,
        ty: TypeId,
        member: jai_source::Symbol,
        span: Span,
        simple: bool,
    ) -> Result<Option<ArgumentInfo>, Diagnostic> {
        use crate::modules::aggregates::parameterized::ReadyInstanceRecordMember;
        let fact = if let Some(fact) = self.ready_instance_record_member(ty, member, span)? {
            Some(fact)
        } else if self.meta.record_specializations.record(ty).is_some() {
            None
        } else if let Some(binding) = self.ready_namespace_member(ty, member) {
            Some(ReadyInstanceRecordMember::Binding(binding))
        } else {
            self.enum_member(ty, member).map(|value| {
                ReadyInstanceRecordMember::Baked(BakedValue::Value(ConstantValue {
                    ty,
                    kind: ConstantKind::Enum(value),
                }))
            })
        };
        let Some(fact) = fact else {
            return Ok(None);
        };
        let mut info = match fact {
            ReadyInstanceRecordMember::Binding(binding) => self.describe_binding(binding, span)?,
            ReadyInstanceRecordMember::Baked(value) => {
                let ty = match &value {
                    BakedValue::Value(value) => value.ty,
                    BakedValue::Type(_) => self.types.meta_type(),
                    BakedValue::Float(value) => self.types.float(value.ty()),
                    BakedValue::String(_) => self.types.string(),
                    BakedValue::Code(_) => self.types.code_type(),
                };
                ArgumentInfo::constant(value, ty)
            }
        };
        if !simple {
            if matches!(
                info.constant,
                Some(ConstantArgument::Value(
                    BakedValue::Type(_) | BakedValue::Code(_)
                ))
            ) {
                return Err(Diagnostic::new(
                    span,
                    "effectful receiver of a compile-time record namespace member requires source type-expression evaluation",
                ));
            }
            info.constant = None;
        }
        Ok(Some(info))
    }

    fn simple_member_receiver(&self, source: &Expression, info: &ArgumentInfo) -> bool {
        if info.is_compile_time_constant() {
            return true;
        }
        let path = match &source.kind {
            E::Name(root) => NamePath {
                root: *root,
                members: vec![],
            },
            E::QualifiedName(path) => path.clone(),
            _ => return false,
        };
        if let Some(binding) = self
            .scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(&path.root))
        {
            return path.members.is_empty() && simple_binding_receiver(binding, info);
        }
        self.graph_scope
            .and_then(|scope| scope.value_root(&path, source.span).ok())
            .is_some_and(|(binding, members)| {
                members.is_empty() && simple_binding_receiver(&binding, info)
            })
    }
    fn describe_call(
        &mut self,
        path: &NamePath,
        args: &[syntax::CallArgument],
        span: Span,
    ) -> Result<ArgumentInfo, Diagnostic> {
        if let Some(query) = self.describe_constant_query_call(path, args, span)? {
            return Ok(query);
        }
        if let Some(crate::Expr::Type(ty)) = self.try_record_application(path, args, span)? {
            return Ok(ArgumentInfo::constant(
                BakedValue::Type(ty),
                self.types.meta_type(),
            ));
        }
        let results = self.describe_call_results(path, args, span)?;
        match results.as_slice() {
            [ty] => Ok(ArgumentInfo::typed(*ty)),
            _ => Err(Diagnostic::new(
                span,
                "call argument requires one procedure result",
            )),
        }
    }
    fn describe_selected_header(
        &mut self,
        header: SelectedHeader,
        span: Span,
    ) -> Result<ArgumentInfo, Diagnostic> {
        let results = self.describe_selected_results(header, span)?;
        match results.as_slice() {
            [ty] => Ok(ArgumentInfo::typed(*ty)),
            _ => Err(Diagnostic::new(
                span,
                "call argument requires one procedure result",
            )),
        }
    }
    pub(crate) fn describe_call_results(
        &mut self,
        path: &NamePath,
        args: &[syntax::CallArgument],
        span: Span,
    ) -> Result<Vec<TypeId>, Diagnostic> {
        if path.members.is_empty()
            && let Some(Binding::LambdaPreview(crate::short_lambdas::PreviewBinding::Parameter(ty))) =
                self.scopes
                    .iter()
                    .rev()
                    .find_map(|scope| scope.get(&path.root))
            && let Ok(signature) = self.types.procedure_definition(*ty)
        {
            let signature = signature.clone();
            return self.preview_indirect_call_results(&signature, args, span);
        }
        if let Some(ty) = self.baked_callable_type(path) {
            let signature = self
                .types
                .procedure_definition(ty)
                .map_err(|error| Diagnostic::new(span, error.to_string()))?
                .clone();
            let contract = self
                .checked_baked_callback_binding_contract(path, span)?
                .ok_or_else(|| {
                    Diagnostic::new(span, "baked callback has no checked source contract")
                })?;
            return self.preview_callback_call_results(&signature, contract.callback(), args, span);
        }
        if matches!(
            self.lookup_path(path, span),
            Ok(Binding::Storage(_) | Binding::TypedConstant(_))
        ) {
            let info = self.describe_path(path, span)?;
            let ty = self.argument_type(&info, span)?;
            let signature = self
                .types
                .procedure_definition(ty)
                .map_err(|error| Diagnostic::new(span, error.to_string()))?
                .clone();
            let source = Expression {
                kind: if path.members.is_empty() {
                    E::Name(path.root)
                } else {
                    E::QualifiedName(path.clone())
                },
                span,
            };
            let metadata = self
                .preview_expected_callback_contract(&source, ty, span)?
                .and_then(|contract| contract.callback().cloned());
            return self.preview_callback_call_results(&signature, metadata.as_ref(), args, span);
        }
        let header = self.select_call_header(path, args, span)?;
        self.describe_selected_results(header, span)
    }
    /// Validate a result group before any source call arguments are lowered.
    pub(crate) fn describe_bound_call_results(
        &mut self,
        path: &NamePath,
        args: &[syntax::CallArgument],
        used: &[bool],
        span: Span,
    ) -> Result<Vec<TypeId>, Diagnostic> {
        let header = self.select_call_header(path, args, span)?;
        let mut usages = match &header {
            SelectedHeader::Concrete(signature) => signature
                .results
                .iter()
                .map(|result| result.usage)
                .collect::<Vec<_>>(),
            SelectedHeader::Generic(matched) => self
                .graph_scope
                .and_then(|scope| scope.generic_result_usages(matched.declaration))
                .ok_or_else(|| {
                    Diagnostic::new(
                        span,
                        "generic result group requires its source result contract",
                    )
                })?,
        };
        let results = self.describe_selected_results(header, span)?;
        // A single dependent result can materialize as the canonical void type.
        if results.is_empty() && usages.len() == 1 {
            usages.clear();
        }
        if used.len() != results.len() {
            return Err(Diagnostic::new(
                span,
                format!(
                    "procedure returns {} results, but {} result destinations were provided",
                    results.len(),
                    used.len(),
                ),
            ));
        }
        if usages.len() != results.len() {
            return Err(Diagnostic::new(
                span,
                "source result contract differs from the checked result count",
            ));
        }
        for (index, usage) in usages.iter().enumerate() {
            if *usage == syntax::ResultUsage::Required && !used[index] {
                return Err(Diagnostic::new(
                    span,
                    format!(
                        "result {} is marked #must and cannot be discarded",
                        index + 1
                    ),
                ));
            }
        }
        Ok(results)
    }
    pub(crate) fn describe_discarded_call_results(
        &mut self,
        path: &NamePath,
        args: &[syntax::CallArgument],
        span: Span,
    ) -> Result<Vec<TypeId>, Diagnostic> {
        let header = self.select_call_header(path, args, span)?;
        match &header {
            SelectedHeader::Concrete(signature) => {
                crate::result_obligations::check_result_use(&signature.results, &[], 0, span)?
            }
            SelectedHeader::Generic(matched) => {
                if self
                    .graph_scope
                    .and_then(|scope| scope.source_results_required(matched.declaration))
                    .ok_or_else(|| {
                        Diagnostic::new(
                            span,
                            "discarded generic callback requires its source result contract",
                        )
                    })?
                {
                    return Err(Diagnostic::new(
                        span,
                        "result is marked #must and cannot be discarded",
                    ));
                }
            }
        }
        self.describe_selected_results(header, span)
    }
    fn describe_selected_results(
        &mut self,
        header: SelectedHeader,
        span: Span,
    ) -> Result<Vec<TypeId>, Diagnostic> {
        match header {
            SelectedHeader::Concrete(signature) => {
                let procedure = self
                    .types
                    .procedure_definition(signature.ty)
                    .map_err(|error| Diagnostic::new(span, error.to_string()))?;
                self.check_call_context(procedure, span)?;
                Ok(signature.results.iter().map(|result| result.ty).collect())
            }
            SelectedHeader::Generic(matched) => {
                let scope = self.graph_scope.expect("generic header has a graph scope");
                let (convention, context) = scope
                    .generic_calling_mode(matched.declaration)
                    .ok_or_else(|| Diagnostic::new(span, "generic calling mode is not ready"))?;
                self.check_call_context(
                    &jai_types::ProcedureType {
                        parameters: Box::new([]),
                        results: Box::new([]),
                        convention,
                        context,
                        variadic: jai_types::Variadic::None,
                    },
                    span,
                )?;
                scope.preview_generic_results(
                    &matched,
                    self.types,
                    &mut self.meta.record_specializations,
                    span,
                )
            }
        }
    }
    pub(crate) fn argument_type(
        &self,
        info: &ArgumentInfo,
        span: Span,
    ) -> Result<TypeId, Diagnostic> {
        Ok(match &info.ty {
            ArgumentType::Known(ty)
            | ArgumentType::StringLiteral(ty)
            | ArgumentType::RecordLiteral { ty: Some(ty), .. }
            | ArgumentType::ArrayLiteral {
                default: Some(ty), ..
            } => *ty,
            ArgumentType::WeakInteger { minimum, maximum }
                if *minimum >= IntegerType::S64.min() && *maximum <= IntegerType::S64.max() =>
            {
                self.types.scalar(ScalarType::Int(IntegerType::S64))
            }
            ArgumentType::WeakFloat { default, .. }
            | ArgumentType::WeakFloatExpression { default, .. } => self.types.float(*default),
            _ => {
                return Err(Diagnostic::new(
                    span,
                    "argument has no context-free default type",
                ));
            }
        })
    }
    fn describe_enum_binary(
        &self,
        operation: syntax::BinaryOp,
        lhs: &ArgumentInfo,
        rhs: &ArgumentInfo,
        span: Span,
    ) -> Option<Result<ArgumentInfo, Diagnostic>> {
        let ty = [lhs, rhs].into_iter().find_map(|value| match value.ty {
            ArgumentType::Known(ty) if matches!(self.types.kind(ty), Ok(TypeKind::Enum(_))) => {
                Some(ty)
            }
            _ => None,
        })?;
        let equality = matches!(
            operation,
            syntax::BinaryOp::Equal | syntax::BinaryOp::NotEqual
        );
        let zero = |value: &ArgumentInfo| {
            matches!(
                value.ty,
                ArgumentType::WeakInteger {
                    minimum: 0,
                    maximum: 0
                }
            ) && matches!(value.constant, Some(ConstantArgument::IntegerLiteral(0)))
        };
        if equality && self.enum_is_flags(ty) && (zero(lhs) || zero(rhs)) {
            return Some(Ok(ArgumentInfo::typed(self.types.scalar(ScalarType::Bool))));
        }
        let matches_enum = |value: &ArgumentInfo| match value.ty {
            ArgumentType::Known(actual) => actual == ty,
            ArgumentType::EnumMember(name) => self.enum_member(ty, name).is_some(),
            _ => false,
        };
        if !matches_enum(lhs) || !matches_enum(rhs) {
            return Some(Err(Diagnostic::new(
                span,
                "enum operation requires values of the same nominal type",
            )));
        }
        Some(match operation {
            syntax::BinaryOp::Equal
            | syntax::BinaryOp::NotEqual
            | syntax::BinaryOp::Less
            | syntax::BinaryOp::LessEqual
            | syntax::BinaryOp::Greater
            | syntax::BinaryOp::GreaterEqual => {
                Ok(ArgumentInfo::typed(self.types.scalar(ScalarType::Bool)))
            }
            syntax::BinaryOp::BitAnd | syntax::BinaryOp::BitOr | syntax::BinaryOp::BitXor
                if self.enum_is_flags(ty) =>
            {
                Ok(ArgumentInfo::typed(ty))
            }
            _ => Err(Diagnostic::new(
                span,
                "enum arithmetic requires an explicit integer conversion",
            )),
        })
    }
    /// A peer can provide a contextual cast target without lowering either operand.
    pub(crate) fn describe_argument_type(
        &mut self,
        expression: &Expression,
    ) -> Result<Option<TypeId>, Diagnostic> {
        let info = self.describe_argument(expression)?;
        Ok(match info.ty {
            ArgumentType::Known(ty)
            | ArgumentType::StringLiteral(ty)
            | ArgumentType::RecordLiteral { ty: Some(ty), .. }
            | ArgumentType::ArrayLiteral {
                explicit: Some(_),
                default: Some(ty),
                ..
            } => Some(ty),
            _ => None,
        })
    }
    pub(crate) fn common_argument(
        &self,
        lhs: ArgumentInfo,
        rhs: ArgumentInfo,
        span: Span,
    ) -> Result<ArgumentInfo, Diagnostic> {
        for (weak, strong) in [(&lhs, &rhs), (&rhs, &lhs)] {
            if let ArgumentType::WeakInteger { minimum, maximum } = weak.ty
                && let ArgumentType::Known(ty) = strong.ty
                && let Ok(TypeKind::Integer(integer)) = self.types.kind(ty)
            {
                if minimum < integer.min() || maximum > integer.max() {
                    return Err(Diagnostic::new(
                        span,
                        "weak integer operand is out of range for the strong operand's type",
                    ));
                }
                return Ok(ArgumentInfo::typed(ty));
            }
        }
        if let (
            ArgumentType::WeakInteger {
                minimum: a,
                maximum: b,
            },
            ArgumentType::WeakInteger {
                minimum: c,
                maximum: d,
            },
        ) = (&lhs.ty, &rhs.ty)
        {
            return Ok(ArgumentInfo {
                ty: ArgumentType::WeakInteger {
                    minimum: (*a).min(*c),
                    maximum: (*b).max(*d),
                },
                constant: None,
            });
        }
        let a = self.argument_type(&lhs, span)?;
        let b = self.argument_type(&rhs, span)?;
        let ty = if a == b {
            a
        } else {
            match (self.types.kind(a), self.types.kind(b)) {
                (Ok(TypeKind::Integer(a)), Ok(TypeKind::Integer(b))) => {
                    self.types
                        .scalar(ScalarType::Int(a.common(*b).ok_or_else(|| {
                            Diagnostic::new(
                                span,
                                "integer argument types have no range-preserving common type",
                            )
                        })?))
                }
                (Ok(TypeKind::Float(a)), Ok(TypeKind::Float(b))) => {
                    self.types
                        .float(if *a == FloatType::F64 || *b == FloatType::F64 {
                            FloatType::F64
                        } else {
                            FloatType::F32
                        })
                }
                _ => {
                    return Err(Diagnostic::new(
                        span,
                        "argument branches have incompatible types",
                    ));
                }
            }
        };
        Ok(ArgumentInfo::typed(ty))
    }
    pub(crate) fn is_float_argument(&self, info: &ArgumentInfo) -> bool {
        match info.ty {
            ArgumentType::WeakFloat { .. } | ArgumentType::WeakFloatExpression { .. } => true,
            ArgumentType::Known(ty) => matches!(self.types.kind(ty), Ok(TypeKind::Float(_))),
            _ => false,
        }
    }
    pub(crate) fn common_float_argument(
        &self,
        lhs: &ArgumentInfo,
        rhs: &ArgumentInfo,
        span: Span,
    ) -> Result<ArgumentInfo, Diagnostic> {
        let strong = |info: &ArgumentInfo| match info.ty {
            ArgumentType::Known(ty) => match self.types.kind(ty) {
                Ok(TypeKind::Float(ty)) => Some(*ty),
                _ => None,
            },
            _ => None,
        };
        let strong_type = match (strong(lhs), strong(rhs)) {
            (Some(FloatType::F64), _) | (_, Some(FloatType::F64)) => Some(FloatType::F64),
            (Some(FloatType::F32), _) | (_, Some(FloatType::F32)) => Some(FloatType::F32),
            _ => None,
        };
        if let Some(ty) = strong_type {
            if !self.float_permits(lhs, ty) || !self.float_permits(rhs, ty) {
                return Err(Diagnostic::new(
                    span,
                    "floating-point operand cannot acquire the strong operand's width",
                ));
            }
            return Ok(ArgumentInfo::typed(self.types.float(ty)));
        }
        let default = |info: &ArgumentInfo| match info.ty {
            ArgumentType::WeakFloat { default, .. }
            | ArgumentType::WeakFloatExpression { default, .. } => default,
            _ => FloatType::F32,
        };
        let default = if default(lhs) == FloatType::F64 || default(rhs) == FloatType::F64 {
            FloatType::F64
        } else {
            FloatType::F32
        };
        let permits_f32 =
            self.float_permits(lhs, FloatType::F32) && self.float_permits(rhs, FloatType::F32);
        let permits_f64 =
            self.float_permits(lhs, FloatType::F64) && self.float_permits(rhs, FloatType::F64);
        if !permits_f64 {
            return Err(Diagnostic::new(
                span,
                "floating-point expression requires weak numeric or typed float operands",
            ));
        }
        Ok(ArgumentInfo {
            ty: ArgumentType::WeakFloatExpression {
                default,
                permits_f32,
                permits_f64,
            },
            constant: None,
        })
    }
    fn float_permits(&self, info: &ArgumentInfo, target: FloatType) -> bool {
        match &info.ty {
            ArgumentType::Known(ty) => {
                matches!(self.types.kind(*ty), Ok(TypeKind::Float(source)) if *source == target || target == FloatType::F64)
            }
            ArgumentType::WeakFloat { spelling, .. } => {
                FloatValue::parse_decimal(target, spelling).is_ok()
            }
            ArgumentType::WeakFloatExpression {
                permits_f32,
                permits_f64,
                ..
            } => match target {
                FloatType::F32 => *permits_f32,
                FloatType::F64 => *permits_f64,
            },
            ArgumentType::WeakInteger { minimum, maximum } => {
                [*minimum, *maximum].into_iter().all(|value| {
                    jai_types::Integer::checked(IntegerType::S64, value)
                        .or_else(|| jai_types::Integer::checked(IntegerType::U64, value))
                        .is_some()
                })
            }
            _ => false,
        }
    }
    fn float_constant_description(
        &self,
        mut info: ArgumentInfo,
        expression: &Expression,
    ) -> ArgumentInfo {
        let round = |target| {
            jai_eval::evaluate_float_paths(expression, target, |path, span| {
                match self.lookup_path(path, span)? {
                    Binding::Constant(value) => Ok(value),
                    _ => Err(Diagnostic::new(
                        span,
                        "argument is not a scalar compile-time constant",
                    )),
                }
            })
            .ok()
        };
        match info.ty {
            ArgumentType::Known(ty) => {
                if let Ok(TypeKind::Float(target)) = self.types.kind(ty)
                    && let Some(value) = round(*target)
                {
                    info.constant = Some(ConstantArgument::Value(BakedValue::Float(value)));
                }
            }
            ArgumentType::WeakFloatExpression { .. } => {
                let f32 = round(FloatType::F32);
                let f64 = round(FloatType::F64);
                if f32.is_some() || f64.is_some() {
                    info.constant = Some(ConstantArgument::FloatExpression { f32, f64 });
                }
            }
            _ => {}
        }
        info
    }
}
