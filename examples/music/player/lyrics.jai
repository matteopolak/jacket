Lyric :: struct {
    milliseconds: s64;
    text: string;
    sequence: s64;
}

// Multiple timestamps per line, fractional seconds, CRLF, metadata and global offset.
// Invalid timestamps are ignored; the last offset tag applies to the entire file.
parse_lyrics :: (text: string) -> [..] Lyric {
    result: [..] Lyric;
    offset: s64;

    for split(text, "\n",, temp) {
        line := trim(it);
        if begins_with(line, "[offset:") && ends_with(line, "]") {
            number := slice(line, 8, line.count - 9);
            value, ok := parse_int(*number);
            if ok && number.count == 0 offset = value;
            continue;
        }

        times: [..] s64;

        defer array_free(times);

        while begins_with(line, "[") {
            end := find_index_from_left(line, #char "]");
            if end < 0 break;

            token := slice(line, 1, end - 1);
            stamp, ok := timestamp(token);

            if ok array_add(*times, stamp);

            line = slice(line, end + 1, line.count - end - 1);
        }

        for times array_add(*result, .{ it, copy_string(line), result.count });
    }

    for *result it.milliseconds = max(cast(s64) 0, it.milliseconds + offset);

    quick_sort(result, lyric_order);

    return result;
}

free_lyrics :: (lyrics: *[..] Lyric) {
    for lyrics.* free(it.text);
    array_free(lyrics.*);
}

lyric_at :: (lyrics: [] Lyric, milliseconds: s64) -> s64 {
    result: s64 = -1;

    for lyrics {
        if it.milliseconds > milliseconds break;
        result = it_index;
    }

    return result;
}

sidecar_path :: (path: string) -> string {
    dot := find_index_from_right(path, #char ".");
    return tprint("%.lrc", slice(path, 0, ifx dot >= 0 then dot else path.count));
} @TempAllocates
#scope_file
lyric_order :: (a: Lyric, b: Lyric) -> s64 {
    if a.milliseconds != b.milliseconds return a.milliseconds - b.milliseconds;
    return a.sequence - b.sequence;
}

digits :: (text: string) -> bool {
    if !text.count return false;
    for text if it < #char "0" || it > #char "9" return false;
    return true;
}

timestamp :: (text: string) -> s64, bool {
    colon := find_index_from_left(text, #char ":");
    if colon < 1 || colon > 6 return 0, false;

    minutes := slice(text, 0, colon);
    rest := slice(text, colon + 1, text.count - colon - 1);
    dot := find_index_from_left(rest, #char ".");
    seconds := ifx dot >= 0 then slice(rest, 0, dot) else rest;
    fraction := ifx dot >= 0 then slice(rest, dot + 1, rest.count - dot - 1) else "";

    if !digits(minutes) || seconds.count != 2 || !digits(seconds) return 0, false;
    if dot >= 0 && (!digits(fraction) || fraction.count > 3) return 0, false;

    sec := string_to_int(seconds);
    if sec >= 60 return 0, false;

    ms := string_to_int(fraction);
    if fraction.count == 1 ms *= 100;
    if fraction.count == 2 ms *= 10;

    return (string_to_int(minutes) * 60 + sec) * 1000 + ms, true;
}
