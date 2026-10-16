#import "Basic";
B :: #import "Bindings_Generator";

main :: () {
    push_context, defer_pop;
    options: B.Generate_Bindings_Options;

    array_add(*options.source_files, tprint("%bindings-model-input.h", #filepath));
    defer array_free(options.source_files);
    generator: B.Generator;

    B.init_generator(*generator, options);
    defer B.deinit_generator(*generator);
    assert(B.run_generator(*generator, options));
    defined, incomplete: *B.Struct;

    for declaration: generator.global_scope.members {
        if declaration.kind != .STRUCT continue;
        if declaration.name == "Forward" defined = cast(*B.Struct) declaration;
        if declaration.name == "Opaque" incomplete = cast(*B.Struct) declaration;
    }

    assert(defined && incomplete && defined != incomplete);
    assert(defined.type.type_of_struct == defined && defined.type.size == 4 && defined.type.alignment == 4);
    assert(defined.declarations.count == 1 && defined.declarations[0].name == "value");
    info, generated := B.find_or_create_extra_info(defined.type, cast(*B.Declaration) defined);

    assert(info && info != *B.struct_extra_info_unknown && generated == defined && info.modified_declarations.count == 1);
    assert(incomplete.type.type_of_struct == incomplete && incomplete.type.size < 0 && incomplete.type.alignment < 0 && incomplete.declarations.count == 0);
    count := generator.extra_infos.count;
    unknown, unknown_generated := B.find_or_create_extra_info(incomplete.type, cast(*B.Declaration) incomplete);

    assert(unknown == *B.struct_extra_info_unknown && unknown_generated == null && generator.extra_infos.count == count);
    assert(!context._bindings_generator_session.failed);
    print("defined Forward size=% alignment=%; incomplete Opaque size=% alignment=%; canonical unknown retained\n", defined.type.size, defined.type.alignment, incomplete.type.size, incomplete.type.alignment);
    print("ok\n");
}
