// Original reduction: generated text must retain variadic argument ordering.
#import "Basic";

Field :: struct {
    name: string;
    type_name: string;
}

type_name :: (field: Field) -> string {
    return field.type_name;
}

generate :: (field: Field) -> string {
    builder: String_Builder;
    defer free_buffers(*builder);
    print(*builder, "%0%: %;\n", "    ", field.name, type_name(field));
    return builder_to_string(*builder);
}

text :: #run generate(Field.{ "count", "s32" });
#assert(text == "    count: s32;\n");

main :: () {
    assert(generate(Field.{ "count", "s32" }) == text);
    print("ok\n");
}
