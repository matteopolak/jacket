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
        outer: B.Struct;
        outer.kind = .STRUCT;
        outer.name = "OriginalContainer";
        outer.output_name = "Container";
        outer_type := B.CType.{ type_of_struct = *outer, size = 8, alignment = 4 };

        outer.type = *outer_type;
        inline_record: B.Struct;
        inline_record.kind = .STRUCT;
        inline_record.flags = .IS_ANON | .IS_ANON_INLINE_TYPE;
        inline_record.parent = cast(*B.Declaration) *outer;
        inline_type := B.CType.{ type_of_struct = *inline_record, size = 4, alignment = 4 };

        inline_record.type = *inline_type;
        payload := B.Declaration.{ kind = .DECLARATION, output_name = "payload", name = "payload", type = generator.type_def_s32, parent = cast(*B.Declaration) *inline_record, bit_offset_in_parent = 0 };

        array_add(*inline_record.declarations, *payload);
        defer array_free(inline_record.declarations);
        inner := B.Declaration.{ kind = .DECLARATION, output_name = "inner", name = "inner", type = *inline_type, parent = cast(*B.Declaration) *outer, bit_offset_in_parent = 0 };
        tail := B.Declaration.{ kind = .DECLARATION, output_name = "tail", name = "tail", type = generator.type_def_s32, parent = cast(*B.Declaration) *outer, bit_offset_in_parent = 32 };

        array_add(*outer.declarations, cast(*B.Declaration) *inline_record);
        array_add(*outer.declarations, *inner);
        array_add(*outer.declarations, *tail);
        defer array_free(outer.declarations);
        output: String_Builder;

        defer free_buffers(*output);
        B.print_struct(*output, *outer, null);
        assert(!output.failed);
        text := builder_to_string(*output);

        defer free(text);
        assert(write_entire_file(tprint("%/developer-tools-inline-generated.jai", args[1]), text));
    }

    print("ok\n");
}
