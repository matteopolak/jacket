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

#assert #run check_heap();
main :: () -> int { return 0; }
