// Native-valid unsupported model input must not poison independent valid
// macro validation before the reviewed user-visitor stage.
#import "Basic";
B :: #import "Bindings_Generator";

diagnostics, visits: s64;
saw_macro: bool;

record_log :: (_message: string, _data: *void, _info: Log_Info) {
    diagnostics += 1;
}

visit :: (declaration: *B.Declaration, _parent: *B.Declaration) -> B.Declaration_Visit_Result {
    visits += 1;

    if declaration.name == "RUN_PENDING" {
        assert(declaration.macro_value_result == .KEEP);
        saw_macro = true;
    }

    return .RECURSE;
}

main :: () {
    push_context, defer_pop;
    context.logger = record_log;
    generator: B.Generator;
    options: B.Generate_Bindings_Options;

    B.init_generator(*generator, options);
    defer B.deinit_generator(*generator);
    array_add(*options.source_files, tprint("%generator-run-model-diagnostic.hpp", #filepath));
    array_add(*options.extra_clang_arguments, "-x");
    array_add(*options.extra_clang_arguments, "c++");
    defer array_free(options.source_files);
    defer array_free(options.extra_clang_arguments);
    options.visitor = visit;
    ok := B.run_generator(*generator, options);

    assert(ok && diagnostics > 0 && visits > 0 && saw_macro);
    print("model-diagnostic ok\n");
}
