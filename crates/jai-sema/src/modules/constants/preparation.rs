//! Retain scalar source dependencies without turning rendered text into control flow.
use super::*;
use crate::modules::aggregates::parameterized::PendingType;

pub(crate) enum ScalarPreparation {
    Ready(ScalarConstant),
    Pending(PendingType),
}

impl Constants<'_> {
    pub(in crate::modules) fn prepare_value(
        &mut self,
        id: DeclarationId,
        site: SourceSpan,
    ) -> Result<ScalarPreparation, LocatedDiagnostic> {
        match self.states.get(&id) {
            Some(ConstantState::Ready(value)) => {
                return Ok(ScalarPreparation::Ready(value.clone()));
            }
            Some(ConstantState::Visiting) => {
                return self.ready(id, site).map(ScalarPreparation::Ready);
            }
            None => {}
        }
        if self
            .states
            .values()
            .filter(|state| matches!(state, ConstantState::Visiting))
            .count()
            >= 256
        {
            return Err(LocatedDiagnostic {
                location: site,
                message: "scalar preparation exceeds constant dependency depth limit".into(),
            });
        }
        let declaration = self
            .graph
            .declaration(id)
            .expect("resolved declaration exists");
        let FileDeclarationKind::Constant(constant) = &declaration.syntax().kind else {
            return self.ready(id, site).map(ScalarPreparation::Ready);
        };
        let file = declaration.file();
        let constant = constant.clone();
        let target = self
            .primitive_annotations
            .get(&id)
            .copied()
            .or(match constant.ty.as_ref() {
                Some(syntax::TypeSyntax::Builtin(syntax::BuiltinType::Scalar(ty))) => {
                    Some(PrimitiveAnnotation::Scalar(*ty))
                }
                Some(syntax::TypeSyntax::Builtin(syntax::BuiltinType::Float(ty))) => {
                    Some(PrimitiveAnnotation::Float(*ty))
                }
                _ => None,
            });
        if target.is_none() && constant.ty.is_some() {
            if self.checked_typed_values.contains_key(&id) {
                return Err(LocatedDiagnostic {
                    location: site,
                    message: "a checked nominal or aggregate constant cannot supply a scalar value"
                        .into(),
                });
            }
            return Ok(ScalarPreparation::Pending(PendingType::Constant {
                declaration: id,
                location: site,
            }));
        }
        self.states.insert(id, ConstantState::Visiting);
        let result = self.prepare_expression(file, &constant.initializer, target, Some((id, site))).and_then(|result| {
            match result {
                ScalarPreparation::Pending(cause) => Ok(ScalarPreparation::Pending(cause)),
                ScalarPreparation::Ready(value) => {
                    let value = match target {
                        Some(PrimitiveAnnotation::Scalar(ty)) => value.coerce(ty, constant.span)
                            .map_err(|error| located(self.graph, file, error))?,
                        Some(PrimitiveAnnotation::Float(_)) => value,
                        None if constant.ty.is_none() => value,
                        None => return Err(located(self.graph, file, Diagnostic::new(constant.span,
                            "typed constant annotation requires canonical semantic resolution"))),
                    };
                    Ok(ScalarPreparation::Ready(value))
                }
            }
        });
        match &result {
            Ok(ScalarPreparation::Ready(value)) => {
                self.states.insert(id, ConstantState::Ready(value.clone()));
            }
            _ => {
                self.states.remove(&id);
            }
        }
        result
    }

    pub(in crate::modules) fn register_checked_typed_value(
        &mut self,
        id: DeclarationId,
        ty: TypeId,
    ) -> Result<(), LocatedDiagnostic> {
        let source = self
            .graph
            .declaration(id)
            .expect("actual typed constant producer");
        if self.annotations.get(&id).copied() != Some(ty) {
            return Err(LocatedDiagnostic {
                location: source.location(),
                message: "checked typed constant differs from its canonical source annotation"
                    .into(),
            });
        }
        self.checked_typed_values.insert(id, ty);
        Ok(())
    }

    pub(in crate::modules) fn register_ready_value(
        &mut self,
        id: DeclarationId,
        value: ScalarConstant,
    ) -> Result<(), LocatedDiagnostic> {
        let declaration = self
            .graph
            .declaration(id)
            .expect("ready scalar origin belongs to graph");
        assert!(matches!(
            declaration.syntax().kind,
            FileDeclarationKind::Constant(_)
        ));
        let value = value.with_fallback_source(declaration.location().source);
        if let Some(ConstantState::Ready(previous)) = self.states.get(&id)
            && previous != &value
        {
            return Err(LocatedDiagnostic {
                location: declaration.location(),
                message: "checked scalar completion changed an already ready constant".into(),
            });
        }
        self.states.insert(id, ConstantState::Ready(value));
        Ok(())
    }

    pub(in crate::modules) fn prepare_evaluate_lazy(
        &mut self,
        file: FileInstanceId,
        expression: &Expression,
    ) -> Result<ScalarPreparation, LocatedDiagnostic> {
        self.prepare_expression(file, expression, None, None)
    }

    fn prepare_expression(
        &mut self,
        file: FileInstanceId,
        expression: &Expression,
        target: Option<PrimitiveAnnotation>,
        typed_origin: Option<(DeclarationId, SourceSpan)>,
    ) -> Result<ScalarPreparation, LocatedDiagnostic> {
        let inference = match self.prepare_expression_domain(
            file,
            expression,
            &mut std::collections::HashSet::new(),
            typed_origin,
        )? {
            DomainPreparation::Ready(inference) => inference,
            DomainPreparation::Pending(cause) => return Ok(ScalarPreparation::Pending(cause)),
        };
        let graph = self.graph;
        let mut pending = None;
        let mut lookup =
            |path: &NamePath, span: Span| match self.prepare_scalar_path(file, path, span) {
                Ok(ScalarPreparation::Ready(value)) => Ok(value),
                Ok(ScalarPreparation::Pending(cause)) => {
                    pending.get_or_insert(cause);
                    let diagnostic = cause.diagnostic(graph);
                    Err(Diagnostic::at_source(
                        diagnostic.location,
                        diagnostic.message,
                    ))
                }
                Err(error) => Err(Diagnostic::at_source(error.location, error.message)),
            };
        let result = match target {
            Some(PrimitiveAnnotation::Float(float)) => inference
                .evaluate_float_paths(float, jai_types::CheckMode::Enabled, &mut lookup)
                .map(ConstantValue::Float),
            _ => inference.evaluate_paths(jai_types::CheckMode::Enabled, &mut lookup),
        };
        match pending {
            Some(cause) => Ok(ScalarPreparation::Pending(cause)),
            None => result
                .map(|value| {
                    ScalarPreparation::Ready(
                        value.with_fallback_source(graph.file(file).unwrap().source()),
                    )
                })
                .map_err(|error| located(graph, file, error)),
        }
    }

    fn prepare_scalar_path(
        &mut self,
        file: FileInstanceId,
        path: &NamePath,
        span: Span,
    ) -> Result<ScalarPreparation, LocatedDiagnostic> {
        let site = SourceSpan {
            source: self.graph.file(file).unwrap().source(),
            span,
        };
        if let Err(error) = self.graph.lookup(file, path)
            && let Ok(pending) = PendingType::from_lookup(error, site)
        {
            return Ok(ScalarPreparation::Pending(pending));
        }
        if let Some(value) = self.external_value(file, path, span)? {
            return Ok(ScalarPreparation::Ready(value));
        }
        let id = declaration_id(self.graph, file, path, span)
            .map_err(|error| located(self.graph, file, error))?;
        self.prepare_value(id, site)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_modules::{GraphOptions, SourceOverlay};

    fn graph(source: &str) -> ModuleGraph {
        let path = std::path::Path::new("/jai-scalar-preparation/main.jai");
        let mut overlay = SourceOverlay::new();
        overlay.insert(path, source.as_bytes().to_vec()).unwrap();
        ModuleGraph::load_with_provider(path, GraphOptions::default(), &overlay).unwrap()
    }

    #[test]
    fn nested_placeholder_keeps_its_real_identity_and_rolls_back_unfinished_visits() {
        let text = "#placeholder Later; INNER::Later+1; OUTER::INNER+1; READY::42;";
        let graph = graph(text);
        let mut constants = Constants::new(&graph);
        let outer = &graph.declarations()[1];
        for _ in 0..2 {
            let result = constants
                .prepare_value(outer.id(), outer.location())
                .unwrap();
            let ScalarPreparation::Pending(PendingType::Placeholder(demand)) = result else {
                panic!("actual source lookup must retain the placeholder dependency");
            };
            assert_eq!(demand.placeholder, graph.placeholders()[0].id());
            assert_eq!(demand.location.span.text(text), "Later");
            assert!(
                constants
                    .states
                    .values()
                    .all(|state| !matches!(state, ConstantState::Visiting))
            );
        }
        let ready = &graph.declarations()[2];
        let ScalarPreparation::Ready(ScalarConstant::Literal(value)) = constants
            .prepare_value(ready.id(), ready.location())
            .unwrap()
        else {
            panic!()
        };
        assert_eq!(value, 42);
        let _ = constants
            .prepare_value(outer.id(), outer.location())
            .unwrap();
        assert!(matches!(
            constants.states.get(&ready.id()),
            Some(ConstantState::Ready(_))
        ));
    }

    #[test]
    fn scalar_source_errors_remain_errors_and_retry_without_false_cycles() {
        let graph = graph("VALUE::UNKNOWN+1;");
        let source = &graph.declarations()[0];
        let mut constants = Constants::new(&graph);
        for _ in 0..2 {
            let error = constants
                .prepare_value(source.id(), source.location())
                .err()
                .unwrap();
            assert!(!error.message.contains("cyclic"), "{error}");
            assert!(!constants.states.contains_key(&source.id()));
        }
    }
}

