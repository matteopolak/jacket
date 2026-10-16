#import "Basic";
B :: #import "Bindings_Generator";
H :: #import "Hash_Table";

S :: type_of(context._bindings_generator_session.*);

Capture :: struct {
    saved: #Context;
    children: [..] B.CXCursor;
}

visit :: (cur: B.CXCursor, _parent: B.CXCursor, data: B.CXClientData) -> B.CXChildVisitResult #c_call {
    capture := cast(*Capture) data;

    push_context {
        context = capture.saved;
        array_add(*capture.children, cur);
    }

    return .Continue;
}

Tracker :: struct {
    underlying: Allocator;
    live: int;
}

tracked :: (mode: Allocator_Mode, bytes: s64, old_bytes: s64, memory: *void, data: *void) -> *void {
    tracker := cast(*Tracker) data;
    result := tracker.underlying.proc(mode, bytes, old_bytes, memory, tracker.underlying.data);

    if (mode == .ALLOCATE || mode == .RESIZE) && !memory && result tracker.live += 1;
    if mode == .FREE && memory tracker.live -= 1;

    return result;
}

diagnostics: int;

capture_error :: (_message: string, _data: *void, info: Log_Info) {
    if (info.common_flags & .ERROR) != 0 diagnostics += 1;
}

exercise_configuration :: () {
    push_context, defer_pop;
    context.logger = capture_error;
    source :: #string CPP
#if __cplusplus < 201103L
#define CONFIGURATION_TYPE int
#else
#define CONFIGURATION_TYPE double
#endif
template<class T = CONFIGURATION_TYPE> struct ConfigurationDefault { T value; };
template<class T = int> struct ReceiptlessBuiltin { T value; };
ConfigurationDefault<> native_configuration;
CPP
    generator: B.Generator;
    options: B.Generate_Bindings_Options;

    B.init_generator(*generator, options);
    defer B.deinit_generator(*generator);
    session := context._bindings_generator_session;

    S.activate(*generator, *options, session);
    generator.clang_index = B.clang_createIndex(0, 0);
    filename_text :: "model-default-configuration.cpp";
    filename := to_c_string(filename_text);

    defer free(filename);
    unsaved: B.CXUnsavedFile;
    unsaved.Filename = filename;
    unsaved.Contents = source.data;
    unsaved.Length = cast(type_of(unsaved.Length)) source.count;
    argument := to_c_string("-std=c++98");

    defer free(argument);
    assert(B.clang_parseTranslationUnit2(generator.clang_index, filename, *argument, 1, *unsaved, 1, cast(u32) B.CXTranslationUnit_Flags.DetailedPreprocessingRecord, *generator.translation_unit) == .Success);
    assert(B.clang_getNumDiagnostics(generator.translation_unit) == 0);
    capture := Capture.{ saved = context };

    B.clang_visitChildren(B.clang_getTranslationUnitCursor(generator.translation_unit), visit, *capture);
    defer array_free(capture.children);
    dependent, independent: B.CXCursor;
    actual: B.CXType;

    for child: capture.children {
        spelling := B.clang_getCursorSpelling(child);

        defer B.clang_disposeString(spelling);
        name := to_string(B.clang_getCString(spelling));
        if name == "ConfigurationDefault" dependent = child;
        if name == "ReceiptlessBuiltin" independent = child;
        if name == "native_configuration" actual = B.clang_Type_getTemplateArgumentAsType(B.clang_getCanonicalType(B.clang_getCursorType(child)), 0);
    }

    assert(actual.kind == .Int && B.clang_Type_getSizeOf(actual) == 4);
    before := diagnostics;

    assert(B.create_struct(dependent) == null && diagnostics > before && !session.failed);
    builtin := B.create_struct(independent);

    assert(builtin && (cast(*B.Type_Reference) builtin.template_type_params[0].expression).type == generator.type_def_s32);
    receipt: type_of(session.parse_input.*);
    receipt.filename = filename_text;
    receipt.arguments = string.["-std=c++98"];
    Overlay :: type_of(receipt.overlays[0]);
    overlays := Overlay.[.{ filename = filename_text, contents = source }];

    receipt.overlays = overlays;
    session.parse_input = *receipt;
    recovered := B.create_struct(dependent);

    assert(recovered && (cast(*B.Type_Reference) recovered.template_type_params[0].expression).type == generator.type_def_s32 && !session.failed);
}

