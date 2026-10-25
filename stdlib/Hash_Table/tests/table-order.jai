#import "Basic";
#import "Extensions/Testing";

table_members_keep_their_order_and_layout :: () {
    names := string.["count", "allocated", "slots_filled", "allocator", "Entry", "entries", "hash_function", "compare_function", "SIZE_MIN"];
    info := cast(*Type_Info_Struct) type_info(T);

    check_order(info, names);
    expect(info.runtime_size == 56 && info.alignment == 8);
    expect(type_info(T) == type_info(Same.Table(string, int)));
    expect(info.name == "Table");
    expected_offsets := s64.[0, 8, 16, 24, -1, 40, -1, -1, -1];

    for member, index: info.members {
        if index == 4 || index >= 6 {
            expect((member.flags & .CONSTANT) != 0);
            expect(member.offset_in_bytes < 0);
        } else {
            expect(!(member.flags & .CONSTANT));
            expect(member.offset_in_bytes == expected_offsets[index]);
        }
    }

    table: T;
    base := cast(u64) *table;

    expect((cast(u64) *table.count) - base == 0);
    expect((cast(u64) *table.allocated) - base == 8);
    expect((cast(u64) *table.slots_filled) - base == 16);
    expect((cast(u64) *table.allocator) - base == 24);
    expect((cast(u64) *table.entries) - base == 40);
    expect(table.SIZE_MIN == 32);
    expect(T.hash_function("key") == H.get_hash("key"));
    expect(T.compare_function("key", "key") && !T.compare_function("key", "different"));
    entry := cast(*Type_Info_Struct) type_info(T.Entry);

    check_order(entry, string.["hash", "key", "value"]);
    expect(entry.runtime_size == 32 && entry.members[0].offset_in_bytes == 0 &&
           entry.members[1].offset_in_bytes == 8 && entry.members[2].offset_in_bytes == 24);
    mixed := cast(*Type_Info_Struct) type_info(Mixed);

    check_order(mixed, string.["f", "n", "Item", "tail"]);
    expect((mixed.members[0].flags & .CONSTANT) != 0 && mixed.members[0].offset_in_bytes < 0);
    expect((mixed.members[2].flags & .CONSTANT) != 0 && mixed.members[2].offset_in_bytes < 0);
    counted := cast(*Type_Info_Struct) type_info(Counted.Table(string, int));

    check_order(counted, string.["count", "allocated", "slots_filled", "allocator", "Entry", "entries", "add_collisions", "find_collisions", "hash_function", "compare_function", "SIZE_MIN"]);
    expect(counted.runtime_size == 72 && counted.members[6].offset_in_bytes == 56 && counted.members[7].offset_in_bytes == 64);
} @test

#scope_file

H :: #import "Hash";
HT :: #import "Hash_Table";
Same :: #import "Hash_Table";
Counted :: #import "Hash_Table"(COUNT_COLLISIONS = true);

T :: HT.Table(string, int);

Mixed :: struct {
    f :: () {}
    n: s64;
    Item :: struct {
        value: u8;
    }
    tail: u8;
}

check_order :: (info: *Type_Info_Struct, names: [] string) {
    expect(info.members.count == names.count);
    for member, index: info.members expect(member.name == names[index], tprint("member % is %, expected %", index, member.name, names[index]));
}
