#import "Basic";
#import "File";
#import "String";
B :: #import "Bindings_Generator";

callbacks: s32;
errors: s32;

before_print :: () {
    callbacks += 1;
}

capture_error :: (text: string, _data: *void, info: Log_Info) {
    assert(text.count > 0);
    if info.common_flags & .ERROR errors += 1;
}

main :: () {
    options: B.Generate_Bindings_Options;

    assert(options.os == OS && options.cpu == CPU);
    assert(options.strip_enum_prefixes && options.alias_original_enum_names);
    assert(options.generate_compile_time_struct_checks && options.generate_bitfield_helpers);
    assert(options.get_func_args_for_printing != null);
    declaration: B.Declaration;

    assert(declaration.macro_value_result == .NEEDS_CHECKING);
    declaration.macro_value_result = .KEEP;
    assert(cast(s64) declaration.macro_value_result == 1);
    array_add(*options.source_files, tprint("%bindings-input.h", #filepath));
    array_add(*options.libraries, B.Library.{ filename = "developer_tools_bindings", identifier = "native" });
    options.will_print_bindings = before_print;
    builder, ok := B.generate_bindings(options);

    defer free_buffers(*builder);
    assert(ok);
    text := builder_to_string(*builder);

    defer free(text);
    assert(contains(text, "sample_apply ::"));
    assert(contains(text, "Original SDK-style record documentation."));
    assert(contains(text, "ADD :: 3"));
    assert(contains(text, "MODE_ADD :: ADD"));
    assert(B.generate_bindings(options, tprint("%../../target/developer-tools-public-generated.jai", #filepath), log_results = false));
    assert(callbacks == 2);
    path := tprint("%../../target/developer-tools-public-reject.jai", #filepath);

    assert(write_entire_file(path, "preserve existing bindings"));
    options.source_files[0] = tprint("%bindings-malformed.h", #filepath);

    push_context {
        context.logger = capture_error;
        assert(!B.generate_bindings(options, path, log_results = false));
    }

    assert(errors > 0 && callbacks == 2);
    preserved, preserved_ok := read_entire_file(path);

    defer free(preserved);
    assert(preserved_ok && preserved == "preserve existing bindings");
    print("ok\n");
}
