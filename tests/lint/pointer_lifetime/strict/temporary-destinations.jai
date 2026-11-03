#import "Basic";

good_temporary_destination :: () {
    push_allocator(temp);
    values := NewArray(1, string);
    values[0] = tprint("%", 1);
    assert(values[0] == "1");
}

bad_heap_destination :: () -> [] string {
    values := NewArray(1, string);
    values[0] = tprint("%", 1);
    return values;
}

bad_stack_in_temporary_destination :: () -> [] *int {
    push_allocator(temp);
    values := NewArray(1, *int);
    local := 1;

    values[0] = *local;

    return values;
} @TempAllocates

good_scoped_heap_destination :: (count: int) {
    values := NewArray(count, string);
    defer free(values.data);
    for values values[it_index] = tprint("%", it_index);
    for values assert(it.count > 0);
}

bad_indexed_return :: (index: int) -> [] string {
    values := NewArray(1, string);
    values[index] = tprint("%", index);
    return values;
}
