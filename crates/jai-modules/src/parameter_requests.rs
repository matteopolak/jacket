//! Retained semantic requests carry source identities, never registry-local IDs.
use super::*;
use jai_syntax::TypeSyntax;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_SESSION: AtomicU64 = AtomicU64::new(1);

/// A request belongs to exactly one retained source discovery session.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ParameterRequestId {
    session: u64,
    index: usize,
}
impl ParameterRequestId {
    pub fn index(self) -> usize {
        self.index
    }
}

/// Original source inputs for work that the graph's pure binder cannot prove.
#[derive(Clone, Debug)]
pub enum ParameterTask {
    ResolveType {
        syntax: TypeSyntax,
    },
    CheckInterface {
        actual: ModuleType,
        required: ModuleType,
    },
    CheckNominal {
        actual: ModuleType,
        required: ModuleType,
    },
    CoerceValue {
        expected: ModuleType,
        value: ParameterValue,
        program_wide: bool,
    },
}
impl ParameterTask {
    fn same_work(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::ResolveType { syntax: left }, Self::ResolveType { syntax: right }) => {
                same_type_source(left, right)
            }
            (
                Self::CheckInterface {
                    actual: left,
                    required: left_required,
                },
                Self::CheckInterface {
                    actual: right,
                    required: right_required,
                },
            ) => left == right && left_required == right_required,
            (
                Self::CheckNominal {
                    actual: left,
                    required: left_required,
                },
                Self::CheckNominal {
                    actual: right,
                    required: right_required,
                },
            ) => left == right && left_required == right_required,
            (
                Self::CoerceValue {
                    expected: left,
                    value: left_value,
                    program_wide: left_program,
                },
                Self::CoerceValue {
                    expected: right,
                    value: right_value,
                    program_wide: right_program,
                },
            ) => left == right && left_value == right_value && left_program == right_program,
            _ => false,
        }
    }
    fn accepts(&self, response: &ParameterResponse) -> bool {
        matches!(
            (self, response),
            (Self::ResolveType { .. }, ParameterResponse::Type(_))
                | (
                    Self::CheckInterface { .. },
                    ParameterResponse::InterfaceSatisfied
                )
                | (Self::CoerceValue { .. }, ParameterResponse::Value(_))
                | (
                    Self::CheckNominal { .. },
                    ParameterResponse::NominalSatisfied
                )
        )
    }
}

/// A semantic decision is expressed in source identities so it can be
/// rematerialized in the final semantic registry. This is not an effect receipt:
/// a scheduler must commit/replay compiler effects before submitting a response.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParameterResponse {
    Type(ModuleType),
    Value(ParameterValue),
    InterfaceSatisfied,
    NominalSatisfied,
}
#[derive(Clone, Debug)]
pub struct DeferredParameter {
    pub id: ParameterRequestId,
    pub file: FileInstanceId,
    pub module: ModuleId,
    pub specialization: Option<SourceSpecializationKey>,
    pub location: SourceSpan,
    pub task: ParameterTask,
    pub response: Option<ParameterResponse>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParameterResponseError {
    UnknownRequest,
    WrongResponseKind,
    InvalidSourceIdentity,
    ContextualValue,
    AlreadyResolved,
}
impl fmt::Display for ParameterResponseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::UnknownRequest => "parameter request does not belong to this discovery session",
            Self::WrongResponseKind => {
                "semantic response does not match the parameter request kind"
            }
            Self::InvalidSourceIdentity => {
                "semantic response references an unavailable source declaration"
            }
            Self::ContextualValue => "semantic response contains an unbound contextual enum member",
            Self::AlreadyResolved => "parameter request already has a different immutable response",
        })
    }
}
impl std::error::Error for ParameterResponseError {}

#[derive(Clone)]
pub(super) struct ParameterRequestStore {
    session: u64,
    requests: Vec<DeferredParameter>,
}
impl Default for ParameterRequestStore {
    fn default() -> Self {
        Self {
            session: NEXT_SESSION.fetch_add(1, Ordering::Relaxed),
            requests: vec![],
        }
    }
}
impl ParameterRequestStore {
    fn retain(
        &mut self,
        file: FileInstanceId,
        module: ModuleId,
        location: SourceSpan,
        specialization: Option<SourceSpecializationKey>,
        task: ParameterTask,
    ) -> Option<ParameterResponse> {
        if let Some(request) = self.requests.iter().find(|request| {
            request.file == file
                && request.location == location
                && request.specialization == specialization
                && request.task.same_work(&task)
        }) {
            return request.response.clone();
        }
        self.requests.push(DeferredParameter {
            id: ParameterRequestId {
                session: self.session,
                index: self.requests.len(),
            },
            file,
            module,
            specialization,
            location,
            task,
            response: None,
        });
        None
    }
}

