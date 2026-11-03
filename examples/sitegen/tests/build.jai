// The whole pipeline: a source folder goes in, an output folder comes out.

#import "Basic";
#import "String";
#import "File";
#import "Extensions/Testing";
#import,dir "../site";

#import,dir "support";

// A source folder with a template, a style sheet and two pages that link to each other.
a_small_site :: (t: *Test) -> Fixture {
    site := fixture(t);

    add_source(site, "_template.html", "<title>{{ title }}</title><link href=\"{{ asset style.css }}\">{{ nav }}{{ content }}");
    add_source(site, "style.css", "body { margin: 0 }");
    add_source(site, "index.md", "# Home\n\nSee [the guide](guide/start.md#top).");
    add_source(site, "guide/start.md", "# Start here\n\nBack to the [home page](../index.md).");

    return site;
} @TempAllocates(0)

pages_and_assets_are_written :: (t: *Test) {
    site := a_small_site(t);
    result := build_site(site.src, site.out);

    defer free_result(*result);

    expect_equal(result.error, "");
    expect_equal(result.pages, 2);
    expect_equal(result.assets, 1);
    expect_equal(result.warnings.count, 0);
    expect(output_exists(site, "index.html"));
    expect(output_exists(site, "guide/start.html"));
    expect(!output_exists(site, "style.css"));
    expect(!output_exists(site, "_template.html"));
} @test

assets_are_published_under_a_hashed_name :: (t: *Test) {
    site := a_small_site(t);
    result := build_site(site.src, site.out);

    defer free_result(*result);

    css := published_as(result, "style.", ".css");

    expect_equal(css.count, "style.xxxxxx.css".count);
    expect_equal(output_text(site, css), "body { margin: 0 }");

    // Each page links to it relative to itself.
    expect(contains(output_text(site, "index.html"), tprint("<link href=\"%\">", css)));
    expect(contains(output_text(site, "guide/start.html"), tprint("<link href=\"../%\">", css)));
} @test

the_hash_changes_with_the_content_only :: (t: *Test) {
    site := a_small_site(t);
    first := build_site(site.src, site.out);

    defer free_result(*first);

    again := build_site(site.src, site.out);

    defer free_result(*again);

    expect_equal(published_as(first, "style.", ".css"), published_as(again, "style.", ".css"));

    add_source(site, "style.css", "body { margin: 1px }");

    changed := build_site(site.src, site.out);

    defer free_result(*changed);

    expect(published_as(first, "style.", ".css") != published_as(changed, "style.", ".css"));
    // The old file stays; the build only adds.
    expect(output_exists(site, published_as(first, "style.", ".css")));
} @test

links_between_pages_end_in_html :: (t: *Test) {
    site := a_small_site(t);
    result := build_site(site.src, site.out);

    defer free_result(*result);

    expect(contains(output_text(site, "index.html"), "<a href=\"guide/start.html#top\">the guide</a>"));
    expect(contains(output_text(site, "guide/start.html"), "<a href=\"../index.html\">home page</a>"));
} @test

the_nav_lists_every_page_and_marks_the_current_one :: (t: *Test) {
    site := a_small_site(t);
    result := build_site(site.src, site.out);

    defer free_result(*result);

    html := output_text(site, "guide/start.html");

    expect(contains(html, "<li><a href=\"../index.html\">Home</a></li>"));
    expect(contains(html, "<li><a href=\"../guide/start.html\" aria-current=\"page\">Start here</a></li>"));
} @test

small_images_are_inlined_and_big_ones_are_linked :: (t: *Test) {
    site := a_small_site(t);

    add_source(site, "img/dot.png", "tiny");
    add_source(site, "img/big.png", "0123456789 0123456789 0123456789 0123456789 0123456789");
    add_source(site, "pics.md", "![dot](img/dot.png) ![big](img/big.png) [download](img/big.png)");

    options := Build_Options.{ inline_limit = 10 };
    result := build_site(site.src, site.out, options);

    defer free_result(*result);

    html := output_text(site, "pics.html");
    big := published_as(result, "img/big.", ".png");

    expect(contains(html, "<img src=\"data:image/png;base64,dGlueQ==\" alt=\"dot\">"));
    expect(contains(html, tprint("<img src=\"%\" alt=\"big\">", big)));
    expect(contains(html, tprint("<a href=\"%\">download</a>", big)));
    // An inlined image is still published, so other sites can link to it.
    expect(published_as(result, "img/dot.", ".png").count > 0);
} @test

pages_in_folders_reach_assets_through_dots :: (t: *Test) {
    site := a_small_site(t);

    add_source(site, "img/big.png", "0123456789 0123456789 0123456789");
    add_source(site, "a/b/deep.md", "![x](../../img/big.png)");

    options := Build_Options.{ inline_limit = 4 };
    result := build_site(site.src, site.out, options);

    defer free_result(*result);

    expect(contains(output_text(site, "a/b/deep.html"), tprint("src=\"../../%\"", published_as(result, "img/big.", ".png"))));
} @test

