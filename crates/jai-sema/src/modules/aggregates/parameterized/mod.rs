//! Instantiate nominal record templates with one mutable semantic owner.
mod arguments;
pub(crate) mod binder;
mod conditions;
mod default_overrides;
pub(crate) use default_overrides::RecordDefaultOverride;
mod expansion;
mod inline_types;
mod integration;
mod materialize;
mod modifier_execution;
mod modifier_intents;
pub(crate) use modifier_intents::{RecordModifierId, RecordModifierReadiness};
mod modifier_source;
pub(crate) use modifier_execution::RecordModifierPolicy;
mod record_body;
use record_body::RecordBody;
mod member_values;
mod namespace;
pub(crate) use namespace::ReadyInstanceRecordMember;
mod pending_types;
pub(crate) use pending_types::PendingType;
mod namespace_roots;
mod preparation;
mod type_queries;
mod type_resolution;
pub(super) use member_values::expression_path;
pub(crate) use member_values::member_value;
pub(crate) use preparation::{
    PendingRecordModifier, TypePreparation, prepare_type, prepare_type_paired,
};
use preparation::{TypeFailure, TypeResult, diagnostic_bridge, failure};
mod lexical;
pub(crate) use lexical::{LexicalTypeArgument, LexicalTypeArguments};
mod procedure_patterns;
mod using;
mod using_members;
pub(crate) use procedure_patterns::{
    FormalPatternRequest, formal_pattern, procedure_application_patterns, procedure_patterns,
    prototype_patterns,
};
mod member_constants;
mod members;
mod module_applications;
pub(crate) use module_applications::{ModuleApplicationRequest, instantiate_module_application};
mod methods;
mod reflection;
mod self_type;
mod state;
pub(crate) use methods::{RecordMethod, RecordMethodId, RecordMethodSource};
pub(crate) use self_type::NominalAnnotationContext;
mod member_enums;
mod source_layouts;
use super::types::Nominals;
use crate::local_declarations::{FieldMetadata, FieldSourceRef, RecordMetadata};
use crate::modules::{declaration_id, located, path};
use crate::polymorphism::{BakedValue, Substitution};
use crate::{Diagnostic, ScalarConstant, Span, TypeId, TypeRegistry, syntax};
use jai_modules::{FileInstanceId, ModuleGraph};
use jai_source::{DeclarationId, LocatedDiagnostic};
use jai_types::{ProcedureType, Variadic};
pub(crate) use member_enums::{materialize_static_record, reserve_static_members};
use state::Reservation;
pub(crate) use state::{
    RecordSpecializationKey, RecordSpecializations, RecordTemplateId, SpecializedRecord,
};
use std::collections::{HashMap, HashSet};

/// This conversion preserves syntax identities and leaves type/value ambiguity
/// to declaration lookup. It never evaluates a runtime procedure call as a type.
pub(crate) fn type_expression(expression: &syntax::Expression) -> Option<syntax::TypeSyntax> {
    use syntax::{ExpressionKind as E, TypeSyntax as T};
    Some(match &expression.kind {
        E::Type(ty) => ty.clone(),
        E::Name(name) => T::Named(path(*name)),
        E::QualifiedName(path) => T::Named(path.clone()),
        E::CompileVariable(name) => T::Variable(*name),
        E::TypeQuery {
            query: syntax::TypeQueryKind::TypeOf,
            value,
        } => T::TypeOf(value.clone()),
        E::AddressOf(inner) => T::Pointer(Box::new(type_expression(inner)?)),
        E::Call(name, arguments) => T::Application(syntax::TypeApplicationSyntax {
            base: Box::new(T::Named(path(*name))),
            arguments: arguments.clone(),
            span: expression.span,
        }),
        E::QualifiedCall(path, arguments) => T::Application(syntax::TypeApplicationSyntax {
            base: Box::new(T::Named(path.clone())),
            arguments: arguments.clone(),
            span: expression.span,
        }),
        _ => return None,
    })
}

pub(crate) struct TypeRequest<'a> {
    file: FileInstanceId,
    syntax: &'a syntax::TypeSyntax,
    substitution: Option<&'a Substitution>,
    lexical: Option<&'a LexicalTypeArguments>,
    nominal_context: NominalAnnotationContext,
    span: Span,
}

impl<'a> TypeRequest<'a> {
    pub(crate) fn new(file: FileInstanceId, syntax: &'a syntax::TypeSyntax, span: Span) -> Self {
        Self {
            file,
            syntax,
            substitution: None,
            lexical: None,
            nominal_context: NominalAnnotationContext::None,
            span,
        }
    }

    pub(crate) fn with_substitution(mut self, substitution: Option<&'a Substitution>) -> Self {
        self.substitution = substitution;
        self
    }

    pub(crate) fn with_lexical(mut self, lexical: &'a LexicalTypeArguments) -> Self {
        self.lexical = Some(lexical);
        self
    }

    pub(crate) fn with_enclosing_nominal(mut self, owner: TypeId) -> Self {
        self.nominal_context = NominalAnnotationContext::Record(owner);
        self
    }
}

