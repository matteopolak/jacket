// rules: reflect.8
// Every instance of a polymorphic struct points at the same generic descriptor.
#import "Basic";

Box :: struct(Item: Type, N: int) {
    slots: [N] Item;
}

main :: () {
    a := type_info(Box(u8, 5)).polymorph_source_struct;
    b := type_info(Box(s64, 2)).polymorph_source_struct;
    assert(a != null && a == b, "instances share polymorph_source_struct");
    print("ok\n");
}