use jai_eval::{DomainInference, ScalarDomain, ScalarInferenceError};
use std::collections::HashSet;

enum DomainPreparation<T> {
    Ready(T),
    Pending(PendingType),
}

impl Constants<'_> {
    fn prepare_expression_domain<'e>(
        &self,
        file: FileInstanceId,
        expression: &'e Expression,
        visiting: &mut HashSet<DeclarationId>,
        typed_origin: Option<(DeclarationId, SourceSpan)>,
    ) -> Result<DomainPreparation<DomainInference<'e>>, LocatedDiagnostic> {
        let mut pending = None;
        let result = DomainInference::infer_for_preparation(expression, |path, span| {
            match self.prepare_domain_path(file, path, span, visiting) {
                Ok(DomainPreparation::Ready(domain)) => Ok(domain),
                Ok(DomainPreparation::Pending(cause)) => {
                    pending.get_or_insert(cause);
                    let error = cause.diagnostic(self.graph);
                    Err(Diagnostic::at_source(error.location, error.message))
                }
                Err(error) => Err(Diagnostic::at_source(error.location, error.message)),
            }
        });
        match pending {
            Some(cause) => Ok(DomainPreparation::Pending(cause)),
            None => match result {
                Ok(inference) => Ok(DomainPreparation::Ready(inference)),
                Err(ScalarInferenceError::RequiresTypedExecution(error)) => match typed_origin {
                    Some((declaration, location)) => {
                        Ok(DomainPreparation::Pending(PendingType::Constant {
                            declaration,
                            location,
                        }))
                    }
                    None => Err(located(self.graph, file, error)),
                },
                Err(ScalarInferenceError::Diagnostic(error)) => {
                    Err(located(self.graph, file, error))
                }
            },
        }
    }

    fn prepare_domain_path(
        &self,
        file: FileInstanceId,
        path: &NamePath,
        span: Span,
        visiting: &mut HashSet<DeclarationId>,
    ) -> Result<DomainPreparation<ScalarDomain>, LocatedDiagnostic> {
        let site = SourceSpan {
            source: self.graph.file(file).unwrap().source(),
            span,
        };
        if let Err(error) = self.graph.lookup(file, path)
            && let Ok(pending) = PendingType::from_lookup(error, site)
        {
            return Ok(DomainPreparation::Pending(pending));
        }
        if let Some(value) = self.external_value(file, path, span)? {
            return Ok(DomainPreparation::Ready(value.domain()));
        }
        let id = declaration_id(self.graph, file, path, span)
            .map_err(|error| located(self.graph, file, error))?;
        if let Some(ConstantState::Ready(value)) = self.states.get(&id) {
            return Ok(DomainPreparation::Ready(value.domain()));
        }
        let declaration = self
            .graph
            .declaration(id)
            .expect("resolved declaration exists");
        let FileDeclarationKind::Constant(constant) = &declaration.syntax().kind else {
            return Err(LocatedDiagnostic {
                location: site,
                message: "declaration cannot supply a scalar constant domain".into(),
            });
        };
        let primitive = self.primitive_annotations.get(&id).copied().or_else(|| {
            constant
                .ty
                .as_ref()
                .and_then(|annotation| self.primitive_annotation(declaration.file(), annotation))
        });
        if let Some(primitive) = primitive {
            return Ok(DomainPreparation::Ready(match primitive {
                PrimitiveAnnotation::Scalar(ScalarType::Int(integer)) => {
                    ScalarDomain::Integer(integer)
                }
                PrimitiveAnnotation::Scalar(ScalarType::Bool) => ScalarDomain::Bool,
                PrimitiveAnnotation::Float(float) => ScalarDomain::Float(float),
            }));
        }
        if constant.ty.is_some() {
            if self.checked_typed_values.contains_key(&id) {
                return Err(LocatedDiagnostic {
                    location: site,
                    message:
                        "a checked nominal or aggregate constant cannot supply a scalar domain"
                            .into(),
                });
            }
            return Ok(DomainPreparation::Pending(PendingType::Constant {
                declaration: id,
                location: site,
            }));
        }
        if visiting.len() >= 256 || !visiting.insert(id) {
            return Err(LocatedDiagnostic {
                location: site,
                message: "cyclic or excessively deep scalar domain dependencies".into(),
            });
        }
        let result = self.prepare_expression_domain(
            declaration.file(),
            &constant.initializer,
            visiting,
            Some((id, site)),
        );
        visiting.remove(&id);
        result.map(|result| match result {
            DomainPreparation::Ready(inference) => DomainPreparation::Ready(inference.domain()),
            DomainPreparation::Pending(cause) => DomainPreparation::Pending(cause),
        })
    }
}

