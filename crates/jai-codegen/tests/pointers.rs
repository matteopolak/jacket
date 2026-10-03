//! Source-to-VM-to-native agreement using only newly generated executables.
#[path = "support/checked_execution.rs"]
mod checked_execution;
use checked_execution::{check, check_execution, check_vm_unsupported};

#[test]
fn native_pointer_alias_parameter_and_null_short_circuit_agree_with_vm() {
    check(
        "set :: (p:*int) { p.* = 42; } main :: ()->int { x := 1; p := *x; set(p); q:*int = null; if q != null && q.* == 1 return 0; if !q && p == *x return x; return 1; }",
        42,
    );
    check(
        "main :: ()->int { x := 42; p:*int = ifx true then *x else null; q:*int = ifx false then null else p; return q.*; }",
        42,
    );
}

#[test]
fn native_nested_field_global_and_array_pointer_mutations_agree_with_vm() {
    check(
        "Inner :: struct { x:int; } Outer :: struct { inner:Inner; } global := 3; main :: ()->int { item:Outer; p := *item.inner.x; p.* = 35; q := *global; q.* += 4; return item.inner.x + global; }",
        42,
    );
    check(
        "main :: ()->int { a:[3]int = .[11,20,31]; p := *a[0]; q := p + 1; q.* += 2; return a[1] + (p + 2).* - 11; }",
        42,
    );
}

#[test]
fn compound_indirect_target_evaluates_once_before_right_operand() {
    check(
        "calls := 0; target :: (p:*int)->*int { calls += 1; return p; } rhs :: ()->int { return calls; } main :: ()->int { x := 40; target(*x).* += rhs(); return x + calls; }",
        42,
    );
}

#[test]
fn compound_array_index_evaluates_once_before_right_operand() {
    check(
        "calls := 0; index :: ()->int { calls += 1; return 0; } rhs :: ()->int { return calls; } main :: ()->int { a:[1]int = .[40]; a[index()] += rhs(); return a[0] + calls; }",
        42,
    );
}

#[test]
fn addressable_array_read_loads_after_its_index_call() {
    check(
        "change :: (p:*int)->int { p.* = 42; return 0; } main :: ()->int { a:[1]int = .[1]; return a[change(*a[0])]; }",
        42,
    );
}

#[test]
fn allocation_and_zero_offset_field_have_the_same_pointer_identity() {
    check(
        "Pair :: struct { x:int; y:int; } main :: ()->int { value:Pair; a := cast(*void) *value; b := cast(*void) *value.x; if a == b return 42; return 0; }",
        42,
    );
}

#[test]
fn one_past_pointer_difference_uses_the_element_stride() {
    check(
        "main :: ()->int { a:[3]int = .[10,20,30]; p := *a[0]; end := p + 3; if end - p == 3 && p - end == -3 return (end - 1).* + 12; return 0; }",
        42,
    );
}

#[test]
fn checked_same_width_pointer_view_casts_preserve_scalar_bits() {
    check(
        "main :: ()->int { value:float = 1.0; bits := cast(*u32) *value; if bits.* == 1065353216 { bits.* = 1073741824; if value == 2.0 return 42; } return 0; }",
        42,
    );
}

#[test]
fn byte_pointer_offsets_and_alias_stores_use_target_bytes() {
    // Both this native fixture and the default VM profile select little endian.
    check(
        "main :: ()->int { value:u64 = 0x0102030405060708; bytes := cast(*u8) *value; if (bytes + 1).* != 7 return 0; bytes[0] = 42; if value == 0x010203040506072A return 42; return 0; }",
        42,
    );
}

#[test]
fn byte_alias_of_boolean_matches_the_selected_native_storage_bit() {
    check(
        "main :: ()->int { value := false; byte := cast(*u8) *value; byte.* = 42; return ifx value 1 else 0; }",
        0,
    );
}

#[test]
fn pointer_types_are_constructed_for_reflection_without_storage_access() {
    check(
        "Pair :: struct { value:int; } main :: ()->int { if size_of(*Pair) == 8 && size_of(**Pair) == 8 return 42; return 0; }",
        42,
    );
}

#[test]
fn packed_field_pointer_access_does_not_assume_natural_alignment() {
    check(
        "Packed :: struct { tag:u8; value:int; } #no_padding main :: ()->int { item:Packed; p := *item.value; p.* = 42; return item.value; }",
        42,
    );
}

#[test]
fn invalid_indirect_and_indexed_accesses_trap_in_native_and_fail_in_vm() {
    for source in [
        "main :: ()->int { p:*int = null; return p.*; }",
        "main :: ()->int { p:*int = null; return p[0]; }",
        "main :: ()->int { a:[1]int = .[42]; index := -1; return a[index]; }",
        "main :: ()->int { a:[1]int = .[42]; index := 1; return a[index]; }",
    ] {
        check_execution(source, None);
    }
}

#[test]
fn builtin_type_values_and_pointer_layout_queries_agree_with_vm() {
    check(
        "main :: ()->int { if size_of(int) != 8 || size_of(s8) != 1 || size_of(s16) != 2 || size_of(s32) != 4 || size_of(s64) != 8 || size_of(u8) != 1 || size_of(u16) != 2 || size_of(u32) != 4 || size_of(u64) != 8 || size_of(float) != 4 || size_of(float32) != 4 || size_of(float64) != 8 || size_of(bool) != 1 return 0; if size_of(*int) != 8 || size_of(*u8) != 8 || size_of(*float32) != 8 || size_of(*bool) != 8 || size_of(*void) != 8 || size_of(*#Context) != 8 return 0; if int == s64 && float == float32 && u8 != u64 && bool != void && *#Context != *void return 42; return 0; }",
        42,
    );
}

