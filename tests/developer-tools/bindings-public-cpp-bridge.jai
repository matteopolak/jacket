#import "Basic";
#import "File";
#import "String";
B :: #import "Bindings_Generator";

S :: type_of(context._bindings_generator_session.*);

unused_jai_callback :: (_func: *B.Function, _type: *B.Function_Type, _mode: B.Arg_Print_Mode) -> [] *B.Declaration {
    assert(false, "C++ bridge must use represented arguments rather than Jai callback");
    return .[];
}

Capture :: struct {
    saved: #Context;
    namespace_cursor: B.CXCursor;
    global_cursor: B.CXCursor;
}

errors: int;

capture_error :: (_text: string, _data: *void, info: Log_Info) {
    if info.common_flags & .ERROR errors += 1;
}

check_defaults :: (builder: *String_Builder, namespace: *B.Namespace, generator: *B.Generator, existing: *[..] *B.Declaration) {
    original_count := existing.count;

    defer existing.count = original_count;
    value: B.Literal;
    value.literal_kind = .INTEGER;
    value.int_value = 5;
    first := B.Declaration.{ kind = .DECLARATION, output_name = "first", type = generator.type_def_s32, expression = cast(*B.Expression) *value };
    second := B.Declaration.{ kind = .DECLARATION, output_name = "second", type = generator.type_def_s32 };
    type: B.Function_Type;
    type.return_type = generator.type_def_s32;
    array_add(*type.arguments, *first);
    array_add(*type.arguments, *second);
    defer array_free(type.arguments);
    function_type := B.CType.{ type_of_function = *type };
    function: B.Function;
    function.kind = .FUNCTION;
    function.name = "sum";
    function.output_name = "nontrailing_default";
    function.parent = cast(*B.Declaration) namespace;
    function.type = *function_type;
    B.append_cpp_wrapper(builder, *function, existing);
    assert(!builder.failed);
    // A nontrailing represented default must not produce invalid C++ syntax.
    text := builder_to_string(builder, do_reset = false);

    defer free(text);
    assert(contains(text, "NativeBridge_nontrailing_default"));
    invalid: B.Expression;

    second.expression = *invalid;
    function.output_name = "unsupported_default";
    failed: String_Builder;

    defer free_buffers(*failed);
    append(*failed, "preserved");

    push_context {
        context.logger = capture_error;
        B.append_cpp_wrapper(*failed, *function, null);
    }

    assert(failed.failed && errors >= 1 && builder_string_length(*failed) == "preserved".count);
}

visit :: (cur: B.CXCursor, _parent: B.CXCursor, data: B.CXClientData) -> B.CXChildVisitResult #c_call {
    capture := cast(*Capture) data;

    push_context {
        context = capture.saved;
        spelling := B.clang_getCursorSpelling(cur);

        defer B.clang_disposeString(spelling);
        if cur.kind == .Namespace && to_string(B.clang_getCString(spelling)) == "NativeBridge" capture.namespace_cursor = cur;
        if cur.kind == .FunctionDecl && to_string(B.clang_getCString(spelling)) == "global_cpp" capture.global_cursor = cur;
    }

    return .Continue;
}

emit :: (builder: *String_Builder, declaration: *B.Declaration, existing: *[..] *B.Declaration) {
    if declaration.kind == .FUNCTION {
        if declaration.name == "sum" declaration.type.type_of_function.arguments[0].type = context.generator.type_def_float64;
        if declaration.name == "message" declaration.decl_flags |= .NEEDS_PRINTF_WRAPPER;
        B.append_cpp_wrapper(builder, cast(*B.Function) declaration, existing);
        assert(!builder.failed);
    } else if declaration.kind == .NAMESPACE {
        for member: (cast(*B.Namespace) declaration).declarations emit(builder, member, existing);
    } else if declaration.kind == .STRUCT {
        for member: (cast(*B.Struct) declaration).declarations emit(builder, member, existing);
    }
}

