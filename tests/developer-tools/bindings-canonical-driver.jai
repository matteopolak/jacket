#import "Basic";
#import "File";
#import "String";
B :: #import "Bindings_Generator";

will_print_count: int;
errors: int;
reports: int;

capture_error :: (text: string, _data: *void, info: Log_Info) {
    if info.common_flags & .ERROR errors += 1;
    if contains(text, "Bindings_Generator: emitted ") reports += 1;
}

will_print :: () {
    assert(context.generator != null && context.generator.translation_unit != null && context.generator.clang_index != null);
    assert(context.generator.counts.structs == 0 && context.generator.counts.enums == 0);
    will_print_count += 1;
}

main :: () {
    args := get_command_line_arguments();

    defer array_free(args);
    assert(args.count == 2);
    options: B.Generate_Bindings_Options;

    array_add(*options.source_files, tprint("%bindings-canonical-driver.h", #filepath));
    array_add(*options.system_include_paths, #filepath);
    array_add(*options.system_library_search_paths, args[1]);
    array_add(*options.strip_prefixes, "Drive_");
    array_add(*options.generate_enums_from_macros_with_prefixes, "FAMILY_");
    defer array_free(options.source_files);
    defer array_free(options.system_include_paths);
    defer array_free(options.system_library_search_paths);
    defer array_free(options.strip_prefixes);
    defer array_free(options.generate_enums_from_macros_with_prefixes);
    options.add_generator_command = false;
    options.log_stripped_declarations = false;
    options.will_print_bindings = will_print;
    options.header = "// caller header\n";
    options.footer = "// caller footer\n";
    builder, success := B.generate_bindings(options);

    defer free_buffers(*builder);
    assert(success && !builder.failed && will_print_count == 1);
    assert(context.generator == null);
    text := builder_to_string(*builder, do_reset = false);

    defer free(text);
    assert(contains(text, options.header) && ends_with(text, options.footer));
    assert(contains(text, "Record ::") && contains(text, "Alias ::") && contains(text, "DRIVE_CODE"));
    assert(contains(text, "#assert") && !contains(text, "#foreign_library"));
    assert(write_entire_file(tprint("%/developer-tools-driver-generated.jai", args[1]), text));
    filename := tprint("%/driver-file-overload.jai", args[1]);

    assert(B.generate_bindings(options, filename, log_results = false) && will_print_count == 2);
    file_text, read := read_entire_file(filename);

    defer free(file_text);
    assert(read && file_text == text);

    push_context {
        context.logger = capture_error;
        // The filename overload defaults to reporting; the builder defaults off.
        assert(B.generate_bindings(options, filename));
    }

    assert(reports == 1 && will_print_count == 3);
    bridge := tprint("%/driver-empty-bridge.cpp", args[1]);

    assert(B.generate_bridge(options, bridge) && will_print_count == 3);
    bridge_text, bridge_read := read_entire_file(bridge);

    defer free(bridge_text);
    assert(bridge_read && begins_with(bridge_text, options.header) && ends_with(bridge_text, options.footer));
    assert(!contains(bridge_text, "#foreign_library") && !contains(bridge_text, "#assert"));
    invalid := options;
    invalid.source_files = .{};
    failed: String_Builder;
    unexpected: bool;

    push_context {
        context.logger = capture_error;
        failed, unexpected = B.generate_bindings(invalid);
    }

    defer free_buffers(*failed);
    assert(!unexpected && will_print_count == 3 && errors >= 1);
    previous_errors := errors;

    push_context {
        context.logger = capture_error;
        assert(!B.generate_bindings(invalid, filename));
        assert(!B.generate_bridge(invalid, bridge));
    }

    retained, retained_read := read_entire_file(filename);
    retained_bridge, retained_bridge_read := read_entire_file(bridge);

    defer free(retained);
    defer free(retained_bridge);
    assert(retained_read && retained == text && retained_bridge_read && retained_bridge == bridge_text);
    assert(errors > previous_errors && will_print_count == 3);
    missing_path := tprint("%/never-created-driver-directory/result.jai", args[1]);

    previous_errors = errors;

    push_context {
        context.logger = capture_error;
        assert(!B.generate_bindings(options, missing_path, log_results = false));
        assert(!B.generate_bridge(options, missing_path));
    }

    assert(errors > previous_errors && will_print_count == 4 && reports == 1);
    assert(options.source_files.count == 1 && options.system_library_search_paths.count == 1);
    print("ok\n");
}
