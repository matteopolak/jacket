//! Source identities of actual semantic procedure specializations.
use super::*;
use jai_syntax::{ParameterBinding, Procedure, TypeSyntax};
use std::hash::{Hash, Hasher};

/// Registry-independent identity, ordered by the semantic binder's formal list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceSpecializationKey {
    declaration: DeclarationId,
    procedure: SourceSpan,
    arguments: Box<[(Symbol, ModuleBoundArgument)]>,
}
impl SourceSpecializationKey {
    pub fn new(
        declaration: DeclarationId,
        procedure: SourceSpan,
        arguments: Vec<(Symbol, ModuleBoundArgument)>,
    ) -> Self {
        Self {
            declaration,
            procedure,
            arguments: arguments.into_boxed_slice(),
        }
    }
    pub fn declaration(&self) -> DeclarationId {
        self.declaration
    }
    pub fn procedure(&self) -> SourceSpan {
        self.procedure
    }
    pub fn arguments(&self) -> &[(Symbol, ModuleBoundArgument)] {
        &self.arguments
    }
}
impl Hash for SourceSpecializationKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.declaration.hash(state);
        self.procedure.source.hash(state);
        self.procedure.span.start.hash(state);
        self.procedure.span.end.hash(state);
        self.arguments.hash(state);
    }
}

/// A generic source body whose dependencies await an actual specialization.
#[derive(Clone, Copy, Debug)]
pub struct SourceDependencyTemplate {
    declaration: DeclarationId,
    location: SourceSpan,
    extent: SourceSpan,
}
impl SourceDependencyTemplate {
    pub fn declaration(&self) -> DeclarationId {
        self.declaration
    }
    pub fn location(&self) -> SourceSpan {
        self.location
    }
    /// Original source bounds; declaration identity may cover only its name.
    pub fn extent(&self) -> SourceSpan {
        self.extent
    }
}

impl GraphDiscovery<'_> {
    /// Queue the original source body after the semantic binder has published
    /// its canonical specialization. Equal submissions are idempotent.
    pub fn discover_specialization(
        &mut self,
        key: SourceSpecializationKey,
    ) -> Result<bool, GraphError> {
        self.invalidate_insertion_admissions();
        let graph = &self.builder.graph;
        let declaration = graph.declaration(key.declaration).ok_or_else(|| {
            self.builder.located(
                key.procedure,
                "specialization references an unavailable source declaration",
            )
        })?;
        if graph.files[declaration.file.index()].source != key.procedure.source {
            return Err(self.builder.located(
                key.procedure,
                "specialization procedure does not belong to the defining source",
            ));
        }
        if !graph.dependency_templates.iter().any(|template| {
            template.declaration == key.declaration && template.location == key.procedure
        }) {
            return Err(self.builder.located(
                key.procedure,
                "specialization does not identify a discovered dependency template",
            ));
        }
        let mut names = std::collections::HashSet::new();
        for (name, value) in key.arguments.iter() {
            if !names.insert(*name) {
                return Err(self.builder.located(
                    key.procedure,
                    "specialization contains duplicate named bindings",
                ));
            }
            validate_argument(graph, value)
                .map_err(|error| self.builder.located(key.procedure, error.to_string()))?;
        }
        if self.specialization_discovered(&key)
            || self.builder.pending_specializations.contains(&key)
        {
            return Ok(false);
        }
        self.builder.pending_specializations.push(key);
        Ok(true)
    }
    pub fn has_dependency_templates(&self) -> bool {
        !self.builder.graph.dependency_templates.is_empty()
    }
    pub fn specialization_discovered(&self, key: &SourceSpecializationKey) -> bool {
        self.builder.graph.source_specializations.contains(key)
    }
}

fn validate_argument(
    graph: &ModuleGraph,
    value: &ModuleBoundArgument,
) -> Result<(), ParameterResponseError> {
    let response = match value {
        ModuleBoundArgument::Type(ty) => ParameterResponse::Type(ty.clone()),
        ModuleBoundArgument::Enumeration(value) => {
            ParameterResponse::Value(ParameterValue::Enumeration(*value))
        }
        _ => return Ok(()),
    };
    crate::parameter_requests::validate_response(graph, &response)
}

