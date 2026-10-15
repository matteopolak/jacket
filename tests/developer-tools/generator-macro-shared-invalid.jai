// Independent safety policy for malformed views and unavailable context.
// These negative cases do not extend the reviewed supported input domain.
#import "Basic";
B :: #import "Bindings_Generator";

diagnostics: s64;

record_log :: (_message: string, _data: *void, _info: Log_Info) {
    diagnostics += 1;
}

main :: () {
    push_context, defer_pop;
    context.logger = record_log;
    empty: [] B.CXToken;
    exact_empty := B.expect_tokens(empty, 0);
    insufficient := B.expect_tokens(empty, 0, "missing");

    assert(exact_empty && !insufficient);
    malformed: [] B.CXToken = .{ 1, null };
    bad_match := B.expect_tokens(malformed, 0, "missing");

    assert(!bad_match && diagnostics > 0);
    negative := B.expect_tokens(empty, -1);

    assert(!negative);
    count_before := diagnostics;
    bad_raw := B.raw_string_from_macro_tokens(.{ tokens = malformed });

    assert(bad_raw.count == 0 && diagnostics > count_before);
    owned_empty := B.raw_string_from_macro_tokens(.{});

    defer free(owned_empty);
    assert(owned_empty.count == 0);
    null_cursor := B.clang_getNullCursor();
    child := B.convert_children_to_expression(null_cursor);
    call := B.parse_call_expression(null_cursor, true);

    assert(child == null && call == null);
    print("invalid ok\n");
}