#[cfg(test)]
mod domain_tests {
    use super::*;
    use jai_modules::{GraphOptions, SourceOverlay};
    fn graph(source: &str) -> ModuleGraph {
        let path = std::path::Path::new("/jai-domain-paired/main.jai");
        let mut overlay = SourceOverlay::new();
        overlay.insert(path, source.as_bytes().to_vec()).unwrap();
        ModuleGraph::load_with_provider(path, GraphOptions::default(), &overlay).unwrap()
    }
    #[test]
    fn inactive_integer_value_is_unread_but_its_domain_widens_selected_literal() {
        let graph = graph("BROKEN:u8:1/0; VALUE::ifx true then 7 else BROKEN;");
        let mut types = TypeRegistry::new();
        let mut constants = Constants::new(&graph);
        constants.register_startup_annotations(&mut types).unwrap();
        let value = &graph.declarations()[1];
        let ScalarPreparation::Ready(ScalarConstant::Int(value)) = constants
            .prepare_value(value.id(), value.location())
            .unwrap()
        else {
            panic!()
        };
        assert_eq!(value.ty(), jai_types::IntegerType::U8);
        assert_eq!(value.value(), 7);
        assert!(!constants.states.contains_key(&graph.declarations()[0].id()));
    }
    #[test]
    fn inactive_float_value_is_unread_but_its_domain_preserves_common_width() {
        let graph = graph("BROKEN:float64:1.0/0.0; VALUE::ifx true then 1.25 else BROKEN;");
        let mut types = TypeRegistry::new();
        let mut constants = Constants::new(&graph);
        constants.register_startup_annotations(&mut types).unwrap();
        let value = &graph.declarations()[1];
        let ScalarPreparation::Ready(ScalarConstant::Float(value)) = constants
            .prepare_value(value.id(), value.location())
            .unwrap()
        else {
            panic!()
        };
        assert_eq!(value.ty(), jai_types::FloatType::F64);
        assert!(!constants.states.contains_key(&graph.declarations()[0].id()));
    }
    #[test]
    fn short_circuit_uses_domain_metadata_without_reading_an_inactive_initializer() {
        let graph = graph("BROKEN:bool:1/0; VALUE::true || BROKEN;");
        let mut types = TypeRegistry::new();
        let mut constants = Constants::new(&graph);
        constants.register_startup_annotations(&mut types).unwrap();
        let value = &graph.declarations()[1];
        let ScalarPreparation::Ready(ScalarConstant::Bool(value)) = constants
            .prepare_value(value.id(), value.location())
            .unwrap()
        else {
            panic!()
        };
        assert!(value);
        assert!(!constants.states.contains_key(&graph.declarations()[0].id()));
    }
    #[test]
    fn unavailable_inactive_domain_remains_a_real_placeholder_wait() {
        let graph = graph("#placeholder LATER; VALUE::ifx true then 1 else LATER;");
        let mut constants = Constants::new(&graph);
        let value = &graph.declarations()[0];
        let ScalarPreparation::Pending(PendingType::Placeholder(demand)) = constants
            .prepare_value(value.id(), value.location())
            .unwrap()
        else {
            panic!()
        };
        assert_eq!(demand.placeholder, graph.placeholders()[0].id());
        assert!(constants.states.is_empty());
    }
}

