#import "Basic";
H :: #import "Hash";
HT :: #import "Hash_Table";

Signed :: enum s8 {
    LOW :: -1;
    HIGH :: 42;
}

main :: () {
    assert(H.get_hash(cast(u64) 0) == 2752616464);
    assert(H.get_hash(cast(u64) 1) == 98180695);
    assert(H.get_hash(cast(u64) 42) == 2569769087);
    assert(H.get_hash(cast(s64) -1) == 3182882357);
    assert(H.get_hash(cast(s8) -1) == H.get_hash(cast(s64) -1));
    assert(H.get_hash(cast(s32) -1) == H.get_hash(cast(s64) -1));
    assert(H.get_hash(false) == H.get_hash(cast(u64) 0));
    assert(H.get_hash(true) == H.get_hash(cast(u64) 1));
    assert(H.get_hash(Signed.LOW) == H.get_hash(cast(s64) -1));
    assert(H.get_hash(Signed.HIGH) == H.get_hash(cast(u64) 42));
    pointer := cast(*void) 0x1234;

    assert(H.get_hash(pointer) == H.get_hash(cast(u64) 0x1234));
    assert(H.get_hash(cast(u64) 0, 0) == 0);
    assert(H.get_hash(cast(u64) 1, 0) == 2654435769);
    assert(H.get_hash(cast(u64) 42, 7) == 3485492635);
    table: HT.Table(s64, string);

    defer HT.table_reset(*table);
    for i: -128..128 HT.table_add(*table, i, "value");

    for i: -128..128 {
        found, value := HT.table_find(*table, i);
        assert(found && value == "value");
    }

    for i: -128..0 {
        removed, value := HT.table_remove(*table, i);
        assert(removed && value == "value");
        assert(!HT.table_contains(*table, i));
    }

    for i: 1..128 assert(HT.table_contains(*table, i));

    assert(table.count == 128);
    flags: HT.Table(bool, int);

    defer HT.table_reset(*flags);
    HT.table_add(*flags, true, 9);
    HT.table_add(*flags, false, 7);
    found, flag_value := HT.table_find(*flags, true);

    assert(found && flag_value == 9);
    found, flag_value = HT.table_find(*flags, false);
    assert(found && flag_value == 7);
    pointers: HT.Table(*void, int);

    defer HT.table_reset(*pointers);
    HT.table_add(*pointers, pointer, 3);
    pointer_found, pointer_value := HT.table_find(*pointers, pointer);

    assert(pointer_found && pointer_value == 3);
    enums: HT.Table(Signed, int);

    defer HT.table_reset(*enums);
    HT.table_add(*enums, Signed.LOW, 4);
    enum_found, enum_value := HT.table_find(*enums, Signed.LOW);

    assert(enum_found && enum_value == 4);
    print("ok\n");
}