emit_jai :: (builder: *String_Builder, namespace: *B.Namespace, generator: *B.Generator) {
    append(builder, "native :: #foreign_library \"public_bridge\";\n");

    for member: namespace.declarations {
        if member.kind == .STRUCT && member.name == "Pair" B.print_struct(builder, cast(*B.Struct) member, null);
        if member.kind != .FUNCTION continue;

        function := cast(*B.Function) member;
        type := function.type.type_of_function.*;
        arguments: [..] *B.Declaration;

        defer array_free(arguments);
        output: B.Declaration;
        pointer: B.CType;
        count, data: B.Declaration;
        data_pointer: B.CType;
        base := B.strip_type(type.return_type, typedefs = true);

        if base.type_of_struct {
            pointer.pointer_to = type.return_type;
            output.kind = .DECLARATION;
            output.output_name = "out_return_value";
            output.type = *pointer;
            used := type.arguments;

            B.avoid_name_clash(*used, *output, false, insert = false);
            array_add(*arguments, *output);
            type.return_type = generator.type_def_void;
        }

        formatting := (member.decl_flags & .NEEDS_PRINTF_WRAPPER) != 0;

        for argument, index: type.arguments {
            if formatting && index == type.arguments.count - 1 break;
            array_add(*arguments, argument);
        }

        if formatting {
            count.kind = .DECLARATION;
            count.output_name = "message_count";
            count.type = generator.type_def_s64;
            data.kind = .DECLARATION;
            data.output_name = "message_data";
            data_pointer.pointer_to = generator.type_def_u8;
            data.type = *data_pointer;
            array_add(*arguments, *count);
            array_add(*arguments, *data);
            type.flags &= ~B.Function_Type.Flags.VARIADIC;
        }

        type.arguments.count = arguments.count;
        type.arguments.data = arguments.data;
        print_to_builder(builder, "% :: ", member.output_name);
        B.print_function_type_to_builder(builder, *type, cast(*B.Declaration) namespace);
        print_to_builder(builder, " #foreign native \"%\";\n", member.output_name);
        assert(!builder.failed);
    }
}

main :: () {
    args := get_command_line_arguments();

    defer array_free(args);
    assert(args.count >= 2);
    generator: B.Generator;
    options: B.Generate_Bindings_Options;
    options.get_func_args_for_printing = unused_jai_callback;

    for index: 2..args.count - 2 if args[index] == "-target" {
        triple := args[index + 1];
        if contains(triple, "windows") options.os = .WINDOWS;
        else if contains(triple, "linux") options.os = .LINUX;
        options.cpu = ifx begins_with(triple, "aarch64") || begins_with(triple, "arm64") then .ARM64 else .X64;
    }

    push_context {
        B.init_generator(*generator, options);
        defer B.deinit_generator(*generator);
        S.activate(*generator, *options, context._bindings_generator_session);
        generator.clang_index = B.clang_createIndex(0, 0);
        path := tprint("%bindings-public-cpp-bridge.hpp", #filepath);
        filename := to_c_string(path);

        defer free(filename);
        arguments: [..] *u8;

        array_add(*arguments, to_c_string("-x"));
        array_add(*arguments, to_c_string("c++"));
        array_add(*arguments, to_c_string("-std=c++17"));
        for index: 2..args.count - 1 array_add(*arguments, to_c_string(args[index]));

        defer {
            for argument: arguments free(argument);
            array_free(arguments);
        }

        assert(B.clang_parseTranslationUnit2(generator.clang_index, filename, arguments.data, cast(s32) arguments.count, null, 0, 0, *generator.translation_unit) == .Success);

        for index: 0..cast(int) B.clang_getNumDiagnostics(generator.translation_unit) - 1 {
            diagnostic := B.clang_getDiagnostic(generator.translation_unit, cast(u32) index);

            defer B.clang_disposeDiagnostic(diagnostic);
            spelling := B.clang_getDiagnosticSpelling(diagnostic);

            defer B.clang_disposeString(spelling);
            print("SDK diagnostic: %\n", to_string(B.clang_getCString(spelling)));
        }

        assert(B.clang_getNumDiagnostics(generator.translation_unit) == 0);
        capture: Capture;
        capture.saved = context;
        root := B.clang_getTranslationUnitCursor(generator.translation_unit);

        B.clang_visitChildren(root, visit, *capture);
        assert(B.clang_Cursor_isNull(capture.namespace_cursor) == 0);
        declaration := B.create_declaration(capture.namespace_cursor, root);

        assert(declaration && declaration.kind == .NAMESPACE);
        builder: String_Builder;

        defer free_buffers(*builder);
        print_to_builder(*builder, "#include \"%\"\n", path);
        existing: [..] *B.Declaration;

        defer array_free(existing);
        emit(*builder, declaration, *existing);
        assert(existing.count == 14);
        global := B.create_function(capture.global_cursor, root);

        assert(global != null);
        B.append_cpp_wrapper(*builder, global, *existing);
        assert(!builder.failed && global.output_name == "global_cpp");
        check_defaults(*builder, cast(*B.Namespace) declaration, *generator, *existing);
        text := builder_to_string(*builder);

        defer free(text);
        assert(write_entire_file(tprint("%/developer-tools-public-bridge.cpp", args[1]), text));
        emit_jai(*builder, cast(*B.Namespace) declaration, *generator);
        append(*builder, "global_cpp :: ");
        B.print_function_type_to_builder(*builder, global.type.type_of_function, null);
        append(*builder, " #foreign native \"global_cpp\";\n");
        assert(!builder.failed);
        jai := builder_to_string(*builder);

        defer free(jai);
        assert(write_entire_file(tprint("%/developer-tools-public-bridge-generated.jai", args[1]), jai));
    }

    print("ok\n");
}
