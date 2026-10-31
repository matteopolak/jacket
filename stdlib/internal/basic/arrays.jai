// Makes room for at least `desired_items` elements without changing the count.
//
// Nothing happens when the capacity is already enough, so it never shrinks. The first allocation uses
// `context.allocator` and the array keeps that allocator from then on; later growth goes to the same one.
// Growing may move the elements, which makes every pointer into the array stale. The new space is not
// initialized. A negative `desired_items`, or one whose byte size does not fit an `s64`, fails an assert.
//
// ```jai
// values: [..] int;
// array_reserve(*values, 100);
// assert(values.count == 0 && values.allocated >= 100);
//
// data := values.data;
// for 0..99 array_add(*values, it);
// assert(values.data == data); // no growth, so nothing moved
// array_free(values);
// ```
array_reserve :: inline (array: *[..] $T, desired_items: s64) {
    array_reserve(cast(*Resizable_Array) array, desired_items, size_of(T));
    assert(cast(u64) array.data % align_of(T) == 0, "allocator returned storage with insufficient alignment");
} @GrowsArray

// The untyped form of `array_reserve`, for code that only knows the element size in bytes.
array_reserve :: inline (array: *Resizable_Array, desired_items: s64, element_size: s64) {
    assert(desired_items >= 0 && element_size > 0);
    if desired_items <= array.allocated return;
    if !array.allocator.proc array.allocator = context.allocator;

    assert(desired_items <= 0x7fffffffffffffff / element_size);
    memory := allocator_call(array.allocator, .RESIZE, desired_items * element_size, array.allocated * element_size, array.data);

    assert(memory != null, "array allocation failed");
    array.data = memory;
    array.allocated = desired_items;
} @GrowsArray

maybe_grow :: inline (array: *Resizable_Array, element_size: s64) {
    if array.count < array.allocated return;
    capacity := max(cast(s64) 8, array.allocated * 2);
    array_reserve(array, capacity, element_size);
}

// Appends an element to a dynamic array and returns a pointer to it.
//
// With no item the new element is initialized to the default value of `T`; with one it is set to `item`
// (and nothing is returned); with several, all of them are appended in order. An empty list does nothing.
// The array grows when it is full: the first allocation holds 8 elements, after that the capacity doubles,
// or is exactly enough when a long list is added at once. Growth uses the array's own allocator, or
// `context.allocator` when it has none yet, and may move the elements, so a pointer from an earlier call is
// only good until the next add. Appending a view of the array to itself is fine.
//
// ```jai
// values: [..] int;
// array_add(*values, 10);
// array_add(*values, 20, 30);
//
// slot := array_add(*values); // a zeroed element, here at the end
// slot.* = 40;
//
// assert(values.count == 4 && values[0] == 10 && values[3] == 40);
// array_free(values);
// ```
array_add :: (array: *[..] $T) -> *T {
    maybe_grow(cast(*Resizable_Array) array, size_of(T));
    result := array.data + array.count;

    Initialize(result);
    array.count += 1;

    return result;
} @AddsToArray @GrowsArray

// Appends `item`; see the first `array_add` for how the array grows.
array_add :: (array: *[..] $T, item: T) #no_abc {
    maybe_grow(cast(*Resizable_Array) array, size_of(T));
    array.data[array.count] = item;
    array.count += 1;
} @AddsToArray @GrowsArray

// Appends all of `to_append` in order; see the first `array_add` for how the array grows.
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
} @GrowsArray

// Searches for `item`; returns whether it was found and the index of its first occurrence, or `-1`.
//
// Elements are compared with `==`, or with `compare(element, item)` when a comparator is given. Searching is
// linear and the empty array finds nothing. Long arrays of integers, enums and pointers are searched with
// vector instructions, which does not change the result. A given `compare` always takes the plain loop.
//
// ```jai
// values := int.[5, 8, 5, 13];
// found, index := array_find(values, 5);
// assert(found && index == 0);
//
// found, index = array_find(values, 6);
// assert(!found && index == -1);
//
// same_half :: (a: int, b: int) -> bool { return a / 2 == b / 2; }
// found, index = array_find(values, 9, same_half); // 8 and 9 share a half
// assert(found && index == 1);
// ```
//
// - compare: decides whether two elements are equal. `==` when not given.
array_find :: (array: [] $T, item: T, $compare: (T, T) -> bool = null) -> bool, s64 {
    // A long array of integers, enums or pointers is searched a vector at a time.
    #if !compare && ArrayScan.is_scannable(T) {
        if array.count >= ARRAY_SCAN_MIN {
            index := ArrayScan.find(array.data, array.count, item);
            return index >= 0, index;
        }
    }

    for value, i: array {
        #if compare {
            if compare(value, item) return true, i;
        } else {
            if value == item return true, i;
        }
    }

    return false, -1;
}

