#import "Basic";
#import "File";
#import "String";
B :: #import "Bindings_Generator";
#load "../../../Extensions/test-support/counting-allocator.jai";

check_comments :: () {
    tally: Tally;

    {
        push_context, defer_pop;
        push_allocator(tally_proc, *tally);
        generator: B.Generator;

        context.generator = *generator;
        builder: String_Builder;

        defer free_buffers(*builder);
        B.append_prefix_comment(*builder, .{ text = "/** @param[in,out] x changed\r\n *   continuation\r\n * @code\r\n *     f();\r\n * @endcode\r\n */", location = .{ line = 1 } }, false);
        text := builder_to_string(*builder);

        defer free(text);
        assert(text == "// - x: changed continuation\n//\n// ```c\n//     f();\n// ```\n");
        B.append_prefix_comment(*builder, .{ text = "/*! \\brief Plain summary. */", location = .{ line = 1 } }, false);
        brief := builder_to_string(*builder);

        defer free(brief);
        assert(brief == "// Plain summary.\n");
        B.append_postfix_comment(*builder, .{ text = "//!< first\n//!< second" });
        trailing := builder_to_string(*builder);

        defer free(trailing);
        assert(trailing == " // first second");
    }

    assert(tally.outstanding == 0 && tally.allocations > 0);
}

main :: () {
    check_comments();
    args := get_command_line_arguments();

    defer array_free(args);
    assert(args.count == 2 || args.count == 3);
    if args.count == 3 assert(set_working_directory(args[1]));

    options: B.Generate_Bindings_Options;

    array_add(*options.source_files, tprint("%bindings-doc-comments.h", #filepath));
    defer array_free(options.source_files);
    options.strip_flags = 0;
    options.generate_library_declarations = false;
    options.generate_compile_time_struct_checks = false;
    options.add_generator_command = false;
    result, ok := B.generate_bindings(options);

    defer free_buffers(*result);
    assert(ok);
    text := builder_to_string(*result);

    defer free(text);
    expected, read_ok := read_entire_file(tprint("%bindings-doc-comments.expected", #filepath));

    defer free(expected);
    assert(read_ok);

    if args.count == 2 {
        if text != expected print("%", text);
        assert(text == expected);
    }

    assert(write_entire_file(tprint("%/module.jai", args[1]), text));
    options.try_to_preserve_comments = false;
    quiet, quiet_ok := B.generate_bindings(options);

    defer free_buffers(*quiet);
    assert(quiet_ok);
    quiet_text := builder_to_string(*quiet);

    defer free(quiet_text);
    assert(!contains(quiet_text, "Adds values") && !contains(quiet_text, "Stored value"));
    print("ok\n");
}
