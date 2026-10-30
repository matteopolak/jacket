// Reads the next token from `line` and removes it, with any spaces before it, from `line`.
//
// A token is a run of bytes up to the next space or delimiter. A delimiter that starts a token is a
// token of one byte. The result is the token and `true`; when only spaces are left, it is an empty
// string and `false` and `line` is not changed. The token points into the original text.
//
// ```jai
// line := "  name =value";
// token, ok := parse_token(*line);
// assert(ok && token == "name" && line == " =value");
//
// token, ok = parse_token(*line);
// assert(ok && token == "=" && line == "value");
// ```
//
// - delimiters: bytes that end a token. One that starts a token is a token by itself.
// - spaces: bytes that are skipped in front of a token and that end one. They never appear in a token.
parse_token :: (line: *string, delimiters := DEFAULT_DELIMITERS, spaces := DEFAULT_SPACES) -> value: string, success: bool {
    rest := trim_left(line.*, spaces);
    if rest.count == 0 return "", false;

    count: s64 = 1;

    if !is_any(rest[0], delimiters) {
        while count < rest.count && !is_any(rest[count], delimiters) && !is_any(rest[count], spaces) count += 1;
    }

    line.* = slice(rest, count, rest.count);

    return slice(rest, 0, count), true;
}

// Reads the next token as `parse_token` does and returns it with whether it is equal to `match`.
// The token is removed from `line` whether or not it matches.
//
// ```jai
// line := "if (x)";
// _, matched := match_token(*line, "if");
// assert(matched && line == " (x)");
// ```
match_token :: (line: *string, match: string, delimiters := DEFAULT_DELIMITERS, spaces := DEFAULT_SPACES) -> value: string, success: bool {
    token, ok := parse_token(line, delimiters, spaces);
    return token, ok && token == match;
}

// Like `match_token`, with ASCII letters matched in either case.
//
// ```jai
// line := "IF x";
// _, matched := match_token_nocase(*line, "if");
// assert(matched);
// ```
match_token_nocase :: (line: *string, match: string, delimiters := DEFAULT_DELIMITERS, spaces := DEFAULT_SPACES) -> value: string, success: bool {
    token, ok := parse_token(line, delimiters, spaces);
    return token, ok && equal_nocase(token, match);
}

// Reads the next token and reads it as `true` or `false`, in either case. The second result is `false`
// when the token is anything else or there is none; the token is removed from `line` anyway.
//
// ```jai
// line := "True rest";
// value, ok := parse_bool(*line);
// assert(ok && value && line == " rest");
// ```
parse_bool :: (line: *string, delimiters := DEFAULT_DELIMITERS, spaces := DEFAULT_SPACES) -> value: bool, success: bool {
    token, ok := parse_token(line, delimiters, spaces);
    if !ok return false, false;
    if equal_nocase(token, "true") return true, true;
    if equal_nocase(token, "false") return false, true;

    return false, false;
}

// Reads the next token and looks it up by name among the values of the enum type `T`. The second result is
// `false` when no value has that name, and then the value is zero. The name is case sensitive.
//
// ```jai
// Color :: enum { RED; GREEN; }
//
// line := "GREEN;";
// value, ok := parse_enum(*line, Color);
// assert(ok && value == .GREEN && line == ";");
// ```
parse_enum :: (line: *string, $T: Type, delimiters := DEFAULT_DELIMITERS, spaces := DEFAULT_SPACES) -> value: T, success: bool {
    token, ok := parse_token(line, delimiters, spaces);
    if !ok return cast(T) 0, false;

    info := cast(*Type_Info_Enum) type_info(T);
    for name, i: info.names if name == token return cast(T) info.values[i], true;

    return cast(T) 0, false;
}

// Reads the next token as a whole number of type `T` (`int` by default). Only space and tab are skipped
// in front of it unless you pass `spaces`.
//
// The second result is `false` when the token is not entirely a number, or there is none, or the
// number does not fit `T` (including a minus sign for an unsigned `T`). The value is then `0`.
//
// ```jai
// line := " 42;7";
// value, ok := parse_int(*line);
// assert(ok && value == 42);
//
// big := "257";
// small, small_ok := parse_int(*big, u8);
// assert(!small_ok && small == 0);
// ```
parse_int :: (line: *string, $T: Type = int, spaces := " \t", delimiters := DEFAULT_DELIMITERS) -> value: T, success: bool {
    token, ok := parse_token(line, delimiters, spaces);
    if !ok return cast(T) 0, false;

    value, valid, rest := string_to_int(token, 10, T);
    if !valid || rest.count != 0 return cast(T) 0, false;

    return value, true;
}

// Reads the next token as a decimal number. The second result is `false` when the token is not entirely a
// number, or there is none.
//
// ```jai
// line := "2.5 rest";
// value, ok := parse_float(*line);
// assert(ok && value == 2.5);
// ```
parse_float :: (line: *string, delimiters := DEFAULT_DELIMITERS, spaces := DEFAULT_SPACES) -> value: float, success: bool {
    token, ok := parse_token(line, delimiters, spaces);
    if !ok return 0, false;
    value, valid, rest := string_to_float(token);
    return value, valid && rest.count == 0;
}
