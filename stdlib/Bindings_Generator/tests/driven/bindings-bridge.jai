#import "Basic";
#import "File";
B :: #import "Bindings_Generator";

errors: s64;

capture :: (_text: string, _data: *void, info: Log_Info) {
    if info.common_flags & .ERROR errors += 1;
}

main :: () {
    options: B.Generate_Bindings_Options;

    array_add(*options.source_files, tprint("%bindings-bridge.h", #filepath));
    defer array_free(options.source_files);
    array_add(*options.libraries, B.Library.{ filename = "developer_tools_bridge", identifier = "bridge_native" });
    defer array_free(options.libraries);
    array_add(*options.library_search_paths, tprint("%../../../../target", #filepath));
    defer array_free(options.library_search_paths);
    array_add(*options.extra_clang_arguments, "-x", "c++", "-std=c++17");
    defer array_free(options.extra_clang_arguments);
    options.header = tprint("#include \"%bindings-bridge.h\"\n", #filepath);
    options.add_generator_command = false;
    path := tprint("%../../../../target/developer-tools-bridge.cpp", #filepath);

    push_context {
        context.logger = capture;
        assert(B.generate_bridge(options, path));
    }

    source, ok := read_entire_file(path);

    assert(ok && contains(source, "extern \"C\"") && contains(source, "original_bridge_pair"));
    assert(!contains(source, "original_bridge_deprecated") && !contains(source, "unsupported_method"));
    defer free(source);
    path = tprint("%../../../../target/developer-tools-bridge-overloads.cpp", #filepath);
    options.source_files[0] = tprint("%bindings-bridge-overload.h", #filepath);

    push_context {
        context.logger = capture;
        assert(B.generate_bridge(options, path));
    }

    overloads, overloads_ok := read_entire_file(path);

    defer free(overloads);
    assert(overloads_ok && errors == 0);
    // Overloads that share a bridge name are repaired into distinct exported adapters.
    assert(contains(overloads, "original_bridge_collision(") && contains(overloads, "original_bridge_collision_1("));
    print("ok\n");
}

#scope_file
#import "String";
