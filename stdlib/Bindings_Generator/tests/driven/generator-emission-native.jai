#import "Basic";
#import "String";
B :: #import "Bindings_Generator";

Debug_Probe :: struct {
    descendants: int;
    maximum_depth: int;
    nested: bool;
}

capture :: (text: string, data: *void, _info: Log_Info) {
    probe := cast(*Debug_Probe) data;
    probe.descendants += 1;
    depth := 0;
    while depth < text.count && text[depth] == #char " " depth += 1;

    probe.maximum_depth = max(probe.maximum_depth, depth);
    if text == "  FieldDecl nested\n" probe.nested = true;

    assert(!contains(text, "TranslationUnit "));
}

inspect :: (cur: B.CXCursor, _parent: B.CXCursor, data: *void) -> B.CXChildVisitResult #c_call {
    push_context {
        raw_name := B.clang_getCursorSpelling(cur);

        defer B.clang_disposeString(raw_name);
        pointer := B.clang_getCString(raw_name);
        expected := ifx pointer then to_string(pointer) else "";
        name := B.get_name_for_cursor(cur);

        assert(name == (ifx contains(expected, " at ") then "" else expected));
        free(name);
    }

    if cur.kind != .VarDecl return .Continue;

    count := cast(*int) data;
    count.* += 1;

    push_context {
        first := B.get_name_for_cursor(cur);
        second := B.get_name_for_cursor(cur);

        assert(first == second && first.count > 0 && first.data != second.data);
        first.data[0] = #char "X";
        assert(first != second);
        free(first);
        location: B.Location;

        B.fill_location(*location, cur);
        print("VarDecl % %:%\n", second, location.line, location.column);
        free(second);
        // Native ABI groundwork for the pending comment-fill contract. This
        // probes official raw-comment/range APIs, not an invented fill policy.
        raw := B.clang_Cursor_getRawCommentText(cur);

        defer B.clang_disposeString(raw);
        pointer := B.clang_getCString(raw);
        comment := ifx pointer then to_string(pointer) else "";
        range := B.clang_Cursor_getCommentRange(cur);
        line, column: u32;

        B.clang_getSpellingLocation(B.clang_getRangeStart(range), null, *line, *column, null);
        if count.* == 1 assert(comment == "/// Alpha documentation." && line == 1 && column == 1);
        if count.* == 2 assert(comment == "///< Beta documentation." && line == 3 && column == 11);
        if count.* >= 3 assert(comment == "" && B.clang_Range_isNull(range) != 0);

        print("comment [%] %:%\n", comment, line, column);
        print("definition=% canonical=% self_definition=%\n", B.clang_isCursorDefinition(cur),
              B.clang_equalCursors(cur, B.clang_getCanonicalCursor(cur)),
              B.clang_equalCursors(cur, B.clang_getCursorDefinition(cur)));
        existing: B.Comment = .{ text = "retained", location = .{ line = 99 } };

        B.fill_comment(*existing, cur, location);

        if count.* <= 2 {
            assert(existing.text == comment && existing.text.data != pointer);
            assert(existing.location.line == (ifx count.* == 1 then 1 else 99));
            free(existing.text);
        } else {
            assert(existing.text == "retained" && existing.location.line == 99);
        }

        refreshed: B.Location;
        fresh_comment: B.Comment = .{ text = "retained" };

        B.fill_location_and_comment_if_needed(*refreshed, *fresh_comment, cur);
        assert(refreshed.file == location.file && refreshed.line == location.line);

        if count.* <= 2 {
            assert(fresh_comment.text == comment);
            free(fresh_comment.text);
        } else assert(fresh_comment.text == "retained");

        fresh_comment.text = "preserve";
        fresh_comment.location.line = 77;
        refreshed.line = 888;
        B.fill_location_and_comment_if_needed(*refreshed, *fresh_comment, cur);

        if count.* == 5 {
            assert(refreshed.line == location.line);
            if count.* <= 2 free(fresh_comment.text);
            else assert(fresh_comment.text == "preserve" && fresh_comment.location.line == 77);
        } else assert(refreshed.line == 888 && fresh_comment.text == "preserve" && fresh_comment.location.line == 77);

        before: B.Comment;

        B.fill_comment(*before, cur, .{ line = 0 });
        assert(before.location.line == 0);
        if before.text.count free(before.text);
    }

    return .Continue;
}

main :: () {
    filename := to_c_string(tprint("%generator-emission.h", #filepath));

    defer free(filename);
    index := B.clang_createIndex(0, 0);

    assert(index != null);
    defer B.clang_disposeIndex(index);
    unit: B.CXTranslationUnit;

    assert(B.clang_parseTranslationUnit2(index, filename, null, 0, null, 0, 0, *unit) == .Success);
    defer B.clang_disposeTranslationUnit(unit);
    assert(B.clang_getNumDiagnostics(unit) == 0);
    count := 0;

    B.clang_visitChildren(B.clang_getTranslationUnitCursor(unit), inspect, *count);
    assert(count == 5);
    probe: Debug_Probe;

    push_context {
        context.logger = capture;
        context.logger_data = *probe;
        B.debug_print_cursor(B.clang_getTranslationUnitCursor(unit));
    }

    assert(probe.nested);
    print("debug descendants=% depth=%\n", probe.descendants, probe.maximum_depth);
}
