use super::*;
use jai_types::{CallingConvention, ContextMode, IntegerType, ProcedureType, ScalarType, Variadic};

fn fixture(name: &str, tag: Option<&str>) -> (syntax::ProcedurePrototype, Signature, TypeRegistry) {
    let mut symbols = Symbols::default();
    let name = symbols.intern(name);
    let mut types = TypeRegistry::new();
    let signature = types
        .procedure(ProcedureType {
            parameters: Box::new([]),
            results: Box::new([]),
            return_abi: jai_types::ForeignReturnAbi::Natural,
            convention: CallingConvention::Jai,
            context: ContextMode::None,
            variadic: Variadic::None,
        })
        .unwrap();
    (
        syntax::ProcedurePrototype {
            name,
            header: syntax::CallableHeaderSyntax {
                notes: vec![],
                deprecation: None,
                parameters: vec![],
                results: vec![],
                convention: CallingConvention::Jai,
                return_abi: jai_types::ForeignReturnAbi::Natural,
                context: ContextMode::None,
            },
            binding: syntax::PrototypeBinding::Intrinsic {
                tag: tag.map(str::to_owned),
            },
            span: Span::new(0, 1),
        },
        Signature {
            id: ProcedureId::new(0),
            ty: signature,
            source_variadic: crate::overloads::CandidateVariadic::None,
            parameters: vec![],
            results: vec![],
        },
        types,
    )
}

#[test]
fn only_an_explicit_marker_and_checked_signature_can_bind_a_tagged_intrinsic() {
    let (mut source, signature, types) = fixture("trap_here", Some("llvm.debugtrap"));
    let prototype = bind_prototype(
        &source,
        "trap_here",
        &signature,
        &types,
        Some(LayoutPolicy::lp64()),
    )
    .unwrap();
    assert!(matches!(
        prototype.origin,
        PrototypeOrigin::Intrinsic(RuntimeIntrinsic::DebugTrap)
    ));
    source.binding = syntax::PrototypeBinding::Foreign(syntax::ForeignProcedure {
        library: None,
        symbol: None,
    });
    assert!(
        bind_prototype(
            &source,
            "llvm.debugtrap",
            &signature,
            &types,
            Some(LayoutPolicy::lp64())
        )
        .unwrap_err()
        .message
        .contains("explicitly marked")
    );
}

#[test]
fn unknown_tags_and_missing_target_have_precise_failures() {
    let (source, signature, types) = fixture("invented", Some("invented"));
    assert!(
        bind_prototype(
            &source,
            "invented",
            &signature,
            &types,
            Some(LayoutPolicy::lp64())
        )
        .unwrap_err()
        .message
        .contains("unsupported #intrinsic `invented`")
    );
    let (source, signature, types) = fixture("trap_here", Some("llvm.debugtrap"));
    assert!(
        bind_prototype(&source, "trap_here", &signature, &types, None)
            .unwrap_err()
            .message
            .contains("selected target layout")
    );
}

#[test]
fn matching_name_cannot_override_a_wrong_checked_result_type() {
    let (source, mut signature, mut types) = fixture("memset", None);
    let pointer = types.pointer(types.void()).unwrap();
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    let count = types.scalar(ScalarType::Int(IntegerType::S64));
    signature.ty = types
        .procedure(ProcedureType {
            parameters: vec![pointer, byte, count].into(),
            results: vec![count].into(),
            return_abi: jai_types::ForeignReturnAbi::Natural,
            convention: CallingConvention::Jai,
            context: ContextMode::None,
            variadic: Variadic::None,
        })
        .unwrap();
    assert!(
        bind_prototype(
            &source,
            "memset",
            &signature,
            &types,
            Some(LayoutPolicy::lp64())
        )
        .unwrap_err()
        .message
        .contains("memset requires")
    );
}
