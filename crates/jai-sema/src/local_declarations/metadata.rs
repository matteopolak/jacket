//! Aggregate operations consult local and file metadata through one overlay.
use super::*;
use jai_types::TypeView;

impl LocalDeclarationRegistry {
    pub(crate) fn enum_member_value(&self, ty: TypeId, name: Symbol) -> Option<Integer> {
        self.enumerations.get(&ty)?.members.get(&name).copied()
    }
    pub(crate) fn enum_is_flags(&self, ty: TypeId) -> Option<bool> {
        self.enumerations
            .get(&ty)
            .map(|enumeration| enumeration.flags)
    }
}

impl Resolver<'_> {
    pub(crate) fn record_metadata(
        &self,
        ty: TypeId,
        span: Span,
    ) -> Result<RecordMetadata, Diagnostic> {
        if let Some(schema) = self
            .context
            .filter(|schema| schema.definition.record_type == ty)
        {
            return Ok(schema.record_metadata());
        }
        if let Some(record) = self.meta.record_specializations.record(ty) {
            return Ok(record.shape.clone());
        }
        if let Some(record) = self.meta.local_declarations.records.get(&ty) {
            return Ok(record.clone());
        }
        let record = self
            .graph_scope
            .ok_or_else(|| Diagnostic::new(span, "record declaration metadata is unavailable"))?
            .record(ty, span)?;
        Ok(RecordMetadata {
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
        })
    }

    pub(crate) fn field_path(
        &self,
        ty: TypeId,
        name: Symbol,
        span: Span,
    ) -> Result<Vec<FieldId>, Diagnostic> {
        self.optional_field_path(ty, name, span)?
            .ok_or_else(|| Diagnostic::new(span, "unknown record member"))
    }

    pub(crate) fn optional_field_path(
        &self,
        ty: TypeId,
        name: Symbol,
        span: Span,
    ) -> Result<Option<Vec<FieldId>>, Diagnostic> {
        if let Some(path) = self.reflection_field_path(ty, name, span)? {
            return Ok(Some(path));
        }
        let mut pending = vec![(ty, Vec::new(), HashSet::new())];
        let mut found = None;
        while let Some((ty, prefix, mut ancestors)) = pending.pop() {
            if !ancestors.insert(ty) {
                return Err(Diagnostic::new(span, "cyclic using field promotion"));
            }
            let record = self.record_metadata(ty, span)?;
            if let Some(field) = record.fields.iter().find(|field| field.name == Some(name)) {
                let mut path = prefix.clone();
                path.push(field.id);
                if found.replace(path).is_some() {
                    return Err(Diagnostic::new(span, "ambiguous promoted record member"));
                }
            }
            for field in record
                .fields
                .iter()
                .rev()
                .filter(|field| field.syntax.using())
            {
                let mut path = prefix.clone();
                path.push(field.id);
                pending.push((field.ty, path, ancestors.clone()));
            }
        }
        Ok(found)
    }

    pub(crate) fn field_default_value(
        &self,
        id: FieldId,
        span: Span,
    ) -> Result<jai_ir::ConstantValue, Diagnostic> {
        let owner = self
            .types
            .record_type(id.record())
            .map_err(|error| Diagnostic::new(span, error.to_string()))?;
        self.require_local_record_initializer(owner, span)?;
        self.field_initial_value(id, span)
    }

    pub(crate) fn field_construction_overlay(
        &self,
        id: FieldId,
        span: Span,
    ) -> Result<Option<jai_ir::ConstantValue>, Diagnostic> {
        let owner = self
            .types
            .record_type(id.record())
            .map_err(|error| Diagnostic::new(span, error.to_string()))?;
        self.require_local_record_initializer(owner, span)?;
        if self
            .context
            .is_some_and(|schema| schema.definition.record_type == owner)
            || self.meta.local_declarations.defaults.contains_key(&id)
            || !self
                .meta
                .record_specializations
                .default_overrides(id)
                .is_empty()
        {
            return self.field_default_value(id, span).map(Some);
        }
        let shape = self.record_metadata(owner, span)?;
        let field = shape
            .fields
            .iter()
            .find(|field| field.id == id)
            .ok_or_else(|| {
                Diagnostic::new(span, "record construction field metadata is unavailable")
            })?;
        let actual = self
            .types
            .validate_field(owner, id)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?;
        if actual != field.ty {
            return Err(Diagnostic::new(
                span,
                "record construction field differs from its canonical type",
            ));
        }
        if field.syntax.initializer().is_some() {
            self.field_default_value(id, span).map(Some)
        } else {
            Ok(None)
        }
    }

    pub(super) fn field_initial_value(
        &self,
        id: FieldId,
        span: Span,
    ) -> Result<jai_ir::ConstantValue, Diagnostic> {
        if let Some(value) = self.checked_source_field_default(id) {
            return value;
        }
        let owner = self
            .types
            .record_type(id.record())
            .map_err(|error| Diagnostic::new(span, error.to_string()))?;
        if let Some(schema) = self
            .context
            .filter(|schema| schema.definition.record_type == owner)
        {
            let ty = self
                .types
                .validate_field(owner, id)
                .map_err(|error| Diagnostic::new(span, error.to_string()))?;
            let ConstantKind::Record(defaults) = &schema.definition.default.kind else {
                return Err(Diagnostic::new(
                    span,
                    "context schema default is not a record initializer",
                ));
            };
            let value = defaults
                .get(id.index())
                .ok_or_else(|| Diagnostic::new(span, "context field default is unavailable"))?;
            if value.ty != ty {
                return Err(Diagnostic::new(
                    span,
                    "context field default differs from its canonical field type",
                ));
            }
            return Ok(value.clone());
        }
        if let Some(scope) = self.graph_scope
            && self.meta.record_specializations.field(id).is_some()
        {
            return scope.specialized_field_default(
                id,
                self.types,
                &self.meta.record_specializations,
                span,
            );
        }
        if let Some(value) = self.meta.local_declarations.defaults.get(&id) {
            return Ok(value.clone());
        }
        if let Some(field) = self
            .meta
            .local_declarations
            .records
            .get(&owner)
            .and_then(|record| record.fields.get(id.index()))
        {
            self.require_local_field_default(field)?;
            if field.syntax.is_anonymous() {
                return self.field_type_default_value(field);
            }
        }
        if let Some(scope) = self.graph_scope
            && let Ok(value) = scope.field_default(id, self.types, span)
        {
            return Ok(value);
        }
        let ty = self
            .types
            .field_type(id)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?;
        self.default_value(ty, span)
    }

    fn checked_source_field_default(
        &self,
        id: FieldId,
    ) -> Option<Result<jai_ir::ConstantValue, Diagnostic>> {
        use crate::modules::field_default_jobs::FieldDefaultReadiness;
        if let Some(readiness) = self.meta.field_default_jobs.readiness(id) {
            return Some(match readiness {
                FieldDefaultReadiness::NoWrite(job) => Err(Diagnostic::at_source(
                    job.location,
                    "record construction with no-write field defaults requires sparse initialization support",
                )),
                FieldDefaultReadiness::Ready(value) => Ok(value.clone()),
                FieldDefaultReadiness::Failed(error) => {
                    Err(Diagnostic::at_source(error.location, error.message.clone()))
                }
                FieldDefaultReadiness::Pending(job) => {
                    if let Some(context) = self.compile_time {
                        let mut pending = context.pending_field_defaults.borrow_mut();
                        if !pending.contains(&id) {
                            pending.push(id);
                        }
                    }
                    Err(Diagnostic::at_source(
                        job.location,
                        "record field default is not ready",
                    ))
                }
            });
        }
        if let Some((record, field)) = self.meta.record_specializations.field(id)
            && field
                .syntax
                .initializer()
                .is_some_and(crate::modules::field_default_jobs::contains_typed_leaf)
        {
            if let Some(context) = self.compile_time {
                let mut pending = context.pending_field_defaults.borrow_mut();
                if !pending.contains(&id) {
                    pending.push(id);
                }
            }
            let source = self
                .graph_scope
                .expect("source record field has a file scope")
                .code_file(record.file)
                .source();
            return Some(Err(Diagnostic::at_source(
                jai_source::SourceSpan {
                    source,
                    span: field.syntax.span(),
                },
                "record field default is awaiting typed initializer preparation",
            )));
        }
        None
    }

    pub(crate) fn field_type_default_value(
        &self,
        field: &FieldMetadata,
    ) -> Result<jai_ir::ConstantValue, Diagnostic> {
        let mut remaining = crate::constant_limits::MAX_CONSTANT_CELLS;
        self.lexical_default_inner(
            field.ty,
            field.syntax.span(),
            &mut HashSet::new(),
            &mut remaining,
            field.syntax.is_anonymous(),
        )
    }

    pub(crate) fn default_value(
        &self,
        ty: TypeId,
        span: Span,
    ) -> Result<jai_ir::ConstantValue, Diagnostic> {
        if self
            .meta
            .local_declarations
            .active_record_overrides
            .contains(&ty)
        {
            return Err(Diagnostic::new(
                span,
                "cyclic record default override initializer",
            ));
        }
        self.require_local_record_initializer(ty, span)?;
        if let Some(schema) = self.context
            && schema.definition.record_type == ty
        {
            return Ok(schema.definition.default.clone());
        }
        let mut remaining = crate::constant_limits::MAX_CONSTANT_CELLS;
        self.lexical_default_inner(ty, span, &mut HashSet::new(), &mut remaining, false)
    }

    fn lexical_default_inner(
        &self,
        ty: TypeId,
        span: Span,
        active: &mut HashSet<TypeId>,
        remaining: &mut usize,
        anonymous_member: bool,
    ) -> Result<jai_ir::ConstantValue, Diagnostic> {
        if self
            .meta
            .local_declarations
            .active_record_overrides
            .contains(&ty)
        {
            return Err(Diagnostic::new(
                span,
                "cyclic record default override initializer",
            ));
        }
        self.require_local_record_initializer(ty, span)?;
        if let Some(schema) = self
            .context
            .filter(|schema| schema.definition.record_type == ty)
        {
            let value = &schema.definition.default;
            let charge = crate::constant_limits::cells(value).ok_or_else(|| {
                Diagnostic::new(span, "constant exceeds compiler constant cell budget")
            })?;
            *remaining = remaining.checked_sub(charge).ok_or_else(|| {
                Diagnostic::new(span, "constant exceeds compiler constant cell budget")
            })?;
            return Ok(value.clone());
        }
        crate::record_placements::require_record_construction_recipe(self.types, ty, span)?;
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
        let kind = match self
            .types
            .kind(ty)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?
        {
            TypeKind::Integer(integer) => ConstantKind::Int(IntegerValue::wrapping(*integer, 0)),
            TypeKind::Bool => ConstantKind::Bool(false),
            TypeKind::Float(float) => ConstantKind::Float(match float {
                jai_types::FloatType::F32 => jai_types::FloatValue::F32(0),
                jai_types::FloatType::F64 => jai_types::FloatValue::F64(0),
            }),
            TypeKind::String => ConstantKind::StringBytes(vec![]),
            TypeKind::Enum(_) => {
                let enumeration = self
                    .types
                    .enum_definition(ty)
                    .map_err(|error| Diagnostic::new(span, error.to_string()))?;
                ConstantKind::Enum(if self.enum_is_flags(ty) {
                    Integer::wrapping(enumeration.representation, 0)
                } else {
                    enumeration
                        .values
                        .first()
                        .copied()
                        .unwrap_or_else(|| Integer::wrapping(enumeration.representation, 0))
                })
            }
            TypeKind::Distinct(_) => {
                let base = self
                    .types
                    .distinct_definition(ty)
                    .map_err(|error| Diagnostic::new(span, error.to_string()))?
                    .representation;
                ConstantKind::Distinct(Box::new(
                    self.lexical_default_inner(base, span, active, remaining, false)?,
                ))
            }
            TypeKind::FixedArray {
                element,
                count,
            } => {
                if *count == 0 {
                    ConstantKind::Zero
                } else {
                    let value =
                        self.lexical_default_inner(*element, span, active, remaining, false)?;
                    if crate::constant_limits::is_zero(&value) {
                        ConstantKind::Zero
                    } else {
                        let count = usize::try_from(*count).map_err(|_| {
                            Diagnostic::new(
                                span,
                                "array default count exceeds host addressable limits",
                            )
                        })?;
                        let charge = crate::constant_limits::cells(&value)
                            .and_then(|cells| cells.checked_mul(count.saturating_sub(1)))
                            .ok_or_else(|| {
                                Diagnostic::new(
                                    span,
                                    "array default exceeds compiler constant cell budget",
                                )
                            })?;
                        *remaining = remaining.checked_sub(charge).ok_or_else(|| {
                            Diagnostic::new(
                                span,
                                "array default exceeds compiler constant cell budget",
                            )
                        })?;
                        ConstantKind::Array(vec![value; count])
                    }
                }
            }
            TypeKind::Any(_) => ConstantKind::Zero,
            TypeKind::Record(_) => {
                let record = self.record_metadata(ty, span)?;
                let union = record.kind == RecordKind::Union;
                if union && !anonymous_member {
                    return Err(Diagnostic::new(
                        span,
                        "union default initialization requires an explicit active alternative",
                    ));
                }
                let mut values = Vec::new();
                for field in record.fields {
                    if let Some(value) = self.checked_source_field_default(field.id) {
                        let value = value?;
                        let charge = crate::constant_limits::cells(&value).ok_or_else(|| {
                            Diagnostic::new(span, "constant exceeds compiler constant cell budget")
                        })?;
                        *remaining = remaining.checked_sub(charge).ok_or_else(|| {
                            Diagnostic::new(span, "constant exceeds compiler constant cell budget")
                        })?;
                        values.push(value);
                        continue;
                    }
                    if let Some(scope) = self.graph_scope
                        && self.meta.record_specializations.field(field.id).is_some()
                    {
                        let value = scope.specialized_field_default(
                            field.id,
                            self.types,
                            &self.meta.record_specializations,
                            span,
                        )?;
                        let charge = crate::constant_limits::cells(&value).ok_or_else(|| {
                            Diagnostic::new(span, "constant exceeds compiler constant cell budget")
                        })?;
                        *remaining = remaining.checked_sub(charge).ok_or_else(|| {
                            Diagnostic::new(span, "constant exceeds compiler constant cell budget")
                        })?;
                        values.push(value);
                        continue;
                    }
                    if let Some(value) = self.meta.local_declarations.defaults.get(&field.id) {
                        let charge = crate::constant_limits::cells(value).ok_or_else(|| {
                            Diagnostic::new(span, "constant exceeds compiler constant cell budget")
                        })?;
                        *remaining = remaining.checked_sub(charge).ok_or_else(|| {
                            Diagnostic::new(span, "constant exceeds compiler constant cell budget")
                        })?;
                        values.push(value.clone());
                    } else if let Some(scope) = self.graph_scope
                        && !self.meta.local_declarations.records.contains_key(&ty)
                    {
                        let value = scope.field_default(field.id, self.types, span)?;
                        let charge = crate::constant_limits::cells(&value).ok_or_else(|| {
                            Diagnostic::new(span, "constant exceeds compiler constant cell budget")
                        })?;
                        *remaining = remaining.checked_sub(charge).ok_or_else(|| {
                            Diagnostic::new(span, "constant exceeds compiler constant cell budget")
                        })?;
                        values.push(value);
                    } else {
                        self.require_local_field_default(&field)?;
                        values.push(self.lexical_default_inner(
                            field.ty,
                            span,
                            active,
                            remaining,
                            field.syntax.is_anonymous(),
                        )?);
                    }
                }
                if union {
                    if !values.iter().all(crate::constant_limits::is_zero) {
                        return Err(Diagnostic::new(
                            span,
                            "anonymous union defaults require identical zero storage or an explicit active alternative",
                        ));
                    }
                    ConstantKind::Zero
                } else {
                    ConstantKind::Record(values)
                }
            }
            TypeKind::Type
            | TypeKind::Pointer(_)
            | TypeKind::Slice(_)
            | TypeKind::DynamicArray(_)
            | TypeKind::Procedure(_) => ConstantKind::Zero,
            _ => {
                return Err(Diagnostic::new(
                    span,
                    "type cannot be default initialized into runtime storage",
                ));
            }
        };
        active.remove(&ty);
        Ok(jai_ir::ConstantValue {
            ty,
            kind,
        })
    }

    fn require_local_record_initializer(&self, ty: TypeId, span: Span) -> Result<(), Diagnostic> {
        if let Some(field) = self.local_no_write_field(ty) {
            return Err(Diagnostic::new(
                field.syntax.initializer().unwrap().span,
                "record construction with an uninitialized field default is not supported",
            ));
        }
        let registry = &self.meta.local_declarations;
        if registry.records.contains_key(&ty)
            && registry.namespace_scopes.contains_key(&ty)
            && !registry.record_overrides_ready.contains(&ty)
        {
            return Err(Diagnostic::new(
                span,
                if registry.active_record_overrides.contains(&ty) {
                    "cyclic record default override initializer"
                } else {
                    "record initializer is pending its completed field defaults and construction overrides"
                },
            ));
        }
        Ok(())
    }

    fn require_local_field_default(&self, field: &FieldMetadata) -> Result<(), Diagnostic> {
        let initializer = field.syntax.initializer();
        if let Some(expression) = initializer {
            return Err(Diagnostic::new(
                expression.span,
                if matches!(expression.kind, syntax::ExpressionKind::Uninitialized) {
                    "record construction with an uninitialized field default is not supported"
                } else {
                    "local record field initializer is not ready; cyclic initializer dependencies cannot supply a default"
                },
            ));
        }
        Ok(())
    }

    pub(super) fn local_no_write_field(&self, ty: TypeId) -> Option<&FieldMetadata> {
        self.meta
            .local_declarations
            .records
            .get(&ty)?
            .fields
            .iter()
            .find(|field| {
                field.syntax.initializer().is_some_and(|expression| {
                    matches!(expression.kind, syntax::ExpressionKind::Uninitialized)
                })
            })
    }

    pub(crate) fn local_enum_member(
        &mut self,
        path: &syntax::NamePath,
        span: Span,
    ) -> Result<Option<modules::aggregates::EnumConstant>, Diagnostic> {
        let Some(binding) = self.resolve_local_name(path.root, span)? else {
            return Ok(None);
        };
        let Binding::Type(ty) = binding else {
            return Ok(None);
        };
        if path.members.len() != 1 {
            return Ok(None);
        }
        let Some(enumeration) = self.meta.local_declarations.enumerations.get(&ty) else {
            return Ok(None);
        };
        let value = enumeration
            .members
            .get(&path.members[0])
            .copied()
            .ok_or_else(|| Diagnostic::new(span, "unknown enum member"))?;
        Ok(Some(modules::aggregates::EnumConstant {
            ty,
            value,
        }))
    }
}
