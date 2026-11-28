#load "../internal/developer-tools/generator-macro-expressions.jai";

// Returns the spelling of `token` from the active translation unit, or an empty string when there is none.
//
// The string points into memory that libclang releases when the calling scope ends, so it must be copied to
// outlive that scope.
get_scoped_token_string :: (token: CXToken) -> string #expand {
    unit := gm_token_unit();
    if !unit return "";

    spelling := clang_getTokenSpelling(unit, token);

    `defer clang_disposeString(spelling);

    return to_string(clang_getCString(spelling));
}

// Returns the spellings of the tokens of `value`, joined with single spaces.
//
// Returns an empty string when the tokens or the translation unit are not available. The caller frees the
// result.
raw_string_from_macro_tokens :: (value: Macro_Value) -> string {
    return gme_raw_tokens(value);
}

// Returns whether the tokens from `start_index` spell exactly the strings in `expected`, in order.
//
// Returns false when fewer tokens remain. A negative `start_index` logs an error and gives false. An empty
// `expected` matches.
expect_tokens :: (tokens: [] CXToken, start_index: int, expected: ..string) -> bool {
    return gme_expect_tokens(tokens, start_index, expected);
}

// Returns the expression for the last child of `cur`, or null.
//
// Returns null when the cursor is null, has no children, or its last child has no expression form.
convert_children_to_expression :: (cur: CXCursor) -> *Expression {
    return gme_final_child(cur);
}

// Returns an aggregate expression for a constructor or functional-cast call at `cur`, or null.
//
// A leading type reference is skipped and the remaining children must be expressions. Member and declaration
// references, calls, lambdas, template and namespace references and address-of are not represented: an error
// is logged and null returned. With `expect_type_ref`, null pointers, unary expressions such as `sizeof`, and
// a cast of a literal to a scalar type give a literal instead.
parse_call_expression :: (cur: CXCursor, expect_type_ref: bool) -> *Expression {
    return gme_call_expression(cur, expect_type_ref);
}

// Returns the Jai expression for the replacement tokens of the macro `value`, or an empty string.
//
// Converts C literals, replaces limit macros such as `INT_MAX` (see `TOKENS_TO_REPLACE`), converts casts and
// writes identifiers by their output names. `_enum` is the enum the value belongs to, whose values are then
// written without qualification; `name` is the macro's own name, and a value that refers to itself is
// rejected. Returns an empty string, and with `log_unsupported` logs the reason, when a token cannot be
// converted. The caller frees a non-empty result.
string_from_macro_value :: (value: Macro_Value, _enum: *Enum = null, name: string = "") -> string {
    text, valid, reason := gme_macro_text(value, _enum, name, true);
    if !valid {
        gme_unsupported(name, reason);
        return "";
    }

    return text;
}

// Decides whether the macro definition `decl` is kept, by setting its `macro_value_result` to `.KEEP` or
// `.DISCARD`.
//
// Does nothing unless the result is still `.NEEDS_CHECKING`. The macro is discarded when libclang cannot
// evaluate it to an integer, float, string or null, or its spelling uses syntax the generator does not
// convert. A value that has no Jai spelling but evaluates to a number, such as one using `sizeof`, is kept as
// that evaluated constant, and so is an unsigned value above the signed 64-bit range. Logs an error when
// `decl` is not a macro with replacement tokens.
check_macro_value :: (decl: *Declaration) {
    gme_check_macro(decl);
}

#scope_file
#import "Basic";
