#import "Basic";
#import "File";
B :: #import "Bindings_Generator";

S :: type_of(context._bindings_generator_session.*);

main :: () {
    args := get_command_line_arguments();

    defer array_free(args);
    assert(args.count == 2);
    generator: B.Generator;
    options: B.Generate_Bindings_Options;

    push_context {
        B.init_generator(*generator, options);
        defer B.deinit_generator(*generator);
        S.activate(*generator, *options, context._bindings_generator_session);
        outer := B.Struct.{ kind = .STRUCT, name = "OriginalContainer", output_name = "Container" };
        outer_type := B.CType.{ type_of_struct = *outer, size = 8, alignment = 4 };

        outer.type = *outer_type;
        inner := B.Struct.{ kind = .STRUCT, flags = .IS_ANON | .IS_ANON_INLINE_TYPE, parent = cast(*B.Declaration) *outer };
        inner_type := B.CType.{ type_of_struct = *inner, size = 4, alignment = 4 };

        inner.type = *inner_type;
        first := B.Declaration.{ kind = .DECLARATION, name = "first", output_name = "first", type = generator.type_def_u32, parent = cast(*B.Declaration) *inner, bit_offset_in_parent = 0, bitfield_width = 3 };
        second := B.Declaration.{ kind = .DECLARATION, name = "second", output_name = "second", type = generator.type_def_u32, parent = cast(*B.Declaration) *inner, bit_offset_in_parent = 3, bitfield_width = 5 };
        status := B.Bitfield_Status.{ parent = *inner };
        group := cast(*B.Bitfield) B.maybe_add_to_bitfield(*status, *first);

        assert(group != null && B.maybe_add_to_bitfield(*status, *second) == null);
        B.finish_bitfield(group, 0);
        assert(group.members.count == 2 && group.members[0] == *first && group.members[1] == *second);
        array_add(*inner.declarations, cast(*B.Declaration) group);
        defer array_free(inner.declarations);
        field := B.Declaration.{ kind = .DECLARATION, name = "inner", output_name = "inner", type = *inner_type, parent = cast(*B.Declaration) *outer, bit_offset_in_parent = 0 };
        tail := B.Declaration.{ kind = .DECLARATION, name = "tail", output_name = "tail", type = generator.type_def_u32, parent = cast(*B.Declaration) *outer, bit_offset_in_parent = 32 };

        array_add(*outer.declarations, cast(*B.Declaration) *inner);
        array_add(*outer.declarations, *field);
        array_add(*outer.declarations, *tail);
        defer array_free(outer.declarations);
        output: String_Builder;

        defer free_buffers(*output);
        B.print_struct(*output, *outer, null);
        assert(!output.failed);
        B.append_bitfield_helpers(*output, group, *outer);
        assert(!output.failed);
        append(*output, "native :: #foreign_library \"anonymous_bitfields\";\ninitialize :: (value: *Container) #c_call #foreign native \"original_initialize\";\nread :: (value: *Container) -> u32 #c_call #foreign native \"original_read\";\nsize :: () -> u64 #c_call #foreign native \"original_size\";\nalignment :: () -> u64 #c_call #foreign native \"original_alignment\";\n");
        text := builder_to_string(*output);

        defer free(text);
        assert(write_entire_file(tprint("%/developer-tools-anonymous-bits-generated.jai", args[1]), text));
    }

    print("ok\n");
}
