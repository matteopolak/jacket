//! Declaration-site literal constants use canonical paths and selected defaults.
use super::*;
use crate::modules::aggregates::promoted_literals::{self, FieldDefaults, PreparedLiteral};
use jai_types::TypeView;

impl Defaults<'_, '_> {
    pub(super) fn promoted_record_constant(
        &mut self,
        file: FileInstanceId,
        literal: &syntax::StructLiteral,
        ty: TypeId,
        span: Span,
    ) -> Result<TypedConstant, LocatedDiagnostic> {
        let mut leaves = Vec::with_capacity(literal.fields.len());
        let mut targets = Vec::with_capacity(literal.fields.len());
        for source in &literal.fields {
            let path = crate::record_default_overrides::find_field_path(
                ty,
                source.name,
                source.span,
                self.types,
                |ty| {
                    self.record(ty).map(|record| record.shape).ok_or_else(|| {
                        Diagnostic::new(source.span, "record literal requires a source schema")
                    })
                },
            )
            .map_err(|error| located(self.graph, file, error))?;
            let mut target = ty;
            for &field in &path {
                target = self.types.validate_field(target, field).map_err(|error| {
                    located(
                        self.graph,
                        file,
                        Diagnostic::new(source.span, error.to_string()),
                    )
                })?;
            }
            targets.push(target);
            leaves.push((path, target));
        }
        let plan = PreparedLiteral::new(self.types, ty, leaves).map_err(|error| {
            located(
                self.graph,
                file,
                promoted_literals::path_diagnostic(error, literal, span),
            )
        })?;
        let mut values = Vec::with_capacity(literal.fields.len());
        for (source, target) in literal.fields.iter().zip(targets) {
            values.push(self.expression(file, &source.value, target)?);
        }
        let types = self.types;
        plan.compose_constants(types, values, self)
            .map_err(
                |error| match promoted_literals::build_diagnostic(error, span) {
                    Ok(error) => located(self.graph, file, error),
                    Err(error) => error,
                },
            )
    }
}

impl FieldDefaults for Defaults<'_, '_> {
    type Error = LocatedDiagnostic;
    fn complete(&mut self, field: FieldId) -> Result<TypedConstant, LocatedDiagnostic> {
        self.field(field)
    }
    fn partial(&mut self, field: FieldId) -> Result<Option<TypedConstant>, LocatedDiagnostic> {
        let owner = self
            .types
            .record_type(field.record())
            .expect("promoted literal field owner passed canonical path proof");
        let Some(record) = self.record(owner) else {
            unreachable!("canonical promoted literal path retains source record metadata");
        };
        let metadata = record
            .shape
            .fields
            .iter()
            .find(|metadata| metadata.id == field)
            .expect("canonical promoted literal path retains actual field metadata");
        let explicit = metadata.syntax.initializer().is_some()
            || self
                .specializations
                .is_some_and(|records| !records.default_overrides(field).is_empty())
            || self
                .context
                .is_some_and(|schema| schema.definition.record_type == owner);
        if explicit {
            self.field(field).map(Some)
        } else {
            Ok(None)
        }
    }
}