#[cfg(test)]
mod deferred_tests {
    use super::*;
    #[test]
    fn inactive_checked_value_does_not_defer_a_pure_scalar_alias_or_its_array_count() {
        use crate::modules::aggregates::{
            Nominals,
            parameterized::{RecordSpecializations, TypePreparation},
        };
        let path = std::path::Path::new("/jai-pure-count/main.jai");
        let mut overlay = SourceOverlay::new();
        overlay.insert(path,b"N:int:#run seed();M::ifx true then 42 else N;Alias::#type [M]u8;seed::()->int{return 7;}".to_vec()).unwrap();
        let graph =
            ModuleGraph::load_with_provider(path, GraphOptions::default(), &overlay).unwrap();
        let n = &graph.declarations()[0];
        let m = &graph.declarations()[1];
        let alias = &graph.declarations()[2];
        assert!(crate::modules::deferred_constants::classify(&graph).contains(&m.id()));
        let mut types = TypeRegistry::new();
        let mut nominals = Nominals::reserve(&graph, &mut types).unwrap();
        let mut records = RecordSpecializations::default();
        let mut constants = Constants::new(&graph);
        constants.register_startup_annotations(&mut types).unwrap();
        let Some(TypePreparation::Ready(ty)) = nominals
            .prepare_alias(
                &graph,
                alias.id(),
                &mut types,
                &mut records,
                &mut |file, value| constants.prepare_evaluate_lazy(file, value),
            )
            .unwrap()
        else {
            panic!("inactive checked value must not block the pure scalar alias")
        };
        assert!(matches!(
            types.kind(ty).unwrap(),
            jai_types::TypeKind::FixedArray {
                count: 42,
                ..
            }
        ));
        assert!(!constants.states.contains_key(&n.id()));
        assert!(
            matches!(constants.states.get(&m.id()),Some(ConstantState::Ready(ScalarConstant::Int(value))) if value.value()==42)
        );
    }
    #[test]
    fn selected_checked_value_waits_for_its_actual_producer_not_its_pure_alias() {
        let path = std::path::Path::new("/jai-selected-count/main.jai");
        let mut overlay = SourceOverlay::new();
        overlay
            .insert(
                path,
                b"N:int:#run seed();M::ifx false then 42 else N;seed::()->int{return 7;}".to_vec(),
            )
            .unwrap();
        let graph =
            ModuleGraph::load_with_provider(path, GraphOptions::default(), &overlay).unwrap();
        let n = &graph.declarations()[0];
        let m = &graph.declarations()[1];
        let mut types = TypeRegistry::new();
        let mut constants = Constants::new(&graph);
        constants.register_startup_annotations(&mut types).unwrap();
        let ScalarPreparation::Pending(PendingType::Constant {
            declaration,
            location,
        }) = constants.prepare_value(m.id(), m.location()).unwrap()
        else {
            panic!()
        };
        assert_eq!(declaration, n.id());
        assert_eq!(
            location
                .span
                .text(graph.sources().get(location.source).unwrap().text()),
            "N"
        );
        assert!(constants.states.is_empty());
        // This verifies the accepted-result boundary, not execution of #run.
        constants
            .register_ready_value(
                n.id(),
                ScalarConstant::Int(jai_types::Integer::wrapping(jai_types::IntegerType::S64, 7)),
            )
            .unwrap();
        let ScalarPreparation::Ready(ScalarConstant::Int(value)) =
            constants.prepare_value(m.id(), m.location()).unwrap()
        else {
            panic!()
        };
        assert_eq!(value.value(), 7);
    }

