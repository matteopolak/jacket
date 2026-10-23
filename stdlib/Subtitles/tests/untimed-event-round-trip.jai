#import "Basic";
S :: #import "Subtitles";
Strings :: #import "String";

main :: () {
    ok, subs := S.load_subtitles_from_memory("[1]\n: s\n= A 2.5\n+ blink\n+ fade soft\nHello\n", "untimed");

    assert(ok);
    assert(subs.all_sections[0].events[0].time_begin == 2.5);

    text := S.serialize(*subs);

    assert(Strings.contains(text, "+ blink\n"));
    assert(!Strings.contains(text, "+ blink 2.5"));

    again_ok, again := S.load_subtitles_from_memory(text, "again");

    assert(again_ok);

    event := again.all_sections[0].events[0];

    assert(event.name == "blink" && event.parameter.count == 0);
    assert(event.time_begin == 2.5 && event.time_end == -1);
    assert(again.all_sections[0].events[1].parameter == "soft");

    S.free(*again);
    free(text);
    S.free(*subs);
    print("ok\n");
}