#[test]
fn local_values_shadow_builtin_type_names() {
    check("int := 42; main :: ()->int { return (*int).*; }", 42);
    check(
        "main :: ()->int { int := 40; float32 := 2; p := *int; p.* += float32; return int; }",
        42,
    );
}

#[test]
fn pointer_integer_roundtrips_preserve_typed_storage_aliases() {
    check(
        "main :: ()->int { value := 1; p := *value; address := cast(u64) p; restored := cast(*int) address; restored.* = 42; return value; }",
        42,
    );
    check(
        "main :: ()->int { value := 42; address := cast(int) *value; restored := cast(*int) address; return restored.*; }",
        42,
    );
    check(
        "main :: ()->int { p:*int = null; address := cast(u8) p; restored := cast(*int) address; if restored == null return 42; return 0; }",
        42,
    );
}

#[test]
fn integer_address_differences_use_byte_offsets() {
    check(
        "main :: ()->int { values:[4]u16; base := cast(u64) *values[0]; last := cast(u64) *values[3]; return cast(int)(last-base) + 36; }",
        42,
    );
    check(
        "main :: ()->int { value:u64 = 0x0102030405060708; address := cast(u64) *value; byte := cast(*u8)(address+1); return cast(int)byte.* + 35; }",
        42,
    );
}

#[test]
fn address_integer_storage_views_use_the_same_virtual_address_identity() {
    check(
        "main :: ()->int { value := 1; pointer := *value; stored_bits := (cast(*u64) *pointer).*; address := cast(u64) pointer; if stored_bits != address return 0; restored := cast(*int) stored_bits; restored.* = 42; return value; }",
        42,
    );
    check(
        "AddressBox :: union { address:u64; bytes:[8]u8; } main :: ()->int { value := 1; box:AddressBox; box.address = cast(u64) *value; copy := box; restored := cast(*int) copy.address; restored.* = 42; return value; }",
        42,
    );
}

#[test]
fn integer_left_pointer_offset_evaluates_left_call_first() {
    check(
        "calls := 0; offset :: ()->int { calls = 1; return 1; } pointer :: (p:*int)->*int { if calls == 1 calls = 2; else calls = 99; return p; } main :: ()->int { values:[2]int = .[0,40]; p := offset() + pointer(*values[0]); p.* += calls; return values[1]; }",
        42,
    );
}

#[test]
fn virtual_and_native_addresses_honor_explicit_record_alignment() {
    check(
        "Aligned :: struct { value:int; } #align 64 main :: ()->int { item:Aligned; address := cast(u64) *item; if address % 64 != 0 return 0; restored := cast(*Aligned) address; restored.value = 42; return item.value; }",
        42,
    );
}

#[test]
fn numeric_pointer_sentinels_and_real_address_narrowing_obey_their_origins() {
    // Numeric values are observable; genuine data-address narrowing retains its
    // existing native-range boundary rather than losing allocation provenance.
    check(
        "main :: ()->int { pointer := cast(*void) 1; if pointer != null return 42; return 0; }",
        42,
    );
    check_vm_unsupported(
        "main :: ()->int { value := 1; ignored := cast,no_check(u8) *value; return 42; }",
        42,
    );
    check_execution(
        "main :: ()->int { value := 1; narrowed := cast(u8) *value; return cast(int) narrowed; }",
        None,
    );
}

#[test]
fn absolute_address_indices_have_an_explicit_vm_boundary() {
    check_vm_unsupported(
        "main :: ()->int { values:[3]int = .[42,42,42]; x := 1; index := cast(int)(cast(u64)*x % 3); return values[index]; }",
        42,
    );
    check_vm_unsupported(
        "main :: ()->int { values:[3]int = .[42,42,42]; x := 1; offset := cast(int)(cast(u64)*x % 3); return ((*values[0]) + offset).*; }",
        42,
    );
}

#[test]
fn void_pointer_arithmetic_uses_byte_offsets_and_preserves_its_type() {
    check(
        "main :: ()->int { bytes:[4]u8 = .[5,6,7,8]; pointer:*void = *bytes[0]; next := pointer+2; if type_of(next) != *void return 0; view:*u8 = xx next; view.* = 40; return cast(int)bytes[2] + (next-pointer); }",
        42,
    );
    check(
        "main :: ()->int { bytes:[4]u8 = .[5,6,7,8]; pointer:*void = *bytes[0]; pointer += 3; pointer -= 1; next := 1+pointer; view:*u8 = xx next; view.* = 41; return cast(int)bytes[3] + (next-pointer); }",
        42,
    );
}

#[test]
fn opaque_numeric_pointer_storage_and_run_constants_agree_with_native() {
    check(
        "Box :: struct { p:*void; } main :: ()->int { x:Box; x.p = cast(*void) 1; bits:u64 = 1; q := cast(*void) bits; if !x.p || x.p != q || x.p == null return 0; if cast(u64) x.p != 1 return 0; return 42; }",
        42,
    );
    check(
        "make :: ()->*void { return cast(*void) 1; } address :: #run make(); verify :: ()->bool { return address != null && cast(u64) address == 1; } #assert #run verify(); main :: ()->int { if cast(u64) address == 1 return 42; return 0; }",
        42,
    );
}
