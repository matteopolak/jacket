#import "Basic";
B :: #import "Bindings_Generator";

expected: *B.Generator;
data_calls, plain_calls: s64;
plain_started: bool;
saw_value, saw_group: bool;
original_expression: *B.Expression;
original_location: B.Location;
original_comment: B.Comment;
existing_type: *B.CType;

find :: (members: [] *B.Declaration, name: string) -> *B.Declaration {
    for declaration: members if declaration.name == name return declaration;
    return null;
}

group :: (declaration: *B.Declaration) -> string {
    assert(context.generator == expected);

    if declaration.name == "RUN_EXISTING_ADDED" {
        enumeration := find(expected.global_scope.members, "RunExisting");

        assert(enumeration != null);
        existing_type = enumeration.type;
        assert(existing_type != null);

        return "RunExisting";
    }

    if declaration.name == "RUN_MIXED_NEGATIVE" || declaration.name == "RUN_MIXED_UNSIGNED" return "RunMixed";

    if declaration.name == "RUN_GROUP_ONE" {
        original_expression = declaration.expression;
        original_location = declaration.location;
        original_comment = declaration.comment;
        return "RunGroup";
    }

    return "";
}

with_data :: (declaration: *B.Declaration, parent: *B.Declaration, data: *void) -> B.Declaration_Visit_Result {
    assert(context.generator == expected && data == expected && !plain_started);
    data_calls += 1;
    rejected := find(expected.global_scope.members, "RUN_BAD");

    assert(rejected == null);

    if declaration.name == "RUN_VALUE" {
        assert(declaration.macro_value_result == .KEEP);
        saw_value = true;
    }

    if declaration.name == "RUN_GROUP_ONE" {
        assert(parent != null && parent.kind == .ENUM && parent.name == "RunGroup");
        assert(parent.type == null);
        assert(declaration.type != null && declaration.type.type_of_enum == cast(*B.Enum) parent);
        assert(declaration.expression != null && declaration.expression == original_expression);
        assert(B.is_same_location(declaration.location, original_location));
        assert(declaration.comment.text == original_comment.text);
        saw_group = true;
    }

    if declaration.name == "RunAlias" {
        assert(!(declaration.decl_flags & .OMIT_FROM_OUTPUT));
        declaration.output_name = "s32";
    }

    if declaration.name == "KeepNamespace" {
        namespace := cast(*B.Namespace) declaration;
        assert((namespace.flags & .FLATTEN) != 0);
        namespace.flags &= ~B.Namespace.Flags.FLATTEN;
    }

    if declaration.kind == .NAMESPACE && declaration.output_name == "Merged" {
        namespace := cast(*B.Namespace) declaration;

        assert(namespace.declarations.count == 2);
        child := find(namespace.declarations, "B");

        assert(child != null && child.parent.name == "Second_Merged");
        cpp: String_Builder;

        defer free_buffers(*cpp);
        B.append_cpp_type(*cpp, child.type, null);
        text := builder_to_string(*cpp);

        defer free(text);
        qualified := text;
        if text.count >= 2 && text[0] == #char ":" && text[1] == #char ":" qualified = .{ text.count - 2, text.data + 2 };

        assert(!cpp.failed && qualified == "Second_Merged::B");
    }

    if declaration.name == "RUN_PREFIX___A" || declaration.name == "RUN_PREFIX___B" {
        assert(parent != null && parent.name == "RUN_PREFIX" && parent.kind == .ENUM);
        assert(parent.type == null);
        assert(declaration.type != null && declaration.type.type_of_enum == cast(*B.Enum) parent);
        assert(declaration.output_name == "A" || declaration.output_name == "B");
    }

    if declaration.name == "RUN_MIXED_NEGATIVE" || declaration.name == "RUN_MIXED_UNSIGNED" {
        assert(parent != null && parent.name == "RunMixed" && parent.type == null);
        assert(declaration.expression != null && declaration.type.type_of_enum == cast(*B.Enum) parent);
    }

    if declaration.name == "RUN_EXISTING_ADDED" {
        assert(parent != null && parent.name == "RunExisting" && parent.type == existing_type);
        assert(declaration.type != null && declaration.type.type_of_enum == cast(*B.Enum) parent);
    }

    if parent && parent.name == "EmptyAfter" declaration.decl_flags |= .OMIT_FROM_OUTPUT;

    return .RECURSE;
}

plain :: (declaration: *B.Declaration, _parent: *B.Declaration) -> B.Declaration_Visit_Result {
    assert(context.generator == expected && data_calls > 0 && saw_value && saw_group);
    plain_started = true;
    plain_calls += 1;

    if declaration.name == "RunAlias" {
        assert(declaration.output_name == "s32" && !(declaration.decl_flags & .OMIT_FROM_OUTPUT));
    }

    return .RECURSE;
}

