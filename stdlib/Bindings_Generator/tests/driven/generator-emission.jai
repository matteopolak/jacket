#import "Basic";
B :: #import "Bindings_Generator";
Other :: #import "Bindings_Generator"(TARGET_OS = .LINUX);

extract :: (builder: *String_Builder) -> string {
    return builder_to_string(builder);
}

decl_from_field :: (field: *B.Declaration) -> *B.Declaration {
    return field;
}

main :: () {
    comment: B.Comment;
    comment.text = "// old";
    comment.location.line = 17;
    B.prepend_comment(*comment, "// first");
    assert(comment.text == "// first // old" && comment.location.line == 17);
    allocation := comment.text;

    B.prepend_comment(*comment, "");
    assert(comment.text.data == allocation.data);
    free(allocation);
    standalone: B.Comment;
    supplied := "// borrowed standalone";

    B.prepend_comment(*standalone, supplied);
    assert(standalone.text.data == supplied.data);

    generator: B.Generator;
    generator.tab_depth = 2;
    builder: String_Builder;

    defer free_buffers(*builder);

    push_context {
        context.generator = *generator;
        B.append_indentation(*builder);
        text := extract(*builder);

        assert(text == "  ");
        free(text);
        B.append_prefix_comment(*builder, .{ text = "// alpha\n// beta" }, true);
        text = extract(*builder);
        assert(text == "// alpha\n  // beta\n  ");
        free(text);
    }

    other: Other.Generator;
    other.tab_depth = 3;

    push_context {
        context.generator = *other;
        Other.append_indentation(*builder);
        text := extract(*builder);

        assert(text == "   ");
        free(text);
        other.tab_depth = 0;
        Other.append_indentation(*builder);
        text = extract(*builder);
        assert(text == "");
        free(text);
    }

    B.append_postfix_comment(*builder, .{ text = "///< value" });
    text := extract(*builder);

    assert(text == " ///< value");
    free(text);
    B.append_prefix_comment(*builder, .{ text = " \t// trimmed\r\n \t// next\t ", location = .{ line = 2 } }, false);
    text = extract(*builder);
    assert(text == "// trimmed\n// next\n");
    free(text);
    B.append_postfix_comment(*builder, .{ text = "// one\r// two" });
    text = extract(*builder);
    assert(text == " // one\n// two");
    free(text);
    B.append_prefix_comment(*builder, .{ text = "///< value" }, false);
    B.append_postfix_comment(*builder, .{ text = "// prefix", location = .{ line = 1 } });
    text = extract(*builder);
    assert(text == "");
    free(text);

    first, omitted, renamed: B.Declaration;
    first.name = "long";
    first.kind = .DECLARATION;
    first.output_name = "long";
    omitted.name = "excluded_longest";
    omitted.decl_flags = .OMIT_FROM_OUTPUT;
    omitted.kind = .FUNCTION;
    renamed.name = "source_name";
    renamed.output_name = "x";
    renamed.kind = .TYPEDEF;
    renamed.decl_flags = .QUALIFIER_USING;
    fields: [] *B.Declaration = .[*first, *omitted, *renamed];

    assert(B.get_name_alignment(fields) == 7);
    assert(B.get_name_alignment(fields, decl_from_field) == 7);
    assert(B.get_name_alignment(fields, decl_from_field, true) == 4);
    first.storage_class = .Extern;
    renamed.decl_flags = .QUALIFIER_AS;
    assert(B.get_name_alignment(fields) == 5);
    renamed.output_name = "";
    assert(B.get_name_alignment(fields, decl_from_field, true) == 0);
    renamed.kind = .BITFIELD;
    renamed.output_name = "bitfield";
    renamed.decl_flags = .OMIT_FROM_OUTPUT;
    assert(B.get_name_alignment(fields) == 8);
    renamed.kind = .MACRO_DEFINITION;
    assert(B.get_name_alignment(fields) == 8);
    renamed.kind = .STRUCT;
    assert(B.get_name_alignment(fields) == 0);
    empty: [] *B.Declaration;

    assert(B.get_name_alignment(empty) == 0);

    options: B.Generate_Bindings_Options;
    options.mimic_spacing_flags = .STRUCT;

    push_context {
        context._bindings_generator_options = *options;
        last: B.Location;
        here: B.Location = .{ file = cast(B.CXFile) 1, line = 1 };

        B.maybe_add_spacing(*builder, *last, here, 0);
        text = extract(*builder);
        assert(text == "\n" && last.file == here.file && last.line == 1);
        free(text);
        here.line = 4;
        B.maybe_add_spacing(*builder, *last, here, 0, B.Mimic_Spacing_Flags.STRUCT);
        text = extract(*builder);
        assert(text == "\n\n" && last.line == 4);
        free(text);
        here.line = 1;
        B.maybe_add_spacing(*builder, *last, here, .NO_SPACING, B.Mimic_Spacing_Flags.ENUM);
        text = extract(*builder);
        assert(text == "" && last.line == 1);
        free(text);
        here.file = cast(B.CXFile) 2;
        B.maybe_add_spacing(*builder, *last, here, .NO_SPACING, true);
        text = extract(*builder);
        assert(text == "\n" && last.file == here.file);
        free(text);
        here.line = 2;
        B.maybe_add_spacing(*builder, *last, here, .NO_SPACING);
        B.maybe_add_spacing(*builder, *last, here, .NO_SPACING);
        text = extract(*builder);
        assert(text == "" && last.line == 2);
        free(text);
        last.line = 0xffffffff;
        here.line = 0;
        B.maybe_add_spacing(*builder, *last, here, .NO_SPACING, true);
        text = extract(*builder);
        assert(text == "\n" && last.line == 0);
        free(text);
        other_options: Other.Generate_Bindings_Options;
        other_options.mimic_spacing_flags = .ENUM;

        push_context {
            context._bindings_generator_options = *other_options;
            other_last: Other.Location = .{ file = cast(Other.CXFile) 1, line = 1 };

            Other.maybe_add_spacing(*builder, *other_last, .{ file = cast(Other.CXFile) 1, line = 4 }, 0, Other.Mimic_Spacing_Flags.ENUM);
            text = extract(*builder);
            assert(text == "\n\n");
            free(text);
        }

        assert(context._bindings_generator_options == *options);
    }

    print("ok\n");
}