broken_links_are_warnings :: (t: *Test) {
    site := a_small_site(t);

    add_source(site, "broken.md", "[a](nowhere.md) [b](missing.png) ![c](gone.png) [d](https://example.com) [e](#here) [f](../../out.md)");

    result := build_site(site.src, site.out);

    defer free_result(*result);

    expect_equal(result.error, "");
    expect_equal(result.warnings.count, 4);
    expect(warned_about(result, "broken.md: `nowhere.md` links to a page that does not exist"));
    expect(warned_about(result, "`missing.png` is not a page or an asset"));
    expect(warned_about(result, "`gone.png` is not a page or an asset"));
    expect(warned_about(result, "`../../out.md` points outside the site"));
    // The page is still written, with the links as typed (a page link still ends in .html).
    expect(contains(output_text(site, "broken.html"), "<a href=\"https://example.com\">d</a>"));
} @test

drafts_and_private_files_are_left_out :: (t: *Test) {
    site := a_small_site(t);

    add_source(site, "wip.md", "---\ndraft: true\n---\n# Not yet");
    add_source(site, "_notes.md", "# Private");
    add_source(site, "_data/secret.txt", "secret");
    add_source(site, ".hidden", "hidden");
    add_source(site, "refers.md", "[draft](wip.md)");

    result := build_site(site.src, site.out);

    defer free_result(*result);

    expect_equal(result.pages, 3);
    expect_equal(result.assets, 1);
    expect(!output_exists(site, "wip.html"));
    expect(!output_exists(site, "_notes.html"));
    expect(warned_about(result, "`wip.md` links to a page that does not exist"));
} @test

the_front_matter_title_beats_the_heading :: (t: *Test) {
    site := a_small_site(t);

    add_source(site, "about.md", "---\ntitle: About us\n---\n# Heading\n");
    add_source(site, "plain.md", "No heading at all.");

    result := build_site(site.src, site.out);

    defer free_result(*result);

    expect(contains(output_text(site, "about.html"), "<title>About us</title>"));
    expect(contains(output_text(site, "index.html"), "<title>Home</title>"));
    expect(contains(output_text(site, "plain.html"), "<title>plain</title>"));
} @test

without_a_template_the_built_in_page_is_used :: (t: *Test) {
    site := fixture(t);

    add_source(site, "index.md", "# Hi\n\ntext");

    result := build_site(site.src, site.out);

    defer free_result(*result);

    html := output_text(site, "index.html");

    expect(begins_with(html, "<!doctype html>"));
    expect(contains(html, "<h1 id=\"hi\">Hi</h1>"));
} @test

a_template_can_live_anywhere :: (t: *Test) {
    site := a_small_site(t);
    path := tprint("%/theme.html", temp_dir(t));

    require(write_entire_file(path, "THEME {{ content }}"), "the template can be written");

    result := build_site(site.src, site.out, .{ template_path = path });

    defer free_result(*result);

    expect(begins_with(output_text(site, "index.html"), "THEME <h1"));
} @test

a_bad_template_stops_the_build_before_anything_is_written :: (t: *Test) {
    site := a_small_site(t);

    add_source(site, "_template.html", "<p>{{ nonsense }}</p>");

    result := build_site(site.src, site.out);

    defer free_result(*result);

    expect(contains(result.error, "unknown placeholder `nonsense`"));
    expect(contains(result.error, "_template.html"));
    expect_equal(result.pages, 0);
    expect(!output_exists(site, "index.html"));
} @test

a_template_with_a_missing_asset_warns :: (t: *Test) {
    site := a_small_site(t);

    add_source(site, "_template.html", "{{ asset nothing.css }}{{ content }}");

    result := build_site(site.src, site.out);

    defer free_result(*result);

    expect_equal(result.error, "");
    expect(warned_about(result, "missing asset `nothing.css`"));
} @test

the_output_folder_may_be_inside_the_source :: (t: *Test) {
    site := a_small_site(t);
    inside := tprint("%/public", site.src);

    first := build_site(site.src, inside);

    defer free_result(*first);

    second := build_site(site.src, inside);

    defer free_result(*second);

    // The first build's files are not read back in as pages or assets.
    expect_equal(second.pages, 2);
    expect_equal(second.assets, 1);
} @test

an_empty_source_folder_builds_an_empty_site :: (t: *Test) {
    site := fixture(t);

    require(make_directory_if_it_does_not_exist(site.src, recursive = true), "the folder can be made");

    result := build_site(site.src, site.out);

    defer free_result(*result);

    expect_equal(result.error, "");
    expect_equal(result.pages, 0);
    expect_equal(result.files.count, 0);
} @test