/// A shared outer annotation span can contain several different pending type
/// applications. Original source anchors distinguish those tasks without
/// introducing semantic identities or comparing printed ASTs.
fn same_type_source(left: &TypeSyntax, right: &TypeSyntax) -> bool {
    use TypeSyntax as T;
    match (left, right) {
        (T::This, T::This) => true,
        (T::Builtin(left), T::Builtin(right)) => left == right,
        (T::Named(left), T::Named(right)) => left == right,
        (T::Variable(left), T::Variable(right)) => left == right,
        (
            T::Restricted {
                variable: left,
                restriction: left_restriction,
                span: left_span,
            },
            T::Restricted {
                variable: right,
                restriction: right_restriction,
                span: right_span,
            },
        ) => {
            left == right
                && left_span == right_span
                && match (left_restriction, right_restriction) {
                    (
                        jai_syntax::TypeRestrictionSyntax::Nominal(left),
                        jai_syntax::TypeRestrictionSyntax::Nominal(right),
                    )
                    | (
                        jai_syntax::TypeRestrictionSyntax::Interface(left),
                        jai_syntax::TypeRestrictionSyntax::Interface(right),
                    ) => same_type_source(left, right),
                    _ => false,
                }
        }
        (T::TypeOf(left), T::TypeOf(right)) => left.span == right.span,
        (T::InlineRecord(left), T::InlineRecord(right)) => left.span == right.span,
        (T::InlineEnum(left), T::InlineEnum(right)) => left.span == right.span,
        (T::Application(left), T::Application(right)) => {
            left.span == right.span && same_type_source(&left.base, &right.base)
        }
        (
            T::Variant {
                kind: left_kind,
                base: left,
            },
            T::Variant {
                kind: right_kind,
                base: right,
            },
        ) => left_kind == right_kind && same_type_source(left, right),
        (T::Pointer(left), T::Pointer(right))
        | (T::Slice(left), T::Slice(right))
        | (T::DynamicArray(left), T::DynamicArray(right)) => same_type_source(left, right),
        (
            T::FixedArray {
                count: left_count,
                element: left,
            },
            T::FixedArray {
                count: right_count,
                element: right,
            },
        ) => left_count.span == right_count.span && same_type_source(left, right),
        (T::Procedure(left), T::Procedure(right)) => {
            left.convention == right.convention
                && left.context == right.context
                && left.parameters.len() == right.parameters.len()
                && left.results.len() == right.results.len()
                && left
                    .parameters
                    .iter()
                    .chain(&left.results)
                    .zip(right.parameters.iter().chain(&right.results))
                    .all(|(left, right)| {
                        left.span == right.span && same_type_source(&left.ty, &right.ty)
                    })
        }
        _ => false,
    }
}

