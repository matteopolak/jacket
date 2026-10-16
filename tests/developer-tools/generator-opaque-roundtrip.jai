#import "Basic";
G :: #import "generator-opaque-generated";

main :: () {
    pointer := G.opaque_pointer();

    assert(pointer != null && G.opaque_identity(pointer) == pointer);
    assert(G.opaque_value(pointer) == 73);
    union_pointer := G.opaque_union_pointer();

    assert(union_pointer != null && G.opaque_union_value(union_pointer) == 91);
    assert(G.opaque_native_by_value_control() == 73);
    forward: G.Forward;
    forward.value = 29;
    assert(G.forward_value(*forward) == 29);
    assert(size_of(G.Forward) == 4 && align_of(G.Forward) == 4);
    print("ok\n");
}
