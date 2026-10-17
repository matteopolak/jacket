#import "Basic";
B :: #import "Bindings_Generator";

diagnostics, visits: s64;

record_log :: (_message: string, _data: *void, _info: Log_Info) {
    diagnostics += 1;
}

visit :: (_declaration: *B.Declaration, _parent: *B.Declaration) -> B.Declaration_Visit_Result {
    visits += 1;
    return .RECURSE;
}

run_case :: (mode: s64) {
    push_context, defer_pop;
    context.logger = record_log;
    diagnostics = 0;
    visits = 0;
    generator: B.Generator;
    options: B.Generate_Bindings_Options;

    // Initialize against a valid target independently of the failing input.
    B.init_generator(*generator, options);
    defer B.deinit_generator(*generator);
    filename := ifx mode == 0 then "generator-run-error.h" else "generator-run-warning.h";
    if mode == 2 filename = "owned-generator-run-absent.h";
    if mode == 4 filename = "generator-run-unsupported.hpp";

    array_add(*options.source_files, tprint("%%", #filepath, filename));
    defer array_free(options.source_files);
    options.visitor = visit;
    if mode == 3 array_add(*options.extra_clang_arguments, "-DINVALID\0ARGUMENT");

    if mode == 4 {
        array_add(*options.extra_clang_arguments, "-x");
        array_add(*options.extra_clang_arguments, "c++");
    }

    defer array_free(options.extra_clang_arguments);
    ok := B.run_generator(*generator, options);

    assert(context.generator == *generator);
    assert(diagnostics > 0);

    if mode == 1 || mode == 4 {
        assert(ok && visits > 0 && generator.translation_unit != null);
    } else assert(!ok && visits == 0);
}

main :: () {
    for mode: 0..4 run_case(mode);
    print("failures ok\n");
}