pub(crate) fn resolve_type(
    graph: &ModuleGraph,
    request: TypeRequest<'_>,
    types: &mut TypeRegistry,
    nominals: &Nominals<'_>,
    records: &mut RecordSpecializations,
    evaluate: &mut impl FnMut(
        FileInstanceId,
        &syntax::Expression,
    ) -> Result<ScalarConstant, LocatedDiagnostic>,
) -> Result<TypeId, LocatedDiagnostic> {
    match prepare_type(graph, request, types, nominals, records, evaluate)? {
        TypePreparation::Ready(ty) => Ok(ty),
        TypePreparation::Pending(pending) => Err(pending.diagnostic(graph)),
    }
}

struct TypeResolver<'a, 'b, F> {
    graph: &'a ModuleGraph,
    types: &'b mut TypeRegistry,
    nominals: &'b Nominals<'a>,
    records: &'b mut RecordSpecializations,
    evaluate: &'b mut F,
    scalar_pending: Option<&'b std::cell::Cell<Option<PendingType>>>,
    aliases: HashSet<DeclarationId>,
    lexical: Option<&'b LexicalTypeArguments>,
    lexical_active: bool,
    nominal_context: NominalAnnotationContext,
}
impl<F> TypeResolver<'_, '_, F>
where
    F: FnMut(FileInstanceId, &syntax::Expression) -> Result<ScalarConstant, LocatedDiagnostic>,
{
    fn evaluate_scalar(
        &mut self,
        file: FileInstanceId,
        expression: &syntax::Expression,
    ) -> TypeResult<ScalarConstant> {
        let result = (self.evaluate)(file, expression);
        if let Some(cause) = self.scalar_pending.and_then(std::cell::Cell::get) {
            return Err(TypeFailure::Pending(cause));
        }
        result.map_err(TypeFailure::from)
    }

    fn in_lexical_scope<R>(&mut self, active: bool, operation: impl FnOnce(&mut Self) -> R) -> R {
        let previous = std::mem::replace(&mut self.lexical_active, active);
        let result = operation(self);
        self.lexical_active = previous;
        result
    }
}

pub(crate) fn baked_scalar(value: &BakedValue) -> Option<ScalarConstant> {
    use jai_ir::ConstantKind;
    match value {
        BakedValue::Value(value) => match value.kind {
            ConstantKind::Int(value) => Some(ScalarConstant::Int(value)),
            ConstantKind::Bool(value) => Some(ScalarConstant::Bool(value)),
            ConstantKind::Float(value) => Some(ScalarConstant::Float(value)),
            _ => None,
        },
        BakedValue::Float(value) => Some(ScalarConstant::Float(*value)),
        _ => None,
    }
}

pub(crate) fn template_origin(
    graph: &ModuleGraph,
    file: FileInstanceId,
    path: &syntax::NamePath,
    span: Span,
) -> Result<DeclarationId, Diagnostic> {
    template_origin_declaration(graph, declaration_id(graph, file, path, span)?, span)
}

pub(crate) fn template_origin_declaration(
    graph: &ModuleGraph,
    mut id: DeclarationId,
    span: Span,
) -> Result<DeclarationId, Diagnostic> {
    let mut visited = HashSet::new();
    loop {
        if !visited.insert(id) {
            return Err(Diagnostic::new(span, "cyclic record template alias"));
        }
        let declaration = graph.declaration(id).expect("resolved declaration exists");
        let target = match &declaration.syntax().kind {
            syntax::FileDeclarationKind::Record(_) => return Ok(id),
            syntax::FileDeclarationKind::TypeAlias(alias) => Some(alias.ty.clone()),
            syntax::FileDeclarationKind::Constant(constant) if constant.ty.is_none() => {
                type_expression(&constant.initializer)
            }
            _ => None,
        };
        let Some(syntax::TypeSyntax::Named(target)) = target else {
            return Err(Diagnostic::new(
                span,
                "type application base does not denote a record template",
            ));
        };
        id = declaration_id(graph, declaration.file(), &target, span)?;
    }
}

pub(crate) struct BoundRecordRequest {
    pub declaration: DeclarationId,
    pub substitution: Substitution,
    pub span: Span,
}

pub(crate) fn instantiate_bound(
    graph: &ModuleGraph,
    request: BoundRecordRequest,
    types: &mut TypeRegistry,
    nominals: &Nominals<'_>,
    records: &mut RecordSpecializations,
    evaluate: &mut impl FnMut(
        FileInstanceId,
        &syntax::Expression,
    ) -> Result<ScalarConstant, LocatedDiagnostic>,
) -> Result<TypeId, LocatedDiagnostic> {
    let BoundRecordRequest {
        declaration,
        substitution,
        span,
    } = request;
    TypeResolver {
        graph,
        types,
        nominals,
        records,
        evaluate,
        scalar_pending: None,
        aliases: HashSet::new(),
        lexical: None,
        lexical_active: false,
        nominal_context: NominalAnnotationContext::None,
    }
    .instantiate(declaration, substitution, span)
    .map_err(|error| error.into_diagnostic(graph))
}
