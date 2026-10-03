//! Carry source specialization identity between graph discovery and typed preparation.
use super::*;
use crate::polymorphism::{BakedValue, Substitution};
use jai_modules::{ModuleBoundArgument, SourceSpecializationKey};

pub(super) fn encode(
    declarations: &ScopedDeclarations<'_>,
    types: &TypeRegistry,
    meta: &crate::reflection::MetaContext,
    declaration: DeclarationId,
    procedure: &syntax::Procedure,
    substitution: &Substitution,
) -> Result<SourceSpecializationKey, LocatedDiagnostic> {
    let origin = declarations
        .graph
        .declaration(declaration)
        .ok_or_else(|| LocatedDiagnostic {
            location: SourceSpan {
                source: declarations.graph.files()[0].source(),
                span: procedure.span,
            },
            message: "source specialization requires its original graph declaration".into(),
        })?;
    let location = SourceSpan {
        source: origin.location().source,
        span: procedure.span,
    };
    let mut arguments = vec![];
    for binding in &substitution.types {
        arguments.push((
            binding.name,
            super::parameter_discovery::encode_type(
                declarations.graph,
                &declarations.nominals,
                types,
                &meta.record_specializations,
                binding.ty,
                location,
                0,
            )
            .map(ModuleBoundArgument::Type)?,
        ));
    }
    for binding in &substitution.constants {
        arguments.push((
            binding.name,
            super::parameter_discovery::encode_argument(
                declarations.graph,
                &declarations.nominals,
                types,
                &meta.record_specializations,
                &binding.value,
                location,
                0,
            )?,
        ));
    }
    // Canonicalize by formal source order, independently of named call argument order.
    let mut names = vec![];
    for parameter in &procedure.parameters {
        if parameter.baking != syntax::ParameterBaking::None {
            remember(&mut names, parameter.name);
        }
        match &parameter.binding {
            syntax::ParameterBinding::RequiredType(ty)
            | syntax::ParameterBinding::DefaultedType {
                ty: Some(ty), ..
            } => formal_type(ty, &mut names),
            _ => {}
        }
    }
    arguments.sort_by_key(|(name, _)| {
        names
            .iter()
            .position(|formal| formal == name)
            .unwrap_or(names.len())
    });
    Ok(SourceSpecializationKey::new(
        declaration,
        location,
        arguments,
    ))
}

pub(super) fn decode(
    key: &SourceSpecializationKey,
    declarations: &ScopedDeclarations<'_>,
    types: &mut TypeRegistry,
    meta: &mut crate::reflection::MetaContext,
) -> Result<Substitution, LocatedDiagnostic> {
    let graph = declarations.graph;
    let origin = graph
        .declaration(key.declaration())
        .ok_or_else(|| LocatedDiagnostic {
            location: SourceSpan {
                source: graph.files()[0].source(),
                span: key.procedure().span,
            },
            message: "source specialization no longer has its defining declaration".into(),
        })?;
    let file = origin.file();
    let location = SourceSpan {
        source: origin.location().source,
        span: key.procedure().span,
    };
    let mut constants = Constants::new(graph);
    constants.register_enums(&declarations.nominals);
    let mut substitution = Substitution::default();
    for (name, argument) in key.arguments() {
        let value = match argument {
            ModuleBoundArgument::Type(value) => {
                let ty = declarations
                    .nominals
                    .resolve_module_type_with_specializations(
                        graph,
                        crate::modules::aggregates::types::ModuleTypeRequest {
                            file,
                            value,
                            span: location.span,
                        },
                        types,
                        &mut meta.record_specializations,
                        &mut |file, expression| constants.evaluate_lazy(file, expression),
                    )?;
                substitution.bind_type(*name, ty);
                continue;
            }
            ModuleBoundArgument::String(value) => BakedValue::String(value.clone()),
            ModuleBoundArgument::Float(value) => BakedValue::Float(*value),
            value => {
                let (ty, kind) = match value {
                    ModuleBoundArgument::Integer(value) => (
                        types.scalar(jai_types::ScalarType::Int(value.ty())),
                        ConstantKind::Int(*value),
                    ),
                    ModuleBoundArgument::Boolean(value) => (
                        types.scalar(jai_types::ScalarType::Bool),
                        ConstantKind::Bool(*value),
                    ),
                    ModuleBoundArgument::Enumeration(value) => {
                        let ty = declarations
                            .nominals
                            .declarations
                            .get(&value.declaration)
                            .copied()
                            .ok_or_else(|| LocatedDiagnostic {
                                location,
                                message:
                                    "source specialization enum has no canonical nominal identity"
                                        .into(),
                            })?;
                        (ty, ConstantKind::Enum(value.value))
                    }
                    ModuleBoundArgument::Type(_)
                    | ModuleBoundArgument::String(_)
                    | ModuleBoundArgument::Float(_) => unreachable!(),
                };
                BakedValue::runtime(
                    jai_ir::ConstantValue {
                        ty,
                        kind,
                    },
                    types,
                )
                .map_err(|error| LocatedDiagnostic {
                    location,
                    message: error.to_string(),
                })?
            }
        };
        substitution.bind_constant(*name, value);
    }
    Ok(substitution)
}

fn remember(names: &mut Vec<Symbol>, name: Symbol) {
    if !names.contains(&name) {
        names.push(name);
    }
}
fn formal_type(ty: &syntax::TypeSyntax, names: &mut Vec<Symbol>) {
    use syntax::TypeSyntax as T;
    match ty {
        T::Variable(name) => remember(names, *name),
        T::Restricted {
            variable, ..
        } => remember(names, *variable),
        T::Pointer(value)
        | T::Slice(value)
        | T::DynamicArray(value)
        | T::Variant {
            base: value, ..
        } => formal_type(value, names),
        T::FixedArray {
            count,
            element,
        } => {
            formal_expression(count, names);
            formal_type(element, names);
        }
        T::Procedure(procedure) => {
            for parameter in procedure.parameters.iter().chain(&procedure.results) {
                formal_type(&parameter.ty, names);
            }
        }
        T::Application(application) => {
            formal_type(&application.base, names);
            for argument in &application.arguments {
                formal_expression(&argument.value, names);
            }
        }
        _ => {}
    }
}
fn formal_expression(expression: &syntax::Expression, names: &mut Vec<Symbol>) {
    use syntax::ExpressionKind as E;
    match &expression.kind {
        E::CompileVariable(name) => remember(names, *name),
        E::Type(ty) => formal_type(ty, names),
        E::Unary(_, value)
        | E::Cast(_, _, value)
        | E::InferredCast {
            value, ..
        }
        | E::AddressOf(value)
        | E::Dereference(value) => formal_expression(value, names),
        E::Binary(_, left, right)
        | E::Index {
            base: left,
            index: right,
        } => {
            formal_expression(left, names);
            formal_expression(right, names);
        }
        E::TypeCast {
            ty,
            value,
            ..
        } => {
            formal_type(ty, names);
            formal_expression(value, names);
        }
        _ => {}
    }
}
