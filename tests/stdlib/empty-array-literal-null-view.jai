// `.[]` converted to a view has a null data pointer, so `if view.data free(view.data)` is safe.
#import "Basic";

empty :: () -> [] s64 { return .[]; }

main :: () -> s64 {
    v := empty();
    if v.count != 0 return 1;
    if v.data return 2;
    a := NewArray(0, s64);
    if a.data return 3;
    print("ok\n");
    return 0;
}
