// Library-unit fixture: private imports configure only the exact canonical
// parse-input receipt. All conversion/validation calls use the public APIs.
Capture :: struct {
    declarations: [..] *B.Declaration;
    generator: *B.Generator;
    session: *Sessions.Session;
    enumeration: *B.Enum;
    saved: #Context;
}

visit :: (cursor: B.CXCursor, parent: B.CXCursor, data: B.CXClientData) -> B.CXChildVisitResult #c_call {
    capture := cast(*Capture) data;

    push_context {
        context = capture.saved;

        if cursor.kind == .MacroDefinition {
            native := B.clang_getCursorSpelling(cursor);
            name := to_string(B.clang_getCString(native));
            owned := begins_with(name, "OWN_");

            B.clang_disposeString(native);

            if owned {
                declaration := B.create_macro_definition(cursor);
                assert(declaration != null);
                array_add(*capture.declarations, declaration);
                Sessions.append_owned(capture.session, *capture.generator.global_scope.members, declaration);
            }
        } else if cursor.kind == .EnumDecl {
            capture.enumeration = B.create_enum(cursor);
        } else if cursor.kind == .VarDecl {
            declaration := B.create_declaration(cursor, parent);
            assert(declaration != null);
            Sessions.append_owned(capture.session, *capture.generator.global_scope.members, declaration);
        }
    }

    return .Continue;
}

find :: (capture: *Capture, name: string) -> *B.Declaration {
    for declaration: capture.declarations if declaration.name == name return declaration;
    assert(false, "original fixture declaration not found: %", name);
    return null;
}

diagnostics: s64;

record_log :: (_message: string, _data: *void, _info: Log_Info) {
    diagnostics += 1;
}

