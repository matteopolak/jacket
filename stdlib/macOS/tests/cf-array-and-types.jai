// test: only(.MACOS)
// A CFArray built from CFString values keeps count and order, and CF type IDs tell arrays and strings apart.
#import "Basic";
M :: #import "macOS";

main :: () {
    values: [3] *void;
    values[0] = M.cfstring_from_string("one");
    values[1] = M.cfstring_from_string("two");
    values[2] = M.cfstring_from_string("three");

    array := M.CFArrayCreate(null, *values[0], 3, *M.kCFTypeArrayCallBacks);

    assert(array != null);
    print("count: %\n", M.CFArrayGetCount(array));

    second := cast(M.CFStringRef) M.CFArrayGetValueAtIndex(array, 1);

    print("second length: %\n", M.CFStringGetLength(second));
    second_text := M.to_string(second);

    print("second text: %\n", second_text == "two");
    free(second_text.data);

    print("array type: %\n", M.CFGetTypeID(array) == M.CFArrayGetTypeID());
    print("string type: %\n", M.CFGetTypeID(second) == M.CFStringGetTypeID());
    print("array is not string: %\n", M.CFGetTypeID(array) != M.CFStringGetTypeID());
    print("retain count: %\n", M.CFGetRetainCount(array));

    M.CFRelease(array);
    for i: 0..2 M.CFRelease(values[i]);
}
