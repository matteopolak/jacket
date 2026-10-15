#import "Basic";
S :: #import "Subtitles";
H :: #import "Hash_Table";
F :: #import "File";
Strings :: #import "String";

errors: int;

capture :: (message: string, data: *void, info: Log_Info) {
    assert(message.count > 0 && data == null && (info.common_flags & .ERROR) != 0);
    errors += 1;
}

main :: () {
    args := get_command_line_arguments();

    assert(args.count == 2);
    input := copy_string("[77]\n# ignored\n:: scene embedded\n+ intro p 0 1\n= ALICE 1.25\nHello <b>raw</b> # tail\n++ gently\n= ALICE 2\nSecond \"line\"\n+ flash bright\n= _\nsilence\n: outro\n= BOB\nBye\n");

    defer free(input);
    ok, subs := S.load_subtitles_from_memory(input, "original");

    assert(ok && !subs.data_owned && subs.data.data == input.data);
    assert(subs.all_sections.count == 2);
    section := *subs.all_sections[0];

    assert(section.name == "scene" && section.embed == "embedded" && (section.flags & .FINALIZED) != 0);
    assert(section.instants.count == 3 && section.instants[0].time == 1.25);
    assert(section.instants[0].lines[0].text == "Hello <b>raw</b>");
    assert(section.instants[0].lines[1].type == .DIRECTION);
    assert(section.events.count == 2 && section.events[0].instant_index == -1);
    assert(section.events[1].instant_index == 1 && section.events[1].time_begin == 2);
    found, by_name := H.table_find(*subs.sections_by_name, "scene");

    assert(found && by_name == section);
    chosen := S.get_sections_for_characters(subs, "ALICE", "BOB", "ALICE");

    defer array_free(chosen);
    assert(chosen.count == 2 && chosen[0] == section);
    blank := S.get_sections_for_characters(subs, "");

    assert(blank.count == 1 && blank[0] == section);
    array_free(blank);

    serialized := S.serialize(*subs);

    defer free(serialized);
    ok2, roundtrip := S.load_subtitles_from_memory(serialized, "roundtrip");

    assert(ok2 && roundtrip.all_sections.count == 2);
    assert(roundtrip.all_sections[0].events[0].instant_index == -1);
    assert(roundtrip.all_sections[0].instants[0].lines[0].text == section.instants[0].lines[0].text);
    S.free(*roundtrip);

    names: H.Table(string, string);

    H.table_add(*names, "ALICE", "Alice \"A\"");
    defer H.deinit(*names);
    builder: String_Builder;

    defer free_buffers(*builder);
    S.format_sections_as_html(*builder, chosen, "ALICE", *names);
    html := builder_to_string(*builder);

    defer free(html);
    assert(Strings.contains(html, "<b>raw</b>") && Strings.contains(html, "Alice \"A\""));
    assert(!Strings.contains(html, "silence") && !Strings.contains(html, "intro"));
    assert(Strings.contains(html, "font-weight: bold") && Strings.contains(html, "font-style: italic"));
    reset(*builder);
    S.format_sections_as_csv(*builder, chosen, *names);
    csv := builder_to_string(*builder);

    defer free(csv);
    csv_path := sprint("%/subtitles.csv", args[1]);

    defer free(csv_path);
    assert(F.write_entire_file(csv_path, csv));
    expected := "Line #,Conversation,Character,Dialogue,Directions,Wav Filename,Conversation Filename\n262,scene,\"Alice \"\"A\"\"\",\"Hello <b>raw</b>\nSecond \"\"line\"\"\",\"gently\",scene_01.wav,scene.wav\n263,outro,\"BOB\",\"Bye\",\"\",outro_01.wav,outro.wav\n";

    assert(csv == expected);
    assert(S.csv_line_number == 2 && S.csv_line_index_within_section == 1);
    reset(*builder);
    S.csv_line_number = 900;
    S.format_section_as_csv(*builder, chosen[1]);
    direct := builder_to_string(*builder);

    assert(Strings.begins_with(direct, "1162,") && S.csv_line_number == 901);
    free(direct);
    reset(*builder);
    S.csv_line_number = 0;
    S.csv_line_index_within_section = 0;
    empty_lines: [2] string = .["", ""];
    only_directions: [1] string = .["direction only"];

    S.flush_csv_lines(*builder, "no-row", null, "speaker", empty_lines, only_directions);
    assert(builder_string_length(*builder) == 0 && S.csv_line_number == 0 && S.csv_line_index_within_section == 0);
    S.csv_line_number = 1200;
    S.csv_line_index_within_section = 7;
    line_items: [2] string = .["one", "two"];
    direction_items: [1] string = .["aside"];

    S.flush_csv_lines(*builder, "direct", null, "speaker", line_items, direction_items);
    direct_flush := builder_to_string(*builder);

    assert(direct_flush == "1462,direct,\"speaker\",\"one\ntwo\",\"aside\",direct_08.wav,direct.wav\n");
    assert(S.csv_line_number == 1201 && S.csv_line_index_within_section == 8);
    free(direct_flush);
    reset(*builder);
    S.format_section_as_html(*builder, chosen[1]);
    single_html := builder_to_string(*builder);

    assert(Strings.contains(single_html, "outro") && !Strings.contains(single_html, "<head>"));
    free(single_html);
    assert(type_info(S.Subtitles).name == "Subtitles" && type_info(S.Subtitles_Section).name == "Subtitles_Section");
    assert(size_of(type_of(section.flags)) == 4 && size_of(type_of(section.instants[0].lines[0].type)) == 1);
    quoted := S.csv_quote("a,\"b\"\nc");

    assert(quoted == "\"a,\"\"b\"\"\nc\"");
    free(quoted);

    path := sprint("%/subtitles-ü.txt", args[1]);

    defer free(path);
    assert(F.write_entire_file(path, serialized));
    file_ok, owned := S.load_subtitles_from_file(path);

    assert(file_ok && owned.data_owned && owned.data.data != serialized.data);
    S.free(*owned);
    assert(owned.data.data == null && owned.all_sections.count == 0);

    push_context {
        context.logger = capture;
        context.logger_data = null;
        bad_ok, bad := S.load_subtitles_from_memory("[1]\n: x\ntext\n", "no-instant");

        assert(!bad_ok && bad.all_sections.count == 0);
        bad_ok, bad = S.load_subtitles_from_memory("[1]\n: x\n: x\n", "duplicate");
        assert(!bad_ok && bad.all_sections.count == 0);
        bad_ok, bad = S.load_subtitles_from_memory("[1]\n: x\n+ e p\n", "untimed-event");
        assert(!bad_ok && bad.all_sections.count == 0);
        bad_ok, bad = S.load_subtitles_from_memory("[1]\n: x\n= a invalid\n", "bad-float");
        assert(!bad_ok && bad.all_sections.count == 0);
        bad_ok, bad = S.load_subtitles_from_memory("[1]\n: x\n++ direction\n", "no-direction-instant");
        assert(!bad_ok && bad.all_sections.count == 0);
        bad_ok, bad = S.load_subtitles_from_memory("wrong\n: x\n", "header");
        assert(!bad_ok && bad.data.data == null);
    }

    assert(errors == 6);
    empty_ok, empty := S.load_subtitles_from_memory("[1]\n# no sections\n", "empty");

    assert(empty_ok && empty.all_sections.count == 0);
    S.free(*empty);
    unusual_ok, unusual := S.load_subtitles_from_memory("[3]\r\n: café\r\n+ backwards p 5 2\r\n= A -3\r\nwords\r\n= A -5\r\n++ aside\r\n", "unusual");

    assert(unusual_ok && unusual.all_sections[0].name == "café");
    assert(unusual.all_sections[0].events[0].time_begin == 5 && unusual.all_sections[0].events[0].time_end == 2);
    assert(unusual.all_sections[0].instants[0].time == -3 && unusual.all_sections[0].instants[1].time == -5);
    unusual.all_sections[0].events[0].time_begin = -1;
    unusual.all_sections[0].events[0].time_end = 8;
    negative_serialized := S.serialize(*unusual);

    assert(Strings.contains(negative_serialized, "+ backwards p\n"));
    assert(!Strings.contains(negative_serialized, "+ backwards p 8"));
    free(negative_serialized);
    S.free(*unusual);
    ambiguous_ok, ambiguous := S.load_subtitles_from_memory("[1]\n: s\n+ fade 1.5 2.0\n= A\nspoken\n", "positional");

    assert(ambiguous_ok && ambiguous.all_sections[0].events[0].parameter == "1.5");
    assert(ambiguous.all_sections[0].events[0].time_begin == 2 && ambiguous.all_sections[0].events[0].time_end == -1);
    S.free(*ambiguous);
    S.free(*subs);
    assert(input[0] == 91 && subs.all_sections.count == 0);
    print("ok\n");
}
