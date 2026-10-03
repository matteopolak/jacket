//! Resolve retained module requests using the preparation's existing registry.
use super::*;
use crate::polymorphism::BakedValue;
use jai_modules::{
    DeferredParameter, ModuleBoundArgument, ModuleBuiltin, ModuleType, ParameterRequestId,
    ParameterResponse, ParameterTask,
};
use jai_types::{ScalarType, TypeView};

#[derive(Debug)]
pub struct ParameterDiscoveryOutcome {
    pub decisions: Vec<(ParameterRequestId, ParameterResponse)>,
    pub pending: Vec<ParameterDiscoveryPending>,
}
#[derive(Debug)]
pub struct ParameterDiscoveryPending {
    pub request: ParameterRequestId,
    pub diagnostic: LocatedDiagnostic,
}

pub(super) fn resolve(
    graph: &ModuleGraph,
    requests: &[DeferredParameter],
    declarations: &ScopedDeclarations<'_>,
    types: &mut TypeRegistry,
    meta: &mut crate::reflection::MetaContext,
    constants: &mut Constants<'_>,
) -> ParameterDiscoveryOutcome {
    let nominals = &declarations.nominals;
    let mut outcome = ParameterDiscoveryOutcome {
        decisions: vec![],
        pending: vec![],
    };
    for request in requests.iter().filter(|request| request.response.is_none()) {
        let substitution = match request
            .specialization
            .as_ref()
            .map(|key| super::source_specializations::decode(key, declarations, types, meta))
            .transpose()
        {
            Ok(substitution) => substitution,
            Err(diagnostic) => {
                outcome.pending.push(ParameterDiscoveryPending {
                    request: request.id,
                    diagnostic,
                });
                continue;
            }
        };
        let result = match &request.task {
            ParameterTask::ResolveType {
                syntax: syntax::TypeSyntax::Restricted {
                    ..
                },
            } => Err(LocatedDiagnostic {
                location: request.location,
                message: "module type restriction requires semantic constraint proof".into(),
            }),
            ParameterTask::ResolveType {
                syntax,
            } => aggregates::parameterized::resolve_type(
                graph,
                aggregates::parameterized::TypeRequest::new(
                    request.file,
                    syntax,
                    request.location.span,
                )
                .with_substitution(substitution.as_ref()),
                types,
                nominals,
                &mut meta.record_specializations,
                &mut |file, expression| constants.evaluate_lazy(file, expression),
            )
            .and_then(|ty| {
                encode_type(
                    graph,
                    nominals,
                    types,
                    &meta.record_specializations,
                    ty,
                    request.location,
                    0,
                )
            })
            .map(ParameterResponse::Type),
            ParameterTask::CheckInterface {
                actual,
                required,
            } => super::parameter_interfaces::check(
                graph,
                request,
                super::parameter_interfaces::InterfaceTypes {
                    actual,
                    required,
                },
                nominals,
                types,
                meta,
                constants,
            )
            .map(|()| ParameterResponse::InterfaceSatisfied),
            ParameterTask::CheckNominal {
                actual,
                required,
            } => super::parameter_nominals::check(
                graph,
                request,
                super::parameter_nominals::NominalTypes {
                    actual,
                    required,
                },
                nominals,
                types,
                meta,
                constants,
            )
            .map(|()| ParameterResponse::NominalSatisfied),
            ParameterTask::CoerceValue {
                ..
            } => Err(LocatedDiagnostic {
                location: request.location,
                message:
                    "module aggregate value requires canonical source constant materialization"
                        .into(),
            }),
        };
        match result {
            Ok(response) => outcome.decisions.push((request.id, response)),
            Err(diagnostic) => outcome.pending.push(ParameterDiscoveryPending {
                request: request.id,
                diagnostic,
            }),
        }
    }
    outcome
}

