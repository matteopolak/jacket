// A constant view given for a `[$N] T` struct parameter binds `N` to the view's count.
// rules: struct.26
#import "Basic";

make_types :: () -> [] Type {
    list: [..] Type;

    array_add(*list, int);
    array_add(*list, string);
    array_add(*list, float);

    return list;
}

registry :: #run make_types();

Wrapper :: struct(types: [$SIZE] Type) {
    count :: SIZE;
}

main :: () {
    wrapper: Wrapper(registry);
    assert(wrapper.count == 3);
    assert(wrapper.count == registry.count);
    print("ok\n");
}
