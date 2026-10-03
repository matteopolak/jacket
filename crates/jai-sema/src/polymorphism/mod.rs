//! Typed substitutions and one-time specialization reservation.
pub(crate) mod integration;
mod template;
mod worklist;

pub use crate::procedure_values::contracts::CallablePolicyKey;
use jai_ir::{ConstantKind, ConstantValue};
use jai_source::{DeclarationId, Symbol};
use jai_types::{
    FloatValue, Integer, RecordKind, ScalarType, TypeError, TypeId, TypeKind, TypeRegistry,
    TypeView,
};
pub use template::{
    ProcedureTemplate, ResultPattern, from_procedure, from_procedure_with_patterns,
    from_prototype_with_patterns, is_polymorphic, is_polymorphic_prototype, substituted_pattern,
};
pub use worklist::{
    Readiness, Reservation, Specialization, SpecializationId, SpecializationState, Specializations,
};

/// Immutable compile-time data. Runtime addresses are deliberately not representable.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum BakedValue {
    Value(ConstantValue),
    Type(TypeId),
    Float(FloatValue),
    String(Box<[u8]>),
    Code(jai_types::CodeValueId),
}
impl BakedValue {
    pub fn integer(value: Integer, types: &dyn TypeView) -> Self {
        Self::Value(ConstantValue {
            ty: types.scalar(ScalarType::Int(value.ty())),
            kind: ConstantKind::Int(value),
        })
    }
    pub fn as_integer(&self) -> Option<Integer> {
        match self {
            Self::Value(ConstantValue {
                kind: ConstantKind::Int(value),
                ..
            }) => Some(*value),
            _ => None,
        }
    }
    pub fn as_type(&self) -> Option<TypeId> {
        match *self {
            Self::Type(ty) => Some(ty),
            _ => None,
        }
    }
    pub fn runtime(value: ConstantValue, types: &dyn TypeView) -> Result<Self, TypeError> {
        let value = normalize_constant(value, types)?;
        Ok(match value.kind {
            ConstantKind::RuntimeType(value) => Self::Type(value.identity().ty()),
            ConstantKind::Float(value) => Self::Float(value),
            ConstantKind::StringBytes(value) => Self::String(value.into_boxed_slice()),
            kind => Self::Value(ConstantValue { ty: value.ty, kind }),
        })
    }
    pub fn into_runtime(
        self,
        ty: TypeId,
        types: &dyn TypeView,
    ) -> Result<ConstantValue, TypeError> {
        let kind = match self {
            Self::Value(value) if value.ty == ty => return Ok(value),
            Self::Float(value) if matches!(types.kind(ty), Ok(TypeKind::Float(expected)) if *expected == value.ty()) => {
                ConstantKind::Float(value)
            }
            Self::String(value) if matches!(types.kind(ty), Ok(TypeKind::String)) => {
                ConstantKind::StringBytes(value.into_vec())
            }
            Self::Type(_) | Self::Code(_) => return Err(TypeError::NotAValue(ty)),
            _ => return Err(TypeError::WrongKind(ty)),
        };
        Ok(ConstantValue { ty, kind })
    }
}

