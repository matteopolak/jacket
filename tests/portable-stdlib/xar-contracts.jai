#import "Basic";
#import "Xar";
X :: #import "Xar";

Owner :: struct {
    base: Allocator;
    allocations: s64;
    frees: s64;
}

allocate :: (mode: Allocator_Mode, requested_size: s64, old_size: s64, old_memory: *void, allocator_data: *void) -> *void {
    owner := cast(*Owner) allocator_data;
    result := owner.base.proc(mode, requested_size, old_size, old_memory, owner.base.data);

    if mode == .ALLOCATE && result owner.allocations += 1;
    if mode == .FREE && old_memory owner.frees += 1;

    return result;
}

compare :: (a: s64, b: s64) -> s64 {
    if a < b return -1;
    if a > b return 1;
    return 0;
}

main :: () {
    #assert !#exists(X.location);
    #assert !#exists(X.element_pointer);
    #assert !#exists(X.sort_in_place);
    if type_info(X.Xar(s64, 2, 4)).name != "Xar" exit(2);

    owner := Owner.{ base = context.allocator };
    other := Owner.{ base = context.allocator };
    saved := context.allocator;

    context.allocator = .{ allocate, *owner };
    array: X.Xar(s64, 2, 4);
    pointers: [32] *s64;

    for value: 0..31 {
        pointer := X.xar_add(*array, value);
        if !pointer || pointer.* != value exit(3);
        pointers[value] = pointer;
        for previous: 0..value if pointers[previous].* != previous exit(4);
    }

    context.allocator = .{ allocate, *other };
    if array.count != 32 || owner.allocations != 4 || other.allocations != 0 exit(5);
    for value: 0..31 if array[value] != value || *array[value] != pointers[value] exit(6);

    iterator := X.xar_iterator(*array, 0);

    for value: 0..31 {
        if iterator.it_index != value || X.xar_get(*array, iterator) != pointers[value] exit(7);

        next := X.xar_next(*array, *iterator);

        if value < 31 {
            if next != pointers[value + 1] exit(8);
        } else if next != null exit(9);
    }

    reverse_visits: s64;

    for < value: 0..31 {
        if iterator.it_index != value || X.xar_get(*array, iterator) != pointers[value] exit(10);

        previous := X.xar_prev(*array, *iterator);

        if value > 0 {
            if previous != pointers[value - 1] exit(11);
        } else if previous != null exit(12);

        reverse_visits += 1;
    }

    if reverse_visits != 32 exit(24);

    visited: s64;

    for value, index: array {
        if value != index || index != visited exit(13);
        visited += 1;
    }

    if visited != 32 exit(14);
    for *value: array value.* += 1;
    for value: 0..31 if array[value] != value + 1 exit(15);

    X.xar_unordered_remove(*array, 5);
    if array.count != 31 || pointers[5].* != 32 || array[5] != 32 || owner.frees != 0 exit(16);

    X.xar_add(*array, 6);
    if owner.allocations != 4 || array.count != 32 exit(17);

    for mode: 0..2 {
        for index: 0..31 array[index] = (index * 7) % 32;
        if mode == 0 X.quick_sort(array, compare);
        else if mode == 1 X.quick_sort_simple(array, compare);
        else X.quick_sort_iter(array, compare);
        for index: 0..31 if array[index] != index exit(18);
    }

    X.xar_reset(*array);
    if array.count != 0 || owner.frees != 4 || other.frees != 0 exit(19);
    for chunk: array.chunks if chunk != null exit(20);
    if array.allocator.proc != allocate || array.allocator.data != *owner exit(21);

    pointer := X.xar_add(*array, 77);
    if !pointer || pointer.* != 77 || owner.allocations != 5 || other.allocations != 0 exit(22);

    X.xar_reset(*array);
    context.allocator = saved;
    if owner.frees != 5 exit(23);

    write_string("ok\n");
}
