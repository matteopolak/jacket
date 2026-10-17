#import "Basic";
#import "File";
#import "String";
B :: #import "Bindings_Generator";

callbacks: int;

Allocation_Count :: struct {
    backing: Allocator;
    live: int;
}

count_allocations :: (mode: Allocator_Mode, requested_size: s64, old_size: s64, old_memory: *void, data: *void) -> *void {
    count := cast(*Allocation_Count) data;
    result := count.backing.proc(mode, requested_size, old_size, old_memory, count.backing.data);

    if mode == .ALLOCATE && result count.live += 1;
    else if mode == .RESIZE && !old_memory && result count.live += 1;
    else if mode == .FREE && old_memory count.live -= 1;

    assert(count.live >= 0);

    return result;
}

will_print :: () {
    callbacks += 1;
}

forbidden_args :: (_function: *B.Function, _type: *B.Function_Type, _mode: B.Arg_Print_Mode) -> [] *B.Declaration {
    assert(false, "bridge cannot use Jai argument callback");
    return .[];
}

main :: () {
    args := get_command_line_arguments();

    defer array_free(args);
    assert(args.count == 3);
    options: B.Generate_Bindings_Options;
    source := tprint("%bindings-canonical-bridge.hpp", #filepath);

    array_add(*options.source_files, source);
    array_add(*options.system_include_paths, #filepath);
    array_add(*options.system_library_search_paths, args[1]);
    array_add(*options.library_search_paths, args[1]);
    array_add(*options.libraries, B.Library.{ "libdriver_cpp", "original" });
    array_add(*options.extra_clang_arguments, "-x", "c++", "-std=c++17", "-resource-dir", args[2]);
    defer array_free(options.source_files);
    defer array_free(options.system_include_paths);
    defer array_free(options.system_library_search_paths);
    defer array_free(options.library_search_paths);
    defer array_free(options.libraries);
    defer array_free(options.extra_clang_arguments);
    options.log_stripped_declarations = false;
    options.header = tprint("#include \"%\"\n", source);
    options.footer = "// caller bridge footer\n";
    options.will_print_bindings = will_print;
    options.get_func_args_for_printing = forbidden_args;
    destination := tprint("%/developer-tools-canonical-bridge.cpp", args[1]);

    allocations: Allocation_Count;
    allocations.backing = context.allocator;

    push_context {
        context.allocator = .{ count_allocations, *allocations };
        assert(B.generate_bridge(options, destination));
    }

    assert(allocations.live == 0);
    assert(callbacks == 0 && context.generator == null);
    text, read := read_entire_file(destination);

    defer free(text);
    assert(read && begins_with(text, options.header) && ends_with(text, options.footer));
    assert(contains(text, "DriverCPP_sum") && contains(text, "DriverCPP_make_pair"));
    assert(!contains(text, "original_c_value"));
    assert(!contains(text, "#library") && !contains(text, "#assert"));
    print("ok\n");
}
