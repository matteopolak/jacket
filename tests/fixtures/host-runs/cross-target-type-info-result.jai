// A `#run` result that holds type descriptors points at the program's own descriptors, also
// when the build is a cross build.
#import "Basic";

Holder :: struct {
    boxed: Any;
    info: *Type_Info;
    values: [..] int;
}

make :: () -> Holder {
    h: Holder;
    h.boxed = 42;
    h.info = type_info(float32);
    array_add(*h.values, 3);

    return h;
}

HOLDER :: #run make();

main :: () {
    assert(HOLDER.boxed.type == type_info(int));
    assert(HOLDER.info == type_info(float32));
    assert(HOLDER.info.type == .FLOAT);
    assert(HOLDER.values[0] == 3);
    print("ok\n");
}
