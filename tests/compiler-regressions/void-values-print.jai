#import "Basic";

main :: () {
    varv: void;
    second: void;
    ptr: *void;
    // A void value is printable; dereferencing a null pointer is not the
    // construction of a second value and has no such formatting guarantee.
    s := tprint("% %", varv, second);

    assert(s == "void void");
    assert(tprint("%", ptr) == "null");
    print("ok\n");
}
