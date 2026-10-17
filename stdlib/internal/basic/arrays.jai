array_reserve :: inline (array: *[..] $T, desired_items: s64) {
    array_reserve(cast(*Resizable_Array) array, desired_items, size_of(T));
    assert(cast(u64) array.data % align_of(T) == 0, "allocator returned storage with insufficient alignment");
}

array_reserve :: inline (array: *Resizable_Array, desired_items: s64, element_size: s64) {
    assert(desired_items >= 0 && element_size > 0);
    if desired_items <= array.allocated return;
    if !array.allocator.proc array.allocator = context.allocator;

    assert(desired_items <= 0x7fffffffffffffff / element_size);
    memory := allocator_call(array.allocator, .RESIZE, desired_items * element_size, array.allocated * element_size, array.data);

    assert(memory != null, "array allocation failed");
    array.data = memory;
    array.allocated = desired_items;
}

maybe_grow :: inline (array: *Resizable_Array, element_size: s64) {
    if array.count < array.allocated return;
    capacity := max(cast(s64) 8, array.allocated * 2);
    array_reserve(array, capacity, element_size);
}

array_add :: (array: *[..] $T) -> *T {
    maybe_grow(cast(*Resizable_Array) array, size_of(T));
    result := array.data + array.count;

    Initialize(result);
    array.count += 1;

    return result;
}

array_add :: (array: *[..] $T, item: T) #no_abc {
    maybe_grow(cast(*Resizable_Array) array, size_of(T));
    array.data[array.count] = item;
    array.count += 1;
}

array_add :: (array: *[..] $T, to_append: ..T) {
    if to_append.count == 0 return;

    assert(to_append.count <= 0x7fffffffffffffff - array.count);
    desired := array.count + to_append.count;
    source := to_append;
    source_offset: s64 = -1;
    source_address := cast(u64) source.data;
    array_address := cast(u64) array.data;

    if source_address >= array_address && source_address - array_address < cast(u64) (array.count * size_of(T)) {
        source_offset = cast(s64) (source_address - array_address);
    }

    if desired > array.allocated {
        capacity := desired;
        if array.allocated <= 0x7fffffffffffffff / 2 capacity = max(capacity, array.allocated * 2);
        array_reserve(array, max(cast(s64) 8, capacity));
        if source_offset >= 0 source.data = cast(*T) ((cast(*u8) array.data) + source_offset);
    }

    copy_array_elements(array.data + array.count, source);
    array.count = desired;
}

array_find :: (array: [] $T, item: T, $compare: (T, T) -> bool = null) -> bool, s64 {
    for value, i: array {
        #if compare {
            if compare(value, item) return true, i;
        } else {
            if value == item return true, i;
        }
    }

    return false, -1;
}

array_add_if_unique :: (array: *[..] $T, item: T, $compare: (T, T) -> bool = null) -> bool, s64 {
    found, index := array_find(array.*, item, compare);
    if found return false, index;
    array_add(array, item);
    return true, array.count - 1;
}

array_view :: inline (array: [] $T, index: int) -> [] T {
    return array_view(array, index, array.count - index);
}

array_view :: inline (array: [] $T, index: int, count: int) -> [] T {
    assert(index >= 0);
    if index >= array.count return .{};
    assert(count >= 0);
    return .{ min(count, array.count - index), array.data + index };
}

array_insert_at :: (array: *[..] $T, item: T, index: int) {
    assert(index >= 0 && index <= array.count);
    maybe_grow(cast(*Resizable_Array) array, size_of(T));
    i := array.count;
    while i > index {
        array.data[i] = array.data[i - 1];
        i -= 1;
    }

    array.data[index] = item;
    array.count += 1;
}

array_ordered_remove_by_index :: inline (array_pointer: *[] $T, index: s64) {
    assert(index >= 0 && index < array_pointer.count);
    i := index;
    while i + 1 < array_pointer.count {
        array_pointer.data[i] = array_pointer.data[i + 1];
        i += 1;
    }

    array_pointer.count -= 1;
} @RemovesElements

