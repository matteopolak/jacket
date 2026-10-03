//! Constant result groups use the common VM and preserve one call execution.
#[path = "support/checked_execution.rs"]
mod checked_execution;
#[test]
fn heterogeneous_constants_control_compile_time_branches() {
    checked_execution::check(
        "facts :: ()->bool,bool,s32 {return true,true,32;} main :: ()->int {IS_INTEGER,SIGNED,BITS :: #run facts(); #assert IS_INTEGER; #if SIGNED {return BITS+10;} else {return 1;}}",
        42,
    );
}
#[test]
fn one_group_call_is_cached_across_all_named_projections() {
    checked_execution::check(
        "calls:s32=0; facts :: ()->s32,s32,s32 {calls+=1;return 10,20,11;} observed :: (a:s32,b:s32,c:s32)->s32 {return a+b+c+calls;} main :: ()->int {a,b,c :: #run facts(); answer :: #run observed(a,b,c);return answer;}",
        42,
    );
}
#[test]
fn all_discarded_optional_results_still_execute_the_call() {
    checked_execution::check(
        "calls:s32=0; facts :: ()->s32,s32 {calls+=1;return 10,20;} observed :: ()->s32 {return calls*42;} main :: ()->int {_,_ :: #run facts(); return #run observed();}",
        42,
    );
}

#[test]
fn forward_projection_use_keeps_one_group_execution() {
    checked_execution::check(
        "calls:s32=0; facts :: ()->s32,s32,s32 {calls+=1;return 10,20,11;} observed :: (sum:s32)->s32 {return sum+calls;} main :: ()->int {sum :: a+b+c; a,b,c :: #run facts(); answer :: #run observed(sum); return answer;}",
        42,
    );
}

#[test]
fn shadowed_groups_keep_separate_executions() {
    checked_execution::check(
        "calls:s32=0; facts :: ()->s32,s32 {calls+=1;return calls,10;} observed :: (a:s32,b:s32)->s32 {return a+b+calls*10;} main :: ()->int {a,b :: #run facts(); outer_sum :: a+b; {a,b :: #run facts(); answer :: #run observed(a,b); return outer_sum+answer;}}",
        43,
    );
}