fn normalize_constant(
    value: ConstantValue,
    types: &dyn TypeView,
) -> Result<ConstantValue, TypeError> {
    normalize_constant_inner(value, types, &mut Vec::new())
}
fn normalize_constant_inner(
    value: ConstantValue,
    types: &dyn TypeView,
    active: &mut Vec<TypeId>,
) -> Result<ConstantValue, TypeError> {
    if active.contains(&value.ty) {
        let mut cycle = active.clone();
        cycle.push(value.ty);
        return Err(TypeError::RecursiveValue { cycle });
    }
    active.push(value.ty);
    let outer_type = value.ty;
    let kind = match (types.kind(value.ty)?, value.kind) {
        (TypeKind::Integer(ty), ConstantKind::Zero) => ConstantKind::Int(Integer::wrapping(*ty, 0)),
        (TypeKind::Bool, ConstantKind::Zero) => ConstantKind::Bool(false),
        (TypeKind::Float(ty), ConstantKind::Zero) => ConstantKind::Float(match ty {
            jai_types::FloatType::F32 => FloatValue::F32(0),
            jai_types::FloatType::F64 => FloatValue::F64(0),
        }),
        (TypeKind::String, ConstantKind::Zero) => ConstantKind::StringBytes(Vec::new()),
        (TypeKind::Distinct(_), ConstantKind::Zero) => {
            let representation = types.distinct_definition(outer_type)?.representation;
            ConstantKind::Distinct(Box::new(normalize_constant_inner(
                ConstantValue {
                    ty: representation,
                    kind: ConstantKind::Zero,
                },
                types,
                active,
            )?))
        }
        (TypeKind::Enum(id), ConstantKind::Zero) => {
            ConstantKind::Enum(Integer::wrapping(types.enumeration(*id)?.representation, 0))
        }
        (TypeKind::Record(id) | TypeKind::Any(id), ConstantKind::Zero)
            if types.record(*id)?.kind == RecordKind::Struct =>
        {
            ConstantKind::Record(
                types
                    .record(*id)?
                    .fields
                    .iter()
                    .map(|&ty| {
                        normalize_constant_inner(
                            ConstantValue {
                                ty,
                                kind: ConstantKind::Zero,
                            },
                            types,
                            active,
                        )
                    })
                    .collect::<Result<_, _>>()?,
            )
        }
        (TypeKind::Record(id) | TypeKind::Any(id), ConstantKind::Record(fields)) => {
            let definition = types.record(*id)?;
            if definition.kind != RecordKind::Struct
                || definition.fields.len() != fields.len()
                || definition
                    .fields
                    .iter()
                    .zip(&fields)
                    .any(|(ty, field)| ty != &field.ty)
            {
                return Err(TypeError::WrongKind(value.ty));
            }
            ConstantKind::Record(
                fields
                    .into_iter()
                    .map(|field| normalize_constant_inner(field, types, active))
                    .collect::<Result<_, _>>()?,
            )
        }
        (TypeKind::Record(id), ConstantKind::Union { field, value }) => {
            let definition = types.record(*id)?;
            if definition.kind != RecordKind::Union
                || field.record() != *id
                || types.field_type(field)? != value.ty
            {
                return Err(TypeError::WrongKind(outer_type));
            }
            ConstantKind::Union {
                field,
                value: Box::new(normalize_constant_inner(*value, types, active)?),
            }
        }
        (TypeKind::Integer(ty), ConstantKind::Int(value)) if *ty == value.ty() => {
            ConstantKind::Int(value)
        }
        (TypeKind::Bool, ConstantKind::Bool(value)) => ConstantKind::Bool(value),
        (TypeKind::Float(ty), ConstantKind::Float(value)) if *ty == value.ty() => {
            ConstantKind::Float(value)
        }
        (TypeKind::String, ConstantKind::StringBytes(value)) => ConstantKind::StringBytes(value),
        (TypeKind::Procedure(_), ConstantKind::Procedure(procedure)) => {
            ConstantKind::Procedure(procedure)
        }
        (TypeKind::FixedArray { element, count }, ConstantKind::Array(values))
            if usize::try_from(*count).ok() == Some(values.len())
                && values.iter().all(|value| value.ty == *element) =>
        {
            let values: Vec<_> = values
                .into_iter()
                .map(|value| normalize_constant_inner(value, types, active))
                .collect::<Result<_, _>>()?;
            if values.iter().all(crate::constant_limits::is_zero) {
                ConstantKind::Zero
            } else {
                ConstantKind::Array(values)
            }
        }
        (TypeKind::Distinct(_), ConstantKind::Distinct(value))
            if value.ty == types.distinct_definition(outer_type)?.representation =>
        {
            ConstantKind::Distinct(Box::new(normalize_constant_inner(*value, types, active)?))
        }
        (TypeKind::Enum(id), ConstantKind::Enum(value))
            if types.enumeration(*id)?.representation == value.ty() =>
        {
            ConstantKind::Enum(value)
        }
        (TypeKind::Type, ConstantKind::RuntimeType(constant)) if constant.ty() == value.ty => {
            constant.validate(types).map_err(|error| match error {
                jai_ir::StaticDataError::Type(error) => error,
                _ => TypeError::WrongKind(value.ty),
            })?;
            ConstantKind::RuntimeType(constant)
        }
        (TypeKind::Void | TypeKind::Code, _) => {
            return Err(TypeError::NotAValue(value.ty));
        }
        (_, ConstantKind::Zero) => ConstantKind::Zero,
        _ => return Err(TypeError::WrongKind(value.ty)),
    };
    active.pop();
    Ok(ConstantValue { ty: value.ty, kind })
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TypeBinding {
    pub name: Symbol,
    pub ty: TypeId,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ConstantBinding {
    pub name: Symbol,
    pub value: BakedValue,
}

/// Executable source callable policy, ordered by its actual generic formal.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CallablePolicyBinding {
    pub name: Symbol,
    /// Zero for an ordinary argument; source order within a variadic pack otherwise.
    pub occurrence: usize,
    pub policy: CallablePolicyKey,
}

/// Bindings are ordered by the declaration, never by call-site argument order.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Substitution {
    pub types: Vec<TypeBinding>,
    pub constants: Vec<ConstantBinding>,
    pub callables: Vec<CallablePolicyBinding>,
}
impl Substitution {
    pub fn ty(&self, name: Symbol) -> Option<TypeId> {
        self.types
            .iter()
            .find(|binding| binding.name == name)
            .map(|binding| binding.ty)
            .or_else(|| self.constant(name).and_then(BakedValue::as_type))
    }
    pub fn constant(&self, name: Symbol) -> Option<&BakedValue> {
        self.constants
            .iter()
            .find(|binding| binding.name == name)
            .map(|binding| &binding.value)
    }
    pub(crate) fn bind_type(&mut self, name: Symbol, ty: TypeId) -> bool {
        match self.ty(name) {
            Some(previous) => previous == ty,
            None => {
                self.types.push(TypeBinding { name, ty });
                true
            }
        }
    }
    pub(crate) fn bind_constant(&mut self, name: Symbol, value: BakedValue) -> bool {
        match self.constant(name) {
            Some(previous) => previous == &value,
            None => {
                self.constants.push(ConstantBinding { name, value });
                true
            }
        }
    }
    pub fn key(&self, declaration: DeclarationId) -> SpecializationKey {
        SpecializationKey {
            declaration,
            substitution: self.clone(),
        }
    }
}