array_unordered_remove_by_index :: inline (array_pointer: *[] $T, index: s64) {
    assert(index >= 0 && index < array_pointer.count);
    array_pointer.count -= 1;
    array_pointer.data[index] = array_pointer.data[array_pointer.count];
} @RemovesElements

array_ordered_remove_by_value :: inline (array_pointer: *[] $T, item: T) -> bool {
    found, index := array_find(array_pointer.*, item);
    if found array_ordered_remove_by_index(array_pointer, index);
    return found;
} @RemovesElements

array_unordered_remove_by_value :: inline (array: *[] $T, item: T, $stop_after_first := false, $compare: (T, T) -> bool = null) -> s64 {
    removed: s64;

    while true {
        found, index := array_find(array.*, item, compare);
        if !found break;

        array_unordered_remove_by_index(array, index);
        removed += 1;
        #if stop_after_first break;
    }

    return removed;
} @RemovesElements

peek_pointer :: inline (array: [] $T) -> *T {
    assert(array.count > 0);
    return array.data + array.count - 1;
}

peek :: inline (array: [] $T) -> T {
    return peek_pointer(array).*;
}

pop :: (array: *[] $T) -> T {
    result := peek(array.*);
    array.count -= 1;
    return result;
}

array_reset_keeping_memory :: inline (array: *[..] $T) {
    array.count = 0;
}

array_free :: inline (array: [] $T) {
    free(array.data);
} @Releases

array_free :: (array: [..] $T) {
    if array.allocator.proc allocator_call(array.allocator, .FREE, 0, array.allocated * size_of(T), array.data);
    else free(array.data);
} @Releases

array_reset :: (array: *[] $T) {
    array_free(array.*);
    array.* = .{};
} @Releases

array_reset :: (array: *[..] $T) {
    array_free(array.*);
    array.* = .{};
} @Releases

array_resize :: (array: *[..] $T, new_count: s64, $initialize := true) {
    assert(new_count >= 0);
    array_reserve(array, new_count);

    #if initialize {
        i := array.count;
        while i < new_count {
            Initialize(array.data + i);
            i += 1;
        }
    }

    array.count = new_count;
}

array_resize :: (array: *[] $T, new_count: s64, $initialize := true) {
    assert(new_count >= 0 && new_count <= 0x7fffffffffffffff / max(cast(s64) 1, size_of(T)));

    if new_count == 0 {
        free(array.data);
        array.* = .{};
        return;
    }

    memory := cast(*T) realloc(array.data, new_count * size_of(T), array.count * size_of(T));

    assert(memory != null || size_of(T) == 0, "array allocation failed");
    #if initialize for index: array.count..new_count - 1 Initialize(memory + index);
    array.data = memory;
    array.count = new_count;
}

remember_allocators :: (data: *$T) {
    data.allocator = ifx context.allocator.proc then context.allocator else context.default_allocator;
}

resizable :: (a: [] $T, set_allocators := true) -> [..] T {
    result: [..] T = ---;
    result.data = a.data;
    result.count = a.count;
    result.allocated = a.count;
    if set_allocators remember_allocators(*result);

    return result;
}

array_copy :: (array: [] $T) -> [] T {
    result: [] T = .{ array.count, cast(*T) alloc(array.count * size_of(T)) };
    assert(result.data != null || result.count == 0);
    memcpy(result.data, array.data, array.count * size_of(T));
    return result;
} @UseResult

array_copy :: (dest: *[..] $T, source: [] T) {
    array_resize(dest, source.count, false);
    copy_array_elements(dest.data, source);
}

#scope_file
copy_array_elements :: (destination: *$T, source: [] T) {
    if source.count == 0 || destination == source.data return;

    destination_address := cast(u64) destination;
    source_address := cast(u64) source.data;
    bytes := cast(u64) (source.count * size_of(T));

    if destination_address > source_address {
        if destination_address - source_address >= bytes {
            memcpy(destination, source.data, cast(s64) bytes);
            return;
        }

        remaining := source.count;
        while remaining > 0 {
            remaining -= 1;
            destination[remaining] = source[remaining];
        }
    } else {
        if source_address - destination_address >= bytes {
            memcpy(destination, source.data, cast(s64) bytes);
            return;
        }

        for item, index: source destination[index] = item;
    }
}
