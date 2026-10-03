//! Reserve nominal identities before resolving their structural dependencies.
use super::super::*;
mod module_parameters;
pub(crate) use module_parameters::ModuleTypeRequest;
mod alias_preparation;
mod enum_namespace;
mod type_queries;
use jai_types::{FieldId, Integer, RecordKind, TypeKind};
use std::collections::HashSet;
use syntax::{BuiltinType, EnumKind, FieldBinding, TypeSyntax, TypeVariantKind};
pub(crate) use type_queries::annotation_query_path;

pub(crate) struct FieldInfo<'a> {
    pub name: Symbol,
    pub id: FieldId,
    pub ty: TypeId,
    pub syntax: &'a syntax::FieldDeclaration,
}
pub(crate) struct RecordInfo<'a> {
    pub declaration: DeclarationId,
    pub file: FileInstanceId,
    pub kind: RecordKind,
    pub fields: Vec<FieldInfo<'a>>,
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct EnumConstant {
    pub ty: TypeId,
    pub value: Integer,
}
pub(crate) struct EnumInfo {
    pub representation: IntegerType,
    pub flags: bool,
    pub members: HashMap<Symbol, Integer>,
    pub values: Vec<Integer>,
}
pub(crate) struct Nominals<'a> {
    annotation_target: std::cell::Cell<Option<jai_types::LayoutPolicy>>,
    pub(crate) procedure_annotations:
        std::cell::RefCell<crate::procedure_values::source_annotations::SourceProcedureAnnotations>,
    context_type: std::cell::Cell<Option<TypeId>>,
    pub(crate) reflection_header: std::cell::Cell<Option<TypeId>>,
    pub(crate) generated_reflection:
        std::cell::OnceCell<std::sync::Arc<crate::reflection::schema::TypeInfoSchema>>,
    pub declarations: HashMap<DeclarationId, TypeId>,
    pub(crate) value_types: HashMap<DeclarationId, TypeId>,
    pub(crate) value_constants: HashMap<DeclarationId, jai_ir::ConstantValue>,
    pub(crate) module_parameter_types: HashMap<jai_modules::ParameterId, TypeId>,
    pub(crate) inserted_capture_types: HashMap<(FileInstanceId, Symbol), TypeId>,
    pub records: HashMap<TypeId, RecordInfo<'a>>,
    pub enums: HashMap<TypeId, EnumInfo>,
    enum_representations: HashMap<TypeId, IntegerType>,
}
#[derive(Clone, Copy)]
struct TypeSite<'a> {
    graph: &'a ModuleGraph,
    file: FileInstanceId,
    span: Span,
}
impl<'a> Nominals<'a> {
    pub(crate) fn set_annotation_target(&self, target: Option<jai_types::LayoutPolicy>) {
        self.annotation_target.set(target);
    }
    pub(crate) fn annotation_target(&self) -> Option<jai_types::LayoutPolicy> {
        self.annotation_target.get()
    }
    pub(crate) fn remember_procedure_annotation(
        &self,
        publication: crate::procedure_values::source_annotations::AnnotationPublication<'_>,
        types: &TypeRegistry,
    ) -> Result<(), Diagnostic> {
        let crate::procedure_values::source_annotations::AnnotationPublication {
            file,
            source,
            substitution,
            parameters,
            ty,
            span,
        } = publication;
        if !source
            .parameters
            .iter()
            .any(|parameter| parameter.evaluation == syntax::ParameterEvaluation::Discard)
        {
            return Ok(());
        }
        use crate::procedure_values::source_annotations::{
            CheckedSourceProcedureType, ProcedureAnnotationKey, ProcedureAnnotationOrigin,
        };
        let proof = CheckedSourceProcedureType::checked(ty, source, parameters, types, span)?;
        let key = ProcedureAnnotationKey::new(
            ProcedureAnnotationOrigin::Graph(file),
            source,
            substitution,
            self.annotation_target(),
        );
        self.procedure_annotations
            .borrow_mut()
            .remember(key, proof, span)
    }
    pub(crate) fn value_type(
        &self,
        graph: &ModuleGraph,
        file: FileInstanceId,
        path: &NamePath,
    ) -> Option<TypeId> {
        let jai_modules::Binding::Declaration(id) = graph.lookup(file, path).ok()? else {
            return None;
        };
        self.value_types.get(&id).copied()
    }
    pub(crate) fn context_type_id(&self) -> Option<TypeId> {
        self.context_type.get()
    }
    pub(crate) fn context_type(&self, types: &mut TypeRegistry) -> TypeId {
        if let Some(ty) = self.context_type.get() {
            return ty;
        }
        let ty = types.reserve_record(RecordKind::Struct);
        self.context_type.set(Some(ty));
        ty
    }
    pub fn reserve(
        graph: &'a ModuleGraph,
        types: &mut TypeRegistry,
    ) -> Result<Self, LocatedDiagnostic> {
        let mut nominals = Self {
            annotation_target: std::cell::Cell::new(None),
            procedure_annotations: std::cell::RefCell::new(Default::default()),
            context_type: std::cell::Cell::new(None),
            reflection_header: std::cell::Cell::new(None),
            generated_reflection: std::cell::OnceCell::new(),
            declarations: HashMap::new(),
            value_types: HashMap::new(),
            value_constants: HashMap::new(),
            module_parameter_types: HashMap::new(),
            inserted_capture_types: HashMap::new(),
            records: HashMap::new(),
            enums: HashMap::new(),
            enum_representations: HashMap::new(),
        };
        // Records are available while aliases used by enum representations resolve.
        for declaration in graph.declarations() {
            if let FileDeclarationKind::Record(record) = &declaration.syntax().kind
                && record.parameters.is_empty()
            {
                nominals
                    .declarations
                    .insert(declaration.id(), types.reserve_record(record.kind));
            }
            if let FileDeclarationKind::TypeAlias(alias) = &declaration.syntax().kind
                && let TypeSyntax::Variant { kind, .. } = &alias.ty
            {
                let kind = match kind {
                    TypeVariantKind::Distinct => jai_types::DistinctKind::Distinct,
                    TypeVariantKind::IsA => jai_types::DistinctKind::IsA,
                };
                nominals
                    .declarations
                    .insert(declaration.id(), types.reserve_distinct(kind));
            }
        }
        for declaration in graph.declarations() {
            let FileDeclarationKind::Enum(enumeration) = &declaration.syntax().kind else {
                continue;
            };
            let representation = match &enumeration.representation {
                None => IntegerType::S64,
                Some(syntax) => {
                    let ty = nominals.resolve_type(
                        graph,
                        declaration.file(),
                        syntax,
                        types,
                        enumeration.span,
                        &mut |file, expression| {
                            Err(located(
                                graph,
                                file,
                                Diagnostic::new(
                                    expression.span,
                                    "enum representation requires an integer type",
                                ),
                            ))
                        },
                    )?;
                    match types.kind(ty).expect("resolved type belongs to registry") {
                        TypeKind::Integer(integer) => *integer,
                        _ => {
                            return Err(located(
                                graph,
                                declaration.file(),
                                Diagnostic::new(
                                    enumeration.span,
                                    "enum representation requires an integer type",
                                ),
                            ));
                        }
                    }
                }
            };
            let ty = types.reserve_enum(representation);
            nominals.declarations.insert(declaration.id(), ty);
            nominals.enum_representations.insert(ty, representation);
        }
        Ok(nominals)
    }
    /// Classify ambiguous `Alias :: Other` by graph identity, without consuming
    /// scalar constants or treating unrelated equal-spelled declarations as types.
    pub fn is_type_alias(&self, graph: &ModuleGraph, id: DeclarationId) -> bool {
        self.alias_target(graph, id, &mut HashSet::new())
    }
    fn alias_target(
        &self,
        graph: &ModuleGraph,
        id: DeclarationId,
        visiting: &mut HashSet<DeclarationId>,
    ) -> bool {
        if !visiting.insert(id) {
            return false;
        }
        let declaration = graph.declaration(id).expect("resolved declaration exists");
        match &declaration.syntax().kind {
            FileDeclarationKind::Record(_)
            | FileDeclarationKind::Enum(_)
            | FileDeclarationKind::TypeAlias(_) => true,
            FileDeclarationKind::Constant(constant) if constant.ty.is_none() => {
                let Some(syntax) = expression_type(&constant.initializer) else {
                    return false;
                };
                let path = match named_leaf(&syntax) {
                    TypeSyntax::Builtin(_)
                    | TypeSyntax::TypeOf(_)
                    | TypeSyntax::Procedure(_)
                    | TypeSyntax::InlineRecord(_)
                    | TypeSyntax::InlineEnum(_)
                    | TypeSyntax::Variant { .. } => return true,
                    TypeSyntax::Named(path) => path.clone(),
                    TypeSyntax::Variable(name) => path(*name),
                    _ => return false,
                };
                if path.members.is_empty()
                    && let Some(capture) =
                        graph.insertion_capture_value(declaration.file(), path.root)
                {
                    return matches!(capture, jai_modules::SourceCaptureValue::Type(_));
                }
                match graph.lookup(declaration.file(), &path) {
                    Ok(jai_modules::Binding::Declaration(target)) => {
                        self.alias_target(graph, target, visiting)
                    }
                    Ok(jai_modules::Binding::Parameter(id)) => {
                        graph.parameter(id).is_some_and(|parameter| {
                            matches!(parameter.value, jai_modules::ParameterValue::Type(_))
                        })
                    }
                    Err(jai_modules::LookupError::UnknownName(_)) if path.members.is_empty() => {
                        BuiltinType::from_spelling(graph.symbols().name(path.root)).is_some()
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }
    /// Populate transparent aliases after their scalar array-count dependencies
    /// are ready. Nominal types keep their reserved identity through aliases.
    #[cfg(test)]
    pub fn define_aliases(
        &mut self,
        graph: &ModuleGraph,
        types: &mut TypeRegistry,
        evaluate: &mut impl FnMut(
            FileInstanceId,
            &syntax::Expression,
        ) -> Result<ConstantValue, LocatedDiagnostic>,
    ) -> Result<(), LocatedDiagnostic> {
        self.define_aliases_with_specializations(
            graph,
            types,
            &mut super::parameterized::RecordSpecializations::default(),
            evaluate,
        )
    }
    #[cfg(test)]
    pub(crate) fn define_aliases_with_specializations(
        &mut self,
        graph: &ModuleGraph,
        types: &mut TypeRegistry,
        records: &mut super::parameterized::RecordSpecializations,
        evaluate: &mut impl FnMut(
            FileInstanceId,
            &syntax::Expression,
        ) -> Result<ConstantValue, LocatedDiagnostic>,
    ) -> Result<(), LocatedDiagnostic> {
        for declaration in graph.declarations() {
            let syntax = match &declaration.syntax().kind {
                FileDeclarationKind::TypeAlias(alias) => alias.ty.clone(),
                FileDeclarationKind::Constant(constant)
                    if self.is_type_alias(graph, declaration.id()) =>
                {
                    expression_type(&constant.initializer)
                        .expect("classified alias has type syntax")
                }
                _ => continue,
            };
            let representation = if let TypeSyntax::Variant { base, .. } = &syntax {
                base.as_ref()
            } else {
                &syntax
            };
            if let TypeSyntax::Named(path) = representation
                && let Ok(id) =
                    declaration_id(graph, declaration.file(), path, declaration.location().span)
                && let FileDeclarationKind::Record(record) =
                    &graph.declaration(id).unwrap().syntax().kind
                && !record.parameters.is_empty()
            {
                continue;
            }
            let resolved = super::parameterized::resolve_type(
                graph,
                super::parameterized::TypeRequest::new(
                    declaration.file(),
                    representation,
                    declaration.location().span,
                )
                .with_substitution(None),
                types,
                self,
                records,
                evaluate,
            )?;
            let ty = if matches!(syntax, TypeSyntax::Variant { .. }) {
                let ty = self.declarations[&declaration.id()];
                types.define_distinct(ty, resolved).map_err(|error| {
                    located(
                        graph,
                        declaration.file(),
                        Diagnostic::new(declaration.location().span, error.to_string()),
                    )
                })?;
                ty
            } else {
                resolved
            };
            self.declarations.insert(declaration.id(), ty);
        }
        Ok(())
    }
    pub fn resolve_type(
        &self,
        graph: &ModuleGraph,
        file: FileInstanceId,
        syntax: &TypeSyntax,
        types: &mut TypeRegistry,
        span: Span,
        evaluate: &mut impl FnMut(
            FileInstanceId,
            &syntax::Expression,
        ) -> Result<ConstantValue, LocatedDiagnostic>,
    ) -> Result<TypeId, LocatedDiagnostic> {
        self.resolve_type_inner(
            TypeSite { graph, file, span },
            syntax,
            types,
            evaluate,
            &mut HashSet::new(),
        )
    }
    pub(crate) fn resolve_type_with_specializations(
        &self,
        graph: &ModuleGraph,
        request: super::parameterized::TypeRequest<'_>,
        types: &mut TypeRegistry,
        records: &mut super::parameterized::RecordSpecializations,
        evaluate: &mut impl FnMut(
            FileInstanceId,
            &syntax::Expression,
        ) -> Result<ConstantValue, LocatedDiagnostic>,
    ) -> Result<TypeId, LocatedDiagnostic> {
        super::parameterized::resolve_type(graph, request, types, self, records, evaluate)
    }
    fn resolve_type_inner(
        &self,
        site: TypeSite<'_>,
        syntax: &TypeSyntax,
        types: &mut TypeRegistry,
        evaluate: &mut impl FnMut(
            FileInstanceId,
            &syntax::Expression,
        ) -> Result<ConstantValue, LocatedDiagnostic>,
        visiting: &mut HashSet<DeclarationId>,
    ) -> Result<TypeId, LocatedDiagnostic> {
        let TypeSite { graph, file, span } = site;
        let result = match syntax {
            TypeSyntax::TypeOf(value) => {
                return self.annotation_type_of(site, value, types, evaluate, visiting);
            }
            TypeSyntax::This => {
                return Err(located(
                    graph,
                    file,
                    Diagnostic::new(
                        span,
                        "#this type requires an enclosing record field annotation",
                    ),
                ));
            }
            TypeSyntax::Builtin(builtin) => Ok(match builtin {
                BuiltinType::Scalar(ty) => types.scalar(*ty),
                BuiltinType::Float(ty) => types.float(*ty),
                BuiltinType::String => types.string(),
                BuiltinType::Void => types.void(),
                BuiltinType::Type => {
                    self.runtime_type_for_graph(graph, types).map_err(|error| {
                        located(graph, file, Diagnostic::new(span, error.to_string()))
                    })?
                }
                BuiltinType::Any => self.reserve_any_for_graph(graph, types).map_err(|error| {
                    located(graph, file, Diagnostic::new(span, error.to_string()))
                })?,
                BuiltinType::Context => self.context_type(types),
            }),
            TypeSyntax::Application(_) => {
                return Err(located(
                    graph,
                    file,
                    Diagnostic::new(
                        span,
                        "record application requires an explicit specialization context",
                    ),
                ));
            }
            TypeSyntax::Restricted { variable, .. } => {
                return self.resolve_type_inner(
                    TypeSite { graph, file, span },
                    &TypeSyntax::Variable(*variable),
                    types,
                    evaluate,
                    visiting,
                );
            }
            TypeSyntax::Variable(name) => {
                if let Some(ty) = self
                    .inserted_capture_type(graph, file, *name, span)
                    .map_err(|error| located(graph, file, error))?
                {
                    return Ok(ty);
                }
                if let Ok(jai_modules::Binding::Parameter(id)) = graph.lookup(
                    file,
                    &NamePath {
                        root: *name,
                        members: vec![],
                    },
                ) && let jai_modules::ParameterValue::Type(value) =
                    &graph.parameter(id).unwrap().value
                {
                    if let Some(ty) = self.module_parameter_types.get(&id) {
                        return Ok(*ty);
                    }
                    return self.resolve_module_type(graph, file, value, types, span);
                }
                return Err(located(
                    graph,
                    file,
                    Diagnostic::new(
                        span,
                        "type variables require specialization before type resolution",
                    ),
                ));
            }
            TypeSyntax::Variant { .. } => {
                return Err(located(
                    graph,
                    file,
                    Diagnostic::new(span, "anonymous type variants require a named type alias"),
                ));
            }
            TypeSyntax::InlineRecord(_) | TypeSyntax::InlineEnum(_) => {
                return Err(located(
                    graph,
                    file,
                    Diagnostic::new(
                        span,
                        "anonymous nominal type requires an explicit specialization context",
                    ),
                ));
            }
            TypeSyntax::Named(path) => {
                if path.members.is_empty()
                    && let Some(ty) = self
                        .inserted_capture_type(graph, file, path.root, span)
                        .map_err(|error| located(graph, file, error))?
                {
                    return Ok(ty);
                }
                if let Ok(jai_modules::Binding::Parameter(id)) = graph.lookup(file, path)
                    && let jai_modules::ParameterValue::Type(value) =
                        &graph.parameter(id).unwrap().value
                {
                    if let Some(ty) = self.module_parameter_types.get(&id) {
                        return Ok(*ty);
                    }
                    return self.resolve_module_type(graph, file, value, types, span);
                }
                let id = match declaration_id(graph, file, path, span) {
                    Ok(id) => id,
                    Err(error) => {
                        if let Some(ty) =
                            self.generated_reflection_type(graph, file, path, types, span)?
                        {
                            return Ok(ty);
                        }
                        return Err(located(graph, file, error));
                    }
                };
                if let Some(&ty) = self.declarations.get(&id) {
                    return Ok(ty);
                }
                let declaration = graph.declaration(id).expect("resolved declaration exists");
                if !visiting.insert(id) {
                    return Err(located(
                        graph,
                        file,
                        Diagnostic::new(span, "cyclic type alias dependencies"),
                    ));
                }
                let alias = match &declaration.syntax().kind {
                    FileDeclarationKind::TypeAlias(alias) => Some(alias.ty.clone()),
                    FileDeclarationKind::Constant(constant) if constant.ty.is_none() => {
                        expression_type(&constant.initializer)
                    }
                    _ => None,
                }
                .ok_or_else(|| {
                    located(
                        graph,
                        file,
                        Diagnostic::new(span, "declaration does not denote a type"),
                    )
                })?;
                let result = self.resolve_type_inner(
                    TypeSite {
                        graph,
                        file: declaration.file(),
                        span: declaration.location().span,
                    },
                    &alias,
                    types,
                    evaluate,
                    visiting,
                );
                visiting.remove(&id);
                return result;
            }
            TypeSyntax::Pointer(inner) => {
                let inner = self.resolve_type_inner(site, inner, types, evaluate, visiting)?;
                types.pointer(inner)
            }
            TypeSyntax::FixedArray { count, element } => {
                let element = self.resolve_type_inner(site, element, types, evaluate, visiting)?;
                let value = evaluate(file, count)?;
                let count = match value {
                    ConstantValue::Literal(value) => u64::try_from(value).ok(),
                    ConstantValue::Int(value) => u64::try_from(value.value()).ok(),
                    ConstantValue::Bool(_)
                    | ConstantValue::Float(_)
                    | ConstantValue::WeakFloat(_) => None,
                }
                .ok_or_else(|| {
                    located(
                        graph,
                        file,
                        Diagnostic::new(
                            count.span,
                            "array count requires a nonnegative integer constant",
                        ),
                    )
                })?;
                types.fixed_array(element, count)
            }
            TypeSyntax::Slice(inner) => {
                let inner = self.resolve_type_inner(site, inner, types, evaluate, visiting)?;
                types.slice(inner)
            }
            TypeSyntax::DynamicArray(inner) => {
                let inner = self.resolve_type_inner(site, inner, types, evaluate, visiting)?;
                types.dynamic_array(inner)
            }
            TypeSyntax::Procedure(procedure) => {
                let mut parameters = Vec::new();
                let mut source_parameters = Vec::new();
                let mut results = Vec::new();
                let mut variadic = jai_types::Variadic::None;
                for parameter in &procedure.parameters {
                    let ty = self.resolve_type_inner(
                        TypeSite {
                            span: parameter.span,
                            ..site
                        },
                        &parameter.ty,
                        types,
                        evaluate,
                        visiting,
                    )?;
                    if parameter.variadic && procedure.convention == CallingConvention::C {
                        if parameter.evaluation == syntax::ParameterEvaluation::Evaluate {
                            variadic = jai_types::Variadic::C {
                                fixed_parameters: parameters.len(),
                            };
                        }
                        continue;
                    }
                    if parameter.variadic {
                        let slice = types.slice(ty).map_err(|error| {
                            located(
                                graph,
                                file,
                                Diagnostic::new(parameter.span, error.to_string()),
                            )
                        })?;
                        source_parameters.push(slice);
                        if parameter.evaluation == syntax::ParameterEvaluation::Evaluate {
                            variadic = jai_types::Variadic::Jai {
                                parameter: parameters.len(),
                                element: ty,
                            };
                            parameters.push(slice);
                        }
                    } else {
                        source_parameters.push(ty);
                        if parameter.evaluation == syntax::ParameterEvaluation::Evaluate {
                            parameters.push(ty);
                        }
                    }
                }
                for result in &procedure.results {
                    results.push(self.resolve_type_inner(
                        TypeSite {
                            span: result.span,
                            ..site
                        },
                        &result.ty,
                        types,
                        evaluate,
                        visiting,
                    )?);
                }
                crate::procedure_values::signatures::normalize_results(&mut results, types, |ty| {
                    *ty
                });
                let ty = types
                    .procedure(ProcedureType {
                        parameters: parameters.into_boxed_slice(),
                        results: results.into_boxed_slice(),
                        convention: procedure.convention,
                        context: procedure.context,
                        variadic,
                    })
                    .map_err(|error| {
                        located(graph, file, Diagnostic::new(span, error.to_string()))
                    })?;
                self.remember_procedure_annotation(
                    crate::procedure_values::source_annotations::AnnotationPublication {
                        file,
                        source: procedure,
                        substitution: None,
                        parameters: source_parameters,
                        ty,
                        span,
                    },
                    types,
                )
                .map_err(|error| located(graph, file, error))?;
                return Ok(ty);
            }
        };
        result.map_err(|error| located(graph, file, Diagnostic::new(span, error.to_string())))
    }
    #[cfg(test)]
    pub fn define_records(
        &mut self,
        graph: &'a ModuleGraph,
        types: &mut TypeRegistry,
        evaluate: &mut impl FnMut(
            FileInstanceId,
            &syntax::Expression,
        ) -> Result<ConstantValue, LocatedDiagnostic>,
    ) -> Result<(), LocatedDiagnostic> {
        self.define_records_with_specializations(
            graph,
            types,
            &mut super::parameterized::RecordSpecializations::default(),
            evaluate,
        )
    }
    pub(crate) fn define_records_with_specializations(
        &mut self,
        graph: &'a ModuleGraph,
        types: &mut TypeRegistry,
        records: &mut super::parameterized::RecordSpecializations,
        evaluate: &mut impl FnMut(
            FileInstanceId,
            &syntax::Expression,
        ) -> Result<ConstantValue, LocatedDiagnostic>,
    ) -> Result<(), LocatedDiagnostic> {
        for declaration in graph.declarations() {
            let FileDeclarationKind::Record(record) = &declaration.syntax().kind else {
                continue;
            };
            if !record.parameters.is_empty() {
                continue;
            }
            let ty = self.declarations[&declaration.id()];
            if records.record(ty).is_some()
                || record
                    .members
                    .iter()
                    .any(|member| !matches!(member, syntax::RecordMember::Field(_)))
                || record.fields().any(|field| {
                    matches!(&field.binding,
                    FieldBinding::Explicit { ty, .. } if contains_inline_nominal(ty))
                        || match &field.binding {
                            FieldBinding::Explicit { initializer, .. } => initializer.as_ref(),
                            FieldBinding::Inferred(initializer) => Some(initializer),
                        }
                        .is_some_and(super::super::field_default_jobs::contains_typed_leaf)
                })
            {
                super::parameterized::materialize_static_record(
                    graph,
                    declaration.id(),
                    types,
                    self,
                    records,
                    evaluate,
                )?;
                let fields = record
                    .fields()
                    .map(|field| {
                        let descriptor = records
                            .record(ty)
                            .expect("materialized source record metadata")
                            .shape
                            .fields
                            .iter()
                            .find(|candidate| candidate.name == Some(field.name))
                            .expect("materialized source field retains its name");
                        FieldInfo {
                            name: field.name,
                            id: descriptor.id,
                            ty: descriptor.ty,
                            syntax: field,
                        }
                    })
                    .collect();
                self.records.insert(
                    ty,
                    RecordInfo {
                        declaration: declaration.id(),
                        file: declaration.file(),
                        kind: record.kind,
                        fields,
                    },
                );
                continue;
            }
            if let Some(modify) = &record.modify {
                return Err(located(
                    graph,
                    declaration.file(),
                    Diagnostic::new(modify.span, "record #modify execution is not implemented"),
                ));
            }
            let file = declaration.file();
            let mut field_types = Vec::new();
            let mut names = HashSet::new();
            let owner = ty;
            for field in record.fields() {
                if record.kind == jai_types::RecordKind::Union
                    && field.conversion == syntax::FieldConversion::Implicit
                {
                    return Err(located(
                        graph,
                        file,
                        Diagnostic::new(
                            field.span,
                            "union #as field conversions are not supported",
                        ),
                    ));
                }
                if !names.insert(field.name) {
                    return Err(located(
                        graph,
                        file,
                        Diagnostic::new(field.span, "duplicate record field"),
                    ));
                }
                let field_ty = match &field.binding {
                    FieldBinding::Explicit { ty, .. } => super::parameterized::resolve_type(
                        graph,
                        super::parameterized::TypeRequest::new(file, ty, field.span)
                            .with_substitution(None)
                            .with_enclosing_nominal(owner),
                        types,
                        self,
                        records,
                        evaluate,
                    )?,
                    FieldBinding::Inferred(expression) => {
                        if let syntax::ExpressionKind::TypeCast { ty, .. } = &expression.kind {
                            field_types.push(super::parameterized::resolve_type(
                                graph,
                                super::parameterized::TypeRequest::new(file, ty, expression.span)
                                    .with_substitution(None)
                                    .with_enclosing_nominal(owner),
                                types,
                                self,
                                records,
                                evaluate,
                            )?);
                            continue;
                        }
                        let literal_type = match &expression.kind {
                            syntax::ExpressionKind::StructLiteral(literal) => literal.ty.as_ref(),
                            syntax::ExpressionKind::PositionalStructLiteral(literal) => {
                                literal.ty.as_ref()
                            }
                            _ => None,
                        };
                        if let Some(path) = literal_type {
                            field_types.push(super::parameterized::resolve_type(
                                graph,
                                super::parameterized::TypeRequest::new(
                                    file,
                                    &TypeSyntax::Named(path.clone()),
                                    field.span,
                                )
                                .with_substitution(None),
                                types,
                                self,
                                records,
                                evaluate,
                            )?);
                            continue;
                        }
                        if matches!(expression.kind, syntax::ExpressionKind::String(_)) {
                            field_types.push(types.string());
                            continue;
                        }
                        if let syntax::ExpressionKind::ArrayLiteral(literal) = &expression.kind
                            && let Some(element) = &literal.element_type
                        {
                            let element = super::parameterized::resolve_type(
                                graph,
                                super::parameterized::TypeRequest::new(file, element, field.span)
                                    .with_substitution(None),
                                types,
                                self,
                                records,
                                evaluate,
                            )?;
                            let count = u64::try_from(literal.elements.len()).map_err(|_| {
                                located(
                                    graph,
                                    file,
                                    Diagnostic::new(
                                        expression.span,
                                        "array literal length exceeds the layout count domain",
                                    ),
                                )
                            })?;
                            field_types.push(types.fixed_array(element, count).map_err(
                                |error| {
                                    located(
                                        graph,
                                        file,
                                        Diagnostic::new(expression.span, error.to_string()),
                                    )
                                },
                            )?);
                            continue;
                        }
                        let enum_ty = match expression_type(expression) {
                            Some(TypeSyntax::Named(path)) => self
                                .enum_member(graph, file, &path, expression.span)
                                .map_err(|error| located(graph, file, error))?
                                .map(|member| member.ty)
                                .or_else(|| self.value_type(graph, file, &path)),
                            _ => None,
                        };
                        match enum_ty {
                            Some(ty) => ty,
                            None => evaluate(file, expression)?.type_id(types),
                        }
                    }
                };
                field_types.push(field_ty);
            }
            let layout = super::layout::record_layout(graph, file, record, evaluate)?;
            types
                .define_record_with_layout(ty, field_types, layout)
                .map_err(|error| {
                    located(graph, file, Diagnostic::new(record.span, error.to_string()))
                })?;
            let mut fields = Vec::new();
            for (index, field) in record.fields().enumerate() {
                let descriptor = types.field(ty, index).map_err(|error| {
                    located(graph, file, Diagnostic::new(field.span, error.to_string()))
                })?;
                fields.push(FieldInfo {
                    name: field.name,
                    id: descriptor.id,
                    ty: descriptor.ty,
                    syntax: field,
                });
            }
            self.records.insert(
                ty,
                RecordInfo {
                    declaration: declaration.id(),
                    file,
                    kind: record.kind,
                    fields,
                },
            );
        }
        records.validate_using(graph, self, types, false)
    }
    pub fn define_enums(
        &mut self,
        graph: &ModuleGraph,
        types: &mut TypeRegistry,
        evaluate: &mut impl FnMut(FileInstanceId, &NamePath, Span) -> Result<ConstantValue, Diagnostic>,
    ) -> Result<(), LocatedDiagnostic> {
        for declaration in graph.declarations() {
            let FileDeclarationKind::Enum(enumeration) = &declaration.syntax().kind else {
                continue;
            };
            let ty = self.declarations[&declaration.id()];
            let representation = self.enum_representations[&ty];
            let mut members = HashMap::new();
            let mut values = Vec::new();
            let mut next = Some(if enumeration.kind == EnumKind::Flags {
                1i128
            } else {
                0i128
            });
            for member in &enumeration.members {
                if members.contains_key(&member.name) {
                    return Err(located(
                        graph,
                        declaration.file(),
                        Diagnostic::new(member.span, "duplicate enum member"),
                    ));
                }
                if enumeration.specified && member.initializer.is_none() {
                    return Err(located(
                        graph,
                        declaration.file(),
                        Diagnostic::new(
                            member.span,
                            "specified enum member requires an explicit value",
                        ),
                    ));
                }
                let value = match &member.initializer {
                    Some(expression) => jai_eval::evaluate_paths(expression, |path, span| {
                        let own_member = if path.members.is_empty() {
                            Some(path.root)
                        } else if path.root == enumeration.name && path.members.len() == 1 {
                            Some(path.members[0])
                        } else {
                            None
                        };
                        if let Some(member) = own_member
                            && let Some(value) = members.get(&member)
                        {
                            return Ok(ConstantValue::Int(*value));
                        }
                        evaluate(declaration.file(), path, span)
                    })
                    .map_err(|error| located(graph, declaration.file(), error))?,
                    None => ConstantValue::Literal(next.ok_or_else(|| {
                        located(
                            graph,
                            declaration.file(),
                            Diagnostic::new(
                                member.span,
                                "enum automatic value overflow or unsupported flags progression",
                            ),
                        )
                    })?),
                }
                .coerce(ScalarType::Int(representation), member.span)
                .map_err(|error| located(graph, declaration.file(), error))?;
                let ConstantValue::Int(value) = value else {
                    unreachable!("enum representation is integer")
                };
                next = match enumeration.kind {
                    EnumKind::Values => value.value().checked_add(1),
                    // Explicit combinations are valid; automatic continuation from
                    // them is deliberately rejected rather than guessed.
                    EnumKind::Flags
                        if value.value() > 0 && (value.value() as u128).is_power_of_two() =>
                    {
                        value.value().checked_mul(2)
                    }
                    EnumKind::Flags => None,
                };
                members.insert(member.name, value);
                values.push(value);
            }
            types.define_enum(ty, values.clone()).map_err(|error| {
                located(
                    graph,
                    declaration.file(),
                    Diagnostic::new(enumeration.span, error.to_string()),
                )
            })?;
            self.enums.insert(
                ty,
                EnumInfo {
                    representation,
                    flags: enumeration.kind == EnumKind::Flags,
                    members,
                    values,
                },
            );
        }
        Ok(())
    }
    pub fn enum_member(
        &self,
        graph: &ModuleGraph,
        file: FileInstanceId,
        path: &NamePath,
        span: Span,
    ) -> Result<Option<EnumConstant>, Diagnostic> {
        let Some((&member, parents)) = path.members.split_last() else {
            return Ok(None);
        };
        let parent = NamePath {
            root: path.root,
            members: parents.to_vec(),
        };
        let Ok(id) = declaration_id(graph, file, &parent, span) else {
            return Ok(None);
        };
        let Some(ty) = self.declarations.get(&id) else {
            return Ok(None);
        };
        let Some(enumeration) = self.enums.get(ty) else {
            return Ok(None);
        };
        let value = enumeration
            .members
            .get(&member)
            .copied()
            .ok_or_else(|| Diagnostic::new(span, "unknown enum member"))?;
        Ok(Some(EnumConstant { ty: *ty, value }))
    }
}

fn expression_type(expression: &syntax::Expression) -> Option<TypeSyntax> {
    super::parameterized::type_expression(expression)
}

fn named_leaf(syntax: &TypeSyntax) -> &TypeSyntax {
    match syntax {
        TypeSyntax::Pointer(inner) | TypeSyntax::Slice(inner) | TypeSyntax::DynamicArray(inner) => {
            named_leaf(inner)
        }
        TypeSyntax::FixedArray { element, .. } => named_leaf(element),
        TypeSyntax::Application(application) => named_leaf(&application.base),
        _ => syntax,
    }
}
fn contains_inline_nominal(syntax: &TypeSyntax) -> bool {
    match syntax {
        TypeSyntax::InlineRecord(_) | TypeSyntax::InlineEnum(_) => true,
        TypeSyntax::Pointer(inner) | TypeSyntax::Slice(inner) | TypeSyntax::DynamicArray(inner) => {
            contains_inline_nominal(inner)
        }
        TypeSyntax::FixedArray { element, .. } => contains_inline_nominal(element),
        TypeSyntax::Procedure(procedure) => {
            procedure
                .parameters
                .iter()
                .any(|parameter| contains_inline_nominal(&parameter.ty))
                || procedure
                    .results
                    .iter()
                    .any(|result| contains_inline_nominal(&result.ty))
        }
        _ => false,
    }
}
#[cfg(test)]
#[path = "types/tests.rs"]
mod tests;
