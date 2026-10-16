#import "Basic";
#import "Hash_Table";
H :: #import "Hash";

Event_Id :: #type,isa u64;
Nested_Id :: #type,isa Event_Id;
Signed_Id :: #type,isa s32;
Real32 :: #type,isa float32;
Real64 :: #type,isa float64;
Flag :: #type,isa bool;
Address :: #type,isa * u8;

Code :: enum s16 {
    LOW :: -1;
    HIGH :: 19;
}

Typed_Code :: #type,isa Code;

main :: () {
    id := cast(Event_Id) 7;
    nested := cast(Nested_Id) id;

    assert(type_info(Event_Id).type == .VARIANT);
    assert((cast(*Type_Info_Variant) type_info(Event_Id)).name == "Event_Id");
    assert(H.get_hash(id) == H.get_hash(cast(u64) id));
    assert(H.get_hash(nested, cast(u32) 17) == H.get_hash(cast(u64) 7, cast(u32) 17));
    assert(H.get_hash(cast(Signed_Id) -1) == H.get_hash(cast(s32) -1));
    assert(H.get_hash(cast(Flag) true) == H.get_hash(true));
    assert(H.get_hash(cast(Address) cast(*u8) 0x1234) == H.get_hash(cast(*u8) 0x1234));
    assert(H.get_hash(cast(Typed_Code) Code.LOW) == H.get_hash(Code.LOW));
    single: float32 = 1.25;
    wide: float64 = -2.5;

    assert(H.get_hash(cast(Real32) single) == H.get_hash(single));
    assert(H.get_hash(cast(Real64) wide, cast(u32) 71) == H.get_hash(wide, cast(u32) 71));
    negative_zero_bits: u32 = 0x80000000;
    negative_zero := (cast(*float32) *negative_zero_bits).*;

    assert(H.get_hash(cast(Real32) negative_zero) == H.get_hash(negative_zero));
    assert(H.get_hash(cast(Real32) negative_zero) != H.get_hash(cast(Real32) cast(float32) 0));
    table: Table(Event_Id, s64);

    table_add(*table, id, 42);
    ok, value := table_find(*table, id);

    assert(ok && value == 42);
    deinit(*table);
    print("ok\n");
}
