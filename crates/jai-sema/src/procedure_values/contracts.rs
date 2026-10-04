//! Source contracts follow typed values; equal ABI types never establish source obligations.
use super::bindings::{CallbackArgumentPolicy, CallbackSignature};
use super::*;
mod anonymous_headers;
mod baked_bindings;
mod baked_partial;
pub(crate) mod baked_partial_constants;
mod baked_partial_producers;
mod bound_result_use;
mod compiler_slots;
mod expression_bindings;
mod generic_bindings;
mod generics;
mod policies;
mod records;
pub(crate) use policies::CallablePolicyEncoder;
pub use policies::CallablePolicyKey;
#[path = "contracts/syntax.rs"]
mod source_syntax;
pub(crate) use generics::source_type as callback_source_type;
pub(crate) use source_syntax::{
    CallbackSyntaxOrigin, ContractOrigin, ContractOwner, ContractStep, ContractSyntax,
};

#[derive(Clone)]
pub(crate) struct ValueContract {
    pub(crate) ty: TypeId,
    kind: ContractKind,
}
#[derive(Clone)]
enum ContractKind {
    Callable {
        metadata: CallbackSignature,
        results: Vec<Option<ValueContract>>,
        source: Option<ProcedureId>,
    },
    Sequence(Box<ValueContract>),
    Pointer(Box<ValueContract>),
    Record(RecordContract),
}
#[derive(Clone)]
struct RecordContract {
    file: jai_modules::FileInstanceId,
    bindings: HashMap<Symbol, Option<ValueContract>>,
    fields: HashMap<jai_types::FieldId, ValueContract>,
}
impl ValueContract {
    pub(super) fn same_binding_contract(&self, other: &Self) -> bool {
        if self.ty != other.ty {
            return false;
        }
        match (&self.kind, &other.kind) {
            (
                ContractKind::Callable {
                    metadata: left,
                    results: left_results,
                    source: left_source,
                },
                ContractKind::Callable {
                    metadata: right,
                    results: right_results,
                    source: right_source,
                },
            ) => {
                left_source == right_source
                    && left.argument_policy == right.argument_policy
                    && left.source_variadic == right.source_variadic
                    && left.parameters.len() == right.parameters.len()
                    && left
                        .parameters
                        .iter()
                        .zip(&right.parameters)
                        .all(|(left, right)| {
                            left.name == right.name
                                && left.ty == right.ty
                                && left.evaluation == right.evaluation
                                && same_parameter_default(
                                    left.default.as_ref(),
                                    right.default.as_ref(),
                                )
                        })
                    && left.results.len() == right.results.len()
                    && left
                        .results
                        .iter()
                        .zip(&right.results)
                        .all(|(left, right)| left.ty == right.ty && left.usage == right.usage)
                    && left_results.len() == right_results.len()
                    && left_results.iter().zip(right_results).all(|(left, right)| {
                        match (left, right) {
                            (Some(left), Some(right)) => left.same_binding_contract(right),
                            (None, None) => true,
                            _ => false,
                        }
                    })
            }
            (ContractKind::Pointer(left), ContractKind::Pointer(right))
            | (ContractKind::Sequence(left), ContractKind::Sequence(right)) => {
                left.same_binding_contract(right)
            }
            (ContractKind::Record(left), ContractKind::Record(right)) => {
                left.file == right.file
                    && left.fields.len() == right.fields.len()
                    && left.fields.iter().all(|(field, left)| {
                        right
                            .fields
                            .get(field)
                            .is_some_and(|right| left.same_binding_contract(right))
                    })
                    && left.bindings.len() == right.bindings.len()
                    && left.bindings.iter().all(|(name, left)| {
                        match (left, right.bindings.get(name)) {
                            (Some(left), Some(Some(right))) => left.same_binding_contract(right),
                            (None, Some(None)) => true,
                            _ => false,
                        }
                    })
            }
            _ => false,
        }
    }
    pub(crate) fn callback(&self) -> Option<&CallbackSignature> {
        match &self.kind {
            ContractKind::Callable {
                metadata, ..
            } => Some(metadata),
            _ => None,
        }
    }
    pub(crate) fn returned(&self) -> Vec<Option<Self>> {
        match &self.kind {
            ContractKind::Callable {
                results, ..
            } => results.clone(),
            _ => vec![],
        }
    }
    fn result(&self, index: usize) -> Option<Self> {
        match &self.kind {
            ContractKind::Callable {
                results, ..
            } => results.get(index).cloned().flatten(),
            _ => None,
        }
    }
    fn element(&self) -> Option<Self> {
        match &self.kind {
            ContractKind::Sequence(inner) | ContractKind::Pointer(inner) => Some((**inner).clone()),
            _ => None,
        }
    }
    fn merge(mut self, other: Self) -> Self {
        if self.ty != other.ty {
            return self;
        }
        if self.same_binding_contract(&other) {
            return self;
        }
        match (&mut self.kind, other.kind) {
            (
                ContractKind::Callable {
                    metadata,
                    results,
                    source,
                },
                ContractKind::Callable {
                    metadata: other,
                    results: other_results,
                    source: other_source,
                },
            ) => {
                if other.argument_policy == CallbackArgumentPolicy::Ambiguous
                    || metadata.source_variadic != other.source_variadic
                    || metadata.parameters.len() != other.parameters.len()
                    || metadata
                        .parameters
                        .iter()
                        .zip(&other.parameters)
                        .any(|(left, right)| {
                            left.ty != right.ty || left.evaluation != right.evaluation
                        })
                {
                    metadata.argument_policy = CallbackArgumentPolicy::Ambiguous;
                }
                let same_parameter_policy =
                    metadata.argument_policy == other.argument_policy
                        && metadata.parameters.len() == other.parameters.len()
                        && metadata.source_variadic == other.source_variadic
                        && metadata.parameters.iter().zip(&other.parameters).all(
                            |(left, right)| {
                                left.name == right.name
                                    && left.ty == right.ty
                                    && left.evaluation == right.evaluation
                                    && same_parameter_default(
                                        left.default.as_ref(),
                                        right.default.as_ref(),
                                    )
                            },
                        );
                if *source != other_source {
                    *source = None;
                }
                if !same_parameter_policy {
                    for parameter in &mut metadata.parameters {
                        parameter.name = None;
                        parameter.default = None;
                    }
                }
                for (result, other) in metadata.results.iter_mut().zip(other.results) {
                    if other.usage == syntax::ResultUsage::Required {
                        result.usage = syntax::ResultUsage::Required;
                    }
                }
                for (result, other) in results.iter_mut().zip(other_results) {
                    if let Some(other) = other {
                        *result = Some(match result.take() {
                            Some(value) => value.merge(other),
                            None => other,
                        });
                    }
                }
            }
            (ContractKind::Sequence(inner), ContractKind::Sequence(other))
            | (ContractKind::Pointer(inner), ContractKind::Pointer(other)) => {
                **inner = (**inner).clone().merge(*other)
            }
            (ContractKind::Record(record), ContractKind::Record(other)) => {
                for (field, other) in other.fields {
                    record
                        .fields
                        .entry(field)
                        .and_modify(|value| {
                            *value = value.clone().merge(other.clone());
                        })
                        .or_insert(other);
                }
                for (name, other) in other.bindings {
                    let value = record.bindings.entry(name).or_insert(None);
                    if let Some(other) = other {
                        *value = Some(match value.take() {
                            Some(old) => old.merge(other),
                            None => other,
                        });
                    }
                }
            }
            _ => {}
        }
        self
    }
}

