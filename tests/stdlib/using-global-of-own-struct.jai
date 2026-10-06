// `using g;` of a global pointer to a struct does not make the struct contain itself: names in
// its field types (`s64`, `Inner`) are looked up while it is being laid out
// (UnNabbo/no_api's `using gpu_context;`).
// rules: using.17
#import "Basic";

Inner :: struct {
    x: int;
}

Context_Data :: struct {
    inner: Inner;
    count: s64;
    Limit :: 8;
    table: [Limit] u8;
}

gpu: *Context_Data;
using gpu;

main :: () {
    data: Context_Data;
    gpu = *data;
    count = 4;
    inner.x = 3;
    assert(data.count == 4 && data.inner.x == 3);
    assert(size_of(Context_Data) == 24);
    print("ok\n");
}
