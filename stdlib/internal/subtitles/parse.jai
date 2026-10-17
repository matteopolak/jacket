free :: (using subtitles: *Subtitles) {
    H.deinit(*sections_by_name);

    for *section: all_sections {
        for *instant: section.instants array_free(instant.lines);
        array_free(section.instants);
        array_free(section.events);
    }

    array_free(all_sections);
    if data_owned && data.data != null Basic.free(data);

    subtitles.* = .{};
} @Releases

load_subtitles_from_file :: (path: string) -> bool, Subtitles {
    bytes, read_ok := F.read_entire_file(path);
    if !read_ok return false, .{};

    ok, subtitles := load_subtitles_from_memory(bytes, path);
    if !ok {
        Basic.free(bytes);
        return false, .{};
    }

    subtitles.data_owned = true;

    return true, subtitles;
}

load_subtitles_from_memory :: (data: string, short_name: string) -> bool, Subtitles {
    handler: T.Text_File_Handler;

    T.start_from_memory(*handler, short_name, short_name, data);
    defer T.deinit(*handler);
    subtitles := Subtitles.{ data = data };

    if handler.failed return false, .{};

    section_index: s64 = -1;
    instant_index: s64 = -1;

    while true {
        line, found := T.consume_next_line(*handler);
        if !found break;

        if line[0] == #char ":" {
            finalized := line.count > 1 && line[1] == #char ":";
            prefix: s64 = 1;

            if finalized prefix = 2;

            name, rest := T.break_by_spaces(.{ line.count - prefix, line.data + prefix });
            embed, extra := T.break_by_spaces(rest);

            if name.count == 0 || extra.count != 0 return parse_failed(*subtitles, handler, "invalid section declaration");
            for section: subtitles.all_sections if section.name == name return parse_failed(*subtitles, handler, "duplicate section name");

            section := Subtitles_Section.{ name = name, embed = embed };
            if finalized section.flags = .FINALIZED;

            array_add(*subtitles.all_sections, section);
            section_index = subtitles.all_sections.count - 1;
            instant_index = -1;
            continue;
        }

        if section_index < 0 return parse_failed(*subtitles, handler, "content requires a section");

        section := *subtitles.all_sections[section_index];

        if line[0] == #char "=" {
            speaker, rest := T.break_by_spaces(.{ line.count - 1, line.data + 1 });
            time_token, extra := T.break_by_spaces(rest);

            if speaker.count == 0 || extra.count != 0 return parse_failed(*subtitles, handler, "invalid instant");

            instant := Subtitles_Instant.{ speaker = speaker };
            if speaker == "_" instant.speaker = "";

            if time_token.count > 0 {
                value, valid := parse_time(time_token);
                if !valid return parse_failed(*subtitles, handler, "invalid instant time");
                instant.time = value;
            }

            if section.instants.count >= 0x7fffffff return parse_failed(*subtitles, handler, "too many instants");

            array_add(*section.instants, instant);
            instant_index = section.instants.count - 1;
        } else if line[0] == #char "+" && (line.count < 2 || line[1] != #char "+") {
            name, rest := T.break_by_spaces(.{ line.count - 1, line.data + 1 });
            parameter, after_parameter := T.break_by_spaces(rest);
            begin_token, after_begin := T.break_by_spaces(after_parameter);
            end_token, extra := T.break_by_spaces(after_begin);

            if name.count == 0 || extra.count > 0 return parse_failed(*subtitles, handler, "invalid event");

            event := Subtitles_Event.{ name = name, parameter = parameter, instant_index = cast(s32) instant_index };

            if begin_token.count > 0 {
                value, valid := parse_time(begin_token);
                if !valid return parse_failed(*subtitles, handler, "invalid event begin time");
                event.time_begin = value;
            } else {
                if instant_index < 0 return parse_failed(*subtitles, handler, "untimed event requires an instant");
                event.time_begin = section.instants[instant_index].time;
            }

            if end_token.count > 0 {
                value, valid := parse_time(end_token);
                if !valid return parse_failed(*subtitles, handler, "invalid event end time");
                event.time_end = value;
            }

            array_add(*section.events, event);
        } else {
            if instant_index < 0 return parse_failed(*subtitles, handler, "line requires an instant");

            spoken := Subtitles_Line.{ type = .SPOKEN, text = line };

            if line.count >= 2 && line[0] == #char "+" && line[1] == #char "+" {
                spoken.type = .DIRECTION;
                spoken.text = Strings.trim(.{ line.count - 2, line.data + 2 });
            }

            array_add(*section.instants[instant_index].lines, spoken);
        }
    }

    for *section: subtitles.all_sections H.table_add(*subtitles.sections_by_name, section.name, section);

    return true, subtitles;
}

get_sections_for_characters :: (subtitles: Subtitles, names: ..string) -> [..] *Subtitles_Section {
    selected: [..] *Subtitles_Section;

    for *section: subtitles.all_sections {
        matches := false;

        for instant: section.instants {
            for name: names if name == instant.speaker {
                matches = true;
                break;
            }

            if matches break;
        }

        if matches array_add(*selected, section);
    }

    return selected;
}

#scope_file
Basic :: #import "Basic";

parse_time :: (token: string) -> float64, bool {
    value, valid, rest := string_to_float64(token);
    return value, valid && rest.count == 0;
}

parse_failed :: (subtitles: *Subtitles, handler: T.Text_File_Handler, message: string) -> bool, Subtitles {
    T.error(handler, "%", message);
    free(subtitles);
    return false, .{};
}
