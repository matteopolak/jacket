#import "Basic";
#import "Hash_Table";
#import "Extensions/Toml";

// Hash_Table fields take tables (inline, dotted and `[header]` forms), Toml_Value fields take any
// value, and fixed-size arrays take `[[x]]` headers. Every path frees what it allocated.
Server :: struct {
    host: string;
    port: int; @toml(default)
}

Config :: struct {
    scores: Table(string, int); @toml(default)
    inline: Table(string, int); @toml(default)
    servers: Table(string, Server); @toml(default)
    groups: Table(string, [..] Server); @toml(default)
    nested: Table(string, Table(string, string)); @toml(default)
    extra: Toml_Value; @toml(default)
    pair: [2] Server; @toml(default)
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

check :: (text: string, kind: Toml_Error_Kind, path := "") {
    stats: Stats;

    {
        push_allocator(count_live, *stats);
        value, error := from_toml(Config, text);

        assert(error.kind == kind, tprint("% gave %: %", text, error.kind, error.message));
        if path.count assert(toml_error_path(*error) == path, tprint("% path %", text, toml_error_path(*error)));

        free_toml_value(*value);
    }

    assert(stats.live == 0, tprint("% leaked % allocations", text, stats.live));
}

maps :: () {
    source := "scores.a = 1\nscores.b = 2\nscores.c = 3\ninline = { a = 1 }\n\n[servers.alpha]\nhost = 'a'\n\n[servers.beta]\nhost = 'b'\nport = 9\n\n[[groups.web]]\nhost = 'w1'\n[[groups.web]]\nhost = 'w2'\nport = 2\n\n[nested.outer]\ninner = 'x'\n[nested.outer2]\ninner = 'y'\n";
    value, error := from_toml(Config, source);

    assert(error.kind == .NONE, tprint("%: %", error.line, error.message));
    assert(value.scores.count == 3 && table_find_pointer(*value.scores, "c").* == 3);
    assert(value.servers.count == 2 && table_find_pointer(*value.servers, "beta").port == 9);
    assert(table_find_pointer(*value.servers, "alpha").host == "a");

    web := table_find_pointer(*value.groups, "web");

    assert(web.count == 2 && web.*[1].host == "w2" && web.*[1].port == 2);

    outer := table_find_pointer(*value.nested, "outer");

    assert(outer != null && table_find_pointer(outer, "inner").* == "x");
    assert(table_find_pointer(*value.nested, "outer2") != null);

    // Written back, the maps are tables again.
    text, write_error := to_toml(value);

    assert(write_error.kind == .NONE, write_error.message);
    defer free(text);

    again, again_error := from_toml(Config, text);

    assert(again_error.kind == .NONE, tprint("%: %", again_error.message, text));
    assert(again.scores.count == 3 && table_find_pointer(*again.servers, "beta").port == 9);
    assert(table_find_pointer(*again.groups, "web").*.count == 2);
    assert(table_find_pointer(table_find_pointer(*again.nested, "outer"), "inner").* == "x");
    free_toml_value(*again);
    free_toml_value(*value);

    check(source, .NONE);
    check("scores = { a = 1, a = 2 }\n", .DUPLICATE);
    check("scores = { a = 1 }\nscores.b = 2\n", .DUPLICATE);
    check("[servers.a]\nhost = 'x'\n[servers.a]\n", .DUPLICATE);
    check("[scores]\na = 1\n[scores.b]\n", .TYPE);
    check("[scores]\na = 'x'\n", .TYPE, "$[\"scores\"][\"a\"]");
    check("[servers.a]\nport = 1\n", .TYPE, "$[\"servers\"][\"a\"][\"host\"]");
    check("[servers.a]\nhost = 'x'\nport = 'y'\n", .TYPE, "$[\"servers\"][\"a\"][\"port\"]");
    check("servers = 1\n", .TYPE);
    check("scores = [1]\n", .TYPE);
    check("[[groups.web]]\nhost = 'a'\n[[groups.web]]\nport = 2\n", .TYPE, "$[\"groups\"][\"web\"][1][\"host\"]");
    check("[[scores.a]]\n", .TYPE);
}

values :: () {
    source := "extra = { a = 1, list = [1, 'two', 3.5], nested = { ok = true } }\n";
    value, error := from_toml(Config, source);

    assert(error.kind == .NONE, error.message);
    assert(value.extra.kind == .TABLE && toml_get(*value.extra, "a").integer == 1);

    list := toml_get(*value.extra, "list");

    assert(list.kind == .ARRAY && list.items.count == 3 && list.items[1].text == "two" && list.items[2].real == 3.5);
    assert(toml_get(toml_get(*value.extra, "nested"), "ok").boolean);
    free_toml_value(*value);

    headers := "[extra]\nname = 'n'\ndotted.k = 1\n\n[extra.sub]\nv = 'x'\n\n[[extra.rows]]\nid = 1\n[[extra.rows]]\nid = 2\n[extra.rows.deep]\nz = 3\n\n[servers.s]\nhost = 'h'\n";
    other, other_error := from_toml(Config, headers);

    assert(other_error.kind == .NONE, tprint("%: %", other_error.line, other_error.message));
    assert(toml_get(*other.extra, "name").text == "n");
    assert(toml_get(toml_get(*other.extra, "dotted"), "k").integer == 1);
    assert(toml_get(toml_get(*other.extra, "sub"), "v").text == "x");

    rows := toml_get(*other.extra, "rows");

    assert(rows.items.count == 2 && toml_get(rows.items[1], "id").integer == 2);
    assert(toml_get(toml_get(rows.items[1], "deep"), "z").integer == 3);

    // The decoded tree is the one the document parser builds, and the writer takes it.
    text, write_error := write_toml(*other.extra);

    assert(write_error.kind == .NONE, write_error.message);
    defer free(text);

    doc, doc_error := parse_toml(text);

    assert(doc_error.kind == .NONE, text);
    assert(toml_get(toml_get(doc.root, "sub"), "v").text == "x");
    free_toml(*doc);

    // As a field it is written as the table it is, scalars in place.
    field_text, field_error := to_toml(other);

    assert(field_error.kind == .NONE, field_error.message);
    defer free(field_text);

    round, round_error := from_toml(Config, field_text);

    assert(round_error.kind == .NONE, tprint("%: %", round_error.message, field_text));
    assert(toml_get(toml_get(*round.extra, "sub"), "v").text == "x");
    assert(toml_get(*round.extra, "rows").items.count == 2);
    free_toml_value(*round);
    free_toml_value(*other);

    check(source, .NONE);
    check(headers, .NONE);
    check("extra = 1\n", .NONE);
    check("extra = [1, [2, { a = 'b' }]]\n", .NONE);
    check("extra = { a = 1, a = 2 }\n", .DUPLICATE);
    check("extra = { a = 1 }\nextra.b = 2\n", .DUPLICATE);
    check("[extra]\na = 1\n[extra]\n", .DUPLICATE);
    check("[extra.a]\n[extra]\nb = 1\n[extra.a]\n", .DUPLICATE);
    check("extra = { a = [1, }\n", .SYNTAX);
    check("[extra]\na = 'x\n", .SYNTAX);
    check("extra = { a = { b = 1 }, }\n", .NONE);
    check("[[extra.a]]\nb = 1\n[extra.a]\n", .DUPLICATE);
}

arrays :: () {
    source := "[[pair]]\nhost = 'a'\n[[pair]]\nhost = 'b'\nport = 2\n";
    value, error := from_toml(Config, source);

    assert(error.kind == .NONE, error.message);
    assert(value.pair[0].host == "a" && value.pair[1].host == "b" && value.pair[1].port == 2);

    text, write_error := to_toml(value);

    assert(write_error.kind == .NONE, write_error.message);
    defer free(text);

    again, again_error := from_toml(Config, text);

    assert(again_error.kind == .NONE, tprint("%: %", again_error.message, text));
    assert(again.pair[1].host == "b");
    free_toml_value(*again);
    free_toml_value(*value);

    check(source, .NONE);
    check("pair = [{ host = 'a' }, { host = 'b' }]\n", .NONE);
    check("[[pair]]\nhost = 'a'\n", .TYPE, "$[\"pair\"]");
    check("[[pair]]\nhost = 'a'\n[[pair]]\nhost = 'b'\n[[pair]]\nhost = 'c'\n", .TYPE, "$[\"pair\"]");
    check("[[pair]]\nport = 1\n[[pair]]\nhost = 'b'\n", .TYPE, "$[\"pair\"][0][\"host\"]");
    check("[pair]\nhost = 'a'\n", .TYPE);
}

main :: () {
    maps();
    values();
    arrays();
    print("ok\n");
}
