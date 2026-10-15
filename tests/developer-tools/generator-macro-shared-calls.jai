#import "Basic";
B :: #import "Bindings_Generator";

Capture :: struct {
    active: s64 = -1;
    expressions: [7] B.CXCursor;
    saved: #Context;
}

visit :: (cursor: B.CXCursor, _parent: B.CXCursor, data: B.CXClientData) -> B.CXChildVisitResult #c_call {
    capture := cast(*Capture) data;

    push_context {
        context = capture.saved;

        if cursor.kind == .VarDecl {
            capture.active = -1;
            native := B.clang_getCursorSpelling(cursor);
            name := to_string(B.clang_getCString(native));

            for expected, index: string.["aggregate", "runtime", "null_value", "size_value", "address_value", "wrapped", "scalar_value"] {
                if name == expected {
                    capture.active = index;
                    capture.expressions[index] = B.clang_getNullCursor();
                }
            }

            B.clang_disposeString(native);
        } else if capture.active >= 0 && B.clang_isExpression(cursor.kind) != 0 {
            if B.clang_Cursor_isNull(capture.expressions[capture.active]) != 0 capture.expressions[capture.active] = cursor;
        }
    }

    return .Recurse;
}

diagnostics: s64;

record_log :: (_message: string, _data: *void, _info: Log_Info) {
    diagnostics += 1;
}

main :: () {
    index := B.clang_createIndex(0, 0);

    assert(index != null);
    source := "struct Aggregate { int x; int y; };\nAggregate aggregate{3, -7};\nAggregate wrapped = Aggregate{5,6}; int scalar_value = int(17);\nint ordinary(int); int runtime = ordinary(2);\nvoid *null_value = nullptr; auto size_value = sizeof(Aggregate); int *address_value = &runtime;\n";
    filename := to_c_string("owned-generator-macro-calls.cpp");

    defer free(filename);
    arguments: [3] *u8;
    arguments[0] = to_c_string("-x");
    arguments[1] = to_c_string("c++");
    arguments[2] = to_c_string("-std=c++17");
    defer for argument: arguments free(argument);
    unsaved: B.CXUnsavedFile;
    unsaved.Filename = filename;
    unsaved.Contents = source.data;
    unsaved.Length = cast(type_of(unsaved.Length)) source.count;
    unit: B.CXTranslationUnit;
    error := B.clang_parseTranslationUnit2(index, filename, arguments.data, 3, *unsaved, 1, 0, *unit);

    assert(error == .Success && unit != null);
    generator: B.Generator;
    generator.clang_index = index;
    generator.translation_unit = unit;
    push_context, defer_pop;
    options: B.Generate_Bindings_Options;

    B.init_generator(*generator, options);
    defer B.deinit_generator(*generator);
    capture: Capture;
    capture.saved = context;
    B.clang_visitChildren(B.clang_getTranslationUnitCursor(unit), visit, *capture);

    for cursor: capture.expressions {
        assert(B.clang_Cursor_isNull(cursor) == 0);
    }

    expression := B.parse_call_expression(capture.expressions[0], false);

    assert(expression != null && expression.kind == .STRUCT_LITERAL);
    aggregate := cast(*B.Struct_Literal) expression;

    assert(aggregate.type_name == "Aggregate" && aggregate.values.count == 2);
    first := cast(*B.Literal) aggregate.values[0];

    assert(first.kind == .LITERAL && first.literal_kind == .INTEGER && first.int_value == 3);
    assert(aggregate.values[1].kind == .UNARY_OPERATOR);
    null_expression := B.parse_call_expression(capture.expressions[2], true);

    assert(null_expression != null && null_expression.kind == .LITERAL);
    assert((cast(*B.Literal) null_expression).literal_kind == .NULL);
    size_expression := B.parse_call_expression(capture.expressions[3], true);

    assert(size_expression != null && size_expression.kind == .LITERAL);
    size := cast(*B.Literal) size_expression;

    assert(size.literal_kind == .UNSIGNED_INTEGER && size.uint_value == 8);
    wrapped := B.parse_call_expression(capture.expressions[5], true);

    assert(wrapped != null && wrapped.kind == .STRUCT_LITERAL);
    wrapped_value := cast(*B.Struct_Literal) wrapped;

    assert(wrapped_value.type_name == "Aggregate" && wrapped_value.values.count == 2);
    scalar := B.parse_call_expression(capture.expressions[6], true);

    assert(scalar != null && scalar.kind == .LITERAL);
    assert((cast(*B.Literal) scalar).literal_kind == .INTEGER && (cast(*B.Literal) scalar).int_value == 17);
    context.logger = record_log;
    runtime := B.parse_call_expression(capture.expressions[1], true);
    address := B.parse_call_expression(capture.expressions[4], true);

    assert(runtime == null && address == null && diagnostics > 0);
    print("calls ok\n");
}
