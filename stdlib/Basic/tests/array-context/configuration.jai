A :: #import "Basic"()(TEMP_ALLOCATOR_OVERWRITE_MEMORY = true);
B :: #import "Basic";

#assert A.S128 == B.S128;
#assert A.Apollo_Time == B.Apollo_Time;
#assert A.String_Builder == B.String_Builder;
#assert A.maybe_grow == B.maybe_grow;
#assert !#exists(A.copy_array_elements);

main :: () {
    values := int.[1, 2];
    found, index := A.array_find(values, 2);
    B.assert(found && index == 1);
}
