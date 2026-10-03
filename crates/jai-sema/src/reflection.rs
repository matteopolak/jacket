//! Bind type-valued queries without evaluating runtime expressions.
use super::*;
use jai_types::{ReflectionReadiness, TypeKind, reflected_size};
pub(crate) mod schema;
mod storage;

#[derive(Default)]
pub(crate) struct MetaContext {
    pub(crate) compiler_code: crate::compiler_code::CompilerCodeRegistry,
    pub(crate) short_lambda_preview: Option<crate::short_lambdas::PreviewIdentity>,
    pub(crate) external_globals: crate::local_declarations::ExternalGlobals,
    pub(crate) source_procedure_owners: jai_ir::SourceProcedureOwners,
    pub(crate) expression_bindings: crate::expression_bindings::ExpressionBindingAllocator,
    pub(crate) deprecations: crate::deprecation_warnings::DeprecationWarnings<
        crate::deprecation_warnings::DeprecationKey,
    >,
    pub(crate) source_specialization_keys:
        HashMap<ProcedureId, jai_modules::SourceSpecializationKey>,
    pub(crate) pending_source_specializations: Vec<jai_modules::SourceSpecializationKey>,
    pub(crate) using_names: crate::using_directives::UsingState,
    pub(crate) storage_alignments: jai_ir::StorageAlignments,
    pub(crate) debug_sources: jai_ir::DebugSources,
    pub(crate) procedure_hints: HashMap<ProcedureId, jai_types::InlineHint>,
    pub(crate) procedure_phases: jai_ir::ProcedurePhases,
    pub(crate) compiler_source_signatures:
        HashMap<ProcedureId, modules::compiler_intrinsics::SourceCompilerSignature>,
    pub(crate) callbacks: crate::procedure_values::bindings::CallbackRegistry,
    pub(crate) record_specializations: modules::aggregates::parameterized::RecordSpecializations,
    pub(crate) field_default_jobs: modules::field_default_jobs::FieldDefaultJobs,
    pub(crate) local_declarations: crate::local_declarations::LocalDeclarationRegistry,
    pub(crate) codes: crate::metaprogram::CodeRegistry,
    pub(crate) constants: crate::typed_constants::ConstantPool,
    schema: Option<std::sync::Arc<schema::TypeInfoSchema>>,
    storage: HashMap<TypeId, (TypeId, std::sync::Arc<StaticData>, StaticAddress)>,
    storage_policies: HashMap<TypeId, jai_types::RecordReflectionPolicy>,
    storage_builder: StaticDataBuilder,
}

impl MetaContext {
    pub(crate) fn remember_execution(
        &mut self,
        id: ProcedureId,
        execution: jai_types::ProcedureExecution,
    ) {
        if execution == jai_types::ProcedureExecution::CompileTimeOnly {
            self.procedure_phases.insert(id, execution);
        }
    }
    pub(crate) fn remember_inline_hint(&mut self, id: ProcedureId, hint: jai_types::InlineHint) {
        if hint != jai_types::InlineHint::Automatic {
            self.procedure_hints.insert(id, hint);
        }
    }
    pub(crate) fn intern_constant(
        &mut self,
        value: jai_ir::ConstantValue,
    ) -> crate::typed_constants::ConstantId {
        self.constants.insert(value)
    }
    pub(crate) fn constant(
        &self,
        id: crate::typed_constants::ConstantId,
    ) -> Option<&jai_ir::ConstantValue> {
        self.constants.get(id)
    }
}

pub(crate) fn is_semantic_constant(expression: &syntax::Expression) -> bool {
    let mut expressions = vec![expression];
    while let Some(expression) = expressions.pop() {
        match &expression.kind {
            syntax::ExpressionKind::Type(_)
            | syntax::ExpressionKind::TypeQuery { .. }
            | syntax::ExpressionKind::Code(_)
            | syntax::ExpressionKind::CompileTime(_)
            | syntax::ExpressionKind::SourceLocation
            | syntax::ExpressionKind::SourceFile
            | syntax::ExpressionKind::SourceFilepath
            | syntax::ExpressionKind::SourceLine
            | syntax::ExpressionKind::String(_)
            | syntax::ExpressionKind::HereString(_)
            | syntax::ExpressionKind::ArrayLiteral(_)
            | syntax::ExpressionKind::StructLiteral(_)
            | syntax::ExpressionKind::PositionalStructLiteral(_) => return true,
            syntax::ExpressionKind::Unary(_, expression)
            | syntax::ExpressionKind::Cast(_, _, expression)
            | syntax::ExpressionKind::Member {
                base: expression, ..
            } => expressions.push(expression),
            syntax::ExpressionKind::Binary(_, left, right) => {
                expressions.push(left);
                expressions.push(right);
            }
            syntax::ExpressionKind::Index { base, index } => {
                expressions.push(base);
                expressions.push(index);
            }
            _ => {}
        }
    }
    false
}

