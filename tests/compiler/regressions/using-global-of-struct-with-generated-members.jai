// `using g;` of a global pointer to a struct whose members come from `#if` and `#insert`, when
// one of its field types is laid out first: asking whether the outer struct has a member collects
// its members once, without laying it out.
#import "Basic";

WIDE :: true;

Inner :: struct {
    x: int;
}

Context_Data :: struct {
    inner: Inner;
    #if WIDE {
        count: s64;
    } else {
        count: s32;
    }
    #insert "label: string;";
}

gpu: *Context_Data;
using gpu;

main :: () {
    first: Inner;
    first.x = 2;
    data: Context_Data;

    gpu = *data;
    count = 4;
    label = "set";
    inner.x = first.x + 1;
    assert(data.count == 4 && data.inner.x == 3 && data.label == "set");
    assert(size_of(type_of(data.count)) == 8);
    print("ok\n");
}
