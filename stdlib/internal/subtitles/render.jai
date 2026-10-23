format_sections_as_html :: (builder: *String_Builder, sections: [] *Subtitles_Section, character_to_emphasize := "", name_map: *H.Table(string, string) = null) {
    append(builder, HTML_HEAD);
    append(builder, "<body>\n");
    for section: sections format_section_as_html(builder, section, character_to_emphasize, name_map);
    append(builder, "\n</body>\n");
}

format_section_as_html :: (builder: *String_Builder, section: *Subtitles_Section, character_to_emphasize := "", name_map: *H.Table(string, string) = null) {
    append(builder, "<br><p></p><br>\n<div class=\"keep-together\">\n\n");
    print(builder, "<p class=\"title\">%</p>\n", section.name);

    first := 0;

    while first < section.instants.count {
        speaker := section.instants[first].speaker;

        if speaker.count == 0 {
            first += 1;
            continue;
        }

        // A turn takes every instant of one speaker up to the next speaker's; instants without a
        // speaker belong to no turn and do not end one.
        next := first + 1;

        while next < section.instants.count {
            other := section.instants[next].speaker;

            if other.count > 0 && other != speaker break;
            next += 1;
        }

        append_turn_html(builder, array_view(section.instants, first, next - first), speaker == character_to_emphasize, displayed_name(speaker, name_map));
        first = next;
    }

    append(builder, "</div>");
}

csv_quote :: (s: string) -> string {
    builder: String_Builder;

    defer free_buffers(*builder);
    append(*builder, cast(u8) 34);

    for byte: s {
        if byte == #char "\"" append(*builder, cast(u8) 34);
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
        if instant.speaker.count == 0 continue;

        if instant.speaker != speaker {
            if speaker.count > 0 flush_csv_lines(builder, section.name, name_map, speaker, lines, directions, row_number, row_index);
            lines.count = 0;
            directions.count = 0;
            speaker = instant.speaker;
        }

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
    print(builder, "%,%,%,%,%,%_%.wav,%.wav\n", row_number.* + ROW_NUMBER_BASE, section_name, who, what, how, section_name, formatInt(row_index.* + 1, minimum_digits = 2), section_name);
    row_number.* += 1;
    row_index.* += 1;
}

#scope_file

ROW_NUMBER_BASE :: 262;

HTML_HEAD :: #string END
<head><style>.title          {text-align:left; margin-inline: 15vw; font-weight:800;    font-size:24px; }</style>
<style>.speaker-bold   {text-align:left; margin-inline: 35vw; font-weight:750;    font-size:16px;}</style>
<style>.speaker-nonbold{text-align:left; margin-inline: 35vw; font-weight:400;    font-size:16px;}</style>
<style>.dialog-bold    {text-align:left; margin-inline: 25vw; font-weight:400;    font-size:16px;}</style>
<style>.dialog-nonbold {text-align:left; margin-inline: 25vw; font-weight:100;    font-size:16px;}</style>
<style>.direction      {text-align:left; margin-inline: 15vw; font-style:italic;}</style>
<style>.keep-together  {break-inside:avoid;}</style>
</head>
END

// One speaker's turn: the name, then each run of spoken lines or of directions as a paragraph,
// lines in a run joined by line breaks.
append_turn_html :: (builder: *String_Builder, instants: [] Subtitles_Instant, emphasized: bool, display: string) {
    weight := ifx emphasized then "bold" else "nonbold";

    print(builder, "\n<p class=\"speaker-%\"><b>%</b></p>\n", weight, display);

    run := Subtitles_Line.type.UNINITIALIZED;
    wrote := false;

    for instant: instants {
        if instant.speaker.count == 0 continue;

        for line: instant.lines {
            if line.type == .UNINITIALIZED continue;

            if line.type == run {
                append(builder, "<br>");
            } else {
                if run != .UNINITIALIZED append(builder, "</p>");
                if line.type == .DIRECTION append(builder, "<p class=\"direction\">");
                else print(builder, "<p class=\"dialog-%\">", weight);

                run = line.type;
            }

            append(builder, line.text);
            wrote = true;
        }
    }

    if run != .UNINITIALIZED append(builder, "</p>");
    if wrote append(builder, "\n");
}

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
