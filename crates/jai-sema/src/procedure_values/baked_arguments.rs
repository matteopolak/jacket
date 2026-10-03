//! Partial applications keep checked source formals and call the original target.
use super::*;
use jai_ir::ConstantValue;
use jai_source::{DeclarationId, SourceSpan};
use jai_types::TypeView;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum BakedCallableOrigin {
    Module {
        declaration: DeclarationId,
        file: jai_modules::FileInstanceId,
    },
    Local(crate::local_declarations::LocalDeclarationId),
}

/// The declaration and environment are retained independently of the wrapper ABI.
#[derive(Clone)]
pub(crate) struct BakedProcedureTarget {
    pub(crate) origin: BakedCallableOrigin,
    pub(crate) source: SourceSpan,
    pub(crate) signature: Signature,
    /// Names, defaults and obligations come from this actual checked use.
    pub(crate) metadata: super::bindings::CallbackSignature,
    /// Maps each surviving signature formal to the original source header.
    pub(crate) source_formals: Vec<usize>,
    pub(crate) source_arguments: Option<Vec<syntax::CallArgument>>,
}

pub(super) fn checked_use_metadata(
    target: &BakedProcedureTarget,
    bound: &[BakedProcedureArgument],
    wrapper_ty: TypeId,
    types: &dyn TypeView,
) -> Result<super::bindings::CallbackSignature, Diagnostic> {
    let assigned = assigned_formals(target, bound, types)?;
    let metadata = &target.metadata;
    if metadata.ty != target.signature.ty
        || metadata.source_variadic != target.signature.source_variadic
        || metadata.parameters.len() != target.signature.parameters.len()
        || metadata
            .parameters
            .iter()
            .zip(&target.signature.parameters)
            .any(|(policy, original)| {
                policy.ty != original.ty || policy.evaluation != original.evaluation
            })
        || metadata.results.len() != target.signature.results.len()
        || metadata
            .results
            .iter()
            .zip(&target.signature.results)
            .any(|(policy, original)| policy.ty != original.ty)
    {
        return Err(Diagnostic::at_source(
            target.source,
            "partial callable policy lacks its checked original source-formal projection",
        ));
    }
    let mut metadata = metadata.clone();
    metadata.ty = wrapper_ty;
    metadata.parameters = metadata
        .parameters
        .into_iter()
        .enumerate()
        .filter_map(|(formal, parameter)| assigned[formal].is_none().then_some(parameter))
        .collect();
    Ok(metadata)
}

