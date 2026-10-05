Default :: #import "Default_Allocator";

check_heap :: () -> bool {
    allocator := Default.allocator;
    memory := allocator.proc(.ALLOCATE, 4, 0, null, allocator.data);
    if !memory return false;
    memset(memory, 0, 4);
    (cast(*u8) memory)[0] = 71;
    resized := allocator.proc(.RESIZE, 8, 4, memory, allocator.data);
    if !resized return false;
    if (cast(*u8) resized)[0] != 71 return false;
    if !allocator.proc(.IS_THIS_YOURS, 0, 0, resized, allocator.data) return false;
    heap := allocator.proc(.CREATE_HEAP, 0, 0, null, allocator.data);
    if !heap return false;
    independent := allocator.proc(.ALLOCATE, 16, 0, null, heap);
    if !independent return false;
    if allocator.proc(.IS_THIS_YOURS, 0, 0, independent, allocator.data) return false;
    if !allocator.proc(.IS_THIS_YOURS, 0, 0, independent, heap) return false;
    allocator.proc(.DESTROY_HEAP, 0, 0, null, heap);
    allocator.proc(.FREE, 0, 0, resized, allocator.data);
    return true;
}

// Many live allocations, freed and resized out of order: the ledger must keep finding each one.
check_many :: () -> bool {
    allocator := Default.allocator;
    heap := allocator.proc(.CREATE_HEAP, 0, 0, null, allocator.data);
    COUNT :: 2000;
    pointers: [COUNT] *void;
    for i: 0..COUNT-1 {
        pointers[i] = allocator.proc(.ALLOCATE, 8 + i % 97, 0, null, heap);
        if !pointers[i] return false;
        (cast(*s64) pointers[i]).* = i;
    }
    // Free every third, resize every fifth of the rest, in a scrambled order.
    for k: 0..COUNT-1 {
        i := (k * 7919) % COUNT;
        if i % 3 == 0 {
            allocator.proc(.FREE, 0, 0, pointers[i], heap);
            pointers[i] = null;
        } else if i % 5 == 0 {
            pointers[i] = allocator.proc(.RESIZE, 4096, 8, pointers[i], heap);
            if !pointers[i] || (cast(*s64) pointers[i]).* != i return false;
        }
    }
    for i: 0..COUNT-1 {
        owned := allocator.proc(.IS_THIS_YOURS, 0, 0, pointers[i], heap) != null;
        if owned != (pointers[i] != null) return false;
        if pointers[i] && (cast(*s64) pointers[i]).* != i return false;
    }
    for i: 0..COUNT-1 if pointers[i] && i % 2 allocator.proc(.FREE, 0, 0, pointers[i], heap);
    allocator.proc(.DESTROY_HEAP, 0, 0, null, heap);
    return true;
}

#assert #run check_heap();
#assert #run check_many();
main :: () -> int { return 0; }
