// The small parts of the site builder on their own: paths, asset names, templates, front matter.

#import "Basic";
#import "String";
#import "Sort";
#import "Extensions/Testing";
#import,dir "../site";

resolving_paths_folds_dots :: () {
    expect_equal(resolve_relative("", "a.png"), "a.png");
    expect_equal(resolve_relative("guide/", "../img/a.png"), "img/a.png");
    expect_equal(resolve_relative("guide/", "./a/./b.md"), "guide/a/b.md");
    expect_equal(resolve_relative("guide/", "../../escape.png"), "");
} @test

the_way_back_has_one_step_per_folder :: () {
    expect_equal(root_prefix("index.html"), "");
    expect_equal(root_prefix("guide/intro.html"), "../");
    expect_equal(root_prefix("a/b/c.html"), "../../");
} @test

external_links_are_recognised :: () {
    expect(is_external("https://example.com"));
    expect(is_external("mailto:me@example.com"));
    expect(is_external("#top"));
    expect(is_external("/absolute"));
    expect(!is_external("page.md"));
    expect(!is_external("../page.md#part"));
} @test

fragments_are_split_off :: () {
    path, fragment := split_fragment("page.md#part");

    expect_equal(path, "page.md");
    expect_equal(fragment, "#part");

    path, fragment = split_fragment("page.md");

    expect_equal(path, "page.md");
    expect_equal(fragment, "");
} @test

names_that_start_with_a_dot_or_underscore_are_private :: () {
    expect(is_private("_template.html"));
    expect(is_private(".git/config"));
    expect(is_private("drafts/_notes.md"));
    expect(!is_private("guide/intro.md"));
} @test

a_hashed_name_keeps_the_folder_and_extension :: () {
    // The first six digits of md5("abc").
    expect_equal(hashed_name("style.css", "abc"), "style.900150.css");
    expect_equal(hashed_name("css/site.min.css", "abc"), "css/site.min.900150.css");
    expect_equal(hashed_name("LICENSE", "abc"), "LICENSE.900150");
} @test

a_hashed_name_follows_the_content :: () {
    expect_equal(hashed_name("a.css", "same"), hashed_name("a.css", "same"));
    expect(hashed_name("a.css", "one") != hashed_name("a.css", "two"));
    // The name does not matter to the digest, only the bytes.
    expect_equal(slice(hashed_name("x.css", "same"), 2, 7), slice(hashed_name("y.css", "same"), 2, 7));
} @test

mime_types_follow_the_extension :: () {
    expect_equal(mime_type("a/b.css"), "text/css");
    expect_equal(mime_type("logo.PNG"), "image/png");
    expect_equal(mime_type("photo.jpeg"), "image/jpeg");
    expect_equal(mime_type("icon.svg"), "image/svg+xml");
    expect_equal(mime_type("data.bin"), "application/octet-stream");
    expect_equal(mime_type("README"), "application/octet-stream");
} @test

small_images_can_be_inlined :: () {
    assets: Assets;

    defer free_assets(*assets);

    add_asset(*assets, "dot.png", "hello");
    add_asset(*assets, "page.css", "hi");

    image := find_asset(*assets, "dot.png");
    sheet := find_asset(*assets, "page.css");

    expect(is_inlinable(image, 5));
    expect(!is_inlinable(image, 4));
    expect(!is_inlinable(sheet, 100));
    expect_equal(data_uri(image), "data:image/png;base64,aGVsbG8=");
    expect_equal(find_asset(*assets, "missing.png"), null);
} @test

front_matter_sets_the_title_and_draft_flag :: () {
    matter := split_front_matter("---\ntitle: A: Tale\ndraft: true\nother: x\n---\n# Body\n\nText");

    expect_equal(matter.title, "A: Tale");
    expect(matter.draft);
    expect_equal(matter.body, "# Body\n\nText");
} @test

a_page_without_front_matter_is_all_body :: () {
    matter := split_front_matter("# Just a page\n");

    expect_equal(matter.title, "");
    expect(!matter.draft);
    expect_equal(matter.body, "# Just a page\n");
} @test

