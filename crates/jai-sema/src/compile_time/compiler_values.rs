//! Selected native facts are copied together while their controller frame is live.
use super::{
    CompilerEffects, ConstantValue, Limits, ProcedureProvider, TypeId, TypeView, Value, Vm,
};
use jai_vm::CompilerSlotId;

pub(crate) fn materialize_compiler_captures<P, E>(
    vm: &Vm<'_, P, E>,
    types: &dyn TypeView,
    inputs: &[(CompilerSlotId, TypeId, &Value)],
    mut limits: Limits,
    _publication_occurrence: u64,
) -> Result<Vec<(CompilerSlotId, ConstantValue)>, jai_vm::Error>
where
    P: ProcedureProvider + ?Sized,
    E: CompilerEffects,
{
    limits.value_cells = limits.value_cells.min(
        vm.publication_value_cell_limit()
            .checked_sub(vm.publication_retained_cells()?)
            .ok_or(jai_vm::Error::Limit(jai_vm::LimitKind::ValueCells))?,
    );
    let input_cells = inputs
        .len()
        .checked_mul(3)
        .filter(|n| *n <= limits.value_cells)
        .ok_or(jai_vm::Error::Limit(jai_vm::LimitKind::ValueCells))?;
    let mut seen = std::collections::HashSet::new();
    let mut borrowed = Vec::with_capacity(inputs.len());
    for &(slot, ty, value) in inputs {
        if !seen.insert(slot) {
            return Err(jai_vm::Error::InvalidIr(
                "compiler native capture slot is repeated",
            ));
        }
        super::materializable_type(types, ty, limits.evaluation_depth)?;
        borrowed.push(value);
    }
    let mut output_limits = limits;
    // Materialized values and the portable constant recipe coexist during
    // conversion. Their two complete forests share this admitted residual.
    output_limits.value_cells = (output_limits.value_cells - input_cells) / 2;
    let values = vm.materialize_borrowed_values(&borrowed, output_limits)?;
    inputs
        .iter()
        .zip(values)
        .map(|(&(slot, ty, _), value)| {
            let constant = super::materialize_with_runtime_types(
                types,
                value,
                ty,
                limits.evaluation_depth,
                &mut |value| vm.runtime_type_constant_value(value),
            )?;
            Ok((slot, constant))
        })
        .collect()
}
