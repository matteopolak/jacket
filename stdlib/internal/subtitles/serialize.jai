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
        for event: section.events if event.instant_index < 0 emit_event(*builder, event);

        for instant, index: section.instants {
            speaker := instant.speaker;
            if speaker.count == 0 speaker = "_";

            print(*builder, "= %", speaker);
            if instant.time >= 0 print(*builder, " %", formatFloat(instant.time, mode = .SHORTEST));

            append(*builder, "\n");
            for event: section.events if event.instant_index == index emit_event(*builder, event);

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
emit_event :: (builder: *String_Builder, event: Subtitles_Event) {
    print(builder, "+ %", event.name);
    if event.parameter.count > 0 print(builder, " %", event.parameter);
    if event.time_begin >= 0 print(builder, " %", formatFloat(event.time_begin, mode = .SHORTEST));
    if event.time_begin >= 0 && event.time_end >= 0 print(builder, " %", formatFloat(event.time_end, mode = .SHORTEST));

    append(builder, "\n");
}
