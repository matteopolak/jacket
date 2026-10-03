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

match_token :: (line: *string, match: string, delimiters := DEFAULT_DELIMITERS, spaces := DEFAULT_SPACES) -> value: string, success: bool {
    token, ok := parse_token(line, delimiters, spaces);
    return token, ok && token == match;
}

match_token_nocase :: (line: *string, match: string, delimiters := DEFAULT_DELIMITERS, spaces := DEFAULT_SPACES) -> value: string, success: bool {
    token, ok := parse_token(line, delimiters, spaces);
    return token, ok && equal_nocase(token, match);
}

parse_bool :: (line: *string, delimiters := DEFAULT_DELIMITERS, spaces := DEFAULT_SPACES) -> value: bool, success: bool {
    token, ok := parse_token(line, delimiters, spaces);
    if !ok return false, false;
    if equal_nocase(token, "true") return true, true;
    if equal_nocase(token, "false") return false, true;

    return false, false;
}

parse_enum :: (line: *string, $T: Type, delimiters := DEFAULT_DELIMITERS, spaces := DEFAULT_SPACES) -> value: T, success: bool {
    token, ok := parse_token(line, delimiters, spaces);
    if !ok return cast(T) 0, false;

    info := cast(*Type_Info_Enum) type_info(T);
    for name, i: info.names if name == token return cast(T) info.values[i], true;

    return cast(T) 0, false;
}

parse_int :: (line: *string, $T: Type = int, spaces := " \t", delimiters := DEFAULT_DELIMITERS) -> value: T, success: bool {
    token, ok := parse_token(line, delimiters, spaces);
    if !ok return cast(T) 0, false;
    value, valid, rest := string_to_int(token);
    return cast,truncate(T) value, valid && rest.count == 0;
}

parse_float :: (line: *string, delimiters := DEFAULT_DELIMITERS, spaces := DEFAULT_SPACES) -> value: float, success: bool {
    token, ok := parse_token(line, delimiters, spaces);
    if !ok return 0, false;
    value, valid, rest := string_to_float(token);
    return value, valid && rest.count == 0;
}
