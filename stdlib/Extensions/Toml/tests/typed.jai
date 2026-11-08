#import "Basic";
#import "String";
#import "Extensions/Toml";

Mode :: enum {
    FAST;
    SLOW_START;
}

Point :: struct {
    x: float64;
    y: float64;
}

Limits :: struct {
    cpu: s32;
    memory: u64;
    ratio: float32; @toml(default)
}

Server :: struct {
    host: string;
    port: int;
    tags: [..] string; @toml(default)
    limits: Limits; @toml(default)
}

Item :: struct {
    name: string;
    points: [..] Point; @toml(default)
}

Config :: struct {
    title: string;
    enabled: bool; @toml(default)
    retries: s8 = 3; @toml(default)
    mode: Mode; @toml(default, rename=run_mode)
    nickname: *string; @toml(default)
    ratios: [] float64; @toml(default)
    when: Toml_Datetime; @toml(default)
    server: Server; @toml(default)
    items: [..] Item; @toml(default)
    secret: string; @toml(skip)
    renamed_field: string; @toml(default, rename="fancy key")
    extra: [..] Toml_Unknown; @toml(unknown)
}

Strict :: struct @toml(deny_unknown_fields) {
    a: int;
    b: int; @toml(default)
}

Kebab :: struct @toml(rename_all=kebab-case) {
    max_width: int;
    tab_size: int; @toml(default)
}

Ranged :: struct {
    width: int; @toml(range=1..16, message="width must be an integer from 1 to 16")
}

Wrapper :: struct {
    inner: Ranged; @toml(default)
}

Defaults :: struct @toml(default) {
    name := "guest";
    count := 7;
}

Stats :: struct {
    live: int;
}

count_live :: (mode: Allocator_Mode, size: s64, old_size: s64, memory: *void, data: *void) -> *void {
    stats := cast(*Stats) data;
    backing := context.default_allocator;

    if mode == .ALLOCATE || (mode == .RESIZE && !memory) stats.live += 1;
    if mode == .FREE && memory stats.live -= 1;

    return backing.proc(mode, size, old_size, memory, backing.data);
}

// Everything a decode allocates, on success and on every failure, is released again.
check_balanced :: (text: string, options := Toml_Options.{}) {
    stats: Stats;

    {
        push_allocator(count_live, *stats);
        config, error := from_toml(Config, text, options);
        free_toml_value(*config);
        _ = error;
    }

    assert(stats.live == 0, tprint("% leaked % allocations", text, stats.live));
}

decode :: (text: string, options := Toml_Options.{}) -> Config, Toml_Error {
    return from_toml(Config, text, options);
}

check_kind :: (text: string, kind: Toml_Error_Kind, line := 0, path := "") {
    config, error := decode(text);

    assert(error.kind == kind, tprint("% gave %: %", text, error.kind, error.message));
    if line assert(error.line == line, tprint("% line %", text, error.line));
    if path.count assert(toml_error_path(*error) == path, tprint("% path %", text, toml_error_path(*error)));

    // A failed decode leaves a default value that owns nothing.
    assert(config.title.count == 0 && config.items.count == 0);
}

