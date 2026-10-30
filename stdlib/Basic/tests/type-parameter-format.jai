#import "Basic";

Box :: struct(T: Type = int, amount: int = 3) {
    value: T;
}

Required :: struct(T: Type, amount: int = 3) {
    value: T;
}

Label :: struct(text: string = "same") {
    value: int;
}

Dependent :: struct(first: int = 3, second: int = first + 1) {
    value: int;
}

describe :: (info: *Type_Info, all := false) -> string {
    builder: String_Builder;
    defer free_buffers(*builder);
    print_type_to_builder(*builder, info, all);
    return builder_to_string(*builder);
}

main :: () {
    first := describe(type_info(Box(int, 3)));

    defer free(first);
    assert(first == "Box()");
    all := describe(type_info(Box(int, 3)), true);

    defer free(all);
    assert(all == "Box(T=s64, amount=3)");
    required := describe(type_info(Required(int, 3)));

    defer free(required);
    assert(required == "Required(T=s64)");
    changed := describe(type_info(Box(string, 7)));

    defer free(changed);
    assert(changed == "Box(T=string, amount=7)");
    label := describe(type_info(Label("same")));

    defer free(label);
    assert(label == "Label()");
    dependent := describe(type_info(Dependent(7, 8)));

    defer free(dependent);
    assert(dependent == "Dependent(first=7)");
    print("ok\n");
}
