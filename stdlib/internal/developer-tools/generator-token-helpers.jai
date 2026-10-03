#scope_module
#import "Basic";

gm_token_unit :: () -> CXTranslationUnit {
    generator := gm_active_generator();
    if !generator return null;

    if !generator.translation_unit {
        log_error("Bindings_Generator: token operation needs a live translation unit\n");
        if context._bindings_generator_session context._bindings_generator_session.failed = true;
        return null;
    }

    return generator.translation_unit;
}

gm_tokens_valid :: (tokens: [] CXToken) -> bool {
    if tokens.count < 0 || (tokens.count > 0 && !tokens.data) {
        log_error("Bindings_Generator: malformed token sequence\n");
        if context._bindings_generator_session context._bindings_generator_session.failed = true;
        return false;
    }

    return true;
}

gm_is_token :: (token: CXToken, expected: string) -> bool {
    unit := gm_token_unit();
    if !unit return false;

    spelling := clang_getTokenSpelling(unit, token);

    defer clang_disposeString(spelling);

    return to_string(clang_getCString(spelling)) == expected;
}

gm_count_asterisks :: (tokens: [] CXToken) -> int {
    if !gm_tokens_valid(tokens) return -1;
    if tokens.count == 0 return 0;

    unit := gm_token_unit();
    if !unit return -1;

    result := 0;

    for token: tokens {
        spelling := clang_getTokenSpelling(unit, token);
        star := to_string(clang_getCString(spelling)) == "*";

        clang_disposeString(spelling);
        if !star break;

        result += 1;
    }

    return result;
}

gm_closing_paren :: (tokens: [] CXToken, start: int) -> int {
    if !gm_tokens_valid(tokens) return -1;

    if start < 0 || start > tokens.count {
        log_error("Bindings_Generator: parenthesis search requires a bounded start index\n");
        if context._bindings_generator_session context._bindings_generator_session.failed = true;
        return -1;
    }

    if start == tokens.count return -1;

    unit := gm_token_unit();
    if !unit return -1;

    depth := 0;

    for i: start..tokens.count - 1 {
        spelling := clang_getTokenSpelling(unit, tokens[i]);
        text := to_string(clang_getCString(spelling));

        closes := text == ")" && depth == 0;

        if text == "(" depth += 1;
        if text == ")" && !closes depth -= 1;

        clang_disposeString(spelling);
        if closes return i;
    }

    return -1;
}

gm_macro_values_equal :: (a: Macro_Value, b: Macro_Value) -> bool {
    if !gm_tokens_valid(a.tokens) || !gm_tokens_valid(b.tokens) return false;
    if a.tokens.count != b.tokens.count return false;
    if a.tokens.count == 0 return true;

    unit := gm_token_unit();
    if !unit return false;

    for token, i: a.tokens {
        x := clang_getTokenSpelling(unit, token);
        y := clang_getTokenSpelling(unit, b.tokens[i]);
        equal := to_string(clang_getCString(x)) == to_string(clang_getCString(y));

        clang_disposeString(x);
        clang_disposeString(y);
        if !equal return false;
    }

    return true;
}

gm_macro_literal :: (value: Macro_Value, location: Location) -> *Literal {
    if !gm_tokens_valid(value.tokens) return null;

    session := gm_current_session();
    if !session return null;
    if value.tokens.count > 0 && !gm_token_unit() return null;

    result := GM_Sessions.new_owned(session, Literal);
    result.literal_kind = .MACRO;
    result.location = location;
    result.macro_value.tokens = GM_Sessions.copy_array(session, value.tokens);

    return result;
}
