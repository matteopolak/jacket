#import "Basic";
B :: #import "Bindings_Generator";

mode: *B.Enum;
require_lookup: bool;
values_seen: int;
errors: int;

capture_error :: (_message: string, _data: *void, info: Log_Info) {
    if info.common_flags & .ERROR errors += 1;
}

lookup_boundaries :: (argument: *B.Declaration) {
    local_enum := B.Enum.{ kind = .ENUM, name = "Mode", output_name = "Local_Mode", type = mode.type };
    local_scope := B.Namespace.{ kind = .NAMESPACE, name = "Local" };

    array_add(*local_scope.declarations, cast(*B.Declaration) *local_enum);
    defer array_free(local_scope.declarations);
    local_function := B.Function.{ kind = .FUNCTION, parent = cast(*B.Declaration) *local_scope };
    local_argument := B.Declaration.{ parent = cast(*B.Declaration) *local_function };

    assert(B.find_parent_enum(*local_argument, "Mode") == *local_enum);
    assert(B.find_parent_enum(*local_argument, "Local_Mode") == *local_enum);
    assert(B.find_parent_enum(*local_argument, "Public_Mode") == mode);
    assert(!B.find_parent_enum(argument, "Absent_Mode"));

    push_context {
        context.logger = capture_error;
        before := errors;

        local_function.parent = cast(*B.Declaration) *local_function;
        assert(!B.find_parent_enum(*local_argument, "Mode") && errors == before + 1);
        saved := context.generator.global_scope.members.count;

        context.generator.global_scope.members.count = 0x7fffffffffffffff;
        before = errors;
        assert(!B.find_parent_enum(argument, "Mode") && errors == before + 1);
        context.generator.global_scope.members.count = saved;
        assert(!context._bindings_generator_session.failed);
    }

    other: B.Generator;

    push_context {
        // Existing ledger belongs to the outer owner, not this unrelated record.
        context.generator = *other;
        assert(!B.find_parent_enum(argument, "Mode"));
    }

    assert(B.find_parent_enum(argument, "Mode") == mode);
}

inspect :: (decl: *B.Declaration, parent: *B.Declaration) -> B.Declaration_Visit_Result {
    if decl.kind == .ENUM && decl.name == "Mode" {
        mode = cast(*B.Enum) decl;
        assert(mode.type && mode.enumerates.count == 2);
        native := B.clang_getEnumDeclIntegerType(B.clang_getCursorDefinition(find_native_enum()));

        assert(mode.type.size == B.clang_Type_getSizeOf(native));
        assert(native.kind == .UInt && mode.type.size == 4 && (mode.type.number_flags & .SIGNED) == 0);
        print("enum bytes=% signed=% native-kind=%\n", mode.type.size, (mode.type.number_flags & .SIGNED) != 0, native.kind);

        for value: mode.enumerates {
            literal := cast(*B.Literal) value.expression;
            assert(literal && literal.literal_kind == .UNSIGNED_INTEGER);
            assert((value.name == "MODE_ADD" && literal.uint_value == 3) || (value.name == "MODE_SUBTRACT" && literal.uint_value == 7));
            print("value % kind=% signed=% unsigned=%\n", value.name, literal.literal_kind, literal.int_value, literal.uint_value);
        }

        decl.output_name = "Public_Mode";
    }

    if parent && parent.kind == .FUNCTION && decl.name == "value" {
        assert(mode && decl.parent == parent && parent.parent == null);
        decl.output_name = "number";
        source := B.find_parent_enum(decl, "Mode");
        renamed := B.find_parent_enum(decl, "Public_Mode");

        print("argument parent=% stored-parent-valid=% global-parent-null=% source-found=% output-found=%\n", parent.name, decl.parent == parent, parent.parent == null, source != null, renamed != null);
        values_seen += 1;

        if require_lookup {
            assert(source == mode && renamed == mode);
            lookup_boundaries(decl);
            assert(B.change_type_to_enum(decl, "Mode"));
            assert(decl.type != mode.type && decl.type.type_of_enum == mode && decl.type.size == mode.type.size && decl.type.number_flags == mode.type.number_flags);
            free(decl.type);
            decl.type = context.generator.type_def_s32;
        }
    }

    return .RECURSE;
}

native_enum: B.CXCursor;

capture_native_enum :: (cur: B.CXCursor, _parent: B.CXCursor, _data: B.CXClientData) -> B.CXChildVisitResult #c_call {
    spelling := B.clang_getCursorSpelling(cur);
    defer B.clang_disposeString(spelling);
    if cur.kind == .EnumDecl && to_string(B.clang_getCString(spelling)) == "Mode" native_enum = cur;
    return .Continue;
}

find_native_enum :: () -> B.CXCursor {
    B.clang_visitChildren(B.clang_getTranslationUnitCursor(context.generator.translation_unit), capture_native_enum, null);
    return native_enum;
}

main :: () {
    args := get_command_line_arguments();

    defer array_free(args);
    require_lookup = args.count > 1 && args[1] == "require";
    options: B.Generate_Bindings_Options;

    array_add(*options.source_files, tprint("%bindings-input.h", #filepath));
    defer array_free(options.source_files);
    options.visitor = inspect;
    options.add_generator_command = false;
    builder, ok := B.generate_bindings(options);

    defer free_buffers(*builder);
    assert(ok && mode && values_seen == 1);
    detached: B.Declaration;

    assert(!B.find_parent_enum(*detached, "Mode"));
    print("ok\n");
}
