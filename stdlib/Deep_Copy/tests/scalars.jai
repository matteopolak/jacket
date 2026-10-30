#import "Basic";
D :: #import "Deep_Copy";
H :: #import "Hash_Table";

mapped :: (input: string, data: *void) -> string {
    calls := cast(*s64) data;
    calls.* += 1;
    return "mapped";
}

main :: () {
    #assert !#exists(D.clone_object);
    value: s64 = -91;
    clone := D.Deep_Copy(*value, .{});

    assert(clone != *value && clone.* == value);
    clone.* = 7;
    assert(value == -91);
    free(clone);
    preallocated: s64;

    assert(D.Deep_Copy(*value, .{}, *preallocated) == *preallocated && preallocated == -91);
    assert(D.Deep_Copy(cast(*s64) null, .{}) == null);

    text := "owned";
    copy := D.Deep_Copy(*text, .{});

    assert(copy.* == text && copy.data != text.data);
    free(copy.*);
    free(copy);
    calls: s64;

    copy = D.Deep_Copy(*text, .{ string_mapper_proc = mapped, string_mapper_data = *calls });
    assert(copy.* == "mapped" && calls == 1);
    free(copy);

    source: [..] u32;

    array_add(*source, cast(u32) 3);
    array_add(*source, cast(u32) 5);
    defer array_free(source);
    array := D.Deep_Copy(*source, .{});

    assert(array.count == 2 && array.allocated == source.allocated && array.data != source.data);
    assert(array.allocator.proc == context.allocator.proc && array.*[1] == 5);
    array.*[0] = 77;
    assert(source[0] == 3);
    array_free(array.*);
    free(array);

    map: D.Pointer_Map;
    first := D.Deep_Copy(*value, .{}, pointer_map = *map);
    second := D.Deep_Copy(*value, .{}, pointer_map = *map);

    assert(first == second && first.* == value);
    H.deinit(*map.table);
    free(first);
    assert(D.helper_count > 0 && D.time_total >= 0);
    print("ok\n");
}
