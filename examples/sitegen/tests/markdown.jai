// The Markdown subset: every block and inline form, and the ways they can nest.

#import "Basic";
#import "String";
#import "Extensions/Testing";
#import,dir "../markdown";

// The HTML for `source`, in temporary storage.
html :: (source: string) -> string {
    rendered := render_markdown(source);

    defer free(rendered.html);
    defer free(rendered.title);

    return copy_string(rendered.html,, temp);
}

headings_get_a_level_and_an_id :: () {
    expect_equal(html("# Hello, World!"), "<h1 id=\"hello-world\">Hello, World!</h1>\n");
    expect_equal(html("### Third"), "<h3 id=\"third\">Third</h3>\n");
    expect_equal(html("## Closed ##"), "<h2 id=\"closed\">Closed</h2>\n");
} @test

a_hash_without_a_space_is_text :: () {
    expect_equal(html("#hashtag"), "<p>#hashtag</p>\n");
    expect_equal(html("####### seven"), "<p>####### seven</p>\n");
} @test

the_first_h1_is_the_title :: () {
    rendered := render_markdown("intro\n\n## Not it\n\n# The *Title*\n\n# Second\n");

    defer free(rendered.html);
    defer free(rendered.title);

    expect_equal(rendered.title, "The *Title*");
} @test

a_document_without_h1_has_no_title :: () {
    rendered := render_markdown("## Only a subheading");

    defer free(rendered.html);
    defer free(rendered.title);

    expect_equal(rendered.title, "");
} @test

lines_make_one_paragraph_and_a_blank_line_ends_it :: () {
    expect_equal(html("one\ntwo\n\nthree"), "<p>one\ntwo</p>\n<p>three</p>\n");
} @test

windows_line_endings_are_accepted :: () {
    expect_equal(html("# A\r\n\r\ntext\r\n"), "<h1 id=\"a\">A</h1>\n<p>text</p>\n");
} @test

emphasis_and_strong :: () {
    expect_equal(html("a *b* and _c_ and **d**"), "<p>a <em>b</em> and <em>c</em> and <strong>d</strong></p>\n");
    expect_equal(html("**bold with *inner* em**"), "<p><strong>bold with <em>inner</em> em</strong></p>\n");
} @test

markers_that_do_not_close_are_text :: () {
    expect_equal(html("2 * 3 = 6"), "<p>2 * 3 = 6</p>\n");
    expect_equal(html("a *b"), "<p>a *b</p>\n");
    expect_equal(html("snake_case_name"), "<p>snake_case_name</p>\n");
} @test

code_spans_keep_their_text_verbatim :: () {
    expect_equal(html("use `a < b && *c*` here"), "<p>use <code>a &lt; b &amp;&amp; *c*</code> here</p>\n");
    expect_equal(html("an `open one"), "<p>an `open one</p>\n");
} @test

a_backslash_escapes_punctuation :: () {
    expect_equal(html("\\*not emphasis\\*"), "<p>*not emphasis*</p>\n");
} @test

html_in_the_source_is_escaped :: () {
    expect_equal(html("<script>alert(\"x\")</script>"), "<p>&lt;script&gt;alert(&quot;x&quot;)&lt;/script&gt;</p>\n");
} @test

links_and_images :: () {
    expect_equal(html("see [the *docs*](https://example.com/a?b=1&c=2)"), "<p>see <a href=\"https://example.com/a?b=1&amp;c=2\">the <em>docs</em></a></p>\n");
    expect_equal(html("![A logo](logo.png)"), "<p><img src=\"logo.png\" alt=\"A logo\"></p>\n");
    expect_equal(html("[broken](no close"), "<p>[broken](no close</p>\n");
} @test

empty_links_and_code_are_fine :: () {
    expect_equal(html("[](x.md) ![](y.png) `` **x**"), "<p><a href=\"x.md\"></a> <img src=\"y.png\" alt=\"\"> <code></code> <strong>x</strong></p>\n");
} @test

