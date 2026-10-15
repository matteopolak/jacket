// Reviewed public model/type queries, with safe malformed-model policies.
#import "Basic";
B :: #import "Bindings_Generator";

errors: int;

capture :: (message: string, data: *void, info: Log_Info) {
    if info.common_flags & .ERROR errors += 1;
}

main :: () {
    integer := B.CType.{ size = 4, alignment = 4, number_flags = ._32BIT | .SIGNED };

    assert(B.size_from_number_flags(integer.number_flags) == 4);
    assert(B.size_from_number_flags(._8BIT | .CHAR) == 1);
    assert(B.size_from_number_flags(._16BIT | .SIGNED) == 2);
    assert(B.size_from_number_flags(._64BIT | .FLOAT) == 8);
    assert(!B.type_has_name(*integer));
    assert(B.get_type_name(*integer) == "");

    record := B.Struct.{ kind = .STRUCT, name = "Original", output_name = "Printed" };
    record_type := B.CType.{ type_of_struct = *record };
    pointer := B.CType.{ pointer_to = *record_type };
    alias := B.Typedef.{ kind = .TYPEDEF, name = "OriginalAlias", output_name = "PrintedAlias", type = *pointer };
    alias_type := B.CType.{ type_of_typedef = *alias };
    array := B.CType.{ array_element_type = *alias_type, array_element_count = 2 };

    assert(B.type_has_name(*record_type));
    assert(B.type_has_name(*alias_type));
    assert(!B.type_has_name(*pointer));
    assert(B.get_type_name(*record_type) == "Printed");
    assert(B.get_type_name(*record_type, output_name = false) == "Original");
    assert(B.get_type_name(*alias_type) == "PrintedAlias");
    assert(B.get_type_name(*pointer) == "");
    assert(B.get_type_name(*pointer, strip_pointers = true) == "Printed");
    assert(B.try_find_struct_in_type(*array) == *record);
    assert(B.try_find_struct_in_type(*integer) == null);
    assert(B.try_find_struct_in_type(null) == null);
    assert(B.get_type_name(null) == "");

    hardcoded := B.CType.{ hardcoded_jai_string = "CustomScalar" };

    assert(B.type_has_name(*hardcoded));
    assert(B.get_type_name(*hardcoded) == "CustomScalar");
    unnamed := B.Struct.{ kind = .STRUCT };
    unnamed_type := B.CType.{ type_of_struct = *unnamed };

    assert(!B.type_has_name(*unnamed_type));

    cyclic: B.CType;
    cyclic.pointer_to = *cyclic;

    push_context {
        context.logger = capture;
        assert(B.get_type_name(*cyclic, strip_pointers = true) == "");
        assert(B.try_find_struct_in_type(*cyclic) == null);
    }

    assert(errors == 2);
    assert(cyclic.pointer_to == *cyclic);

    builder: String_Builder;

    defer free_buffers(*builder);
    B.print_type_to_builder(*builder, *record_type, null, flags = .NO_PATH);
    assert(!builder.failed);
    rendered := builder_to_string(*builder);

    defer free(rendered);
    assert(rendered == B.get_type_name(*record_type));

    // The query returns borrowed public storage, rather than synthesizing a name.
    assert(B.get_type_name(*record_type).data == record.output_name.data);
    print("ok\n");
}
