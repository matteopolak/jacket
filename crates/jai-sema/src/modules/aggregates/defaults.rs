//! Evaluate declaration-site defaults into immutable typed constants.
use super::parameterized::{RecordSpecializations, baked_scalar};
use super::*;
use crate::local_declarations::{FieldMetadata, RecordMetadata};
use crate::polymorphism::{BakedValue, Substitution};
use jai_ir::{ConstantKind, ConstantValue as TypedConstant};
use std::collections::HashSet;
use syntax::{Expression, ExpressionKind};
mod inferred_casts;
mod native_pointer_constants;
mod promoted_literals;

enum ScalarSource<'a, 'b> {
    Constants(&'b Constants<'a>),
    Evaluate(
        &'b mut dyn FnMut(FileInstanceId, &Expression) -> Result<ConstantValue, LocatedDiagnostic>,
    ),
}

#[derive(Clone)]
struct DefaultRecord {
    file: FileInstanceId,
    shape: RecordMetadata,
    substitution: Option<Substitution>,
}

pub(crate) struct Defaults<'a, 'b> {
    pub graph: &'a ModuleGraph,
    pub types: &'b TypeRegistry,
    pub nominals: &'b Nominals<'a>,
    scalar_source: ScalarSource<'a, 'b>,
    specializations: Option<&'b RecordSpecializations>,
    context: Option<&'b crate::context::Schema>,
    substitution: Option<Substitution>,
    conversion_fields: Option<&'b HashMap<TypeId, Vec<(FieldId, TypeId)>>>,
    pub fields: HashMap<FieldId, TypedConstant>,
    pub named: HashMap<DeclarationId, TypedConstant>,
    active: HashSet<FieldId>,
    blocked_fields: HashSet<FieldId>,
    depth: usize,
    remaining_cells: usize,
}
impl<'a, 'b> Defaults<'a, 'b> {
    pub(crate) fn coerce_field_constant(
        &self,
        value: TypedConstant,
        target: TypeId,
        span: Span,
    ) -> Result<TypedConstant, Diagnostic> {
        if value.ty == target {
            return Ok(value);
        }
        let path =
            crate::field_conversions::find_conversion_path(value.ty, Some(target), span, |ty| {
                if let Some(fields) = self.conversion_fields.and_then(|fields| fields.get(&ty)) {
                    for &(field, expected) in fields {
                        let actual = self
                            .types
                            .validate_field(ty, field)
                            .map_err(|error| Diagnostic::new(span, error.to_string()))?;
                        if actual != expected {
                            return Err(Diagnostic::new(
                                span,
                                "implicit conversion field metadata has a different canonical type",
                            ));
                        }
                    }
                    return Ok(fields.clone());
                }
                let Some(record) = self.record(ty) else {
                    return Ok(Vec::new());
                };
                let mut fields = Vec::new();
                for field in record.shape.fields {
                    if field.syntax.conversion() != syntax::FieldConversion::Implicit {
                        continue;
                    }
                    if record.shape.kind == jai_types::RecordKind::Union {
                        return Err(Diagnostic::new(
                            span,
                            "union #as fields require active-alternative conversion semantics",
                        ));
                    }
                    let ty = self
                        .types
                        .validate_field(ty, field.id)
                        .map_err(|error| Diagnostic::new(span, error.to_string()))?;
                    if ty != field.ty {
                        return Err(Diagnostic::new(
                            span,
                            "implicit conversion field metadata has a different canonical type",
                        ));
                    }
                    fields.push((field.id, ty));
                }
                Ok(fields)
            })?
            .ok_or_else(|| Diagnostic::new(span, "typed constant has a different type"))?;
        crate::field_conversions::project_constant(value, &path, self.types, span)
    }
    pub fn new(
        graph: &'a ModuleGraph,
        types: &'b TypeRegistry,
        nominals: &'b Nominals<'a>,
        constants: &'b Constants<'a>,
    ) -> Self {
        Self {
            graph,
            types,
            nominals,
            scalar_source: ScalarSource::Constants(constants),
            specializations: None,
            context: None,
            substitution: None,
            conversion_fields: None,
            fields: HashMap::new(),
            named: nominals.value_constants.clone(),
            active: HashSet::new(),
            blocked_fields: HashSet::new(),
            depth: 0,
            remaining_cells: crate::constant_limits::MAX_CONSTANT_CELLS,
        }
    }
    pub(crate) fn with_evaluator(
        graph: &'a ModuleGraph,
        types: &'b TypeRegistry,
        nominals: &'b Nominals<'a>,
        evaluate: &'b mut dyn FnMut(
            FileInstanceId,
            &Expression,
        ) -> Result<ConstantValue, LocatedDiagnostic>,
    ) -> Self {
        Self {
            graph,
            types,
            nominals,
            scalar_source: ScalarSource::Evaluate(evaluate),
            specializations: None,
            context: None,
            substitution: None,
            conversion_fields: None,
            fields: HashMap::new(),
            named: nominals.value_constants.clone(),
            active: HashSet::new(),
            blocked_fields: HashSet::new(),
            depth: 0,
            remaining_cells: crate::constant_limits::MAX_CONSTANT_CELLS,
        }
    }
    pub(crate) fn with_specializations(mut self, records: &'b RecordSpecializations) -> Self {
        self.specializations = Some(records);
        self
    }
    /// Context defaults were already evaluated in each field's defining file.
    /// Retain that canonical schema instead of reevaluating fields in the consumer's scope.
    pub(crate) fn with_context(mut self, context: Option<&'b crate::context::Schema>) -> Self {
        self.context = context;
        self
    }
    pub(crate) fn with_substitution(mut self, substitution: Option<Substitution>) -> Self {
        self.substitution = substitution;
        self
    }
    pub(crate) fn with_conversion_fields(
        mut self,
        fields: &'b HashMap<TypeId, Vec<(FieldId, TypeId)>>,
    ) -> Self {
        self.conversion_fields = Some(fields);
        self
    }
    fn record(&self, ty: TypeId) -> Option<DefaultRecord> {
        if let Some(context) = self
            .context
            .filter(|context| context.definition.record_type == ty)
        {
            return Some(DefaultRecord {
                file: self.graph.module(self.graph.root()).unwrap().entry(),
                shape: context.record_metadata(),
                substitution: None,
            });
        }
        if let Some(record) = self.specializations.and_then(|records| records.record(ty)) {
            return Some(DefaultRecord {
                file: record.file,
                shape: record.shape.clone(),
                substitution: Some(record.substitution.clone()),
            });
        }
        self.nominals.records.get(&ty).map(|record| DefaultRecord {
            file: record.file,
            substitution: None,
            shape: RecordMetadata {
                name: None,
                kind: record.kind,
                fields: record
                    .fields
                    .iter()
                    .map(|field| FieldMetadata {
                        name: Some(field.name),
                        id: field.id,
                        ty: field.ty,
                        syntax: field.syntax.clone().into(),
                    })
                    .collect(),
            },
        })
    }
    fn literal_type(
        &self,
        file: FileInstanceId,
        path: &syntax::NamePath,
        span: Span,
    ) -> Result<TypeId, LocatedDiagnostic> {
        let bound = self
            .specializations
            .and_then(|records| {
                super::parameterized::member_value(
                    self.graph,
                    file,
                    self.nominals,
                    records,
                    self.substitution.as_ref(),
                    path,
                    span,
                )
            })
            .or_else(|| {
                path.members
                    .is_empty()
                    .then(|| {
                        self.substitution
                            .as_ref()
                            .and_then(|scope| scope.ty(path.root))
                    })
                    .flatten()
                    .map(BakedValue::Type)
            });
        if let Some(value) = bound {
            return match value {
                BakedValue::Type(ty) => Ok(ty),
                _ => Err(located(
                    self.graph,
                    file,
                    Diagnostic::new(span, "literal declaration does not denote a type"),
                )),
            };
        }
        let declaration = declaration_id(self.graph, file, path, span)
            .map_err(|error| located(self.graph, file, error))?;
        self.nominals
            .declarations
            .get(&declaration)
            .copied()
            .ok_or_else(|| {
                located(
                    self.graph,
                    file,
                    Diagnostic::new(span, "literal declaration does not denote a type"),
                )
            })
    }
    fn scalar(
        &mut self,
        file: FileInstanceId,
        expression: &Expression,
    ) -> Result<ConstantValue, LocatedDiagnostic> {
        let substitution = self.substitution.clone();
        if let Some(substitution) = substitution {
            return jai_eval::evaluate_paths(expression, |path, span| {
                if path.members.is_empty()
                    && let Some(value) = substitution.constant(path.root)
                {
                    return baked_scalar(value).ok_or_else(|| {
                        Diagnostic::new(span, "baked template value requires a scalar constant")
                    });
                }
                self.source_scalar(
                    file,
                    &Expression {
                        kind: ExpressionKind::QualifiedName(path.clone()),
                        span,
                    },
                )
                .map_err(|error| Diagnostic::at_source(error.location, error.message))
            })
            .map_err(|error| located(self.graph, file, error));
        }
        self.source_scalar(file, expression)
    }
    fn source_scalar(
        &mut self,
        file: FileInstanceId,
        expression: &Expression,
    ) -> Result<ConstantValue, LocatedDiagnostic> {
        match &mut self.scalar_source {
            ScalarSource::Constants(constants) => constants.evaluate(file, expression),
            ScalarSource::Evaluate(evaluate) => evaluate(file, expression),
        }
    }
    pub(crate) fn field_default(
        &mut self,
        id: FieldId,
    ) -> Result<TypedConstant, LocatedDiagnostic> {
        self.field(id)
    }
    pub(crate) fn prepare(
        mut self,
        jobs: &crate::modules::field_default_jobs::FieldDefaultJobs,
    ) -> Result<crate::modules::field_default_jobs::PreparedFieldDefaults, LocatedDiagnostic> {
        self.blocked_fields = jobs.blocked_fields();
        let pending = jobs.pending_fields();
        for (field, value) in jobs.ready() {
            self.fields.insert(field, value.clone());
        }
        Ok(crate::modules::field_default_jobs::PreparedFieldDefaults {
            ready: self.build()?,
            pending,
        })
    }
    pub fn build(mut self) -> Result<HashMap<FieldId, TypedConstant>, LocatedDiagnostic> {
        let mut record_types = self
            .nominals
            .records
            .keys()
            .copied()
            .collect::<HashSet<_>>();
        if let Some(records) = self.specializations {
            record_types.extend(records.records().map(|(ty, _)| ty));
        }
        for ty in record_types {
            let record = self.record(ty).expect("known record metadata");
            for field in &record.shape.fields {
                if self.blocked_fields.contains(&field.id) {
                    continue;
                }
                if field.syntax.initializer().is_some()
                    || self
                        .specializations
                        .is_some_and(|records| !records.default_overrides(field.id).is_empty())
                {
                    self.field(field.id)?;
                }
            }
        }
        Ok(self.fields)
    }
    fn field(&mut self, id: FieldId) -> Result<TypedConstant, LocatedDiagnostic> {
        let (record, field) = self
            .specializations
            .and_then(|records| records.field(id))
            .map(|(record, field)| {
                (
                    DefaultRecord {
                        file: record.file,
                        shape: record.shape.clone(),
                        substitution: Some(record.substitution.clone()),
                    },
                    field.clone(),
                )
            })
            .or_else(|| {
                self.nominals.records.iter().find_map(|(&ty, record)| {
                    record
                        .fields
                        .iter()
                        .find(|field| field.id == id)
                        .map(|field| {
                            (
                                self.record(ty).unwrap(),
                                FieldMetadata {
                                    name: Some(field.name),
                                    id: field.id,
                                    ty: field.ty,
                                    syntax: field.syntax.clone().into(),
                                },
                            )
                        })
                })
            })
            .or_else(|| {
                let context = self.context?;
                let shape = context.record_metadata();
                let field = shape.fields.iter().find(|field| field.id == id)?.clone();
                Some((
                    DefaultRecord {
                        file: context
                            .field_origin(id)
                            .expect("context field has a defining file"),
                        shape,
                        substitution: None,
                    },
                    field,
                ))
            })
            .expect("field belongs to record schema");
        let context_default = self.context.and_then(|context| {
            self.types
                .validate_field(context.definition.record_type, id)
                .ok()?;
            let index = id.index();
            let ConstantKind::Record(defaults) = &context.definition.default.kind else {
                unreachable!("context schema default is a record");
            };
            Some(&defaults[index])
        });
        if let Some(value) = context_default.or_else(|| self.fields.get(&id)) {
            let cells = crate::constant_limits::cells(value).ok_or_else(|| {
                located(
                    self.graph,
                    record.file,
                    Diagnostic::new(
                        field.syntax.span(),
                        "constant exceeds compiler constant cell budget",
                    ),
                )
            })?;
            self.charge(record.file, field.syntax.span(), cells)?;
            return Ok(context_default.unwrap_or_else(|| &self.fields[&id]).clone());
        }
        if self.blocked_fields.contains(&id) {
            return Err(located(
                self.graph,
                record.file,
                Diagnostic::new(
                    field.syntax.span(),
                    "record field default requires a ready typed initializer job",
                ),
            ));
        }
        if !self.active.insert(id) {
            return Err(located(
                self.graph,
                record.file,
                Diagnostic::new(field.syntax.span(), "cyclic record field defaults"),
            ));
        }
        let expression = field.syntax.initializer();
        let previous = std::mem::replace(&mut self.substitution, record.substitution.clone());
        let overrides = self
            .specializations
            .map(|records| records.default_overrides(id).to_vec())
            .unwrap_or_default();
        let result = (|| {
            let mut value = match expression {
                Some(expression) => self.expression(record.file, expression, field.ty),
                None => {
                    self.enter(record.file, field.syntax.span())?;
                    let value = self.default_value_inner(
                        record.file,
                        field.ty,
                        field.syntax.span(),
                        field.syntax.is_anonymous(),
                    );
                    self.depth -= 1;
                    value
                }
            }?;
            for override_ in overrides {
                let expected = self
                    .types
                    .field_type(*override_.path.last().expect("override has a root field"))
                    .map_err(|error| {
                        located(
                            self.graph,
                            record.file,
                            Diagnostic::new(override_.span, error.to_string()),
                        )
                    })?;
                let replacement = self.expression(record.file, &override_.value, expected)?;
                let before = crate::constant_limits::cells(&value).ok_or_else(|| {
                    located(
                        self.graph,
                        record.file,
                        Diagnostic::new(
                            override_.span,
                            "record default exceeds compiler constant cell budget",
                        ),
                    )
                })?;
                value = crate::record_default_overrides::replace_constant(
                    value,
                    &override_.path[1..],
                    replacement,
                    self.types,
                    override_.span,
                )
                .map_err(|error| located(self.graph, record.file, error))?;
                let after = crate::constant_limits::cells(&value).ok_or_else(|| {
                    located(
                        self.graph,
                        record.file,
                        Diagnostic::new(
                            override_.span,
                            "record default exceeds compiler constant cell budget",
                        ),
                    )
                })?;
                self.charge(record.file, override_.span, after.saturating_sub(before))?;
            }
            Ok(value)
        })();
        self.substitution = previous;
        self.active.remove(&id);
        let value = result?;
        self.fields.insert(id, value.clone());
        Ok(value)
    }
    fn charge(
        &mut self,
        file: FileInstanceId,
        span: Span,
        cells: usize,
    ) -> Result<(), LocatedDiagnostic> {
        self.remaining_cells = self.remaining_cells.checked_sub(cells).ok_or_else(|| {
            located(
                self.graph,
                file,
                Diagnostic::new(span, "constant exceeds compiler constant cell budget"),
            )
        })?;
        Ok(())
    }
    fn enter(&mut self, file: FileInstanceId, span: Span) -> Result<(), LocatedDiagnostic> {
        if self.depth >= crate::constant_limits::MAX_CONSTANT_DEPTH {
            return Err(located(
                self.graph,
                file,
                Diagnostic::new(span, "constant exceeds compiler constant depth budget"),
            ));
        }
        self.charge(file, span, 1)?;
        self.depth += 1;
        Ok(())
    }
    pub fn default_value(
        &mut self,
        file: FileInstanceId,
        ty: TypeId,
        span: Span,
    ) -> Result<TypedConstant, LocatedDiagnostic> {
        self.enter(file, span)?;
        let result = self.default_value_inner(file, ty, span, false);
        self.depth -= 1;
        result
    }
    pub fn expression(
        &mut self,
        file: FileInstanceId,
        expression: &Expression,
        ty: TypeId,
    ) -> Result<TypedConstant, LocatedDiagnostic> {
        self.enter(file, expression.span)?;
        let result = self.expression_inner(file, expression, ty);
        self.depth -= 1;
        result
    }
    fn default_value_inner(
        &mut self,
        file: FileInstanceId,
        ty: TypeId,
        span: Span,
        anonymous_member: bool,
    ) -> Result<TypedConstant, LocatedDiagnostic> {
        let kind = match self
            .types
            .kind(ty)
            .map_err(|error| located(self.graph, file, Diagnostic::new(span, error.to_string())))?
        {
            TypeKind::Integer(integer) => ConstantKind::Int(IntegerValue::wrapping(*integer, 0)),
            TypeKind::Bool => ConstantKind::Bool(false),
            TypeKind::Float(float) => ConstantKind::Float(match float {
                jai_types::FloatType::F32 => jai_types::FloatValue::F32(0),
                jai_types::FloatType::F64 => jai_types::FloatValue::F64(0),
            }),
            TypeKind::String => ConstantKind::StringBytes(Vec::new()),
            TypeKind::Distinct(_) => {
                let representation = self
                    .types
                    .distinct_definition(ty)
                    .map_err(|error| {
                        located(self.graph, file, Diagnostic::new(span, error.to_string()))
                    })?
                    .representation;
                ConstantKind::Distinct(Box::new(self.default_value(file, representation, span)?))
            }
            TypeKind::FixedArray { element, count } => {
                let element = *element;
                let count = usize::try_from(*count).map_err(|_| {
                    located(
                        self.graph,
                        file,
                        Diagnostic::new(
                            span,
                            "array default count exceeds host addressable limits",
                        ),
                    )
                })?;
                if count == 0 {
                    ConstantKind::Zero
                } else {
                    let initial = self.default_value(file, element, span)?;
                    if crate::constant_limits::is_zero(&initial) {
                        ConstantKind::Zero
                    } else {
                        let cells = crate::constant_limits::cells(&initial)
                            .and_then(|cells| cells.checked_mul(count.saturating_sub(1)))
                            .ok_or_else(|| {
                                located(
                                    self.graph,
                                    file,
                                    Diagnostic::new(
                                        span,
                                        "array default exceeds compiler constant cell budget",
                                    ),
                                )
                            })?;
                        self.charge(file, span, cells)?;
                        ConstantKind::Array(vec![initial; count])
                    }
                }
            }
            TypeKind::Enum(_) => {
                if let Some(enumeration) = self
                    .specializations
                    .and_then(|records| records.member_enum(ty))
                {
                    if !enumeration.flags
                        && enumeration
                            .values
                            .first()
                            .is_none_or(|(_, value)| value.value() != 0)
                    {
                        return Err(located(
                            self.graph,
                            file,
                            Diagnostic::new(
                                span,
                                "enum default initialization requires an explicit zero value",
                            ),
                        ));
                    }
                    return Ok(TypedConstant {
                        ty,
                        kind: ConstantKind::Enum(IntegerValue::wrapping(
                            enumeration.representation,
                            0,
                        )),
                    });
                }
                let enumeration = &self.nominals.enums[&ty];
                if !enumeration.flags
                    && enumeration
                        .values
                        .first()
                        .is_none_or(|value| value.value() != 0)
                {
                    return Err(located(
                        self.graph,
                        file,
                        Diagnostic::new(
                            span,
                            "enum default initialization requires an explicit value until its language rule is established",
                        ),
                    ));
                }
                ConstantKind::Enum(IntegerValue::wrapping(enumeration.representation, 0))
            }
            TypeKind::Any(_) => ConstantKind::Zero,
            TypeKind::Record(_) => {
                let record = self.record(ty).ok_or_else(|| {
                    located(
                        self.graph,
                        file,
                        Diagnostic::new(
                            span,
                            "record default requires an established source schema",
                        ),
                    )
                })?;
                let union = record.shape.kind == jai_types::RecordKind::Union;
                if union && !anonymous_member {
                    return Err(located(
                        self.graph,
                        file,
                        Diagnostic::new(span, "union default initialization is not implemented"),
                    ));
                }
                let mut fields = Vec::new();
                for field in &record.shape.fields {
                    fields.push(self.field(field.id)?);
                }
                if union {
                    if !fields.iter().all(crate::constant_limits::is_zero) {
                        return Err(located(
                            self.graph,
                            file,
                            Diagnostic::new(
                                span,
                                "anonymous union defaults require identical zero storage or an explicit active alternative",
                            ),
                        ));
                    }
                    ConstantKind::Zero
                } else {
                    ConstantKind::Record(fields)
                }
            }
            TypeKind::Type
            | TypeKind::Pointer(_)
            | TypeKind::Slice(_)
            | TypeKind::DynamicArray(_)
            | TypeKind::Procedure(_) => ConstantKind::Zero,
            _ => {
                return Err(located(
                    self.graph,
                    file,
                    Diagnostic::new(
                        span,
                        "default initialization is not implemented for this type",
                    ),
                ));
            }
        };
        Ok(TypedConstant { ty, kind })
    }
    fn annotation_matches(
        &mut self,
        file: FileInstanceId,
        annotation: &syntax::TypeSyntax,
        expected: TypeId,
        span: Span,
    ) -> Result<bool, LocatedDiagnostic> {
        let actual = self
            .types
            .kind(expected)
            .map_err(|error| located(self.graph, file, Diagnostic::new(span, error.to_string())))?;
        Ok(match annotation {
            syntax::TypeSyntax::Builtin(builtin) => match builtin {
                syntax::BuiltinType::Scalar(ty) => self.types.scalar(*ty) == expected,
                syntax::BuiltinType::Float(ty) => self.types.float(*ty) == expected,
                syntax::BuiltinType::String => self.types.string() == expected,
                syntax::BuiltinType::Void => self.types.void() == expected,
                syntax::BuiltinType::Type => self.types.meta_type() == expected,
                syntax::BuiltinType::Any => self.types.any_type() == Some(expected),
                syntax::BuiltinType::Context => self.nominals.context_type_id() == Some(expected),
            },
            syntax::TypeSyntax::Named(path) => {
                if let Some(BakedValue::Type(ty)) = self.specializations.and_then(|records| {
                    super::parameterized::member_value(
                        self.graph,
                        file,
                        self.nominals,
                        records,
                        self.substitution.as_ref(),
                        path,
                        span,
                    )
                }) {
                    return Ok(ty == expected);
                }
                declaration_id(self.graph, file, path, span)
                    .ok()
                    .and_then(|id| self.nominals.declarations.get(&id).copied())
                    == Some(expected)
            }
            syntax::TypeSyntax::Pointer(inner) => match actual {
                TypeKind::Pointer(element) => {
                    self.annotation_matches(file, inner, *element, span)?
                }
                _ => false,
            },
            syntax::TypeSyntax::Slice(inner) => match actual {
                TypeKind::Slice(element) => self.annotation_matches(file, inner, *element, span)?,
                _ => false,
            },
            syntax::TypeSyntax::DynamicArray(inner) => match actual {
                TypeKind::DynamicArray(element) => {
                    self.annotation_matches(file, inner, *element, span)?
                }
                _ => false,
            },
            syntax::TypeSyntax::FixedArray { count, element } => match actual {
                TypeKind::FixedArray {
                    element: expected_element,
                    count: expected_count,
                } => {
                    let count = match self.scalar(file, count)? {
                        ConstantValue::Literal(value) => u64::try_from(value).ok(),
                        ConstantValue::Int(value) => u64::try_from(value.value()).ok(),
                        _ => None,
                    };
                    count == Some(*expected_count)
                        && self.annotation_matches(file, element, *expected_element, span)?
                }
                _ => false,
            },
            syntax::TypeSyntax::Procedure(source) => {
                let Ok(signature) = self.types.procedure_definition(expected).cloned() else {
                    return Ok(false);
                };
                if source.convention != signature.convention
                    || source.context != signature.context
                    || source.results.len() != signature.results.len()
                {
                    return Ok(false);
                }
                let source_pack = source
                    .parameters
                    .iter()
                    .position(|parameter| parameter.variadic);
                if source
                    .parameters
                    .iter()
                    .filter(|parameter| parameter.variadic)
                    .count()
                    > 1
                {
                    return Ok(false);
                }
                let matches_pack = match (source_pack, signature.variadic) {
                    (None, jai_types::Variadic::None) => {
                        source.parameters.len() == signature.parameters.len()
                    }
                    (Some(index), jai_types::Variadic::C { fixed_parameters }) => {
                        source.convention == CallingConvention::C
                            && index == fixed_parameters
                            && source.parameters.len() == fixed_parameters + 1
                            && signature.parameters.len() == fixed_parameters
                    }
                    (Some(index), jai_types::Variadic::Jai { parameter, .. }) => {
                        index == parameter && source.parameters.len() == signature.parameters.len()
                    }
                    _ => false,
                };
                if !matches_pack {
                    return Ok(false);
                }
                for (index, (&ty, parameter)) in signature
                    .parameters
                    .iter()
                    .zip(&source.parameters)
                    .enumerate()
                {
                    let expected = match signature.variadic {
                        jai_types::Variadic::Jai { parameter, element } if parameter == index => {
                            element
                        }
                        _ => ty,
                    };
                    if !self.annotation_matches(file, &parameter.ty, expected, span)? {
                        return Ok(false);
                    }
                }
                for (&ty, result) in signature.results.iter().zip(&source.results) {
                    if !self.annotation_matches(file, &result.ty, ty, span)? {
                        return Ok(false);
                    }
                }
                true
            }
            _ => {
                return Err(located(
                    self.graph,
                    file,
                    Diagnostic::new(
                        span,
                        "literal element annotation requires a resolved declaration type",
                    ),
                ));
            }
        })
    }
    fn expression_inner(
        &mut self,
        file: FileInstanceId,
        expression: &Expression,
        ty: TypeId,
    ) -> Result<TypedConstant, LocatedDiagnostic> {
        if matches!(
            expression.kind,
            ExpressionKind::Cast(jai_types::CastMode::Force(_), _, _)
                | ExpressionKind::TypeCast {
                    mode: jai_types::CastMode::Force(_),
                    ..
                }
                | ExpressionKind::InferredCast {
                    mode: jai_types::CastMode::Force(_),
                    ..
                }
        ) {
            return Err(located(
                self.graph,
                file,
                Diagnostic::new(
                    expression.span,
                    "storage cast declaration defaults require target-layout VM constant materialization",
                ),
            ));
        }
        if let ExpressionKind::InferredCast { mode, value } = &expression.kind {
            return self.inferred_cast_constant(file, value, ty, *mode, expression.span);
        }
        let kind = self
            .types
            .kind(ty)
            .map_err(|error| {
                located(
                    self.graph,
                    file,
                    Diagnostic::new(expression.span, error.to_string()),
                )
            })?
            .clone();
        if matches!(
            expression.kind,
            ExpressionKind::SourceFile
                | ExpressionKind::SourceFilepath
                | ExpressionKind::SourceLine
                | ExpressionKind::SourceLocation
        ) {
            let location = SourceSpan {
                source: self
                    .graph
                    .file(file)
                    .expect("resolved source file")
                    .source(),
                span: expression.span,
            };
            let record = self.graph.sources().get(location.source).ok_or_else(|| {
                located(
                    self.graph,
                    file,
                    Diagnostic::at_source(location, "source origin record is not retained"),
                )
            })?;
            let point = crate::source_locations::source_point(record, location)
                .map_err(|error| located(self.graph, file, error))?;
            let constant = match (&expression.kind, &kind) {
                (ExpressionKind::SourceFile, TypeKind::String) => {
                    self.charge(file, expression.span, point.filename.len())?;
                    ConstantKind::StringBytes(point.filename.as_bytes().to_vec())
                }
                (ExpressionKind::SourceFilepath, TypeKind::String) => {
                    let directory = point.directory.ok_or_else(|| {
                        located(
                            self.graph,
                            file,
                            Diagnostic::at_source(
                                location,
                                "#filepath requires a retained source directory",
                            ),
                        )
                    })?;
                    self.charge(file, expression.span, directory.len())?;
                    ConstantKind::StringBytes(directory.as_bytes().to_vec())
                }
                (ExpressionKind::SourceLine, TypeKind::Integer(integer)) => {
                    let value =
                        IntegerValue::checked(*integer, point.line as i128).ok_or_else(|| {
                            located(
                                self.graph,
                                file,
                                Diagnostic::at_source(
                                    location,
                                    "source line exceeds its declared integer type",
                                ),
                            )
                        })?;
                    ConstantKind::Int(value)
                }
                (ExpressionKind::SourceLocation, _) => {
                    crate::caller_locations::validate_target(
                        self.graph,
                        self.nominals,
                        self.types,
                        ty,
                        expression.span,
                    )
                    .map_err(|error| located(self.graph, file, error))?;
                    self.charge(file, expression.span, point.filename.len() + 3)?;
                    return crate::source_locations::location_constant(
                        record, location, ty, self.types,
                    )
                    .map_err(|error| located(self.graph, file, error));
                }
                _ => {
                    return Err(located(
                        self.graph,
                        file,
                        Diagnostic::at_source(
                            location,
                            "source directive differs from its declared type",
                        ),
                    ));
                }
            };
            return Ok(TypedConstant { ty, kind: constant });
        }
        if let ExpressionKind::TypeCast {
            ty: annotation,
            value,
            mode,
        } = &expression.kind
            && matches!(kind, TypeKind::Pointer(_))
        {
            if !self.annotation_matches(file, annotation, ty, expression.span)? {
                return Err(located(
                    self.graph,
                    file,
                    Diagnostic::new(
                        expression.span,
                        "pointer constant cast requires the declared canonical target",
                    ),
                ));
            }
            return self.inferred_cast_constant(file, value, ty, *mode, expression.span);
        }
        if let ExpressionKind::TypeCast {
            ty: annotation,
            value,
            mode,
        } = &expression.kind
            && matches!(kind, TypeKind::Procedure(_))
        {
            if *mode == jai_types::CastMode::Truncate {
                return Err(located(
                    self.graph,
                    file,
                    Diagnostic::new(
                        expression.span,
                        "trunc cast is supported only for integer and pointer representations",
                    ),
                ));
            }
            if !self.annotation_matches(file, annotation, ty, expression.span)? {
                return Err(located(
                    self.graph,
                    file,
                    Diagnostic::new(
                        expression.span,
                        "procedure constant cast requires the same canonical signature",
                    ),
                ));
            }
            return self.expression(file, value, ty);
        }
        if let Some(path) = super::parameterized::expression_path(expression)
            && let Some(BakedValue::Value(value)) = self.specializations.and_then(|records| {
                super::parameterized::member_value(
                    self.graph,
                    file,
                    self.nominals,
                    records,
                    self.substitution.as_ref(),
                    &path,
                    expression.span,
                )
            })
        {
            if value.ty == ty || matches!(self.types.kind(value.ty), Ok(TypeKind::Record(_))) {
                return self
                    .coerce_field_constant(value, ty, expression.span)
                    .map_err(|error| located(self.graph, file, error));
            }
            if matches!(value.kind, ConstantKind::Enum(_)) {
                return Err(located(
                    self.graph,
                    file,
                    Diagnostic::new(
                        expression.span,
                        "enum constant belongs to a different nominal type",
                    ),
                ));
            }
        }
        let parameter_name = match &expression.kind {
            ExpressionKind::Name(name) | ExpressionKind::CompileVariable(name) => Some(*name),
            ExpressionKind::QualifiedName(path) if path.members.is_empty() => Some(path.root),
            _ => None,
        };
        if let Some(value) = parameter_name
            .and_then(|name| self.substitution.as_ref()?.constant(name))
            .cloned()
        {
            let value = match value {
                BakedValue::Value(value)
                    if value.ty == ty
                        || matches!(self.types.kind(value.ty), Ok(TypeKind::Record(_))) =>
                {
                    Some(
                        self.coerce_field_constant(value, ty, expression.span)
                            .map_err(|error| located(self.graph, file, error))?,
                    )
                }
                BakedValue::String(bytes) if matches!(kind, TypeKind::String) => {
                    Some(TypedConstant {
                        ty,
                        kind: ConstantKind::StringBytes(bytes.to_vec()),
                    })
                }
                _ => None,
            };
            if let Some(value) = value {
                return Ok(value);
            }
        }
        let composite_source = matches!(
            expression.kind,
            ExpressionKind::StructLiteral(_) | ExpressionKind::PositionalStructLiteral(_)
        ) || super::parameterized::expression_path(expression)
            .and_then(|path| declaration_id(self.graph, file, &path, expression.span).ok())
            .is_some_and(|id| {
                self.named.contains_key(&id)
                    || self.graph.declaration(id).is_some_and(|declaration| {
                        crate::modules::sequence_constants::is_sequence_constant(
                            self.graph,
                            declaration,
                        )
                    })
            });
        match (&expression.kind, &kind) {
            (ExpressionKind::InferredMember(name), TypeKind::Enum(_)) => {
                let value = self
                    .specializations
                    .and_then(|records| records.member_enum(ty))
                    .and_then(|enumeration| {
                        enumeration
                            .values
                            .iter()
                            .find(|(member, _)| member == name)
                            .map(|(_, value)| *value)
                    })
                    .or_else(|| self.nominals.enums.get(&ty)?.members.get(name).copied())
                    .ok_or_else(|| {
                        located(
                            self.graph,
                            file,
                            Diagnostic::new(
                                expression.span,
                                "leading-dot member does not belong to the contextual enum",
                            ),
                        )
                    })?;
                return Ok(TypedConstant {
                    ty,
                    kind: ConstantKind::Enum(value),
                });
            }
            (ExpressionKind::Null, TypeKind::Pointer(_) | TypeKind::Procedure(_)) => {
                return Ok(TypedConstant {
                    ty,
                    kind: ConstantKind::Zero,
                });
            }
            (ExpressionKind::String(bytes), TypeKind::String) => {
                self.charge(file, expression.span, bytes.len())?;
                return Ok(TypedConstant {
                    ty,
                    kind: ConstantKind::StringBytes(bytes.clone()),
                });
            }
            (ExpressionKind::HereString(literal), TypeKind::String) => {
                self.charge(file, expression.span, literal.bytes.len())?;
                return Ok(TypedConstant {
                    ty,
                    kind: ConstantKind::StringBytes(literal.bytes.clone()),
                });
            }
            (ExpressionKind::ArrayLiteral(literal), TypeKind::FixedArray { element, count }) => {
                if let Some(annotation) = &literal.element_type
                    && !self.annotation_matches(file, annotation, *element, expression.span)?
                {
                    return Err(located(
                        self.graph,
                        file,
                        Diagnostic::new(
                            expression.span,
                            "array literal element type differs from its context",
                        ),
                    ));
                }
                if u64::try_from(literal.elements.len()).ok() != Some(*count) {
                    return Err(located(
                        self.graph,
                        file,
                        Diagnostic::new(
                            expression.span,
                            "array constant length differs from its type",
                        ),
                    ));
                }
                let elements = literal
                    .elements
                    .iter()
                    .map(|expression| self.expression(file, expression, *element))
                    .collect::<Result<Vec<_>, _>>()?;
                return Ok(TypedConstant {
                    ty,
                    kind: ConstantKind::Array(elements),
                });
            }
            (_, TypeKind::Float(float)) if !composite_source => {
                let value = jai_eval::evaluate_float_paths(expression, *float, |path, span| {
                    let expression = Expression {
                        kind: ExpressionKind::QualifiedName(path.clone()),
                        span,
                    };
                    self.scalar(file, &expression)
                        .map_err(|error| Diagnostic::at_source(error.location, error.message))
                })
                .map_err(|error| located(self.graph, file, error))?;
                return Ok(TypedConstant {
                    ty,
                    kind: ConstantKind::Float(value),
                });
            }
            (
                ExpressionKind::StructLiteral(literal),
                TypeKind::String | TypeKind::Slice(_) | TypeKind::DynamicArray(_),
            ) if literal.ty.is_none() && literal.fields.is_empty() => {
                return self.default_value(file, ty, expression.span);
            }
            _ => {}
        }
        if let ExpressionKind::PositionalStructLiteral(literal) = &expression.kind {
            let actual = match &literal.ty {
                None => ty,
                Some(path) => self.literal_type(file, path, expression.span)?,
            };
            if actual != ty {
                let value = self.expression(file, expression, actual)?;
                return self
                    .coerce_field_constant(value, ty, expression.span)
                    .map_err(|error| located(self.graph, file, error));
            }
            let record = self.record(ty).ok_or_else(|| {
                located(
                    self.graph,
                    file,
                    Diagnostic::new(
                        expression.span,
                        "positional record literal requires a record type",
                    ),
                )
            })?;
            if record.shape.kind != jai_types::RecordKind::Struct {
                return Err(located(
                    self.graph,
                    file,
                    Diagnostic::new(
                        expression.span,
                        "positional union constants require an explicit alternative",
                    ),
                ));
            }
            if literal.values.len() > record.shape.fields.len() {
                return Err(located(
                    self.graph,
                    file,
                    Diagnostic::new(
                        expression.span,
                        "positional record constant has too many values",
                    ),
                ));
            }
            let mut fields = Vec::with_capacity(record.shape.fields.len());
            for (index, field) in record.shape.fields.iter().enumerate() {
                fields.push(match literal.values.get(index) {
                    Some(expression) => self.expression(file, expression, field.ty)?,
                    None => self.field(field.id)?,
                });
            }
            return Ok(TypedConstant {
                ty,
                kind: ConstantKind::Record(fields),
            });
        }
        if let ExpressionKind::StructLiteral(literal) = &expression.kind {
            let actual = match &literal.ty {
                None => ty,
                Some(path) => self.literal_type(file, path, expression.span)?,
            };
            if actual != ty {
                let value = self.expression(file, expression, actual)?;
                return self
                    .coerce_field_constant(value, ty, expression.span)
                    .map_err(|error| located(self.graph, file, error));
            }
            self.record(ty).ok_or_else(|| {
                located(
                    self.graph,
                    file,
                    Diagnostic::new(expression.span, "record literal requires a record type"),
                )
            })?;
            return self.promoted_record_constant(file, literal, ty, expression.span);
        }
        let name = match &expression.kind {
            ExpressionKind::Name(name) => Some(path(*name)),
            ExpressionKind::QualifiedName(path) => Some(path.clone()),
            _ => None,
        };
        if let Some(path) = name {
            if let Ok(jai_modules::Binding::Parameter(id)) = self.graph.lookup(file, &path)
                && let jai_modules::ParameterValue::Enumeration(value) =
                    &self.graph.parameter(id).unwrap().value
            {
                let actual = self
                    .nominals
                    .declarations
                    .get(&value.declaration)
                    .copied()
                    .ok_or_else(|| {
                        located(
                            self.graph,
                            file,
                            Diagnostic::new(
                                expression.span,
                                "module enum parameter has no resolved nominal declaration",
                            ),
                        )
                    })?;
                if actual != ty {
                    return Err(located(
                        self.graph,
                        file,
                        Diagnostic::new(
                            expression.span,
                            "module enum parameter has a different nominal type",
                        ),
                    ));
                }
                return Ok(TypedConstant {
                    ty,
                    kind: ConstantKind::Enum(value.value),
                });
            }
            if matches!(kind, TypeKind::String)
                && let Ok(jai_modules::Binding::Parameter(id)) = self.graph.lookup(file, &path)
                && let jai_modules::ParameterValue::String(bytes) = &self
                    .graph
                    .parameter(id)
                    .expect("parameter identity exists")
                    .value
            {
                self.charge(file, expression.span, bytes.len())?;
                return Ok(TypedConstant {
                    ty,
                    kind: ConstantKind::StringBytes(bytes.as_bytes().to_vec()),
                });
            }
            if let Ok(id) = declaration_id(self.graph, file, &path, expression.span) {
                if let Some(value) = self.named.get(&id) {
                    let cells = crate::constant_limits::cells(value).ok_or_else(|| {
                        located(
                            self.graph,
                            file,
                            Diagnostic::new(
                                expression.span,
                                "constant exceeds compiler constant cell budget",
                            ),
                        )
                    })?;
                    self.charge(file, expression.span, cells)?;
                    return self
                        .coerce_field_constant(self.named[&id].clone(), ty, expression.span)
                        .map_err(|error| located(self.graph, file, error));
                }
                let declaration = self.graph.declaration(id).expect("declaration exists");
                if crate::modules::sequence_constants::is_sequence_constant(self.graph, declaration)
                    && let FileDeclarationKind::Constant(constant) = &declaration.syntax().kind
                {
                    return self.expression(declaration.file(), &constant.initializer, ty);
                }
            }
            if let Some(member) = self
                .nominals
                .enum_member(self.graph, file, &path, expression.span)
                .map_err(|error| located(self.graph, file, error))?
            {
                if member.ty != ty {
                    return Err(located(
                        self.graph,
                        file,
                        Diagnostic::new(
                            expression.span,
                            "enum constant has a different nominal type",
                        ),
                    ));
                }
                return Ok(TypedConstant {
                    ty,
                    kind: ConstantKind::Enum(member.value),
                });
            }
        }
        let value = self.scalar(file, expression)?;
        let kind = match self.types.kind(ty).map_err(|error| {
            located(
                self.graph,
                file,
                Diagnostic::new(expression.span, error.to_string()),
            )
        })? {
            TypeKind::Integer(integer) => match value
                .coerce(ScalarType::Int(*integer), expression.span)
                .map_err(|error| located(self.graph, file, error))?
            {
                ConstantValue::Int(value) => ConstantKind::Int(value),
                _ => unreachable!(),
            },
            TypeKind::Bool => match value
                .coerce(ScalarType::Bool, expression.span)
                .map_err(|error| located(self.graph, file, error))?
            {
                ConstantValue::Bool(value) => ConstantKind::Bool(value),
                _ => unreachable!(),
            },
            TypeKind::Enum(_)
                if matches!(value, ConstantValue::Literal(0))
                    && self
                        .specializations
                        .and_then(|records| records.member_enum(ty))
                        .map(|enumeration| enumeration.flags)
                        .or_else(|| self.nominals.enums.get(&ty).map(|enum_| enum_.flags))
                        .unwrap_or(false) =>
            {
                let representation = self
                    .types
                    .enum_definition(ty)
                    .map_err(|error| {
                        located(
                            self.graph,
                            file,
                            Diagnostic::new(expression.span, error.to_string()),
                        )
                    })?
                    .representation;
                ConstantKind::Enum(IntegerValue::wrapping(representation, 0))
            }
            _ => {
                return Err(located(
                    self.graph,
                    file,
                    Diagnostic::new(
                        expression.span,
                        "constant expression does not match its nominal type",
                    ),
                ));
            }
        };
        Ok(TypedConstant { ty, kind })
    }
}
