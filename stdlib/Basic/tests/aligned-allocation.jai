#import "Basic";

Wide :: struct #align 64 {
    value: s64;
}

#add_context wide: Wide;

main :: () {
    assert(cast(u64) *context % align_of(#Context) == 0);
    pointers: [32] *Wide;

    for i: 0..31 {
        pointers[i] = New(Wide);
        assert(cast(u64) pointers[i] % 64 == 0);
        pointers[i].value = i;
    }

    for p, i: pointers {
        assert(p.value == i);
        free(p);
    }

    array: [..] Wide;
    for i: 0..16 array_add(*array, Wide.{ i });

    assert(cast(u64) array.data % 64 == 0);
    for value, i: array assert(value.value == i);

    array_free(array);
    values, base := NewArray(7, Wide, alignment = 256);

    assert(cast(u64) values.data % 256 == 0 && values[6].value == 0);
    free(base);
    memory := cast(*u8) alloc(1024);
    for i: 0..1023 memory[i] = cast,truncate(u8) i;

    memory = cast(*u8) realloc(memory, 33, 1024);
    assert(memory != null);
    for i: 0..32 assert(memory[i] == cast(u8) i);

    memory = cast(*u8) realloc(memory, 256, 33);
    assert(memory != null);
    for i: 0..32 assert(memory[i] == cast(u8) i);

    free(memory);
    mark := get_temporary_storage_mark();

    {
        push_allocator(temp);
        p := New(Wide);
        assert(cast(u64) p % 64 == 0);
    }

    set_temporary_storage_mark(mark);
    print("ok\n");
}
