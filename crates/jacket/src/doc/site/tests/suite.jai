#load "../theme.jai";
#import "Extensions/Testing";

fixture :: ($T: Type, name: string) -> T {
    text, ok := F.read_entire_file(tprint("%fixtures/%.json", #filepath, name));
    assert(ok);
    defer free(text);
    value, error := from_json(T, text);
    assert(error.kind == .NONE, error_text(error));
    return value;
}

render :: (site: *Site_View, view: *Page_View, module: *Module_View = null) -> string {
    out: Out;
    defer deinit(*out);
    Render(*out, site, view, module);
    return copy_string(to_string(*out));
}

the_experimental_badge_is_beside_what_is_tagged :: () {
    site := fixture(Site_View, "site");
    moving := fixture(Module_View, "moving");
    steady := fixture(Module_View, "steady");
    defer {
        free_json(*site);
        free_json(*moving);
        free_json(*steady);
    }
    // main.jai stamps asset URLs; the fixture has none.
    site.categories_url = copy_string("/assets/categories.css?v=00000000");
    a := render(*site, *moving.page, *moving);
    b := render(*site, *steady.page, *steady);
    tagged := render(*site, *moving.items[0], *moving);
    inherited := render(*site, *moving.items[1], *moving);
    untagged := render(*site, *steady.items[1], *steady);
    defer {
        free(a);
        free(b);
        free(tagged);
        free(inherited);
        free(untagged);
    }
    expect(contains(a, "<span class=\"badge exp\""));
    expect(!contains(b, "<span class=\"badge exp\""));
    expect(contains(tagged, "<span class=\"badge exp\""));
    expect(contains(inherited, "badge exp"));
    expect(!contains(untagged, "badge exp"));
} @test

a_category_badge_marks_its_modules_and_their_items :: () {
    site := fixture(Site_View, "site");
    moving := fixture(Module_View, "moving");
    steady := fixture(Module_View, "steady");
    defer {
        free_json(*site);
        free_json(*moving);
        free_json(*steady);
    }
    // main.jai stamps asset URLs; the fixture has none.
    site.categories_url = copy_string("/assets/categories.css?v=00000000");
    a := render(*site, *moving.page, *moving);
    item := render(*site, *moving.items[1], *moving);
    b := render(*site, *steady.page, *steady);
    home := render(*site, *site.index);
    defer {
        free(a);
        free(item);
        free(b);
        free(home);
    }
    badge := "<span class=\"badge cat-1\" title=\"Not part of the library\">extension</span>";
    expect(contains(a, badge));
    expect(contains(item, badge));
    expect(!contains(b, badge));
    expect(contains(home, "<link rel=\"stylesheet\" href=\"/assets/categories.css?v=00000000\">"));
    colors: Out;
    defer deinit(*colors);
    CategoryStyle(*colors, *site);
    expect(contains(to_string(*colors), ".cat-1{--c:#1f7a8c}"));
} @test

lists_mark_experimental_modules_and_items :: () {
    site := fixture(Site_View, "site");
    steady := fixture(Module_View, "steady");
    defer {
        free_json(*site);
        free_json(*steady);
    }
    list := render(*site, *steady.page, *steady);
    defer free(list);
    first := find_index_from_left(list, "class=\"badge exp sm\"");
    expect(first >= 0);
    expect(find_index_from_left(list, "class=\"badge exp sm\"", first + 1) < 0);
    expect(contains(list, "Moving<span class=\"dot exp\""));
    expect(contains(list, "<button type=\"button\" class=\"pf full\" data-pf=\"macOS\""));
} @test

foreign_declarations_follow_the_own_items_and_have_pages :: () {
    site := fixture(Site_View, "site");
    vendor := fixture(Module_View, "vendor");
    defer {
        free_json(*site);
        free_json(*vendor);
    }
    page := render(*site, *vendor.page, *vendor);
    defer free(page);
    own := find_index_from_left(page, "id=\"own\"");
    max := find_index_from_left(page, "id=\"MAX\"");
    expect(own >= 0 && max > own);
    expect(contains(advance(page, max), "class=\"badge sm foreign\""));
    expect(contains(page, "FAST"));
    expect(contains(page, "href=\"/api/Vendor/MAX/\""));
} @test

module_parameters_have_a_section_before_the_items :: () {
    site := fixture(Site_View, "site");
    steady := fixture(Module_View, "steady");
    moving := fixture(Module_View, "moving");
    defer {
        free_json(*site);
        free_json(*steady);
        free_json(*moving);
    }
    page := render(*site, *steady.page, *steady);
    other := render(*site, *moving.page, *moving);
    defer {
        free(page);
        free(other);
    }
    at := find_index_from_left(page, "id=\"parameters\"");
    expect(at >= 0 && at < find_index_from_left(page, "id=\"constants\""));
    expect(contains(page, "<span class=\"pname\">STRICT</span><span class=\"colon\">: </span><span class=\"ty\">bool</span><span class=\"colon\"> = </span><span class=\"k\">false</span>"));
    expect(contains(page, "Checks more."));
    expect(contains(page, "href=\"#parameters\""));
    expect(!contains(other, "Module parameters"));
} @test

examples_and_legacy_escaping :: () {
    out: Out;
    defer deinit(*out);
    note := "Can't run \"x\" & y";
    e := Example.{ code_html = "<b>x</b>", note = *note, show_note = true };
    ExampleBlock(*out, *e);
    expect(contains(to_string(*out), "disabled title=\"Can't run &quot;x&quot; &amp; y\""));
    expect(contains(to_string(*out), "<p class=\"ex-note\">Can't run &quot;x&quot; &amp; y</p>"));
    reset(*out);
    id := "abc";
    e.id = *id;
    e.compact = true;
    e.show_note = false;
    ExampleBlock(*out, *e);
    expect(contains(to_string(*out), "ex compact"));
    expect(contains(to_string(*out), "data-act=\"reset\""));
    expect(contains(to_string(*out), "data-program=\"abc\""));
    reset(*out);
    e.plain = true;
    ExampleBlock(*out, *e);
    expect_equal(to_string(*out), "<figure class=\"ex plain\"><pre class=\"code\"><code><b>x</b></code></pre></figure>");
} @test

sidebar_windows_and_toc :: () {
    site := fixture(Site_View, "site");
    moving := fixture(Module_View, "moving");
    defer {
        free_json(*site);
        free_json(*moving);
    }
    side := get_pointer(*moving.items[1].sidebar, Sidebar_Item);
    side.start = 1;
    page := render(*site, *moving.items[1], *moving);
    defer free(page);
    expect(!contains(page, "href=\"/api/Extensions/Moving/moving/\""));
    expect(contains(page, "href=\"/api/Extensions/Moving/steady/\" aria-current=\"page\""));
    out: Out;
    defer deinit(*out);
    items := Outline_Item.[.{ id = "a", label = "A" }, .{ id = "b", label = "B" }];
    entry := variant(Outline, Outline_Items.{ items });
    OutlineEntry(*out, *entry);
    expect(contains(to_string(*out), "<li class=\"items\"><ul>"));
    expect(contains(to_string(*out), "<li><a href=\"#b\">B</a></li>"));
} @test

head_metadata_escapes_attributes_and_preserves_json :: () {
    site := fixture(Site_View, "site");
    view := fixture(Page_View, "head");
    defer {
        free_json(*site);
        free_json(*view);
    }
    out: Out;
    defer deinit(*out);
    HeadMeta(*out, *site, *view);
    text := to_string(*out);
    expect(contains(text, "<meta name=\"description\" content=\"A &lt;description&gt; &amp; &quot;quote&quot;\">"));
    expect(contains(text, "<meta property=\"og:title\" content=\"&lt;/script&gt;&quot;&amp;\">"));
    expect(contains(text, "<meta property=\"og:description\" content=\"A &lt;description&gt; &amp; &quot;quote&quot;\">"));
    expect(contains(text, "<meta property=\"og:site_name\" content=\"fixture &amp; &quot;docs&quot;\">"));
    expect(contains(text, "<meta property=\"og:type\" content=\"article&quot;&amp;\">"));
    expect(contains(text, "<meta property=\"og:url\" content=\"https://example.com/?a=1&amp;b=&quot;url&quot;\">"));
    expect(contains(text, "<meta property=\"og:image\" content=\"https://example.com/image?x=&quot;&amp;\">"));
    expect(contains(text, "<meta name=\"twitter:card\" content=\"summary_large_image&quot;&amp;\">"));
    expect(contains(text, "<link rel=\"canonical\" href=\"https://example.com/?a=1&amp;b=&quot;canonical&quot;\">"));
    expect(contains(text, "<meta name=\"robots\" content=\"noindex\">"));
    expect(contains(text, tprint("<script type=\"application/ld+json\">%</script>", view.head.json_ld)));
    expect(contains(text, "<\\/script>"));
    // The shell actually dispatches through HeadMeta too.
    page := render(*site, *view);
    defer free(page);
    expect(contains(page, text));
} @test

head_metadata_omits_unset_values :: () {
    site: Site_View;
    view: Page_View;
    out: Out;
    defer deinit(*out);
    HeadMeta(*out, *site, *view);
    expect_equal(to_string(*out), "");
    view.head.twitter_card = "summary";
    HeadMeta(*out, *site, *view);
    expect_equal(to_string(*out), "<meta name=\"twitter:card\" content=\"summary\">");
} @test

icons_put_the_fragment_after_the_stamp :: () {
    site := fixture(Site_View, "site");
    defer free_json(*site);
    site.icons_url = copy_string("/assets/icons.svg?v=12345678");
    site.repo = copy_string("https://example.com");
    page := render(*site, *site.index);
    defer free(page);
    for name: string.["search", "system", "chevron", "repo"] {
        expect(contains(page, tprint("href=\"/assets/icons.svg?v=12345678#%\"", name)));
    }
    expect(!contains(page, "assets/icons.svg#"));
} @test

shared_item_navigation_keeps_a_focused_fallback_window :: () {
    links: [12] Nav_Link;
    for *link, n: links {
        link.href = tprint("/api/Test/item%/", n);
        link.label = tprint("item%", n);
    }
    groups := [] Nav_Link.[links];
    module: Module_View;
    module.page.path = "api/Test/index.html";
    module.nav = groups;
    writer := writer_init("unused", "/", dry_run = true);
    defer deinit(*writer);
    nav_out: Out;
    defer deinit(*nav_out);
    stamps := WriteItemNav(*nav_out, *module, *writer);
    defer {
        for stamp: stamps free(stamp);
        free(stamps.data);
    }
    side := Sidebar_Item.{ start = 0, end = 12, current = 6 };
    out: Out;
    defer deinit(*out);
    ItemNavList(*out, *side, *module, stamps);
    expect(contains(to_string(*out), tprint("data-source=\"assets/nav/api/Test/0.html?v=%\"", stamps[0])));
    expect(contains(to_string(*out), "data-start=\"0\" data-end=\"12\" data-first=\"4\""));
    reset(*out);
    ItemNav(*out, *side, *module);
    text := to_string(*out);
    expect(!contains(text, ">item3<"));
    expect(contains(text, ">item4<"));
    expect(contains(text, "href=\"/api/Test/item6/\" aria-current=\"page\""));
    expect(contains(text, ">item8<"));
    expect(!contains(text, ">item9<"));
    reset(*out);
    side.current = 0;
    ItemNav(*out, *side, *module);
    expect(contains(to_string(*out), ">item2<"));
    expect(!contains(to_string(*out), ">item3<"));
    reset(*out);
    side.start = 9;
    side.current = 11;
    ItemNavList(*out, *side, *module, stamps);
    expect(!contains(to_string(*out), "data-source"));
    ItemNav(*out, *side, *module);
    expect(contains(to_string(*out), ">item9<"));
    expect(contains(to_string(*out), "aria-current=\"page\">item11<"));
} @test

shared_navigation_urls_track_the_written_bytes :: () {
    links := NewArray(8, Nav_Link);
    defer array_free(links);
    for *link, n: links {
        link.href = tprint("/api/Test/item%/", n);
        link.label = tprint("item%", n);
    }
    groups := [] Nav_Link.[links, links];
    module: Module_View;
    module.page.path = "api/Test/index.html";
    module.nav = groups;
    side := Sidebar_Item.{ start = 0, end = 8, current = 4 };
    writer := writer_init("unused", "/", dry_run = true);
    changed_writer := writer_init("unused", "/", dry_run = true);
    defer {
        deinit(*writer);
        deinit(*changed_writer);
    }
    out: Out;
    defer deinit(*out);
    stamps := WriteItemNav(*out, *module, *writer);
    defer {
        for stamp: stamps free(stamp);
        free(stamps.data);
    }
    // Independently render the complete shared file, not just the inline window.
    for link: links NavLink(*out, *link);
    expect_equal(stamps[0], content_stamp(to_string(*out)));
    reset(*out);
    ItemNavList(*out, *side, *module, stamps);
    before := copy_string(to_string(*out));
    defer free(before);
    expect(contains(before, tprint("data-source=\"assets/nav/api/Test/0.html?v=%\"", stamps[0])));
    reset(*out);
    changed_links := array_copy(links);
    defer array_free(changed_links);
    // This row is outside the fallback, but changing it must still change the URL.
    changed_links[0].label = "renamed";
    groups[0] = changed_links;
    changed := WriteItemNav(*out, *module, *changed_writer);
    defer {
        for stamp: changed free(stamp);
        free(changed.data);
    }
    expect(stamps[0] != changed[0]);
    expect_equal(stamps[1], changed[1]);
    ItemNavList(*out, *side, *module, changed);
    after := to_string(*out);
    expect(contains(after, tprint("data-source=\"assets/nav/api/Test/0.html?v=%\"", changed[0])));
    expect(before != after);
    expect(writer.errors.count == 0 && changed_writer.errors.count == 0);
} @test
