use super::*;
use jai_source::Identities;
use jai_types::{ProcedureType, RecordKind, ScalarType, TypeRegistry};
fn library() -> ForeignLibrary {
    ForeignLibrary {
        id: ForeignLibraryId::new(Identities::default().declaration()),
        kind: ForeignLibraryKind::System {
            name: "libc".into(),
        },
        options: Default::default(),
    }
}
fn signature(types: &mut TypeRegistry, parameters: &[TypeId], results: &[TypeId]) -> TypeId {
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
fn file(types: &mut TypeRegistry) -> TypeId {
    let file = types.reserve_record(RecordKind::Struct);
    types.define_record(file, []).unwrap();
    file
}
fn prototype(
    id: ProcedureId,
    signature: TypeId,
    name: &str,
    library: ForeignLibrary,
) -> ProcedurePrototype {
    ProcedurePrototype {
        id,
        signature,
        origin: PrototypeOrigin::Foreign {
            symbol: name.into(),
            library: Some(library),
        },
    }
}
#[test]
fn full_stdio_catalog_validates_c_shapes_and_nominal_file_identity() {
    let mut types = TypeRegistry::new();
    let file = file(&mut types);
    let stream = types.pointer(file).unwrap();
    let void = types.pointer(types.void()).unwrap();
    let u8 = types.scalar(ScalarType::Int(IntegerType::U8));
    let text = types.pointer(u8).unwrap();
    let u64 = types.scalar(ScalarType::Int(IntegerType::U64));
    let s64 = types.scalar(ScalarType::Int(IntegerType::S64));
    let s32 = types.scalar(ScalarType::Int(IntegerType::S32));
    let library = library();
    let shapes = [
        (FileAbiOperation::Open, "fopen", vec![text, text], stream),
        (
            FileAbiOperation::Read,
            "fread",
            vec![void, u64, u64, stream],
            u64,
        ),
        (
            FileAbiOperation::Write,
            "fwrite",
            vec![void, u64, u64, stream],
            u64,
        ),
        (FileAbiOperation::Seek, "fseek", vec![stream, s64, s32], s32),
        (FileAbiOperation::Tell, "ftello", vec![stream], s64),
        (FileAbiOperation::Eof, "feof", vec![stream], s32),
        (FileAbiOperation::Close, "fclose", vec![stream], s32),
    ];
    for (index, (operation, name, parameters, result)) in shapes.into_iter().enumerate() {
        let id = ProcedureId::new(index);
        let signature = signature(&mut types, &parameters, &[result]);
        let authority =
            StdioAuthority::from_verified_source(library.clone(), file, [(id, operation)], &types)
                .unwrap();
        let bound = authority
            .bind(&prototype(id, signature, name, library.clone()), &types)
            .unwrap();
        assert_eq!(bound.procedure(), id);
        assert_eq!(bound.library(), library.id);
        assert_eq!(bound.operation(), operation);
        assert_eq!(bound.file_type(), file);
        bound.validate(&types).unwrap();
    }
}
#[test]
fn matching_spelling_without_exact_decl_library_receipt_never_grants_power() {
    let mut types = TypeRegistry::new();
    let file = file(&mut types);
    let stream = types.pointer(file).unwrap();
    let s32 = types.scalar(ScalarType::Int(IntegerType::S32));
    let signature = signature(&mut types, &[stream], &[s32]);
    let library = library();
    let id = ProcedureId::new(1);
    let authority = StdioAuthority::from_verified_source(
        library.clone(),
        file,
        [(id, FileAbiOperation::Close)],
        &types,
    )
    .unwrap();
    let good = prototype(id, signature, "fclose", library.clone());
    assert!(authority.bind(&good, &types).is_ok());
    let mut wrong = good.clone();
    wrong.id = ProcedureId::new(2);
    assert!(authority.bind(&wrong, &types).is_err());
    let mut wrong = good.clone();
    wrong.origin = PrototypeOrigin::Compiler;
    assert!(authority.bind(&wrong, &types).is_err());
    let mut wrong = good.clone();
    wrong.origin = PrototypeOrigin::Foreign {
        symbol: "fclose".into(),
        library: None,
    };
    assert!(authority.bind(&wrong, &types).is_err());
    let mut other = library.clone();
    let mut identities = Identities::default();
    identities.declaration();
    other.id = ForeignLibraryId::new(identities.declaration());
    assert!(
        authority
            .bind(&prototype(id, signature, "fclose", other), &types)
            .is_err()
    );
    assert!(
        authority
            .bind(&prototype(id, signature, "fopen", library), &types)
            .is_err()
    );
}
#[test]
fn wrong_pointer_nominal_width_context_convention_and_varargs_are_rejected() {
    let mut types = TypeRegistry::new();
    let file = file(&mut types);
    let stream = types.pointer(file).unwrap();
    let other = self::file(&mut types);
    let other_stream = types.pointer(other).unwrap();
    let s32 = types.scalar(ScalarType::Int(IntegerType::S32));
    let s64 = types.scalar(ScalarType::Int(IntegerType::S64));
    let library = library();
    let id = ProcedureId::new(1);
    let authority = StdioAuthority::from_verified_source(
        library.clone(),
        file,
        [(id, FileAbiOperation::Close)],
        &types,
    )
    .unwrap();
    for (pointer, result, convention, context, variadic) in [
        (
            other_stream,
            s32,
            CallingConvention::C,
            ContextMode::None,
            Variadic::None,
        ),
        (
            stream,
            s64,
            CallingConvention::C,
            ContextMode::None,
            Variadic::None,
        ),
        (
            stream,
            s32,
            CallingConvention::Jai,
            ContextMode::None,
            Variadic::None,
        ),
        (
            stream,
            s32,
            CallingConvention::C,
            ContextMode::Implicit,
            Variadic::None,
        ),
        (
            stream,
            s32,
            CallingConvention::C,
            ContextMode::None,
            Variadic::C {
                fixed_parameters: 1,
            },
        ),
    ] {
        let signature = types
            .procedure(ProcedureType {
                parameters: vec![pointer].into(),
                results: vec![result].into(),
                convention,
                context,
                variadic,
            })
            .unwrap();
        assert!(
            authority
                .bind(&prototype(id, signature, "fclose", library.clone()), &types)
                .is_err()
        );
    }
}