pub(super) fn encode_type(
    graph: &ModuleGraph,
    nominals: &Nominals<'_>,
    types: &TypeRegistry,
    records: &aggregates::parameterized::RecordSpecializations,
    ty: TypeId,
    location: SourceSpan,
    depth: usize,
) -> Result<ModuleType, LocatedDiagnostic> {
    let fail = |message: &str| LocatedDiagnostic {
        location,
        message: message.into(),
    };
    if depth >= 128 {
        return Err(fail(
            "module source type identity exceeds the recursion limit",
        ));
    }
    if let Some(key) = records.key_for_type(ty) {
        let declaration = graph
            .declaration(key.template.0)
            .ok_or_else(|| fail("module type template is unavailable"))?;
        if matches!(&declaration.syntax().kind, FileDeclarationKind::Record(record) if record.modify.is_some())
        {
            return Err(fail(
                "modified module types require committed modifier replay outcomes",
            ));
        }
        let arguments = key
            .arguments
            .iter()
            .map(|argument| {
                encode_argument(
                    graph,
                    nominals,
                    types,
                    records,
                    argument,
                    location,
                    depth + 1,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        return Ok(ModuleType::Application {
            template: key.template.0,
            arguments,
        });
    }
    let kind = types.kind(ty).map_err(|error| fail(&error.to_string()))?;
    let builtin = match kind {
        TypeKind::Void => Some(ModuleBuiltin::Void),
        TypeKind::Type => Some(ModuleBuiltin::Type),
        TypeKind::Bool => Some(ModuleBuiltin::Scalar(ScalarType::Bool)),
        TypeKind::Integer(ty) => Some(ModuleBuiltin::Scalar(ScalarType::Int(*ty))),
        TypeKind::Float(ty) => Some(ModuleBuiltin::Float(*ty)),
        TypeKind::String => Some(ModuleBuiltin::String),
        TypeKind::Any(_) => Some(ModuleBuiltin::Any),
        _ if nominals.context_type_id() == Some(ty) => Some(ModuleBuiltin::Context),
        _ => None,
    };
    if let Some(builtin) = builtin {
        return Ok(ModuleType::Builtin(builtin));
    }
    let nested = |ty| encode_type(graph, nominals, types, records, ty, location, depth + 1);
    Ok(match kind {
        TypeKind::Pointer(inner) => ModuleType::Pointer(Box::new(nested(*inner)?)),
        TypeKind::Slice(inner) => ModuleType::Slice(Box::new(nested(*inner)?)),
        TypeKind::DynamicArray(inner) => ModuleType::DynamicArray(Box::new(nested(*inner)?)),
        TypeKind::FixedArray {
            element,
            count,
        } => ModuleType::FixedArray {
            element: Box::new(nested(*element)?),
            count: *count,
        },
        TypeKind::Procedure(id) => {
            let signature = types
                .procedure_type(*id)
                .map_err(|error| fail(&error.to_string()))?;
            let mut parameters = signature
                .parameters
                .iter()
                .copied()
                .map(nested)
                .collect::<Result<Vec<_>, _>>()?;
            let variadic = match signature.variadic {
                jai_types::Variadic::None => jai_modules::ModuleVariadic::None,
                jai_types::Variadic::C {
                    fixed_parameters,
                } => jai_modules::ModuleVariadic::C {
                    fixed_parameters,
                },
                jai_types::Variadic::Jai {
                    parameter,
                    element,
                } => {
                    let entry = parameters.get_mut(parameter).ok_or_else(|| {
                        fail("module procedure variadic parameter is unavailable")
                    })?;
                    *entry = nested(element)?;
                    jai_modules::ModuleVariadic::Jai {
                        parameter,
                    }
                }
            };
            ModuleType::Procedure(jai_modules::ModuleProcedureType {
                parameters,
                results: signature
                    .results
                    .iter()
                    .copied()
                    .map(nested)
                    .collect::<Result<Vec<_>, _>>()?,
                return_abi: signature.return_abi,
                convention: signature.convention,
                context: signature.context,
                variadic,
            })
        }
        TypeKind::Record(_) | TypeKind::Enum(_) | TypeKind::Distinct(_) => {
            let origin = nominals.declarations.iter().find_map(|(id, &registered)| {
                (registered == ty && matches!(graph.declaration(*id).map(|d| &d.syntax().kind),
                    Some(FileDeclarationKind::Record(_) | FileDeclarationKind::Enum(_))
                    | Some(FileDeclarationKind::TypeAlias(syntax::TypeAliasDeclaration { ty: syntax::TypeSyntax::Variant { .. }, .. }))))
                    .then_some(*id)
            }).ok_or_else(|| fail("anonymous or generated module type requires a retained nominal source identity"))?;
            if matches!(graph.declaration(origin).map(|declaration| &declaration.syntax().kind), Some(FileDeclarationKind::Record(record)) if record.modify.is_some())
            {
                return Err(fail(
                    "modified module types require committed modifier replay outcomes",
                ));
            }
            ModuleType::Declaration(origin)
        }
        _ => return Err(fail("module source type identity is not representable")),
    })
}
pub(super) fn encode_argument(
    graph: &ModuleGraph,
    nominals: &Nominals<'_>,
    types: &TypeRegistry,
    records: &aggregates::parameterized::RecordSpecializations,
    value: &BakedValue,
    location: SourceSpan,
    depth: usize,
) -> Result<ModuleBoundArgument, LocatedDiagnostic> {
    let fail = |message: &str| LocatedDiagnostic {
        location,
        message: message.into(),
    };
    Ok(match value {
        BakedValue::Type(ty) => ModuleBoundArgument::Type(encode_type(
            graph, nominals, types, records, *ty, location, depth,
        )?),
        BakedValue::Float(value) => ModuleBoundArgument::Float(*value),
        BakedValue::String(value) => ModuleBoundArgument::String(value.clone()),
        BakedValue::Value(value) => match &value.kind {
            jai_ir::ConstantKind::Int(value) => ModuleBoundArgument::Integer(*value),
            jai_ir::ConstantKind::Bool(value) => ModuleBoundArgument::Boolean(*value),
            jai_ir::ConstantKind::Float(value) => ModuleBoundArgument::Float(*value),
            jai_ir::ConstantKind::StringBytes(value) => {
                ModuleBoundArgument::String(value.clone().into_boxed_slice())
            }
            jai_ir::ConstantKind::Enum(integer) => {
                let ModuleType::Declaration(declaration) =
                    encode_type(graph, nominals, types, records, value.ty, location, depth)?
                else {
                    return Err(fail(
                        "module enum argument requires its original source declaration",
                    ));
                };
                ModuleBoundArgument::Enumeration(jai_modules::EnumParameter {
                    declaration,
                    value: *integer,
                })
            }
            _ => {
                return Err(fail(
                    "aggregate module specialization arguments require a canonical source constant codec",
                ));
            }
        },
        BakedValue::Code(_) => {
            return Err(fail(
                "module code arguments require retained source code identity",
            ));
        }
    })
}
