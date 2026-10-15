// Reviewed public type-emission APIs; C++ validity is checked by the official SDK.
#import "Basic";
#import "File";
B :: #import "Bindings_Generator";

cpp_alias :: (builder: *String_Builder, name: string, type: *B.CType) {
    print_to_builder(builder, "using % = ", name);
    B.append_cpp_type(builder, type, null);
    append(builder, ";\n");
    assert(!builder.failed);
}

main :: () {
    integer := B.CType.{ size = 4, alignment = 4, number_flags = ._32BIT | .SIGNED };
    real := B.CType.{ size = 8, alignment = 8, number_flags = ._64BIT | .FLOAT };
    argument := B.Declaration.{ kind = .DECLARATION, output_name = "value", type = *real };
    function: B.Function_Type;
    function.return_type = *integer;
    array_add(*function.arguments, *argument);
    defer array_free(function.arguments);
    callable := B.CType.{ type_of_function = *function };
    callback := B.CType.{ pointer_to = *callable };
    array := B.CType.{ array_element_type = *integer, array_element_count = 4 };
    array_pointer := B.CType.{ pointer_to = *array };
    array_reference := B.CType.{ pointer_to = *array, flags = .REFERENCE };
    constant := integer;
    constant.flags = .CONST | .VOLATILE;
    pointer_constant := B.CType.{ pointer_to = *integer, flags = .CONST };
    namespace := B.Namespace.{ kind = .NAMESPACE, name = "OriginalTypes", output_name = "PrintedTypes" };
    record := B.Struct.{ kind = .STRUCT, name = "NativeRecord", output_name = "RenamedRecord", parent = cast(*B.Declaration) *namespace };
    record_type := B.CType.{ type_of_struct = *record };
    cpp: String_Builder;

    defer free_buffers(*cpp);
    append(*cpp, "namespace OriginalTypes { struct NativeRecord { int value; }; }\n");
    cpp_alias(*cpp, "Printed_Callback", *callback);
    cpp_alias(*cpp, "Printed_Array", *array_pointer);
    cpp_alias(*cpp, "Printed_Reference", *array_reference);
    cpp_alias(*cpp, "Printed_ConstPointer", *pointer_constant);
    cpp_alias(*cpp, "Printed_Record", *record_type);
    append(*cpp, "namespace OriginalTypes { using Printed_Local = ");
    B.append_cpp_type(*cpp, *record_type, null, .NO_PATH);
    append(*cpp, "; }\nstatic_assert(__is_same(Printed_ConstPointer, int *const));\nstatic_assert(__is_same(Printed_Record, OriginalTypes::Printed_Local));\n");
    append(*cpp, "using Printed_Qualified = int");
    B.append_cpp_type_modifiers(*cpp, *constant, .C_TYPE_DETAILS);
    append(*cpp, ";\nstatic_assert(__is_same(Printed_Callback, int (*)(double)));\nstatic_assert(__is_same(Printed_Array, int (*)[4]));\nstatic_assert(__is_same(Printed_Reference, int (&)[4]));\nstatic_assert(__is_same(Printed_Qualified, const int));\nextern \"C\" int printed_call(Printed_Callback callback, double value) { return callback(value); }\nextern \"C\" int printed_array(int *data) { Printed_Reference array = *reinterpret_cast<Printed_Array>(data); array[2] += 7; return array[2]; }\n");
    append(*cpp, "extern \"C\" int printed_identity(int value) { return value; }\n");
    append(*cpp, "extern \"C\" int printed_bool(bool value) { return value ? 1 : 0; }\n");
    text := builder_to_string(*cpp);

    defer free(text);
    assert(write_entire_file(tprint("%../../target/developer-tools-type-emission.cpp", #filepath), text));
    jai: String_Builder;

    defer free_buffers(*jai);
    append(*jai, "Printed_Callback :: #type ");
    B.print_type_to_builder(*jai, *callback, null);
    append(*jai, ";\n");
    append(*jai, "Printed_Function :: #type ");
    // The native foreign header below is generated through the same public API.
    B.print_function_type_to_builder(*jai, *function, null);
    append(*jai, ";\nnative :: #foreign_library \"developer_tools_type_emission\";\ndefault_value :: ");
    default_literal: B.Literal;
    default_literal.literal_kind = .INTEGER;
    default_literal.int_value = 42;
    default_argument := B.Declaration.{ output_name = "value", type = *integer, expression = cast(*B.Expression) *default_literal };
    default_function := B.Function_Type.{ return_type = *integer };

    array_add(*default_function.arguments, *default_argument);
    defer array_free(default_function.arguments);
    B.print_function_type_to_builder(*jai, *default_function, null);
    append(*jai, " #foreign native \"printed_identity\"");
    append(*jai, ";\ndefault_bool :: ");
    boolean := B.CType.{ flags = .BOOL, size = 1 };

    default_argument.type = *boolean;
    default_literal.int_value = 1;
    B.print_function_type_to_builder(*jai, *default_function, null);
    append(*jai, " #foreign native \"printed_bool\"");
    append(*jai, ";\n");
    assert(!jai.failed);
    jai_text := builder_to_string(*jai);

    defer free(jai_text);
    assert(write_entire_file(tprint("%../../target/developer-tools-type-emission-generated.jai", #filepath), jai_text));
    name := B.type_to_string(*array_pointer, null);

    defer free(name);
    assert(name == "*[4] s32");
    qualified_name := B.type_to_string(*record_type, null);

    defer free(qualified_name);
    short_name := B.type_to_string(*record_type, null, .NO_PATH);

    defer free(short_name);
    assert(qualified_name == "PrintedTypes.RenamedRecord" && short_name == "RenamedRecord");
    assert(B.exceeds_signed_int_value(0x80000000, *integer));
    assert(!B.exceeds_signed_int_value(0x7fffffff, *integer));

    for bytes: s64.[1, 2, 4, 8] {
        signed_type := integer;
        signed_type.size = bytes;
        maximum := (cast(u64) 1 << (bytes * 8 - 1)) - 1;

        assert(!B.exceeds_signed_int_value(maximum, *signed_type));
        assert(B.exceeds_signed_int_value(maximum + 1, *signed_type));
        signed_type.number_flags &= ~B.Number_Flags.SIGNED;
    }

    enumeration := B.Enum.{ type = *integer };
    enum_type := B.CType.{ type_of_enum = *enumeration };
    alias := B.Typedef.{ type = *enum_type };
    aliased := B.CType.{ type_of_typedef = *alias };
    pointer := B.CType.{ pointer_to = *aliased };

    assert(B.strip_type(*pointer) == *pointer);
    assert(B.strip_type(*pointer, pointers = true) == *aliased);
    assert(B.strip_type(*pointer, pointers = true, typedefs = true) == *enum_type);
    assert(B.strip_type(*pointer, pointers = true, typedefs = true, enums = true) == *integer);
    print("ok\n");
}