run_case :: (multiple: bool, infer_cpp := false) {
    push_context, defer_pop;
    data_calls = 0;
    plain_calls = 0;
    plain_started = false;
    saw_value = false;
    saw_group = false;
    generator: B.Generator;

    expected = *generator;
    options: B.Generate_Bindings_Options;

    array_add(*options.source_files, tprint("%generator-run-pipeline.hpp", #filepath));
    if multiple array_add(*options.source_files, tprint("%generator-run-second.hpp", #filepath));

    owned_argument := copy_string("-DRUN_ARGUMENT=40");

    defer free(owned_argument);
    array_add(*options.extra_clang_arguments, owned_argument);

    if !infer_cpp {
        array_add(*options.extra_clang_arguments, "-x");
        array_add(*options.extra_clang_arguments, "c++");
        array_add(*options.extra_clang_arguments, "-std=c++17");
    }

    array_add(*options.macro_prefixes_to_unwrap, "RUN_UNWRAP_");
    array_add(*options.flatten_namespaces, "FlattenMe");
    array_add(*options.flatten_namespaces, "KeepNamespace");
    array_add(*options.strip_prefixes, "First_");
    array_add(*options.strip_prefixes, "Second_");
    array_add(*options.generate_enums_from_macros_with_prefixes, "RUN_PREFIX___");
    // These invalid legacy paths would fail loading if direct run normalized.
    array_add(*options.libnames, "__run_legacy_must_not_load__");
    array_add(*options.libpaths, "/__run_legacy_missing__");
    array_add(*options.system_library_names, "__run_system_legacy_must_not_load__");
    array_add(*options.system_library_paths, "/__run_system_legacy_missing__");
    defer array_free(options.libnames);
    defer array_free(options.libpaths);
    defer array_free(options.system_library_names);
    defer array_free(options.system_library_paths);
    defer array_free(options.source_files);
    defer array_free(options.extra_clang_arguments);
    defer array_free(options.macro_prefixes_to_unwrap);
    defer array_free(options.flatten_namespaces);
    defer array_free(options.strip_prefixes);
    defer array_free(options.generate_enums_from_macros_with_prefixes);
    options.visitor_with_data = with_data;
    options.visitor_data = *generator;
    options.visitor = plain;
    options.convert_macro_value_to_enum_callback = group;
    B.init_generator(*generator, options);
    defer B.deinit_generator(*generator);
    initialized := generator.type_def_s32;
    ok := B.run_generator(*generator, options);

    assert(ok && context.generator == *generator && generator.type_def_s32 == initialized);
    assert(generator.libraries.count == 0);
    assert(options.libnames.count == 1 && options.system_library_names.count == 1);
    assert(options.libraries.count == 0 && options.system_libraries.count == 0);
    fresh := cast(*B.Enum) find(generator.global_scope.members, "RUN_PREFIX");
    mixed := cast(*B.Enum) find(generator.global_scope.members, "RunMixed");
    existing := cast(*B.Enum) find(generator.global_scope.members, "RunExisting");

    assert(fresh != null && fresh.type == null && fresh.enumerates.count == 2);
    assert(mixed != null && mixed.type == null && mixed.enumerates.count == 2);
    assert(existing != null && existing.type == existing_type && existing.enumerates.count == 2);
    assert(data_calls > 0 && plain_calls > 0 && saw_value && saw_group);
    alias := find(generator.global_scope.members, "RunAlias");
    unwrap := find(generator.global_scope.members, "RUN_UNWRAP_ONE");
    empty := cast(*B.Namespace) find(generator.global_scope.members, "EmptyAfter");

    assert(alias != null && (alias.decl_flags & .OMIT_FROM_OUTPUT) != 0);
    assert(unwrap != null && unwrap.should_unwrap && (unwrap.decl_flags & .OMIT_FROM_OUTPUT) != 0);
    found, substituted, _, _ := B.find_and_check_macro_identifier("RUN_UNWRAP_ONE", ignore_omit_flag = true);

    assert(found && substituted == "9");
    assert(empty != null && (empty.flags & .IS_EMPTY) != 0);
    retained := find(generator.flattened_global_scope_members, "Retained");

    assert(retained != null && find(generator.flattened_global_scope_members, "EmptyAfter") == null);
    assert(find(generator.flattened_global_scope_members, "KeepNamespace") != null);
    assert(find(generator.flattened_global_scope_members, "Kept") == null);
    assert(find(generator.global_scope.members, "Second_Merged") == null);
    assert(generator.translation_unit != null && generator.clang_index != null);
    dependent := find(generator.global_scope.members, "RUN_SECOND");

    assert((dependent != null) == multiple);
    // Revalidation must use the retained exact receipt, after caller-owned
    // argument bytes change. A guessed/reborrowed -DRUN_ARGUMENT=XX fails.
    owned_argument[owned_argument.count - 1] = #char "X";
    owned_argument[owned_argument.count - 2] = #char "X";
    scalar := find(generator.global_scope.members, "RUN_VALUE");
    scalar.macro_value_result = .NEEDS_CHECKING;
    B.check_macro_value(scalar);
    assert(scalar.macro_value_result == .KEEP);
    // Borrowed model/TU remains live after run, until the separate deinit call.
    value := find(generator.global_scope.members, "RUN_UTF8");
    text := B.string_from_macro_value((cast(*B.Literal) value.expression).macro_value, name = value.name);

    defer free(text);
    assert(text == "\"héllo\"");
}

main :: () {
    prior := context.generator;

    run_case(false);
    assert(context.generator == prior);
    run_case(true);
    assert(context.generator == prior);
    // The original first .hpp already selects C++ in the official SDK. An
    // aggregate wrapper must preserve that language when no -x is supplied.
    run_case(true, true);
    assert(context.generator == prior);
    print("pipeline ok\n");
}
