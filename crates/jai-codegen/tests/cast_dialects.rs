//! Independently authored source cases, evaluated in the VM and fresh native code.
#[path = "support/checked_execution.rs"]
mod checked_execution;
use checked_execution::{check, check_execution, check_vm_unsupported};

#[test]
fn integer_truncation_and_checked_casts_keep_different_behavior() {
    check(
        "main::()->int { size:=256; small:=cast,trunc(u8)size; mask:=cast(u32,-32,trunc); return cast(int)small + cast(int)(mask & 255) - 182; }",
        42,
    );
    check(
        "main::()->int { hash:u64=0x10000002a; return cast(int)cast,trunc(u32)hash; }",
        42,
    );
    check_execution(
        "main::()->int { size:=256; return cast(int)cast(u8)size; }",
        None,
    );
}

#[test]
fn contextual_defaults_and_nominal_representations_keep_truncation() {
    check(
        "Kind::enum u8 { ANSWER::42; } Count::#type,distinct u8; Box::struct { value:u8=xx,trunc 298; kind:Kind=xx,trunc 298; } take::(value:u8=xx,trunc 298)->int{return cast(int)value;} main::()->int { box:Box; value:Count=xx,trunc 298; return take()+cast(int)box.value+cast(int)box.kind+cast(int)value-126; }",
        42,
    );
    check(
        "global:*void=cast(*void,0,trunc); take::(value:*int=cast,trunc(*int)0)->int { if value==null && global==null return 42; return 0; } main::()->int{return take();}",
        42,
    );
}

#[test]
fn comma_pointer_operands_preserve_source_order_and_aliases() {
    check(
        "calls:=0; bytes:[24]u8; address::()->*u8 { calls+=1; return *bytes[0]; } main::()->int { pointer:=cast(*u32,cast(*u8,address())+16); pointer.*=40; return cast(int)cast(*u32).* (cast(*u8,*bytes[0])+16)+calls*2; }",
        42,
    );
    check(
        "main::()->int { pointer:=cast(*int,0,trunc); if pointer==null return 42; return 0; }",
        42,
    );
    check(
        "main::()->int { sentinel:=cast,trunc(*void)-1; if sentinel != null return 42; return 0; }",
        42,
    );
    check(
        "main::()->int { value:=42; address:=cast(u64)*value; pointer:=cast,trunc(*int)address; return pointer.*; }",
        42,
    );
    check_vm_unsupported(
        "main::()->int { value:=42; pointer:=*value; small:=cast,trunc(u8)pointer; full:=cast(u64)pointer; if small==cast,trunc(u8)full return 42; return 0; }",
        42,
    );
}

#[test]
fn postfix_enum_and_pointer_casts_use_the_same_checked_type_identity() {
    check(
        "Gpu_Queue::enum u8{FIRST::1;SECOND::2;} main::()->int { index:=1; queue:=(index+1).(Gpu_Queue); value:u32=40; bytes:=(*value).(*u8); result:=(bytes).(*u32).*; return cast(int)result+cast(int)queue; }",
        42,
    );
    check("main::()->int{return cast(int)(298).(u8,trunc);}", 42);
    check_execution(
        "main::()->int { value:=298; return cast(int)value.(u8); }",
        None,
    );
}
