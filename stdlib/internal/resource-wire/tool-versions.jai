parse_version :: (text: string, fields: s32, prefix: string = "") -> bool, [4] u32 {
    result: [4] u32;
    if fields < 1 || fields > 4 || text.count <= prefix.count return false, result;
    for byte, index: prefix if text[index] != byte return false, result;

    cursor := prefix.count;

    for part: 0..fields - 1 {
        start := cursor;
        value: u64;

        while cursor < text.count && text[cursor] >= 48 && text[cursor] <= 57 {
            value = value * 10 + text[cursor] - 48;
            if value > 0xffffffff return false, result;
            cursor += 1;
        }

        if cursor == start return false, result;

        result[part] = cast(u32) value;

        if part + 1 < fields {
            if cursor >= text.count || text[cursor] != 46 return false, result;
            cursor += 1;
        }
    }

    return cursor == text.count, result;
}

is_later :: (left: [4] u32, right: [4] u32) -> bool {
    for value, index: left {
        if value != right[index] return value > right[index];
    }

    return false;
}