impl Builder<'_> {
    pub(super) fn scan_source_procedure(
        &mut self,
        declaration: DeclarationId,
        file: FileInstanceId,
        procedure: &Procedure,
    ) -> bool {
        let generic = needs_specialization(procedure);
        if generic {
            self.retain_dependency_template(declaration, file, procedure);
        }
        let Some(key) = self.active_specialization.as_ref() else {
            return !generic;
        };
        let source = self.graph.files[file.index()].source;
        if key.declaration != declaration || key.procedure.source != source {
            return false;
        }
        if generic {
            key.procedure.span == procedure.span
        } else {
            let Some(target) = self.graph.dependency_templates.iter().find(|template| {
                template.declaration == key.declaration && template.location == key.procedure
            }) else {
                return false;
            };
            let enclosing = procedure_extent(procedure);
            target.extent.span.start <= enclosing.end && target.extent.span.end >= enclosing.start
        }
    }
    pub(super) fn retain_dependency_template(
        &mut self,
        declaration: DeclarationId,
        file: FileInstanceId,
        procedure: &Procedure,
    ) {
        let location = SourceSpan {
            source: self.graph.files[file.index()].source,
            span: procedure.span,
        };
        if !self
            .graph
            .dependency_templates
            .iter()
            .any(|template| template.declaration == declaration && template.location == location)
        {
            self.graph
                .dependency_templates
                .push(SourceDependencyTemplate {
                    declaration,
                    location,
                    extent: SourceSpan {
                        source: location.source,
                        span: procedure_extent(procedure),
                    },
                });
        }
    }
}

fn procedure_extent(procedure: &Procedure) -> jai_source::Span {
    procedure
        .body
        .iter()
        .fold(procedure.span, |extent, statement| {
            jai_source::Span::new(
                extent.start.min(statement.span.start),
                extent.end.max(statement.span.end),
            )
        })
}

/// Structural syntax markers only; semantic matching supplies the actual key.
pub(super) fn needs_specialization(procedure: &Procedure) -> bool {
    procedure.modify.is_some()
        || procedure.parameters.iter().any(|parameter| {
            parameter.baking != jai_syntax::ParameterBaking::None
                || match &parameter.binding {
                    ParameterBinding::RequiredType(ty)
                    | ParameterBinding::DefaultedType {
                        ty: Some(ty), ..
                    } => generic_type(ty),
                    _ => false,
                }
        })
}
fn generic_type(ty: &TypeSyntax) -> bool {
    match ty {
        TypeSyntax::TypeOf(expression) => generic_expression(expression),
        TypeSyntax::Variable(_) => true,
        TypeSyntax::Restricted {
            ..
        } => true,
        TypeSyntax::Pointer(inner) | TypeSyntax::Slice(inner) | TypeSyntax::DynamicArray(inner) => {
            generic_type(inner)
        }
        TypeSyntax::FixedArray {
            element,
            count,
        } => generic_type(element) || generic_expression(count),
        TypeSyntax::Procedure(procedure) => procedure
            .parameters
            .iter()
            .chain(&procedure.results)
            .any(|parameter| generic_type(&parameter.ty)),
        TypeSyntax::Application(application) => application
            .arguments
            .iter()
            .any(|argument| generic_expression(&argument.value)),
        _ => false,
    }
}
fn generic_expression(expression: &jai_syntax::Expression) -> bool {
    use jai_syntax::ExpressionKind;
    match &expression.kind {
        ExpressionKind::CompileVariable(_) => true,
        ExpressionKind::Type(ty) => generic_type(ty),
        ExpressionKind::Call(_, arguments) | ExpressionKind::QualifiedCall(_, arguments) => {
            arguments
                .iter()
                .any(|argument| generic_expression(&argument.value))
        }
        ExpressionKind::AddressOf(inner)
        | ExpressionKind::CallHint {
            call: inner, ..
        }
        | ExpressionKind::InferredCast {
            value: inner, ..
        } => generic_expression(inner),
        _ => false,
    }
}
