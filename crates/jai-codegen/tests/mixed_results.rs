//! Independent source fixtures verify mixed declaration/assignment VM and native agreement.
#[path = "support/checked_execution.rs"]
mod checked_execution;
#[test]
fn per_result_colons_declare_new_slots_and_reuse_unmarked_slots() {
    checked_execution::check(
        "calls:s32=0; parse :: (old:s32)->bool,s32,s32 {calls+=1;return true,2,old+1;} main :: ()->int {args:s32=39;success:,plugins_to_create:,args = parse(args);if !success return 0;return args+plugins_to_create+calls-1;}",
        42,
    );
}
#[test]
fn reuses_existing_binding_and_declares_other_call_result_once() {
    checked_execution::check(
        "calls:s32=0; pair :: ()->s32,s32 {calls+=1;return 40,2;} main :: ()->int {old:s32=0;old=,fresh := pair(); return old+fresh+calls-1;}",
        42,
    );
}
#[test]
fn captures_all_values_before_updating_any_existing_binding() {
    checked_execution::check(
        "main :: ()->int {old:s32=40;old=,fresh := old+1,old;return old+fresh-39;}",
        42,
    );
}
#[test]
fn typed_new_binding_and_discard_preserve_result_positions() {
    checked_execution::check(
        "pair :: ()->s32,s32,s32 {return 40,99,2;} main :: ()->int {old:s32=0;old=,_,fresh :s64 = pair();return old+fresh;}",
        42,
    );
}

#[test]
fn existing_callback_retains_its_explicit_source_contract() {
    checked_execution::check(
        "required :: ()->s32 #must {return 42;} optional :: ()->s32 {return 0;} pair :: ()->(cb:()->s32 #must,count:s32) {return required,42;} main :: ()->int {callback:()->s32 = optional;callback=,answer := pair();callback();return answer;}",
        42,
    );
}