#[derive(Clone)]
pub(crate) struct BakedProcedureArgument {
    pub(crate) origin: BakedCallableOrigin,
    pub(crate) formal: usize,
    pub(crate) source_formal: usize,
    pub(crate) value: ConstantValue,
    pub(crate) source: SourceSpan,
    pub(crate) contract: Option<super::contracts::ValueContract>,
    pub(crate) policy: Option<super::contracts::CallablePolicyKey>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct BakedWrapperKey {
    origin: BakedCallableOrigin,
    target: ProcedureId,
    target_type: TypeId,
    arguments: Vec<(
        usize,
        ConstantValue,
        Option<super::contracts::CallablePolicyKey>,
    )>,
}

#[derive(Default)]
pub(crate) struct BakedWrappers {
    procedures: HashMap<BakedWrapperKey, ProcedureId>,
}

fn wrapper_key(
    target: &BakedProcedureTarget,
    bound: &[BakedProcedureArgument],
    types: &dyn TypeView,
) -> Result<BakedWrapperKey, Diagnostic> {
    let mut arguments = bound
        .iter()
        .map(|argument| {
            let value = crate::polymorphism::normalize_constant(argument.value.clone(), types)
                .map_err(|error| Diagnostic::at_source(argument.source, error.to_string()))?;
            Ok((argument.source_formal, value, argument.policy.clone()))
        })
        .collect::<Result<Vec<_>, Diagnostic>>()?;
    arguments.sort_by_key(|(formal, _, _)| *formal);
    Ok(BakedWrapperKey {
        origin: target.origin,
        target: target.signature.id,
        target_type: target.signature.ty,
        arguments,
    })
}

fn assigned_formals(
    target: &BakedProcedureTarget,
    bound: &[BakedProcedureArgument],
    types: &dyn TypeView,
) -> Result<Vec<Option<ConstantValue>>, Diagnostic> {
    let signature = &target.signature;
    if target.source_formals.len() != signature.parameters.len()
        || target
            .source_formals
            .windows(2)
            .any(|pair| pair[0] >= pair[1])
    {
        return Err(Diagnostic::at_source(
            target.source,
            "baked target lacks its checked original source-formal projection",
        ));
    }
    let descriptor = types
        .procedure_definition(signature.ty)
        .map_err(|error| Diagnostic::at_source(target.source, error.to_string()))?;
    let evaluated = signature
        .parameters
        .iter()
        .filter(|parameter| parameter.evaluation == syntax::ParameterEvaluation::Evaluate)
        .map(|parameter| parameter.ty)
        .collect::<Vec<_>>();
    if evaluated.as_slice() != descriptor.parameters.as_ref()
        || signature
            .results
            .iter()
            .map(|result| result.ty)
            .collect::<Vec<_>>()
            .as_slice()
            != descriptor.results.as_ref()
    {
        return Err(Diagnostic::at_source(
            target.source,
            "baked procedure source formals differ from its checked signature",
        ));
    }
    if signature.source_variadic != crate::overloads::CandidateVariadic::None
        || descriptor.variadic != jai_types::Variadic::None
    {
        return Err(Diagnostic::at_source(
            target.source,
            "baking a variadic procedure requires retained source pack binding",
        ));
    }
    let mut assigned = vec![None; signature.parameters.len()];
    for argument in bound {
        if argument.origin != target.origin {
            return Err(Diagnostic::at_source(
                argument.source,
                "baked argument belongs to another source declaration",
            ));
        }
        let Some(parameter) = signature.parameters.get(argument.formal) else {
            return Err(Diagnostic::at_source(
                argument.source,
                "baked argument does not identify an original source formal",
            ));
        };
        if target.source_formals[argument.formal] != argument.source_formal {
            return Err(Diagnostic::at_source(
                argument.source,
                "baked argument does not match its original source formal",
            ));
        }
        if parameter.evaluation == syntax::ParameterEvaluation::Discard {
            return Err(Diagnostic::at_source(
                argument.source,
                "#bake_arguments cannot supply a discarded parameter",
            ));
        }
        if parameter.ty != argument.value.ty {
            return Err(Diagnostic::at_source(
                argument.source,
                "baked argument type differs from its checked source formal",
            ));
        }
        // The shared normalizer validates nominal and aggregate shapes. The
        // program finalizer separately checks every nested procedure identity.
        crate::polymorphism::BakedValue::runtime(argument.value.clone(), types)
            .map_err(|error| Diagnostic::at_source(argument.source, error.to_string()))?;
        if assigned[argument.formal]
            .replace(argument.value.clone())
            .is_some()
        {
            return Err(Diagnostic::at_source(
                argument.source,
                "duplicate baked parameter",
            ));
        }
    }
    Ok(assigned)
}

/// This is an actual body containing a direct checked call, never a renamed
/// target or a prototype with a smaller signature.
pub(crate) fn checked_wrapper(
    types: &mut TypeRegistry,
    owner: ProcedureId,
    target: &BakedProcedureTarget,
    bound: &[BakedProcedureArgument],
) -> Result<(Signature, Procedure), Diagnostic> {
    let assigned = assigned_formals(target, bound, types)?;
    let descriptor = types
        .procedure_definition(target.signature.ty)
        .map_err(|error| Diagnostic::at_source(target.source, error.to_string()))?
        .clone();
    let formals = target
        .signature
        .parameters
        .iter()
        .enumerate()
        .filter(|(index, _)| assigned[*index].is_none())
        .map(|(_, parameter)| parameter.clone())
        .collect::<Vec<_>>();
    let ty = types
        .procedure(ProcedureType {
            parameters: formals
                .iter()
                .filter(|parameter| parameter.evaluation == syntax::ParameterEvaluation::Evaluate)
                .map(|parameter| parameter.ty)
                .collect(),
            results: descriptor.results,
            return_abi: descriptor.return_abi,
            convention: descriptor.convention,
            context: descriptor.context,
            variadic: jai_types::Variadic::None,
        })
        .map_err(|error| Diagnostic::at_source(target.source, error.to_string()))?;
    let mut parameters = Vec::new();
    let mut arguments = Vec::new();
    let mut original_runtime_index = 0;
    for (formal, parameter) in target.signature.parameters.iter().enumerate() {
        if parameter.evaluation == syntax::ParameterEvaluation::Discard {
            continue;
        }
        let value = if let Some(value) = &assigned[formal] {
            value.clone().into_expression()
        } else {
            let local = Local::new_typed(owner, parameters.len(), parameter.ty, types)
                .map_err(|error| Diagnostic::at_source(target.source, error.to_string()))?;
            parameters.push(local);
            ValueExpr::Load(local.place())
        };
        arguments.push((ParameterId::new(original_runtime_index), value));
        original_runtime_index += 1;
    }
    let call = Call::new(target.signature.id, arguments);
    let mut locals = parameters.clone();
    let mut returns = Vec::new();
    let mut destinations = Vec::new();
    for result in &target.signature.results {
        let local = Local::new_typed(owner, locals.len(), result.ty, types)
            .map_err(|error| Diagnostic::at_source(target.source, error.to_string()))?;
        locals.push(local);
        returns.push(ValueExpr::Load(local.place()));
        destinations.push(Some(local.place()));
    }
    let (invocation, transfer) = if returns.is_empty() {
        (Statement::CallVoid(call), Transfer::ReturnVoid)
    } else {
        (
            Statement::CallResults {
                call,
                destinations,
            },
            Transfer::ReturnValues(returns),
        )
    };
    let signature = Signature {
        id: owner,
        ty,
        parameters: formals,
        results: target.signature.results.clone(),
        source_variadic: crate::overloads::CandidateVariadic::None,
    };
    let procedure = Procedure {
        id: owner,
        signature: ty,
        parameters,
        locals,
        cleanups: vec![],
        body: Block {
            flow: Flow::Terminates,
            statements: vec![
                invocation,
                Statement::Exit(Exit {
                    cleanups: vec![],
                    transfer,
                }),
            ],
        },
    };
    Ok((signature, procedure))
}

pub(super) struct BakedGenericMatch<'source> {
    pub(super) candidate: crate::overloads::Candidate,
    pub(super) slots: Vec<
        Option<(
            &'source syntax::CallArgument,
            crate::overloads::ArgumentInfo,
        )>,
    >,
    pub(super) matched: crate::overloads::Match,
    pub(super) descriptions: Vec<crate::overloads::Argument>,
}