    #[test]
    fn array_count_retains_checked_constant_identity_without_publishing_an_alias() {
        use crate::modules::aggregates::{
            Nominals,
            parameterized::{RecordSpecializations, TypePreparation},
        };
        let path = std::path::Path::new("/jai-pending-count/main.jai");
        let mut overlay = SourceOverlay::new();
        overlay
            .insert(
                path,
                b"count::()->int{return 42;}N::#run count();Alias::#type [N]int;".to_vec(),
            )
            .unwrap();
        let graph =
            ModuleGraph::load_with_provider(path, GraphOptions::default(), &overlay).unwrap();
        let mut types = TypeRegistry::new();
        let mut nominals = Nominals::reserve(&graph, &mut types).unwrap();
        let mut records = RecordSpecializations::default();
        let mut constants = Constants::new(&graph);
        let count = &graph.declarations()[1];
        let alias = &graph.declarations()[2];
        let Some(TypePreparation::Pending(PendingType::Constant {
            declaration, ..
        })) = nominals
            .prepare_alias(
                &graph,
                alias.id(),
                &mut types,
                &mut records,
                &mut |file, value| constants.prepare_evaluate_lazy(file, value),
            )
            .unwrap()
        else {
            panic!()
        };
        assert_eq!(declaration, count.id());
        assert!(!nominals.declarations.contains_key(&alias.id()));
        // This verifies the accepted-result boundary, not execution of #run.
        constants
            .register_ready_value(count.id(), ScalarConstant::Literal(42))
            .unwrap();
        let Some(TypePreparation::Ready(ty)) = nominals
            .prepare_alias(
                &graph,
                alias.id(),
                &mut types,
                &mut records,
                &mut |file, value| constants.prepare_evaluate_lazy(file, value),
            )
            .unwrap()
        else {
            panic!()
        };
        assert_eq!(nominals.declarations.get(&alias.id()), Some(&ty));
        assert!(matches!(
            types.kind(ty).unwrap(),
            jai_types::TypeKind::FixedArray {
                count: 42,
                ..
            }
        ));
    }

