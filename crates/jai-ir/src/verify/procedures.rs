//! Validate the shared identity space of source definitions and external prototypes.
use super::*;

pub(super) fn declarations(library: &Library) -> Result<(), IrError> {
    let mut libraries = HashMap::new();
    for dependency in &library.foreign_libraries {
        dependency
            .validate()
            .map_err(|_| unknown("foreign library metadata", dependency.id.index()))?;
        if let ForeignLibraryId::Local {
            procedure, ..
        } = dependency.id
            && library.procedure_by_id(procedure).is_none()
            && library.source_procedure_owners().get(procedure).is_none()
        {
            return Err(unknown(
                "foreign library procedure owner",
                procedure.index(),
            ));
        }
        if libraries.insert(dependency.id, dependency).is_some() {
            return Err(IrError::DuplicateIdentity {
                kind: "foreign library",
                index: dependency.id.index(),
            });
        }
    }
    let mut identities = HashSet::new();
    for (index, procedure) in library.procedures.iter().enumerate() {
        if !identities.insert(procedure.id) {
            return Err(IrError::DuplicateIdentity {
                kind: "procedure",
                index: procedure.id.index(),
            });
        }
        if library.procedure_indices.get(&procedure.id) != Some(&index) {
            return Err(unknown("procedure index", procedure.id.index()));
        }
        same_type(
            procedure.signature,
            *library
                .signatures
                .get(&procedure.id)
                .ok_or_else(|| unknown("procedure signature", procedure.id.index()))?,
        )?;
        library.types.procedure_definition(procedure.signature)?;
    }
    for prototype in &library.prototypes {
        if !identities.insert(prototype.id) {
            return Err(IrError::DuplicateIdentity {
                kind: "procedure prototype",
                index: prototype.id.index(),
            });
        }
        same_type(
            prototype.signature,
            *library
                .signatures
                .get(&prototype.id)
                .ok_or_else(|| unknown("prototype signature", prototype.id.index()))?,
        )?;
        let signature = library.types.procedure_definition(prototype.signature)?;
        for &ty in signature.parameters.iter().chain(signature.results.iter()) {
            storage::runtime_type(&library.types, ty)?;
        }
        if let PrototypeOrigin::Intrinsic(intrinsic) = prototype.origin {
            intrinsic.validate_signature_shape(prototype.signature, &library.types)?;
        }
        if let PrototypeOrigin::SourceContract {
            symbol,
        } = &prototype.origin
        {
            if symbol.is_empty() || symbol.as_bytes().contains(&0) {
                return Err(unknown("source contract symbol", prototype.id.index()));
            }
            if signature.convention != jai_types::CallingConvention::Jai
                || !matches!(signature.variadic, jai_types::Variadic::Jai { .. })
            {
                return Err(unknown("source contract ABI", prototype.id.index()));
            }
        }
        if let PrototypeOrigin::Foreign {
            symbol,
            library: dependency,
        } = &prototype.origin
        {
            if let Some(dependency) = dependency {
                dependency
                    .validate()
                    .map_err(|_| unknown("foreign library metadata", dependency.id.index()))?;
                if libraries.get(&dependency.id).copied() != Some(dependency) {
                    return Err(unknown("foreign library binding", dependency.id.index()));
                }
            }
            if symbol.is_empty() || symbol.as_bytes().contains(&0) {
                return Err(unknown("foreign symbol", prototype.id.index()));
            }
            if !signature.convention.uses_c_abi()
                || signature.context != jai_types::ContextMode::None
            {
                return Err(unknown("foreign ABI", prototype.id.index()));
            }
        }
    }
    arity(
        "procedure signatures",
        identities.len(),
        library.signatures.len(),
    )?;
    for &id in library.procedure_ids.values() {
        if !identities.contains(&id) {
            return Err(unknown("procedure declaration", id.index()));
        }
    }
    Ok(())
}

impl Context<'_> {
    pub(super) fn indirect_call(
        &self,
        callee: &ValueExpr,
        arguments: &[(ParameterId, ValueExpr)],
        inline_hint: jai_types::InlineHint,
    ) -> Result<Vec<TypeId>, IrError> {
        let ty = self.value(callee)?;
        if inline_hint == jai_types::InlineHint::Always
            && !matches!(callee, ValueExpr::ProcedureValue { .. })
        {
            return Err(IrError::InvalidValue(ty));
        }
        let signature = self.types.procedure_definition(ty)?;
        self.call_context(signature)?;
        self.call_arguments(signature, arguments)?;
        Ok(signature.results.to_vec())
    }

    pub(super) fn call_arguments(
        &self,
        signature: &jai_types::ProcedureType,
        arguments: &[(ParameterId, ValueExpr)],
    ) -> Result<(), IrError> {
        let fixed = signature.parameters.len();
        match signature.variadic {
            jai_types::Variadic::C {
                ..
            } if arguments.len() < fixed => {
                return Err(IrError::Arity {
                    kind: "fixed call arguments",
                    expected: fixed,
                    actual: arguments.len(),
                });
            }
            jai_types::Variadic::C {
                ..
            } => {}
            _ => arity("call arguments", fixed, arguments.len())?,
        }
        let mut seen = HashSet::new();
        for (parameter, value) in arguments {
            if !seen.insert(*parameter) {
                return Err(IrError::DuplicateIdentity {
                    kind: "call parameter",
                    index: parameter.index(),
                });
            }
            let actual = if let ValueExpr::SequenceConcat {
                ty,
                parts,
            } = value
            {
                let jai_types::Variadic::Jai {
                    parameter: pack,
                    element,
                } = signature.variadic
                else {
                    return Err(IrError::InvalidValue(*ty));
                };
                if parameter.index() != pack {
                    return Err(IrError::InvalidValue(*ty));
                }
                self.sequence_concat(*ty, element, parts)?;
                *ty
            } else {
                self.value(value)?
            };
            if let Some(&expected) = signature.parameters.get(parameter.index()) {
                same_type(expected, actual)?;
            } else if matches!(signature.variadic, jai_types::Variadic::C { .. }) {
                match self.types.kind(actual)? {
                    TypeKind::Integer(integer) if integer.bits() >= 32 => {}
                    TypeKind::Float(jai_types::FloatType::F64) | TypeKind::Pointer(_) => {}
                    _ => return Err(IrError::InvalidValue(actual)),
                }
            } else {
                return Err(unknown("call parameter", parameter.index()));
            }
        }
        for index in 0..arguments.len() {
            if !seen.contains(&ParameterId::new(index)) {
                return Err(unknown("call parameter", index));
            }
        }
        Ok(())
    }
}
