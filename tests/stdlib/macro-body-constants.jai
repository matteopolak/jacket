// Constants in a for_expansion body exist in every expansion (here two loops in one procedure).
#import "Basic";

Array :: struct(T: Type) {
    data: *T;
    count: s32;
}

for_expansion :: (array: *Array($T), body: Code, flags: For_Flags) #expand {
    DO_POINTER :: cast(bool) (flags & .POINTER);
    `it_index: s32;
    #if DO_POINTER {
        `it: *T;
    } else {
        `it: T;
    }
    it_index = -1;
    while it_index < array.count - 1 {
        it_index += 1;
        #if DO_POINTER {
            it = *array.data[it_index];
        } else {
            it = array.data[it_index];
        }
        #insert body;
    }
}

main :: () {
    v := int.[1, 2, 3];
    a: Array(int);
    a.data = v.data;
    a.count = 3;
    sum := 0;
    for a sum += it;
    for *a it.* += 1;
    for a sum += it;
    assert(sum == 15);
}
