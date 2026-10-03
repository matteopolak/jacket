//! Register after graph selected-call return and discard transport is implemented.
use super::check;

#[test]
fn graph_operator_results_retain_original_callback_cast_contracts() {
    let declarations = "Box::struct{value:int;}Optional::#type(value:int)->int;Required::#type(value:int)->int #must;answer::(value:int)->int{return value;}operator []::(a:Box,callback:$F)->F{return callback;}";
    let body = "required:=Box.{value=1}[cast(Required)answer];optional:=Box.{value=2}[cast(Optional)answer];";
    check(&format!(
        "{declarations}main::()->int{{{body}optional(value=0);return required(value=42);}}"
    ))
    .unwrap();
    let source = format!("{declarations}main::()->int{{{body}required(value=42);return 0;}}");
    let error = check(&source)
        .err()
        .expect("required source cast on graph operator result");
    assert!(error.message.contains("#must"), "{error:?}");
    let start = source.rfind("required(value=42)").unwrap();
    assert_eq!(error.location.span.start, start, "{error:?}");
}

#[test]
fn a_required_operator_result_cannot_be_discarded_at_a_root_destination() {
    let declarations =
        "Box::struct{value:int;}operator []::(a:Box,index:int)->int #must{return a.value+index;}";
    check(&format!(
        "{declarations}main::()->int{{return Box.{{value=40}}[2];}}"
    ))
    .unwrap();
    for discard in [
        "Box.{value=40}[2];",
        "_:=Box.{value=40}[2];",
        "_=Box.{value=40}[2];",
    ] {
        let source = format!("{declarations}main::(){{{discard}}}");
        let error = check(&source)
            .err()
            .expect("required operator result root discard");
        assert!(error.message.contains("#must"), "{error:?}");
        let start = source.rfind("Box.{value=40}[2]").unwrap();
        assert_eq!(error.location.span.start, start, "{error:?}");
    }
}
