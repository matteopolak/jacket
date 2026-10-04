//! Adapt graph lookup into semantic declaration metadata.
use super::*;
use jai_modules::{Binding as GraphBinding, LookupError};
mod compiler_code;
mod imports;
mod operators;
mod storage_members;
#[derive(Clone, Copy)]
pub(crate) struct FileScope<'a> {
    pub(super) declarations: &'a ScopedDeclarations<'a>,
    pub(super) file: FileInstanceId,
    pub(crate) substitution: Option<&'a crate::polymorphism::Substitution>,
}
impl<'a> FileScope<'a> {
    pub(crate) fn type_value_name_absent(&self, path: &NamePath) -> bool {
        if !path.members.is_empty()
            || self.substitution.is_some_and(|bindings| {
                bindings.ty(path.root).is_some() || bindings.constant(path.root).is_some()
            })
            || self
                .declarations
                .graph
                .insertion_capture_value(self.file, path.root)
                .is_some()
        {
            return false;
        }
        matches!(
            self.declarations.graph.lookup(self.file, path),
            Err(LookupError::UnknownName(_))
        )
    }

    pub(crate) fn inserted_capture_type(
        &self,
        name: Symbol,
        span: Span,
    ) -> Result<Option<TypeId>, Diagnostic> {
        self.declarations.nominals.inserted_capture_type(
            self.declarations.graph,
            self.file,
            name,
            span,
        )
    }
    pub(crate) fn module_type_parameter(
        &self,
        name: Symbol,
        span: Span,
    ) -> Result<Option<TypeId>, Diagnostic> {
        if let Some(ty) = self.inserted_capture_type(name, span)? {
            return Ok(Some(ty));
        }
        let path = NamePath {
            root: name,
            members: Vec::new(),
        };
        let Ok(GraphBinding::Parameter(id)) = self.declarations.graph.lookup(self.file, &path)
        else {
            return Ok(None);
        };
        if !matches!(
            self.declarations
                .graph
                .parameter(id)
                .map(|parameter| &parameter.value),
            Some(jai_modules::ParameterValue::Type(_))
        ) {
            return Err(Diagnostic::new(
                span,
                "module parameter does not denote a type",
            ));
        }
        self.declarations
            .nominals
            .module_parameter_types
            .get(&id)
            .copied()
            .map(Some)
            .ok_or_else(|| {
                Diagnostic::new(
                    span,
                    "module type parameter requires canonical semantic materialization",
                )
            })
    }
    pub(crate) fn annotation_target(&self) -> Option<jai_types::LayoutPolicy> {
        self.declarations.nominals.annotation_target()
    }
    pub(crate) fn checked_procedure_annotation(
        &self,
        key: &crate::procedure_values::source_annotations::ProcedureAnnotationKey,
    ) -> Option<
        std::sync::Arc<crate::procedure_values::source_annotations::CheckedSourceProcedureType>,
    > {
        self.declarations
            .nominals
            .procedure_annotations
            .borrow()
            .get(key)
    }
    pub(crate) fn has_inline_body(&self, procedure: ProcedureId) -> Option<bool> {
        let mut prototype_body = None;
        for (id, signature) in &self.declarations.signatures {
            if signature.id != procedure {
                continue;
            }
            match &self.declarations.graph.declaration(*id)?.syntax().kind {
                FileDeclarationKind::Procedure(_) => return Some(true),
                FileDeclarationKind::ProcedurePrototype(prototype) => {
                    prototype_body = Some(matches!(
                        prototype.binding,
                        syntax::PrototypeBinding::Intrinsic { .. }
                    ))
                }
                _ => {}
            }
        }
        if prototype_body.is_some() {
            return prototype_body;
        }
        let declaration = self
            .declarations
            .generics
            .borrow()
            .source_declaration(procedure)?;
        match &self
            .declarations
            .graph
            .declaration(declaration)?
            .syntax()
            .kind
        {
            FileDeclarationKind::Procedure(_) => Some(true),
            FileDeclarationKind::ProcedurePrototype(prototype) => Some(matches!(
                prototype.binding,
                syntax::PrototypeBinding::Intrinsic { .. }
            )),
            _ => None,
        }
    }
    pub(crate) fn declared_inline_hint(
        &self,
        procedure: ProcedureId,
    ) -> Option<jai_types::InlineHint> {
        if let Some(hint) = self
            .declarations
            .signatures
            .iter()
            .find_map(|(id, signature)| {
                if signature.id != procedure {
                    return None;
                }
                let FileDeclarationKind::Procedure(source) =
                    &self.declarations.graph.declaration(*id)?.syntax().kind
                else {
                    return None;
                };
                Some(source.inline_hint)
            })
        {
            return Some(hint);
        }
        let declaration = self
            .declarations
            .generics
            .borrow()
            .source_declaration(procedure)?;
        let FileDeclarationKind::Procedure(source) = &self
            .declarations
            .graph
            .declaration(declaration)?
            .syntax()
            .kind
        else {
            return None;
        };
        Some(source.inline_hint)
    }
    /// Exact single-return source summary, available before callee body checking.
    pub(crate) fn forwarding_parameter(&self, procedure: ProcedureId) -> Option<usize> {
        let (id, signature) = self
            .declarations
            .signatures
            .iter()
            .find(|(_, signature)| signature.id == procedure)?;
        let source = self.declarations.graph.declaration(*id)?;
        let FileDeclarationKind::Procedure(body) = &source.syntax().kind else {
            return None;
        };
        let [statement] = body.body.as_slice() else {
            return None;
        };
        let syntax::StatementKind::Return(Some(value)) = &statement.kind else {
            return None;
        };
        let name = match &value.kind {
            syntax::ExpressionKind::Name(name) => *name,
            syntax::ExpressionKind::QualifiedName(path)
                if path.members.len() == 1
                    && self.declarations.graph.symbols().name(path.members[0]) == "data" =>
            {
                path.root
            }
            _ => return None,
        };
        // Match defining-file formal identities, never a use-site spelling.
        let index = body.parameters.iter().position(|parameter| {
            parameter.baking == syntax::ParameterBaking::None && parameter.name == name
        })?;
        if body
            .parameters
            .iter()
            .any(|parameter| parameter.baking != syntax::ParameterBaking::None)
            || signature.parameters.len() != body.parameters.len()
        {
            return None;
        }
        let formal = signature.parameters.get(index)?;
        if formal.evaluation == syntax::ParameterEvaluation::Discard {
            return None;
        }
        Some(
            signature.parameters[..index]
                .iter()
                .filter(|parameter| parameter.evaluation == syntax::ParameterEvaluation::Evaluate)
                .count(),
        )
    }