verify :: (with_receipt: bool) {
    index := B.clang_createIndex(0, 0);

    assert(index != null);
    filename := to_c_string(tprint("%../../../stdlib/Bindings_Generator/tests/driven/generator-macro-shared-state.h", #filepath));

    defer free(filename);
    source := "const int outside = 7;\nenum Other { NATIVE_VALUE = 11 };\n#define OWN_ARITH (7+35)\n#define OWN_UNSIGNED 0xffffffffffffffffULL\n#define OWN_FLOAT 1.5F\n#define OWN_UTF8 \"héllo\"\n#define OWN_CAST ((int)-3)\n#define OWN_ALIAS OWN_ARITH\n#define OWN_ENUM NATIVE_VALUE\n#define OWN_GAPPED (7  +35)\n#define OWN_DOUBLE ((7+35))\n#define OWN_BITAND (7 & 3)\n#define OWN_PREFIX (1 && &outside)\n#define OWN_CAST_ADDRESS ((bool)&outside)\n#define OWN_UNKNOWN unknown_identifier\n#define OWN_SELF OWN_SELF\n#define OWN_BRACES {1}\n#define OWN_COMMA (1,2)\n#define OWN_ADDRESS (&outside)\n#define OWN_MEMBER object.member\n#define OWN_EMPTY ordinary()\n#define OWN_CONCAT \"a\" \"b\"\n#define OWN_NUL \"A\\0B\"\n#define OWN_WIDE L\"wide\"\n";
    arguments: [3] string = .["-x", "c++", "-std=c++17"];
    native_arguments: [3] *u8;

    for argument, index: arguments native_arguments[index] = to_c_string(argument);

    defer for argument: native_arguments free(argument);
    overlay: B.CXUnsavedFile;
    overlay.Filename = filename;
    overlay.Contents = source.data;
    overlay.Length = cast(type_of(overlay.Length)) source.count;
    unit: B.CXTranslationUnit;
    error := B.clang_parseTranslationUnit2(index, filename, native_arguments.data, 3, *overlay, 1, 1, *unit);

    assert(error == .Success && unit != null);
    generator: B.Generator;
    generator.clang_index = index;
    generator.translation_unit = unit;
    options: B.Generate_Bindings_Options;
    options.log_unsupported = true;
    push_context, defer_pop;
    B.init_generator(*generator, options);
    defer B.deinit_generator(*generator);
    session := Sessions.current(OS);

    assert(session != null);
    Sessions.activate(*generator, *options, session);
    context.logger = record_log;
    overlays: [1] Sessions.Source_Overlay;
    overlays[0] = .{ to_string(filename), source };
    input: Sessions.Parse_Input;
    input.filename = to_string(filename);
    input.arguments = arguments;
    input.overlays = overlays;
    if with_receipt session.parse_input = *input;

    capture: Capture;
    capture.generator = *generator;
    capture.session = session;
    capture.saved = context;
    defer array_free(capture.declarations);
    B.clang_visitChildren(B.clang_getTranslationUnitCursor(unit), visit, *capture);
    assert(capture.enumeration != null && capture.enumeration.enumerates.count == 1);
    capture.enumeration.output_name = Sessions.copy_text(session, "RenamedEnum");
    member := capture.enumeration.enumerates[0];
    member.output_name = Sessions.copy_text(session, "Member");
    H.table_add(*generator.macros_converted_to_enum_values, "NATIVE_VALUE", capture.enumeration);
    original_cursors := generator.declarations_by_cursor.count;
    arithmetic := find(*capture, "OWN_ARITH");
    original_expression := arithmetic.expression;
    original_type := arithmetic.type;

    B.check_macro_value(arithmetic);

    if !with_receipt {
        assert(arithmetic.macro_value_result == .NEEDS_CHECKING);
        assert(arithmetic.expression == original_expression && diagnostics > 0);
        return;
    }

    assert(arithmetic.macro_value_result == .KEEP && arithmetic.expression == original_expression);
    assert(arithmetic.type == original_type);
    arithmetic.output_name = Sessions.copy_text(session, "RenamedArithmetic");
    value := (cast(*B.Literal) arithmetic.expression).macro_value;
    text := B.string_from_macro_value(value, name = arithmetic.name);

    defer free(text);
    assert(text == "7+35");

    for name: string.["OWN_UNSIGNED", "OWN_FLOAT", "OWN_UTF8", "OWN_CAST", "OWN_ALIAS", "OWN_ENUM", "OWN_GAPPED", "OWN_DOUBLE", "OWN_BITAND"] {
        declaration := find(*capture, name);
        original_type := declaration.type;

        assert(declaration.macro_value_result == .NEEDS_CHECKING);

        if name == "OWN_UNSIGNED" {
            found, _output, _type, _enum := B.find_and_check_macro_identifier(name);
            assert(found);
        } else B.check_macro_value(declaration);

        assert(declaration.macro_value_result == .KEEP);
        assert(declaration.type == original_type);
        // Omission happens after both referenced and referring values were
        // checked. Rendering must still resolve the retained canonical name.
        if name == "OWN_ALIAS" arithmetic.decl_flags |= .OMIT_FROM_OUTPUT;

        macro_value := (cast(*B.Literal) declaration.expression).macro_value;
        converted := B.string_from_macro_value(macro_value, name = declaration.name);

        if name == "OWN_UNSIGNED" assert(converted == "0xffffffffffffffff");
        if name == "OWN_FLOAT" assert(converted == "1.5");
        if name == "OWN_UTF8" assert(converted == "\"héllo\"");
        if name == "OWN_CAST" assert(converted == "cast,truncate(s32) -3");
        if name == "OWN_ALIAS" assert(converted == "RenamedArithmetic");

        if name == "OWN_ENUM" {
            assert(converted == "RenamedEnum.Member");
            member.decl_flags |= .OMIT_FROM_OUTPUT;
            local := B.string_from_macro_value(macro_value, capture.enumeration, declaration.name);

            assert(local == "Member");
            free(local);
        }

        if name == "OWN_GAPPED" assert(converted == "7 +35");
        if name == "OWN_DOUBLE" assert(converted == "(7+35)");
        if name == "OWN_BITAND" assert(converted == "7 & 3");

        free(converted);
    }

    alias := find(*capture, "OWN_ALIAS");
    found, output, type, _parent := B.find_and_check_macro_identifier(alias.name, ignore_omit_flag = true);

    assert(found && output == alias.output_name && type == alias.type);

    for name: string.["OWN_UNKNOWN", "OWN_SELF", "OWN_BRACES", "OWN_COMMA", "OWN_ADDRESS", "OWN_MEMBER", "OWN_EMPTY", "OWN_CONCAT", "OWN_NUL", "OWN_WIDE", "OWN_PREFIX", "OWN_CAST_ADDRESS"] {
        declaration := find(*capture, name);

        B.check_macro_value(declaration);
        assert(declaration.macro_value_result == .DISCARD, "macro % was not discarded", name);
        accepted, _output, _type, _enum := B.find_and_check_macro_identifier(declaration.name, ignore_omit_flag = true);

        assert(!accepted);
    }

    assert(generator.declarations_by_cursor.count == original_cursors);
    assert(diagnostics > 0);
}

main :: () {
    verify(false);
    verify(true);
    print("state ok\n");
}

#scope_file
#import "Basic";
#import "String";
B :: #import "Bindings_Generator";
Sessions :: #import "internal/developer-tools/generator-session";
H :: #import "Hash_Table";
