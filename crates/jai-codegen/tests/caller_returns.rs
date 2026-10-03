//! Real caller return transfers agree in the VM and newly generated LLVM programs.
#[path = "support/checked_execution.rs"]
mod checked_execution;
#[path = "../../jai-sema/tests/support/supplied_caller_return.rs"]
mod supplied_caller_return;

#[test]
fn caller_return_storage_cleanup_context_and_result_contracts_match_native_code() {
    for source in [
        include_str!("../../jai-sema/tests/fixtures/caller-return-lifecycle.jai"),
        include_str!("../../jai-sema/tests/fixtures/caller-return-named.jai"),
        include_str!("../../jai-sema/tests/fixtures/caller-return-aggregate.jai"),
        include_str!("../../jai-sema/tests/fixtures/caller-return-nested.jai"),
        include_str!("../../jai-sema/tests/fixtures/caller-return-context.jai"),
        include_str!("../../jai-sema/tests/fixtures/caller-return-quotes.jai"),
        include_str!("../../jai-sema/tests/fixtures/caller-return-callback.jai"),
        include_str!("../../jai-sema/tests/fixtures/macro-loop-exits.jai"),
    ] {
        checked_execution::check_optimized(source, 42);
    }
}

#[test]
fn void_bool_float_and_string_caller_returns_match_native_code() {
    for source in [
        "wrap::(body:Code)#expand{#insert body;}main::()->int{code::#code `return 42;wrap(code);}",
        "target::(input:int=40)->int#must{return input+2;}finish::()#expand{`return target;}answer::()->(result:=target){finish();}main::()->int{result:=answer();return result();}",
        "total:int; finish::()#expand{defer total=42;`return;} answer::(){finish();} main::()->int{answer();return total;}",
        "finish::()#expand{`return true;} answer::()->bool{finish();} main::()->int{if answer() return 42;return 0;}",
        "finish::()#expand{`return 42.0;} answer::()->float64{finish();} main::()->int{return cast(int)answer();}",
        "finish::()#expand{`return \"answer\";} answer::()->string{finish();} main::()->int{return answer().count+36;}",
    ] {
        checked_execution::check_optimized(source, 42);
    }
}

#[test]
fn unchanged_supplied_sgpu_macro_matches_newly_generated_native_code() {
    checked_execution::check_optimized(&supplied_caller_return::sgpu_source(), 42);
}
