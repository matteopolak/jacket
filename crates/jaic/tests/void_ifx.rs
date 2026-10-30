//! Discarded void conditional expressions retain branch effects, never a value.
use test_support::compiler::compile_main_file as compile;


#[test]
fn discarded_void_ifx_executes_only_the_selected_branch() {
    compile(include_str!(
        "../../../tests/fixtures/void-ifx/ifx-discarded-void.jai"
    ))
    .unwrap();
}

#[test]
fn void_arms_keep_deferred_effects_and_nested_conditionals() {
    compile("touch :: (n:*int) {n.* += 1;} exercise :: (flag:bool)->int { n:int; ifx flag {defer touch(*n); touch(*n);} else {ifx !flag {touch(*n);} else {touch(*n); touch(*n);}} return n;} #assert exercise(true)==2; #assert exercise(false)==1; main :: () {}").unwrap();
}

#[test]
fn omitted_else_does_not_execute_a_false_arm_to_infer_its_type() {
    compile("touch :: (n:*int) {n.* += 1;} number :: (n:*int)->int {touch(n);return 7;} exercise :: ()->int {n:int; ifx false {touch(*n);} value := ifx false then number(*n); return n+value;} #assert exercise()==0; main :: () {}").unwrap();
}

#[test]
fn void_ifx_does_not_supply_values_or_accept_incompatible_arms() {
    for body in [
        "n:int = ifx flag {touch();} else {touch();};",
        "n := 1 + (ifx flag {touch();} else {touch();});",
        "n := ifx flag {touch();} else 3;",
        "n := ifx flag then 3 else {touch();};",
    ] {
        let source = format!("touch :: () {{}} exercise :: (flag:bool) {{{body}}} main :: () {{}}");
        let error = compile(&source).expect_err("void must never become a fabricated value");
        assert!(
            error.contains("void") || error.contains("no value"),
            "{error}"
        );
    }
}

#[test]
fn the_untaken_arm_of_a_constant_ifx_infers_from_the_target() {
    compile("Cpu :: enum {A; B;} K :: 1; S :: struct {cpu: Cpu;} main :: () {s: S; s.cpu = ifx K == 1 then .B else .A; s.cpu = ifx K == 2 then .B else .A; c: Cpu = ifx K == 1 then .A else .B;}").unwrap();
}