    use jai_modules::{GraphOptions, SourceOverlay};
    #[test]
    fn checked_constant_wait_keeps_its_actual_origin_and_requires_a_completed_scalar() {
        let path = std::path::Path::new("/jai-pending-constant/main.jai");
        let mut overlay = SourceOverlay::new();
        overlay
            .insert(
                path,
                b"count::()->int{return 42;}N::#run count();Alias::#type [N]int;".to_vec(),
            )
            .unwrap();
        let graph =
            ModuleGraph::load_with_provider(path, GraphOptions::default(), &overlay).unwrap();
        let source = &graph.declarations()[1];
        let mut constants = Constants::new(&graph);
        let ScalarPreparation::Pending(PendingType::Constant {
            declaration,
            location,
        }) = constants
            .prepare_value(source.id(), source.location())
            .unwrap()
        else {
            panic!()
        };
        assert_eq!(declaration, source.id());
        assert_eq!(location, source.location());
        assert!(constants.states.is_empty());
        constants
            .register_ready_value(source.id(), ScalarConstant::Literal(42))
            .unwrap();
        let ScalarPreparation::Ready(ScalarConstant::Literal(value)) = constants
            .prepare_value(source.id(), source.location())
            .unwrap()
        else {
            panic!()
        };
        assert_eq!(value, 42);
        constants
            .register_ready_value(source.id(), ScalarConstant::Literal(42))
            .unwrap();
        assert!(
            constants
                .register_ready_value(source.id(), ScalarConstant::Literal(43))
                .is_err()
        );
    }
}
