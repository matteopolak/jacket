O :: #import "Overwriting_Allocator";

main :: () {
    allocator := O.get_overwriting_allocator();
    memory := allocator.proc(.ALLOCATE, 16, 0, null, allocator.data);
    allocator.proc(.FREE, 0, 0, memory, allocator.data);
    allocator.proc(.FREE, 0, 0, memory, allocator.data);
}
