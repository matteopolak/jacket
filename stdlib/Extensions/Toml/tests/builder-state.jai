#import "Basic";
#import "Extensions/Toml";
#load "../../test-support/counting-allocator.jai";

failures := 0;

check :: (condition: bool, reason: string) {
    if !condition {
        print("FAIL: %\n", reason);
        failures += 1;
    }
}

main :: () {
    setters();
    admission();
    attachment();
    depth_and_strings();
    changed_allocator();
    if failures exit(1);

    print("ok\n");
}

setters :: () {
    doc, _ := new_toml_document(.{ max_retained_bytes = 1024 });

    defer free_toml(*doc);
    value, _ := toml_new_value(*doc, .STRING);

    assert(toml_set_text(*doc, value, "a").kind == .NONE);
    before := doc.retained_bytes;
    bad := u8.[0xc0, 0x80];
    invalid := toml_set_text(*doc, value, string.{ bad.count, bad.data });

    check(invalid.kind == .ENCODING && doc.retained_bytes == before && value.text == "a", "rejected UTF-8 setter preserves value and budget");

    for 0..1999 {
        text := ifx it % 2 then "b" else "c";
        error := toml_set_text(*doc, value, text);

        if error.kind != .NONE {
            check(false, "repeated same-size replacement retains a fixed budget");
            break;
        }
    }

    check(doc.retained_bytes == before, "replacement charges actual retained delta");
}

admission :: () {
    probe, _ := new_toml_document();
    count := probe.nodes.allocated;

    for 1..count - 1 toml_new_value(*probe, .STRING);

    budget := probe.retained_bytes + size_of(Toml_Value);

    free_toml(*probe);
    doc, _ := new_toml_document(.{ max_retained_bytes = budget });

    defer free_toml(*doc);
    first: *Toml_Value;

    for 1..count - 1 {
        value, error := toml_new_value(*doc, .STRING);
        assert(error.kind == .NONE);
        if !first first = value;
    }

    before := doc.retained_bytes;
    _, failure := toml_new_value(*doc, .BOOL);

    check(failure.kind == .LIMIT && doc.retained_bytes == before, "failed node reservation does not charge nonexistent storage");
    check(toml_set_text(*doc, first, "x").kind == .NONE, "smaller operation works after rejected reservation");
}

attachment :: () {
    doc, _ := new_toml_document();
    other, _ := new_toml_document();

    defer free_toml(*doc);
    defer free_toml(*other);
    alien, _ := toml_new_value(*other, .BOOL);

    check(toml_insert(*doc, doc.root, "alien", alien).kind == .OUTPUT, "reject cross-document ownership");
    a, _ := toml_new_value(*doc, .TABLE);
    b, _ := toml_new_value(*doc, .TABLE);

    check(toml_insert(*doc, a, "root", doc.root).kind == .OUTPUT, "reject attaching document root");
    assert(toml_insert(*doc, a, "b", b).kind == .NONE);
    check(toml_insert(*doc, b, "a", a).kind == .OUTPUT, "reject cycle through detached subtree");
    first, _ := toml_new_value(*doc, .INTEGER);
    first.integer = 7;
    assert(toml_insert(*doc, doc.root, "first", first).kind == .NONE);
    pointer := toml_get(doc.root, "first");

    for 0..63 {
        value, _ := toml_new_value(*doc, .INTEGER);
        value.integer = it;
        assert(toml_insert(*doc, doc.root, tprint("key-%", it), value).kind == .NONE);
    }

    check(pointer == toml_get(doc.root, "first") && pointer.integer == 7, "lookup pointer survives member/index growth");
}

depth_and_strings :: () {
    doc, _ := new_toml_document(.{ max_depth = 1, max_string_bytes = 3 });

    defer free_toml(*doc);
    table, _ := toml_new_value(*doc, .TABLE);
    scalar, _ := toml_new_value(*doc, .INTEGER);

    assert(toml_insert(*doc, doc.root, "a", table).kind == .NONE);
    check(toml_insert(*doc, table, "b", scalar).kind == .LIMIT, "builder enforces global depth before attachment");
    text, _ := toml_new_value(*doc, .STRING);

    assert(toml_set_text(*doc, text, "abc").kind == .NONE);
    before := doc.retained_bytes;

    check(toml_set_text(*doc, text, "abcd").kind == .LIMIT && text.text == "abc" && doc.retained_bytes == before, "string limit failure preserves state");
    parsed, parsed_error := parse_toml("a='abc'\nb='xyz'\n", .{ max_string_bytes = 3 });

    check(parsed_error.kind == .NONE, "string limit is per decoded key/string/fraction; total is bounded by retained bytes");
    output, output_error := write_toml(parsed.root, .{ max_string_bytes = 3 });

    check(output_error.kind == .NONE, "writer string limit is per key/string/fraction too");
    if output.data free(output);

    free_toml(*parsed);
    tree, _ := new_toml_document(.{ max_depth = 1 });

    defer free_toml(*tree);
    a, _ := toml_new_value(*tree, .TABLE);
    b, _ := toml_new_value(*tree, .INTEGER);

    assert(toml_insert(*tree, a, "b", b).kind == .NONE);
    check(toml_insert(*tree, tree.root, "a", a).kind == .LIMIT, "attaching a subtree checks its complete height");
}

changed_allocator :: () {
    tally: Tally;
    doc: Toml_Document;
    value: *Toml_Value;

    {
        push_allocator(tally_proc, *tally);
        doc, _ = new_toml_document();
        value, _ = toml_new_value(*doc, .STRING);
    }

    assert(toml_set_text(*doc, value, "owned").kind == .NONE);
    assert(toml_insert(*doc, doc.root, "key", value).kind == .NONE);
    free_toml(*doc);
    check(tally.outstanding == 0, "setters and inserts use the captured allocator");
}
