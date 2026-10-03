//! Basic formatter cell views use actual generated function addresses in native code.
#[path = "support/checked_execution.rs"]
#[allow(
    dead_code,
    reason = "This shared harness also exposes an unsupported-capability runner used by other suites."
)]
mod checked_execution;
use checked_execution::check;

#[test]
fn opaque_function_cells_preserve_identity_truth_and_callable_recovery() {
    check(
        r#"
        Increment::#type (value:s32)->s32;
        increment::(value:s32)->s32{return value+1;}
        main::()->int {
            first:Any=increment; second:Any=increment;
            a:=(cast(**void)first.value_pointer).*;
            b:=(cast(**void)second.value_pointer).*;
            if !a || a!=b return 1;
            boxed:Any=a;
            restored:=(cast(*Increment)boxed.value_pointer).*;
            return restored(41);
        }
    "#,
        42,
    );
}

#[test]
fn null_callable_cells_have_zero_opaque_addresses() {
    check(
        r#"
        Increment::#type (value:s32)->s32;
        main::()->int {
            empty:Increment=null; boxed:Any=empty;
            address:=(cast(**void)boxed.value_pointer).*;
            bits:=(cast(*u64)boxed.value_pointer).*;
            if address || address!=null || bits!=0 return 1;
            return 42;
        }
    "#,
        42,
    );
}

#[test]
fn raw_integer_views_and_opaque_integer_roundtrips_keep_the_function_address() {
    check(
        r#"
        Increment::#type (value:s32)->s32;
        increment::(value:s32)->s32{return value+1;}
        main::()->int {
            boxed:Any=increment;
            address:=(cast(**void)boxed.value_pointer).*;
            cell_bits:=(cast(*u64)boxed.value_pointer).*;
            bits:=cast(u64)address;
            if bits!=cell_bits return 1;
            restored_address:=cast(*void)bits;
            if restored_address!=address return 2;
            copied:Any=restored_address;
            callable:=(cast(*Increment)copied.value_pointer).*;
            return callable(41);
        }
    "#,
        42,
    );
}

#[test]
fn returned_code_address_outlives_the_frame_of_its_original_any_cell() {
    check(
        r#"
        Increment::#type (value:s32)->s32;
        increment::(value:s32)->s32{return value+1;}
        opaque::()->*void {
            boxed:Any=increment;
            return (cast(**void)boxed.value_pointer).*;
        }
        main::()->int {
            address:=opaque(); boxed:Any=address;
            callable:=(cast(*Increment)boxed.value_pointer).*;
            return callable(41);
        }
    "#,
        42,
    );
}

#[test]
fn formatter_digit_storage_runs_natively_and_preserves_the_vm_capability_boundary() {
    checked_execution::check_vm_unsupported(
        include_str!("../../jai-sema/tests/fixtures/procedure-address-formatting.jai.pending"),
        42,
    );
}
