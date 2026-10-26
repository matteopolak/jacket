// What the layouts and the serializer write for a script, kept as snapshots next to the tests.
#import "Basic";
#import "Extensions/Testing";
S :: #import "Subtitles";

SCRIPT :: "[1]\n: s1\n= ALICE\n: s2\n= ALICE\nLine1\n\nLine2\n: s3\n= BOB\n++ dir first\nspoken after\n: s4\n= A\none\n= B\nthree\n= A\nfour\n: s5\n= A\n+ fade soft\nHello <b> & co\n";

layouts_write_html_and_csv :: (t: *Test) {
    ok, script := S.load_subtitles_from_memory(copy_string(SCRIPT), "layouts");

    require(ok);
    defer S.free(*script);

    all: [..] *S.Subtitles_Section;
    all.allocator = temp;
    for *script.all_sections array_add(*all, it);

    builder: String_Builder;
    builder.allocator = temp;

    S.format_sections_as_html(*builder, all);
    expect_snapshot(t, builder_to_string(*builder,, temp), "html");

    reset(*builder);
    S.format_sections_as_csv(*builder, all);
    expect_snapshot(t, builder_to_string(*builder,, temp), "csv");
} @test @allow_leaks

serializing_writes_the_script_back :: (t: *Test) {
    ok, script := S.load_subtitles_from_memory(copy_string(SCRIPT), "serialized");

    require(ok);
    defer S.free(*script);

    text := S.serialize(*script);

    defer free(text);

    expect_snapshot(t, text);
} @test @allow_leaks