pub(crate) fn same_parameter_default(
    left: Option<&ParameterDefault>,
    right: Option<&ParameterDefault>,
) -> bool {
    if let Some(ParameterDefault::Source(source)) = left {
        if let Some(ready) = source.ready() {
            return same_parameter_default(Some(ready), right);
        }
    }
    if let Some(ParameterDefault::Source(source)) = right {
        if let Some(ready) = source.ready() {
            return same_parameter_default(left, Some(ready));
        }
    }
    match (left, right) {
        (Some(ParameterDefault::Source(left)), Some(ParameterDefault::Source(right))) => {
            left.key == right.key
        }
        (None, None)
        | (Some(ParameterDefault::CallerLocation), Some(ParameterDefault::CallerLocation))
        | (Some(ParameterDefault::Discarded), Some(ParameterDefault::Discarded)) => true,
        (Some(ParameterDefault::Constant(left)), Some(ParameterDefault::Constant(right))) => {
            left == right
        }
        (Some(ParameterDefault::RuntimeRead(left)), Some(ParameterDefault::RuntimeRead(right))) => {
            left.root() == right.root() && left.steps() == right.steps() && left.ty() == right.ty()
        }
        (
            Some(ParameterDefault::CodeNull {
                ty: left,
            }),
            Some(ParameterDefault::CodeNull {
                ty: right,
            }),
        ) => left == right,
        _ => false,
    }
}

