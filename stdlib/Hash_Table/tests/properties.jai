// A table behaves like a list of key and value pairs under any sequence of sets and removes.
#import "Basic";
#import "Extensions/Testing";

Operation :: struct {
    key: s8;
    value: s32;
    deletes: bool;
}

// Few keys, so the same one comes up again and again.
few_keys :: (rng: *Rng) -> s8 {
    return cast(s8) rng_range(rng, -6, 6);
}

behaves_like_a_list_of_pairs :: (operations: [] Operation) {
    table: H.Table(s8, s32);

    defer H.deinit(*table);

    keys: [..] s8;
    values: [..] s32;

    keys.allocator = temp;
    values.allocator = temp;

    for operation: operations {
        at := -1;
        for keys if it == operation.key at = it_index;

        if operation.deletes {
            ok, removed := H.table_remove(*table, operation.key);

            expect_equal(ok, at >= 0);

            if at >= 0 {
                expect_equal(removed, values[at]);
                array_ordered_remove_by_index(*keys, at);
                array_ordered_remove_by_index(*values, at);
            }
        } else {
            H.table_set(*table, operation.key, operation.value);

            if at >= 0 {
                values[at] = operation.value;
            } else {
                array_add(*keys, operation.key);
                array_add(*values, operation.value);
            }
        }

        expect_equal(table.count, keys.count);
    }

    for keys {
        ok, value := H.table_find(*table, it);
        expect(ok, "a key that was set is not found");
        expect_equal(value, values[it_index]);
    }
} @test @property(200) @generate(few_keys)

#scope_file

H :: #import "Hash_Table";
