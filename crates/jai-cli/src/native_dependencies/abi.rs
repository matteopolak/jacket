//! Checked source signatures for the independently probed VMA virtual API.
use crate::Error;
use jai_codegen::{native_reachability::Reachable, target::NativeTarget};
use jai_sema::{ForeignLibraryId, Library};
use jai_types::{
    CallingConvention, ContextMode, IntegerType, LayoutEngine, RecordKind, TypeId, TypeKind,
    TypeView, Variadic,
};

#[derive(Clone, Copy)]
enum Shape {
    Integer(IntegerType),
    Pointer,
    PointerToPointer,
    PointerToInteger(IntegerType),
    PointerToRecord(Record),
}
#[derive(Clone, Copy)]
enum Record {
    Block,
    Allocation,
    Info,
}

pub(super) fn validate(
    library: &Library,
    target: &NativeTarget,
    reachable: &Reachable,
    dependency: ForeignLibraryId,
) -> Result<(), Error> {
    if library.globals().iter().any(|global| {
        reachable.contains_global(global.id())
            && matches!(global.initializer(),
            jai_sema::GlobalInitializer::External(data) if matches!(data.source(),
                jai_sema::ExternalDataSource::Library(owner) if owner.id == dependency))
    }) {
        return Err(Error::Source(
            "reachable external data is outside the reviewed VMA virtual ABI".into(),
        ));
    }
    let policy = target
        .layout_policy()
        .map_err(|error| Error::Source(error.to_string()))?;
    if policy.pointer().size != 8 || policy.pointer().alignment != 8 {
        return Err(Error::Source(
            "reviewed virtual allocator requires the proved 64-bit pointer ABI".into(),
        ));
    }
    let mut layouts = LayoutEngine::new(library.types(), policy);
    for prototype in library.prototypes() {
        let jai_sema::PrototypeOrigin::Foreign {
            symbol,
            library: Some(owner),
        } = &prototype.origin
        else {
            continue;
        };
        if owner.id != dependency || !reachable.contains(prototype.id) {
            continue;
        }
        validate_signature(library.types(), &mut layouts, prototype.signature, symbol)?;
    }
    Ok(())
}

fn validate_signature(
    types: &dyn TypeView,
    layouts: &mut LayoutEngine<'_>,
    ty: TypeId,
    symbol: &str,
) -> Result<(), Error> {
    use IntegerType::{S32, U32, U64};
    use Shape::*;
    let (parameters, result): (&[Shape], Option<IntegerType>) = match symbol {
        "vmaCreateVirtualBlock" => (
            &[PointerToRecord(Record::Block), PointerToPointer],
            Some(S32),
        ),
        "vmaDestroyVirtualBlock" => (&[Pointer], None),
        "vmaVirtualAllocate" => (
            &[
                Pointer,
                PointerToRecord(Record::Allocation),
                PointerToPointer,
                PointerToInteger(U64),
            ],
            Some(S32),
        ),
        "vmaVirtualFree" => (&[Pointer, Pointer], None),
        "vmaGetVirtualAllocationInfo" => (&[Pointer, Pointer, PointerToRecord(Record::Info)], None),
        "vmaIsVirtualBlockEmpty" => (&[Pointer], Some(U32)),
        _ => {
            return Err(Error::Source(format!(
                "native dependency symbol {symbol:?} is outside the reviewed VMA virtual ABI; no native dependency was linked"
            )));
        }
    };
    let procedure = types.procedure_definition(ty).map_err(source_error)?;
    if procedure.convention != CallingConvention::C
        || procedure.context != ContextMode::None
        || procedure.variadic != Variadic::None
        || procedure.parameters.len() != parameters.len()
        || procedure.results.len() != usize::from(result.is_some())
    {
        return Err(mismatch(symbol));
    }
    for (&ty, shape) in procedure.parameters.iter().zip(parameters) {
        check_shape(types, layouts, ty, *shape).map_err(|_| mismatch(symbol))?;
    }
    if let Some(integer) = result {
        check_shape(types, layouts, procedure.results[0], Integer(integer))
            .map_err(|_| mismatch(symbol))?;
    }
    Ok(())
}

