//! Register when compile-time result policy is paired with its checked source producer.
use super::check;

#[test]
fn compile_time_factory_results_keep_their_declared_callback_contract() {
    let source = "Required::#type(value:int)->int #must;answer::(value:int)->int{return value;}factory::()->Required{return answer;}callback::#run factory();main::(){callback(value=42);}";
    let error = check(source)
        .err()
        .expect("compile-time returned callback result is required");
    assert!(error.message.contains("#must"), "{error:?}");
    let start = source.find("callback(value=42)").unwrap();
    assert_eq!(error.location.span.start, start, "{error:?}");
}

#[test]
fn compile_time_optional_result_does_not_inherit_the_target_usage() {
    let source = "Optional::#type(value:int)->int;answer::(value:int)->int #must{return value;}factory::()->Optional{return answer;}callback::#run factory();main::()->int{callback(value=0);return callback(value=42);}";
    check(source).unwrap();
}
