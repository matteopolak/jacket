#import "Basic";
#import "String";
B :: #import "Bindings_Generator";
#load "../../../../target/developer-tools-cpp-visitor-oracle.jai";

Cursors :: struct {
    rows: [512] B.CXCursor;
    count: s64;
    overflow: bool;
    stop: bool;
}

retain :: (state: *Cursors, cursor: B.CXCursor) #c_call {
    if state.count == state.rows.count {
        state.overflow = true;
        return;
    }

    state.rows[state.count] = cursor;
    state.count += 1;
}

visit :: (cursor: B.CXCursor, _parent: B.CXCursor, data: B.CXClientData) -> B.CXChildVisitResult #c_call {
    retain(cast(*Cursors) data, cursor);
    return .Recurse;
}

visit_field :: (cursor: B.CXCursor, data: B.CXClientData) -> B.CXVisitorResult #c_call {
    state := cast(*Cursors) data;
    retain(state, cursor);
    return ifx state.stop then .Break else .Continue;
}

main :: () {
    index := B.clang_createIndex(0, 0);

    assert(index != null);
    defer B.clang_disposeIndex(index);
    filename := to_c_string(tprint("%bindings-clang-cpp.h", #filepath));

    defer free(filename);
    args := string.["-x", "c++", "-std=c++17", "-Wall", "-Wextra", "-Werror"];
    native_args: [..] *u8;

    defer {
        for argument: native_args free(argument);
        array_free(native_args);
    }

    for argument: args array_add(*native_args, to_c_string(argument));

    unit: B.CXTranslationUnit;

    assert(B.clang_parseTranslationUnit2(index, filename, native_args.data, cast(s32) native_args.count, null, 0, 0, *unit) == .Success);
    defer B.clang_disposeTranslationUnit(unit);
    assert(B.clang_getNumDiagnostics(unit) == 0);
    state: Cursors;

    B.clang_visitChildren(B.clang_getTranslationUnitCursor(unit), visit, *state);
    assert(!state.overflow);
    base, derived: B.CXCursor;
    constructors, default_constructor, copy_constructor, move_constructor, explicit_constructor: s64;
    copy_assignment, move_assignment, measures, utilities, disabled, destructors, inline_namespaces, tls, entries: s64;

    for i: 0..state.count - 1 {
        cursor := state.rows[i];
        spelling := B.clang_getCursorSpelling(cursor);

        defer B.clang_disposeString(spelling);
        name := to_string(B.clang_getCString(spelling));

        if cursor.kind == .ClassDecl {
            assert(B.clang_getCursorLanguage(cursor) == .CPlusPlus);
            assert(B.clang_getCursorAvailability(cursor) == .Available);
            assert(B.clang_getCursorLinkage(cursor) == .External);
            assert(B.clang_getCursorVisibility(cursor) == .Default);
            assert(B.clang_Cursor_isAnonymous(cursor) == 0 && B.clang_Cursor_isAnonymousRecordDecl(cursor) == 0);

            if name == "OriginalBase" {
                base = cursor;
                assert(B.clang_CXXRecord_isAbstract(cursor) != 0);
            }

            if name == "OriginalDerived" {
                derived = cursor;
                assert(B.clang_CXXRecord_isAbstract(cursor) == 0);
            }
        }

        if cursor.kind == .Constructor {
            constructors += 1;
            if B.clang_CXXConstructor_isDefaultConstructor(cursor) != 0 default_constructor += 1;
            if B.clang_CXXConstructor_isCopyConstructor(cursor) != 0 copy_constructor += 1;
            if B.clang_CXXConstructor_isMoveConstructor(cursor) != 0 move_constructor += 1;

            if B.clang_CXXMethod_isExplicit(cursor) != 0 {
                explicit_constructor += 1;
                assert(B.clang_CXXConstructor_isConvertingConstructor(cursor) == 0);
            } else assert(B.clang_CXXMethod_isDefaulted(cursor) != 0);
        }

        if cursor.kind == .Destructor {
            destructors += 1;
            assert(B.clang_CXXMethod_isVirtual(cursor) != 0 && B.clang_CXXMethod_isDefaulted(cursor) != 0);
        }

        if cursor.kind == .CXXMethod {
            if name == "measure" {
                measures += 1;
                assert(B.clang_CXXMethod_isConst(cursor) != 0 && B.clang_CXXMethod_isVirtual(cursor) != 0);
                assert(B.clang_CXXMethod_isStatic(cursor) == 0);
                parent := B.clang_getCursorSemanticParent(cursor);

                assert((B.clang_CXXMethod_isPureVirtual(cursor) != 0) == (B.clang_CXXRecord_isAbstract(parent) != 0));
                mangling := B.clang_Cursor_getMangling(cursor);

                defer B.clang_disposeString(mangling);
                assert(contains(to_string(B.clang_getCString(mangling)), "measure"));
            }

            if name == "utility" {
                utilities += 1;
                assert(B.clang_CXXMethod_isStatic(cursor) != 0 && B.clang_CXXMethod_isConst(cursor) == 0);
            }

            if name == "disabled" {
                disabled += 1;
                assert(B.clang_CXXMethod_isDeleted(cursor) != 0);
            }

            if name == "operator=" {
                if B.clang_CXXMethod_isCopyAssignmentOperator(cursor) != 0 copy_assignment += 1;
                if B.clang_CXXMethod_isMoveAssignmentOperator(cursor) != 0 move_assignment += 1;
                assert(B.clang_CXXMethod_isDefaulted(cursor) != 0);
            }
        }

        if cursor.kind == .Namespace {
            inline_namespaces += 1;
            assert(B.clang_Cursor_isInlineNamespace(cursor) != 0);
        }

        if cursor.kind == .VarDecl && name == "original_tls" {
            tls += 1;
            assert(B.clang_getCursorTLSKind(cursor) == .Dynamic);
        }

        if cursor.kind == .FunctionDecl && name == "original_c_entry" {
            entries += 1;
            assert(B.clang_getCursorLinkage(cursor) == .External && B.clang_getCursorTLSKind(cursor) == .None);
            mangling := B.clang_Cursor_getMangling(cursor);

            defer B.clang_disposeString(mangling);
            #if OS == .MACOS assert(to_string(B.clang_getCString(mangling)) == "_original_c_entry");
            else assert(to_string(B.clang_getCString(mangling)) == "original_c_entry");
        }
    }

    assert(constructors == 4 && default_constructor == 1 && copy_constructor == 1 && move_constructor == 1 && explicit_constructor == 1);
    assert(copy_assignment == 1 && move_assignment == 1 && measures == 2 && utilities == 1 && disabled == 1 && destructors == 1);
    assert(inline_namespaces == 1 && tls == 1 && entries == 1);
    assert(B.clang_getCursorKind(base) == .ClassDecl && B.clang_getCursorKind(derived) == .ClassDecl);
    fields, bases, methods: Cursors;
    type := B.clang_getCursorType(derived);

    assert(B.clang_Type_visitFields(type, visit_field, *fields) == SDK_Visitor_Results[0]);
    assert(fields.count == 1 && fields.rows[0].kind == .FieldDecl && !fields.overflow);
    assert(B.clang_visitCXXBaseClasses(type, visit_field, *bases) == SDK_Visitor_Results[2]);
    assert(bases.count == 1 && bases.rows[0].kind == .CXXBaseSpecifier && !bases.overflow);
    assert(B.clang_visitCXXMethods(type, visit_field, *methods) == SDK_Visitor_Results[4]);
    assert(methods.count >= 5 && !methods.overflow);
    stopped: Cursors;
    stopped.stop = true;
    assert(B.clang_visitCXXMethods(type, visit_field, *stopped) == SDK_Visitor_Results[6] && stopped.count == 1);
    assert(fields.count == SDK_Visitor_Results[1] && bases.count == SDK_Visitor_Results[3]);
    assert(methods.count == SDK_Visitor_Results[5] && stopped.count == SDK_Visitor_Results[7]);
    print("ok\n");
}
