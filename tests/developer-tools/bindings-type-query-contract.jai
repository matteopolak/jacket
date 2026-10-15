// Reviewed Shared emission and parser helper contracts: narrow type queries.
#import "Basic";
B :: #import "Bindings_Generator";

main :: () {
    integer: B.CType;
    integer.size = 4;
    integer.number_flags = ._32BIT | .SIGNED;
    assert(B.type_has_name(*integer));
    assert(B.size_from_number_flags(0) == 0);
    assert(B.size_from_number_flags(.CHAR) == 0);
    assert(B.size_from_number_flags(._64BIT | ._16BIT | ._8BIT) == 1);
    assert(B.size_from_number_flags(._64BIT | ._32BIT) == 4);
    hardcoded: B.CType;
    hardcoded.hardcoded_jai_string = "s32";
    assert(B.type_has_name(*hardcoded));
    assert(B.get_type_name(*hardcoded) == "");
    record: B.Struct;
    record.kind = .STRUCT;
    record.output_name = "SyntheticOutput";
    nominal: B.CType;
    nominal.type_of_struct = *record;
    assert(!B.type_has_name(*nominal));
    record.name = "Original";
    assert(B.type_has_name(*nominal));
    assert(B.get_type_name(*nominal).data == record.output_name.data);
    pointer: B.CType;
    pointer.pointer_to = *nominal;
    array: B.CType;
    array.array_element_type = *pointer;
    array.array_element_count = 2;
    assert(B.type_has_name(*pointer));
    assert(B.try_find_struct_in_type(*array) == *record);
    alias: B.Typedef;
    alias.type = *array;
    alias_type: B.CType;
    alias_type.type_of_typedef = *alias;
    assert(B.try_find_struct_in_type(*alias_type) == null);

    builder: String_Builder;

    defer free_buffers(*builder);
    integer.flags = .CONST | .VOLATILE;
    B.append_cpp_type_modifiers(*builder, *integer, 0);
    assert(builder_string_length(*builder) == 0);
    B.append_cpp_type_modifiers(*builder, *integer, .C_TYPE_DETAILS);
    text := builder_to_string(*builder);

    defer free(text);
    assert(text == " const");
    integer.flags = .REFERENCE | .CONST | .VOLATILE;
    B.append_cpp_type_modifiers(*builder, *integer, 0);
    reference := builder_to_string(*builder);

    defer free(reference);
    assert(reference == " &");
    print("ok\n");
}
