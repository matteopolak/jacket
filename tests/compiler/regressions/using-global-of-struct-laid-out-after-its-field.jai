// `using g;` of a global pointer to a struct, when one of that struct's field types is laid out
// first: a name looked up from the field type's own fields (`int`) asks whether the outer struct
// has such a member, which must not lay the outer struct out.
#import "Basic";

Inner :: struct {
    x: int;
}

Context_Data :: struct {
    inner: Inner;
    count: s64;
}

gpu: *Context_Data;
using gpu;

main :: () {
    first: Inner;
    first.x = 2;
    data: Context_Data;

    gpu = *data;
    count = 4;
    inner.x = first.x + 1;
    assert(data.count == 4 && data.inner.x == 3);
    print("ok\n");
}
