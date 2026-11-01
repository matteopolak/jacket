#import "Basic";
#import "Extensions/Web";
#import "Extensions/Testing";
F :: #import "File";
FU :: #import "File_Utilities";

Module_Page :: struct {
    name: string;
}

Item_Page :: struct {
    name: string;
}

Side :: (dest: *Out, page: *Module_Page) {
    html(dest, "<aside>module {page.name}</aside>");
}

Side :: (dest: *Out, page: *Item_Page) {
    html(dest, "<aside>item {page.name}</aside>");
}

Shell :: (dest: *Out, page: *$P) {
    html(dest, "<main>{Side(page)}</main>");
}

Empty :: (dest: *Out) {
    html(dest, "<hr>");
}

label :: (n: int) -> string {
    return "A & B";
}

contexts_and_types :: () {
    out: Out;

    defer deinit(*out);
    s := "<&>\"'";
    safe := cast(Html) ("<b>safe</b>");
    n: s8 = -12;
    u: u64 = 18446744073709551615;

    html(*out, "<p>{s}{safe}{n}/{u}/{1 + 2}/{label(1)}</p><a title=\"{s}\" data-x='{s}'></a><script>{cast(Html) (\"x < 2\")}</script><style>{cast(Html) (\"a > b {}\")}</style>");
    expect_equal(to_string(*out), "<p>&lt;&amp;&gt;\"'<b>safe</b>-12/18446744073709551615/3/A &amp; B</p><a title=\"&lt;&amp;&gt;&quot;&#39;\" data-x='&lt;&amp;&gt;&quot;&#39;'></a><script>x < 2</script><style>a > b {}</style>");
    reset(*out);
    a: s16 = -32768;
    b: s32 = -123;
    c: s64 = -456;
    d: u8 = 255;
    e: u16 = 65535;
    f: u32 = 4294967295;

    html(*out, "{a}/{b}/{c}/{d}/{e}/{f}");
    expect_equal(to_string(*out), "-32768/-123/-456/255/65535/4294967295");
} @test

components_and_whitespace :: () {
    out: Out;

    defer deinit(*out);
    page := Module_Page.{ "<&" };
    item := Item_Page.{ "hi" };

    html(*out, " \n {Shell(*page)}\t{Shell(*item)}{Empty()}\n{{a} } ");
    expect_equal(to_string(*out), " \n <main><aside>module &lt;&amp;</aside></main>\t<main><aside>item hi</aside></main><hr>\n{a} } ");
} @test

controls_and_links :: () {
    out: Out;

    defer deinit(*out);
    values := int.[0, 1, 2];
    yes := true;
    url := "x?a=1&b=2";

    html(*out, "<a{if yes} class=\"on\"{else} title=\"off\"{end} href=\"/d/{url}\">{for n: values}{if n == 0}zero{else if n == 1}{for values}{it_index}{end}{else}{n}{end}{end}</a><img src='pic.png'>");
    expect_equal(to_string(*out), "<a class=\"on\" href=\"/d/x?a=1&amp;b=2\">zero0122</a><img src='pic.png'>");
    expect_equal(out.links.count, 2);
    expect_equal(out.links[0], "/d/x?a=1&b=2");
    expect_equal(out.links[1], "pic.png");
    reset(*out);
    expect_equal(out.links.count, 0);
    out.record_links = false;
    html(*out, "<a href=\"{url}\">a</a>");
    expect_equal(out.links.count, 0);
    expect_equal(to_string(*out), "<a href=\"x?a=1&amp;b=2\">a</a>");
} @test

folder_links_resolve_to_their_index :: () {
    w := writer_init("unused", "/d/", true);

    defer deinit(*w);
    out: Out;

    defer deinit(*out);
    html(*out, "<a href=\"/d/api/A/\">A</a><a href=\"/d/\">home</a>");
    expect(write_page(*w, "", *out));
    reset(*out);
    expect(write_page(*w, "api/A/", *out));
    problems := check_links(*w);

    defer array_free(problems);
    expect_equal(problems.count, 0);
} @test

html_and_dangling_links_are_reported :: () {
    w := writer_init("unused", "/d/", true);

    defer deinit(*w);
    out: Out;

    defer deinit(*out);
    html(*out, "<a href=\"/d/api/A.html\">A</a><a href=\"/d/api/B/\">B</a><a href=\"https://x.test/a.html\">x</a><a href=\"/d/api/B/\">duplicate</a>");
    expect(write_page(*w, "", *out));
    problems := check_links(*w);

    defer {
        for problems free(it);
        array_free(problems);
    }

    expect_equal(problems.count, 2);
    expect_equal(problems[0], "index.html: link `/d/api/A.html` ends in .html");
    expect_equal(problems[1], "index.html: link `/d/api/B/` leads to no written file");
    partial := check_links(*w, resolve = false);

    defer {
        for partial free(it);
        array_free(partial);
    }

    expect_equal(partial.count, 1);
} @test