    pub(crate) fn program_entry(&self, span: Span) -> Result<Signature, Diagnostic> {
        super::program_exports::application_signature(
            self.declarations.graph,
            self.declarations,
            span,
        )
    }
    pub(crate) fn record_template_origin(
        &self,
        path: &NamePath,
        span: Span,
    ) -> Result<DeclarationId, Diagnostic> {
        aggregates::parameterized::template_origin(self.declarations.graph, self.file, path, span)
    }
    pub(crate) fn annotation_with_specializations(
        &self,
        syntax: &syntax::TypeSyntax,
        types: &mut TypeRegistry,
        records: &mut aggregates::parameterized::RecordSpecializations,
        span: Span,
    ) -> Result<TypeId, Diagnostic> {
        aggregates::parameterized::resolve_type(
            self.declarations.graph,
            aggregates::parameterized::TypeRequest::new(self.file, syntax, span)
                .with_substitution(self.substitution),
            types,
            &self.declarations.nominals,
            records,
            &mut |file, expression| {
                jai_eval::evaluate_paths(expression, |path, span| {
                    let scope = FileScope {
                        file,
                        substitution: None,
                        ..*self
                    };
                    match scope.value(path, span)? {
                        Binding::Constant(value) => Ok(value),
                        Binding::Enum(value) => Ok(ConstantValue::Int(value.value)),
                        _ => Err(Diagnostic::new(
                            span,
                            "record template value requires a compile-time scalar constant",
                        )),
                    }
                })
                .map_err(|error| located(self.declarations.graph, file, error))
            },
        )
        .map_err(|error| Diagnostic::at_source(error.location, error.message))
    }

    pub(crate) fn annotation_with_lexical_bindings(
        &self,
        syntax: &syntax::TypeSyntax,
        types: &mut TypeRegistry,
        records: &mut aggregates::parameterized::RecordSpecializations,
        lexical: &aggregates::parameterized::LexicalTypeArguments,
        span: Span,
    ) -> Result<TypeId, Diagnostic> {
        aggregates::parameterized::resolve_type(
            self.declarations.graph,
            aggregates::parameterized::TypeRequest::new(self.file, syntax, span)
                .with_substitution(self.substitution)
                .with_lexical(lexical),
            types,
            &self.declarations.nominals,
            records,
            &mut |file, expression| {
                jai_eval::evaluate_paths(expression, |path, span| {
                    let scope = FileScope {
                        file,
                        substitution: None,
                        ..*self
                    };
                    match scope.value(path, span)? {
                        Binding::Constant(value) => Ok(value),
                        Binding::Enum(value) => Ok(ConstantValue::Int(value.value)),
                        _ => Err(Diagnostic::new(
                            span,
                            "record template value requires a compile-time scalar constant",
                        )),
                    }
                })
                .map_err(|error| located(self.declarations.graph, file, error))
            },
        )
        .map_err(|error| Diagnostic::at_source(error.location, error.message))
    }