impl Resolver<'_> {
    pub(crate) fn bind_semantic_constant(
        &mut self,
        constant: &syntax::ConstantDeclaration,
    ) -> Result<(), Diagnostic> {
        if let Some(annotation) = &constant.ty {
            let ty = self.lexical_annotation(annotation, constant.span)?;
            if ty == self.types.code_type() {
                let Expr::Code(id) = self.expr(&constant.initializer)? else {
                    return Err(Diagnostic::new(
                        constant.initializer.span,
                        "a Code constant requires a checked code value",
                    ));
                };
                return self.bind_name(constant.name, Binding::Code(id));
            }
            let value = self.local_typed_constant(&constant.initializer, ty)?;
            self.annotation_value_contract(ty, annotation, constant.span)?;
            let binding =
                crate::compile_time::materialized_binding(value, None, constant.span, self.meta)?;
            return self.bind_name(constant.name, binding);
        }
        if crate::compile_time::is_run_constant(&constant.initializer) {
            return self.bind_run_constant(constant);
        }
        match self.expr(&constant.initializer)? {
            Expr::Code(id) if constant.ty.is_none() => {
                self.bind_name(constant.name, Binding::Code(id))
            }
            Expr::Typed { value, .. } => {
                let value = self.literal_constant(value, constant.span)?;
                let id = self.meta.intern_constant(value);
                self.bind_name(constant.name, Binding::TypedConstant(id))
            }
            Expr::Type(ty) if constant.ty.is_none() => {
                self.bind_name(constant.name, Binding::Type(ty))
            }
            Expr::Int(value) => match value.kind() {
                IntExprKind::Constant(value) => self.bind_name(
                    constant.name,
                    Binding::Constant(ScalarConstant::Int(*value)),
                ),
                _ => self.bind_pure_semantic_constant(
                    constant.name,
                    ValueExpr::Int(value),
                    constant.span,
                ),
            },
            Expr::Bool(BoolExpr::Constant(value)) => self.bind_name(
                constant.name,
                Binding::Constant(ScalarConstant::Bool(value)),
            ),
            Expr::Bool(value) => self.bind_pure_semantic_constant(
                constant.name,
                ValueExpr::Bool(value),
                constant.span,
            ),
            Expr::Float(value) => self.bind_pure_semantic_constant(
                constant.name,
                ValueExpr::Float(value),
                constant.span,
            ),
            _ => Err(Diagnostic::new(
                constant.span,
                "this typed constant cannot be materialized yet",
            )),
        }
    }

    fn bind_pure_semantic_constant(
        &mut self,
        name: Symbol,
        value: ValueExpr,
        span: Span,
    ) -> Result<(), Diagnostic> {
        let value = self.evaluate_pure_constant(value, span).map_err(|mut error| {
            if error.message.starts_with("#if condition requires compile-time values") {
                error.message = "semantic constant requires compile-time values; runtime storage and procedure calls require explicit #run".into();
            }
            error
        })?;
        let binding = crate::compile_time::materialized_binding(value, None, span, self.meta)?;
        self.bind_name(name, binding)
    }
    pub(crate) fn reflected_type_syntax(
        &mut self,
        ty: &syntax::TypeSyntax,
        span: Span,
    ) -> Result<TypeId, Diagnostic> {
        self.lexical_annotation(ty, span)
    }

    pub(crate) fn type_query(
        &mut self,
        query: syntax::TypeQueryKind,
        value: &syntax::Expression,
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        if query == syntax::TypeQueryKind::TypeOf {
            let ty = self.queried_expression_type(value)?;
            return Ok(Expr::Type(ty));
        }
        if query == syntax::TypeQueryKind::CodeOf {
            return Err(Diagnostic::new(
                span,
                "code_of is waiting for immutable checked code capture",
            ));
        }
        let Expr::Type(ty) = self.expr(value)? else {
            return Err(Diagnostic::new(
                value.span,
                "type query requires a type value; use type_of for a runtime expression",
            ));
        };
        match query {
            syntax::TypeQueryKind::SizeOf => {
                match reflected_size(self.types, ty, self.target_layout)
                    .map_err(|error| Diagnostic::new(span, error.to_string()))?
                {
                    ReflectionReadiness::Ready(size) => {
                        let value = IntegerValue::checked(IntegerType::S64, i128::from(size))
                            .ok_or_else(|| {
                                Diagnostic::new(
                                    span,
                                    "target size cannot be represented by Jai int",
                                )
                            })?;
                        Ok(Expr::Int(IntExpr::constant(value)))
                    }
                    ReflectionReadiness::Pending(dependencies) => Err(Diagnostic::new(
                        span,
                        format!(
                            "size_of is waiting for target or nominal layout dependencies: {dependencies:?}"
                        ),
                    )),
                }
            }
            syntax::TypeQueryKind::InitializerOf => {
                let value = self.default_value(ty, span)?;
                self.typed_value(value.into_expression(), ty, span)
            }
            syntax::TypeQueryKind::TypeInfo => self.type_info_expression(ty, span),
            syntax::TypeQueryKind::TypeOf | syntax::TypeQueryKind::CodeOf => {
                unreachable!("handled above")
            }
        }
    }

    pub(crate) fn annotation_expression_type(
        &mut self,
        value: &syntax::Expression,
    ) -> Result<TypeId, Diagnostic> {
        let path = crate::modules::aggregates::types::annotation_query_path(value)?;
        if !self
            .scopes
            .iter()
            .rev()
            .any(|frame| frame.contains_key(&path.root))
            && let Some(ty) =
                self.with_local_constant_source(&path, value.span, |resolver, _, constant| {
                    if let Some(ty) = &constant.ty {
                        return resolver.lexical_annotation(ty, constant.span).map(Some);
                    }
                    Err(Diagnostic::new(
                        value.span,
                        "annotation type_of is waiting for this lexical constant's checked type",
                    ))
                })?
        {
            // The source callback is evaluated in its retained declaration
            // environment; no initializer or compile-time procedure is run.
            return Ok(ty);
        }
        if let Some(Binding::Type(ty)) = self
            .scopes
            .iter()
            .rev()
            .find_map(|frame| frame.get(&path.root))
            .cloned()
        {
            if path.members.is_empty() {
                self.schema_header_type(value.span)?;
                return Ok(self.types.meta_type());
            }
            return self.queried_declared_fields(ty, &path.members, value.span);
        }
        self.queried_expression_type(value)
    }

    pub(crate) fn queried_expression_type(
        &mut self,
        value: &syntax::Expression,
    ) -> Result<TypeId, Diagnostic> {
        if let syntax::ExpressionKind::Name(name) = value.kind
            && let Some(ty) = self.local_declared_type(name, value.span)?
        {
            return Ok(ty);
        }
        // Record member declarations can be queried through the type itself;
        // no instance or runtime field load is created for this operation.
        let path = crate::modules::aggregates::types::annotation_query_path(value).ok();
        if let Some(path) = &path
            && !path.members.is_empty()
            && let Some(Binding::Type(ty)) = self
                .scopes
                .iter()
                .rev()
                .find_map(|scope| scope.get(&path.root))
        {
            return self.queried_declared_fields(*ty, &path.members, value.span);
        }
        if let Some(path) = path
            && !self
                .scopes
                .iter()
                .rev()
                .any(|scope| scope.contains_key(&path.root))
        {
            for count in (0..path.members.len()).rev() {
                let prefix = syntax::NamePath {
                    root: path.root,
                    members: path.members[..count].to_vec(),
                };
                if let Ok(ty) = self.local_type_name(&prefix, value.span) {
                    return self.queried_declared_fields(ty, &path.members[count..], value.span);
                }
            }
        }
        if matches!(value.kind, syntax::ExpressionKind::Context) {
            let context = self.context_expression(value.span)?;
            return self.expression_type(&context, value.span);
        }
        let description = self.describe_argument(value)?;
        self.argument_type(&description, value.span)
    }

    fn queried_declared_fields(
        &self,
        mut ty: TypeId,
        members: &[Symbol],
        span: Span,
    ) -> Result<TypeId, Diagnostic> {
        for member in members {
            let mut pointer_depth = 0;
            while let TypeKind::Pointer(pointee) = self
                .types
                .kind(ty)
                .map_err(|error| Diagnostic::new(span, error.to_string()))?
            {
                pointer_depth += 1;
                if pointer_depth > 256 {
                    return Err(Diagnostic::new(
                        span,
                        "type_of exceeds pointer type depth limit",
                    ));
                }
                ty = *pointee;
            }
            for field in self.field_path(ty, *member, span)? {
                ty = self
                    .types
                    .field_type(field)
                    .map_err(|error| Diagnostic::new(span, error.to_string()))?;
            }
        }
        Ok(ty)
    }
}