fn mismatch(symbol: &str) -> Error {
    Error::Source(format!(
        "source prototype {symbol:?} does not match the reviewed native signature and field layout; no native dependency was linked"
    ))
}
fn source_error(error: jai_types::TypeError) -> Error {
    Error::Source(error.to_string())
}
fn representation(types: &dyn TypeView, mut ty: TypeId) -> Result<TypeId, Error> {
    for _ in 0..64 {
        if !matches!(types.kind(ty).map_err(source_error)?, TypeKind::Distinct(_)) {
            return Ok(ty);
        }
        ty = types
            .distinct_definition(ty)
            .map_err(source_error)?
            .representation;
    }
    Err(Error::Source(
        "native ABI distinct representation exceeds validation depth".into(),
    ))
}
fn pointee(types: &dyn TypeView, ty: TypeId) -> Result<TypeId, Error> {
    let ty = representation(types, ty)?;
    let TypeKind::Pointer(ty) = types.kind(ty).map_err(source_error)? else {
        return Err(Error::Source("native ABI requires a data pointer".into()));
    };
    Ok(*ty)
}
fn check_shape(
    types: &dyn TypeView,
    layouts: &mut LayoutEngine<'_>,
    ty: TypeId,
    shape: Shape,
) -> Result<(), Error> {
    match shape {
        Shape::Integer(expected) => {
            let ty = representation(types, ty)?;
            let actual = match types.kind(ty).map_err(source_error)? {
                TypeKind::Integer(integer) => *integer,
                TypeKind::Enum(_) => {
                    types
                        .enum_definition(ty)
                        .map_err(source_error)?
                        .representation
                }
                _ => {
                    return Err(Error::Source(
                        "native ABI requires an integer representation".into(),
                    ));
                }
            };
            if actual != expected {
                return Err(Error::Source(
                    "native ABI integer width or signedness differs".into(),
                ));
            }
        }
        Shape::Pointer => {
            pointee(types, ty)?;
        }
        Shape::PointerToPointer => {
            pointee(types, pointee(types, ty)?)?;
        }
        Shape::PointerToInteger(integer) => {
            check_shape(types, layouts, pointee(types, ty)?, Shape::Integer(integer))?
        }
        Shape::PointerToRecord(record) => {
            check_record(types, layouts, pointee(types, ty)?, record)?
        }
    }
    Ok(())
}
fn check_record(
    types: &dyn TypeView,
    layouts: &mut LayoutEngine<'_>,
    ty: TypeId,
    record: Record,
) -> Result<(), Error> {
    use IntegerType::{U32, U64};
    use Shape::{Integer, Pointer};
    let ty = representation(types, ty)?;
    let (size, offsets, fields): (u64, &[u64], &[Shape]) = match record {
        Record::Block => (24, &[0, 8, 16], &[Integer(U64), Integer(U32), Pointer]),
        Record::Allocation => (
            32,
            &[0, 8, 16, 24],
            &[Integer(U64), Integer(U64), Integer(U32), Pointer],
        ),
        Record::Info => (24, &[0, 8, 16], &[Integer(U64), Integer(U64), Pointer]),
    };
    let definition = types.record_definition(ty).map_err(source_error)?;
    let layout = layouts
        .layout(ty)
        .map_err(|error| Error::Source(error.to_string()))?;
    if definition.kind != RecordKind::Struct
        || definition.fields.len() != fields.len()
        || layout.size != size
        || layout.alignment != 8
        || layout.field_offsets.as_ref() != offsets
    {
        return Err(Error::Source(
            "native ABI record size/alignment/field offsets differ".into(),
        ));
    }
    for (&ty, &shape) in definition.fields.iter().zip(fields) {
        check_shape(types, layouts, ty, shape)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_types::{LayoutPolicy, ProcedureType, RecordLayout, ScalarType, TypeRegistry};

    fn create_signature(
        types: &mut TypeRegistry,
        size: IntegerType,
        layout: RecordLayout,
        convention: CallingConvention,
        context: ContextMode,
        variadic: Variadic,
    ) -> TypeId {
        let pointer = types.pointer(types.void()).unwrap();
        let output = types.pointer(pointer).unwrap();
        let record = types.reserve_record(RecordKind::Struct);
        let fields = vec![
            types.scalar(ScalarType::Int(size)),
            types.scalar(ScalarType::Int(IntegerType::U32)),
            pointer,
        ];
        types
            .define_record_with_layout(record, fields, layout)
            .unwrap();
        let input = types.pointer(record).unwrap();
        types
            .procedure(ProcedureType {
                parameters: vec![input, output].into_boxed_slice(),
                results: vec![types.scalar(ScalarType::Int(IntegerType::S32))].into_boxed_slice(),
                return_abi: jai_types::ForeignReturnAbi::Natural,
                convention,
                context,
                variadic,
            })
            .unwrap()
    }

    #[test]
    fn source_layout_and_signature_match_the_reviewed_virtual_block() {
        let mut types = TypeRegistry::default();
        let signature = create_signature(
            &mut types,
            IntegerType::U64,
            RecordLayout::default(),
            CallingConvention::C,
            ContextMode::None,
            Variadic::None,
        );
        let mut layouts = LayoutEngine::new(&types, LayoutPolicy::lp64());
        assert!(
            validate_signature(&types, &mut layouts, signature, "vmaCreateVirtualBlock").is_ok()
        );
        assert!(validate_signature(&types, &mut layouts, signature, "vmaCreateAllocator").is_err());
    }

    #[test]
    fn matching_record_size_cannot_hide_signedness_or_calling_contract_drift() {
        let cases = [
            (
                IntegerType::S64,
                RecordLayout::default(),
                CallingConvention::C,
                ContextMode::None,
                Variadic::None,
            ),
            (
                IntegerType::U32,
                RecordLayout::default(),
                CallingConvention::C,
                ContextMode::None,
                Variadic::None,
            ),
            (
                IntegerType::U64,
                RecordLayout {
                    packed: true,
                    ..RecordLayout::default()
                },
                CallingConvention::C,
                ContextMode::None,
                Variadic::None,
            ),
            (
                IntegerType::U64,
                RecordLayout::default(),
                CallingConvention::Jai,
                ContextMode::Implicit,
                Variadic::None,
            ),
            (
                IntegerType::U64,
                RecordLayout::default(),
                CallingConvention::C,
                ContextMode::None,
                Variadic::C {
                    fixed_parameters: 2,
                },
            ),
        ];
        for (size, layout, convention, context, variadic) in cases {
            let mut types = TypeRegistry::default();
            let signature =
                create_signature(&mut types, size, layout, convention, context, variadic);
            let mut layouts = LayoutEngine::new(&types, LayoutPolicy::lp64());
            assert!(
                validate_signature(&types, &mut layouts, signature, "vmaCreateVirtualBlock")
                    .is_err()
            );
        }
    }
}
