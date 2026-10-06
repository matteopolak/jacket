#import "Basic";

// A `#cpp_method` procedure type is C-ABI like `#c_call`: calls through it pass no context pointer,
// so the first argument the callee sees is `this`.
Obj :: struct {
    base: s32;
}
Vtable :: struct {
    add: #type (this: *Obj, x: s32) -> s32 #cpp_method;
}

add_impl :: (this: *Obj, x: s32) -> s32 #c_call {
    return this.base + x;
}

take_ptr :: (p: *u8) -> u8 {
    return p[0];
}

main :: () {
    o := Obj.{ base = 40 };
    vt := Vtable.{ add = add_impl };
    assert(vt.add(*o, 2) == 42);

    // A fixed array decays to a pointer to its first element (preferred over boxing into Any), so
    // `to_string` on a C `char[N]` field reads the NUL-terminated text.
    name: [16] u8;
    name[0] = #char "h";
    name[1] = #char "i";
    assert(take_ptr(name) == #char "h");
    s := to_string(name);
    assert(s == "hi");
}
