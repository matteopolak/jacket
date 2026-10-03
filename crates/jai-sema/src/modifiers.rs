//! Execute checked specialization modifiers without publishing partial bindings.
use crate::compile_time::materialize;
use crate::polymorphism::{BakedValue, ConstantBinding, Substitution, TypeBinding};
use jai_ir::Call;
use jai_source::Symbol;
use jai_types::{TypeId, TypeView};
use jai_vm::{Dependency, Error, Outcome, ProcedureProvider, Value, Vm};
use std::collections::HashSet;

#[derive(Clone, Copy, Debug)]
pub enum ModifierSlot {
    Type { name: Symbol },
    Baked { name: Symbol, ty: TypeId },
}
impl ModifierSlot {
    fn name(self) -> Symbol {
        match self {
            Self::Type {
                name,
            }
            | Self::Baked {
                name, ..
            } => name,
        }
    }
}

/// A checked modifier returns acceptance, explanation, then these binding values.
#[derive(Clone, Debug)]
pub struct ModifierPlan {
    slots: Vec<ModifierSlot>,
}
impl ModifierPlan {
    pub fn new(slots: Vec<ModifierSlot>) -> Result<Self, Error> {
        let mut names = HashSet::new();
        if slots.iter().any(|slot| !names.insert(slot.name())) {
            return Err(Error::InvalidIr("duplicate specialization modifier slot"));
        }
        Ok(Self {
            slots,
        })
    }
    pub fn slots(&self) -> &[ModifierSlot] {
        &self.slots
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ModifierOutcome {
    Accepted(Substitution),
    Rejected { reason: String },
    Pending(Vec<Dependency>),
    Failed(Error),
}

/// The VM proves the actual returned descriptor belongs to the active registry.
/// No numeric pointer-to-TypeId conversion is permitted by this interface.
pub fn execute<P: ProcedureProvider + ?Sized>(
    vm: &mut Vm<'_, P, jai_vm::NoEffects>,
    call: &Call,
    plan: &ModifierPlan,
    initial: &Substitution,
    types: &dyn TypeView,
    maximum_depth: usize,
) -> ModifierOutcome {
    for slot in plan.slots() {
        if let ModifierSlot::Baked {
            name,
            ty,
        } = *slot
        {
            let actual = match initial.constant(name) {
                Some(BakedValue::Value(value)) => value.ty,
                Some(BakedValue::Float(value)) => types.float(value.ty()),
                Some(BakedValue::String(_)) => match types.lookup(&jai_types::TypeKind::String) {
                    Some(ty) => ty,
                    None => {
                        return ModifierOutcome::Failed(Error::InvalidIr(
                            "modifier registry has no string type",
                        ));
                    }
                },
                Some(BakedValue::Code(_)) => types.code_type(),
                Some(BakedValue::Type(_)) | None => {
                    return ModifierOutcome::Failed(Error::InvalidIr(
                        "baked modifier slot requires an existing value binding",
                    ));
                }
            };
            if actual != ty {
                return ModifierOutcome::Failed(Error::TypeMismatch {
                    expected: actual,
                });
            }
            if let Err(error) = types.kind(ty) {
                return ModifierOutcome::Failed(error.into());
            }
        }
    }
    let mut staged = None;
    let mut rejection = None;
    let execution = vm.evaluate_call_validated(call, |vm, values| {
        if values.len() != plan.slots.len() + 2 {
            return Err(Error::InvalidIr(
                "specialization modifier result count mismatch",
            ));
        }
        let Value::Bool(accept) = values[0] else {
            return Err(Error::InvalidIr(
                "specialization modifier acceptance must be bool",
            ));
        };
        let Value::String(reason) = vm.materialize_value(&values[1])? else {
            return Err(Error::InvalidIr(
                "specialization modifier explanation must be string",
            ));
        };
        if !accept {
            let reason = String::from_utf8_lossy(&reason).into_owned();
            rejection = Some(reason.clone());
            return Err(Error::CompilerReported(reason));
        }
        let mut draft = initial.clone();
        for (slot, value) in plan.slots.iter().zip(&values[2..]) {
            match *slot {
                ModifierSlot::Type {
                    name,
                } => {
                    let meta = types
                        .lookup(&jai_types::TypeKind::Type)
                        .ok_or(Error::InvalidIr("modifier registry has no Type metatype"))?;
                    value.validate(types, meta, maximum_depth)?;
                    let identity = vm.runtime_type_identity(value)?;
                    identity.validate(types)?;
                    let ty = identity.ty();
                    let mut updated = false;
                    if let Some(binding) =
                        draft.types.iter_mut().find(|binding| binding.name == name)
                    {
                        binding.ty = ty;
                        updated = true;
                    }
                    if let Some(binding) = draft
                        .constants
                        .iter_mut()
                        .find(|binding| binding.name == name)
                    {
                        if !matches!(binding.value, BakedValue::Type(_)) {
                            return Err(Error::InvalidIr(
                                "modifier cannot change a baked value into a type",
                            ));
                        }
                        binding.value = BakedValue::Type(ty);
                        updated = true;
                    }
                    if !updated {
                        draft.types.push(TypeBinding {
                            name,
                            ty,
                        });
                    }
                }
                ModifierSlot::Baked {
                    name,
                    ty,
                } => {
                    if draft.types.iter().any(|binding| binding.name == name) {
                        return Err(Error::InvalidIr(
                            "modifier cannot change a type binding into a baked value",
                        ));
                    }
                    let constant =
                        materialize(types, vm.materialize_value(value)?, ty, maximum_depth)?;
                    let value = BakedValue::runtime(constant, types)?;
                    if let Some(binding) = draft
                        .constants
                        .iter_mut()
                        .find(|binding| binding.name == name)
                    {
                        binding.value = value;
                    } else {
                        draft.constants.push(ConstantBinding {
                            name,
                            value,
                        });
                    }
                }
            }
        }
        staged = Some(draft);
        Ok(())
    });
    match execution.outcome {
        Outcome::Complete(_) => match staged {
            Some(substitution) => ModifierOutcome::Accepted(substitution),
            None => ModifierOutcome::Failed(Error::InvalidIr(
                "modifier completed without validated bindings",
            )),
        },
        Outcome::Pending(dependencies) => ModifierOutcome::Pending(dependencies),
        Outcome::Failed(Error::CompilerReported(reason)) if rejection.as_ref() == Some(&reason) => {
            ModifierOutcome::Rejected {
                reason,
            }
        }
        Outcome::Failed(error) => ModifierOutcome::Failed(error),
    }
}

#[cfg(test)]
mod tests;
