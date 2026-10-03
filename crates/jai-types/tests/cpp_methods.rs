use jai_types::*;
fn method(types: &mut TypeRegistry) -> ProcedureType {
    let receiver = types.pointer(types.void()).unwrap();
    ProcedureType {
        parameters: Box::new([receiver]),
        results: Box::new([]),
        convention: CallingConvention::CppMethod,
        context: ContextMode::None,
        variadic: Variadic::None,
    }
}
#[test]
fn method_identity_retains_the_convention() {
    let mut types = TypeRegistry::new();
    let signature = method(&mut types);
    let cpp = types.procedure(signature.clone()).unwrap();
    let mut c_signature = signature;
    c_signature.convention = CallingConvention::C;
    let c = types.procedure(c_signature).unwrap();
    assert_ne!(c, cpp);
    assert_eq!(
        types.procedure_definition(cpp).unwrap().convention,
        CallingConvention::CppMethod
    );
}
#[test]
fn malformed_method_signatures_are_rejected_at_construction() {
    let mut types = TypeRegistry::new();
    let signature = method(&mut types);
    for issue in [
        CppMethodIssue::Receiver,
        CppMethodIssue::Context,
        CppMethodIssue::Variadic,
        CppMethodIssue::MultipleResults,
    ] {
        let mut invalid = signature.clone();
        match issue {
            CppMethodIssue::Receiver => invalid.parameters = Box::new([]),
            CppMethodIssue::Context => invalid.context = ContextMode::Implicit,
            CppMethodIssue::Variadic => {
                invalid.variadic = Variadic::C {
                    fixed_parameters: 1,
                }
            }
            CppMethodIssue::MultipleResults => {
                invalid.results = Box::new([types.scalar(ScalarType::Bool); 2])
            }
        }
        assert_eq!(
            types.procedure(invalid),
            Err(TypeError::InvalidCppMethod(issue))
        );
    }
}