main :: () {
    tracker := Tracker.{ underlying = context.allocator };

    push_context {
        context.allocator = .{ tracked, *tracker };
        generator: B.Generator;
        options: B.Generate_Bindings_Options;

        B.init_generator(*generator, options);
        context.allocator = tracker.underlying;
        session := context._bindings_generator_session;

        S.activate(*generator, *options, session);
        generator.clang_index = B.clang_createIndex(0, 0);
        filename_text := tprint("%generator-model-defaults.hpp", #filepath);
        filename := to_c_string(filename_text);

        defer free(filename);
        arguments := (*u8).[to_c_string("-x"), to_c_string("c++"), to_c_string("-std=c++17")];

        defer for argument: arguments free(argument);
        assert(B.clang_parseTranslationUnit2(generator.clang_index, filename, arguments.data, 3, null, 0, cast(u32) B.CXTranslationUnit_Flags.DetailedPreprocessingRecord, *generator.translation_unit) == .Success);
        assert(B.clang_getNumDiagnostics(generator.translation_unit) == 0);
        unit := generator.translation_unit;
        receipt: type_of(session.parse_input.*);
        receipt.filename = filename_text;
        receipt.arguments = string.["-x", "c++", "-std=c++17"];
        session.parse_input = *receipt;
        capture := Capture.{ saved = context };

        B.clang_visitChildren(B.clang_getTranslationUnitCursor(unit), visit, *capture);
        defer array_free(capture.children);
        generic, pointer, array, callback, reference, alias, decltype_record, signed, unsigned: *B.Struct;
        named: *B.Typedef;
        qualified: *B.Declaration;
        macro_before, scoped: *B.Struct;
        first_repeat, second_repeat: *B.Struct;

        for child: capture.children {
            spelling := B.clang_getCursorSpelling(child);

            defer B.clang_disposeString(spelling);
            name := to_string(B.clang_getCString(spelling));

            if child.kind == .TypedefDecl named = B.create_typedef(child);
            if name == "model_qualified_wide" qualified = B.create_declaration(child, B.clang_getTranslationUnitCursor(unit));

            if child.kind == .Namespace {
                namespace := cast(*B.Namespace) B.create_declaration(child, B.clang_getTranslationUnitCursor(unit));
                for member: namespace.declarations if member.kind == .STRUCT {
                    if name == "DefaultLookup" scoped = cast(*B.Struct) member;
                    if name == "DefaultFirst" first_repeat = cast(*B.Struct) member;
                    if name == "DefaultSecond" second_repeat = cast(*B.Struct) member;
                }
            }

            if child.kind != .ClassTemplate continue;

            record := B.create_struct(child);

            assert(record && B.create_struct(child) == record && record.template_type_params.count > 0);
            assert(record.template_type_params[0].expression && record.template_type_params[0].expression.kind == .TYPE_REFERENCE);
            if name == "IndependentGeneric" generic = record;
            if name == "DefaultPointer" pointer = record;
            if name == "DefaultArray" array = record;
            if name == "DefaultCallback" callback = record;
            if name == "DefaultReference" reference = record;
            if name == "DefaultAlias" alias = record;
            if name == "DefaultDecltype" decltype_record = record;
            if name == "DefaultSignedWide" signed = record;
            if name == "DefaultUnsignedWide" unsigned = record;
            if name == "DefaultMacroBefore" macro_before = record;

            assert(generator.translation_unit == unit && session.parse_input == *receipt && !session.failed);
        }

        assert(generic && pointer && array && callback && reference && alias && decltype_record && signed && unsigned && named && qualified);
        generic_type := (cast(*B.Type_Reference) generic.template_type_params[0].expression).type;

        assert(generic_type == generator.type_def_s32);
        assert(macro_before && (cast(*B.Type_Reference) macro_before.template_type_params[0].expression).type == generator.type_def_s32);
        assert(scoped && (cast(*B.Type_Reference) scoped.template_type_params[0].expression).type == generator.type_def_float64);
        assert(first_repeat && (cast(*B.Type_Reference) first_repeat.template_type_params[0].expression).type == generator.type_def_s32);
        assert(second_repeat && (cast(*B.Type_Reference) second_repeat.template_type_params[0].expression).type == generator.type_def_float64);
        assert((cast(*B.Literal) generic.template_type_params[1].expression).int_value == 3);
        pointer_type := (cast(*B.Type_Reference) pointer.template_type_params[0].expression).type;

        assert(pointer_type.pointer_to && pointer_type.pointer_to.size == 8 && (pointer_type.pointer_to.flags & .CONST) != 0);
        array_type := (cast(*B.Type_Reference) array.template_type_params[0].expression).type;

        assert(array_type.array_element_type == generator.type_def_s32 && array_type.array_element_count == 3);
        callback_type := (cast(*B.Type_Reference) callback.template_type_params[0].expression).type;
        signature := callback_type.pointer_to.type_of_function;

        assert(signature && signature.arguments.count == 1 && signature.return_type == generator.type_def_void);
        assert(signature.arguments[0].type == generator.type_def_s32 && signature.arguments[0].location.file == null);
        reference_type := (cast(*B.Type_Reference) reference.template_type_params[0].expression).type;

        assert(reference_type.pointer_to == generator.type_def_s32 && (reference_type.flags & .REFERENCE) != 0);
        alias_type := (cast(*B.Type_Reference) alias.template_type_params[0].expression).type;

        assert(alias_type.type_of_typedef == named);
        assert((cast(*B.Type_Reference) decltype_record.template_type_params[0].expression).type == generator.type_def_s32);
        assert((cast(*B.Type_Reference) signed.template_type_params[0].expression).type == generator.type_def_s128);
        assert((cast(*B.Type_Reference) unsigned.template_type_params[0].expression).type == generator.type_def_u128);
        assert(generator.type_def_s128.hardcoded_jai_string == "S128" && generator.type_def_u128.hardcoded_jai_string == "U128");
        wide_array := B.CType.{ array_element_type = generator.type_def_u128, array_element_count = 3 };
        opaque := B.CType.{ hardcoded_jai_string = "OpaqueValue", size = 16, alignment = 16 };

        assert(B.get_primitive_or_array_primitive_type_size(*wide_array, null) == 48);
        assert(B.get_primitive_or_array_primitive_type_size(*opaque, null) == -1);
        assert(qualified.type.pointer_to.hardcoded_jai_string == "S128");
        assert(qualified.type.pointer_to.hardcoded_jai_string.data != generator.type_def_s128.hardcoded_jai_string.data);
        qualified.type.pointer_to.hardcoded_jai_string.data[0] = #char "X";
        assert(generator.type_def_s128.hardcoded_jai_string == "S128" && generator.type_def_u128.hardcoded_jai_string == "U128");

        for entry: generator.declarations_by_cursor.entries {
            if entry.hash < H.FIRST_VALID_HASH continue;
            assert(B.clang_Cursor_getTranslationUnit(entry.key) == unit);
        }

        saved_signed := generator.type_def_s128;

        B.init_generator(*generator, options);
        assert(generator.type_def_s128 == saved_signed && generator.type_def_s128.hardcoded_jai_string == "S128");
        B.deinit_generator(*generator);
    }

    assert(tracker.live == 0);
    exercise_configuration();
    print("ok\n");
}
