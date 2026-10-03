#[path = "support/checked_execution.rs"]
mod checked_execution;

#[test]
fn anonymous_standalone_call_and_return_use_real_bodies() {
    for source in [
        "main::()->int{ f:=(value:s32)->s32{return value+2;};return f(40);}",
        "Callback::()->s32;make::()->Callback{return ()->s32{return 42;};}main::()->int{return make()();}",
        "main::()->int{return ((value:s32)->s32{return value+2;})(40);}",
        "Callback::(value:s32)->s32;apply::(f:Callback)->s32{return f(40);}main::()->int{return apply((value:s32)->s32{return value+2;});}",
        "answer::(()->s32{return 42;});main::()->int{return answer();}",
        "Callback::()->void;invoke::(f:Callback){f();}main::()->int{invoke(()->void{return;});return 42;}",
    ] {
        checked_execution::check_optimized(source, 42);
    }
}

#[test]
fn anonymous_c_assignment_preserves_context_and_convention() {
    checked_execution::check_optimized(
        "Callback::(value:s32)->s32 #c_call; main::()->int { f:Callback=(value:s32)->s32 #c_call{return value+2;};return f(40); }",
        42,
    );
}

#[test]
fn anonymous_names_defaults_and_discarded_formals_remain_source_metadata() {
    checked_execution::check_optimized(
        "calls:s32=0;next::()->s32{calls+=1;return 9;}main::()->int{f:=(#discard ignored:s32,value:s32,extra:s32=2)->s32{return value+extra;};return f(next(),value=40)+calls;}",
        42,
    );
}

#[test]
fn anonymous_named_result_defaults_use_the_concrete_body_binder() {
    checked_execution::check_optimized(
        "main::()->int{f:=()->(answer:s32=42){return;};return f();}",
        42,
    );
}

#[test]
fn actual_nominal_global_and_constant_callbacks_match_vm_at_o0_and_o2() {
    for source in [
        include_str!("../../jai-sema/tests/fixtures/full-anonymous/nominal-global-callback.jai"),
        include_str!("../../jai-sema/tests/fixtures/full-anonymous/nominal-constant-callback.jai"),
        include_str!("../../jai-sema/tests/fixtures/full-anonymous/direct-global-procedure.jai"),
        include_str!(
            "../../jai-sema/tests/fixtures/full-anonymous/global-storage-from-anonymous.jai"
        ),
    ] {
        checked_execution::check_optimized(source, 42);
    }
}