impl Resolver<'_> {
    pub(super) fn match_baked_generic_procedure<'source>(
        &mut self,
        callee: &syntax::Expression,
        supplied: &'source [syntax::CallArgument],
        span: Span,
        materialize: bool,
    ) -> Result<Option<BakedGenericMatch<'source>>, Diagnostic> {
        use crate::overloads::{Argument, ArgumentInfo, ConstantArgument};
        use crate::polymorphism::{BakedValue, Substitution};
        let path = match &callee.kind {
            syntax::ExpressionKind::Name(root) => syntax::NamePath {
                root: *root,
                members: vec![],
            },
            syntax::ExpressionKind::QualifiedName(path) => path.clone(),
            _ => return Ok(None),
        };
        if self.local_name_present(path.root) {
            return Ok(None);
        }
        let Some(scope) = self.graph_scope else {
            return Ok(None);
        };
        let Ok(declarations) = scope.callable_declarations(&path, callee.span) else {
            return Ok(None);
        };
        let [declaration] = declarations.as_slice() else {
            return Err(Diagnostic::new(
                callee.span,
                "baking an overload set requires a selected original callable",
            ));
        };
        if scope.concrete_signature(*declaration).is_some() {
            return Ok(None);
        }
        let candidate = scope.candidate(*declaration, self.types, callee.span)?;
        if candidate.variadic != crate::overloads::CandidateVariadic::None {
            return Err(Diagnostic::new(
                span,
                "baking a generic variadic target requires retained source pack binding",
            ));
        }
        let mut slots = vec![None; candidate.parameters.len()];
        for argument in supplied {
            let name = argument.name.ok_or_else(|| {
                Diagnostic::new(
                    argument.value.span,
                    "#bake_arguments requires named constant arguments",
                )
            })?;
            if argument.spread {
                return Err(Diagnostic::new(
                    argument.value.span,
                    "#bake_arguments cannot spread a runtime pack",
                ));
            }
            let formal = candidate
                .parameters
                .iter()
                .position(|parameter| parameter.name == name)
                .ok_or_else(|| Diagnostic::new(argument.value.span, "unknown baked parameter"))?;
            if candidate.parameters[formal].evaluation == syntax::ParameterEvaluation::Discard {
                return Err(Diagnostic::new(
                    argument.value.span,
                    "#bake_arguments cannot supply a discarded parameter",
                ));
            }
            let info = self.describe_argument(&argument.value)?;
            if !info.is_compile_time_constant() {
                if !self.optional_baking_needs_materialization(&argument.value)? {
                    return Err(Diagnostic::new(
                        argument.value.span,
                        "baked argument must be constant",
                    ));
                }
            }
            if slots[formal].replace((argument, info)).is_some() {
                return Err(Diagnostic::new(
                    argument.value.span,
                    "duplicate baked parameter",
                ));
            }
        }
        for (formal, parameter) in candidate.parameters.iter().enumerate() {
            if slots[formal].is_none() && parameter.baking == syntax::ParameterBaking::Required {
                return Err(Diagnostic::new(
                    span,
                    "baked generic procedure still has an unbound required compile-time formal",
                ));
            }
        }
        // First retain genuine supplied Type/count bindings. Unbound runtime
        // formals are signature promises, not invented runtime argument values.
        let mut initial = Substitution::default();
        for (formal, parameter) in candidate.parameters.iter().enumerate() {
            let Some((argument, info)) = &slots[formal] else {
                continue;
            };
            if parameter.baking != syntax::ParameterBaking::None && info.is_compile_time_constant()
            {
                if let Some(ConstantArgument::Value(BakedValue::Type(ty))) = &info.constant {
                    initial.bind_type(parameter.name, *ty);
                }
                let value = crate::overloads::bake(
                    self.types,
                    &parameter.ty,
                    info,
                    &initial,
                    argument.value.span,
                )?;
                initial.bind_constant(parameter.name, value);
            }
        }
        if materialize {
            // Validate the whole named header before a selected recipe can run.
            // Source argument order remains the preparation order; ready genuine
            // Type/count arguments already belong to the defining substitution.
            for argument in supplied {
                let formal = candidate
                    .parameters
                    .iter()
                    .position(|parameter| Some(parameter.name) == argument.name)
                    .expect("the original named header was validated");
                let (_, info) = slots[formal]
                    .as_mut()
                    .expect("the original supplied formal was retained");
                if info.is_compile_time_constant() {
                    continue;
                }
                let expected = scope
                    .materialize_pattern(
                        &candidate.parameters[formal].ty,
                        &initial,
                        self.types,
                        &mut self.meta.record_specializations,
                        argument.value.span,
                    )
                    .or_else(|_| self.argument_type(info, argument.value.span))?;
                *info = self.materialize_selected_source_constant(&argument.value, expected)?;
                if candidate.parameters[formal].baking != syntax::ParameterBaking::None {
                    if let Some(ConstantArgument::Value(BakedValue::Type(ty))) = &info.constant {
                        initial.bind_type(candidate.parameters[formal].name, *ty);
                    }
                    let value = crate::overloads::bake(
                        self.types,
                        &candidate.parameters[formal].ty,
                        info,
                        &initial,
                        argument.value.span,
                    )?;
                    initial.bind_constant(candidate.parameters[formal].name, value);
                }
            }
        }
        let mut descriptions = Vec::with_capacity(candidate.parameters.len());
        for (formal, parameter) in candidate.parameters.iter().enumerate() {
            let (info, location) = match &slots[formal] {
                Some((argument, info)) => (info.clone(), argument.value.span),
                None => {
                    if parameter.baking == syntax::ParameterBaking::Required {
                        return Err(Diagnostic::new(
                            span,
                            "baked generic procedure still has an unbound required compile-time formal",
                        ));
                    }
                    let ty = scope
                        .materialize_pattern(
                            &parameter.ty,
                            &initial,
                            self.types,
                            &mut self.meta.record_specializations,
                            span,
                        )
                        .map_err(|error| {
                            Diagnostic::new(
                                span,
                                format!(
                                    "baked target needs a concrete remaining signature: {}",
                                    error.message
                                ),
                            )
                        })?;
                    (ArgumentInfo::typed(ty), callee.span)
                }
            };
            descriptions.push(Argument {
                name: Some(parameter.name),
                spread: false,
                info,
                span: location,
            });
        }
        let mut preview_candidate = candidate.clone();
        if !materialize {
            for (formal, slot) in slots.iter().enumerate() {
                if slot
                    .as_ref()
                    .is_some_and(|(_, info)| !info.is_compile_time_constant())
                {
                    preview_candidate.parameters[formal].baking = syntax::ParameterBaking::None;
                }
            }
        }
        let matched = crate::overloads::match_candidate_with_nominals(
            self.types,
            self,
            &preview_candidate,
            &descriptions,
            span,
        )?;
        Ok(Some(BakedGenericMatch {
            candidate,
            slots,
            matched,
            descriptions,
        }))
    }

    fn prepare_baked_generic_procedure(
        &mut self,
        callee: &syntax::Expression,
        supplied: &[syntax::CallArgument],
        span: Span,
    ) -> Result<Option<Expr>, Diagnostic> {
        let Some(BakedGenericMatch {
            candidate,
            slots,
            matched,
            descriptions,
        }) = self.match_baked_generic_procedure(callee, supplied, span, true)?
        else {
            return Ok(None);
        };
        let scope = self
            .graph_scope
            .expect("a generic match retains its module origin");
        let matched = self.refine_declaration_match(&candidate, &descriptions, matched, span)?;
        let signature = self.materialize_declaration_match(matched.clone(), span)?;
        let mut target = scope.baked_procedure_target(signature.id).ok_or_else(|| {
            Diagnostic::new(
                span,
                "selected baked generic target has no original source declaration",
            )
        })?;
        target.source_arguments = Some(supplied.to_vec());
        let mut bound = Vec::new();
        for (original_formal, slot) in slots.iter().enumerate() {
            let Some((argument, info)) = slot else {
                continue;
            };
            let original = &candidate.parameters[original_formal];
            // A compile-time source formal already belongs to the selected
            // body's substitution and has no original runtime ABI slot.
            if original.is_baked(&matched.substitution) {
                continue;
            }
            let formal = signature
                .parameters
                .iter()
                .position(|parameter| parameter.name == original.name)
                .ok_or_else(|| {
                    Diagnostic::new(
                        argument.value.span,
                        "selected signature lost its original source formal",
                    )
                })?;
            let value = crate::overloads::bake(
                self.types,
                &crate::overloads::TypePattern::Concrete(signature.parameters[formal].ty),
                info,
                &matched.substitution,
                argument.value.span,
            )?
            .into_runtime(signature.parameters[formal].ty, self.types)
            .map_err(|error| Diagnostic::new(argument.value.span, error.to_string()))?;
            let source = self.debug.source().unwrap_or_else(|| scope.source());
            let checked = value.clone().into_expression();
            let contract =
                match self.selected_source_constant_contract(&argument.value, value.ty)? {
                    Some(contract) => contract,
                    None => self.callback_expression_contract(
                        &argument.value,
                        &checked,
                        argument.value.span,
                    )?,
                };
            let policy = self.specialization_callable_policy(
                &argument.value,
                value.ty,
                argument.value.span,
            )?;
            bound.push(BakedProcedureArgument {
                origin: target.origin,
                formal,
                source_formal: original_formal,
                value,
                contract,
                policy,
                source: SourceSpan {
                    source,
                    span: argument.value.span,
                },
            });
        }
        if bound.is_empty() {
            let returned =
                self.call_result_contracts_for_source(signature.id, &[], Some(supplied), span, 0)?;
            let metadata = checked_use_metadata(&target, &bound, signature.ty, self.types)?;
            let value = self.bind_baked_wrapper_contract(&signature, metadata, returned, span)?;
            return self.typed_value(value, signature.ty, span).map(Some);
        }
        self.publish_baked_procedure_wrapper(&target, &bound, span)
            .map(Some)
    }

    pub(crate) fn baked_procedure_value(
        &mut self,
        callee: &syntax::Expression,
        arguments: &[syntax::CallArgument],
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        if let Some(value) = self.prepare_baked_generic_procedure(callee, arguments, span)? {
            return Ok(value);
        }
        let target_value = self.expr(callee)?;
        let target_ty = self.expression_type(&target_value, callee.span)?;
        let target_value = self.coerce_value(target_value, target_ty, callee.span)?;
        let metadata = self
            .callback_expression_contract(callee, &target_value, callee.span)?
            .and_then(|contract| contract.callback().cloned())
            .ok_or_else(|| {
                Diagnostic::new(
                    callee.span,
                    "baked target has no checked callable use policy",
                )
            })?;
        let constant = self
            .literal_constant(target_value, callee.span)
            .map_err(|_| {
                Diagnostic::new(
                    callee.span,
                    "#bake_arguments requires a constant checked procedure target",
                )
            })?;
        let ConstantKind::Procedure(target_id) = constant.kind else {
            return Err(Diagnostic::new(
                callee.span,
                "#bake_arguments target is not a checked procedure",
            ));
        };
        let mut target = self
            .meta
            .local_declarations
            .baked_procedure_target(target_id)
            .or_else(|| {
                self.graph_scope
                    .and_then(|scope| scope.baked_procedure_target(target_id))
            })
            .ok_or_else(|| {
                Diagnostic::new(
                    callee.span,
                    "baked procedure has no retained source declaration and environment",
                )
            })?;
        if target.signature.ty != target_ty {
            return Err(Diagnostic::new(
                callee.span,
                "baked target value differs from its original checked source signature",
            ));
        }
        target.metadata = metadata;
        checked_use_metadata(&target, &[], target_ty, self.types)?;
        let bound = self.bind_baked_procedure_arguments(&target, arguments, &[])?;
        self.publish_baked_procedure_wrapper(&target, &bound, span)
    }

    pub(crate) fn publish_baked_procedure_wrapper(
        &mut self,
        target: &BakedProcedureTarget,
        bound: &[BakedProcedureArgument],
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        assigned_formals(target, bound, self.types)?;
        let key = wrapper_key(target, bound, self.types)?;
        let existing = self.meta.baked_wrappers.procedures.get(&key).copied();
        let owner = match existing {
            Some(owner) => owner,
            None => self.reserve_generated_procedure(span)?,
        };
        let (signature, procedure) = checked_wrapper(self.types, owner, target, bound)?;
        let original_arguments = match &procedure.body.statements[0] {
            Statement::CallResults {
                call, ..
            }
            | Statement::CallVoid(call) => &call.arguments,
            _ => unreachable!("checked wrapper contains its original direct call"),
        };
        let returned =
            self.baked_wrapper_result_contracts(target, original_arguments, bound, span)?;
        let metadata = checked_use_metadata(target, bound, signature.ty, self.types)?;
        if existing.is_none() {
            self.remember_anonymous_procedure_source(owner, signature.ty, span)?;
            self.meta
                .local_declarations
                .publish_generated(signature.clone(), procedure)?;
            self.meta
                .remember_execution(owner, self.meta.procedure_phases.get(target.signature.id));
            self.meta.baked_wrappers.procedures.insert(key, owner);
        } else if self
            .meta
            .local_declarations
            .signature(owner)
            .map(|checked| checked.ty)
            != Some(signature.ty)
        {
            return Err(Diagnostic::new(
                span,
                "baked wrapper reservation has no matching checked signature",
            ));
        }
        let value = self.bind_baked_wrapper_contract(&signature, metadata, returned, span)?;
        self.typed_value(value, signature.ty, span)
    }

    pub(crate) fn bind_baked_procedure_arguments(
        &mut self,
        target: &BakedProcedureTarget,
        arguments: &[syntax::CallArgument],
        previous: &[BakedProcedureArgument],
    ) -> Result<Vec<BakedProcedureArgument>, Diagnostic> {
        let assigned = assigned_formals(target, previous, self.types)?;
        let mut occupied = assigned.iter().map(Option::is_some).collect::<Vec<_>>();
        let mut bound = previous.to_vec();
        checked_use_metadata(target, previous, target.signature.ty, self.types)?;
        if target.metadata.argument_policy != super::bindings::CallbackArgumentPolicy::Established {
            return Err(Diagnostic::new(
                target.source.span,
                "baked target's checked parameter names are ambiguous",
            ));
        }
        let mut selected = Vec::with_capacity(arguments.len());
        for argument in arguments {
            let span = argument.value.span;
            let name = argument.name.ok_or_else(|| {
                Diagnostic::new(span, "#bake_arguments requires named constant arguments")
            })?;
            if argument.spread {
                return Err(Diagnostic::new(
                    span,
                    "#bake_arguments cannot spread a runtime pack",
                ));
            }
            let formal = target
                .metadata
                .parameters
                .iter()
                .position(|parameter| parameter.name == Some(name))
                .ok_or_else(|| Diagnostic::new(span, "unknown baked parameter"))?;
            if std::mem::replace(&mut occupied[formal], true) {
                return Err(Diagnostic::new(span, "duplicate baked parameter"));
            }
            let parameter = &target.signature.parameters[formal];
            if parameter.evaluation == syntax::ParameterEvaluation::Discard {
                return Err(Diagnostic::new(
                    span,
                    "#bake_arguments cannot supply a discarded parameter",
                ));
            }
            selected.push((argument, formal));
        }
        for (argument, formal) in &selected {
            let info = self.describe_argument(&argument.value)?;
            if !info.is_compile_time_constant()
                && !self.optional_baking_needs_materialization(&argument.value)?
            {
                return Err(Diagnostic::new(
                    argument.value.span,
                    "baked argument must be constant",
                ));
            }
            self.preview_expected_callback_contract(
                &argument.value,
                target.signature.parameters[*formal].ty,
                argument.value.span,
            )?;
        }
        for (argument, formal) in selected {
            let span = argument.value.span;
            let parameter = &target.signature.parameters[formal];
            let value = self.expr_expected(&argument.value, parameter.ty)?;
            let value = self.coerce_value(value, parameter.ty, span)?;
            let contract = self.callback_expression_contract(&argument.value, &value, span)?;
            let policy =
                self.specialization_callable_policy(&argument.value, parameter.ty, span)?;
            let value = self
                .literal_constant(value.clone(), span)
                .or_else(|_| self.evaluate_pure_constant(value, span))
                .map_err(|error| {
                    Diagnostic::new(
                        span,
                        format!("baked argument must be constant: {}", error.message),
                    )
                })?;
            let source = self
                .debug
                .source()
                .or_else(|| self.graph_scope.map(|scope| scope.source()))
                .ok_or_else(|| {
                    Diagnostic::new(span, "baked argument requires its retained defining source")
                })?;
            bound.push(BakedProcedureArgument {
                origin: target.origin,
                formal,
                source_formal: target.source_formals[formal],
                value,
                contract,
                policy,
                source: SourceSpan {
                    source,
                    span,
                },
            });
        }
        Ok(bound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrapper_calls_the_original_target_and_keeps_runtime_parameter_indices() {
        let path = std::path::Path::new("/baked-wrapper/main.jai");
        let source = "target::(#discard ignored:int,a:int,b:int,c:int)->int {return a*100+b*10+c;} main::()->int{return target(0,1,2,3);}";
        let mut overlay = jai_modules::SourceOverlay::new();
        overlay.insert(path, source.as_bytes().to_vec()).unwrap();
        let graph = jai_modules::ModuleGraph::load_with_provider(
            path,
            jai_modules::GraphOptions::default(),
            &overlay,
        )
        .unwrap();
        let declaration = graph
            .declarations()
            .iter()
            .find(|declaration| graph.symbols().name(declaration.name()) == "target")
            .unwrap();
        let mut symbols = graph.symbols().clone();
        let mut types = TypeRegistry::new();
        let int = types.scalar(ScalarType::Int(IntegerType::S64));
        let target_id = ProcedureId::new(0);
        let target_ty = types
            .procedure(ProcedureType {
                parameters: vec![int, int, int].into(),
                results: vec![int].into(),
                return_abi: jai_types::ForeignReturnAbi::Natural,
                convention: CallingConvention::Jai,
                context: ContextMode::None,
                variadic: jai_types::Variadic::None,
            })
            .unwrap();
        let origin = BakedCallableOrigin::Module {
            declaration: declaration.id(),
            file: declaration.file(),
        };
        let signature = Signature {
            id: target_id,
            ty: target_ty,
            parameters: ["ignored", "a", "b", "c"]
                .into_iter()
                .enumerate()
                .map(|(index, name)| ParameterSignature {
                    name: symbols.intern(name),
                    ty: int,
                    default: None,
                    evaluation: if index == 0 {
                        syntax::ParameterEvaluation::Discard
                    } else {
                        syntax::ParameterEvaluation::Evaluate
                    },
                })
                .collect(),
            results: vec![ResultSignature {
                name: None,
                ty: int,
                default: None,
                usage: syntax::ResultUsage::Optional,
            }],
            source_variadic: crate::overloads::CandidateVariadic::None,
        };
        let target = BakedProcedureTarget {
            origin,
            source: declaration.location(),
            source_formals: (0..4).collect(),
            source_arguments: None,
            metadata: super::super::bindings::CallbackSignature::source(&signature),
            signature,
        };
        let argument = BakedProcedureArgument {
            origin,
            formal: 2,
            source_formal: 2,
            contract: None,
            policy: None,
            source: declaration.location(),
            value: ConstantValue {
                ty: int,
                kind: ConstantKind::Int(jai_types::Integer::wrapping(IntegerType::S64, 9)),
            },
        };
        let (signature, wrapper) =
            checked_wrapper(&mut types, ProcedureId::new(80), &target, &[argument]).unwrap();
        assert_eq!(signature.parameters.len(), 3);
        assert_eq!(
            signature.parameters[0].evaluation,
            syntax::ParameterEvaluation::Discard
        );
        assert_eq!(wrapper.parameters.len(), 2);
        let Statement::CallResults {
            call, ..
        } = &wrapper.body.statements[0]
        else {
            panic!("the wrapper must contain a real direct call")
        };
        assert_eq!(call.procedure, target_id);
        assert_eq!(
            call.arguments
                .iter()
                .map(|(parameter, _)| parameter.index())
                .collect::<Vec<_>>(),
            [0, 1, 2]
        );
        assert!(matches!(call.arguments[1].1, ValueExpr::Int(_)));
    }
}
