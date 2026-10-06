// libc calls back into #c_call procedures: under the interpreter they are reached through
// native thunks (see docs/compiler/interpreter.md).
#import "Basic";

libc :: #system_library "libc";
qsort :: (base: *void, count: u64, size: u64, compare: (*void, *void) -> s32 #c_call) #foreign libc;
bsearch :: (key: *void, base: *void, count: u64, size: u64, compare: (*void, *void) -> s32 #c_call) -> *void #foreign libc;

by_value :: (a: *void, b: *void) -> s32 #c_call {
    x, y := (cast(*s32) a).*, (cast(*s32) b).*;
    return cast(s32) (x > y) - cast(s32) (x < y);
}

compared := 0;
counting :: (a: *void, b: *void) -> s32 #c_call {
    compared += 1; // Globals are shared with the interpreter.
    return by_value(a, b);
}

main :: () {
    xs := s32.[5, 3, 9, 1, 7];
    qsort(xs.data, xs.count, size_of(s32), by_value);
    print("%\n", xs);
    key: s32 = 7;
    found := cast(*s32) bsearch(*key, xs.data, xs.count, size_of(s32), counting);
    print("% at index %\n", found.*, found - xs.data);
    assert(compared > 0);
    assert(bsearch(*key, xs.data, xs.count, size_of(s32), counting) == found);
}
