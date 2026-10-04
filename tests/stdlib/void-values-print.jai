#import "Basic";

main :: () {
    varv: void;
    ptr: *void;
    s := tprint("% %", varv, << ptr);
    assert(s == "void void");
    print("ok\n");
}