// Appends `item` unless the array already holds an equal element.
//
// Returns whether it was appended and its index: the new last index, or the index of the equal element that
// was already there. Equality is `==`, or `compare` when given, as for `array_find`. Costs a linear
// search, so it suits small sets; keep a `Table` for large ones.
//
// ```jai
// values: [..] string;
// added, index := array_add_if_unique(*values, "red");
// assert(added && index == 0);
// added, index = array_add_if_unique(*values, "blue");
// assert(added && index == 1);
//
// added, index = array_add_if_unique(*values, "red");
// assert(!added && index == 0 && values.count == 2);
// array_free(values);
// ```
array_add_if_unique :: (array: *[..] $T, item: T, $compare: (T, T) -> bool = null) -> bool, s64 {
    found, index := array_find(array.*, item, compare);
    if found return false, index;
    array_add(array, item);
    return true, array.count - 1;
} @GrowsArray

// A view of the elements of `array` from `index` on, or of `count` of them from `index`.
//
// The view shares the array's memory, so writes through it change the array, and it is only good while the
// array is not freed or grown. It is clamped, not an error, to run past the end: a `count` that is too large
// is cut short, and an `index` at or beyond the end gives an empty view. A negative `index` or `count` fails
// an assert.
//
// ```jai
// values := int.[1, 2, 3, 4, 5];
// middle := array_view(values, 1, 3);
// assert(middle.count == 3 && middle[0] == 2);
//
// tail := array_view(values, 3);
// assert(tail.count == 2 && tail[1] == 5);
//
// middle[0] = 20; // writes into `values`
// assert(values[1] == 20);
// assert(array_view(values, 9).count == 0);
// ```
array_view :: inline (array: [] $T, index: int) -> [] T {
    return array_view(array, index, array.count - index);
}

// At most `count` elements from `index`; see the first `array_view` for the clamping rules.
array_view :: inline (array: [] $T, index: int, count: int) -> [] T {
    assert(index >= 0);
    if index >= array.count return .{};
    assert(count >= 0);
    return .{ min(count, array.count - index), array.data + index };
}

// Inserts `item` so that it ends up at `index`, shifting the elements from there on up by one.
//
// `index` may equal the count, which appends. Anything else out of range fails an assert. The array grows as
// in `array_add`, and every element after `index` moves, so pointers to them are stale. Costs time linear in
// the number of elements shifted.
//
// ```jai
// list: [..] int;
// array_add(*list, 1, 2, 4);
// array_insert_at(*list, 3, 2);
// assert(list.count == 4 && list[2] == 3 && list[3] == 4);
// array_free(list);
// ```
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
} @GrowsArray

// Removes the element at `index` and shifts the later ones down, keeping their order.
//
// Takes a pointer to the array (a dynamic array or a view) and only lowers its count: the memory stays, and
// the removed element is not destroyed or freed. An out-of-range `index` fails an assert. Linear in the
// number of elements shifted; `array_unordered_remove_by_index` is constant time when order does not matter.
//
// ```jai
// values: [..] int;
// array_add(*values, 1, 2, 3, 4);
// array_ordered_remove_by_index(*values, 1);
// assert(values.count == 3 && values[1] == 3 && values[2] == 4);
// array_free(values);
// ```
array_ordered_remove_by_index :: inline (array_pointer: *[] $T, index: s64) {
    assert(index >= 0 && index < array_pointer.count);
    i := index;
    while i + 1 < array_pointer.count {
        array_pointer.data[i] = array_pointer.data[i + 1];
        i += 1;
    }

    array_pointer.count -= 1;
} @RemovesFromArray

