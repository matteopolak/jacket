//! Staged indexed declaration-site constants; activate with source literal targets.
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
        if literal.fields.len() > crate::constant_limits::MAX_CONSTANT_CELLS {
            return Err(located(
                self.graph,
                file,
                Diagnostic::new(span, "literal initializer exceeds compiler cell budget"),
            ));
        }
        let mut leaves = Vec::with_capacity(literal.fields.len());
        let mut targets = Vec::with_capacity(literal.fields.len());
        let mut path_cells = 0usize;
        for source in &literal.fields {
            let path = promoted_literals::indexed_source::resolve(
                ty,
                &source.target,
                self.types,
                |owner, name, source_span| {
                    crate::record_default_overrides::find_field_path(
                        owner,
                        name,
                        source_span,
                        self.types,
                        |owner| {
                            self.record(owner)
                                .map(|record| record.shape)
                                .ok_or_else(|| {
                                    Diagnostic::new(
                                        source_span,
                                        "literal path requires a ready source schema",
                                    )
                                })
                        },
                    )
                },
                |index| self.ready_literal_index(file, index),
            )
            .map_err(|error| located(self.graph, file, error))?;
            path_cells = path_cells.saturating_add(path.len());
            if path_cells > crate::constant_limits::MAX_CONSTANT_CELLS {
                return Err(located(
                    self.graph,
                    file,
                    Diagnostic::new(source.span, "literal paths exceed compiler cell budget"),
                ));
            }
            let target = path
                .iter()
                .try_fold(ty, |owner, step| match *step {
                    promoted_literals::PathStep::Field(field) => self
                        .types
                        .validate_field(owner, field)
                        .map_err(|error| Diagnostic::new(source.span, error.to_string())),
                    promoted_literals::PathStep::Element {
                        owner: array, ..
                    } => {
                        let TypeKind::FixedArray {
                            element, ..
                        } = *self
                            .types
                            .kind(array)
                            .map_err(|error| Diagnostic::new(source.span, error.to_string()))?
                        else {
                            return Err(Diagnostic::new(
                                source.span,
                                "checked literal path is not a fixed array",
                            ));
                        };
                        Ok(element)
                    }
                })
                .map_err(|error| located(self.graph, file, error))?;
            targets.push(target);
            leaves.push((path, target));
        }
        let plan = PreparedLiteral::with_paths(self.types, ty, leaves).map_err(|error| {
            located(
                self.graph,
                file,
                promoted_literals::path_diagnostic_at(
                    error,
                    &literal
                        .fields
                        .iter()
                        .map(|field| field.span)
                        .collect::<Vec<_>>(),
                    span,
                ),
            )
        })?;
        let types = self.types;
        plan.validate_defaults(types, self).map_err(|error| {
            match promoted_literals::build_diagnostic(error, span) {
                Ok(error) => located(self.graph, file, error),
                Err(error) => error,
            }
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

impl Defaults<'_, '_> {
    // The pure evaluator binds actual ready facts and never invokes ScalarSource::Evaluate.
    fn ready_literal_index(
        &self,
        file: FileInstanceId,
        source: &Expression,
    ) -> Result<Option<jai_types::Integer>, Diagnostic> {
        let value = match &self.scalar_source {
            ScalarSource::Constants(constants) => constants
                .evaluate(file, source)
                .map_err(|error| Diagnostic::at_source(error.location, error.message))?,
            ScalarSource::Evaluate(_) => jai_eval::evaluate_paths_with_overflow_check(
                source,
                jai_types::CheckMode::Enabled,
                |path, span| {
                    if path.members.is_empty()
                        && let Some(value) = self
                            .substitution
                            .as_ref()
                            .and_then(|scope| scope.constant(path.root))
                    {
                        return baked_scalar(value).ok_or_else(|| {
                            Diagnostic::new(span, "literal index requires an integer constant")
                        });
                    }
                    let id = declaration_id(self.graph, file, path, span)?;
                    let value = self
                        .named
                        .get(&id)
                        .or_else(|| self.nominals.value_constants.get(&id))
                        .ok_or_else(|| {
                            Diagnostic::new(
                                span,
                                "literal index requires an already checked constant",
                            )
                        })?;
                    match &value.kind {
                        ConstantKind::Int(value) => Ok(ScalarConstant::Int(*value)),
                        _ => Err(Diagnostic::new(
                            span,
                            "literal index requires an integer constant",
                        )),
                    }
                },
            )?,
        };
        match value {
            ScalarConstant::Int(value) => Ok(Some(value)),
            ScalarConstant::Literal(value) => {
                let ty = if value < 0 {
                    jai_types::IntegerType::S64
                } else {
                    jai_types::IntegerType::U64
                };
                jai_types::Integer::checked(ty, value)
                    .map(Some)
                    .ok_or_else(|| {
                        Diagnostic::new(
                            source.span,
                            "literal index integer is outside the supported range",
                        )
                    })
            }
            _ => Err(Diagnostic::new(
                source.span,
                "literal index requires an integer constant",
            )),
        }
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
    fn array_element(
        &mut self,
        field: FieldId,
        ty: TypeId,
    ) -> Result<TypedConstant, LocatedDiagnostic> {
        let owner = self
            .types
            .record_type(field.record())
            .expect("declaring array field passed canonical path validation");
        let record = self
            .record(owner)
            .expect("declaring array field retains original source metadata");
        let source_span = record
            .shape
            .fields
            .iter()
            .find(|metadata| metadata.id == field)
            .expect("array context identifies the original field")
            .syntax
            .span();
        let previous = std::mem::replace(&mut self.substitution, record.substitution);
        let value = self.default_value(record.file, ty, source_span);
        self.substitution = previous;
        value
    }
}
