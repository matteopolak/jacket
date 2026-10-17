#import "Basic";
#import "File";
B :: #import "Bindings_Generator";

visits: s64;
foreign_prints: s64;
inner_prints: s64;
outer_prints: s64;
ordinary_functions: s64;
reversed: [2] *B.Declaration;
invalid_names: bool;
errors: s64;

visit :: (declaration: *B.Declaration, parent: *B.Declaration) -> B.Declaration_Visit_Result {
    if declaration.kind == .STRUCT declaration.output_name = "Custom_Record";
    if parent && parent.kind == .STRUCT declaration.output_name = "number";

    if declaration.kind == .FUNCTION {
        if declaration.name == "original_cpp_custom" {
            arguments := declaration.type.type_of_function.arguments;
            floating := (arguments[0].type.number_flags & .FLOAT) != 0;
            declaration.output_name = ifx invalid_names then "invalid-custom-name" else ifx floating then "custom_floating" else "custom_integer";
        } else if declaration.name == "original_cpp_record" declaration.output_name = "make_custom_record";
        else if declaration.name == "original_cpp_reference" declaration.output_name = "reference_custom_record";
        else if declaration.name == "original_cpp_omitted" declaration.decl_flags |= .OMIT_FROM_OUTPUT;

        if !(declaration.decl_flags & .OMIT_FROM_OUTPUT) {
            assert(!(declaration.decl_flags & (.NEEDS_ARGUMENT_WRAPPER | .NEEDS_PRINTF_WRAPPER)));
            ordinary_functions += 1;
        }
    }

    if parent && parent.kind == .FUNCTION {
        if declaration.name == "left" declaration.output_name = "a";
        if declaration.name == "right" declaration.output_name = "b";
    }

    return .RECURSE;
}

visit_data :: (_declaration: *B.Declaration, _parent: *B.Declaration, data: *void) -> B.Declaration_Visit_Result {
    (cast(*s64) data).* += 1;
    return .RECURSE;
}

arguments :: (_function: *B.Function, type: *B.Function_Type, mode: B.Arg_Print_Mode) -> [] *B.Declaration {
    if mode == .FOREIGN_FUNC_DECLARATION {
        foreign_prints += 1;

        if type.arguments.count == 2 {
            reversed[0] = type.arguments[1];
            reversed[1] = type.arguments[0];
            return reversed;
        }
    } else if mode == .INNER_WRAPPER_CALL inner_prints += 1;
    else if mode == .OUTER_WRAPPER outer_prints += 1;

    return type.arguments;
}

capture :: (_text: string, _data: *void, info: Log_Info) {
    if info.common_flags & .ERROR errors += 1;
}

callback_visit :: (declaration: *B.Declaration, parent: *B.Declaration) -> B.Declaration_Visit_Result {
    if declaration.kind == .FUNCTION declaration.decl_flags |= .NEEDS_ARGUMENT_WRAPPER;
    if parent && parent.kind == .FUNCTION declaration.decl_flags |= .POINTER_TO_VALUE;
    return .RECURSE;
}

generate_callback_control :: () {
    options: B.Generate_Bindings_Options;

    array_add(*options.source_files, tprint("%../../../../target/developer-tools-cpp-callback.h", #filepath));
    defer array_free(options.source_files);
    array_add(*options.libraries, B.Library.{ filename = "developer_tools_cpp_callback", identifier = "native" });
    defer array_free(options.libraries);
    array_add(*options.library_search_paths, tprint("%../../../../target", #filepath));
    defer array_free(options.library_search_paths);
    options.add_generator_command = false;
    options.visitor = callback_visit;
    options.get_func_args_for_printing = arguments;
    foreign_before := foreign_prints;
    outer_before := outer_prints;
    inner_before := inner_prints;

    assert(B.generate_bindings(options, tprint("%../../../../target/developer-tools-cpp-callback-generated.jai", #filepath), log_results = false));
    // One real pointer-to-value wrapper needs each of the three argument views.
    assert(foreign_prints == foreign_before + 1);
    assert(outer_prints == outer_before + 1 && inner_prints == inner_before + 1);
}

main :: () {
    options: B.Generate_Bindings_Options;

    array_add(*options.source_files, tprint("%bindings-cpp-custom.h", #filepath));
    defer array_free(options.source_files);
    array_add(*options.libraries, B.Library.{ filename = "developer_tools_cpp_custom", identifier = "native" });
    defer array_free(options.libraries);
    array_add(*options.library_search_paths, tprint("%../../../../target", #filepath));
    defer array_free(options.library_search_paths);
    array_add(*options.extra_clang_arguments, "-x", "c++", "-std=c++17");
    defer array_free(options.extra_clang_arguments);
    options.add_generator_command = false;
    options.visitor = visit;
    options.visitor_with_data = visit_data;
    options.visitor_data = *visits;
    options.get_func_args_for_printing = arguments;
    assert(B.generate_bindings(options, tprint("%../../../../target/developer-tools-cpp-custom-generated.jai", #filepath), log_results = false));
    // Four visible non-wrapper functions, including both overloads; the
    // omitted declaration and repeated integer declaration add no callbacks.
    assert(visits > 0 && ordinary_functions == 4);
    assert(foreign_prints == ordinary_functions && outer_prints == 0 && inner_prints == 0);
    foreign_before := foreign_prints;
    outer_before := outer_prints;
    inner_before := inner_prints;

    options.header = tprint("#include \"%bindings-cpp-custom.h\"\n", #filepath);
    assert(B.generate_bridge(options, tprint("%../../../../target/developer-tools-cpp-custom.cpp", #filepath)));
    // C adapters consume represented arguments, independently of Jai printing.
    assert(foreign_prints == foreign_before && outer_prints == outer_before && inner_prints == inner_before);
    // A separate schema describes the actual C adapter ABI. Ordinary bindings
    // retain C++ symbols and the original aggregate-by-value return contract.
    bridge_schema := #string END
Custom_Record :: struct #align 4 {
    number: s32;
}
custom_integer :: (a: s32, b: s32) -> s32 #c_call #foreign native;
custom_floating :: (a: float64, b: float64) -> float64 #c_call #foreign native;
make_custom_record :: (result: *Custom_Record, value: s32) #c_call #foreign native;
reference_custom_record :: (value: *Custom_Record) -> *Custom_Record #c_call #foreign native;
#scope_file
native :: #library "developer_tools_cpp_custom";
#assert size_of(Custom_Record) == 4;
#assert align_of(Custom_Record) == 4;
#assert offset_of(Custom_Record, "number") == 0;
END
    assert(write_entire_file(tprint("%../../../../target/developer-tools-cpp-custom-bridge.jai", #filepath), bridge_schema));
    path := tprint("%../../../../target/developer-tools-cpp-custom-rejected.jai", #filepath);

    assert(write_entire_file(path, "original custom destination\n"));
    invalid_names = true;

    push_context {
        context.logger = capture;
        assert(!B.generate_bindings(options, path, log_results = false));
        assert(!B.generate_bridge(options, path));
    }

    preserved, ok := read_entire_file(path);

    defer free(preserved);
    assert(ok && preserved == "original custom destination\n" && errors >= 2);
    generate_callback_control();
    print("ok\n");
}
