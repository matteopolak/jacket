//! Source module requests reuse the current session's canonical record binder.
use super::*;
use jai_modules::ModuleBoundArgument;

pub(crate) struct ModuleApplicationRequest<'a> {
    pub file: FileInstanceId,
    pub template: DeclarationId,
    pub arguments: &'a [ModuleBoundArgument],
    pub span: Span,
}

pub(crate) fn instantiate_module_application(
    graph: &ModuleGraph,
    request: ModuleApplicationRequest<'_>,
    types: &mut TypeRegistry,
    nominals: &Nominals<'_>,
    records: &mut RecordSpecializations,
    evaluate: &mut impl FnMut(
        FileInstanceId,
        &syntax::Expression,
    ) -> Result<ScalarConstant, LocatedDiagnostic>,
) -> Result<TypeId, LocatedDiagnostic> {
    let ModuleApplicationRequest {
        file,
        template,
        arguments,
        span,
    } = request;
    let template = template_origin_declaration(graph, template, span)
        .map_err(|error| located(graph, file, error))?;
    let declaration = graph
        .declaration(template)
        .expect("source template belongs to graph");
    let syntax::FileDeclarationKind::Record(record) = &declaration.syntax().kind else {
        unreachable!("template origin validates record declaration");
    };
    if record.parameters.len() != arguments.len() {
        return Err(located(
            graph,
            file,
            Diagnostic::new(
                span,
                "module record type request differs from the declared formal argument count",
            ),
        ));
    }
    let mut substitution = Substitution::default();
    for (parameter, argument) in record.parameters.iter().zip(arguments) {
        let value = match argument {
            ModuleBoundArgument::Type(value) => {
                BakedValue::Type(nominals.resolve_module_type_with_specializations(
                    graph,
                    crate::modules::aggregates::types::ModuleTypeRequest {
                        file,
                        value,
                        span,
                    },
                    types,
                    records,
                    evaluate,
                )?)
            }
            ModuleBoundArgument::String(value) => BakedValue::String(value.clone()),
            argument => {
                use jai_ir::ConstantKind as K;
                let (ty, kind) = match argument {
                    ModuleBoundArgument::Integer(value) => (
                        types.scalar(jai_types::ScalarType::Int(value.ty())),
                        K::Int(*value),
                    ),
                    ModuleBoundArgument::Boolean(value) => {
                        (types.scalar(jai_types::ScalarType::Bool), K::Bool(*value))
                    }
                    ModuleBoundArgument::Float(value) => {
                        (types.float(value.ty()), K::Float(*value))
                    }
                    ModuleBoundArgument::Enumeration(value) => {
                        let ty = nominals.declarations.get(&value.declaration).copied().ok_or_else(|| located(
                            graph, file, Diagnostic::new(span, "module enumeration argument has no canonical source identity")))?;
                        (ty, K::Enum(value.value))
                    }
                    ModuleBoundArgument::Type(_) | ModuleBoundArgument::String(_) => unreachable!(),
                };
                BakedValue::runtime(
                    jai_ir::ConstantValue {
                        ty,
                        kind,
                    },
                    types,
                )
                .map_err(|error| located(graph, file, Diagnostic::new(span, error.to_string())))?
            }
        };
        substitution.bind_constant(parameter.name, value);
    }
    instantiate_bound(
        graph,
        BoundRecordRequest {
            declaration: template,
            substitution,
            span,
        },
        types,
        nominals,
        records,
        evaluate,
    )
}