impl GraphDiscovery<'_> {
    /// Immutable snapshots retain the original AST and defining file/module.
    pub fn parameter_requests(&self) -> Vec<DeferredParameter> {
        self.builder.semantic_parameters.borrow().requests.clone()
    }
    pub fn pending_parameter_requests(&self) -> Vec<DeferredParameter> {
        self.parameter_requests()
            .into_iter()
            .filter(|request| request.response.is_none())
            .collect()
    }
    /// The semantic scheduler is responsible for conformance and evaluation.
    /// Submission validates session, response shape, and source identity. Equal
    /// repeats are idempotent; changing a committed response is rejected.
    pub fn resolve_parameter(
        &mut self,
        id: ParameterRequestId,
        response: ParameterResponse,
    ) -> Result<(), ParameterResponseError> {
        self.invalidate_insertion_admissions();
        let mut store = self.builder.semantic_parameters.borrow_mut();
        if id.session != store.session {
            return Err(ParameterResponseError::UnknownRequest);
        }
        let request = store
            .requests
            .get_mut(id.index)
            .ok_or(ParameterResponseError::UnknownRequest)?;
        if !request.task.accepts(&response) {
            return Err(ParameterResponseError::WrongResponseKind);
        }
        validate_response(&self.builder.graph, &response)?;
        if let Some(previous) = &request.response {
            if *previous != response {
                return Err(ParameterResponseError::AlreadyResolved);
            }
        } else {
            request.response = Some(response);
        }
        Ok(())
    }
}
pub(super) fn validate_response(
    graph: &ModuleGraph,
    response: &ParameterResponse,
) -> Result<(), ParameterResponseError> {
    match response {
        ParameterResponse::Type(ty) => validate_type(graph, ty),
        ParameterResponse::Value(value) => validate_value(graph, value),
        ParameterResponse::InterfaceSatisfied | ParameterResponse::NominalSatisfied => Ok(()),
    }
}
fn validate_value(
    graph: &ModuleGraph,
    value: &ParameterValue,
) -> Result<(), ParameterResponseError> {
    match value {
        ParameterValue::Type(ty) => validate_type(graph, ty),
        ParameterValue::Enumeration(value) => {
            if !matches!(
                graph
                    .declaration(value.declaration)
                    .map(|d| &d.syntax().kind),
                Some(FileDeclarationKind::Enum(_))
            ) {
                return Err(ParameterResponseError::InvalidSourceIdentity);
            }
            Ok(())
        }
        ParameterValue::ContextualMember(_) => Err(ParameterResponseError::ContextualValue),
        ParameterValue::Scalar(_) | ParameterValue::String(_) => Ok(()),
    }
}
fn validate_type(graph: &ModuleGraph, ty: &ModuleType) -> Result<(), ParameterResponseError> {
    match ty {
        ModuleType::Builtin(_) => Ok(()),
        ModuleType::Declaration(id) => {
            let nominal = match graph
                .declaration(*id)
                .map(|declaration| &declaration.syntax().kind)
            {
                Some(FileDeclarationKind::Record(record)) => record.parameters.is_empty(),
                Some(FileDeclarationKind::Enum(_)) => true,
                Some(FileDeclarationKind::TypeAlias(alias)) => {
                    matches!(alias.ty, TypeSyntax::Variant { .. })
                }
                _ => false,
            };
            if !nominal {
                return Err(ParameterResponseError::InvalidSourceIdentity);
            }
            Ok(())
        }
        ModuleType::Application {
            template,
            arguments,
        } => {
            let Some(declaration) = graph.declaration(*template) else {
                return Err(ParameterResponseError::InvalidSourceIdentity);
            };
            let FileDeclarationKind::Record(record) = &declaration.syntax().kind else {
                return Err(ParameterResponseError::InvalidSourceIdentity);
            };
            if record.parameters.is_empty() || record.parameters.len() != arguments.len() {
                return Err(ParameterResponseError::InvalidSourceIdentity);
            }
            for argument in arguments {
                match argument {
                    ModuleBoundArgument::Type(ty) => validate_type(graph, ty)?,
                    ModuleBoundArgument::Enumeration(value) => {
                        validate_value(graph, &ParameterValue::Enumeration(*value))?
                    }
                    _ => {}
                }
            }
            Ok(())
        }
        ModuleType::Pointer(inner)
        | ModuleType::Slice(inner)
        | ModuleType::DynamicArray(inner)
        | ModuleType::FixedArray { element: inner, .. } => validate_type(graph, inner),
        ModuleType::Procedure(signature) => {
            for ty in signature.parameters.iter().chain(&signature.results) {
                validate_type(graph, ty)?;
            }
            Ok(())
        }
    }
}
impl Builder<'_> {
    pub(super) fn semantic_parameter(
        &self,
        file: FileInstanceId,
        location: SourceSpan,
        task: ParameterTask,
        message: &str,
    ) -> Result<ParameterResponse, GraphError> {
        let module = self.graph.files[file.index()].module;
        if let Some(response) = self.semantic_parameters.borrow_mut().retain(
            file,
            module,
            location,
            self.specialization_for_span(file, location.span).cloned(),
            task,
        ) {
            return Ok(response);
        }
        let diagnostic = self.graph.diagnostic(location, message);
        let rendered = diagnostic.render(&self.graph.sources);
        Err(GraphError::Pending {
            diagnostic,
            rendered,
        })
    }
}
