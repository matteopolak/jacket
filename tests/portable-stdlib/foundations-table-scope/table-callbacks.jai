#import "Basic";
HT :: #import "Hash_Table";

Key :: struct {
    value: s64;
}

Custom :: HT.Table(Key, string, given_hash_function = hash_key, given_compare_function = same_key);
Alias :: HT.Table(Key, string, given_hash_function = hash_key, given_compare_function = same_key);
hash_calls: s64;
compare_calls: s64;

main :: () {
    assert(type_info(Custom) == type_info(Alias));
    assert(Custom.hash_function == hash_key && Custom.compare_function == same_key);
    info := cast(*Type_Info_Struct) type_info(Custom);
    expected := string.["count", "allocated", "slots_filled", "allocator", "Entry", "entries", "hash_function", "compare_function", "SIZE_MIN"];

    assert(info.members.count == expected.count);
    for member, index: info.members assert(member.name == expected[index]);

    table: Custom;

    defer HT.deinit(*table);
    for value: 0..63 HT.table_add(*table, .{ value }, "payload");

    assert(table.count == 64 && hash_calls >= 64);

    for value: 0..63 {
        found, result := HT.table_find(*table, .{ value });
        assert(found && result == "payload");
    }

    assert(compare_calls > 0);

    for value: 0..31 {
        found, result := HT.table_remove(*table, .{ value });
        assert(found && result == "payload");
    }

    assert(table.count == 32);
    HT.table_set(*table, .{ 63 }, "updated");
    found, result := HT.table_find(*table, .{ 63 });

    assert(found && result == "updated" && table.count == 32);
    HT.table_reset_keeping_memory(*table);
    assert(table.count == 0 && table.allocated > 0);
    HT.table_add(*table, .{ 99 }, "reused");
    found, result = HT.table_find(*table, .{ 99 });
    assert(found && result == "reused");
    print("ok\n");
}

// Forward references in the specialized Table above remain valid.
hash_key :: (key: Key) -> u32 {
    hash_calls += 1;
    return 11;
}

same_key :: (a: Key, b: Key) -> bool {
    compare_calls += 1;
    return a.value == b.value;
}
