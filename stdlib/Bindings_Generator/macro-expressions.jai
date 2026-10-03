#load "../internal/developer-tools/generator-macro-expressions.jai";

get_scoped_token_string :: (token: CXToken) -> string #expand {
    unit := gm_token_unit();
    if !unit return "";

    spelling := clang_getTokenSpelling(unit, token);

    `defer clang_disposeString(spelling);

    return to_string(clang_getCString(spelling));
}

raw_string_from_macro_tokens :: (value: Macro_Value) -> string {
    return gme_raw_tokens(value);
}

expect_tokens :: (tokens: [] CXToken, start_index: int, expected: ..string) -> bool {
    return gme_expect_tokens(tokens, start_index, expected);
}

convert_children_to_expression :: (cur: CXCursor) -> *Expression {
    return gme_final_child(cur);
}

parse_call_expression :: (cur: CXCursor, expect_type_ref: bool) -> *Expression {
    return gme_call_expression(cur, expect_type_ref);
}

string_from_macro_value :: (value: Macro_Value, _enum: *Enum = null, name: string = "") -> string {
    text, valid, reason := gme_macro_text(value, _enum, name, true);
    if !valid {
        gme_unsupported(name, reason);
        return "";
    }

    return text;
}

check_macro_value :: (decl: *Declaration) {
    gme_check_macro(decl);
}

#scope_file
#import "Basic";
