//! Short lambdas publish ordinary checked procedures with canonical signatures.
use super::*;
use crate::local_declarations::LocalCallableSpecialization;
use crate::procedure_values::IndirectCallBinding;
use jai_types::{TypeKind, Variadic};
use std::collections::HashSet;
mod annotations;
mod block_preview;

/// Type facts used only while checking a callback candidate's source expression.
#[derive(Clone, Debug)]
pub(crate) enum PreviewBinding {
    Parameter(TypeId),
    RuntimeCapture,
}

/// A checked source header supplies type facts without a runtime procedure identity.
#[derive(Clone, Copy, Debug)]
pub(crate) struct PreviewIdentity {
    pub(crate) ty: TypeId,
    pub(crate) span: Span,
}

impl Resolver<'_> {
    fn with_named_short_lambda_source<T>(
        &mut self,
        path: &syntax::NamePath,
        span: Span,
        evaluate: impl Fn(
            &mut Resolver<'_>,
            Option<jai_source::DeclarationId>,
            &syntax::ConstantDeclaration,
        ) -> Result<Option<T>, Diagnostic>,
    ) -> Result<Option<T>, Diagnostic> {
        if let Some(value) =
            self.with_local_constant_source(path, span, |definition, _, constant| {
                evaluate(definition, None, constant)
            })?
        {
            return Ok(Some(value));
        }
        let Some(scope) = self.graph_scope else {
            return Ok(None);
        };
        let imported = match self.resolve_local_name(path.root, span)? {
            None => None,
            Some(Binding::Imported(binding)) if path.members.is_empty() => Some(binding),
            Some(Binding::Namespace(module))
            | Some(Binding::Imported(jai_modules::Binding::Module(module)))
                if !path.members.is_empty() =>
            {
                Some(scope.namespace_binding(module, &path.members, span)?)
            }
            Some(_) => return Ok(None),
        };
        let Some((origin, definition_scope, constant)) = scope.short_lambda_source(path, imported)
        else {
            return Ok(None);
        };
        let (file, _, source) = definition_scope.code_origin();
        let definition_context = self
            .compile_time
            .map(|context| context.for_source(context.owner, file, source));
        let result = {
            let mut definition = Resolver {
                debug: crate::debug_capture::Capture::new(Some(source)),
                checks: crate::safety_checks::ActiveChecks::default(),
                context: self.context,
                context_available: self.context_available,
                meta: &mut *self.meta,
                graph_scope: Some(definition_scope),
                compile_time: definition_context.as_ref(),
                target_layout: self.target_layout,
                procedure: self.procedure,
                expression_owner: self.expression_owner.map(|owner| {
                    definition_context
                        .as_ref()
                        .map_or(owner, |context| context.owner)
                }),
                types: &mut *self.types,
                places: &mut *self.places,
                signatures: self.signatures,
                symbols: self.symbols,
                scopes: vec![],
                local_scopes: crate::local_declarations::LocalScopes::default(),
                globals: self.globals,
                locals: vec![],
                span: constant.span,
                results: &[],
                loops: vec![],
                next_loop: 0,
                cleanups: vec![],
                active_push: None,
                next_push: 0,
                deferred_scopes: vec![],
                cleanup_context: None,
            };
            evaluate(&mut definition, Some(origin), &constant)
        };
        if let (Some(parent), Some(child)) = (self.compile_time, definition_context.as_ref()) {
            parent.merge_pending_from(child);
        }
        result.map_err(|error| error.with_fallback_source(source))
    }

    pub(crate) fn named_short_lambda_value(
        &mut self,
        path: &syntax::NamePath,
        expected: TypeId,
        span: Span,
    ) -> Result<Option<Expr>, Diagnostic> {
        self.with_named_short_lambda_source(path, span, |definition, origin, constant| {
            let syntax::ExpressionKind::ShortLambda(source) = &constant.initializer.kind else {
                return Ok(None);
            };
            let annotated = definition.short_lambda_constant_type(constant)?;
            if annotated.is_some_and(|annotated| annotated != expected) {
                return Err(Diagnostic::new(
                    constant.span,
                    "short lambda constant annotation does not match its contextual type",
                ));
            }
            definition
                .short_lambda_value_with_origin(
                    source,
                    constant.initializer.span,
                    Some(expected),
                    None,
                    origin,
                )
                .map(Some)
        })
    }

    pub(crate) fn named_short_lambda_call(
        &mut self,
        path: &syntax::NamePath,
        arguments: &[syntax::CallArgument],
        span: Span,
    ) -> Result<Option<Expr>, Diagnostic> {
        let Some((callee, values, results)) =
            self.named_short_lambda_call_binding(path, arguments, span)?
        else {
            return Ok(None);
        };
        self.indirect_call_expression(callee, values, &results, span)
            .map(Some)
    }

    pub(crate) fn named_short_lambda_call_binding(
        &mut self,
        path: &syntax::NamePath,
        arguments: &[syntax::CallArgument],
        span: Span,
    ) -> Result<Option<IndirectCallBinding>, Diagnostic> {
        let Some((callee, metadata)) = self.named_short_lambda_callee(path, arguments, span)?
        else {
            return Ok(None);
        };
        self.resolve_indirect_call_binding_with_metadata(callee, arguments, span, metadata)
            .map(Some)
    }

    fn named_short_lambda_callee(
        &mut self,
        path: &syntax::NamePath,
        arguments: &[syntax::CallArgument],
        span: Span,
    ) -> Result<
        Option<(
            Expr,
            Option<crate::procedure_values::bindings::CallbackSignature>,
        )>,
        Diagnostic,
    > {
        let source =
            self.with_named_short_lambda_source(path, span, |definition, _, constant| {
                match &constant.initializer.kind {
                    syntax::ExpressionKind::ShortLambda(source) => {
                        let expected = definition.short_lambda_constant_type(constant)?;
                        let annotations = definition.short_lambda_parameter_context(
                            source,
                            expected,
                            constant.initializer.span,
                        )?;
                        let metadata = expected
                            .map(|ty| definition.short_lambda_constant_metadata(constant, ty))
                            .transpose()?
                            .flatten();
                        Ok(Some((source.clone(), annotations, expected, metadata)))
                    }
                    _ => Ok(None),
                }
            })?;
        let Some((source, annotations, expected, metadata)) = source else {
            return Ok(None);
        };
        // Argument types belong to the caller; the source body belongs to its definition.
        let parameters = if let (Some(expected), Some(metadata)) = (expected, metadata.as_ref()) {
            let descriptor = self
                .types
                .procedure_definition(expected)
                .map_err(|error| Diagnostic::new(span, error.to_string()))?
                .clone();
            self.preview_callback_call_results(&descriptor, Some(metadata), arguments, span)?;
            descriptor.parameters.to_vec()
        } else {
            self.short_lambda_argument_types(&source, &annotations, expected, arguments, span)?
        };
        let callee = self
            .with_named_short_lambda_source(path, span, |definition, origin, constant| {
                let syntax::ExpressionKind::ShortLambda(source) = &constant.initializer.kind else {
                    return Ok(None);
                };
                definition
                    .short_lambda_value_with_origin(
                        source,
                        constant.initializer.span,
                        expected,
                        Some(&parameters),
                        origin,
                    )
                    .map(Some)
            })?
            .ok_or_else(|| {
                Diagnostic::new(
                    span,
                    "short lambda source identity changed during specialization",
                )
            })?;
        Ok(Some((callee, metadata)))
    }

    fn short_lambda_constant_type(
        &mut self,
        constant: &syntax::ConstantDeclaration,
    ) -> Result<Option<TypeId>, Diagnostic> {
        constant
            .ty
            .as_ref()
            .map(|annotation| {
                let ty = self.lexical_annotation(annotation, constant.span)?;
                self.types.procedure_definition(ty).map_err(|_| {
                    Diagnostic::new(
                        constant.span,
                        "short lambda constant annotation requires a procedure type",
                    )
                })?;
                Ok(ty)
            })
            .transpose()
    }

    fn short_lambda_constant_metadata(
        &self,
        constant: &syntax::ConstantDeclaration,
        expected: TypeId,
    ) -> Result<Option<crate::procedure_values::bindings::CallbackSignature>, Diagnostic> {
        let Some(annotation) = &constant.ty else {
            return Ok(None);
        };
        self.annotation_value_contract(expected, annotation, constant.span)?
            .and_then(|contract| contract.callback().cloned())
            .map(Some)
            .ok_or_else(|| {
                Diagnostic::new(
                    constant.span,
                    "short lambda constant annotation has no checked callback source contract",
                )
            })
    }

    pub(crate) fn describe_contextual_short_lambda(
        &mut self,
        source: &syntax::ShortLambda,
        targets: &[TypeId],
        span: Span,
    ) -> Result<crate::overloads::ArgumentInfo, Diagnostic> {
        let mut compatible_signatures = Vec::new();
        let mut failure = None;
        for &target in targets {
            match self.preview_short_lambda(source, target, span) {
                Ok(()) => compatible_signatures.push(target),
                Err(error) => {
                    if failure.is_none() {
                        failure = Some(error);
                    }
                }
            }
        }
        if compatible_signatures.is_empty() {
            return Err(failure.unwrap_or_else(|| {
                Diagnostic::new(
                    span,
                    "short lambda requires a concrete callback signature before overload selection",
                )
            }));
        }
        Ok(crate::overloads::ArgumentInfo {
            ty: crate::overloads::ArgumentType::ContextualProcedure {
                compatible_signatures: compatible_signatures.into_boxed_slice(),
            },
            constant: None,
        })
    }

    pub(crate) fn describe_contextual_named_short_lambda(
        &mut self,
        path: &syntax::NamePath,
        targets: &[TypeId],
        span: Span,
    ) -> Result<Option<crate::overloads::ArgumentInfo>, Diagnostic> {
        self.with_named_short_lambda_source(path, span, |definition, _, constant| {
            let syntax::ExpressionKind::ShortLambda(source) = &constant.initializer.kind else {
                return Ok(None);
            };
            let annotated = definition.short_lambda_constant_type(constant)?;
            if let Some(annotated) = annotated {
                if !targets.contains(&annotated) {
                    return Err(Diagnostic::new(
                        constant.span,
                        "short lambda constant annotation does not match a callback target",
                    ));
                }
                return definition
                    .describe_contextual_short_lambda(
                        source,
                        &[annotated],
                        constant.initializer.span,
                    )
                    .map(Some);
            }
            definition
                .describe_contextual_short_lambda(source, targets, constant.initializer.span)
                .map(Some)
        })
    }

    pub(crate) fn preview_short_lambda(
        &mut self,
        source: &syntax::ShortLambda,
        expected: TypeId,
        span: Span,
    ) -> Result<(), Diagnostic> {
        let descriptor = self
            .types
            .procedure_definition(expected)
            .cloned()
            .map_err(|_| Diagnostic::new(span, "short lambda requires a procedure type context"))?;
        let Some(description) =
            self.preview_short_lambda_description(source, &descriptor, false, span)?
        else {
            return Ok(());
        };
        if let [result] = descriptor.results.as_ref() {
            crate::overloads::contextual_conversion(
                self.types,
                self,
                *result,
                &description.ty,
                source.body.span,
            )?;
        } else {
            let result = self.argument_type(&description, source.body.span)?;
            if matches!(self.types.kind(result), Ok(TypeKind::Code | TypeKind::Void)) {
                return Err(Diagnostic::new(
                    source.body.span,
                    "short lambda body requires a runtime value expression",
                ));
            }
        }
        Ok(())
    }

    pub(crate) fn preview_inferred_short_lambda_result(
        &mut self,
        source: &syntax::ShortLambda,
        parameters: &ProcedureType,
        span: Span,
    ) -> Result<Option<crate::overloads::ArgumentInfo>, Diagnostic> {
        let description = self.preview_short_lambda_description(source, parameters, true, span)?;
        if let Some(info) = &description {
            let result = self.argument_type(info, source.body.span)?;
            if matches!(self.types.kind(result), Ok(TypeKind::Code | TypeKind::Void)) {
                return Err(Diagnostic::new(
                    source.body.span,
                    "short lambda result requires a runtime value type",
                ));
            }
        }
        Ok(description)
    }

    pub(crate) fn named_short_lambda_source_present(
        &mut self,
        path: &syntax::NamePath,
        span: Span,
    ) -> Result<bool, Diagnostic> {
        Ok(
            self.with_named_short_lambda_source(path, span, |_, _, constant| {
                Ok(Some(matches!(
                    constant.initializer.kind,
                    syntax::ExpressionKind::ShortLambda(_)
                )))
            })? == Some(true),
        )
    }

    pub(crate) fn preview_inferred_named_short_lambda_result(
        &mut self,
        path: &syntax::NamePath,
        parameters: &ProcedureType,
        span: Span,
    ) -> Result<Option<Option<crate::overloads::ArgumentInfo>>, Diagnostic> {
        self.with_named_short_lambda_source(path, span, |definition, _, constant| {
            let syntax::ExpressionKind::ShortLambda(source) = &constant.initializer.kind else {
                return Ok(None);
            };
            if let Some(expected) = definition.short_lambda_constant_type(constant)? {
                let descriptor = definition
                    .types
                    .procedure_definition(expected)
                    .map_err(|error| Diagnostic::new(constant.span, error.to_string()))?
                    .clone();
                if descriptor.parameters != parameters.parameters
                    || descriptor.convention != parameters.convention
                    || descriptor.context != parameters.context
                    || descriptor.variadic != parameters.variadic
                {
                    return Err(Diagnostic::new(
                        constant.span,
                        "short lambda constant annotation does not match its callback parameters",
                    ));
                }
                definition.preview_short_lambda(source, expected, constant.initializer.span)?;
                let result = match descriptor.results.as_ref() {
                    [] => None,
                    [result] => Some(crate::overloads::ArgumentInfo::typed(*result)),
                    _ => unreachable!("checked short lambda has at most one result"),
                };
                return Ok(Some(result));
            }
            definition
                .preview_inferred_short_lambda_result(source, parameters, constant.initializer.span)
                .map(Some)
        })
    }

    pub(crate) fn preview_short_lambda_signature(
        &mut self,
        source: &syntax::ShortLambda,
        expected: TypeId,
        span: Span,
    ) -> Result<crate::procedure_values::bindings::CallbackSignature, Diagnostic> {
        self.preview_short_lambda(source, expected, span)?;
        let descriptor = self
            .types
            .procedure_definition(expected)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?;
        Ok(crate::procedure_values::bindings::CallbackSignature {
            argument_policy: crate::procedure_values::bindings::CallbackArgumentPolicy::Established,
            ty: expected,
            parameters: source
                .parameters
                .iter()
                .zip(&descriptor.parameters)
                .map(
                    |(parameter, &ty)| crate::procedure_values::bindings::CallbackParameter {
                        name: Some(parameter.name),
                        ty,
                        evaluation: syntax::ParameterEvaluation::Evaluate,
                        default: None,
                    },
                )
                .collect(),
            source_variadic: crate::overloads::CandidateVariadic::None,
            results: descriptor
                .results
                .iter()
                .map(|&ty| ResultSignature {
                    name: None,
                    ty,
                    default: None,
                    usage: syntax::ResultUsage::Optional,
                })
                .collect(),
        })
    }

    pub(crate) fn preview_named_short_lambda_signature(
        &mut self,
        path: &syntax::NamePath,
        expected: TypeId,
        span: Span,
    ) -> Result<Option<crate::procedure_values::bindings::CallbackSignature>, Diagnostic> {
        self.with_named_short_lambda_source(path, span, |definition, _, constant| {
            let syntax::ExpressionKind::ShortLambda(source) = &constant.initializer.kind else {
                return Ok(None);
            };
            if let Some(annotated) = definition.short_lambda_constant_type(constant)? {
                if annotated != expected {
                    return Err(Diagnostic::new(
                        constant.span,
                        "short lambda constant annotation does not match its contextual type",
                    ));
                }
                definition.preview_short_lambda(source, expected, constant.initializer.span)?;
                return definition.short_lambda_constant_metadata(constant, expected);
            }
            definition
                .preview_short_lambda_signature(source, expected, constant.initializer.span)
                .map(Some)
        })
    }

    fn preview_short_lambda_description(
        &mut self,
        source: &syntax::ShortLambda,
        descriptor: &ProcedureType,
        inferred_result: bool,
        span: Span,
    ) -> Result<Option<crate::overloads::ArgumentInfo>, Diagnostic> {
        let original_owner = self.expression_owner;
        let original_identity = self.meta.short_lambda_preview;
        self.expression_owner = None;
        self.meta.short_lambda_preview = None;
        let result =
            self.preview_short_lambda_description_inner(source, descriptor, inferred_result, span);
        self.meta.short_lambda_preview = original_identity;
        self.expression_owner = original_owner;
        result
    }

    fn preview_short_lambda_description_inner(
        &mut self,
        source: &syntax::ShortLambda,
        descriptor: &ProcedureType,
        inferred_result: bool,
        span: Span,
    ) -> Result<Option<crate::overloads::ArgumentInfo>, Diagnostic> {
        if descriptor.parameters.len() != source.parameters.len() {
            return Err(Diagnostic::new(
                span,
                "short lambda parameter count does not match its procedure type",
            ));
        }
        if descriptor.results.len() > 1 || descriptor.variadic != Variadic::None {
            return Err(Diagnostic::new(
                span,
                "short lambdas require a non-variadic procedure with at most one result",
            ));
        }
        let preview_identity = if inferred_result {
            None
        } else {
            Some(PreviewIdentity {
                ty: self
                    .types
                    .procedure(descriptor.clone())
                    .map_err(|error| Diagnostic::new(span, error.to_string()))?,
                span,
            })
        };
        let mut parameter_facts = HashMap::new();
        for (parameter, &ty) in source.parameters.iter().zip(&descriptor.parameters) {
            if parameter_facts
                .insert(
                    parameter.name,
                    Binding::LambdaPreview(PreviewBinding::Parameter(ty)),
                )
                .is_some()
            {
                return Err(Diagnostic::new(
                    parameter.span,
                    "duplicate short lambda parameter name",
                ));
            }
            if let Some(annotation) = &parameter.ty {
                let declared = self.preview_annotation(annotation, parameter.span)?;
                if declared != ty {
                    return Err(Diagnostic::new(
                        parameter.span,
                        "short lambda parameter annotation does not match its contextual type",
                    ));
                }
            }
        }
        let original_scopes = self.scopes.clone();
        let mut preview_scopes = original_scopes.clone();
        for scope in &mut preview_scopes {
            for binding in scope.values_mut() {
                match binding {
                    Binding::Storage(storage)
                        if self.short_lambda_captures_storage(*storage, span)? =>
                    {
                        *binding = Binding::LambdaPreview(PreviewBinding::RuntimeCapture);
                    }
                    Binding::LambdaPreview(PreviewBinding::Parameter(_)) => {
                        *binding = Binding::LambdaPreview(PreviewBinding::RuntimeCapture);
                    }
                    _ => {}
                }
            }
        }
        preview_scopes.push(parameter_facts);
        self.scopes = preview_scopes;
        let original_context_available = self.context_available;
        self.meta.short_lambda_preview = preview_identity;
        self.context_available = descriptor.convention != CallingConvention::C
            && descriptor.context == ContextMode::Implicit;
        let description = (|| {
            let expression_source = match &source.body.kind {
                syntax::ShortLambdaBodyKind::Block(body) => {
                    return self.preview_short_lambda_block(
                        body,
                        descriptor,
                        inferred_result,
                        source.body.span,
                    );
                }
                syntax::ShortLambdaBodyKind::Expression(expression) => expression,
            };
            self.validate_discarded_expression(expression_source)?;
            if inferred_result {
                self.preview_short_lambda_inferred_body(expression_source)
            } else if descriptor.results.is_empty() {
                self.preview_short_lambda_void_body(expression_source)
            } else {
                self.describe_argument(expression_source).map(Some)
            }
        })();
        self.context_available = original_context_available;
        self.scopes = original_scopes;
        description
    }

    fn preview_short_lambda_inferred_body(
        &mut self,
        expression: &syntax::Expression,
    ) -> Result<Option<crate::overloads::ArgumentInfo>, Diagnostic> {
        let results = match &expression.kind {
            syntax::ExpressionKind::Call(name, args) => Some(self.describe_call_results(
                &syntax::NamePath {
                    root: *name,
                    members: vec![],
                },
                args,
                expression.span,
            )?),
            syntax::ExpressionKind::QualifiedCall(path, args) => {
                Some(self.describe_call_results(path, args, expression.span)?)
            }
            syntax::ExpressionKind::CallHint { call, .. } => {
                return self.preview_short_lambda_inferred_body(call);
            }
            _ => None,
        };
        if let Some(results) = results {
            return match results.as_slice() {
                [] => Ok(None),
                [ty] => Ok(Some(crate::overloads::ArgumentInfo::typed(*ty))),
                _ => Err(Diagnostic::new(
                    expression.span,
                    "short lambda expression requires at most one procedure result",
                )),
            };
        }
        self.describe_argument(expression).map(Some)
    }

    fn preview_short_lambda_void_body(
        &mut self,
        expression: &syntax::Expression,
    ) -> Result<Option<crate::overloads::ArgumentInfo>, Diagnostic> {
        let results = match &expression.kind {
            syntax::ExpressionKind::Call(name, args) => {
                Some(self.describe_discarded_call_results(
                    &syntax::NamePath {
                        root: *name,
                        members: vec![],
                    },
                    args,
                    expression.span,
                )?)
            }
            syntax::ExpressionKind::QualifiedCall(path, args) => {
                Some(self.describe_discarded_call_results(path, args, expression.span)?)
            }
            syntax::ExpressionKind::CallHint { call, .. } => {
                return self.preview_short_lambda_void_body(call);
            }
            syntax::ExpressionKind::IndirectCall { .. } => {
                return Err(Diagnostic::new(
                    expression.span,
                    "discarded indirect callback preview requires its source result contract",
                ));
            }
            _ => None,
        };
        if let Some(results) = results {
            return match results.as_slice() {
                [] => Ok(None),
                [ty] => Ok(Some(crate::overloads::ArgumentInfo::typed(*ty))),
                _ => Err(Diagnostic::new(
                    expression.span,
                    "short lambda expression cannot discard multiple procedure results",
                )),
            };
        }
        self.describe_argument(expression).map(Some)
    }

    pub(crate) fn short_lambda_captures_storage(
        &self,
        storage: Storage,
        span: Span,
    ) -> Result<bool, Diagnostic> {
        let mut place = storage.place();
        loop {
            place = match place.kind() {
                PlaceKind::Global(_) => return Ok(false),
                PlaceKind::Field(id) => {
                    self.places
                        .projection(id)
                        .map_err(|error| Diagnostic::new(span, error.to_string()))?
                        .base
                }
                PlaceKind::Index(id) => {
                    self.places
                        .index_projection(id)
                        .map_err(|error| Diagnostic::new(span, error.to_string()))?
                        .base
                }
                PlaceKind::SequenceField(id) => {
                    self.places
                        .sequence_projection(id)
                        .map_err(|error| Diagnostic::new(span, error.to_string()))?
                        .base
                }
                // A lexical dereference alias also depends on its enclosing invocation.
                _ => return Ok(true),
            };
        }
    }

    fn short_lambda_parameter_annotations(
        &mut self,
        source: &syntax::ShortLambda,
    ) -> Result<Vec<Option<TypeId>>, Diagnostic> {
        let mut names = HashSet::new();
        source
            .parameters
            .iter()
            .map(|parameter| {
                if !names.insert(parameter.name) {
                    return Err(Diagnostic::new(
                        parameter.span,
                        "duplicate short lambda parameter name",
                    ));
                }
                parameter
                    .ty
                    .as_ref()
                    .map(|ty| self.lexical_annotation(ty, parameter.span))
                    .transpose()
            })
            .collect()
    }

    fn short_lambda_parameter_context(
        &mut self,
        source: &syntax::ShortLambda,
        expected: Option<TypeId>,
        span: Span,
    ) -> Result<Vec<Option<TypeId>>, Diagnostic> {
        let annotations = self.short_lambda_parameter_annotations(source)?;
        let Some(expected) = expected else {
            return Ok(annotations);
        };
        let descriptor = self
            .types
            .procedure_definition(expected)
            .map_err(|_| Diagnostic::new(span, "short lambda requires a procedure type context"))?;
        if descriptor.parameters.len() != source.parameters.len()
            || descriptor.results.len() > 1
            || descriptor.variadic != Variadic::None
        {
            return Err(Diagnostic::new(
                span,
                "short lambda annotation has incompatible parameters, results, or variadic policy",
            ));
        }
        annotations
            .into_iter()
            .zip(&descriptor.parameters)
            .zip(&source.parameters)
            .map(|((annotation, &ty), source)| {
                if annotation.is_some_and(|annotation| annotation != ty) {
                    Err(Diagnostic::new(
                        source.span,
                        "short lambda parameter annotation does not match its contextual type",
                    ))
                } else {
                    Ok(Some(ty))
                }
            })
            .collect()
    }

    fn short_lambda_argument_types(
        &mut self,
        source: &syntax::ShortLambda,
        annotations: &[Option<TypeId>],
        expected: Option<TypeId>,
        arguments: &[syntax::CallArgument],
        span: Span,
    ) -> Result<Vec<TypeId>, Diagnostic> {
        let mut bound = vec![None; source.parameters.len()];
        let mut positional = 0;
        let mut named = false;
        for argument in arguments {
            if argument.spread {
                return Err(Diagnostic::new(
                    argument.value.span,
                    "short lambda arguments cannot be spread",
                ));
            }
            let index = if let Some(name) = argument.name {
                named = true;
                source
                    .parameters
                    .iter()
                    .position(|parameter| parameter.name == name)
                    .ok_or_else(|| {
                        Diagnostic::new(argument.value.span, "unknown named short lambda argument")
                    })?
            } else {
                if named {
                    return Err(Diagnostic::new(
                        argument.value.span,
                        "positional argument cannot follow a named argument",
                    ));
                }
                let index = positional;
                positional += 1;
                index
            };
            let binding = bound.get_mut(index).ok_or_else(|| {
                Diagnostic::new(argument.value.span, "too many short lambda arguments")
            })?;
            if binding.is_some() {
                return Err(Diagnostic::new(
                    argument.value.span,
                    "duplicate short lambda argument",
                ));
            }
            let ty = if let Some(target) = annotations[index] {
                if let syntax::ExpressionKind::ShortLambda(source) = &argument.value.kind {
                    self.preview_short_lambda(source, target, argument.value.span)?;
                } else {
                    let path = match &argument.value.kind {
                        syntax::ExpressionKind::Name(root) => Some(syntax::NamePath {
                            root: *root,
                            members: vec![],
                        }),
                        syntax::ExpressionKind::QualifiedName(path) => Some(path.clone()),
                        _ => None,
                    };
                    let contextual_lambda =
                        matches!(self.types.kind(target), Ok(TypeKind::Procedure(_)))
                            && if let Some(path) = path {
                                self.describe_contextual_named_short_lambda(
                                    &path,
                                    &[target],
                                    argument.value.span,
                                )?
                                .is_some()
                            } else {
                                false
                            };
                    if !contextual_lambda {
                        let description = self.describe_argument(&argument.value)?;
                        crate::overloads::contextual_conversion(
                            self.types,
                            self,
                            target,
                            &description.ty,
                            argument.value.span,
                        )?;
                    }
                }
                target
            } else {
                let description = self.describe_argument(&argument.value)?;
                self.argument_type(&description, argument.value.span)?
            };
            *binding = Some(ty);
        }
        let parameters: Vec<_> = bound
            .into_iter()
            .map(|ty| ty.ok_or_else(|| Diagnostic::new(span, "missing short lambda argument")))
            .collect::<Result<_, _>>()?;
        let descriptor = if let Some(expected) = expected {
            self.types
                .procedure_definition(expected)
                .map_err(|_| {
                    Diagnostic::new(span, "short lambda requires a procedure type context")
                })?
                .clone()
        } else {
            ProcedureType {
                parameters: parameters.clone().into(),
                results: Box::new([]),
                convention: CallingConvention::Jai,
                context: ContextMode::Implicit,
                variadic: Variadic::None,
            }
        };
        self.check_call_context(&descriptor, span)?;
        Ok(parameters)
    }

    pub(crate) fn short_lambda_call(
        &mut self,
        source: &syntax::ShortLambda,
        source_span: Span,
        arguments: &[syntax::CallArgument],
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        let annotations = self.short_lambda_parameter_annotations(source)?;
        let parameters =
            self.short_lambda_argument_types(source, &annotations, None, arguments, span)?;
        let callee = self.short_lambda_value(source, source_span, None, Some(&parameters))?;
        self.indirect_call(callee, arguments, span)
    }

    pub(crate) fn short_lambda_value(
        &mut self,
        source: &syntax::ShortLambda,
        span: Span,
        expected: Option<TypeId>,
        inferred_parameters: Option<&[TypeId]>,
    ) -> Result<Expr, Diagnostic> {
        self.short_lambda_value_with_origin(source, span, expected, inferred_parameters, None)
    }

    fn short_lambda_value_with_origin(
        &mut self,
        source: &syntax::ShortLambda,
        span: Span,
        expected: Option<TypeId>,
        inferred_parameters: Option<&[TypeId]>,
        origin: Option<jai_source::DeclarationId>,
    ) -> Result<Expr, Diagnostic> {
        let expected_signature = expected
            .map(|ty| {
                self.types.procedure_definition(ty).cloned().map_err(|_| {
                    Diagnostic::new(span, "short lambda requires a procedure type context")
                })
            })
            .transpose()?;
        if let Some(signature) = &expected_signature {
            if signature.parameters.len() != source.parameters.len() {
                return Err(Diagnostic::new(
                    span,
                    "short lambda parameter count does not match its procedure type",
                ));
            }
            if signature.results.len() > 1 || signature.variadic != Variadic::None {
                return Err(Diagnostic::new(
                    span,
                    "short lambdas require a non-variadic procedure with at most one result",
                ));
            }
        }
        if inferred_parameters.is_some_and(|parameters| parameters.len() != source.parameters.len())
        {
            return Err(Diagnostic::new(
                span,
                "short lambda argument count does not match its parameters",
            ));
        }
        let mut names = HashSet::new();
        let mut parameter_signatures = Vec::with_capacity(source.parameters.len());
        for (index, parameter) in source.parameters.iter().enumerate() {
            if !names.insert(parameter.name) {
                return Err(Diagnostic::new(
                    parameter.span,
                    "duplicate short lambda parameter name",
                ));
            }
            let contextual = expected_signature
                .as_ref()
                .map(|signature| signature.parameters[index])
                .or_else(|| inferred_parameters.map(|parameters| parameters[index]));
            let declared = parameter
                .ty
                .as_ref()
                .map(|ty| self.lexical_annotation(ty, parameter.span))
                .transpose()?;
            let ty = match (declared, contextual) {
                (Some(declared), Some(contextual))
                    if expected.is_some() && declared != contextual =>
                {
                    return Err(Diagnostic::new(
                        parameter.span,
                        "short lambda parameter annotation does not match its contextual type",
                    ));
                }
                (Some(ty), _) | (_, Some(ty)) => ty,
                (None, None) => {
                    return Err(Diagnostic::new(
                        parameter.span,
                        "short lambda parameter type requires a call or procedure type context",
                    ));
                }
            };
            parameter_signatures.push(ParameterSignature {
                name: parameter.name,
                ty,
                default: None,
                evaluation: syntax::ParameterEvaluation::Evaluate,
            });
        }
        let convention = expected_signature
            .as_ref()
            .map_or(CallingConvention::Jai, |signature| signature.convention);
        let context = expected_signature
            .as_ref()
            .map_or(ContextMode::Implicit, |signature| signature.context);
        let mut result_signatures: Vec<_> = expected_signature
            .as_ref()
            .into_iter()
            .flat_map(|signature| &signature.results)
            .map(|&ty| ResultSignature {
                name: None,
                ty,
                default: None,
                usage: syntax::ResultUsage::Optional,
            })
            .collect();
        let specialization = match expected {
            Some(ty) => LocalCallableSpecialization::Expected(ty),
            None => LocalCallableSpecialization::InferredParameters(
                self.types
                    .procedure(ProcedureType {
                        parameters: parameter_signatures
                            .iter()
                            .map(|parameter| parameter.ty)
                            .collect(),
                        results: vec![].into(),
                        convention,
                        context,
                        variadic: Variadic::None,
                    })
                    .map_err(|error| Diagnostic::new(span, error.to_string()))?,
            ),
        };
        let procedure = match origin {
            Some(origin) => self.reserve_graph_anonymous_procedure(origin, span, specialization)?,
            None => self.reserve_local_anonymous_procedure(span, specialization)?,
        };
        if let Some(ty) = expected {
            self.remember_anonymous_procedure_source(procedure, ty, span)?;
        }
        if let Some(signature) = self.meta.local_declarations.signature(procedure)
            && self
                .meta
                .local_declarations
                .ready_procedure(procedure)
                .is_some()
        {
            return self.typed_value(
                ValueExpr::ProcedureValue {
                    procedure,
                    ty: signature.ty,
                },
                signature.ty,
                span,
            );
        }
        self.meta
            .local_declarations
            .begin_anonymous_procedure(procedure, span)?;
        let mut scopes = self.scopes.clone();
        scopes.push(HashMap::new());
        let local_scopes = self.local_scopes.for_procedure(procedure);
        let child_context = self.compile_time.map(|context| {
            let (file, source) = self
                .graph_scope
                .map(|scope| {
                    let (file, _, source) = scope.code_origin();
                    (file, source)
                })
                .unwrap_or((context.file, context.source));
            context.for_source(procedure, file, source)
        });
        self.meta.storage_alignments.clear_procedure(procedure);
        let lowered = (|| {
            let mut debug = crate::debug_capture::Capture::new(self.debug.source());
            debug.enter_policy(self.debug.policy());
            let mut child = Resolver {
                debug,
                checks: self.checks,
                context: self.context,
                context_available: convention != CallingConvention::C
                    && context == ContextMode::Implicit,
                meta: &mut *self.meta,
                graph_scope: self.graph_scope,
                compile_time: child_context.as_ref(),
                target_layout: self.target_layout,
                procedure,
                expression_owner: Some(procedure),
                types: &mut *self.types,
                places: &mut *self.places,
                signatures: self.signatures,
                symbols: self.symbols,
                scopes,
                local_scopes,
                globals: self.globals,
                locals: vec![],
                span,
                results: &result_signatures,
                loops: vec![],
                next_loop: 0,
                cleanups: vec![],
                active_push: None,
                next_push: 0,
                deferred_scopes: vec![],
                cleanup_context: None,
            };
            let mut parameters = Vec::with_capacity(parameter_signatures.len());
            for (parameter, source_parameter) in parameter_signatures.iter().zip(&source.parameters)
            {
                let local = child.declare_typed(parameter.name, parameter.ty)?;
                if let Some(annotation) = &source_parameter.ty {
                    child.bind_callback_annotation(
                        local.place(),
                        annotation,
                        source_parameter.span,
                    )?;
                }
                parameters.push(local);
            }
            let (body, inferred_result) = match &source.body.kind {
                syntax::ShortLambdaBodyKind::Block(statements) => {
                    let body = child.block(statements, false)?;
                    if !result_signatures.is_empty() && body.flow != Flow::Terminates {
                        return Err(Diagnostic::new(
                            source.body.span,
                            "value-returning short lambda block may reach its end",
                        ));
                    }
                    (body, None)
                }
                syntax::ShortLambdaBodyKind::Expression(expression_source) => {
                    if expected_signature
                        .as_ref()
                        .is_some_and(|signature| signature.results.is_empty())
                        && let Some(statement) = child.discard_call_results(expression_source)?
                    {
                        let body = Block {
                            statements: vec![
                                statement,
                                Statement::Exit(Exit {
                                    cleanups: vec![],
                                    transfer: Transfer::ReturnVoid,
                                }),
                            ],
                            flow: Flow::Terminates,
                        };
                        return Ok((parameters, child.locals, child.cleanups, body, None));
                    }
                    let expression = match result_signatures.as_slice() {
                        [result] => child.expr_expected(expression_source, result.ty)?,
                        [] => child.expr(expression_source)?,
                        _ => unreachable!("short lambda result count was checked"),
                    };
                    let expression = child.runtime_type_expression(expression, source.body.span)?;
                    let mut statements = Vec::new();
                    let inferred_result = if expected_signature
                        .as_ref()
                        .is_some_and(|signature| signature.results.is_empty())
                    {
                        statements.push(match expression {
                            Expr::Void(call) => Statement::CallVoid(call),
                            Expr::IndirectVoid {
                                inline_hint,
                                callee,
                                arguments,
                            } => Statement::IndirectCallResults {
                                inline_hint,
                                callee,
                                arguments,
                                destinations: vec![],
                            },
                            value => Statement::DiscardValue(value.value(source.body.span)?),
                        });
                        statements.push(Statement::Exit(Exit {
                            cleanups: vec![],
                            transfer: Transfer::ReturnVoid,
                        }));
                        None
                    } else if matches!(expression, Expr::Void(_) | Expr::IndirectVoid { .. })
                        && expected.is_none()
                    {
                        statements.push(match expression {
                            Expr::Void(call) => Statement::CallVoid(call),
                            Expr::IndirectVoid {
                                inline_hint,
                                callee,
                                arguments,
                            } => Statement::IndirectCallResults {
                                inline_hint,
                                callee,
                                arguments,
                                destinations: vec![],
                            },
                            _ => unreachable!(),
                        });
                        statements.push(Statement::Exit(Exit {
                            cleanups: vec![],
                            transfer: Transfer::ReturnVoid,
                        }));
                        None
                    } else {
                        let ty = result_signatures.first().map_or_else(
                            || child.expression_type(&expression, source.body.span),
                            |result| Ok(result.ty),
                        )?;
                        if matches!(child.types.kind(ty), Ok(TypeKind::Code | TypeKind::Void)) {
                            return Err(Diagnostic::new(
                                source.body.span,
                                "short lambda result requires a runtime value type",
                            ));
                        }
                        let value = child.coerce_value(expression, ty, source.body.span)?;
                        statements.push(Statement::Exit(Exit {
                            cleanups: vec![],
                            transfer: Transfer::ReturnValues(vec![value]),
                        }));
                        Some(ty)
                    };
                    (
                        Block {
                            statements,
                            flow: Flow::Terminates,
                        },
                        inferred_result,
                    )
                }
            };
            Ok((
                parameters,
                child.locals,
                child.cleanups,
                body,
                inferred_result,
            ))
        })();
        if let (Some(parent), Some(child)) = (self.compile_time, child_context.as_ref()) {
            parent.merge_pending_from(child);
        }
        self.meta
            .local_declarations
            .end_anonymous_procedure(procedure);
        let (parameters, locals, cleanups, body, inferred_result) = lowered?;
        if expected.is_none()
            && let Some(ty) = inferred_result
        {
            result_signatures.push(ResultSignature {
                name: None,
                ty,
                default: None,
                usage: syntax::ResultUsage::Optional,
            });
        }
        let ty = match expected {
            Some(ty) => ty,
            None => self
                .types
                .procedure(ProcedureType {
                    parameters: parameter_signatures
                        .iter()
                        .map(|parameter| parameter.ty)
                        .collect(),
                    results: result_signatures.iter().map(|result| result.ty).collect(),
                    convention,
                    context,
                    variadic: Variadic::None,
                })
                .map_err(|error| Diagnostic::new(span, error.to_string()))?,
        };
        self.remember_anonymous_procedure_source(procedure, ty, span)?;
        let signature = Signature {
            id: procedure,
            ty,
            parameters: parameter_signatures,
            source_variadic: crate::overloads::CandidateVariadic::None,
            results: result_signatures,
        };
        self.meta.local_declarations.publish_generated(
            signature,
            Procedure {
                id: procedure,
                signature: ty,
                parameters,
                locals,
                cleanups,
                body,
            },
        )?;
        self.typed_value(ValueExpr::ProcedureValue { procedure, ty }, ty, span)
    }
}
