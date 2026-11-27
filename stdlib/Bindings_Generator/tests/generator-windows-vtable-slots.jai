// test: only(.LINUX,.MACOS)
// A Windows virtual slot that returns a record keeps its natural signature: `#cpp_method` passes the result
// pointer after `this`, so the slot declares neither a hidden parameter nor a void result.
#import "Basic";
#import "String";
W :: #import "Bindings_Generator"(TARGET_OS = .WINDOWS);

main :: () {
    options: W.Generate_Bindings_Options;

    array_add(*options.source_files, tprint("%driven/bindings-vtable-abi.hpp", #filepath));
    defer array_free(options.source_files);
    for argument: string.["-x", "c++", "-target", "x86_64-pc-windows-msvc"] array_add(*options.extra_clang_arguments, argument);

    defer array_free(options.extra_clang_arguments);
    options.add_generator_command = false;
    options.log_stripped_declarations = false;
    builder, success := W.generate_bindings(options);

    defer free_buffers(*builder);
    assert(success && !builder.failed, "generation fails");
    text := builder_to_string(*builder);

    defer free(text);
    assert(contains(text, "small: #type (this: *DispatchDerived, value: s32) -> DispatchSmall #cpp_method;"), "a small record result is returned directly");
    assert(contains(text, ") -> DispatchBig #cpp_method;"), "a large record result is returned directly");
    assert(contains(text, "eight: #type (this: *DispatchDerived, value: s32) -> DispatchEight #cpp_method;"), "an 8 byte record result is returned directly");
    assert(contains(text, "sixteen: #type (this: *DispatchDerived, value: s32) -> DispatchSixteen #cpp_method;"), "a 16 byte record result is returned directly");
    assert(contains(text, ") -> DispatchDoublePair #cpp_method;") && contains(text, ") -> DispatchFloatPair #cpp_method;"), "homogeneous float aggregates are returned directly");
    assert(!contains(text, "__result_"), "no hidden result parameter is declared");

    // A class with a user-provided constructor is not a plain record; its slot is refused, not passed through.
    refused: W.Generate_Bindings_Options;

    array_add(*refused.source_files, tprint("%driven/generator-vtable-non-pod.hpp", #filepath));
    defer array_free(refused.source_files);
    for argument: string.["-x", "c++", "-target", "x86_64-pc-windows-msvc"] array_add(*refused.extra_clang_arguments, argument);

    defer array_free(refused.extra_clang_arguments);
    refused.add_generator_command = false;

    push_context {
        context.logger = quiet;
        refused_builder, refused_success := W.generate_bindings(refused);

        defer free_buffers(*refused_builder);
        assert(!refused_success || refused_builder.failed, "a slot returning a class with a constructor is refused");
    }
}

quiet :: (_text: string, _data: *void, _info: Log_Info) {}
