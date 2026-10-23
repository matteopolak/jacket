// Text that HTML would read as markup is written as entities in titles, names and lines.
#import "Basic";
#import "String";
S :: #import "Subtitles";

INPUT :: "[1]\n: a<b\n= R&D\nx < y && \"z\" > w\n";

main :: () {
    ok, subs := S.load_subtitles_from_memory(copy_string(INPUT), "escape");

    assert(ok);

    all: [..] *S.Subtitles_Section;
    for *subs.all_sections array_add(*all, it);

    builder: String_Builder;

    S.format_sections_as_html(*builder, all);
    html := builder_to_string(*builder);

    assert(contains(html, "<p class=\"title\">a&lt;b</p>"));
    assert(contains(html, "<b>R&amp;D</b>"));
    assert(contains(html, "x &lt; y &amp;&amp; &quot;z&quot; &gt; w"));
    assert(!contains(html, "a<b"));
    print("ok\n");
}
