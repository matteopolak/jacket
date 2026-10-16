#import "Basic";
B :: #import "Bindings_Generator";
#if OS == .WINDOWS W :: #import "Windows";
else P :: #import "POSIX";

diagnostics, visits: s64;
expected: *B.Generator;

record_log :: (_message: string, _data: *void, _info: Log_Info) {
    diagnostics += 1;
}

visit :: (declaration: *B.Declaration, _parent: *B.Declaration) -> B.Declaration_Visit_Result {
    visits += 1;

    if declaration.name == "run_library_symbol" {
        assert(context.generator == expected && declaration.library == expected.libraries[0]);
        assert(!(declaration.decl_flags & .OMIT_FROM_OUTPUT));
    }

    return .RECURSE;
}

release :: (library: *B.Library_Info) {
    if library.handle {
        #if OS == .WINDOWS {
            ok := W.FreeLibrary(library.handle);
            assert(ok != 0);
        } else {
            status := P.dlclose(library.handle);
            assert(status == 0);
        }
    }

    free(library.identifier);
    free(library.full_path);
    for symbol: library.symbol_names free(symbol);

    array_free(library.symbol_names);
    free(library);
}

run_case :: (path: string, fail_load: bool) {
    push_context, defer_pop;
    context.logger = record_log;
    diagnostics = 0;
    visits = 0;
    generator: B.Generator;

    expected = *generator;
    options: B.Generate_Bindings_Options;

    array_add(*options.source_files, tprint("%generator-run-library.h", #filepath));
    array_add(*options.libraries, B.Library.{ filename = path, identifier = "Ordinary" });
    if fail_load array_add(*options.libraries, B.Library.{ filename = "owned-unavailable-library", identifier = "Unavailable" });

    array_add(*options.system_libraries, B.Library.{ filename = path, identifier = "System" });
    defer array_free(options.source_files);
    defer array_free(options.libraries);
    defer array_free(options.system_libraries);
    options.visitor = visit;
    B.init_generator(*generator, options);
    ok := B.run_generator(*generator, options);

    assert(ok == !fail_load);
    assert(generator.libraries.count == 2);
    ordinary := generator.libraries[0];
    system := generator.libraries[1];

    assert(ordinary.identifier == "Ordinary" && !ordinary.is_system_library);
    assert(system.identifier == "System" && system.is_system_library);
    if fail_load assert(diagnostics > 0 && visits == 0);
    else assert(diagnostics == 0 && visits > 0);

    // External library handles/records belong to tooling/caller, not the model
    // ledger. Preserve them and release before model graph cleanup.
    release(ordinary);
    release(system);
    B.deinit_generator(*generator);
    assert(context.generator == *generator && generator.translation_unit == null && generator.clang_index == null);
}

main :: () {
    arguments := get_command_line_arguments();

    defer free(arguments.data);
    assert(arguments.count == 2);
    run_case(arguments[1], false);
    run_case(arguments[1], true);
    print("libraries ok\n");
}
