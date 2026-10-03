//! Count the expanded portable type recipe before its encoder allocates it.
use super::aggregates::parameterized::RecordSpecializations;
use crate::{Diagnostic, polymorphism::BakedValue};
use jai_source::SourceSpan;
use jai_types::{TypeId, TypeKind, TypeRegistry, TypeView};

impl super::FileScope<'_> {
    pub(crate) fn insertion_capture_type_cost(
        &self,
        types: &TypeRegistry,
        records: &RecordSpecializations,
        ty: TypeId,
        location: SourceSpan,
        node_limit: usize,
        byte_limit: usize,
    ) -> Result<InsertionCaptureCost, Diagnostic> {
        insertion_capture_type_cost(types, records, ty, location, node_limit, byte_limit)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct InsertionCaptureCost {
    pub(crate) nodes: usize,
    pub(crate) bytes: usize,
}

enum Work<'a> {
    Type(TypeId, usize),
    Types(&'a [TypeId], usize),
    Argument(&'a BakedValue, usize),
    Arguments(&'a [BakedValue], usize),
}

/// This is a resource preflight, not source-identity admission. The existing
/// source encoder must still prove each nominal, specialization, and callable.
pub(crate) fn insertion_capture_type_cost(
    types: &TypeRegistry,
    records: &RecordSpecializations,
    ty: TypeId,
    location: SourceSpan,
    node_limit: usize,
    byte_limit: usize,
) -> Result<InsertionCaptureCost, Diagnostic> {
    count(
        types,
        records,
        Work::Type(ty, 0),
        location,
        node_limit,
        byte_limit,
    )
}

fn count<'a>(
    types: &'a TypeRegistry,
    records: &'a RecordSpecializations,
    first: Work<'a>,
    location: SourceSpan,
    node_limit: usize,
    byte_limit: usize,
) -> Result<InsertionCaptureCost, Diagnostic> {
    let mut cost = InsertionCaptureCost::default();
    let mut pending = vec![first];
    let fail = |message: &str| Diagnostic::at_source(location, message);
    while let Some(work) = pending.pop() {
        match work {
            Work::Types(values, depth) => {
                if let Some((first, rest)) = values.split_first() {
                    pending.push(Work::Types(rest, depth));
                    pending.push(Work::Type(*first, depth));
                }
            }
            Work::Arguments(values, depth) => {
                if let Some((first, rest)) = values.split_first() {
                    pending.push(Work::Arguments(rest, depth));
                    pending.push(Work::Argument(first, depth));
                }
            }
            Work::Type(ty, depth) => {
                if depth >= 128 {
                    return Err(fail(
                        "module source type identity exceeds the recursion limit",
                    ));
                }
                add(&mut cost.nodes, 1, node_limit, location)?;
                let kind = types.kind(ty).map_err(|error| fail(&error.to_string()))?;
                if let Some(key) = records.key_for_type(ty) {
                    pending.push(Work::Arguments(&key.arguments, depth + 1));
                    continue;
                }
                match kind {
                    TypeKind::Pointer(inner)
                    | TypeKind::Slice(inner)
                    | TypeKind::DynamicArray(inner) => {
                        pending.push(Work::Type(*inner, depth + 1));
                    }
                    TypeKind::FixedArray {
                        element, ..
                    } => {
                        pending.push(Work::Type(*element, depth + 1));
                    }
                    TypeKind::Procedure(id) => {
                        let signature = types
                            .procedure_type(*id)
                            .map_err(|error| fail(&error.to_string()))?;
                        pending.push(Work::Types(&signature.results, depth + 1));
                        // The current encoder builds the declared parameter
                        // first, then replaces a Jai pack with its element.
                        // Count both allocations before admitting either one.
                        if let jai_types::Variadic::Jai {
                            element, ..
                        } = signature.variadic
                        {
                            pending.push(Work::Type(element, depth + 1));
                        }
                        pending.push(Work::Types(&signature.parameters, depth + 1));
                    }
                    _ => {}
                }
            }
            Work::Argument(value, depth) => {
                add(&mut cost.nodes, 1, node_limit, location)?;
                match value {
                    BakedValue::Type(ty) => pending.push(Work::Type(*ty, depth)),
                    BakedValue::String(value) => {
                        add(&mut cost.bytes, value.len(), byte_limit, location)?;
                    }
                    BakedValue::Value(value) => match &value.kind {
                        jai_ir::ConstantKind::StringBytes(value) => {
                            add(&mut cost.bytes, value.len(), byte_limit, location)?;
                        }
                        jai_ir::ConstantKind::Enum(_) => {
                            pending.push(Work::Type(value.ty, depth));
                        }
                        _ => {}
                    },
                    BakedValue::Float(_) | BakedValue::Code(_) => {}
                }
            }
        }
    }
    Ok(cost)
}

fn add(
    used: &mut usize,
    count: usize,
    limit: usize,
    location: SourceSpan,
) -> Result<(), Diagnostic> {
    *used = used
        .checked_add(count)
        .filter(|next| *next <= limit)
        .ok_or_else(|| {
            Diagnostic::at_source(
                location,
                "declaration capture exceeds its portable value budget",
            )
        })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_source::{SourceMap, Span};
    use jai_types::{CallingConvention, ContextMode, ProcedureType, ScalarType, Variadic};

    fn location() -> SourceSpan {
        let mut sources = SourceMap::default();
        SourceSpan {
            source: sources.insert("capture.jai".into(), "#code {}".into()),
            span: Span::new(0, 8),
        }
    }

    fn pair(types: &mut TypeRegistry, child: TypeId) -> TypeId {
        types
            .procedure(ProcedureType {
                parameters: vec![child, child].into_boxed_slice(),
                results: Box::new([]),
                return_abi: jai_types::ForeignReturnAbi::Natural,
                convention: CallingConvention::Jai,
                context: ContextMode::None,
                variadic: Variadic::None,
            })
            .unwrap()
    }

    #[test]
    fn shared_registry_types_charge_every_portable_occurrence() {
        let mut types = TypeRegistry::new();
        let records = RecordSpecializations::default();
        let mut ty = types.scalar(ScalarType::Bool);
        for _ in 0..10 {
            ty = pair(&mut types, ty);
        }
        let location = location();
        assert_eq!(
            insertion_capture_type_cost(&types, &records, ty, location, 2_047, 0).unwrap(),
            InsertionCaptureCost {
                nodes: 2_047,
                bytes: 0,
            }
        );
        let error =
            insertion_capture_type_cost(&types, &records, ty, location, 2_046, 0).unwrap_err();
        assert_eq!(error.span, location.span);
        assert!(error.message.contains("portable value budget"));
    }

    #[test]
    fn oversized_type_recipe_stops_before_encoding_an_exponential_tree() {
        let mut types = TypeRegistry::new();
        let records = RecordSpecializations::default();
        let mut ty = types.scalar(ScalarType::Bool);
        for _ in 0..64 {
            ty = pair(&mut types, ty);
        }
        assert!(insertion_capture_type_cost(&types, &records, ty, location(), 1_024, 0).is_err());
    }

    #[test]
    fn source_argument_bytes_and_type_recipes_share_one_preflight_budget() {
        let types = TypeRegistry::new();
        let records = RecordSpecializations::default();
        let values = [
            BakedValue::String(vec![0, 255, 17, 23].into_boxed_slice()),
            BakedValue::Type(types.scalar(ScalarType::Bool)),
        ];
        assert_eq!(
            count(
                &types,
                &records,
                Work::Arguments(&values, 1),
                location(),
                3,
                4,
            )
            .unwrap(),
            InsertionCaptureCost {
                nodes: 3,
                bytes: 4
            }
        );
        assert!(
            count(
                &types,
                &records,
                Work::Arguments(&values, 1),
                location(),
                3,
                3,
            )
            .is_err()
        );
    }
}
