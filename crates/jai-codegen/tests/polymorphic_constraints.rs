//! Independently authored restriction and optional-baking programs agree in VM/native execution.
#[path = "support/checked_execution.rs"]
#[allow(
    dead_code,
    reason = "The shared harness also exposes foreign execution runners."
)]
mod checked_execution;

#[test]
fn using_ancestry_keeps_the_complete_derived_argument() {
    checked_execution::check(
        "Base::struct{value:int;} Middle::struct{using base:Base;} \
         Derived::struct{using middle:Middle;extra:int;} \
         read::(item:*$T/Base)->int{return item.value+item.extra;} \
         main::()->int{item:Derived;item.value=40;item.extra=2;return read(*item);}",
        42,
    );
}

#[test]
fn interfaces_accept_promoted_members_with_exact_types() {
    checked_execution::check(
        "Required::struct{value:int;extra:int;} Base::struct{value:int;} \
         Actual::struct{extra:int;using base:Base;unused:bool;} \
         read::(item:$T/interface Required)->int{return item.value+item.extra;} \
         main::()->int{item:Actual;item.value=40;item.extra=2;return read(item);}",
        42,
    );
}

#[test]
fn a_bare_generic_nominal_restriction_uses_its_actual_origin() {
    checked_execution::check(
        "Table::struct(T:Type){value:T;} \
         read::(table:*$T/Table)->int{return table.value;} \
         main::()->int{table:Table(int);table.value=42;return read(*table);}",
        42,
    );
}

#[test]
fn optional_bakes_preserve_constant_and_runtime_behavior() {
    checked_execution::check(
        "probe::($$value:int)->int { \
         #if is_constant(value) {return value;} else {return value+30;} \
         } main::()->int {value:int=6;return probe(6)+probe(value);}",
        42,
    );
}

#[test]
fn optional_bakes_keep_runtime_instance_receiver_effects() {
    checked_execution::check(
        "Bag::struct($N:int){Size::N;value:int;} calls:int=0; \
         make::()->Bag(7){calls+=1;return .{value=7};} \
         probe::($$value:int)->int{#if is_constant(value) {return value+10;} else {return value;}} \
         main::()->int{bag:Bag(7)=.{};return probe(bag.Size)+probe(make().Size)+calls*18;}",
        42,
    );
}

#[test]
fn nonscalar_distinct_casts_keep_their_owned_backing() {
    checked_execution::check(
        "Wrapped::#type,distinct [2]int; \
         probe::($$value:Wrapped)->int{#if !is_constant(value) return 0; \
         view:=cast([]int)value;return view[0]+view[1];} \
         main::()->int{return probe(cast(Wrapped)int.[20,22]);}",
        42,
    );
    checked_execution::check(
        "Wrapped::#type,distinct string; \
         probe::($$value:Wrapped)->int{#if !is_constant(value) return 0; \
         view:=cast([]u8)value;return view.count+39;} \
         main::()->int{return probe(cast(Wrapped)\"abc\");}",
        42,
    );
}

#[test]
fn optional_bakes_preserve_exact_empty_views_and_null_pointer_wrappers() {
    checked_execution::check(
        "Empty::struct{values:[]int;} empty::Empty.{}; \
         probe::($$value:[]int)->int{#if is_constant(value) {return value.count+42;} else {return 0;}} \
         main::()->int{return probe(cast([]int)empty.values);}",
        42,
    );
    checked_execution::check(
        "Opaque::#type,distinct *void; \
         probe::($$value:Opaque)->int{#if !is_constant(value) return 0; \
         if cast(*void)value==null return 42;return 1;} \
         main::()->int{return probe(cast(Opaque)cast(*int)null);}",
        42,
    );
}
