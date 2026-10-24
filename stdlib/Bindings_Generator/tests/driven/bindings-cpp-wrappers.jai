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
    assert(has(shapes, "area: #type (this: *IShape) -> s32 #cpp_method;"));
    assert(has(shapes, "scale_1: #type (this: *IShape, by: *Point) -> s32 #cpp_method;"));
    assert(has(shapes, "_cast: #type (this: *IShape, value: s32) -> s32 #cpp_method;"));
    assert(has(shapes, "origin: #type (this: *IShape) -> *Point #cpp_method;"));
    assert(has(shapes, "IShape_VTable :: struct #type_info_none {"));

    // A record holds its base, so a pointer to it converts to a pointer to the base; its own pointer overlays the base's.
    assert(has(shapes, "#as using,except(vtable) irefcounted: IRefCounted;"));
    assert(has(shapes, "#overlay (irefcounted) vtable: *IShape_VTable; // Overlay parent's vtable with IShape’s vtable"));
    assert(has(shapes, "#as using,except(vtable) ibox: IBox;"));
    assert(has(shapes, "using vtable: IBox_VTable;"));
    assert(!has(shapes, "__vtable"));

    // A wrapper per method, named for the class that declares it and taking that class's pointer.
    assert(has(shapes, "IRefCounted_release :: inline (this: *IRefCounted) -> u32 { return this.vtable.release(this); }"));
    assert(has(shapes, "IShape_scale :: inline (this: *IShape, by: s32) -> s32 { return this.vtable.scale(this, by); }"));
    assert(has(shapes, "IShape_scale_1 :: inline (this: *IShape, by: *Point) -> s32 { return this.vtable.scale_1(this, by); }"));
    assert(has(shapes, "IShape_cast :: inline (this: *IShape, value: s32) -> s32 { return this.vtable._cast(this, value); }"));
    assert(has(shapes, "IShape_origin :: inline (this: *IShape) -> *Point { return this.vtable.origin(this); }"));
    assert(has(shapes, "IBox_split :: inline (this: *IBox, first: **IShape, second: **IShape) -> s32 { return this.vtable.split(this, first, second); }"));
    assert(has(shapes, "ICube_volume :: inline (this: *ICube) -> s32 { return this.vtable.volume(this); }"));

    // Nothing is repeated for a derived record: the base's wrapper takes its pointer.
    assert(!has(shapes, "IRefCounted_release :: inline (this: *ICube)") && !has(shapes, "ICube_area ::") && !has(shapes, "ICube_release ::"));

    // The typed vtable accessor does not depend on the wrappers.
    assert(has(shapes, "vtable :: (obj: *ICube) -> *ICube_VTable { return obj.vtable; }"));

    bare := generate("bindings-cpp-wrappers.hpp", false, .[]);

    defer free(bare);
    assert(has(bare, "vtable :: (obj: *ICube)") && !has(bare, "IShape_area ::") && has(bare, "area: #type (this: *IShape)"));

    sdk := generate("bindings-cpp-wrappers-slang.hpp", true, .["slang"]);

    defer free(sdk);
    assert(has(sdk, "IGlobalSession_createSession :: inline (this: *IGlobalSession, desc: *SessionDesc, outSession: **ISession) -> SlangResult { return this.vtable.createSession(this, desc, outSession); }"));
    assert(has(sdk, "ISlangUnknown_release :: inline (this: *ISlangUnknown) -> u32 { return this.vtable.release(this); }"));
    assert(!has(sdk, "ISlangUnknown_release :: inline (this: *ISession)"));
    assert(has(sdk, "ISession_describe :: inline (this: *ISession, outDesc: *SessionDesc) -> SlangResult"));

    // The interface-identity macro puts an inline static member in each struct and the declaration is split over lines:
    // neither takes a slot or hides the wrapper the SDK's callers use.
    assert(has(sdk, "createSession: #type (this: *IGlobalSession, desc: *SessionDesc, outSession: **ISession) -> SlangResult #cpp_method;"));
    assert(has(sdk, "using vtable: ISlangUnknown_VTable;") && !has(sdk, "getTypeGuid"));

    assert(write_entire_file(tprint("%/developer-tools-cpp-wrappers-shapes.jai", args[1]), shapes));
    assert(write_entire_file(tprint("%/developer-tools-cpp-wrappers-sdk.jai", args[1]), sdk));
    print("ok\n");
}
