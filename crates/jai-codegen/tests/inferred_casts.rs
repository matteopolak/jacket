//! The contextual cast target comes from the source destination, never a guess.
#[path = "support/checked_execution.rs"]
mod checked_execution;
use checked_execution::{check, check_execution, check_vm_unsupported};

#[test]
fn assignments_and_checked_or_unchecked_narrowing_use_the_destination() {
    check(
        "main :: ()->int { source:s64 = 42; small:u8 = xx source; source = xx small; return source; }",
        42,
    );
    check(
        "main :: ()->int { source:s64 = 300; small:u8 = xx,no_check source; return cast(int)small - 2; }",
        42,
    );
    check_execution(
        "main :: ()->int { source:s64 = 300; small:u8 = xx source; return cast(int)small; }",
        None,
    );
}

#[test]
fn fixed_named_and_indirect_calls_supply_parameter_types() {
    check(
        "take :: (small:u8)->int { return cast(int)small; } main :: ()->int { source := 42; return take(xx source); }",
        42,
    );
    check(
        "take :: (small:u8)->int { return cast(int)small; } main :: ()->int { callback := take; source := 42; return callback(small=xx source); }",
        42,
    );
}

#[test]
fn enum_parameter_and_record_defaults_cast_to_the_declared_type() {
    check(
        "Bits :: enum_flags u8 { A::1; B::2; } Box :: struct { flags:Bits = xx 3; small:u8 = xx,no_check 300; } take :: (flags:Bits = xx 3)->int { return cast(int)flags; } main :: ()->int { box:Box; return take()+cast(int)box.flags+cast(int)box.small-8; }",
        42,
    );
    check(
        "global:u8 = xx,no_check 300; main :: ()->int { return cast(int)global-2; }",
        42,
    );
}

#[test]
fn aggregate_elements_and_return_values_supply_context() {
    check(
        "Box :: struct { small:u8; } answer :: ()->u8 { value := 42; return xx value; } main :: ()->int { values:[2]u8 = .[xx 20,xx 22]; box:Box = .{small=xx (cast(int)values[0]+cast(int)values[1])}; return cast(int)box.small + cast(int)answer() - 42; }",
        42,
    );
}

#[test]
fn float_casts_round_to_the_declared_destination() {
    check(
        "take :: (value:float32)->int { return cast(int)value; } main :: ()->int { value:float64 = 42.9; small:float32 = xx value; answer:u8 = xx small; return take(xx value)+cast(int)answer-42; }",
        42,
    );
}

#[test]
fn contextual_pointer_views_and_implicit_void_erasure_preserve_aliases() {
    check(
        "read :: (pointer:*void)->int { restored:*int = xx pointer; return restored.*; } main :: ()->int { value := 42; bytes:*u8 = xx *value; restored:*int = xx bytes; return read(restored); }",
        42,
    );
    check(
        "main :: ()->int { p:*int = xx null; address:u64 = xx p; restored:*int = xx address; if restored == null return 42; return 0; }",
        42,
    );
    check_vm_unsupported(
        "main :: ()->int { value := 1; narrowed:u8 = xx,no_check *value; return 42; }",
        42,
    );
}

#[test]
fn conditional_binary_and_index_context_preserves_checked_modes() {
    check(
        "main :: ()->int { value:u8 = ifx true then xx 40 else xx,no_check 300; source:int = 2; value += xx source; if xx value && xx 1 return cast(int)value; return 0; }",
        42,
    );
    check(
        "main :: ()->int { values:[2]int = .[0,42]; source:u8 = 1; return values[xx source]; }",
        42,
    );
    check(
        "calls := 0; value :: ()->int { calls += 1; return 40; } take :: (small:u8)->int { return cast(int)small + calls*2; } main :: ()->int { return take(xx value()); }",
        42,
    );
}

#[test]
fn candidate_matching_keeps_enum_and_distinct_cast_domains() {
    check(
        "Bits :: enum_flags u8 { A::1; B::2; } take :: (value:bool)->int { return ifx value then 42 else 0; } main :: ()->int { flags:Bits=.A; return take(xx flags); }",
        42,
    );
    check(
        "Kind :: enum u8 { ANSWER::42; } take :: (value:Kind)->int { return cast(int)value; } main :: ()->int { return take(xx .ANSWER); }",
        42,
    );
    check(
        "Count :: #type,distinct u8; take :: (value:Count)->int { return cast(int)value; } main :: ()->int { return take(xx 42); }",
        42,
    );
    check(
        "Opaque :: #type,distinct *void; take :: (value:Opaque)->int { pointer := cast(*int)cast(*void)value; return pointer.*; } main :: ()->int { value:=42; return take(xx *value); }",
        42,
    );
}

#[test]
fn sequence_casts_receive_the_destination_element_type() {
    check(
        "main :: ()->int { values:[]u8 = xx .[xx 20,xx 22]; return cast(int)values[0]+cast(int)values[1]; }",
        42,
    );
    check(
        "take :: (values:[]u8)->int { return cast(int)values[0]; } main :: ()->int { text:=\"*\"; return take(xx text); }",
        42,
    );
}

#[test]
fn declared_conditional_values_supply_boolean_condition_context() {
    check(
        "main :: ()->int { condition:int=1; value:u8=ifx xx condition then xx 42 else xx 0; pointer:*u8=ifx xx condition then *value else null; return cast(int)pointer.*; }",
        42,
    );
    check(
        "main :: ()->int { value:int=1; count:=0; while xx value { count+=1; value=0; } if !xx value return ifx xx count then 42 else 0; return 0; }",
        42,
    );
}
