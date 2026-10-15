// Numeric slow paths, shortest formatting and empty keys release their allocations.
#import "Basic";
#import "Extensions/Json";
#import "Hash_Table";

outstanding: int;
allocations: int;
backing: Allocator;

tracked :: (mode: Allocator_Mode, size: s64, old_size: s64, memory: *void, data: *void) -> *void {
    result := backing.proc(mode, size, old_size, memory, backing.data);
    if (mode == .ALLOCATE || (mode == .RESIZE && !memory)) && result {
        outstanding += 1;
        allocations += 1;
    }

    if mode == .FREE && memory outstanding -= 1;

    return result;
}

main :: () {
    backing = context.allocator;

    {
        push_allocator(tracked, null);

        for 1..20 {
            wide, error := from_json(float64, "1.00000000000000011102230246251565404236316680908203125");

            assert(error.kind == .NONE && wide == 1.0);
            free_json(*error);

            narrow, error32 := from_json(float32, "1.0000000596046447753906250000000000000000001");

            assert(error32.kind == .NONE && (cast,force(u32) narrow) == 0x3f800001);
            free_json(*error32);

            subnormal := to_json(cast,force(float64) cast(u64) 1);

            assert(subnormal == "5e-324");
            free(subnormal);
            text := to_json(narrow);

            free(text);

            _, bad := from_json(float64, "1e999999");

            assert(bad.kind == .OUT_OF_RANGE);
            free_json(*bad);

            object, object_error := from_json(Json_Value, "{\"\":1}");

            assert(object_error.kind == .NONE);
            assert(object.members.count == 1 && object.members[0].key == "" && object.members[0].value.text == "1");
            free_json(*object);
            free_json(*object_error);

            table, table_error := from_json(Table(string, int), "{\"\":1}");

            assert(table_error.kind == .NONE);
            entry := table_find_pointer(*table, "");

            assert(entry != null && entry.* == 1);
            free_json(*table);
            free_json(*table_error);
        }
    }

    assert(allocations > 0 && outstanding == 0, "numeric paths leaked % allocations", outstanding);
    print("ok\n");
}
