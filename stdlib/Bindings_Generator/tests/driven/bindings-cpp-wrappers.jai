// Generates bindings for two C++ interface headers and checks the slot wrappers printed for their virtual
// methods, then writes the files the round trip program loads.
#import "Basic";
#import "File";
#import "String";
B :: #import "Bindings_Generator";

generate :: (header: string, wrappers: bool, flatten: [] string) -> string {
    options: B.Generate_Bindings_Options;

    defer array_free(options.source_files);
    defer array_free(options.extra_clang_arguments);
    defer array_free(options.flatten_namespaces);

    array_add(*options.source_files, tprint("%%", #filepath, header));
    array_add(*options.extra_clang_arguments, "-x");
    array_add(*options.extra_clang_arguments, "c++");
    for name: flatten array_add(*options.flatten_namespaces, name);

    options.add_generator_command = false;
    options.log_stripped_declarations = false;
    options.generate_c_style_api_for_vtable = wrappers;
    builder, success := B.generate_bindings(options);

    defer free_buffers(*builder);
    assert(success && !builder.failed);

    return builder_to_string(*builder);
}

has :: (text: string, line: string) -> bool {
    return contains(text, line);
}

main :: () {
    args := get_command_line_arguments();

    defer array_free(args);
    assert(args.count == 2);

    shapes := generate("bindings-cpp-wrappers.hpp", true, .[]);

    defer free(shapes);

    // A slot is named for its method; an overload takes the numeric suffix and a reserved word an underscore.
    assert(has(shapes, "  area: (this: *IShape) -> s32 #c_call; // C++ virtual: area"));
    assert(has(shapes, "  scale_1: (this: *IShape, by: *Point) -> s32 #c_call; // C++ virtual: scale"));
    assert(has(shapes, "  _cast: (this: *IShape, value: s32) -> s32 #c_call; // C++ virtual: cast"));
    assert(has(shapes, "  origin: (this: *IShape) -> *Point #c_call; // C++ virtual: origin"));

    // A wrapper per method, named for the class that declares it and taking that class's pointer.
    assert(has(shapes, "IRefCounted_release :: inline (this: *IRefCounted) -> u32 { return this.__vtable.release(this); }"));
    assert(has(shapes, "IShape_scale :: inline (this: *IShape, by: s32) -> s32 { return this.__vtable.scale(this, by); }"));
    assert(has(shapes, "IShape_scale_1 :: inline (this: *IShape, by: *Point) -> s32 { return this.__vtable.scale_1(this, by); }"));
    assert(has(shapes, "IShape_cast :: inline (this: *IShape, value: s32) -> s32 { return this.__vtable._cast(this, value); }"));
    assert(has(shapes, "IShape_origin :: inline (this: *IShape) -> *Point { return this.__vtable.origin(this); }"));
    assert(has(shapes, "IBox_split :: inline (this: *IBox, first: **IShape, second: **IShape) -> s32 { return this.__vtable.split(this, first, second); }"));
    assert(has(shapes, "ICube_volume :: inline (this: *ICube) -> s32 { return this.__vtable.volume(this); }"));

    // The same wrapper for each derived record, reached through the base class's name.
    assert(has(shapes, "IRefCounted_release :: inline (this: *ICube) -> u32 { return this.__vtable.release(cast(*IRefCounted) this); }"));
    assert(has(shapes, "IShape_area :: inline (this: *ICube) -> s32 { return this.__vtable.area(cast(*IShape) this); }"));
    assert(has(shapes, "IBox_side :: inline (this: *ICube) -> s32 { return this.__vtable.side(cast(*IBox) this); }"));
    assert(!has(shapes, "ICube_area ::") && !has(shapes, "ICube_release ::"));

    // The typed vtable accessor does not depend on the wrappers.
    bare := generate("bindings-cpp-wrappers.hpp", false, .[]);

    defer free(bare);
    assert(has(bare, "vtable :: inline (obj: *ICube)") && !has(bare, "IShape_area ::") && has(bare, "area: (this: *IShape)"));

    sdk := generate("bindings-cpp-wrappers-slang.hpp", true, .["slang"]);

    defer free(sdk);
    assert(has(sdk, "IGlobalSession_createSession :: inline (this: *IGlobalSession, desc: *SessionDesc, outSession: **ISession) -> SlangResult { return this.__vtable.createSession(this, desc, outSession); }"));
    assert(has(sdk, "ISlangUnknown_release :: inline (this: *ISlangUnknown) -> u32 { return this.__vtable.release(this); }"));
    assert(has(sdk, "ISlangUnknown_release :: inline (this: *ISession) -> u32 { return this.__vtable.release(cast(*ISlangUnknown) this); }"));
    assert(has(sdk, "ISession_describe :: inline (this: *ISession, outDesc: *SessionDesc) -> SlangResult"));

    assert(write_entire_file(tprint("%/developer-tools-cpp-wrappers-shapes.jai", args[1]), shapes));
    assert(write_entire_file(tprint("%/developer-tools-cpp-wrappers-sdk.jai", args[1]), sdk));
    print("ok\n");
}