main :: () {
    source :: #string TOML
title = "demo"
enabled = true
retries = -4
run_mode = "slow_start"
nickname = "neo"
ratios = [1.5, 2, 0x10]
when = 2024-03-01T10:20:30.25Z
"fancy key" = "yes"
unknown_top = { a = 1 }

[server]
host = 'localhost'
port = 8080
tags = ["a", "b"]

[server.limits]
cpu = 2
memory = 1_024

[[items]]
name = "first"

[[items.points]]
x = 1.0
y = 2.0

[[items.points]]
x = 3
y = 4

[[items]]
name = "second"
TOML;

    config, error := decode(source);

    assert(error.kind == .NONE, tprint("%: %", error.line, error.message));
    assert(config.title == "demo" && config.enabled && config.retries == -4);
    assert(config.mode == .SLOW_START && config.nickname && config.nickname.* == "neo");
    assert(config.ratios.count == 3 && config.ratios[1] == 2 && config.ratios[2] == 16);
    assert(config.when.kind == .OFFSET_DATETIME && config.when.value.year == 2024 && config.when.value.fraction == "25");
    assert(config.renamed_field == "yes");
    assert(config.server.host == "localhost" && config.server.port == 8080 && config.server.tags.count == 2 && config.server.tags[1] == "b");
    assert(config.server.limits.cpu == 2 && config.server.limits.memory == 1024);
    assert(config.items.count == 2 && config.items[0].points.count == 2 && config.items[1].points.count == 0);
    assert(config.items[0].points[1].x == 3 && config.items[0].points[1].y == 4 && config.items[1].name == "second");
    assert(config.extra.count == 1 && config.extra[0].path == "unknown_top" && !config.extra[0].header);
    assert(slice(source, config.extra[0].start, config.extra[0].end - config.extra[0].start) == "unknown_top = { a = 1 }");

    // What it writes decodes to the same value, and the DOM sees the same document.
    text, write_error := to_toml(config);

    assert(write_error.kind == .NONE, write_error.message);
    again, again_error := decode(text);

    assert(again_error.kind == .NONE, tprint("%: %", again_error.message, text));
    assert(again.title == "demo" && again.items.count == 2 && again.items[0].points[1].x == 3);
    assert(again.mode == .SLOW_START && again.ratios.count == 3 && again.server.tags.count == 2 && again.renamed_field == "yes");
    doc, doc_error := parse_toml(text);

    assert(doc_error.kind == .NONE);
    assert(toml_get(toml_get(doc.root, "server"), "port").integer == 8080);
    free_toml(*doc);
    free(text);
    free_toml_value(*again);
    free_toml_value(*config);
    assert(config.title.count == 0 && config.items.count == 0 && config.server.tags.count == 0);

    // Defaults stay in place and are never freed; missing required keys name the key.
    only_title, only_error := decode("title = 'x'\n");

    assert(only_error.kind == .NONE && only_title.retries == 3 && only_title.nickname == null && !only_title.enabled);
    free_toml_value(*only_title);
    check_kind("enabled = true\n", .TYPE, 0, "$[\"title\"]");
    check_kind("title = 'x'\n[server]\nhost = 'h'\n", .TYPE, 0, "$[\"server\"][\"port\"]");

    // Wrong types and ranges.
    check_kind("title = 1\n", .TYPE, 1, "$[\"title\"]");
    check_kind("title = 'x'\nenabled = 'yes'\n", .TYPE, 2);
    check_kind("title = 'x'\nretries = 300\n", .RANGE, 2);
    check_kind("title = 'x'\nrun_mode = 'turbo'\n", .TYPE, 2);
    check_kind("title = 'x'\nratios = [1, 'a']\n", .TYPE, 2, "$[\"ratios\"][1]");
    check_kind("title = 'x'\nitems = [{ name = 'a' }, { nme = 'b' }]\n", .TYPE, 2);
    check_kind("title = 'x'\nserver = 3\n", .TYPE, 2);
    check_kind("title = 'x'\n[retries]\n", .TYPE, 2);
    check_kind("title = 'x'\nretries = 99999999999999999999\n", .RANGE, 2);

    // TOML's declaration rules are enforced without a document.
    check_kind("title = 'x'\ntitle = 'y'\n", .DUPLICATE, 2, "$[\"title\"]");
    check_kind("title = 'x'\n[server]\nhost = 'a'\nport = 1\n[server]\n", .DUPLICATE, 5);
    check_kind("title = 'x'\nserver.host = 'a'\nserver.port = 1\n[server]\n", .DUPLICATE, 4);
    check_kind("title = 'x'\nserver = { host = 'a', port = 1 }\nserver.port = 2\n", .DUPLICATE, 3);
    check_kind("title = 'x'\nserver = { host = 'a', host = 'b', port = 1 }\n", .DUPLICATE, 2);
    check_kind("title = 'x'\nitems = [{ name = 'a' }]\n[[items]]\nname = 'b'\n", .DUPLICATE, 3);
    check_kind("title = 'x'\n[[server]]\n", .TYPE, 2);
    check_kind("title = 'x'\n[server]\nhost = 'a'\nport = 1\n[[server]]\n", .DUPLICATE, 5);
    check_kind("title = 'x'\n[unknown_a.b]\nx = 1\n[unknown_a]\ny = 1\n[unknown_a.b]\n", .DUPLICATE, 6);
    check_kind("title = 'x'\nu = { a = 1, a = 2 }\n", .DUPLICATE, 2);
    check_kind("title = 'x'\nu = 1\n[u]\n", .DUPLICATE, 3);
    check_kind("title = 'x'\n[[u]]\n[u]\n", .DUPLICATE, 3);
    reordered, reordered_error := decode("title = 'x'\n[server.limits]\ncpu = 1\nmemory = 1\n[server]\nhost = 'h'\nport = 1\n");

    assert(reordered_error.kind == .NONE && reordered.server.port == 1 && reordered.server.limits.cpu == 1);
    free_toml_value(*reordered);
    check_kind("title = 'x'\n[[items]]\nname = 'a'\n[items.points]\n", .TYPE, 4);
    check_kind("title = 'x'\n[[items]]\nname = 'a'\n[[items.points]]\nx = 1\ny = 1\n[items.points]\n", .DUPLICATE, 7);

    // Syntax errors are the parser's, wherever they are, including inside skipped values.
    check_kind("title = 'x'\nu = [1, 2\n", .SYNTAX);
    check_kind("title = 'x'\nu = 2000-13-01\n", .RANGE);
    check_kind("title = 'x'\nu = 'a' 'b'\n", .SYNTAX);
    check_kind("title 'x'\n", .SYNTAX, 1);

    // The parser's limits apply.
    config, error = decode("title = 'abc'\n", .{ max_string_bytes = 2 });
    assert(error.kind == .LIMIT);
    _, error = decode("title = 'x'\nu = [[[[1]]]]\n", .{ max_depth = 3 });
    assert(error.kind == .LIMIT);
    _, error = decode("title = 'x'\nu = [1, 2, 3, 4]\n", .{ max_nodes = 4 });
    assert(error.kind == .LIMIT);
    _, error = decode("title = 'abcdef'\n", .{ max_retained_bytes = 3 });
    assert(error.kind == .LIMIT);
    _, error = decode("title = 'abcdef'\n", .{ max_input_bytes = 3 });
    assert(error.kind == .LIMIT);
    bytes := u8.[116, 105, 116, 108, 101, 32, 61, 32, 39, 0xc0, 0x80, 39];

    _, error = decode(string.{ bytes.count, bytes.data });
    assert(error.kind == .ENCODING);

    // Unknown keys: collected with their spans, or refused.
    config, error = decode("title = 't'\n[nest.deep]\nx = 1\n[nest]\ny = 2\n[other]\n[[arr]]\n[[arr]]\n");
    assert(error.kind == .NONE, error.message);
    paths: [..] string;
    for config.extra array_add(*paths, it.path);

    assert(paths.count == 5 && paths[0] == "nest.deep" && paths[1] == "nest" && paths[2] == "other" && paths[3] == "arr" && paths[4] == "arr", tprint("%", paths));
    assert(config.extra[0].header && config.extra[2].header);
    free_toml_value(*config);
    _, error = decode("title = 't'\nx = 1\n", .{ deny_unknown_fields = true });
    assert(error.kind == .TYPE && error.line == 2);

    strict, strict_error := from_toml(Strict, "a = 1\nc = 2\n");

    assert(strict_error.kind == .TYPE && strict_error.line == 2 && toml_error_path(*strict_error) == "$[\"c\"]", tprint("% % %", strict_error.kind, strict_error.line, toml_error_path(*strict_error)));
    strict, strict_error = from_toml(Strict, "a = 1\n");
    assert(strict_error.kind == .NONE && strict.a == 1);

    kebab, kebab_error := from_toml(Kebab, "max-width = 80\ntab-size = 2\n");

    assert(kebab_error.kind == .NONE && kebab.max_width == 80 && kebab.tab_size == 2);
    kebab_text, kebab_write := to_toml(kebab);

    assert(kebab_write.kind == .NONE && kebab_text == "max-width = 80\ntab-size = 2\n", kebab_text);
    free(kebab_text);

    // A field note chooses the error message and bounds.
    wrapped, wrapped_error := from_toml(Wrapper, "[inner]\nwidth = 40\n");

    assert(wrapped_error.kind == .RANGE && wrapped_error.message == "width must be an integer from 1 to 16" && wrapped_error.line == 2);
    wrapped, wrapped_error = from_toml(Wrapper, "inner.width = 'x'\n");
    assert(wrapped_error.kind == .TYPE && wrapped_error.message == "width must be an integer from 1 to 16");
    wrapped, wrapped_error = from_toml(Wrapper, "inner.width = 16\n");
    assert(wrapped_error.kind == .NONE && wrapped.inner.width == 16);

    // A struct note makes every field optional, and omitted fields keep their declared default.
    defaults, defaults_error := from_toml(Defaults, "");

    assert(defaults_error.kind == .NONE && defaults.name == "guest" && defaults.count == 7);
    defaults, defaults_error = from_toml(Defaults, "name = 'ann'\n");
    assert(defaults_error.kind == .NONE && defaults.name == "ann" && defaults.count == 7);
    free_toml_value(*defaults);
    assert(defaults.name == "guest");

    // Encoding: headers for tables, array-of-tables headers, shortest floats, and skipped fields.
    value: Config;
    value.title = "t\"q";
    value.secret = "never";
    value.server.host = "h";
    value.server.port = 1;
    array_add(*value.server.tags, "x");
    value.ratios = float64.[0.1];
    array_add(*value.items, Item.{ name = "i" });
    array_add(*value.items[0].points, Point.{ 1, 2.5 });
    out, out_error := to_toml(value);

    assert(out_error.kind == .NONE);
    assert(contains(out, "[server]\nhost = \"h\"\nport = 1\ntags = [\"x\"]\n"), out);
    assert(contains(out, "\n[server.limits]\n"), out);
    assert(contains(out, "\n[[items]]\nname = \"i\"\n\n[[items.points]]\nx = 1.0\ny = 2.5\n"), out);
    assert(contains(out, "ratios = [0.1]\n") && !contains(out, "secret") && !contains(out, "17"), out);
    free(out);

    sci, sci_error := to_toml(Point.{ 1.0e300, 5.0e-324 });

    assert(sci_error.kind == .NONE);
    back, back_error := from_toml(Point, sci);

    assert(back_error.kind == .NONE && back.x == 1.0e300 && back.y == 5.0e-324, sci);
    free(sci);

    // No leaks, whether the input is accepted or fails anywhere.
    check_balanced(source);
    for string.["title = 'x'\nu = [1, [2, { a = 'q' }]]\n", "title = 'x'\nitems = [{ name = 'a', points = [{ x = 1, y = 2 }] }]\n", "title = 'x'\nitems = [{ name = 'a' }, { nme = 'b' }]\n", "title = 'x'\nserver.tags = ['a', 3]\n", "title = 'x'\n[[items]]\nname = 'a'\n[[items]]\n", "title = 'x'\n[[items]]\nname = 'a'\n[[items.points]]\nx = 1\n[[items]]\nname = 'b'\n[[items.points]]\nx = 1\ny = 2\n", "\"\\u0061\" = 1\ntitle = 'x'\n[\"a\\u0062\"]\nz = 1\n[[\"a\\u0062c\"]]\n", "nickname = 'n'\ntitle = 1\n", "when = 2024-03-01T10:20:30.25Z\ntitle = 1\n", "title = 'x'\nratios = [1, 2, 'a']\n", "title = 'x'\nu = { a = 'z', a = 'z' }\n"] check_balanced(it);

    print("ok\n");
}
