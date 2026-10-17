#import "Basic";
#import "File";
B :: #import "Bindings_Generator";

main :: () {
    args := get_command_line_arguments();

    defer array_free(args);
    assert(args.count == 2);
    header: String_Builder;

    defer free_buffers(*header);
    for index: 0..79 print_to_builder(*header, "namespace N% {\n", index);

    append(*header, "struct Payload { int value; };\n");
    for 0..79 append(*header, "}\n");

    source := tprint("%/deep-original.hpp", args[1]);

    assert(write_entire_file(source, *header, false));
    options: B.Generate_Bindings_Options;

    array_add(*options.source_files, source);
    array_add(*options.system_include_paths, #filepath);
    array_add(*options.system_library_search_paths, args[1]);
    array_add(*options.extra_clang_arguments, "-x", "c++", "-std=c++17", "-fbracket-depth=512");
    defer array_free(options.source_files);
    defer array_free(options.system_include_paths);
    defer array_free(options.system_library_search_paths);
    defer array_free(options.extra_clang_arguments);
    options.add_generator_command = false;
    builder, success := B.generate_bindings(options);

    defer free_buffers(*builder);
    assert(success && !builder.failed);
    generator: B.Generator;

    B.init_generator(*generator, options);
    defer B.deinit_generator(*generator);
    pointers: [96] B.CType;
    root := generator.type_def_s32;

    for *pointer: pointers {
        pointer.pointer_to = root;
        pointer.size = generator.pointer_size;
        pointer.alignment = generator.pointer_size;
        root = pointer;
    }

    append(*builder, "\n#scope_export\nDeep :: ");
    B.print_type_to_builder(*builder, root, null);
    append(*builder, ";\nDeepValue :: ");
    literal: B.Literal;
    literal.literal_kind = .INTEGER;
    literal.int_value = 42;
    expression := cast(*B.Expression) *literal;
    parentheses: [96] B.Paren_Expression;

    for *parenthesis: parentheses {
        parenthesis.kind = .PAREN_EXPRESSION;
        parenthesis.child = expression;
        expression = cast(*B.Expression) parenthesis;
    }

    sum: B.Binary_Operator;
    sum.kind = .BINARY_OPERATOR;
    sum.op = "+";
    sum.lhs = expression;
    sum.rhs = expression;
    B.print_expression_to_builder(*builder, cast(*B.Expression) *sum, generator.type_def_s32, null);
    append(*builder, ";\n");
    assert(!builder.failed);
    assert(write_entire_file(tprint("%/developer-tools-deep-generated.jai", args[1]), *builder, false));
    consumer: String_Builder;

    defer free_buffers(*consumer);
    append(*consumer, "#import \"Basic\"; G :: #import \"developer-tools-deep-generated\";\nProbe :: G");
    for index: 0..79 print_to_builder(*consumer, ".N%", index);

    append(*consumer, ".Payload;\nmain :: () { value := Probe.{42}; assert(value.value == 42 && size_of(Probe) == 4 && G.DeepValue == 84); base: s32 = 42;\n");
    append(*consumer, "p0 := *base;\n");
    for index: 1..95 print_to_builder(*consumer, "p% := *p%;\n", index, index - 1);

    append(*consumer, "result: G.Deep = p95;\nresult");
    for 0..95 append(*consumer, ".*");

    append(*consumer, " = 17; assert(base == 17); print(\"ok\\n\"); }\n");
    assert(write_entire_file(tprint("%/deep-consumer.jai", args[1]), *consumer, false));
    cpp: String_Builder;

    defer free_buffers(*cpp);
    append(*cpp, "#include <stdint.h>\nusing DeepCPP = ");
    B.append_cpp_type(*cpp, root, null);
    append(*cpp, ";\nconstexpr int deep_value = ");
    assert(B.append_cpp_expression(*cpp, cast(*B.Expression) *sum));
    append(*cpp, ";\nstatic_assert(deep_value == 84, \"shared DAG value\");\nint main() { int value = 42; auto p0 = &value;\n");
    for index: 1..95 print_to_builder(*cpp, "auto p% = &p%;\n", index, index - 1);

    append(*cpp, "DeepCPP result = p95;\n");
    for 0..95 append(*cpp, "*");

    append(*cpp, "result = 17; return value == 17 ? 0 : 1; }\n");
    assert(!cpp.failed && write_entire_file(tprint("%/deep-consumer.cpp", args[1]), *cpp, false));
    fixed: B.CType;
    fixed.array_element_type = generator.type_def_s32;
    fixed.array_element_count = 4;
    fixed.size = 16;
    fixed.alignment = 4;
    incomplete: [96] B.CType;
    array_root := *fixed;

    for *array: incomplete {
        array.array_element_type = array_root;
        array.array_element_count = -1;
        array.size = -1;
        array.flags = .SHARED;
        array_root = array;
    }

    normalized, changed := B.maybe_convert_fixed_array_type(array_root);

    assert(changed && normalized != array_root);

    for 0..95 {
        assert(normalized.array_element_type != null);
        normalized = normalized.array_element_type;
    }

    assert(normalized.pointer_to == *fixed && normalized.flags & .INSERTED);
    for array: incomplete assert(array.flags & .SHARED && array.pointer_to == null);

    cycle: B.Namespace;
    cycle.kind = .NAMESPACE;
    cycle.name = "Cycle";
    cycle.output_name = "Cycle";
    array_add(*cycle.declarations, cast(*B.Declaration) *cycle);
    defer array_free(cycle.declarations);
    failed: String_Builder;

    defer free_buffers(*failed);
    append(*failed, "preserved");

    push_context {
        context.logger = capture;
        B.print_namespace(*failed, *cycle);
    }

    assert(failed.failed && builder_string_length(*failed) == "preserved".count && diagnostics > 0);
    recursive_type: B.CType;
    recursive_type.pointer_to = *recursive_type;
    recursive_expression: B.Paren_Expression;
    recursive_expression.kind = .PAREN_EXPRESSION;
    recursive_expression.child = cast(*B.Expression) *recursive_expression;

    for mode: 0..2 {
        negative: String_Builder;

        defer free_buffers(*negative);
        append(*negative, "preserved");
        prior := diagnostics;

        push_context {
            context.logger = capture;
            if mode == 0 B.print_type_to_builder(*negative, *recursive_type, null);
            else if mode == 1 B.append_cpp_type(*negative, *recursive_type, null);
            else B.print_expression_to_builder(*negative, cast(*B.Expression) *recursive_expression, null, null);
        }

        assert(negative.failed && builder_string_length(*negative) == "preserved".count && diagnostics > prior);
    }

    recovered := B.type_to_string(generator.type_def_s32, null);

    defer free(recovered);
    assert(recovered == "s32");
    array_cycle: B.CType;
    array_cycle.array_element_type = *array_cycle;
    array_cycle.array_element_count = -1;
    prior := diagnostics;

    push_context {
        context.logger = capture;
        unchanged, converted := B.maybe_convert_fixed_array_type(*array_cycle);

        assert(unchanged == *array_cycle && !converted);
        negative: String_Builder;

        defer free_buffers(*negative);
        append(*negative, "preserved");
        B.append_cpp_type(*negative, *array_cycle, null);
        assert(negative.failed && builder_string_length(*negative) == "preserved".count);
    }

    assert(diagnostics > prior);
    print("ok\n");
}

diagnostics: int;

capture :: (_text: string, _data: *void, info: Log_Info) {
    if info.common_flags & .ERROR diagnostics += 1;
}
