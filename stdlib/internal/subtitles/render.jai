format_sections_as_html :: (builder: *String_Builder, sections: [] *Subtitles_Section, character_to_emphasize := "", name_map: *H.Table(string, string) = null) {
    append(builder, "<html><head><meta charset=\"UTF-8\"><style>.section{break-inside:avoid}.emphasized{font-weight: bold}.direction{font-style: italic}</style></head><body>\n");
    for section: sections format_section_as_html(builder, section, character_to_emphasize, name_map);
    append(builder, "</body></html>\n");
}

format_section_as_html :: (builder: *String_Builder, section: *Subtitles_Section, character_to_emphasize := "", name_map: *H.Table(string, string) = null) {
    print(builder, "<div class=\"section\" style=\"break-inside:avoid\"><h2>%</h2>\n", section.name);

    for instant: section.instants {
        if instant.speaker.count == 0 continue;

        display := displayed_name(instant.speaker, name_map);

        if instant.speaker == character_to_emphasize append(builder, "<div class=\"emphasized\" style=\"font-weight: bold\">");
        else append(builder, "<div>");

        print(builder, "<span>%</span><br>\n", display);

        for line: instant.lines {
            if line.type == .DIRECTION print(builder, "<span class=\"direction\" style=\"font-style: italic\">%</span><br>\n", line.text);
            else print(builder, "%<br>\n", line.text);
        }

        append(builder, "</div>\n");
    }

    append(builder, "</div>\n");
}

csv_quote :: (s: string) -> string {
    builder: String_Builder;

    defer free_buffers(*builder);
    append(*builder, cast(u8) 34);

    for byte: s {
        if byte == 34 append(*builder, cast(u8) 34);
        append(*builder, byte);
    }

    append(*builder, cast(u8) 34);

    return builder_to_string(*builder);
}

format_sections_as_csv :: (builder: *String_Builder, sections: [] *Subtitles_Section, name_map: *H.Table(string, string), row_number: *int, row_index: *int) {
    row_number.* = 0;
    append(builder, "Line #,Conversation,Character,Dialogue,Directions,Wav Filename,Conversation Filename\n");
    for section: sections format_section_as_csv(builder, section, name_map, row_number, row_index);
}

format_section_as_csv :: (builder: *String_Builder, section: *Subtitles_Section, name_map: *H.Table(string, string), row_number: *int, row_index: *int) {
    row_index.* = 0;
    speaker: string;
    lines: [..] string;
    directions: [..] string;

    defer array_free(lines);
    defer array_free(directions);

    for instant: section.instants {
        if instant.speaker != speaker {
            if speaker.count > 0 flush_csv_lines(builder, section.name, name_map, speaker, lines, directions, row_number, row_index);
            lines.count = 0;
            directions.count = 0;
            speaker = instant.speaker;
        }

        if speaker.count == 0 continue;

        for line: instant.lines {
            if line.type == .DIRECTION array_add(*directions, line.text);
            else if line.type == .SPOKEN array_add(*lines, line.text);
        }
    }

    if speaker.count > 0 flush_csv_lines(builder, section.name, name_map, speaker, lines, directions, row_number, row_index);
}

flush_csv_lines :: (builder: *String_Builder, section_name: string, name_map: *H.Table(string, string), _speaker_name: string, lines: [] string, directions: [] string, row_number: *int, row_index: *int) {
    if _speaker_name.count == 0 return;

    nonempty_spoken := false;

    for line: lines if line.count > 0 {
        nonempty_spoken = true;
        break;
    }

    if !nonempty_spoken return;

    spoken := join_lines(lines);
    directed := join_lines(directions);

    defer Basic.free(spoken);
    defer Basic.free(directed);
    who := csv_quote(displayed_name(_speaker_name, name_map));
    what := csv_quote(spoken);
    how := csv_quote(directed);

    defer Basic.free(who);
    defer Basic.free(what);
    defer Basic.free(how);
    print(builder, "%,%,%,%,%,%_%.wav,%.wav\n", row_number.* + 262, section_name, who, what, how, section_name, formatInt(row_index.* + 1, minimum_digits = 2), section_name);
    row_number.* += 1;
    row_index.* += 1;
}

#scope_file
Basic :: #import "Basic";

displayed_name :: (name: string, name_map: *H.Table(string, string)) -> string {
    if name_map {
        found, mapped := H.table_find(name_map, name);
        if found return mapped;
    }

    return name;
}

join_lines :: (lines: [] string) -> string {
    builder: String_Builder;

    defer free_buffers(*builder);

    for line, index: lines {
        if index > 0 append(*builder, "\n");
        append(*builder, line);
    }

    return builder_to_string(*builder);
}