/// Semantic identity uses the originating declaration and typed arguments.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SpecializationKey {
    pub declaration: DeclarationId,
    pub substitution: Substitution,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SubstitutionError {
    UnboundType(Symbol),
    UnboundCount(Symbol),
    InvalidCount(Symbol),
    UnboundValue(Symbol),
    MissingNominalTemplate(DeclarationId),
    NominalResolution(String),
    Type(TypeError),
}
impl From<TypeError> for SubstitutionError {
    fn from(error: TypeError) -> Self {
        Self::Type(error)
    }
}

/// Intern a chosen signature only after overload selection has finished.
pub fn materialize(
    registry: &mut TypeRegistry,
    pattern: &crate::overloads::TypePattern,
    substitution: &Substitution,
) -> Result<TypeId, SubstitutionError> {
    materialize_with_nominals(registry, pattern, substitution, &mut |_, declaration, _| {
        Err(SubstitutionError::MissingNominalTemplate(declaration))
    })
}
pub fn materialize_with_nominals(
    registry: &mut TypeRegistry,
    pattern: &crate::overloads::TypePattern,
    substitution: &Substitution,
    nominal: &mut impl FnMut(
        &mut TypeRegistry,
        DeclarationId,
        Substitution,
    ) -> Result<TypeId, SubstitutionError>,
) -> Result<TypeId, SubstitutionError> {
    use crate::overloads::{CountPattern, NominalArgumentKind, TypePattern};
    Ok(match pattern {
        TypePattern::Concrete(ty) => {
            registry.kind(*ty)?;
            *ty
        }
        TypePattern::Infer(name) | TypePattern::Variable(name) => {
            let ty = substitution
                .ty(*name)
                .ok_or(SubstitutionError::UnboundType(*name))?;
            registry.kind(ty)?;
            ty
        }
        TypePattern::Restricted { ty, .. } => {
            materialize_with_nominals(registry, ty, substitution, nominal)?
        }
        TypePattern::Pointer(pointee) => {
            let ty = materialize_with_nominals(registry, pointee, substitution, nominal)?;
            registry.pointer(ty)?
        }
        TypePattern::FixedArray { element, count } => {
            let count = match count {
                CountPattern::Exact(count) => *count,
                CountPattern::Infer(name) | CountPattern::Variable(name) => {
                    let value = substitution
                        .constant(*name)
                        .ok_or(SubstitutionError::UnboundCount(*name))?;
                    value
                        .as_integer()
                        .and_then(|value| u64::try_from(value.value()).ok())
                        .ok_or(SubstitutionError::InvalidCount(*name))?
                }
            };
            let element = materialize_with_nominals(registry, element, substitution, nominal)?;
            registry.fixed_array(element, count)?
        }
        TypePattern::Slice(element) => {
            let element = materialize_with_nominals(registry, element, substitution, nominal)?;
            registry.slice(element)?
        }
        TypePattern::DynamicArray(element) => {
            let element = materialize_with_nominals(registry, element, substitution, nominal)?;
            registry.dynamic_array(element)?
        }
        TypePattern::Procedure(procedure) => {
            let parameters = procedure
                .parameters
                .iter()
                .map(|pattern| materialize_with_nominals(registry, pattern, substitution, nominal))
                .collect::<Result<Vec<_>, _>>()?;
            let mut results = procedure
                .results
                .iter()
                .map(|pattern| materialize_with_nominals(registry, pattern, substitution, nominal))
                .collect::<Result<Vec<_>, _>>()?;
            if results.len() == 1 && matches!(registry.kind(results[0]), Ok(TypeKind::Void)) {
                results.clear();
            }
            let variadic = match procedure.variadic {
                crate::overloads::CandidateVariadic::None => jai_types::Variadic::None,
                crate::overloads::CandidateVariadic::C { fixed_parameters } => {
                    jai_types::Variadic::C { fixed_parameters }
                }
                crate::overloads::CandidateVariadic::Jai { parameter } => {
                    let ty = parameters
                        .get(parameter)
                        .copied()
                        .ok_or(SubstitutionError::Type(TypeError::InvalidVariadic {
                            variadic: jai_types::Variadic::None,
                            issue: jai_types::VariadicIssue::ParameterOutOfBounds,
                        }))?;
                    let TypeKind::Slice(element) = registry.kind(ty)? else {
                        return Err(SubstitutionError::Type(TypeError::WrongKind(ty)));
                    };
                    jai_types::Variadic::Jai {
                        parameter,
                        element: *element,
                    }
                }
            };
            registry.procedure(jai_types::ProcedureType {
                parameters: parameters.into_boxed_slice(),
                results: results.into_boxed_slice(),
                convention: procedure.convention,
                context: procedure.context,
                variadic,
            })?
        }
        TypePattern::NominalApplication {
            declaration,
            arguments,
        } => {
            let mut bound = Substitution::default();
            for argument in arguments {
                let value = match &argument.kind {
                    NominalArgumentKind::Default => continue,
                    NominalArgumentKind::Type(pattern) => BakedValue::Type(
                        materialize_with_nominals(registry, pattern, substitution, nominal)?,
                    ),
                    NominalArgumentKind::InferValue(name)
                    | NominalArgumentKind::ValueVariable(name) => substitution
                        .constant(*name)
                        .cloned()
                        .ok_or(SubstitutionError::UnboundValue(*name))?,
                    NominalArgumentKind::Value(value) => value.clone(),
                };
                bound.bind_constant(argument.name, value);
            }
            nominal(registry, *declaration, bound)?
        }
    })
}

