#import "Basic";
B :: #import "Bindings_Generator";

main :: () {
    generator: B.Generator;

    push_context {
        B.init_generator(*generator, .{});
        defer B.deinit_generator(*generator);
        existing, other: B.Declaration;
        existing.output_name = "value";
        other.output_name = "value";
        declarations: [..] *B.Declaration;

        defer array_free(declarations);
        array_add(*declarations, *existing);
        B.avoid_name_clash(*declarations, *other, insert = false);
        assert(other.output_name == "value_1" && declarations.count == 1);
        B.avoid_name_clash(*declarations, *other);
        B.avoid_name_clash(*declarations, *other);
        assert(declarations.count == 2);
        reserved: B.Declaration;
        reserved.output_name = "cast";
        B.avoid_name_clash(null, *reserved);
        assert(reserved.output_name == "_cast");
        discard: B.Declaration;
        discard.output_name = "_";
        B.avoid_name_clash(*declarations, *discard);
        assert(declarations.count == 2 && discard.output_name == "_");
        parent: B.Namespace;
        parent.output_name = "Scope";
        prefixed: B.Declaration;
        prefixed.output_name = "value";
        prefixed.parent = cast(*B.Declaration) *parent;
        B.avoid_name_clash(*declarations, *prefixed, try_parent_prefix = true);
        assert(prefixed.output_name == "Scope_value");
        record: B.Struct;
        record.kind = .STRUCT;
        record.output_name = "Handle";
        array_add(*generator.global_scope.members, cast(*B.Declaration) *record);
        defer array_free(generator.global_scope.members);
        function: B.Function;
        function.kind = .FUNCTION;
        function.output_name = "Handle";
        B.avoid_name_clash(null, cast(*B.Declaration) *function);
        assert(function.output_name == "Handle_");
        first, second, repeated: B.Function;
        first.kind = .FUNCTION;
        first.name = "Choose";
        first.output_name = "choose";
        second = first;
        repeated = first;
        first_type, second_type, repeated_type: B.Function_Type;
        first_type.return_type = generator.type_def_s32;
        second_type.return_type = generator.type_def_s32;
        repeated_type.return_type = generator.type_def_s32;
        first_arg, second_arg: B.Declaration;
        first_arg.type = generator.type_def_s32;
        second_arg.type = generator.type_def_float64;
        array_add(*first_type.arguments, *first_arg);
        array_add(*second_type.arguments, *second_arg);
        array_add(*repeated_type.arguments, *first_arg);
        defer array_free(first_type.arguments);
        defer array_free(second_type.arguments);
        defer array_free(repeated_type.arguments);
        first_descriptor, second_descriptor, repeated_descriptor: B.CType;
        first_descriptor.type_of_function = *first_type;
        second_descriptor.type_of_function = *second_type;
        repeated_descriptor.type_of_function = *repeated_type;
        first.type = *first_descriptor;
        second.type = *second_descriptor;
        repeated.type = *repeated_descriptor;
        B.avoid_name_clash(*declarations, cast(*B.Declaration) *first);
        B.avoid_name_clash(*declarations, cast(*B.Declaration) *second);
        assert(first.output_name == "choose" && second.output_name == "choose");
        assert((first_type.flags & .ADD_C_TYPE_DETAILS) != 0 && (second_type.flags & .ADD_C_TYPE_DETAILS) != 0);
        B.avoid_name_clash(*declarations, cast(*B.Declaration) *repeated);
        assert(repeated.output_name == "choose_1");
    }

    print("ok\n");
}
