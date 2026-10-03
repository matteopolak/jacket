use super::*;
use jai_source::Identities;
use jai_types::{ProcedureType, ScalarType, TypeRegistry};
fn library() -> ForeignLibrary {
    ForeignLibrary {
        id: ForeignLibraryId::new(Identities::default().declaration()),
        kind: ForeignLibraryKind::System {
            name: "libc".into(),
        },
        options: Default::default(),
    }
}
fn signature(types: &mut TypeRegistry, operation: HeapAbiOperation) -> TypeId {
    let pointer = types.pointer(types.void()).unwrap();
    let size = types.scalar(ScalarType::Int(IntegerType::U64));
    let (parameters, results) = match operation {
        HeapAbiOperation::Malloc => (vec![size], vec![pointer]),
        HeapAbiOperation::Realloc => (vec![pointer, size], vec![pointer]),
        HeapAbiOperation::Free => (vec![pointer], vec![]),
    };
    types
        .procedure(ProcedureType {
            parameters: parameters.into(),
            results: results.into(),
            convention: CallingConvention::C,
            context: ContextMode::None,
            variadic: Variadic::None,
        })
        .unwrap()
}
#[test]
fn exact_decl_library_origin_and_full_c_shapes_are_required() {
    let mut types = TypeRegistry::new();
    let library = library();
    for (index, operation) in [
        HeapAbiOperation::Malloc,
        HeapAbiOperation::Realloc,
        HeapAbiOperation::Free,
    ]
    .into_iter()
    .enumerate()
    {
        let id = ProcedureId::new(index);
        let signature = signature(&mut types, operation);
        let authority =
            HeapAuthority::from_verified_source(library.clone(), [(id, operation)]).unwrap();
        let prototype = ProcedurePrototype {
            id,
            signature,
            origin: PrototypeOrigin::Foreign {
                symbol: operation.symbol().into(),
                library: Some(library.clone()),
            },
        };
        let bound = authority.bind(&prototype, &types).unwrap();
        assert_eq!(bound.procedure(), id);
        bound.validate(&types).unwrap();
        let mut wrong = prototype.clone();
        wrong.id = ProcedureId::new(index + 10);
        assert!(authority.bind(&wrong, &types).is_err());
        let mut wrong = prototype.clone();
        wrong.origin = PrototypeOrigin::Foreign {
            symbol: operation.symbol().into(),
            library: None,
        };
        assert!(authority.bind(&wrong, &types).is_err());
        let mut wrong = prototype.clone();
        wrong.origin = PrototypeOrigin::Compiler;
        assert!(authority.bind(&wrong, &types).is_err());
        let mut wrong = prototype.clone();
        wrong.signature = types
            .procedure(ProcedureType {
                parameters: [].into(),
                results: [].into(),
                convention: CallingConvention::C,
                context: ContextMode::None,
                variadic: Variadic::None,
            })
            .unwrap();
        assert!(authority.bind(&wrong, &types).is_err());
    }
}
#[test]
fn wrong_width_context_convention_and_variadic_are_rejected() {
    let mut types = TypeRegistry::new();
    let pointer = types.pointer(types.void()).unwrap();
    let id = ProcedureId::new(1);
    let library = library();
    let authority =
        HeapAuthority::from_verified_source(library.clone(), [(id, HeapAbiOperation::Malloc)])
            .unwrap();
    for (size, convention, context, variadic) in [
        (
            IntegerType::S64,
            CallingConvention::C,
            ContextMode::None,
            Variadic::None,
        ),
        (
            IntegerType::U32,
            CallingConvention::C,
            ContextMode::None,
            Variadic::None,
        ),
        (
            IntegerType::U64,
            CallingConvention::Jai,
            ContextMode::None,
            Variadic::None,
        ),
        (
            IntegerType::U64,
            CallingConvention::C,
            ContextMode::Implicit,
            Variadic::None,
        ),
        (
            IntegerType::U64,
            CallingConvention::C,
            ContextMode::None,
            Variadic::C {
                fixed_parameters: 1,
            },
        ),
    ] {
        let size = types.scalar(ScalarType::Int(size));
        let signature = types
            .procedure(ProcedureType {
                parameters: [size].into(),
                results: [pointer].into(),
                convention,
                context,
                variadic,
            })
            .unwrap();
        let prototype = ProcedurePrototype {
            id,
            signature,
            origin: PrototypeOrigin::Foreign {
                symbol: "malloc".into(),
                library: Some(library.clone()),
            },
        };
        assert!(authority.bind(&prototype, &types).is_err());
    }
}
#[test]
fn adapter_executes_only_virtual_storage_and_free_has_no_results() {
    let mut types = TypeRegistry::new();
    let library = library();
    let malloc = ProcedureId::new(1);
    let free = ProcedureId::new(2);
    let authority = HeapAuthority::from_verified_source(
        library.clone(),
        [
            (malloc, HeapAbiOperation::Malloc),
            (free, HeapAbiOperation::Free),
        ],
    )
    .unwrap();
    let make = |id, signature, symbol: &str| ProcedurePrototype {
        id,
        signature,
        origin: PrototypeOrigin::Foreign {
            symbol: symbol.into(),
            library: Some(library.clone()),
        },
    };
    let malloc_signature = signature(&mut types, HeapAbiOperation::Malloc);
    let free_signature = signature(&mut types, HeapAbiOperation::Free);
    let malloc = authority
        .bind(&make(malloc, malloc_signature, "malloc"), &types)
        .unwrap();
    let free = authority
        .bind(&make(free, free_signature, "free"), &types)
        .unwrap();
    let mut memory = Memory::new(crate::Limits::default());
    let mut heap = VirtualHeap::default();
    let args = [Value::Int(jai_types::Integer::wrapping(
        IntegerType::U64,
        32,
    ))];
    assert!(malloc.work_cost(&args, &memory, &types, &heap).unwrap() >= 32);
    let result = malloc
        .invoke(&args, &mut memory, &types, &mut heap)
        .unwrap();
    assert_eq!(heap.live_bytes(), 32);
    assert!(
        free.invoke(&result, &mut memory, &types, &mut heap)
            .unwrap()
            .is_empty()
    );
    assert_eq!(heap.allocation_count(), 0);
}