rewrite_url :: (url: string, kind: Link_Kind, data: *void) -> string {
    prefix := ifx kind == .IMAGE then "img:" else "page:";

    return join(prefix, url,, temp);
}

urls_go_through_the_rewrite_hook :: () {
    options := Options.{ rewrite_url = rewrite_url };
    rendered := render_markdown("[a](b.md) ![c](d.png)", *options);

    defer free(rendered.html);
    defer free(rendered.title);

    expect_equal(rendered.html, "<p><a href=\"page:b.md\">a</a> <img src=\"img:d.png\" alt=\"c\"></p>\n");
} @test

fenced_code_is_escaped_and_not_parsed :: () {
    source := "```jai\nx := a < b; # not a heading\n*not em*\n```\nafter";

    expect_equal(html(source), "<pre><code class=\"language-jai\">x := a &lt; b; # not a heading\n*not em*\n</code></pre>\n<p>after</p>\n");
} @test

an_unclosed_fence_runs_to_the_end :: () {
    expect_equal(html("```\nopen\n\nstill code"), "<pre><code>open\n\nstill code\n</code></pre>\n");
} @test

bullet_lists :: () {
    expect_equal(html("- one\n- two *x*\n* three"), "<ul>\n<li>one</li>\n<li>two <em>x</em></li>\n<li>three</li>\n</ul>\n");
} @test

numbered_lists :: () {
    expect_equal(html("1. first\n2. second\n10. third"), "<ol>\n<li>first</li>\n<li>second</li>\n<li>third</li>\n</ol>\n");
} @test

an_item_continues_on_the_next_line :: () {
    expect_equal(html("- one\n  more\n- two"), "<ul>\n<li>one\nmore</li>\n<li>two</li>\n</ul>\n");
} @test

lists_nest_by_indenting :: () {
    expect_equal(html("- a\n  - a1\n  - a2\n- b"), "<ul>\n<li>a\n<ul>\n<li>a1</li>\n<li>a2</li>\n</ul></li>\n<li>b</li>\n</ul>\n");
    expect_equal(html("1. a\n   - x\n2. b"), "<ol>\n<li>a\n<ul>\n<li>x</li>\n</ul></li>\n<li>b</li>\n</ol>\n");
} @test

a_blank_line_ends_a_list_unless_it_goes_on :: () {
    expect_equal(html("- a\n\nafter"), "<ul>\n<li>a</li>\n</ul>\n<p>after</p>\n");
    expect_equal(html("- a\n\n- b"), "<ul>\n<li>a</li>\n<li>b</li>\n</ul>\n");
} @test

a_different_marker_starts_another_list :: () {
    expect_equal(html("- a\n\n1. b"), "<ul>\n<li>a</li>\n</ul>\n<ol>\n<li>b</li>\n</ol>\n");
    expect_equal(html("1. a\n- b"), "<ol>\n<li>a</li>\n</ol>\n<ul>\n<li>b</li>\n</ul>\n");
} @test

a_heading_ends_a_list :: () {
    expect_equal(html("- a\n## H"), "<ul>\n<li>a</li>\n</ul>\n<h2 id=\"h\">H</h2>\n");
} @test

block_quotes_hold_blocks :: () {
    expect_equal(html("> # Title\n> some *text*\n> - item"), "<blockquote>\n<h1 id=\"title\">Title</h1>\n<p>some <em>text</em></p>\n<ul>\n<li>item</li>\n</ul>\n</blockquote>\n");
} @test

rules :: () {
    expect_equal(html("a\n\n---\n\nb"), "<p>a</p>\n<hr>\n<p>b</p>\n");
    expect_equal(html("***"), "<hr>\n");
} @test

an_empty_document_is_empty :: () {
    expect_equal(html(""), "");
    expect_equal(html("\n\n  \n"), "");
} @test
