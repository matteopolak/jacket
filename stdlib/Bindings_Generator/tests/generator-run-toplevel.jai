// Independent native input; no optional assert contains required work.
#import "Basic";
B :: #import "Bindings_Generator";
S :: #import "internal/developer-tools/generator-session";
ST :: #import "String";

warnings: s64;
unexpected_diagnostics: s64;

record_log :: (message: string, _data: *void, _info: Log_Info) {
    if ST.contains(message, "macro RUN_CHANGED replaces a differing previous definition") warnings += 1;
    else unexpected_diagnostics += 1;
}

find :: (generator: *B.Generator, name: string) -> *B.Declaration {
    result: *B.Declaration;

    for declaration: generator.global_scope.members if declaration.name == name {
        assert(result == null, "duplicate global member");
        result = declaration;
    }

    return result;
}

run_case :: (exempt_path: bool, exempt_namespace := false) -> u32 {
    push_context, defer_pop;
    context.logger = record_log;
    warnings = 0;
    unexpected_diagnostics = 0;
    generator: B.Generator;
    options: B.Generate_Bindings_Options;

    array_add(*options.system_types_to_include, "RunIncluded");
    array_add(*options.system_types_to_include, "NestedType");
    if exempt_namespace array_add(*options.system_types_to_include, "SystemNamespace");

    defer array_free(options.system_types_to_include);
    if exempt_path array_add(*options.path_fragments_to_treat_as_non_system_paths, "generator-run-system.h");

    defer array_free(options.path_fragments_to_treat_as_non_system_paths);
    B.init_generator(*generator, options);
    defer B.deinit_generator(*generator);
    S.activate(*generator, *options, context._bindings_generator_session);
    generator.clang_index = B.clang_createIndex(0, 0);
    assert(generator.clang_index != null);
    filename := to_c_string(tprint("%../../../stdlib/Bindings_Generator/tests/driven/generator-run-main.hpp", #filepath));
    argument := to_c_string("-DRUN_ARGUMENT=40");

    defer free(filename);
    defer free(argument);
    arguments := (*u8).[argument];
    error := B.clang_parseTranslationUnit2(generator.clang_index, filename, arguments.data, 1, null, 0, 1, *generator.translation_unit);

    assert(error == .Success && generator.translation_unit != null);
    assert(B.clang_getNumDiagnostics(generator.translation_unit) == 0);
    saved := context;

    // Callback must recover its supplied context, rather than native ambient state.
    push_context {
        context.generator = null;
        B.clang_visitChildren(B.clang_getTranslationUnitCursor(generator.translation_unit), B.handle_toplevel_cursors, *saved);
        assert(context.generator == null);
    }

    assert(context.generator == *generator);
    hidden := find(*generator, "RunHidden");
    included := find(*generator, "RunIncluded");
    function := find(*generator, "run_function");
    same := find(*generator, "RUN_SAME");
    changed := find(*generator, "RUN_CHANGED");
    scope := find(*generator, "RunScope");

    assert(hidden != null);
    assert(((hidden.decl_flags & .OMIT_FROM_OUTPUT) != 0) == !exempt_path);
    assert(included != null && !(included.decl_flags & .OMIT_FROM_OUTPUT));
    system_record := cast(*B.Struct) find(*generator, "RunRetainedSystem");
    system_namespace := cast(*B.Namespace) find(*generator, "SystemNamespace");

    assert(system_record != null && system_record.declarations.count == 1);
    assert(system_record.declarations[0].name == "retained_member");
    assert(system_namespace != null);
    assert(((system_namespace.decl_flags & .OMIT_FROM_OUTPUT) != 0) == !(exempt_path || exempt_namespace));
    assert(system_namespace.declarations.count == (ifx exempt_path || exempt_namespace then 2 else 1));
    nested: *B.Struct;
    for child: system_namespace.declarations if child.name == "NestedType" nested = cast(*B.Struct) child;

    assert(nested != null && nested.declarations.count == 1);
    assert(nested.declarations[0].name == "nested_member");
    system_macro := find(*generator, "RUN_SYSTEM");

    assert((system_macro != null) == exempt_path);
    assert(function != null && !(function.decl_flags & .OMIT_FROM_OUTPUT));
    assert(same != null && changed != null && scope != null);
    equivalent_count, equivalent_omitted, different_count: s64;
    first_equivalent: *B.Function;

    for declaration: generator.global_scope.members {
        if declaration.name == "run_equivalent" {
            equivalent_count += 1;
            if declaration.decl_flags & .OMIT_FROM_OUTPUT equivalent_omitted += 1;

            function := cast(*B.Function) declaration;

            if first_equivalent {
                assert(first_equivalent != function);
                assert(B.functions_are_equivalent(first_equivalent, function));
            } else first_equivalent = function;
        }

        if declaration.name == "run_different" {
            different_count += 1;
            assert(!(declaration.decl_flags & .OMIT_FROM_OUTPUT));
        }
    }

    assert(equivalent_count == 2 && equivalent_omitted == 1 && different_count == 2);
    // Ignored non-types are suppressed at intake. Three retained types/scopes
    // in our header acquire omission/counting; the included record is exempt.
    expected_count := ifx exempt_path then 0 else ifx exempt_namespace then 2 else 3;

    assert(generator.counts.ignored_system_declarations == expected_count);

    changed_text := B.raw_string_from_macro_tokens((cast(*B.Literal) changed.expression).macro_value);

    defer free(changed_text);
    assert(changed_text == "2");
    assert(warnings == 1 && unexpected_diagnostics == 0);

    return generator.counts.ignored_system_declarations;
}

main :: () {
    prior := context.generator;

    ordinary_ignored := run_case(false);

    assert(context.generator == prior);
    exempt_ignored := run_case(true);

    assert(context.generator == prior);
    assert(ordinary_ignored == exempt_ignored + 3);
    namespace_ignored := run_case(false, true);

    assert(context.generator == prior && namespace_ignored == 2);
    print("toplevel ok\n");
}
