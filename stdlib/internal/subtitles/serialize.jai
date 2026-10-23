serialize :: (using subtitles: *Subtitles) -> string {
    builder: String_Builder;

    defer free_buffers(*builder);
    append(*builder, "[1]\n");

    for section: all_sections {
        append(*builder, ":");
        if (section.flags & .FINALIZED) != 0 append(*builder, ":");

        print(*builder, " %", section.name);
        if section.embed.count > 0 print(*builder, " %", section.embed);

        append(*builder, "\n");
        for event: section.events if event.instant_index < 0 emit_event(*builder, event, -1);

        for instant, index: section.instants {
            speaker := instant.speaker;
            if speaker.count == 0 speaker = "_";

            print(*builder, "= %", speaker);
            if instant.time >= 0 print(*builder, " %", formatFloat(instant.time, mode = .SHORTEST));

            append(*builder, "\n");
            for event: section.events if event.instant_index == index emit_event(*builder, event, instant.time);

            for line: instant.lines {
                if line.type == .DIRECTION append(*builder, "++ ");
                append(*builder, line.text);
                append(*builder, "\n");
            }
        }
    }

    return builder_to_string(*builder);
}

#scope_file
emit_event :: (builder: *String_Builder, event: Subtitles_Event, instant_time: float64) {
    print(builder, "+ %", event.name);

    // Loading gives an event without times its instant's time. A bare time after the
    // name would read back as the parameter, so such an event is written without times.
    // jailint: allow(float_equality): the event copied this exact time when it was loaded
    if event.parameter.count == 0 && event.instant_index >= 0 && event.time_begin == instant_time && event.time_end < 0 {
        append(builder, "\n");
        return;
    }

    if event.parameter.count > 0 print(builder, " %", event.parameter);
    if event.time_begin >= 0 print(builder, " %", formatFloat(event.time_begin, mode = .SHORTEST));
    if event.time_begin >= 0 && event.time_end >= 0 print(builder, " %", formatFloat(event.time_end, mode = .SHORTEST));

    append(builder, "\n");
}