// Removes the element at `index` by moving the last element into its place, in constant time.
//
// The order of the array changes (unless the removed element was the last one), so do not use it where order
// matters, or while iterating forward by index without revisiting `index`. Only the count is lowered; the
// memory is kept. An out-of-range `index` fails an assert.
//
// ```jai
// values: [..] int;
// array_add(*values, 1, 2, 3, 4);
// array_unordered_remove_by_index(*values, 0);
// assert(values.count == 3 && values[0] == 4 && values[1] == 2);
// array_free(values);
// ```
array_unordered_remove_by_index :: inline (array_pointer: *[] $T, index: s64) {
    assert(index >= 0 && index < array_pointer.count);
    array_pointer.count -= 1;
    array_pointer.data[index] = array_pointer.data[array_pointer.count];
} @RemovesFromArray

// Removes the first element equal to `item` (by `==`), keeping the order of the rest.
//
// Returns whether one was found and removed; later duplicates stay.
//
// ```jai
// values: [..] int;
// array_add(*values, 7, 8, 7);
// assert(array_ordered_remove_by_value(*values, 7));
// assert(values.count == 2 && values[0] == 8 && values[1] == 7);
// assert(!array_ordered_remove_by_value(*values, 99));
// array_free(values);
// ```
array_ordered_remove_by_value :: inline (array_pointer: *[] $T, item: T) -> bool {
    found, index := array_find(array_pointer.*, item);
    if found array_ordered_remove_by_index(array_pointer, index);
    return found;
} @RemovesFromArray

// Removes every element equal to `item`, each by moving the last element into its place; returns how many were removed.
//
// With `$stop_after_first` set it removes at most one. Equality is `==`, or `compare` when given. The order of
// the survivors is not kept. Each removal searches from the start again, so removing many matches from a long
// array is quadratic.
//
// ```jai
// values: [..] int;
// array_add(*values, 1, 2, 1, 3, 1);
// removed := array_unordered_remove_by_value(*values, 1);
// assert(removed == 3 && values.count == 2);
//
// once := array_unordered_remove_by_value(*values, 2, stop_after_first = true);
// assert(once == 1 && values.count == 1 && values[0] == 3);
// array_free(values);
// ```
//
// - stop_after_first: removes only the first match.
// - compare: decides whether two elements are equal. `==` when not given.
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
} @RemovesFromArray

// A pointer to the last element, which stays valid until the array is changed. Fails an assert when the array is empty.
peek_pointer :: inline (array: [] $T) -> *T {
    assert(array.count > 0);
    return array.data + array.count - 1;
}

// The last element of the array, without removing it. Fails an assert when the array is empty.
//
// ```jai
// values := int.[3, 6, 9];
// assert(peek(values) == 9);
// peek_pointer(values).* = 10;
// assert(values[2] == 10);
// ```
peek :: inline (array: [] $T) -> T {
    return peek_pointer(array).*;
}

// Removes the last element and returns it. Fails an assert when the array is empty.
//
// Only the count is lowered, so a pointer taken to the element earlier still reads its old value until
// the slot is reused.
//
// ```jai
// values: [..] int;
// array_add(*values, 1, 2, 3);
// assert(pop(*values) == 3 && pop(*values) == 2);
// assert(values.count == 1);
// array_free(values);
// ```
pop :: (array: *[] $T) -> T {
    result := peek(array.*);
    array.count -= 1;
    return result;
}

// Empties the array but keeps its storage, so refilling it up to the old capacity allocates nothing.
//
// The elements are not touched or destroyed; if they own memory, free that first.
//
// ```jai
// values: [..] int;
// array_add(*values, 1, 2, 3);
// capacity := values.allocated;
// array_reset_keeping_memory(*values);
// assert(values.count == 0 && values.allocated == capacity);
// array_free(values);
// ```
array_reset_keeping_memory :: inline (array: *[..] $T) {
    array.count = 0;
}

// Frees the storage of an array. A view is freed with `free`, i.e. through `context.allocator`; a dynamic
// array goes back to the allocator it grew with.
//
// The argument is not changed, so it dangles afterwards: use `array_reset` to free and clear in one go.
// Freeing an array that was never allocated (a zero-initialized one) is fine. Elements that own memory are
// not freed.
//
// ```jai
// values: [..] int;
// array_add(*values, 1, 2, 3);
// array_free(values);
// values = .{};
// ```
array_free :: inline (array: [] $T) {
    free(array.data);
} @FreesArgument

