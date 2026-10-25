#import "Basic";
#import "Extensions/Testing";

freeing_goes_to_the_current_allocator :: () {
    owner = .{};
    owner.backing = context.allocator;

    {
        push_allocator(tracked, *owner);
        memory := New(int);

        free(memory);
        text := copy_string("free through the current allocator");

        free(text);
    }
} @test @serial

deinit_uses_the_allocator_the_table_was_built_with :: () {
    owner = .{};
    owner.backing = context.allocator;

    verify(cast(s32) -71);
    verify(cast(u64) 0xffffffffffffffff);
    verify(true);
    verify(Choice.B);
    verify(cast(float32) 1.25);
    verify(cast(float64) -12.5);
    verify("shared supported string hash");
    value: u8;

    verify(*value);
    custom: H.Table(Key, int, key_hash, key_equal);

    {
        push_allocator(tracked, *owner);
        H.table_add(*custom, Key.{ 11 }, 71);
    }

    expect_equal(H.table_find_pointer(*custom, Key.{ 11 }).*, 71);
    H.deinit(*custom);
    expect(owner.allocated > 0 && owner.allocated == owner.freed);
} @test @serial

#scope_file

H :: #import "Hash_Table";

Owner :: struct {
    allocated: int;
    freed: int;
    backing: Allocator;
}

owner: Owner;

tracked :: (mode: Allocator_Mode, size: s64, old_size: s64, memory: *void, data: *void) -> *void {
    state := cast(*Owner) data;
    result := state.backing.proc(mode, size, old_size, memory, state.backing.data);

    if result && (mode == .ALLOCATE || (mode == .RESIZE && !memory)) state.allocated += 1;
    if mode == .FREE && memory state.freed += 1;

    return result;
}

verify :: (key: $T) {
    table: H.Table(T, int);

    {
        push_allocator(tracked, *owner);
        H.table_add(*table, key, 17);
    }

    // Deinit must use the table's recorded owner, not this current allocator.
    pointer := H.table_find_pointer(*table, key);

    require(pointer != null);
    expect_equal(pointer.*, 17);
    H.table_set(*table, key, 29);
    expect(table.count == 1 && H.table_find_pointer(*table, key).* == 29);
    H.table_add(*table, key, 37);
    expect(table.count == 2 && H.table_find_pointer(*table, key).* == 29);
    H.deinit(*table);
}

Choice :: enum {
    A;
    B;
}

Key :: struct {
    number: int;
}

key_hash :: (key: Key) -> u32 {
    return cast(u32) key.number;
}

key_equal :: (a: Key, b: Key) -> bool {
    return a.number == b.number;
}