impl Resolver<'_> {
    pub(crate) fn retained_callback_syntax(
        &self,
        syntax: &syntax::TypeSyntax,
        span: Span,
    ) -> Result<ContractSyntax, Diagnostic> {
        self.contract_syntax(syntax, span, 0)
    }
    fn contract_syntax(
        &self,
        syntax: &syntax::TypeSyntax,
        span: Span,
        depth: usize,
    ) -> Result<ContractSyntax, Diagnostic> {
        if depth >= crate::constant_limits::MAX_CONSTANT_DEPTH {
            return Err(Diagnostic::new(
                span,
                "callback contract exceeds source type depth",
            ));
        }
        if let syntax::TypeSyntax::Named(path) = syntax {
            if let Some(alias) = self.local_callback_alias(path) {
                return Ok(alias);
            }
            let lexical = path.members.is_empty()
                && (matches!(
                    self.scopes
                        .iter()
                        .rev()
                        .find_map(|scope| scope.get(&path.root)),
                    Some(Binding::Type(_))
                ) || self
                    .graph_scope
                    .and_then(|scope| scope.substitution)
                    .is_some_and(|substitution| substitution.ty(path.root).is_some()));
            let local_nominal = self.local_name_present(path.root)
                && self.local_ready_type_path(path).is_some_and(|ty| {
                    matches!(
                        self.types.kind(ty),
                        Ok(jai_types::TypeKind::Record(_) | jai_types::TypeKind::Enum(_))
                    )
                });
            if !lexical
                && !local_nominal
                && let Some(scope) = self.graph_scope
            {
                return scope.callback_contract_syntax(syntax, span);
            }
        }
        let origin = ContractOrigin {
            file: self.graph_scope.map(|scope| scope.contract_file()),
            source: self
                .debug
                .source()
                .or_else(|| self.graph_scope.map(|scope| scope.source())),
            owner: ContractOwner::Procedure(
                self.local_scopes.body_owner().unwrap_or(self.procedure),
            ),
            lexical_scopes: self.local_scopes.capture_identity(),
            target: self.target_layout,
            substitution: self
                .graph_scope
                .and_then(|scope| scope.substitution.cloned()),
        };
        let mut retained = ContractSyntax::resolve(syntax, origin, |child| {
            self.contract_syntax(child, span, depth + 1)
        })?;
        if let Some(callback) = &mut retained.callback {
            callback.proof = self.original_annotation_proof(callback);
        }
        Ok(retained)
    }
    pub(crate) fn annotation_value_contract(
        &self,
        ty: TypeId,
        syntax: &syntax::TypeSyntax,
        span: Span,
    ) -> Result<Option<ValueContract>, Diagnostic> {
        let syntax = self.contract_syntax(syntax, span, 0)?;
        let variable = match &syntax.syntax {
            syntax::TypeSyntax::Variable(name)
            | syntax::TypeSyntax::Restricted {
                variable: name, ..
            } => Some(*name),
            syntax::TypeSyntax::Named(path) if path.members.is_empty() => Some(path.root),
            _ => None,
        };
        if let Some(contract) = variable
            .and_then(|name| {
                self.meta
                    .callbacks
                    .generic_type_arguments
                    .get(&(self.procedure, name))
            })
            .filter(|contract| contract.ty == ty)
        {
            return Ok(Some(contract.clone()));
        }
        self.normalized_value_contract(ty, &syntax, span, 0)
    }
    fn normalized_value_contract(
        &self,
        ty: TypeId,
        source: &ContractSyntax,
        span: Span,
        depth: usize,
    ) -> Result<Option<ValueContract>, Diagnostic> {
        self.bound_syntax_contract(ty, source, &HashMap::new(), span, depth)
    }
    fn bound_syntax_contract(
        &self,
        ty: TypeId,
        source: &ContractSyntax,
        bindings: &HashMap<Symbol, Option<ValueContract>>,
        span: Span,
        depth: usize,
    ) -> Result<Option<ValueContract>, Diagnostic> {
        if depth >= crate::constant_limits::MAX_CONSTANT_DEPTH {
            return Err(Diagnostic::new(
                span,
                "callback contract exceeds source type depth",
            ));
        }
        let variable = match &source.syntax {
            syntax::TypeSyntax::Variable(name)
            | syntax::TypeSyntax::Restricted {
                variable: name, ..
            } => Some(*name),
            syntax::TypeSyntax::Named(path) if path.members.is_empty() => Some(path.root),
            _ => None,
        };
        if let Some(contract) = variable.and_then(|name| bindings.get(&name)) {
            return Ok(contract
                .as_ref()
                .filter(|contract| contract.ty == ty)
                .cloned());
        }
        if source.origin.owner == ContractOwner::Procedure(self.procedure)
            && let Some(contract) = variable.and_then(|name| {
                self.meta
                    .callbacks
                    .generic_type_arguments
                    .get(&(self.procedure, name))
            })
            && contract.ty == ty
        {
            return Ok(Some(contract.clone()));
        }
        let kind = match (self.types.kind(ty), &source.syntax) {
            (Ok(jai_types::TypeKind::Record(_)), syntax::TypeSyntax::Application(application)) => {
                return self.record_application_contract(
                    ty,
                    application,
                    source,
                    bindings,
                    span,
                    depth + 1,
                );
            }
            (Ok(jai_types::TypeKind::Record(_)), syntax::TypeSyntax::InlineRecord(_)) => {
                let file = source.origin.file.ok_or_else(|| {
                    Diagnostic::new(span, "record callback contract has no defining file")
                })?;
                return self.record_shape_contract(ty, file, bindings, span, depth + 1);
            }
            (
                Ok(jai_types::TypeKind::Procedure(_)),
                syntax::TypeSyntax::Procedure(signature_source),
            ) => {
                let signature = self
                    .types
                    .procedure_definition(ty)
                    .map_err(|error| Diagnostic::new(span, error.to_string()))?;
                let results = signature
                    .results
                    .iter()
                    .enumerate()
                    .map(|(index, ty)| {
                        self.bound_syntax_contract(
                            *ty,
                            source.child(ContractStep::Result(index)),
                            bindings,
                            span,
                            depth + 1,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let source_parameters =
                    if signature_source.parameters.iter().any(|parameter| {
                        parameter.evaluation == syntax::ParameterEvaluation::Discard
                    }) {
                        self.checked_callback_origin_parameter_types(
                            ty,
                            source.callback.as_ref().expect("callback producer origin"),
                            span,
                        )?
                    } else {
                        signature.parameters.to_vec()
                    };
                let defaults = if signature_source
                    .parameters
                    .iter()
                    .any(|parameter| parameter.default.is_some())
                {
                    let origin = source.callback.as_ref().expect("callback producer origin");
                    let proof = origin
                        .proof
                        .clone()
                        .or_else(|| self.original_annotation_proof(origin))
                        .ok_or_else(|| {
                            Diagnostic::new(
                                span,
                                "callback defaults are pending their original checked annotation",
                            )
                        })?;
                    if proof.ty() != ty || proof.parameters() != source_parameters.as_slice() {
                        return Err(Diagnostic::new(
                            span,
                            "callback default proof has a different canonical signature",
                        ));
                    }
                    proof.defaults().to_vec()
                } else {
                    Vec::new()
                };
                ContractKind::Callable {
                    metadata: CallbackSignature::annotation(
                        ty,
                        signature_source,
                        self.types,
                        &source_parameters,
                        &defaults,
                    ),
                    results,
                    source: None,
                }
            }
            (Ok(jai_types::TypeKind::Pointer(element)), syntax::TypeSyntax::Pointer(_)) => {
                let Some(inner) = self.bound_syntax_contract(
                    *element,
                    source.child(ContractStep::Element),
                    bindings,
                    span,
                    depth + 1,
                )?
                else {
                    return Ok(None);
                };
                ContractKind::Pointer(Box::new(inner))
            }
            (
                Ok(
                    jai_types::TypeKind::FixedArray {
                        element, ..
                    }
                    | jai_types::TypeKind::Slice(element)
                    | jai_types::TypeKind::DynamicArray(element),
                ),
                syntax::TypeSyntax::FixedArray {
                    ..
                }
                | syntax::TypeSyntax::Slice(_)
                | syntax::TypeSyntax::DynamicArray(_),
            ) => {
                let Some(inner) = self.bound_syntax_contract(
                    *element,
                    source.child(ContractStep::Element),
                    bindings,
                    span,
                    depth + 1,
                )?
                else {
                    return Ok(None);
                };
                ContractKind::Sequence(Box::new(inner))
            }
            _ => return Ok(None),
        };
        Ok(Some(ValueContract {
            ty,
            kind,
        }))
    }
    pub(crate) fn register_result_contracts(
        &mut self,
        procedure: ProcedureId,
        source: &[syntax::ProcedureResult],
        results: &[ResultSignature],
    ) -> Result<(), Diagnostic> {
        let mut contracts = Vec::with_capacity(results.len());
        for (source, result) in source.iter().zip(results) {
            contracts.push(match &source.binding {
                syntax::ResultBinding::Typed {
                    ty, ..
                } => self.annotation_value_contract(result.ty, ty, source.span)?,
                syntax::ResultBinding::InferredDefault(expression) => match &result.default {
                    Some(value) => self.callback_expression_contract(
                        expression,
                        &value.clone().into_expression(),
                        expression.span,
                    )?,
                    None => None,
                },
            });
        }
        self.meta
            .callbacks
            .returned_contracts
            .insert(procedure, contracts);
        Ok(())
    }
    pub(crate) fn procedure_result_contracts(
        &mut self,
        procedure: ProcedureId,
        span: Span,
    ) -> Result<Vec<Option<ValueContract>>, Diagnostic> {
        if let Some(contracts) = self.meta.callbacks.returned_contracts.get(&procedure) {
            return Ok(contracts.clone());
        }
        let Some(scope) = self.graph_scope else {
            return Ok(vec![]);
        };
        let Some(source) = scope.callback_result_contract_sources(procedure, span)? else {
            return Ok(vec![]);
        };
        let Some(signature) = self.contract_procedure_signature(procedure) else {
            return Ok(vec![]);
        };
        let mut contracts = Vec::with_capacity(signature.results.len());
        for (result, source) in signature.results.iter().zip(source) {
            contracts.push(match source {
                Some(source) => self.normalized_value_contract(result.ty, &source, span, 0)?,
                None => match &result.default {
                    Some(value) => {
                        self.callback_value_contract(&value.clone().into_expression(), span)?
                    }
                    None => None,
                },
            });
        }
        self.meta
            .callbacks
            .returned_contracts
            .insert(procedure, contracts.clone());
        Ok(contracts)
    }
    fn contract_procedure_signature(&self, procedure: ProcedureId) -> Option<Signature> {
        self.meta
            .local_declarations
            .signature(procedure)
            .cloned()
            .or_else(|| {
                self.signatures
                    .values()
                    .find(|signature| signature.id == procedure)
                    .cloned()
            })
            .or_else(|| {
                self.graph_scope
                    .and_then(|scope| scope.callback_procedure_signature(procedure))
            })
    }
    pub(crate) fn callback_expression_contract(
        &mut self,
        source: &syntax::Expression,
        value: &ValueExpr,
        span: Span,
    ) -> Result<Option<ValueContract>, Diagnostic> {
        self.callback_expression_contract_inner(source, value, span, 0)
    }
    fn callback_expression_contract_inner(
        &mut self,
        source: &syntax::Expression,
        value: &ValueExpr,
        span: Span,
        depth: usize,
    ) -> Result<Option<ValueContract>, Diagnostic> {
        if depth >= crate::constant_limits::MAX_CONSTANT_DEPTH {
            return Err(Diagnostic::new(
                span,
                "callback source contract exceeds expression depth",
            ));
        }
        match &source.kind {
            syntax::ExpressionKind::Name(name) => {
                let path = syntax::NamePath {
                    root: *name,
                    members: vec![],
                };
                if let Some(contract) = self.checked_baked_constant_binding_contract(&path, span)? {
                    return Ok(contract);
                }
                if let Some(contract) = self.checked_baked_callback_binding_contract(&path, span)? {
                    return Ok(Some(contract));
                }
                self.callback_value_contract(value, span)
            }
            syntax::ExpressionKind::QualifiedName(path) => {
                if let Some(contract) = self.checked_baked_constant_binding_contract(path, span)? {
                    return Ok(contract);
                }
                if let Some(contract) = self.checked_baked_callback_binding_contract(path, span)? {
                    return Ok(Some(contract));
                }
                self.callback_value_contract(value, span)
            }
            syntax::ExpressionKind::StructLiteral(literal) => {
                if let Some(path) = &literal.ty {
                    return self.annotation_value_contract(value.type_id(self.types), path, span);
                }
                self.callback_value_contract(value, span)
            }
            syntax::ExpressionKind::PositionalStructLiteral(literal) => {
                if let Some(path) = &literal.ty {
                    return self.annotation_value_contract(value.type_id(self.types), path, span);
                }
                self.callback_value_contract(value, span)
            }
            syntax::ExpressionKind::Call(_, arguments)
            | syntax::ExpressionKind::QualifiedCall(_, arguments) => {
                if let ValueExpr::IndirectCall {
                    callee, ..
                } = value
                {
                    let callee_source = syntax::Expression {
                        kind: match &source.kind {
                            syntax::ExpressionKind::Call(name, _) => {
                                syntax::ExpressionKind::Name(*name)
                            }
                            syntax::ExpressionKind::QualifiedCall(path, _) => {
                                syntax::ExpressionKind::QualifiedName(path.clone())
                            }
                            _ => unreachable!(),
                        },
                        span: source.span,
                    };
                    return Ok(self
                        .callback_expression_contract_inner(
                            &callee_source,
                            callee,
                            span,
                            depth + 1,
                        )?
                        .and_then(|contract| contract.result(0)));
                }
                if let ValueExpr::Call {
                    call, ..
                } = value
                {
                    return Ok(self
                        .call_result_contracts_for_source(
                            call.procedure,
                            &call.arguments,
                            Some(arguments),
                            span,
                            depth + 1,
                        )?
                        .into_iter()
                        .next()
                        .flatten());
                }
                self.callback_value_contract(value, span)
            }
            syntax::ExpressionKind::TypeCast {
                ty, ..
            } => self.annotation_value_contract(value.type_id(self.types), ty, span),
            syntax::ExpressionKind::CallHint {
                call, ..
            } => self.callback_expression_contract_inner(call, value, span, depth + 1),
            syntax::ExpressionKind::Unary(_, _)
            | syntax::ExpressionKind::Binary(_, _, _)
            | syntax::ExpressionKind::Index {
                ..
            } => {
                if let ValueExpr::Call {
                    call, ..
                } = value
                    && let Some(contracts) = self.local_operator_expression_result_contracts(
                        source,
                        call,
                        span,
                        depth + 1,
                    )?
                {
                    return Ok(contracts.into_iter().next().flatten());
                }
                self.callback_value_contract(value, span)
            }
            syntax::ExpressionKind::IndirectCall {
                callee, ..
            } => {
                if let ValueExpr::IndirectCall {
                    callee: value, ..
                } = value
                {
                    return Ok(self
                        .callback_expression_contract_inner(callee, value, span, depth + 1)?
                        .and_then(|contract| contract.result(0)));
                }
                self.callback_value_contract(value, span)
            }
            syntax::ExpressionKind::Conditional(source) => {
                if let ValueExpr::Conditional {
                    expression, ..
                } = value
                {
                    let left = self.callback_expression_contract_inner(
                        source.then_source(),
                        &expression.then_value,
                        span,
                        depth + 1,
                    )?;
                    let right = match &source.else_value {
                        Some(source) => self.callback_expression_contract_inner(
                            source,
                            &expression.else_value,
                            span,
                            depth + 1,
                        )?,
                        None => None,
                    };
                    return Ok(match (left, right) {
                        (Some(left), Some(right)) => Some(left.merge(right)),
                        (left, right) => left.or(right),
                    });
                }
                self.callback_value_contract(value, span)
            }
            syntax::ExpressionKind::ArrayLiteral(source) => {
                if let ValueExpr::Array {
                    ty,
                    elements,
                } = value
                {
                    if let Some(source) = &source.element_type
                        && let Ok(jai_types::TypeKind::FixedArray {
                            element, ..
                        }) = self.types.kind(*ty)
                    {
                        return Ok(self.annotation_value_contract(*element, source, span)?.map(
                            |inner| ValueContract {
                                ty: *ty,
                                kind: ContractKind::Sequence(Box::new(inner)),
                            },
                        ));
                    }
                    let mut merged = None;
                    for (source, value) in source.elements.iter().zip(elements) {
                        if let Some(contract) =
                            self.callback_expression_contract_inner(source, value, span, depth + 1)?
                        {
                            merged = Some(match merged {
                                Some(old) => ValueContract::merge(old, contract),
                                None => contract,
                            });
                        }
                    }
                    return Ok(merged.map(|inner| ValueContract {
                        ty: *ty,
                        kind: ContractKind::Sequence(Box::new(inner)),
                    }));
                }
                self.callback_value_contract(value, span)
            }
            _ => self.callback_value_contract(value, span),
        }
    }
    pub(crate) fn callback_value_contract(
        &mut self,
        value: &ValueExpr,
        span: Span,
    ) -> Result<Option<ValueContract>, Diagnostic> {
        self.callback_value_contract_inner(value, span, 0)
    }
    fn callback_value_contract_inner(
        &mut self,
        value: &ValueExpr,
        span: Span,
        depth: usize,
    ) -> Result<Option<ValueContract>, Diagnostic> {
        if depth >= crate::constant_limits::MAX_CONSTANT_DEPTH {
            return Err(Diagnostic::new(
                span,
                "callback value contract exceeds expression depth",
            ));
        }
        if !self.callback_contract_type(value.type_id(self.types), depth) {
            return Ok(None);
        }
        let result = match value {
            ValueExpr::Bind {
                bindings,
                body,
                ..
            } => self.bound_expression_contract(bindings, body, span, depth + 1)?,
            ValueExpr::Bound {
                binding,
                ty,
            } => self.expression_binding_contract(*binding, *ty, span)?,
            // Physical aliases and partial paths cannot be represented by the
            // independent semantic-field callback contract.
            ValueExpr::OrderedRecord {
                ..
            } => None,
            ValueExpr::RecordBuild {
                ty,
                initializers,
            } => {
                let initializers = initializers
                    .iter()
                    .map(|(field, value)| (*field, value))
                    .collect::<Vec<_>>();
                self.captured_record_contract(*ty, &initializers, span, depth + 1)?
            }
            ValueExpr::Record {
                ty,
                fields,
            } => {
                let initializers = fields
                    .iter()
                    .enumerate()
                    .map(|(index, value)| {
                        self.types
                            .field(*ty, index)
                            .map(|field| (field.id, value))
                            .map_err(|error| Diagnostic::new(span, error.to_string()))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                self.captured_record_contract(*ty, &initializers, span, depth + 1)?
            }
            ValueExpr::Union {
                ty,
                field,
                value,
            } => {
                self.captured_record_contract(*ty, &[(*field, value.as_ref())], span, depth + 1)?
            }
            ValueExpr::ProcedureValue {
                procedure,
                ty,
            } => {
                let Some(signature) = self.contract_procedure_signature(*procedure) else {
                    return Ok(None);
                };
                Some(ValueContract {
                    ty: *ty,
                    kind: ContractKind::Callable {
                        metadata: CallbackSignature::source(&signature),
                        results: self.procedure_result_contracts(*procedure, span)?,
                        source: Some(*procedure),
                    },
                })
            }
            ValueExpr::Call {
                call, ..
            } => self
                .call_result_contracts_for_depth(call.procedure, &call.arguments, span, depth + 1)?
                .into_iter()
                .next()
                .flatten(),
            ValueExpr::IndirectCall {
                callee, ..
            } => self
                .callback_value_contract_inner(callee, span, depth + 1)?
                .and_then(|contract| contract.result(0)),
            ValueExpr::Load(place) => self.callback_place_contract(*place, span, depth + 1)?,
            ValueExpr::Field {
                base,
                field,
                ..
            } => {
                let contract = self.callback_value_contract_inner(base, span, depth + 1)?;
                match contract
                    .as_ref()
                    .map(|contract| {
                        self.record_field_value_contract(contract, *field, span, depth + 1)
                    })
                    .transpose()?
                    .flatten()
                {
                    Some(contract) => Some(contract),
                    None => self.callback_field_contract(base.type_id(self.types), *field, span)?,
                }
            }
            ValueExpr::Conditional {
                expression, ..
            } => {
                let mut merged = None;
                let mut pending = vec![&expression.then_value, &expression.else_value];
                while let Some(branch) = pending.pop() {
                    if let ValueExpr::Conditional {
                        expression, ..
                    } = branch
                    {
                        pending.push(&expression.then_value);
                        pending.push(&expression.else_value);
                    } else if let Some(contract) =
                        self.callback_value_contract_inner(branch, span, depth + 1)?
                    {
                        merged = Some(match merged {
                            Some(old) => ValueContract::merge(old, contract),
                            None => contract,
                        });
                    }
                }
                merged
            }
            ValueExpr::Array {
                ty,
                elements,
            } => {
                let mut merged = None;
                for element in elements {
                    if let Some(contract) =
                        self.callback_value_contract_inner(element, span, depth + 1)?
                    {
                        merged = Some(match merged {
                            Some(old) => ValueContract::merge(old, contract),
                            None => contract,
                        });
                    }
                }
                merged.map(|inner| ValueContract {
                    ty: *ty,
                    kind: ContractKind::Sequence(Box::new(inner)),
                })
            }
            ValueExpr::SequenceBuild {
                ty,
                initializers,
            } => {
                match initializers
                    .iter()
                    .find(|(field, _)| *field == jai_ir::SequenceField::Data)
                {
                    Some((_, pointer)) => self
                        .callback_value_contract_inner(pointer, span, depth + 1)?
                        .and_then(|contract| contract.element())
                        .map(|element| ValueContract {
                            ty: *ty,
                            kind: ContractKind::Sequence(Box::new(element)),
                        }),
                    None => None,
                }
            }
            ValueExpr::SequenceField {
                base,
                field: jai_ir::SequenceField::Data,
                ty,
            } => self
                .callback_value_contract_inner(base, span, depth + 1)?
                .and_then(|contract| contract.element())
                .map(|inner| ValueContract {
                    ty: *ty,
                    kind: ContractKind::Pointer(Box::new(inner)),
                }),
            ValueExpr::Index {
                base, ..
            } => self
                .callback_value_contract_inner(base, span, depth + 1)?
                .and_then(|contract| contract.element()),
            ValueExpr::ArrayToSlice {
                array,
                ty,
            } => self
                .callback_place_contract(*array, span, depth + 1)?
                .map(|mut contract| {
                    contract.ty = *ty;
                    contract
                }),
            ValueExpr::ArrayView {
                array,
                ty,
            }
            | ValueExpr::SequenceView {
                sequence: array,
                ty,
            } => self
                .callback_value_contract_inner(array, span, depth + 1)?
                .map(|mut contract| {
                    contract.ty = *ty;
                    contract
                }),
            ValueExpr::AddressOf {
                place,
                ty,
            } => self
                .callback_place_contract(*place, span, depth + 1)?
                .map(|inner| ValueContract {
                    ty: *ty,
                    kind: ContractKind::Pointer(Box::new(inner)),
                }),
            ValueExpr::AddressOfValue {
                value,
                ty,
            } => self
                .callback_value_contract_inner(value, span, depth + 1)?
                .map(|inner| ValueContract {
                    ty: *ty,
                    kind: ContractKind::Pointer(Box::new(inner)),
                }),
            ValueExpr::PointerCast {
                value,
                ty,
                ..
            }
            | ValueExpr::PointerOffset {
                pointer: value,
                ty,
                ..
            }
            | ValueExpr::PointerOffsetLeft {
                pointer: value,
                ty,
                ..
            } => {
                let contract = self.callback_value_contract_inner(value, span, depth + 1)?;
                contract.and_then(|mut contract| {
                    let element = contract.element()?;
                    if matches!(self.types.kind(*ty), Ok(jai_types::TypeKind::Pointer(target)) if *target == element.ty)
                    {
                        contract.ty = *ty;
                        Some(contract)
                    } else {
                        None
                    }
                })
            }
            _ => None,
        };
        if let Some(contract) = &result
            && contract.ty != value.type_id(self.types)
        {
            return Err(Diagnostic::new(
                span,
                "callback value contract has a different canonical type",
            ));
        }
        Ok(result)
    }
    pub(crate) fn callback_place_contract(
        &mut self,
        place: Place,
        span: Span,
        depth: usize,
    ) -> Result<Option<ValueContract>, Diagnostic> {
        if depth >= crate::constant_limits::MAX_CONSTANT_DEPTH {
            return Err(Diagnostic::new(
                span,
                "callback place contract exceeds projection depth",
            ));
        }
        if !self.callback_contract_type(place.ty(), depth) {
            return Ok(None);
        }
        if let Some(contract) = self.meta.callbacks.value_contracts.get(&place) {
            return Ok(Some(contract.clone()));
        }
        match place.kind() {
            PlaceKind::Field(id) => {
                let projection = *self
                    .places
                    .projection(id)
                    .map_err(|error| Diagnostic::new(span, error.to_string()))?;
                let contract = self.callback_place_contract(projection.base, span, depth + 1)?;
                if let Some(contract) = contract
                    .as_ref()
                    .map(|contract| {
                        self.record_field_value_contract(
                            contract,
                            projection.field,
                            span,
                            depth + 1,
                        )
                    })
                    .transpose()?
                    .flatten()
                {
                    return Ok(Some(contract));
                }
                self.callback_field_contract(projection.base.ty(), projection.field, span)
            }
            PlaceKind::Index(id) => {
                let base = self
                    .places
                    .index_projection(id)
                    .map_err(|error| Diagnostic::new(span, error.to_string()))?
                    .base;
                Ok(self
                    .callback_place_contract(base, span, depth + 1)?
                    .and_then(|contract| contract.element()))
            }
            PlaceKind::SequenceField(id) => {
                let projection = *self
                    .places
                    .sequence_projection(id)
                    .map_err(|error| Diagnostic::new(span, error.to_string()))?;
                if projection.field != jai_ir::SequenceField::Data {
                    return Ok(None);
                }
                Ok(self
                    .callback_place_contract(projection.base, span, depth + 1)?
                    .and_then(|contract| contract.element())
                    .map(|inner| ValueContract {
                        ty: place.ty(),
                        kind: ContractKind::Pointer(Box::new(inner)),
                    }))
            }
            PlaceKind::Dereference(id) => {
                let pointer = self
                    .places
                    .dereference_projection(id)
                    .map_err(|error| Diagnostic::new(span, error.to_string()))?
                    .pointer
                    .clone();
                Ok(self
                    .callback_value_contract_inner(&pointer, span, depth + 1)?
                    .and_then(|contract| contract.element()))
            }
            _ => {
                if let Some(scope) = self.graph_scope {
                    if let Some(source) = scope.callback_global_contract_syntax(place, span)? {
                        return self.normalized_value_contract(place.ty(), &source, span, 0);
                    }
                    if matches!(
                        self.types.kind(place.ty()),
                        Ok(jai_types::TypeKind::Procedure(_))
                    ) && let Some(signature) =
                        scope.callback_inferred_global_signature(place, span)?
                    {
                        return self.callback_value_contract_inner(
                            &ValueExpr::ProcedureValue {
                                procedure: signature.id,
                                ty: signature.ty,
                            },
                            span,
                            depth + 1,
                        );
                    }
                }
                Ok(self
                    .meta
                    .callbacks
                    .places
                    .get(&place)
                    .cloned()
                    .map(|metadata| ValueContract {
                        ty: place.ty(),
                        kind: ContractKind::Callable {
                            metadata,
                            results: vec![],
                            source: None,
                        },
                    }))
            }
        }
    }
    fn callback_contract_type(&self, ty: TypeId, depth: usize) -> bool {
        if depth >= crate::constant_limits::MAX_CONSTANT_DEPTH {
            return false;
        }
        let mut pending = vec![ty];
        let mut visited = std::collections::HashSet::new();
        while let Some(ty) = pending.pop() {
            if !visited.insert(ty) {
                continue;
            }
            match self.types.kind(ty) {
                Ok(jai_types::TypeKind::Procedure(_)) => return true,
                Ok(jai_types::TypeKind::Record(_)) => {
                    if let Ok(record) = self.types.record_definition(ty) {
                        pending.extend(record.fields.iter().copied());
                    }
                }
                Ok(
                    jai_types::TypeKind::Pointer(element)
                    | jai_types::TypeKind::Slice(element)
                    | jai_types::TypeKind::DynamicArray(element)
                    | jai_types::TypeKind::FixedArray {
                        element, ..
                    },
                ) => pending.push(*element),
                _ => {}
            }
        }
        false
    }
    fn callback_field_contract(
        &mut self,
        record: TypeId,
        field: jai_types::FieldId,
        span: Span,
    ) -> Result<Option<ValueContract>, Diagnostic> {
        if let Some(contract) = self.meta.callbacks.field_contracts.get(&field) {
            return Ok(Some(contract.clone()));
        }
        let ty = self
            .types
            .field_type(field)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?;
        let contract = if let Some(source) = self
            .graph_scope
            .map(|scope| scope.callback_field_contract_syntax(record, field, span))
            .transpose()?
            .flatten()
        {
            self.normalized_value_contract(ty, &source, span, 0)?
        } else {
            let metadata = self.record_metadata(record, span)?;
            let Some(metadata) = metadata.fields.iter().find(|metadata| metadata.id == field)
            else {
                return Ok(None);
            };
            match metadata.syntax.named_binding() {
                Some(syntax::FieldBinding::Explicit {
                    ty: source, ..
                }) => {
                    if let Some(origin) = self.meta.record_specializations.record(record)
                        && let Some(scope) = self.graph_scope
                    {
                        let source =
                            scope.callback_contract_syntax_in_file(origin.file, source, span)?;
                        self.normalized_value_contract(ty, &source, span, 0)?
                    } else {
                        self.annotation_value_contract(ty, source, span)?
                    }
                }
                Some(syntax::FieldBinding::Inferred(_)) => {
                    let context_default = self
                        .context
                        .filter(|schema| schema.definition.record_type == record)
                        .and_then(|schema| match &schema.definition.default.kind {
                            ConstantKind::Record(fields) => fields.get(field.index()).cloned(),
                            _ => None,
                        });
                    let value = match context_default {
                        Some(value) => value,
                        None => self.field_default_value(field, span)?,
                    };
                    self.callback_value_contract(&value.into_expression(), span)?
                }
                None => None,
            }
        };
        if let Some(contract) = &contract {
            self.meta
                .callbacks
                .field_contracts
                .insert(field, contract.clone());
        }
        Ok(contract)
    }
    pub(crate) fn bind_value_contract(
        &mut self,
        place: Place,
        contract: Option<ValueContract>,
        span: Span,
    ) -> Result<(), Diagnostic> {
        if let Some(contract) = contract {
            if contract.ty != place.ty() {
                return Err(Diagnostic::new(
                    span,
                    "callback binding contract has a different canonical type",
                ));
            }
            self.meta.callbacks.value_contracts.insert(place, contract);
        } else {
            self.meta.callbacks.value_contracts.remove(&place);
            self.meta.callbacks.places.remove(&place);
        }
        Ok(())
    }
}
