use jai_ir::*;
use jai_source::{DeclarationId, Identities};
use jai_types::{
    CallingConvention, ContextMode, ProcedureType, ScalarType, TypeRegistry, Variadic,
};
use std::collections::HashMap;

fn declaration() -> DeclarationId {
    Identities::default().declaration()
}

fn export(declaration: DeclarationId, target: ExportTarget, name: &str) -> ProgramExport {
    ProgramExport {
        declaration,
        target,
        symbol: NativeSymbol::new(name).unwrap(),
    }
}

fn fixture() -> (ProgramBuilder, jai_types::TypeId) {
    let mut types = TypeRegistry::new();
    let signature = types
        .procedure(ProcedureType {
            parameters: Box::new([]),
            results: Box::new([]),
            return_abi: jai_types::ForeignReturnAbi::Natural,
            convention: CallingConvention::C,
            context: ContextMode::None,
            variadic: Variadic::None,
        })
        .unwrap();
    let global = Global::new(0, GlobalInitializer::Bool(false), &types);
    let procedures = [0, 1]
        .into_iter()
        .map(|id| Procedure {
            id: ProcedureId::new(id),
            signature,
            parameters: vec![],
            locals: vec![],
            body: Block {
                statements: vec![],
                flow: Flow::FallsThrough,
            },
            cleanups: vec![],
        })
        .collect();
    (
        ProgramBuilder::new(types.freeze().unwrap())
            .procedures(procedures)
            .globals(vec![global]),
        signature,
    )
}

#[test]
fn checked_source_mapping_must_identify_the_exported_target() {
    let source = declaration();
    for target in [
        ExportTarget::Procedure(ProcedureId::new(1)),
        ExportTarget::Global(GlobalId::new(0)),
    ] {
        let (builder, _) = fixture();
        assert!(
            matches!(builder.declarations(HashMap::from([(source, ProcedureId::new(0))]))
            .program_exports(vec![export(source, target, "own_symbol")]).finish_library(),
            Err(IrError::ProgramExport(ExportError::DeclarationTargetMismatch(id))) if id == source)
        );
    }
    let (builder, _) = fixture();
    assert!(
        builder
            .declarations(HashMap::from([(source, ProcedureId::new(0))]))
            .program_exports(vec![export(
                source,
                ExportTarget::Procedure(ProcedureId::new(0)),
                "own_symbol"
            )])
            .finish_library()
            .is_ok()
    );
}

#[test]
fn manually_staged_exports_need_no_optional_source_mapping() {
    let (builder, _) = fixture();
    let library = builder
        .program_exports(vec![export(
            declaration(),
            ExportTarget::Global(GlobalId::new(0)),
            "own_global",
        )])
        .finish_library()
        .unwrap();
    assert_eq!(library.program_exports()[0].symbol.as_str(), "own_global");
}

#[test]
fn export_names_declarations_and_targets_are_unique() {
    let source = declaration();
    let p0 = ExportTarget::Procedure(ProcedureId::new(0));
    let p1 = ExportTarget::Procedure(ProcedureId::new(1));
    let cases = [
        vec![
            export(source, p0, "same"),
            export(declaration(), p1, "same"),
        ],
        vec![export(source, p0, "first"), export(source, p1, "second")],
        vec![
            export(source, p0, "first"),
            export(declaration(), p0, "second"),
        ],
    ];
    for exports in cases {
        let (builder, _) = fixture();
        assert!(matches!(
            builder.program_exports(exports).finish_library(),
            Err(IrError::ProgramExport(_))
        ));
    }
}

