Xar :: struct(Element_Type: Type, first_chunk_size_shift: u8, max_chunks: int) {
    count: s64;
    chunks: [max_chunks] *Element_Type;
    allocator: Allocator;
    Iterator :: struct {
        it_index: int;
        index_in_chunk: int;
        chunk_cap: int;
        chunk_pointer: **Element_Type;
    }
    #assert max_chunks > 0 && cast(s64) first_chunk_size_shift + max_chunks - 1 < 63;
}

xar_add :: (using array: *Xar, el: array.Element_Type) -> *array.Element_Type {
    chunk, offset, cap := location(array.*, count);
    if !chunks[chunk] {
        if !allocator.proc allocator = context.allocator;

        #if size_of(Element_Type) > 0 assert(cap <= 0x7fffffffffffffff / size_of(Element_Type));
        memory := allocator.proc(.ALLOCATE, cap * size_of(Element_Type), 0, null, allocator.data);
        if !memory return null;

        chunks[chunk] = cast(*Element_Type) memory;
    }

    result := chunks[chunk] + offset;
    result.* = el;
    count += 1;

    return result;
}

xar_reset :: (using array: *Xar) {
    cap := (cast(s64) 1) << first_chunk_size_shift;

    for i: 0..max_chunks - 1 {
        if chunks[i] allocator.proc(.FREE, 0, cap * size_of(Element_Type), chunks[i], allocator.data);
        chunks[i] = null;
        if i > 0 cap *= 2;
    }

    count = 0;
}

xar_unordered_remove :: (using array: *Xar, index: int) {
    assert(index >= 0 && index < count);
    destination := element_pointer(array.*, index);
    last := element_pointer(array.*, count - 1);

    destination.* = last.*;
    count -= 1;
}

operator *[] :: inline (using array: Xar, index: int) -> *array.Element_Type {
    return element_pointer(array, index);
}

xar_iterator :: inline (using array: *Xar, index: int) -> array.Iterator {
    chunk, offset, cap := location(array.*, index);
    return .{ it_index = index, index_in_chunk = offset, chunk_cap = cap, chunk_pointer = *chunks[chunk] };
}

xar_get :: inline (using array: *Xar, iterator: array.Iterator) -> *array.Element_Type {
    assert(iterator.it_index >= 0 && iterator.it_index < count && iterator.chunk_pointer != null);
    return iterator.chunk_pointer.* + iterator.index_in_chunk;
}

xar_next :: (using array: *Xar, iterator: *array.Iterator) -> *array.Element_Type {
    if iterator.it_index + 1 >= count return null;
    iterator.* = xar_iterator(array, iterator.it_index + 1);
    return xar_get(array, iterator.*);
}

xar_prev :: (using array: *Xar, iterator: *array.Iterator) -> *array.Element_Type {
    if iterator.it_index <= 0 return null;
    iterator.* = xar_iterator(array, iterator.it_index - 1);
    return xar_get(array, iterator.*);
}

for_expansion :: (using array: *$T/Xar, body: Code, flags: For_Flags) #expand {
    #assert (flags & .REVERSE) == 0, "reverse Xar iteration is unsupported";

    for index: 0..count - 1 {
        pointer := element_pointer(array.*, index);

        `it_index := index;

        #if (flags & .POINTER) != 0 {
            `it := pointer;
            #insert body;
        } else {
            `it := pointer.*;
            #insert body;
        }
    }
}

quick_sort :: quick_sort_iter;

quick_sort_simple :: (array: $T/Xar, f: (T.Element_Type, T.Element_Type) -> $R) {
    sort_in_place(array, f);
}

quick_sort_iter :: (array: $T/Xar, f: (T.Element_Type, T.Element_Type) -> $R) {
    sort_in_place(array, f);
}

#scope_file
#import "Basic";

location :: (array: $T/Xar, index: int) -> (chunk: int, offset: int, cap: int) {
    assert(index >= 0);
    cap := (cast(s64) 1) << array.first_chunk_size_shift;
    chunk: s64;
    offset := index;

    while offset >= cap {
        offset -= cap;
        if chunk > 0 cap *= 2;
        chunk += 1;
        assert(chunk < array.max_chunks, "Xar configured capacity exceeded");
    }

    return chunk, offset, cap;
}

element_pointer :: (array: $T/Xar, index: int) -> *array.Element_Type {
    chunk, offset := location(array, index);
    assert(array.chunks[chunk] != null);
    return array.chunks[chunk] + offset;
}

sort_in_place :: (array: $T/Xar, f: (T.Element_Type, T.Element_Type) -> $R) {
    root := array.count / 2;
    while root > 0 {
        root -= 1;
        sift(array, root, array.count, f);
    }

    end := array.count;
    while end > 1 {
        end -= 1;
        swap(element_pointer(array, 0), element_pointer(array, end));
        sift(array, 0, end, f);
    }
}

sift :: (array: $T/Xar, root: s64, count: s64, f: (T.Element_Type, T.Element_Type) -> $R) {
    index := root;
    while index * 2 + 1 < count {
        child := index * 2 + 1;
        if child + 1 < count && f(element_pointer(array, child).*, element_pointer(array, child + 1).*) < 0 child += 1;
        if f(element_pointer(array, index).*, element_pointer(array, child).*) >= 0 break;

        swap(element_pointer(array, index), element_pointer(array, child));
        index = child;
    }
}