// Frees a dynamic array through the allocator it grew with, or `context.allocator` if it never allocated.
array_free :: (array: [..] $T) {
    if array.allocator.proc allocator_call(array.allocator, .FREE, 0, array.allocated * size_of(T), array.data);
    else free(array.data);
} @FreesArgument

// Frees the storage of an array (as `array_free` does) and sets the array to empty, so it can be reused.
//
// Safe on an array that never allocated. The elements themselves are not freed.
//
// ```jai
// values: [..] int;
// array_add(*values, 1, 2, 3);
// array_reset(*values);
// assert(values.count == 0 && values.data == null);
// ```
array_reset :: (array: *[] $T) {
    array_free(array.*);
    array.* = .{};
} @FreesArgument

// Frees a dynamic array through its own allocator and sets it to empty, which also forgets that allocator.
array_reset :: (array: *[..] $T) {
    array_free(array.*);
    array.* = .{};
} @FreesArgument

// Sets the count of an array to `new_count`, growing the storage if needed.
//
// Growth fills the new elements with the default value of `T`, unless `$initialize` is `false`, which leaves
// them uninitialized. Shrinking only lowers the count and keeps the storage. For a view (a `[]` array
// rather than a `[..]`) the storage is reallocated to exactly `new_count` elements through `context.allocator`,
// and a count of `0` frees it and leaves an empty view. A negative `new_count` fails an assert.
//
// ```jai
// values: [..] int;
// array_resize(*values, 4);
// assert(values.count == 4 && values[3] == 0);
//
// array_resize(*values, 2);
// assert(values.count == 2);
// array_free(values);
// ```
//
// - initialize: set to `false` to leave new elements as the allocator returned them instead of setting
//   them to their defaults.
array_resize :: (array: *[..] $T, new_count: s64, $initialize := true) {
    assert(new_count >= 0);
    array_reserve(array, new_count);

    #if initialize if new_count > array.count initialize_elements(array.data + array.count, new_count - array.count);

    array.count = new_count;
} @GrowsArray

// Resizes a view's storage to exactly `new_count` elements with `context.allocator`; see the first `array_resize`.
array_resize :: (array: *[] $T, new_count: s64, $initialize := true) {
    assert(new_count >= 0 && new_count <= 0x7fffffffffffffff / max(cast(s64) 1, size_of(T)));

    if new_count == 0 {
        free(array.data);
        array.* = .{};
        return;
    }

    memory := cast(*T) realloc(array.data, new_count * size_of(T), array.count * size_of(T));

    assert(memory != null || size_of(T) == 0, "array allocation failed");
    #if initialize if new_count > array.count initialize_elements(memory + array.count, new_count - array.count);
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

// A new array holding a copy of the elements, in memory from `context.allocator`.
//
// The caller owns the copy and frees it with `array_free`. The elements are copied shallowly, so pointers
// and strings in them still refer to the original data. With a destination, `array_copy(*dest, source)`
// instead replaces the contents of the dynamic array `dest`, reusing its storage when that is large enough,
// and the source may overlap `dest`.
//
// ```jai
// original := int.[1, 2, 3];
// copy := array_copy(original);
// copy[0] = 100;
// assert(original[0] == 1 && copy.count == 3);
// array_free(copy);
//
// list: [..] int;
// array_copy(*list, original);
// assert(list.count == 3 && list[2] == 3);
// array_free(list);
// ```
array_copy :: (array: [] $T) -> [] T {
    result: [] T = .{ array.count, cast(*T) alloc(array.count * size_of(T)) };
    assert(result.data != null || result.count == 0);
    memcpy(result.data, array.data, array.count * size_of(T));
    return result;
} @UseResult @Allocates

// Replaces the contents of `dest` with a copy of `source`; see the first `array_copy`.
array_copy :: (dest: *[..] $T, source: [] T) {
    array_resize(dest, source.count, false);
    copy_array_elements(dest.data, source);
}

#scope_file
ArrayScan :: #import "internal/array-scan";

// Arrays shorter than this are searched one item at a time: the vector search costs more to start.
ARRAY_SCAN_MIN :: 32;

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