#[test]
fn foreign_symbol_alias_requires_identical_checked_signature() {
    let (builder, signature) = fixture();
    let prototype = ProcedurePrototype {
        id: ProcedureId::new(2),
        signature,
        origin: PrototypeOrigin::Foreign {
            symbol: "own_symbol".into(),
            library: None,
        },
    };
    assert!(
        builder
            .prototypes(vec![prototype.clone()])
            .program_exports(vec![export(
                declaration(),
                ExportTarget::Procedure(ProcedureId::new(0)),
                "own_symbol"
            )])
            .finish_library()
            .is_ok()
    );
    let (builder, _) = fixture();
    assert!(matches!(
        builder
            .prototypes(vec![prototype])
            .program_exports(vec![export(
                declaration(),
                ExportTarget::Global(GlobalId::new(0)),
                "own_symbol"
            )])
            .finish_library(),
        Err(IrError::ProgramExport(
            ExportError::ConflictingForeignSymbol(_)
        ))
    ));

    let mut types = TypeRegistry::new();
    let signature = types
        .procedure(ProcedureType {
            parameters: Box::new([]),
            results: Box::new([]),
            return_abi: jai_types::ForeignReturnAbi::Natural,
            convention: CallingConvention::C,
            context: ContextMode::None,
            variadic: Variadic::None,
        })
        .unwrap();
    let integer = types.scalar(ScalarType::Int(jai_types::IntegerType::S32));
    let different = types
        .procedure(ProcedureType {
            parameters: vec![integer].into_boxed_slice(),
            results: Box::new([]),
            return_abi: jai_types::ForeignReturnAbi::Natural,
            convention: CallingConvention::C,
            context: ContextMode::None,
            variadic: Variadic::None,
        })
        .unwrap();
    let body = Procedure {
        id: ProcedureId::new(0),
        signature,
        parameters: vec![],
        locals: vec![],
        cleanups: vec![],
        body: Block {
            statements: vec![],
            flow: Flow::FallsThrough,
        },
    };
    let prototype = ProcedurePrototype {
        id: ProcedureId::new(1),
        signature: different,
        origin: PrototypeOrigin::Foreign {
            symbol: "own_symbol".into(),
            library: None,
        },
    };
    assert!(matches!(
        ProgramBuilder::new(types.freeze().unwrap())
            .procedures(vec![body])
            .prototypes(vec![prototype])
            .program_exports(vec![export(
                declaration(),
                ExportTarget::Procedure(ProcedureId::new(0)),
                "own_symbol"
            )])
            .finish_library(),
        Err(IrError::ProgramExport(
            ExportError::ConflictingForeignSymbol(_)
        ))
    ));
}

#[test]
fn undefined_exports_and_invalid_main_cannot_publish() {
    for (target, symbol) in [
        (ExportTarget::Procedure(ProcedureId::new(99)), "missing"),
        (ExportTarget::Procedure(ProcedureId::new(0)), "main"),
        (ExportTarget::Global(GlobalId::new(0)), "main"),
    ] {
        let (builder, _) = fixture();
        assert!(matches!(
            builder
                .program_exports(vec![export(declaration(), target, symbol)])
                .finish_library(),
            Err(IrError::ProgramExport(_))
        ));
    }
}

#[test]
fn main_policy_checks_real_c_signature_without_debug_names() {
    let mut types = TypeRegistry::new();
    let integer = types.scalar(ScalarType::Int(jai_types::IntegerType::S32));
    let byte = types.scalar(ScalarType::Int(jai_types::IntegerType::U8));
    let pointer = types.pointer(byte).unwrap();
    let argv = types.pointer(pointer).unwrap();
    let mut signature = ProcedureType {
        parameters: vec![integer, argv].into_boxed_slice(),
        results: vec![integer].into_boxed_slice(),
        return_abi: jai_types::ForeignReturnAbi::Natural,
        convention: CallingConvention::C,
        context: ContextMode::None,
        variadic: Variadic::None,
    };
    validate_main_signature(&signature, &types).unwrap();
    signature.convention = CallingConvention::Jai;
    assert!(matches!(
        validate_main_signature(&signature, &types),
        Err(ExportError::InvalidMain)
    ));
    for name in ["", "invalid\0name", "jai.p0", "llvm.some_intrinsic"] {
        assert!(NativeSymbol::new(name).is_err());
    }
}
