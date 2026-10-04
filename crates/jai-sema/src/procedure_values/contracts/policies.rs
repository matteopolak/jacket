//! Executable callback policies are checked source facts, independent of result usage.
use super::*;
use jai_types::FieldId;
use std::hash::{Hash, Hasher};
mod encoding;
pub(crate) use encoding::CallablePolicyEncoder;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CallablePolicyKey {
    ty: TypeId,
    policy: ValuePolicy,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum ValuePolicy {
    Callable {
        argument_policy: CallbackArgumentPolicy,
        parameters: Vec<ParameterPolicy>,
        variadic: VariadicPolicy,
        results: Vec<(TypeId, Option<CallablePolicyKey>)>,
    },
    Pointer(Box<CallablePolicyKey>),
    Sequence(Box<CallablePolicyKey>),
    Record {
        bindings: RecordBindings,
        fields: Vec<(FieldId, Option<CallablePolicyKey>)>,
    },
    RecursiveRecord(RecordBindings),
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct RecordBindings(HashMap<Symbol, Option<CallablePolicyKey>>);
impl Hash for RecordBindings {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let mut entries = self
            .0
            .iter()
            .map(|entry| {
                let mut entry_hash = std::collections::hash_map::DefaultHasher::new();
                entry.hash(&mut entry_hash);
                entry_hash.finish()
            })
            .collect::<Vec<_>>();
        entries.sort_unstable();
        entries.hash(state);
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct ParameterPolicy {
    name: Option<Symbol>,
    ty: TypeId,
    evaluation: EvaluationPolicy,
    default: Option<DefaultPolicy>,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum EvaluationPolicy {
    Evaluate,
    Discard,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum VariadicPolicy {
    None,
    Jai(usize),
    C(usize),
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum DefaultPolicy {
    PendingSource {
        key: crate::source_parameter_defaults::SourceParameterDefaultKey,
        bytes: std::sync::Arc<str>,
    },
    Constant(jai_ir::ConstantValue),
    RuntimeRead(RuntimeReadPolicy),
    CallerLocation(TypeId),
    CodeNull(TypeId),
    Discarded,
}
// Diagnostic locations do not change the executable storage-read recipe.
#[derive(Clone, Debug)]
struct RuntimeReadPolicy(crate::runtime_defaults::RuntimeDefaultRead);
impl PartialEq for RuntimeReadPolicy {
    fn eq(&self, other: &Self) -> bool {
        self.0.root() == other.0.root()
            && self.0.steps() == other.0.steps()
            && self.0.ty() == other.0.ty()
    }
}
impl Eq for RuntimeReadPolicy {
}
impl Hash for RuntimeReadPolicy {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.root().hash(state);
        self.0.steps().hash(state);
        self.0.ty().hash(state);
    }
}
impl Resolver<'_> {
    pub(crate) fn specialization_callable_policy(
        &mut self,
        source: &syntax::Expression,
        expected: TypeId,
        span: Span,
    ) -> Result<Option<super::CallablePolicyKey>, Diagnostic> {
        self.preview_expected_callback_contract(source, expected, span)?
            .map(|contract| {
                self.callable_policy_key(&contract, &mut std::collections::HashSet::new(), span, 0)
            })
            .transpose()
    }
    pub(crate) fn preview_expected_callback_contract(
        &mut self,
        source: &syntax::Expression,
        expected: TypeId,
        span: Span,
    ) -> Result<Option<ValueContract>, Diagnostic> {
        let contract = match &source.kind {
            syntax::ExpressionKind::BakeArguments(source) => {
                let metadata =
                    self.preview_baked_procedure(&source.callee, &source.arguments, span)?;
                Some(Self::preview_lambda_contract(metadata))
            }
            syntax::ExpressionKind::AnonymousProcedure(procedure) => {
                let header = self.preview_anonymous_procedure_argument(procedure)?;
                if header.ty != expected {
                    return Err(Diagnostic::new(
                        span,
                        "anonymous procedure header does not match its contextual type",
                    ));
                }
                Some(self.preview_source_header_contract(
                    &header,
                    &procedure.header.callable,
                    span,
                )?)
            }
            syntax::ExpressionKind::ShortLambda(lambda) => {
                let metadata = self.preview_short_lambda_signature(lambda, expected, span)?;
                Some(Self::preview_lambda_contract(metadata))
            }
            syntax::ExpressionKind::TypeCast {
                ty, ..
            } => self.annotation_value_contract(expected, ty, span)?,
            syntax::ExpressionKind::Name(name) => {
                let path = syntax::NamePath {
                    root: *name,
                    members: vec![],
                };
                match self.preview_named_short_lambda_signature(&path, expected, span)? {
                    Some(metadata) => Some(Self::preview_lambda_contract(metadata)),
                    None => self.preview_callback_expression_contract(source)?,
                }
            }
            syntax::ExpressionKind::QualifiedName(path) => {
                match self.preview_named_short_lambda_signature(path, expected, span)? {
                    Some(metadata) => Some(Self::preview_lambda_contract(metadata)),
                    None => self.preview_callback_expression_contract(source)?,
                }
            }
            _ => self.preview_callback_expression_contract(source)?,
        };
        match contract {
            Some(contract) => self.preview_converted_contract(contract, expected, span),
            None => {
                if self.callback_contract_type(expected, 0)
                    && matches!(
                        source.kind,
                        syntax::ExpressionKind::Unary(_, _)
                            | syntax::ExpressionKind::Binary(_, _, _)
                            | syntax::ExpressionKind::Index { .. }
                    )
                    && let Some(preview) = self.describe_operator_expression(source)
                {
                    preview?;
                    return Err(Diagnostic::new(
                        source.span,
                        "nested operator callback policies require a checked source result preview",
                    ));
                }
                Ok(None)
            }
        }
    }
    fn preview_converted_contract(
        &mut self,
        mut contract: ValueContract,
        expected: TypeId,
        span: Span,
    ) -> Result<Option<ValueContract>, Diagnostic> {
        if contract.ty == expected {
            return Ok(Some(contract));
        }
        if let (
            Ok(
                jai_types::TypeKind::FixedArray {
                    element, ..
                }
                | jai_types::TypeKind::Slice(element)
                | jai_types::TypeKind::DynamicArray(element),
            ),
            Ok(jai_types::TypeKind::Slice(target)),
        ) = (self.types.kind(contract.ty), self.types.kind(expected))
            && element == target
            && matches!(contract.kind, ContractKind::Sequence(_))
        {
            contract.ty = expected;
            return Ok(Some(contract));
        }
        let (source, target, pointer) =
            match (self.types.kind(contract.ty), self.types.kind(expected)) {
                (Ok(jai_types::TypeKind::Record(_)), _) => (contract.ty, expected, false),
                (
                    Ok(jai_types::TypeKind::Pointer(source)),
                    Ok(jai_types::TypeKind::Pointer(target)),
                ) => (*source, *target, true),
                _ => return Ok(None),
            };
        let Some(path) = self.conversion_path(source, Some(target), span)? else {
            return Ok(None);
        };
        let mut receiver = (
            source,
            if pointer {
                contract.element()
            } else {
                Some(contract)
            },
        );
        for field in path {
            receiver = self.preview_field_contract(receiver, field, span, 0)?;
        }
        Ok(receiver.1.map(|contract| {
            if pointer {
                ValueContract {
                    ty: expected,
                    kind: ContractKind::Pointer(Box::new(contract)),
                }
            } else {
                contract
            }
        }))
    }
    pub(crate) fn specialization_type_callable_policy(
        &self,
        source: &syntax::TypeSyntax,
        actual: TypeId,
        span: Span,
    ) -> Result<Option<super::CallablePolicyKey>, Diagnostic> {
        self.annotation_value_contract(actual, source, span)?
            .map(|contract| {
                self.callable_policy_key(&contract, &mut std::collections::HashSet::new(), span, 0)
            })
            .transpose()
    }
    pub(crate) fn preview_callback_expression_contract(
        &mut self,
        source: &syntax::Expression,
    ) -> Result<Option<ValueContract>, Diagnostic> {
        self.preview_source_contract(source, 0)
    }
    fn preview_lambda_contract(metadata: CallbackSignature) -> ValueContract {
        ValueContract {
            ty: metadata.ty,
            kind: ContractKind::Callable {
                results: vec![None; metadata.results.len()],
                metadata,
                source: None,
            },
        }
    }
    fn preview_source_contract(
        &mut self,
        source: &syntax::Expression,
        depth: usize,
    ) -> Result<Option<ValueContract>, Diagnostic> {
        if depth >= crate::constant_limits::MAX_CONSTANT_DEPTH {
            return Err(Diagnostic::new(
                source.span,
                "callback policy exceeds source expression depth",
            ));
        }
        let contract = match &source.kind {
            syntax::ExpressionKind::BakeArguments(value) => {
                let metadata =
                    self.preview_baked_procedure(&value.callee, &value.arguments, source.span)?;
                Some(Self::preview_lambda_contract(metadata))
            }
            syntax::ExpressionKind::AnonymousProcedure(procedure) => {
                let header = self.preview_anonymous_procedure_argument(procedure)?;
                Some(self.preview_source_header_contract(
                    &header,
                    &procedure.header.callable,
                    source.span,
                )?)
            }
            syntax::ExpressionKind::Name(name) => self.preview_binding_contract(
                &syntax::NamePath {
                    root: *name,
                    members: vec![],
                },
                source.span,
            )?,
            syntax::ExpressionKind::QualifiedName(path) => {
                self.preview_binding_contract(path, source.span)?
            }
            syntax::ExpressionKind::CallHint {
                call, ..
            } => self.preview_source_contract(call, depth + 1)?,
            syntax::ExpressionKind::InferredCast {
                value, ..
            } => self.preview_source_contract(value, depth + 1)?,
            syntax::ExpressionKind::TypeCast {
                ty, ..
            } => {
                let retained = self.retained_callback_syntax(ty, source.span)?;
                let checked = retained
                    .callback
                    .as_ref()
                    .and_then(|callback| callback.proof.as_ref())
                    .map(|proof| proof.ty())
                    .or_else(|| match ty {
                        syntax::TypeSyntax::Named(path) => self.local_ready_type_path(path),
                        _ => None,
                    });
                match checked {
                    Some(ty) => {
                        self.normalized_value_contract(ty, &retained, source.span, depth + 1)?
                    }
                    None => None,
                }
            }
            syntax::ExpressionKind::Member {
                ..
            } => self
                .preview_receiver_contract(source, depth + 1)?
                .and_then(|(_, contract)| contract),
            syntax::ExpressionKind::Index {
                base, ..
            } => match self.preview_source_contract(base, depth + 1)? {
                Some(ValueContract {
                    kind: ContractKind::Sequence(element),
                    ..
                }) => Some(*element),
                _ => None,
            },
            syntax::ExpressionKind::Dereference(base) => {
                match self.preview_source_contract(base, depth + 1)? {
                    Some(ValueContract {
                        kind: ContractKind::Pointer(element),
                        ..
                    }) => Some(*element),
                    _ => None,
                }
            }
            syntax::ExpressionKind::Call(name, _) => self.preview_named_call_contract(
                &syntax::NamePath {
                    root: *name,
                    members: vec![],
                },
                source.span,
            )?,
            syntax::ExpressionKind::QualifiedCall(path, _) => {
                self.preview_named_call_contract(path, source.span)?
            }
            syntax::ExpressionKind::IndirectCall {
                callee, ..
            }
            | syntax::ExpressionKind::ContextCall {
                callee, ..
            } => self
                .preview_source_contract(callee, depth + 1)?
                .and_then(|contract| contract.result(0)),
            syntax::ExpressionKind::Conditional(branches) => match (
                self.preview_source_contract(branches.then_source(), depth + 1)?,
                branches
                    .else_value
                    .as_deref()
                    .map(|value| self.preview_source_contract(value, depth + 1))
                    .transpose()?
                    .flatten(),
            ) {
                (Some(left), Some(right)) if left.ty == right.ty => Some(left.merge(right)),
                _ => None,
            },
            _ => None,
        };
        Ok(contract)
    }
    fn preview_binding_contract(
        &mut self,
        path: &syntax::NamePath,
        span: Span,
    ) -> Result<Option<ValueContract>, Diagnostic> {
        if let Some(contract) = self.checked_baked_constant_binding_contract(path, span)? {
            return Ok(contract);
        }
        if let Some(contract) = self.checked_baked_callback_binding_contract(path, span)? {
            return Ok(Some(contract));
        }
        if let Some(binding) = self.lexical_graph_binding_ready(path, span)?
            && let Some(scope) = self.graph_scope
            && let Some(declarations) = scope.callback_imported_declarations(binding, span)?
        {
            let [declaration] = declarations.as_slice() else {
                return Ok(None);
            };
            return match scope.concrete_signature(*declaration) {
                Some(signature) => self.preview_procedure_contract(signature.id, span),
                None => Ok(None),
            };
        }
        if !path.members.is_empty() {
            let root = syntax::NamePath {
                root: path.root,
                members: vec![],
            };
            if let Ok(Binding::Storage(storage)) = self.lookup_path(&root, span) {
                let mut receiver = (
                    storage.place().ty(),
                    self.callback_place_contract(storage.place(), span, 0)?,
                );
                for member in &path.members {
                    receiver = self.preview_member_contract(receiver, *member, span, 0)?;
                }
                return Ok(receiver.1);
            }
        }
        let binding = match self.lookup_path(path, span) {
            Ok(binding) => binding,
            Err(error) => {
                if !self.local_name_present(path.root)
                    && let Some(scope) = self.graph_scope
                    && let Ok(signature) = scope.signature(path, span)
                {
                    return self.preview_procedure_contract(signature.id, span);
                }
                return Err(error);
            }
        };
        match binding {
            Binding::Storage(storage) => self.callback_place_contract(storage.place(), span, 0),
            Binding::Procedure {
                procedure, ..
            } => self.preview_procedure_contract(procedure, span),
            Binding::TypedConstant(id) => {
                let value = self.meta.constant(id).cloned().ok_or_else(|| {
                    Diagnostic::new(span, "callback constant identity is unavailable")
                })?;
                self.callback_value_contract(&value.into_expression(), span)
            }
            _ => Ok(None),
        }
    }
    fn preview_receiver_contract(
        &mut self,
        source: &syntax::Expression,
        depth: usize,
    ) -> Result<Option<(TypeId, Option<ValueContract>)>, Diagnostic> {
        if depth >= crate::constant_limits::MAX_CONSTANT_DEPTH {
            return Err(Diagnostic::new(
                source.span,
                "callback receiver exceeds source expression depth",
            ));
        }
        match &source.kind {
            syntax::ExpressionKind::Name(name) => {
                let path = syntax::NamePath {
                    root: *name,
                    members: vec![],
                };
                if let Ok(Binding::Storage(storage)) = self.lookup_path(&path, source.span) {
                    return Ok(Some((
                        storage.place().ty(),
                        self.callback_place_contract(storage.place(), source.span, depth + 1)?,
                    )));
                }
            }
            syntax::ExpressionKind::QualifiedName(path) => {
                let root = syntax::NamePath {
                    root: path.root,
                    members: vec![],
                };
                if let Ok(Binding::Storage(storage)) = self.lookup_path(&root, source.span) {
                    let mut receiver = (
                        storage.place().ty(),
                        self.callback_place_contract(storage.place(), source.span, depth + 1)?,
                    );
                    for member in &path.members {
                        receiver = self.preview_member_contract(
                            receiver,
                            *member,
                            source.span,
                            depth + 1,
                        )?;
                    }
                    return Ok(Some(receiver));
                }
            }
            syntax::ExpressionKind::Member {
                base,
                member,
            } => {
                return self
                    .preview_receiver_contract(base, depth + 1)?
                    .map(|receiver| {
                        self.preview_member_contract(receiver, *member, source.span, depth + 1)
                    })
                    .transpose();
            }
            syntax::ExpressionKind::Dereference(base) => {
                return Ok(self.preview_receiver_contract(base, depth + 1)?.and_then(
                    |(ty, contract)| {
                        let jai_types::TypeKind::Pointer(element) = self.types.kind(ty).ok()?
                        else {
                            return None;
                        };
                        Some((*element, contract.and_then(|contract| contract.element())))
                    },
                ));
            }
            syntax::ExpressionKind::Index {
                base, ..
            } => {
                return Ok(self.preview_receiver_contract(base, depth + 1)?.and_then(
                    |(ty, contract)| {
                        let element = match self.types.kind(ty).ok()? {
                            jai_types::TypeKind::FixedArray {
                                element, ..
                            }
                            | jai_types::TypeKind::Slice(element)
                            | jai_types::TypeKind::DynamicArray(element) => *element,
                            _ => return None,
                        };
                        Some((element, contract.and_then(|contract| contract.element())))
                    },
                ));
            }
            _ => {}
        }
        Ok(self
            .preview_source_contract(source, depth + 1)?
            .map(|contract| (contract.ty, Some(contract))))
    }
    fn preview_member_contract(
        &mut self,
        mut receiver: (TypeId, Option<ValueContract>),
        member: Symbol,
        span: Span,
        depth: usize,
    ) -> Result<(TypeId, Option<ValueContract>), Diagnostic> {
        if let Ok(jai_types::TypeKind::Pointer(element)) = self.types.kind(receiver.0) {
            receiver.0 = *element;
            receiver.1 = receiver.1.and_then(|contract| contract.element());
        }
        if let Some(path) = self.optional_field_path(receiver.0, member, span)? {
            for field in path {
                receiver = self.preview_field_contract(receiver, field, span, depth + 1)?;
            }
            return Ok(receiver);
        }
        use crate::modules::aggregates::parameterized::ReadyInstanceRecordMember;
        let fact =
            if let Some(fact) = self.ready_instance_record_member(receiver.0, member, span)? {
                Some(fact)
            } else if self
                .meta
                .record_specializations
                .record(receiver.0)
                .is_some()
            {
                None
            } else {
                self.ready_namespace_member(receiver.0, member)
                    .map(ReadyInstanceRecordMember::Binding)
            };
        match fact.ok_or_else(|| Diagnostic::new(span, "unknown record member"))? {
            ReadyInstanceRecordMember::Binding(binding) => {
                let info = self.describe_binding(binding.clone(), span)?;
                let ty = self.argument_type(&info, span)?;
                let contract = match binding {
                    Binding::Procedure {
                        procedure, ..
                    } => self.preview_procedure_contract(procedure, span)?,
                    Binding::Storage(storage) => {
                        self.callback_place_contract(storage.place(), span, depth + 1)?
                    }
                    Binding::TypedConstant(id) => {
                        let value = self.meta.constant(id).cloned().ok_or_else(|| {
                            Diagnostic::new(span, "callback constant identity is unavailable")
                        })?;
                        self.callback_value_contract_inner(
                            &value.into_expression(),
                            span,
                            depth + 1,
                        )?
                    }
                    _ => None,
                };
                Ok((ty, contract))
            }
            ReadyInstanceRecordMember::Baked(value) => {
                let (ty, contract) = match value {
                    crate::polymorphism::BakedValue::Value(value) => {
                        let ty = value.ty;
                        let contract = self.callback_value_contract_inner(
                            &value.into_expression(),
                            span,
                            depth + 1,
                        )?;
                        (ty, contract)
                    }
                    crate::polymorphism::BakedValue::Type(_) => (self.types.meta_type(), None),
                    crate::polymorphism::BakedValue::Float(value) => {
                        (self.types.float(value.ty()), None)
                    }
                    crate::polymorphism::BakedValue::String(_) => (self.types.string(), None),
                    crate::polymorphism::BakedValue::Code(_) => (self.types.code_type(), None),
                };
                Ok((ty, contract))
            }
        }
    }
    fn preview_field_contract(
        &mut self,
        receiver: (TypeId, Option<ValueContract>),
        field: FieldId,
        span: Span,
        depth: usize,
    ) -> Result<(TypeId, Option<ValueContract>), Diagnostic> {
        let contract = receiver
            .1
            .as_ref()
            .map(|contract| self.record_field_value_contract(contract, field, span, depth + 1))
            .transpose()?
            .flatten();
        let contract = match contract {
            Some(contract) => Some(contract),
            None => self.callback_field_contract(receiver.0, field, span)?,
        };
        let ty = self
            .types
            .field_type(field)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?;
        Ok((ty, contract))
    }
    fn preview_procedure_contract(
        &mut self,
        procedure: ProcedureId,
        span: Span,
    ) -> Result<Option<ValueContract>, Diagnostic> {
        let Some(signature) = self.contract_procedure_signature(procedure) else {
            return Ok(None);
        };
        let results = self.procedure_result_contracts(procedure, span)?;
        Ok(Some(ValueContract {
            ty: signature.ty,
            kind: ContractKind::Callable {
                metadata: CallbackSignature::source(&signature),
                results,
                source: Some(procedure),
            },
        }))
    }
    fn preview_named_call_contract(
        &mut self,
        path: &syntax::NamePath,
        span: Span,
    ) -> Result<Option<ValueContract>, Diagnostic> {
        Ok(self
            .preview_binding_contract(path, span)?
            .and_then(|contract| contract.result(0)))
    }
    fn callable_policy_key(
        &self,
        contract: &ValueContract,
        active: &mut std::collections::HashSet<(TypeId, RecordBindings)>,
        span: Span,
        depth: usize,
    ) -> Result<CallablePolicyKey, Diagnostic> {
        if depth >= crate::constant_limits::MAX_CONSTANT_DEPTH {
            return Err(Diagnostic::new(
                span,
                "callback policy exceeds checked contract depth",
            ));
        }
        let policy = match &contract.kind {
            ContractKind::Callable {
                metadata,
                results,
                ..
            } => {
                let parameters = metadata
                    .parameters
                    .iter()
                    .map(|parameter| ParameterPolicy {
                        name: parameter.name,
                        ty: parameter.ty,
                        evaluation: match parameter.evaluation {
                            syntax::ParameterEvaluation::Evaluate => EvaluationPolicy::Evaluate,
                            syntax::ParameterEvaluation::Discard => EvaluationPolicy::Discard,
                        },
                        default: parameter
                            .default
                            .as_ref()
                            .map(|default| default_policy(default, parameter.ty)),
                    })
                    .collect();
                let variadic = match metadata.source_variadic {
                    crate::overloads::CandidateVariadic::None => VariadicPolicy::None,
                    crate::overloads::CandidateVariadic::Jai {
                        parameter,
                    } => VariadicPolicy::Jai(parameter),
                    crate::overloads::CandidateVariadic::C {
                        fixed_parameters,
                    } => VariadicPolicy::C(fixed_parameters),
                };
                let results = metadata
                    .results
                    .iter()
                    .enumerate()
                    .map(|(index, result)| {
                        let nested = results
                            .get(index)
                            .and_then(Option::as_ref)
                            .map(|contract| {
                                self.callable_policy_key(contract, active, span, depth + 1)
                            })
                            .transpose()?;
                        Ok((result.ty, nested))
                    })
                    .collect::<Result<Vec<_>, Diagnostic>>()?;
                ValuePolicy::Callable {
                    argument_policy: metadata.argument_policy,
                    parameters,
                    variadic,
                    results,
                }
            }
            ContractKind::Pointer(element) => ValuePolicy::Pointer(Box::new(
                self.callable_policy_key(element, active, span, depth + 1)?,
            )),
            ContractKind::Sequence(element) => ValuePolicy::Sequence(Box::new(
                self.callable_policy_key(element, active, span, depth + 1)?,
            )),
            ContractKind::Record(record) => {
                let bindings = record
                    .bindings
                    .iter()
                    .map(|(&name, contract)| {
                        Ok((
                            name,
                            contract
                                .as_ref()
                                .map(|contract| {
                                    self.callable_policy_key(contract, active, span, depth + 1)
                                })
                                .transpose()?,
                        ))
                    })
                    .collect::<Result<HashMap<_, _>, Diagnostic>>()?;
                let bindings = RecordBindings(bindings);
                let identity = (contract.ty, bindings.clone());
                if !active.insert(identity.clone()) {
                    ValuePolicy::RecursiveRecord(bindings)
                } else {
                    let metadata = self.record_metadata(contract.ty, span)?;
                    let fields = metadata
                        .fields
                        .iter()
                        .map(|field| {
                            let nested = self.record_field_value_contract(
                                contract,
                                field.id,
                                span,
                                depth + 1,
                            )?;
                            Ok((
                                field.id,
                                nested
                                    .as_ref()
                                    .map(|contract| {
                                        self.callable_policy_key(contract, active, span, depth + 1)
                                    })
                                    .transpose()?,
                            ))
                        })
                        .collect::<Result<Vec<_>, Diagnostic>>()?;
                    active.remove(&identity);
                    ValuePolicy::Record {
                        bindings,
                        fields,
                    }
                }
            }
        };
        Ok(CallablePolicyKey {
            ty: contract.ty,
            policy,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime_defaults::{DefaultReadRoot, DefaultReadStep, RuntimeDefaultRead};
    use jai_source::{SourceMap, SourceSpan};
    use jai_types::{IntegerType, RecordKind, ScalarType, TypeRegistry};

    #[test]
    fn runtime_default_policy_ignores_diagnostic_span_but_retains_checked_path() {
        let mut types = TypeRegistry::new();
        let integer = types.scalar(ScalarType::Int(IntegerType::S64));
        let record = types.reserve_record(RecordKind::Struct);
        types.define_record(record, vec![integer, integer]).unwrap();
        let mut sources = SourceMap::default();
        let source = sources.insert("defaults.jai".into(), "left right".into());
        let policy = |index, span| {
            RuntimeReadPolicy(
                RuntimeDefaultRead::checked(
                    DefaultReadRoot::Context {
                        ty: record,
                    },
                    vec![DefaultReadStep::Field(
                        types.field(record, index).unwrap().id,
                    )],
                    integer,
                    SourceSpan {
                        source,
                        span,
                    },
                    &types,
                )
                .unwrap(),
            )
        };
        let first = policy(0, Span::new(0, 4));
        let same = policy(0, Span::new(5, 10));
        let other = policy(1, Span::new(0, 4));
        assert_eq!(first, same);
        assert_ne!(first, other);
        let hash = |value: &RuntimeReadPolicy| {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            value.hash(&mut hasher);
            hasher.finish()
        };
        assert_eq!(hash(&first), hash(&same));
    }
}

fn default_policy(default: &ParameterDefault, ty: TypeId) -> DefaultPolicy {
    match default {
        ParameterDefault::Source(source) => match source.ready() {
            Some(value) => default_policy(value, ty),
            None => DefaultPolicy::PendingSource {
                key: source.key,
                bytes: source.source_bytes.clone(),
            },
        },
        ParameterDefault::Constant(value) => DefaultPolicy::Constant(value.clone()),
        ParameterDefault::RuntimeRead(read) => {
            DefaultPolicy::RuntimeRead(RuntimeReadPolicy(read.clone()))
        }
        ParameterDefault::CallerLocation => DefaultPolicy::CallerLocation(ty),
        ParameterDefault::CodeNull {
            ty,
        } => DefaultPolicy::CodeNull(*ty),
        ParameterDefault::Discarded => DefaultPolicy::Discarded,
    }
}