empty_and_last_front_matter_blocks_work :: () {
    matter := split_front_matter("---\n---\nbody");

    expect_equal(matter.body, "body");
    expect_equal(matter.title, "");

    matter = split_front_matter("---\ntitle: Only\n---");

    expect_equal(matter.body, "");
    expect_equal(matter.title, "Only");
} @test

an_unclosed_front_matter_block_is_a_rule :: () {
    matter := split_front_matter("---\nnot closed\n");

    expect_equal(matter.body, "---\nnot closed\n");
} @test

pages_get_html_names_and_folders :: () {
    expect_equal(page_output("guide/intro.md"), "guide/intro.html");
    expect_equal(page_folder("guide/intro.md"), "guide/");
    expect_equal(page_folder("index.md"), "");
    expect_equal(title_from_name("guide/intro.md"), "intro");
} @test

the_index_comes_first_in_the_nav :: () {
    index := Page.{ source = "index.md", output = "index.html", title = "Home" };
    about := Page.{ source = "about.md", output = "about.html", title = "About <us>" };
    deep := Page.{ source = "z/deep.md", output = "z/deep.html", title = "Deep" };
    pages := (*Page).[*deep, *about, *index];

    quick_sort(pages, page_before);

    expect(pages[0] == *index && pages[1] == *about && pages[2] == *deep);

    nav := render_nav(pages, *deep);

    expect_equal(nav, "<ul>\n<li><a href=\"../index.html\">Home</a></li>\n<li><a href=\"../about.html\">About &lt;us&gt;</a></li>\n<li><a href=\"../z/deep.html\" aria-current=\"page\">Deep</a></li>\n</ul>");
} @test

parse_and_expand :: (source: string, values: Page_Values, assets: *Assets, warnings: *[..] string) -> string {
    template, error := parse_template(source);

    expect_equal(error, "");

    defer free_template(*template);

    return expand_template(*template, values, assets, 100, warnings);
}

a_template_fills_in_its_placeholders :: () {
    assets: Assets;
    warnings: [..] string;

    defer free_assets(*assets);

    add_asset(*assets, "site.css", "abc");
    add_asset(*assets, "dot.png", "hello");
    add_asset(*assets, "big.png", "this one is far too large to be inlined into a page, so it stays a file: 0123456789 0123456789 0123456789 0123456789");

    values := Page_Values.{ title = "Fish & Chips", content = "<p>body</p>", nav = "<ul></ul>", path = "a/b.html" };
    source := "<t>{{title}}</t>{{ nav }}{{ content }}|{{root}}|{{ path }}|{{ asset site.css }}|{{ inline dot.png }}|{{ inline big.png }}";
    html := parse_and_expand(source, values, *assets, *warnings);

    expect(begins_with(html, "<t>Fish &amp; Chips</t><ul></ul><p>body</p>|../|a/b.html|../site.900150.css|data:image/png;base64,aGVsbG8=|../big."));
    expect_equal(warnings.count, 0);
} @test

a_missing_asset_is_a_warning_not_a_failure :: () {
    assets: Assets;
    warnings: [..] string;

    defer {
        for warnings free(it);

        array_free(warnings);
    }

    html := parse_and_expand("[{{ asset nope.css }}]", .{}, *assets, *warnings);

    expect_equal(html, "[]");
    expect_equal(warnings.count, 1);
    expect(contains(warnings[0], "nope.css"));
} @test

template_mistakes_name_the_line :: () {
    _, error := parse_template("fine\nfine\n{{ title");

    expect_equal(error, "line 3: `{{` is never closed");
    free(error);

    _, error = parse_template("a\n{{ colour }}");
    expect_equal(error, "line 2: unknown placeholder `colour`");
    free(error);

    _, error = parse_template("{{ asset }}");
    expect_equal(error, "line 1: `asset` needs a file name");
    free(error);

    _, error = parse_template("{{ title now }}");
    expect_equal(error, "line 1: `title` takes no argument");
    free(error);
} @test

the_built_in_template_parses :: () {
    template, error := parse_template(DEFAULT_TEMPLATE);

    defer free_template(*template);

    expect_equal(error, "");
    expect(template.parts.count > 0);
} @test
