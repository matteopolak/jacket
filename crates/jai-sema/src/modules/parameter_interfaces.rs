//! Interface conformance compares ready semantic members in one registry.
use super::*;
use jai_modules::{DeferredParameter, ModuleType};
use jai_types::{CallingConvention, ProcedureType, Variadic};
use std::collections::HashSet;

pub(super) struct InterfaceTypes<'a> {
    pub actual: &'a ModuleType,
    pub required: &'a ModuleType,
}
pub(super) fn check(
    graph: &ModuleGraph,
    request: &DeferredParameter,
    input: InterfaceTypes<'_>,
    nominals: &Nominals<'_>,
    types: &mut TypeRegistry,
    meta: &mut crate::reflection::MetaContext,
    constants: &mut Constants<'_>,
) -> Result<(), LocatedDiagnostic> {
    let mut evaluate =
        |file, expression: &syntax::Expression| constants.evaluate_lazy(file, expression);
    let actual = nominals.resolve_module_type_with_specializations(
        graph,
        crate::modules::aggregates::types::ModuleTypeRequest {
            file: request.file,
            value: input.actual,
            span: request.location.span,
        },
        types,
        &mut meta.record_specializations,
        &mut evaluate,
    )?;
    let required = nominals.resolve_module_type_with_specializations(
        graph,
        crate::modules::aggregates::types::ModuleTypeRequest {
            file: request.file,
            value: input.required,
            span: request.location.span,
        },
        types,
        &mut meta.record_specializations,
        &mut evaluate,
    )?;
    let required_fields = fields(
        graph,
        request,
        required,
        nominals,
        &meta.record_specializations,
    )?;
    let actual_fields = fields(
        graph,
        request,
        actual,
        nominals,
        &meta.record_specializations,
    )?;
    for (name, ty) in required_fields {
        match actual_fields.get(&name) {
            None => {
                return Err(failure(
                    request,
                    format!(
                        "module type does not satisfy interface: missing member '{}'",
                        graph.symbols().name(name)
                    ),
                ));
            }
            Some(actual) if *actual != ty => {
                return Err(failure(
                    request,
                    format!(
                        "module type does not satisfy interface: incompatible member '{}'",
                        graph.symbols().name(name)
                    ),
                ));
            }
            _ => {}
        }
    }
    let required_methods = meta
        .record_specializations
        .methods(required)
        .unwrap_or_default()
        .to_vec();
    let actual_methods = meta
        .record_specializations
        .methods(actual)
        .unwrap_or_default()
        .to_vec();
    let required_substitution = meta
        .record_specializations
        .member_bindings(required)
        .cloned();
    let actual_substitution = meta.record_specializations.member_bindings(actual).cloned();
    for required_method in required_methods {
        let name = required_method.source.name();
        let required_signature = method_type(
            graph,
            request,
            MethodInput {
                method: &required_method,
                substitution: required_substitution.as_ref(),
            },
            nominals,
            types,
            &mut meta.record_specializations,
            &mut evaluate,
        )?;
        let mut candidates = vec![];
        for method in actual_methods
            .iter()
            .filter(|method| method.source.name() == name)
        {
            candidates.push(method_type(
                graph,
                request,
                MethodInput {
                    method,
                    substitution: actual_substitution.as_ref(),
                },
                nominals,
                types,
                &mut meta.record_specializations,
                &mut evaluate,
            )?);
        }
        if candidates.is_empty() {
            // Instance method promotion must follow the semantic member resolver;
            // a matching printed name in a nested record is insufficient proof.
            if has_using(actual, nominals, &meta.record_specializations) {
                return Err(failure(
                    request,
                    "promoted interface methods require semantic method resolution",
                ));
            }
            return Err(failure(
                request,
                format!(
                    "module type does not satisfy interface: missing member '{}'",
                    graph.symbols().name(name)
                ),
            ));
        }
        if !candidates.contains(&required_signature) {
            return Err(failure(
                request,
                format!(
                    "module type does not satisfy interface: incompatible member '{}'",
                    graph.symbols().name(name)
                ),
            ));
        }
    }
    Ok(())
}
fn failure(request: &DeferredParameter, message: impl Into<String>) -> LocatedDiagnostic {
    LocatedDiagnostic {
        location: request.location,
        message: message.into(),
    }
}
fn has_using(
    ty: TypeId,
    nominals: &Nominals<'_>,
    records: &aggregates::parameterized::RecordSpecializations,
) -> bool {
    records
        .record(ty)
        .is_some_and(|record| record.shape.fields.iter().any(|field| field.syntax.using()))
        || nominals
            .records
            .get(&ty)
            .is_some_and(|record| record.fields.iter().any(|field| field.syntax.using))
}
fn fields(
    graph: &ModuleGraph,
    request: &DeferredParameter,
    root: TypeId,
    nominals: &Nominals<'_>,
    records: &aggregates::parameterized::RecordSpecializations,
) -> Result<HashMap<Symbol, TypeId>, LocatedDiagnostic> {
    let mut result = HashMap::new();
    let mut pending = vec![(root, HashSet::new())];
    while let Some((ty, mut ancestors)) = pending.pop() {
        if !ancestors.insert(ty) {
            return Err(failure(
                request,
                "cyclic using field promotion in module interface",
            ));
        }
        let origin = records
            .record(ty)
            .and_then(|record| record.origin.map(|origin| origin.0))
            .or_else(|| nominals.records.get(&ty).map(|record| record.declaration));
        if origin.and_then(|id| graph.declaration(id)).is_some_and(|declaration| matches!(&declaration.syntax().kind, FileDeclarationKind::Record(record) if record.modify.is_some())) {
            return Err(failure(request, "modified module interfaces require committed modifier replay outcomes"));
        }
        let fields: Vec<_> = if let Some(record) = records.record(ty) {
            record
                .shape
                .fields
                .iter()
                .map(|field| (field.name, field.ty, field.syntax.using()))
                .collect()
        } else if let Some(record) = nominals.records.get(&ty) {
            record
                .fields
                .iter()
                .map(|field| (Some(field.name), field.ty, field.syntax.using))
                .collect()
        } else {
            return Err(failure(
                request,
                "module interface requires a ready record schema",
            ));
        };
        for (name, ty, using) in fields {
            if let Some(name) = name
                && result.insert(name, ty).is_some()
            {
                return Err(failure(
                    request,
                    format!(
                        "ambiguous promoted interface member '{}'",
                        graph.symbols().name(name)
                    ),
                ));
            }
            if using {
                pending.push((ty, ancestors.clone()));
            }
        }
    }
    Ok(result)
}
struct MethodInput<'a> {
    method: &'a aggregates::parameterized::RecordMethod,
    substitution: Option<&'a crate::polymorphism::Substitution>,
}
fn method_type(
    graph: &ModuleGraph,
    request: &DeferredParameter,
    input: MethodInput<'_>,
    nominals: &Nominals<'_>,
    types: &mut TypeRegistry,
    records: &mut aggregates::parameterized::RecordSpecializations,
    evaluate: &mut impl FnMut(
        FileInstanceId,
        &syntax::Expression,
    ) -> Result<ConstantValue, LocatedDiagnostic>,
) -> Result<TypeId, LocatedDiagnostic> {
    let MethodInput {
        method,
        substitution,
    } = input;
    use aggregates::parameterized::RecordMethodSource;
    use syntax::{ParameterBinding as P, ResultBinding as R};
    let (parameters, results, convention, return_abi, context) = match &method.source {
        RecordMethodSource::Procedure(source) => (
            &source.parameters,
            &source.results,
            source.convention,
            source.return_abi,
            source.context,
        ),
        RecordMethodSource::Prototype(source) => (
            &source.parameters,
            &source.results,
            source.convention,
            source.return_abi,
            source.context,
        ),
        RecordMethodSource::Constant(source) => {
            if let Some(crate::polymorphism::BakedValue::Value(value)) = records
                .member_bindings(method.id.owner)
                .and_then(|bindings| bindings.constant(source.name))
                && matches!(value.kind, jai_ir::ConstantKind::Procedure(_))
                && matches!(types.kind(value.ty), Ok(TypeKind::Procedure(_)))
            {
                return Ok(value.ty);
            }
            return Err(failure(
                request,
                "contextual lambda interface member requires a ready canonical callback signature",
            ));
        }
    };
    let mut bound_parameters = vec![];
    let mut variadic = Variadic::None;
    let mut seen_variadic = false;
    for parameter in parameters {
        if parameter.baking != syntax::ParameterBaking::None {
            return Err(failure(
                request,
                "generic interface methods require semantic specialization",
            ));
        }
        if parameter.variadic {
            if seen_variadic {
                return Err(failure(request, "duplicate variadic interface parameter"));
            }
            seen_variadic = true;
        }
        if parameter.variadic && convention == CallingConvention::C {
            if parameter.evaluation == syntax::ParameterEvaluation::Evaluate {
                variadic = Variadic::C {
                    fixed_parameters: bound_parameters.len(),
                };
            }
            continue;
        }
        let syntax = match &parameter.binding {
            P::Required(ty)
            | P::Defaulted {
                ty: Some(ty), ..
            } => syntax::TypeSyntax::Builtin(syntax::BuiltinType::Scalar(*ty)),
            P::RequiredType(ty)
            | P::DefaultedType {
                ty: Some(ty), ..
            } => ty.clone(),
            _ => {
                return Err(failure(
                    request,
                    "inferred interface method parameters require semantic type inference",
                ));
            }
        };
        let mut ty = aggregates::parameterized::resolve_type(
            graph,
            aggregates::parameterized::TypeRequest::new(method.file, &syntax, parameter.span)
                .with_substitution(substitution),
            types,
            nominals,
            records,
            evaluate,
        )?;
        if parameter.variadic {
            if parameter.evaluation == syntax::ParameterEvaluation::Evaluate {
                variadic = Variadic::Jai {
                    parameter: bound_parameters.len(),
                    element: ty,
                };
            }
            ty = types
                .slice(ty)
                .map_err(|error| failure(request, error.to_string()))?;
        }
        if parameter.evaluation == syntax::ParameterEvaluation::Evaluate {
            bound_parameters.push(ty);
        }
    }
    let mut bound_results = vec![];
    for result in results {
        let R::Typed {
            ty, ..
        } = &result.binding
        else {
            return Err(failure(
                request,
                "inferred interface method results require semantic type inference",
            ));
        };
        bound_results.push(aggregates::parameterized::resolve_type(
            graph,
            aggregates::parameterized::TypeRequest::new(method.file, ty, result.span)
                .with_substitution(substitution),
            types,
            nominals,
            records,
            evaluate,
        )?);
    }
    crate::procedure_values::signatures::normalize_results(&mut bound_results, types, |ty| *ty);
    types
        .procedure(ProcedureType {
            parameters: bound_parameters.into_boxed_slice(),
            results: bound_results.into_boxed_slice(),
            return_abi: return_abi,
            convention,
            context,
            variadic,
        })
        .map_err(|error| failure(request, error.to_string()))
}