    pub(crate) fn specialized_field_default(
        &self,
        id: FieldId,
        types: &TypeRegistry,
        records: &aggregates::parameterized::RecordSpecializations,
        span: Span,
    ) -> Result<jai_ir::ConstantValue, Diagnostic> {
        let mut evaluate = |file, expression: &syntax::Expression| {
            jai_eval::evaluate_paths(expression, |path, span| {
                let scope = FileScope {
                    file,
                    substitution: None,
                    ..*self
                };
                match scope.value(path, span)? {
                    Binding::Constant(value) => Ok(value),
                    Binding::Enum(value) => Ok(ConstantValue::Int(value.value)),
                    _ => Err(Diagnostic::new(
                        span,
                        "record field default requires an immutable constant",
                    )),
                }
            })
            .map_err(|error| located(self.declarations.graph, file, error))
        };
        let mut defaults = aggregates::Defaults::with_evaluator(
            self.declarations.graph,
            types,
            &self.declarations.nominals,
            &mut evaluate,
        )
        .with_specializations(records);
        defaults
            .field_default(id)
            .map_err(|error| Diagnostic::new(span, error.message))
    }
    pub(crate) fn source_path(&self) -> &'a std::path::Path {
        self.declarations
            .graph
            .sources()
            .get(self.source())
            .unwrap()
            .path()
    }
    pub(crate) fn source_path_for(
        &self,
        source: jai_source::SourceId,
    ) -> Option<&'a std::path::Path> {
        self.declarations
            .graph
            .sources()
            .get(source)
            .map(|source| source.path())
    }
    pub(crate) fn source_record(
        &self,
        source: jai_source::SourceId,
    ) -> Option<&'a jai_source::SourceRecord> {
        self.declarations.graph.sources().get(source)
    }
    pub(crate) fn caller_location_type(
        &self,
        types: &TypeRegistry,
        span: Span,
    ) -> Result<TypeId, Diagnostic> {
        crate::caller_locations::infer_target(
            self.declarations.graph,
            self.file,
            &self.declarations.nominals,
            types,
            span,
        )
    }
    pub(crate) fn validate_caller_location_type(
        &self,
        ty: TypeId,
        types: &TypeRegistry,
        span: Span,
    ) -> Result<(), Diagnostic> {
        crate::caller_locations::validate_target(
            self.declarations.graph,
            &self.declarations.nominals,
            types,
            ty,
            span,
        )
    }
    pub(crate) fn foreign_library(
        &self,
        path: &NamePath,
        span: Span,
    ) -> Result<jai_ir::ForeignLibrary, Diagnostic> {
        let id = self.declaration(path, span)?;
        super::foreign_libraries::declaration(
            self.declarations.graph,
            self.declarations.graph.declaration(id).unwrap(),
        )
        .map_err(|error| Diagnostic::new(span, error.to_string()))
    }
    pub(crate) fn source(&self) -> jai_source::SourceId {
        self.declarations.graph.file(self.file).unwrap().source()
    }

    pub(crate) fn enum_flags(&self, ty: TypeId) -> bool {
        self.declarations
            .nominals
            .enums
            .get(&ty)
            .is_some_and(|info| info.flags)
    }
    pub(crate) fn enum_member_value(&self, ty: TypeId, name: Symbol) -> Option<IntegerValue> {
        self.declarations
            .nominals
            .enums
            .get(&ty)?
            .members
            .get(&name)
            .copied()
    }
    pub(crate) fn type_name(&self, path: &NamePath, span: Span) -> Result<TypeId, Diagnostic> {
        if path.members.is_empty()
            && let Some(ty) = self
                .substitution
                .and_then(|substitution| substitution.ty(path.root))
        {
            return Ok(ty);
        }
        if let Ok(GraphBinding::Parameter(id)) = self.declarations.graph.lookup(self.file, path) {
            if !matches!(
                self.declarations
                    .graph
                    .parameter(id)
                    .map(|parameter| &parameter.value),
                Some(jai_modules::ParameterValue::Type(_))
            ) {
                return Err(Diagnostic::new(
                    span,
                    "module parameter does not denote a type",
                ));
            }
            return self.declarations.nominals.module_parameter_types.get(&id).copied()
                .ok_or_else(|| Diagnostic::new(span, "module type parameter is not materialized in the canonical semantic session"));
        }
        let id = self.declaration(path, span)?;
        self.declarations
            .nominals
            .declarations
            .get(&id)
            .copied()
            .ok_or_else(|| Diagnostic::new(span, "declaration does not denote a type"))
    }
    pub(crate) fn annotation(
        &self,
        syntax: &syntax::TypeSyntax,
        types: &mut TypeRegistry,
        span: Span,
    ) -> Result<TypeId, Diagnostic> {
        if let Some(substitution) = self.substitution {
            let pattern = crate::polymorphism::substituted_pattern(
                syntax,
                substitution,
                span,
                |syntax, span| {
                    FileScope {
                        substitution: None,
                        ..*self
                    }
                    .annotation(syntax, types, span)
                },
                |expression| {
                    let value = jai_eval::evaluate_paths(expression, |path, span| {
                        match self.value(path, span)? {
                            Binding::Constant(value) => Ok(value),
                            _ => Err(Diagnostic::new(
                                span,
                                "type count requires a compile-time integer",
                            )),
                        }
                    })?;
                    let integer = match value {
                        ConstantValue::Literal(value) => value,
                        ConstantValue::Int(value) => value.value(),
                        _ => {
                            return Err(Diagnostic::new(
                                expression.span,
                                "type count requires an integer",
                            ));
                        }
                    };
                    u64::try_from(integer).map_err(|_| {
                        Diagnostic::new(expression.span, "array count is out of range")
                    })
                },
            )?;
            return crate::polymorphism::materialize(types, &pattern, substitution).map_err(
                |error| Diagnostic::new(span, format!("invalid substituted type: {error:?}")),
            );
        }
        self.declarations
            .nominals
            .resolve_type(
                self.declarations.graph,
                self.file,
                syntax,
                types,
                span,
                &mut |file, expression| {
                    jai_eval::evaluate_paths(expression, |path, span| {
                        let scope = FileScope {
                            declarations: self.declarations,
                            file,
                            substitution: None,
                        };
                        match scope.value(path, span)? {
                            Binding::Constant(value) => Ok(value),
                            _ => Err(Diagnostic::new(
                                span,
                                "type count requires a scalar compile-time constant",
                            )),
                        }
                    })
                    .map_err(|error| located(self.declarations.graph, file, error))
                },
            )
            .map_err(|error| Diagnostic::at_source(error.location, error.message))
    }
    pub(crate) fn record(
        &self,
        ty: TypeId,
        span: Span,
    ) -> Result<&'a aggregates::types::RecordInfo<'a>, Diagnostic> {
        self.declarations
            .nominals
            .records
            .get(&ty)
            .ok_or_else(|| Diagnostic::new(span, "value is not a record"))
    }
    pub(crate) fn field_default(
        &self,
        id: FieldId,
        types: &TypeRegistry,
        span: Span,
    ) -> Result<jai_ir::ConstantValue, Diagnostic> {
        if let Some(value) = self.declarations.defaults.get(&id) {
            return Ok(value.clone());
        }
        let ty = types
            .field_type(id)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?;
        self.default_value(ty, types, span)
    }
    pub(crate) fn default_value(
        &self,
        ty: TypeId,
        types: &TypeRegistry,
        span: Span,
    ) -> Result<jai_ir::ConstantValue, Diagnostic> {
        if let Some(schema) = self
            .declarations
            .context
            .as_ref()
            .filter(|schema| schema.definition.record_type == ty)
        {
            return Ok(schema.definition.default.clone());
        }
        let mut remaining = crate::constant_limits::MAX_CONSTANT_CELLS;
        self.default_value_inner(
            ty,
            types,
            span,
            &mut std::collections::HashSet::new(),
            &mut remaining,
        )
    }
    fn default_value_inner(
        &self,
        ty: TypeId,
        types: &TypeRegistry,
        span: Span,
        active: &mut std::collections::HashSet<TypeId>,
        remaining: &mut usize,
    ) -> Result<jai_ir::ConstantValue, Diagnostic> {
        crate::record_placements::require_record_storage_ready(types, ty, span)?;
        if active.len() >= crate::constant_limits::MAX_CONSTANT_DEPTH {
            return Err(Diagnostic::new(
                span,
                "constant exceeds compiler constant depth budget",
            ));
        }
        *remaining = remaining.checked_sub(1).ok_or_else(|| {
            Diagnostic::new(span, "constant exceeds compiler constant cell budget")
        })?;
        if !active.insert(ty) {
            return Err(Diagnostic::new(
                span,
                "cyclic value type cannot be default initialized",
            ));
        }
        let kind = match types
            .kind(ty)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?
        {
            TypeKind::Integer(integer) => {
                jai_ir::ConstantKind::Int(IntegerValue::wrapping(*integer, 0))
            }
            TypeKind::Bool => jai_ir::ConstantKind::Bool(false),
            TypeKind::Float(float) => jai_ir::ConstantKind::Float(match float {
                jai_types::FloatType::F32 => jai_types::FloatValue::F32(0),
                jai_types::FloatType::F64 => jai_types::FloatValue::F64(0),
            }),
            TypeKind::String => jai_ir::ConstantKind::StringBytes(Vec::new()),
            TypeKind::Distinct(_) => {
                let representation = types
                    .distinct_definition(ty)
                    .map_err(|error| Diagnostic::new(span, error.to_string()))?
                    .representation;
                jai_ir::ConstantKind::Distinct(Box::new(self.default_value_inner(
                    representation,
                    types,
                    span,
                    active,
                    remaining,
                )?))
            }
            TypeKind::FixedArray {
                element,
                count,
            } => {
                let count = usize::try_from(*count).map_err(|_| {
                    Diagnostic::new(span, "array default count exceeds host addressable limits")
                })?;
                if count == 0 {
                    jai_ir::ConstantKind::Zero
                } else {
                    let initial =
                        self.default_value_inner(*element, types, span, active, remaining)?;
                    if crate::constant_limits::is_zero(&initial) {
                        jai_ir::ConstantKind::Zero
                    } else {
                        let cells = crate::constant_limits::cells(&initial)
                            .and_then(|cells| cells.checked_mul(count.saturating_sub(1)))
                            .ok_or_else(|| {
                                Diagnostic::new(
                                    span,
                                    "array default exceeds compiler constant cell budget",
                                )
                            })?;
                        *remaining = remaining.checked_sub(cells).ok_or_else(|| {
                            Diagnostic::new(
                                span,
                                "array default exceeds compiler constant cell budget",
                            )
                        })?;
                        jai_ir::ConstantKind::Array(vec![initial; count])
                    }
                }
            }
            TypeKind::Enum(_) => {
                let enumeration = &self.declarations.nominals.enums[&ty];
                if !enumeration.flags
                    && enumeration
                        .values
                        .first()
                        .is_none_or(|value| value.value() != 0)
                {
                    return Err(Diagnostic::new(
                        span,
                        "enum default initialization requires an explicit value until its language rule is established",
                    ));
                }
                jai_ir::ConstantKind::Enum(IntegerValue::wrapping(enumeration.representation, 0))
            }
            TypeKind::Record(_) => {
                let record = self.record(ty, span)?;
                if record.kind != jai_types::RecordKind::Struct {
                    return Err(Diagnostic::new(
                        span,
                        "union default initialization is not implemented",
                    ));
                }
                let mut fields = Vec::new();
                for field in &record.fields {
                    fields.push(match self.declarations.defaults.get(&field.id) {
                        Some(value) => {
                            let cells = crate::constant_limits::cells(value).ok_or_else(|| {
                                Diagnostic::new(
                                    span,
                                    "constant exceeds compiler constant cell budget",
                                )
                            })?;
                            *remaining = remaining.checked_sub(cells).ok_or_else(|| {
                                Diagnostic::new(
                                    span,
                                    "constant exceeds compiler constant cell budget",
                                )
                            })?;
                            value.clone()
                        }
                        None => {
                            self.default_value_inner(field.ty, types, span, active, remaining)?
                        }
                    });
                }
                jai_ir::ConstantKind::Record(fields)
            }
            TypeKind::Type
            | TypeKind::Pointer(_)
            | TypeKind::Slice(_)
            | TypeKind::DynamicArray(_)
            | TypeKind::Procedure(_) => jai_ir::ConstantKind::Zero,
            _ => {
                return Err(Diagnostic::new(
                    span,
                    "default initialization is not implemented for this type",
                ));
            }
        };
        active.remove(&ty);
        Ok(jai_ir::ConstantValue {
            ty,
            kind,
        })
    }
    pub(crate) fn parameter_string(&self, path: &NamePath) -> Option<(Vec<u8>, Vec<Symbol>)> {
        for count in (0..=path.members.len()).rev() {
            let prefix = NamePath {
                root: path.root,
                members: path.members[..count].to_vec(),
            };
            if let Ok(GraphBinding::Parameter(id)) =
                self.declarations.graph.lookup(self.file, &prefix)
                && let jai_modules::ParameterValue::String(bytes) =
                    &self.declarations.graph.parameter(id)?.value
            {
                return Some((bytes.as_bytes().to_vec(), path.members[count..].to_vec()));
            }
        }
        None
    }
    pub(crate) fn value_root(
        &self,
        path: &NamePath,
        span: Span,
    ) -> Result<(Binding, Vec<Symbol>), Diagnostic> {
        if let Ok(binding) = self.value(path, span) {
            return Ok((binding, Vec::new()));
        }
        let original = self.value(path, span).unwrap_err();
        for count in (0..path.members.len()).rev() {
            let prefix = NamePath {
                root: path.root,
                members: path.members[..count].to_vec(),
            };
            if let Ok(binding) = self.value(&prefix, span) {
                return Ok((binding, path.members[count..].to_vec()));
            }
        }
        Err(original)
    }

    fn declaration(&self, path: &NamePath, span: Span) -> Result<DeclarationId, Diagnostic> {
        let lookup = self.declarations.graph.lookup(self.file, path);
        let marker = lookup.as_ref().err().and_then(|error| {
            self.declarations.source_lookup.record(
                self.declarations.graph,
                self.file,
                path,
                span,
                *error,
            )
        });
        declaration_lookup(self.declarations.graph, span, lookup).map_err(|error| match marker {
            Some(marker) => error.with_marker(marker),
            None => error,
        })
    }
    pub(crate) fn value(&self, path: &NamePath, span: Span) -> Result<Binding, Diagnostic> {
        if let Some(value) = self.target_value(path, span)? {
            return Ok(value);
        }
        if let Some(value) = self.baked_value(path) {
            use crate::polymorphism::BakedValue;
            return match value {
                BakedValue::Code(id) => Ok(Binding::Code(id)),
                BakedValue::Value(jai_ir::ConstantValue {
                    kind: jai_ir::ConstantKind::Int(value),
                    ..
                }) => Ok(Binding::Constant(ConstantValue::Int(value))),
                BakedValue::Value(jai_ir::ConstantValue {
                    kind: jai_ir::ConstantKind::Bool(value),
                    ..
                }) => Ok(Binding::Constant(ConstantValue::Bool(value))),
                BakedValue::Value(jai_ir::ConstantValue {
                    ty,
                    kind: jai_ir::ConstantKind::Enum(value),
                }) => Ok(Binding::Enum(aggregates::EnumConstant {
                    ty,
                    value,
                })),
                BakedValue::Value(jai_ir::ConstantValue {
                    ty,
                    kind: jai_ir::ConstantKind::Procedure(procedure),
                }) => Ok(Binding::Procedure {
                    procedure,
                    ty,
                }),
                BakedValue::Float(value)
                | BakedValue::Value(jai_ir::ConstantValue {
                    kind: jai_ir::ConstantKind::Float(value),
                    ..
                }) => Ok(Binding::Constant(ConstantValue::Float(value))),
                _ => Err(Diagnostic::new(
                    span,
                    "baked aggregate/type value requires typed constant lowering",
                )),
            };
        }
        if let Ok(
            binding @ (GraphBinding::SourceMember {
                ..
            }
            | GraphBinding::StorageMember(_)),
        ) = self.declarations.graph.lookup(self.file, path)
        {
            return self.imported_value(binding, span);
        }
        if let Ok(GraphBinding::Parameter(id)) = self.declarations.graph.lookup(self.file, path) {
            return match &self.declarations.graph.parameter(id).unwrap().value {
                jai_modules::ParameterValue::Scalar(value) => Ok(Binding::Constant(value.clone())),
                jai_modules::ParameterValue::Type(_) => self
                    .declarations
                    .nominals
                    .module_parameter_types
                    .get(&id)
                    .copied()
                    .map(Binding::Type)
                    .ok_or_else(|| {
                        Diagnostic::new(
                            span,
                            "module type parameter has no canonical type in this semantic session",
                        )
                    }),
                jai_modules::ParameterValue::Enumeration(value) => {
                    let ty = self
                        .declarations
                        .nominals
                        .declarations
                        .get(&value.declaration)
                        .copied()
                        .ok_or_else(|| {
                            Diagnostic::new(
                                span,
                                "module enum parameter has no resolved nominal declaration",
                            )
                        })?;
                    Ok(Binding::Enum(aggregates::EnumConstant {
                        ty,
                        value: value.value,
                    }))
                }
                jai_modules::ParameterValue::ContextualMember(_) => Err(Diagnostic::new(
                    span,
                    "module parameter contains an unbound contextual member",
                )),
                jai_modules::ParameterValue::String(_) => Err(Diagnostic::new(
                    span,
                    "string module parameter requires string lowering",
                )),
            };
        }
        if let Some(member) = self.declarations.nominals.enum_member(
            self.declarations.graph,
            self.file,
            path,
            span,
        )? {
            return Ok(Binding::Enum(member));
        }
        let id = self.declaration(path, span)?;
        self.declarations
            .values
            .get(&id)
            .cloned()
            .ok_or_else(|| Diagnostic::new(span, "procedure cannot supply a scalar value"))
    }
    pub(crate) fn signature(
        &self,
        path: &NamePath,
        span: Span,
    ) -> Result<&'a Signature, Diagnostic> {
        let declarations = self.callable_declarations(path, span)?;
        let [id] = declarations.as_slice() else {
            return Err(Diagnostic::new(
                span,
                "procedure overload requires argument matching",
            ));
        };
        self.declarations
            .signatures
            .get(id)
            .ok_or_else(|| Diagnostic::new(span, "scalar value is not a procedure"))
    }
    pub(crate) fn baked_value(&self, path: &NamePath) -> Option<crate::polymorphism::BakedValue> {
        if path.members.is_empty()
            && let Some(value) = self
                .substitution
                .and_then(|substitution| substitution.constant(path.root))
        {
            return Some(value.clone());
        }
        if let Ok(GraphBinding::Parameter(id)) = self.declarations.graph.lookup(self.file, path)
            && let jai_modules::ParameterValue::String(value) =
                &self.declarations.graph.parameter(id)?.value
        {
            return Some(crate::polymorphism::BakedValue::String(
                value.as_bytes().into(),
            ));
        }
        None
    }
    pub(crate) fn callable_declarations(
        &self,
        path: &NamePath,
        span: Span,
    ) -> Result<Vec<DeclarationId>, Diagnostic> {
        match self.declarations.graph.lookup(self.file, path) {
            Ok(GraphBinding::OverloadSet(id)) => self.checked_callable_members(
                self.declarations
                    .graph
                    .overload_set(id)
                    .expect("graph overload exists")
                    .declarations(),
                span,
            ),
            Ok(GraphBinding::Declaration(id)) => {
                if let Some(targets) = self.declarations.callable_aliases.get(&id) {
                    return Ok(targets.clone());
                }
                if self.declarations.signatures.contains_key(&id)
                    || self.declarations.generics.borrow().is_template(id)
                {
                    Ok(vec![id])
                } else {
                    Err(Diagnostic::new(span, "declaration is not a procedure"))
                }
            }
            _ => self
                .declaration(path, span)
                .and_then(|_| Err(Diagnostic::new(span, "declaration is not a procedure"))),
        }
    }
    fn checked_callable_members(
        &self,
        members: &[DeclarationId],
        span: Span,
    ) -> Result<Vec<DeclarationId>, Diagnostic> {
        let mut targets = Vec::new();
        for &member in members {
            let resolved = self
                .declarations
                .callable_aliases
                .get(&member)
                .map_or(std::slice::from_ref(&member), Vec::as_slice);
            for &target in resolved {
                if !self.declarations.signatures.contains_key(&target)
                    && !self.declarations.generics.borrow().is_template(target)
                {
                    return Err(Diagnostic::new(
                        span,
                        "callable overload member is pending its checked procedure declaration",
                    ));
                }
                targets.push(target);
            }
        }
        targets.sort_unstable_by_key(|id| id.index());
        targets.dedup();
        Ok(targets)
    }
    pub(crate) fn candidate(
        &self,
        id: DeclarationId,
        types: &TypeRegistry,
        span: Span,
    ) -> Result<crate::overloads::Candidate, Diagnostic> {
        if let Some(signature) = self.declarations.signatures.get(&id) {
            return Ok(crate::polymorphism::integration::concrete_candidate(
                id, signature, types,
            ));
        }
        self.declarations
            .generics
            .borrow()
            .candidate(id)
            .ok_or_else(|| Diagnostic::new(span, "unknown procedure declaration"))
    }
    pub(crate) fn concrete_signature(&self, id: DeclarationId) -> Option<&'a Signature> {
        self.declarations.signatures.get(&id)
    }
    pub(crate) fn has_source_name(&self, path: &NamePath) -> bool {
        (path.members.is_empty()
            && self.substitution.is_some_and(|bindings| {
                bindings.ty(path.root).is_some() || bindings.constant(path.root).is_some()
            }))
            || self.declarations.graph.lookup(self.file, path).is_ok()
    }
    pub(crate) fn modifier_source(&self, id: DeclarationId) -> Option<&'a syntax::Procedure> {
        let FileDeclarationKind::Procedure(source) =
            &self.declarations.graph.declaration(id)?.syntax().kind
        else {
            return None;
        };
        source.modify.as_ref()?;
        Some(source)
    }
    pub(crate) fn generic_calling_mode(
        &self,
        id: DeclarationId,
    ) -> Option<(
        jai_types::CallingConvention,
        jai_types::ForeignReturnAbi,
        jai_types::ContextMode,
    )> {
        self.declarations.generics.borrow().calling_mode(id)
    }
    pub(crate) fn expanded_formal_pattern(
        &self,
        syntax: &syntax::TypeSyntax,
        types: &mut TypeRegistry,
        records: &mut aggregates::parameterized::RecordSpecializations,
        substitution: Option<&crate::polymorphism::Substitution>,
        span: Span,
    ) -> Result<crate::overloads::TypePattern, Diagnostic> {
        let scope = FileScope {
            declarations: self.declarations,
            file: self.file,
            substitution,
        };
        aggregates::parameterized::formal_pattern(
            self.declarations.graph,
            aggregates::parameterized::FormalPatternRequest {
                file: self.file,
                syntax,
                span,
                substitution,
            },
            types,
            &self.declarations.nominals,
            records,
            &mut |file, expression| {
                let definition = FileScope {
                    file,
                    ..scope
                };
                jai_eval::evaluate_paths(expression, |path, span| {
                    match definition.value(path, span)? {
                        Binding::Constant(value) => Ok(value),
                        Binding::Enum(value) => Ok(ConstantValue::Int(value.value)),
                        _ => Err(Diagnostic::new(
                            span,
                            "macro formal count requires a definition-site compile-time constant",
                        )),
                    }
                })
                .map_err(|error| located(self.declarations.graph, file, error))
            },
        )
        .map_err(|error| Diagnostic::at_source(error.location, error.message))
    }
    pub(crate) fn generic_result_usages(
        &self,
        declaration: DeclarationId,
    ) -> Option<Vec<syntax::ResultUsage>> {
        self.declarations
            .generics
            .borrow()
            .result_usages(declaration)
    }
    pub(crate) fn preview_generic_results(
        &self,
        matched: &crate::overloads::Match,
        types: &mut TypeRegistry,
        records: &mut aggregates::parameterized::RecordSpecializations,
        span: Span,
    ) -> Result<Vec<TypeId>, Diagnostic> {
        self.declarations.generics.borrow().preview_results(
            matched,
            types,
            span,
            &mut |types, declaration, substitution| {
                self.materialize_generic_record(declaration, substitution, types, records, span)
            },
        )
    }
    pub(crate) fn materialize_pattern(
        &self,
        pattern: &crate::overloads::TypePattern,
        substitution: &crate::polymorphism::Substitution,
        types: &mut TypeRegistry,
        records: &mut aggregates::parameterized::RecordSpecializations,
        span: Span,
    ) -> Result<TypeId, Diagnostic> {
        crate::polymorphism::materialize_with_nominals(
            types,
            pattern,
            substitution,
            &mut |types, declaration, substitution| {
                self.materialize_generic_record(declaration, substitution, types, records, span)
            },
        )
        .map_err(|error| Diagnostic::new(span, format!("invalid callback type pattern: {error:?}")))
    }
    pub(super) fn materialize_generic_record(
        &self,
        declaration: DeclarationId,
        substitution: crate::polymorphism::Substitution,
        types: &mut TypeRegistry,
        records: &mut aggregates::parameterized::RecordSpecializations,
        span: Span,
    ) -> Result<TypeId, crate::polymorphism::SubstitutionError> {
        aggregates::parameterized::instantiate_bound(
            self.declarations.graph,
            aggregates::parameterized::BoundRecordRequest {
                declaration,
                substitution,
                span,
            },
            types,
            &self.declarations.nominals,
            records,
            &mut |file, expression| {
                let scope = FileScope {
                    file,
                    substitution: None,
                    ..*self
                };
                jai_eval::evaluate_paths(expression, |path, span| {
                    match scope.value(path, span)? {
                        Binding::Constant(value) => Ok(value),
                        Binding::Enum(value) => Ok(ConstantValue::Int(value.value)),
                        _ => Err(Diagnostic::new(
                            span,
                            "record specialization requires an immutable scalar constant",
                        )),
                    }
                })
                .map_err(|error| located(self.declarations.graph, file, error))
            },
        )
        .map_err(|error| crate::polymorphism::SubstitutionError::NominalResolution(error.message))
    }
    pub(crate) fn specialize(
        &self,
        matched: &crate::overloads::Match,
        types: &mut TypeRegistry,
        records: &mut aggregates::parameterized::RecordSpecializations,
        layout: Option<jai_types::LayoutPolicy>,
        span: Span,
    ) -> Result<Signature, Diagnostic> {
        self.declarations
            .generics
            .borrow_mut()
            .specialize_with_origins(matched, types, span, &mut |types, declaration, substitution| {
                self.materialize_generic_record(declaration, substitution, types, records, span)
            }, &mut |declaration, signature, types| {
                let source = self.declarations.graph.declaration(declaration)
                    .ok_or_else(|| Diagnostic::new(span, "generic prototype origin is outside the module graph"))?;
                let FileDeclarationKind::ProcedurePrototype(prototype) = &source.syntax().kind else {
                    return Err(Diagnostic::new(span, "generic prototype origin has a source body"));
                };
                match &prototype.binding {
                    syntax::PrototypeBinding::Intrinsic { .. } => super::runtime_intrinsics::bind_prototype(
                        prototype, self.declarations.graph.symbols().name(prototype.name), signature, types, layout,
                    ),
                    syntax::PrototypeBinding::Foreign(_) => Ok(ProcedurePrototype {
                        id: signature.id, signature: signature.ty,
                        origin: super::foreign_libraries::origin(self.declarations.graph, source.file(), prototype)
                            .map_err(|error| Diagnostic::at_source(error.location, error.message))?,
                    }),
                    syntax::PrototypeBinding::Compiler(_) => Err(Diagnostic::new(span, "generic compiler prototypes require specialization-aware compiler binding")),
                    syntax::PrototypeBinding::EntryPoint => Err(Diagnostic::new(span, "#entry_point aliases cannot be polymorphic")),
                }
            })
    }
    pub(crate) fn reserve_local_procedure(&self) -> Result<ProcedureId, Diagnostic> {
        self.declarations
            .generics
            .borrow_mut()
            .reserve_local_procedure()
    }
}
pub(super) fn path(name: Symbol) -> NamePath {
    NamePath {
        root: name,
        members: Vec::new(),
    }
}
pub(super) fn declaration_id(
    graph: &ModuleGraph,
    file: FileInstanceId,
    path: &NamePath,
    span: Span,
) -> Result<DeclarationId, Diagnostic> {
    declaration_lookup(graph, span, graph.lookup(file, path))
}
fn declaration_lookup(
    graph: &ModuleGraph,
    span: Span,
    lookup: Result<GraphBinding, LookupError>,
) -> Result<DeclarationId, Diagnostic> {
    match lookup {
        Ok(GraphBinding::Declaration(id)) => Ok(id),
        Ok(GraphBinding::Parameter(_)) => Err(Diagnostic::new(
            span,
            "module parameter does not denote a declaration",
        )),
        Ok(GraphBinding::Module(_)) => {
            Err(Diagnostic::new(span, "namespace cannot supply a value"))
        }
        Ok(GraphBinding::OverloadSet(_)) => Err(Diagnostic::new(
            span,
            "binding does not denote a single declaration",
        )),
        Ok(GraphBinding::SourceMember {
            ..
        }) => Err(Diagnostic::new(
            span,
            "source member does not denote a standalone declaration",
        )),
        Ok(GraphBinding::StorageMember(_)) => Err(Diagnostic::new(
            span,
            "storage member does not denote a standalone declaration",
        )),
        Err(LookupError::UnfilledPlaceholder(placeholder)) => Err(
            super::placeholder_demands::unfilled_placeholder(graph, placeholder, span),
        ),
        Err(error) => Err(Diagnostic::new(
            span,
            match error {
                LookupError::InvalidFile => "invalid defining file".into(),
                LookupError::UnknownName(name) => {
                    format!("unknown name '{}'", graph.symbols().name(name))
                }
                LookupError::NotNamespace(name) => {
                    format!("'{}' is not a namespace", graph.symbols().name(name))
                }
                LookupError::UnknownMember {
                    name, ..
                } => {
                    format!("unknown module member '{}'", graph.symbols().name(name))
                }
                LookupError::PrivateMember {
                    name, ..
                } => {
                    format!("module member '{}' is private", graph.symbols().name(name))
                }
                LookupError::UnfilledPlaceholder(_) => {
                    unreachable!("placeholder demand is handled before ordinary lookup errors")
                }
            },
        )),
    }
}
pub(super) fn located(
    graph: &ModuleGraph,
    file: FileInstanceId,
    diagnostic: Diagnostic,
) -> LocatedDiagnostic {
    LocatedDiagnostic::new(
        graph
            .file(file)
            .expect("graph declaration has a defining file")
            .source(),
        diagnostic,
    )
}
