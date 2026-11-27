// Integers cast to pointers at compile time (`SIG_ERR`, registry roots) keep their values in
// the program, whatever its pointer width.
Handler :: #type (signal: s32) #c_call;

FAILED :: #run cast(Handler) cast(s64) -1;

handlers: [1] Handler = #run make_handlers();
roots: [2] *void = #run make_roots();

make_handlers :: () -> [1] Handler {
    out: [1] Handler;
    out[0] = cast(Handler) cast(s64) -2;
    return out;
}

// Past a 32-bit program's compile-time memory, and the top of its address space.
make_roots :: () -> [2] *void {
    out: [2] *void;
    out[0] = cast(*void) cast(u32) 0xC000_1000;
    out[1] = cast(*void) cast(s64) -16;
    return out;
}

main :: () {
    failed := cast(Handler) cast(s64) -1;
    other := cast(Handler) cast(s64) -2;
    past := cast(*void) cast(u32) 0xC000_1000;
    top := cast(*void) cast(s64) -16;

    print("% % % %\n", FAILED == failed, handlers[0] == other, roots[0] == past, roots[1] == top);
}

#import "Basic";