link_rules_and_assets :: () {
    w := writer_init("unused", "/d/", true);

    defer deinit(*w);
    out: Out;

    defer deinit(*out);
    html(*out, "<a href=\"#here\"></a><a href=\"\"></a><a href=\"/other/\"></a><a href=\"https://x/a.html\"></a><a href=\"/d/a.css?v=1#x\"></a><a href=\"a.css\"></a><a href=\"/d/?x=1\"></a>");
    expect(write_page(*w, "", *out));
    expect(write_file(*w, "a.css", "x"));
    problems := check_links(*w);

    defer array_free(problems);
    expect_equal(problems.count, 0);
    expect(!write_file(*w, "../escape", ""));
    expect_equal(w.errors.count, 1);
} @test

render_page :: (out: *Out, n: *int, w: *Writer) {
    html(out, "<p>{n.*}</p><a href=\"/d/\">home</a>");
    assert(write_page(w, tprint("%/", n.*), out));
}

parallel_matches_serial :: () {
    // The runner gives each test process its own working directory.
    directory := "web-output-test";

    defer FU.delete_directory_tree(directory);
    serial := writer_init(tprint("%/serial", directory), "/d/");
    parallel := writer_init(tprint("%/parallel", directory), "/d/");

    defer deinit(*serial);
    defer deinit(*parallel);
    jobs: [24] int;
    out: Out;

    defer deinit(*out);
    for *jobs it.* = it_index;

    for *jobs {
        render_page(*out, it, *serial);
        reset(*out);
    }

    render_parallel(jobs, render_page, *parallel, threads = 3);
    expect_equal(parallel.errors.count, 0);

    for jobs {
        a, ok_a := F.read_entire_file(tprint("%/serial/%/index.html", directory, it),, temp);
        b, ok_b := F.read_entire_file(tprint("%/parallel/%/index.html", directory, it),, temp);
        expect(ok_a && ok_b);
        expect_equal(a, b);
    }
} @test

stamp_vectors :: () {
    expect_equal(content_stamp(""), "cbf29ce4");
    expect_equal(content_stamp("a"), "af63dc4c");
    expect_equal(content_stamp("hello"), "a430d846");
    expect_equal(content_stamp("foobar"), "85944171");
    expect_equal(content_stamp("π\0\n"), "144e4ee7");
} @test

compact_assets :: () {
    expect_equal(compact_css(" /* start */\n  a { color: red; }  \n\n"), "a { color: red; }\n");
    expect_equal(compact_css(" a { content: '/* keep */'; x: \"a\\\"/* keep */\"; } /* drop */"), "a { content: '/* keep */'; x: \"a\\\"/* keep */\"; }\n");
    expect_equal(compact_css("a/**/b {}"), "a b {}\n");
    expect_equal(compact_js("  // drop\n\n  let u = 'https://x//y'; // keep\n  let r = /a\\/\\/b/;\n"), "let u = 'https://x//y'; // keep\nlet r = /a\\/\\/b/;\n");
    expect_equal(compact_js("let s = `a\n  // keep\n  b`;\n"), "let s = `a\n  // keep\n  b`;\n");
} @test

branch_states_and_tokenizer :: () {
    out: Out;

    defer deinit(*out);
    yes := false;
    __web := "caller";

    html(*out, "{if yes}<p>{else if false}<p>{else}<p>{end}{__web}</p><!-- <bad> --><SCRIPT>{cast(Html) (\"x = '</other>';\")}</SCRIPT><A HREF = '/d/'></A>");
    expect_equal(to_string(*out), "<p>caller</p><!-- <bad> --><SCRIPT>x = '</other>';</SCRIPT><A HREF = '/d/'></A>");
    expect_equal(out.links.count, 1);
    expect_equal(out.links[0], "/d/");
    reset(*out);
    html(*out, #string HTML
  <p>
    exact
  </p>
HTML
    );
    expect_equal(to_string(*out), "  <p>\n    exact\n  </p>\n");
} @test

large_page_storage_is_reused :: () {
    out: Out;

    defer deinit(*out);
    text := alloc_string(100_000);

    defer free(text);
    for *text it.* = #char "x";

    html(*out, "<a href=\"{text}\">{text}</a>");
    first := out.cached.count;

    reset(*out);

    for 1..10 {
        html(*out, "<a href=\"{text}\">{text}</a>");
        expect_equal(out.links[0], text);
        expect_equal(out.cached.count, first);
        reset(*out);
    }
} @test

writer_errors_and_dry_run :: () {
    directory := "web-dry-test";

    expect(!FU.file_exists(directory));
    w := writer_init(directory, "/", true);

    defer deinit(*w);
    out: Out;

    defer deinit(*out);
    expect(write_page(*w, "", *out));
    expect(write_file(*w, "a/b.txt", "bytes"));
    expect(!FU.file_exists(directory));
    blocking := "web-blocking-file";

    assert(F.write_entire_file(blocking, "x"));
    defer F.file_delete(blocking);
    bad := writer_init(blocking, "/");

    defer deinit(*bad);
    expect(!write_file(*bad, "a/b.txt", "x"));
    expect_equal(bad.errors.count, 1);
    expect_equal(bad.files.count, 0);
} @test

compact_lexical_edges :: () {
    expect_equal(compact_js("  let r = /['\"/]/;  \n  // drop\n  next();\n"), "let r = /['\"/]/;  \nnext();\n");
    expect_equal(compact_js("  /* start\n  // keep\n  end */\n  next();\n"), "/* start\n  // keep\n  end */\nnext();\n");
} @test