#[cfg(test)]
mod procedure_constant_keys {
    use super::*;

    #[test]
    fn baked_procedure_keys_include_canonical_signature_and_target_identity() {
        let mut types = TypeRegistry::new();
        let int = types.scalar(ScalarType::Int(jai_types::IntegerType::S64));
        let mut signature = jai_types::ProcedureType {
            parameters: Box::new([]),
            results: vec![int].into_boxed_slice(),
            convention: jai_types::CallingConvention::Jai,
            context: jai_types::ContextMode::None,
            variadic: jai_types::Variadic::None,
        };
        let jai = types.procedure(signature.clone()).unwrap();
        signature.convention = jai_types::CallingConvention::C;
        let c = types.procedure(signature).unwrap();
        let value = |ty, index| {
            BakedValue::runtime(
                ConstantValue {
                    ty,
                    kind: ConstantKind::Procedure(jai_ir::ProcedureId::new(index)),
                },
                &types,
            )
            .unwrap()
        };
        let first = value(jai, 1);
        let keys = std::collections::HashSet::from([first.clone(), value(jai, 2), value(c, 1)]);
        assert_eq!(keys.len(), 3);
        assert!(keys.contains(&value(jai, 1)));
        assert_eq!(
            first.into_runtime(jai, &types).unwrap().kind,
            ConstantKind::Procedure(jai_ir::ProcedureId::new(1))
        );
    }
}
