// Diagnostics preserve prior builder bytes and never fabricate a type.
#import "Basic";
B :: #import "Bindings_Generator";

errors: s64;

capture :: (message: string, data: *void, info: Log_Info) {
    if info.common_flags & .ERROR errors += 1;
}

main :: () {
    cyclic: B.CType;
    cyclic.pointer_to = *cyclic;
    cpp: String_Builder;
    jai: String_Builder;
    defaults: String_Builder;

    defer free_buffers(*cpp);
    defer free_buffers(*jai);
    defer free_buffers(*defaults);
    append(*cpp, "keep");
    append(*jai, "keep");
    append(*defaults, "keep");
    integer := B.CType.{ size = 4, number_flags = ._32BIT | .SIGNED };
    literal: B.Literal;
    literal.literal_kind = .INTEGER;
    argument := B.Declaration.{ output_name = "value", type = *integer, expression = cast(*B.Expression) *literal };
    function := B.Function_Type.{ return_type = *integer };

    array_add(*function.arguments, *argument);
    defer array_free(function.arguments);
    alias: B.Typedef;
    aliased := B.CType.{ type_of_typedef = *alias };

    alias.type = *aliased;

    push_context {
        context.logger = capture;
        B.append_cpp_type(*cpp, *cyclic, null);
        B.print_type_to_builder(*jai, *cyclic, null);
        B.print_function_type_to_builder(*defaults, *function, null);
        assert(B.strip_type(*aliased, typedefs = true) == null);
    }

    assert(errors == 4);
    assert(cpp.failed && jai.failed && defaults.failed);
    assert(builder_string_length(*cpp) == 4 && builder_string_length(*jai) == 4 && builder_string_length(*defaults) == 4);
    assert(cyclic.pointer_to == *cyclic && alias.type == *aliased);
    without_defaults: String_Builder;

    defer free_buffers(*without_defaults);
    B.print_function_type_to_builder(*without_defaults, *function, null, flags = .NO_ARGUMENT_DEFAULTS);
    assert(!without_defaults.failed);
    print("ok\n");
}
