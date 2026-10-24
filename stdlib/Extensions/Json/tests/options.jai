// Json_Options (deny_unknown_fields, missing_fields_default, duplicate_keys, borrow), Json_Maybe
// and json_path_text.
//
//     jaic stdlib/Extensions/Json/tests/options.jai -interp

#import "Basic";
#import "String";
#import "Hash_Table";
#import "Extensions/Json";

failures := 0;

check :: (condition: bool, what: string, loc := #caller_location) {
    if condition return;
    print("FAIL (line %): %\n", loc.line_number, what);
    failures += 1;
}

Tally :: struct {
    outstanding: int;
}

tally_proc :: (mode: Allocator_Mode, size: s64, old_size: s64, old_memory: *void, data: *void) -> *void {
    tally := cast(*Tally) data;
    backing := context.default_allocator;
    result := backing.proc(mode, size, old_size, old_memory, backing.data);

    if mode == .ALLOCATE && result {
        tally.outstanding += 1;
    } else if mode == .RESIZE && result && !old_memory {
        tally.outstanding += 1;
    } else if mode == .FREE && old_memory {
        tally.outstanding -= 1;
    }

    return result;
}

Plain :: struct {
    a: int;
}

Strict :: struct @json(deny_unknown_fields) {
    a: int;
}

Defaults :: struct {
    n := 5;
    s: string;
    flag := true;
}

Owner :: struct @json(default) {
    name: string;
    tags: [..] string;
    inner: Plain;
    extra: Json_Value;
    table: Table(string, string);
    link: *Plain;
}

Settings :: struct {
    theme: string;
    size: Json_Maybe(int);
    note: Json_Maybe(string);
    flag: Json_Maybe(bool);
    items: Json_Maybe([..] string);
}

Holder :: struct {
    list: [..] Json_Maybe(int);
    name: string;
}

within :: (inner: string, text: string) -> bool {
    return inner.data >= text.data && inner.data + inner.count <= text.data + text.count;
}

unknown_fields :: () {
    text := "{\"a\":1,\"b\":[1,2]}";

    plain, error := from_json(Plain, text);

    check(error.kind == .NONE && plain.a == 1, "unknown keys are ignored by default");

    _, denied := from_json(Plain, text, .{ deny_unknown_fields = true });

    check(denied.kind == .UNKNOWN_FIELD, "deny_unknown_fields turns them into errors");
    free_json(*denied);

    _, own := from_json(Strict, text);

    check(own.kind == .UNKNOWN_FIELD, "a struct note denies without the option");
    free_json(*own);

    ok, none := from_json(Plain, "{\"a\":2}", .{ deny_unknown_fields = true });

    check(none.kind == .NONE && ok.a == 2, "no unknown keys, no error");
}

missing_fields :: () {
    _, missing := from_json(Defaults, "{\"s\":\"x\"}");

    check(missing.kind == .MISSING_FIELD, "a missing field is an error by default");
    free_json(*missing);

    kept, fine := from_json(Defaults, "{\"n\":9}", .{ missing_fields_default = true });

    check(fine.kind == .NONE && kept.n == 9 && kept.s == "" && kept.flag, "missing fields keep their initializers");
}

duplicate_keys :: () {
    text := "{\"name\":\"first\",\"tags\":[\"a\",\"b\"],\"inner\":{\"a\":1},\"extra\":{\"k\":[1]},\"table\":{\"x\":\"1\",\"x\":\"2\"},\"link\":{\"a\":3},\"name\":\"second\",\"tags\":[\"c\"],\"inner\":{\"a\":2},\"extra\":[true],\"link\":{\"a\":4}}";

    _, error := from_json(Owner, text);

    check(error.kind == .DUPLICATE_FIELD, "a repeated key is an error by default");
    free_json(*error);

    tally: Tally;
    name: string;
    tags: int;
    last_a: int;
    link_a: int;
    kind: Json_Error_Kind;
    entries: int;
    extra_kind: Json_Kind;

    {
        push_allocator(tally_proc, *tally);

        value, failure := from_json(Owner, text, .{ duplicate_keys = .LAST });

        kind = failure.kind;

        if kind == .NONE {
            name = value.name;
            tags = value.tags.count;
            last_a = value.inner.a;
            link_a = value.link.a;
            entries = value.table.count;
            extra_kind = value.extra.kind;
            check(value.name == "second", "the last name wins");
            check(value.tags.count == 1 && value.tags[0] == "c", "the last array wins");
            check(value.table.count == 1 && table_find_pointer(*value.table, "x").* == "2", "the last table entry wins");
            free_json(*value);
        }

        free_json(*failure);
    }

    check(kind == .NONE, "duplicate_keys = .LAST decodes");
    check(tags == 1 && last_a == 2 && link_a == 4 && entries == 1 && extra_kind == .ARRAY, "the last values win");
    check(tally.outstanding == 0, tprint("replaced values are released (% blocks left)", tally.outstanding));

    // A failure after a repeat still releases everything.
    tally = .{};

    {
        push_allocator(tally_proc, *tally);

        _, failure := from_json(Owner, "{\"name\":\"a\",\"tags\":[\"x\"],\"name\":\"b\",\"tags\":[\"y\"],\"inner\":7}", .{ duplicate_keys = .LAST });

        kind = failure.kind;
        free_json(*failure);
    }

    check(kind == .TYPE && tally.outstanding == 0, tprint("a failure after a repeat leaves % blocks", tally.outstanding));

    // A repeated Json_Maybe.
    tally = .{};

    {
        push_allocator(tally_proc, *tally);

        value, failure := from_json(Settings, "{\"theme\":\"a\",\"note\":\"first\",\"note\":null,\"items\":[\"p\"],\"items\":[\"q\",\"r\"],\"theme\":\"b\"}", .{ duplicate_keys = .LAST });

        check(failure.kind == .NONE && value.note.state == .NULL && value.items.state == .SET && value.items.value.count == 2 && value.theme == "b", "repeated Json_Maybe keeps the last");
        free_json(*value);
        free_json(*failure);
    }

    check(tally.outstanding == 0, tprint("repeated Json_Maybe left % blocks", tally.outstanding));

    // A Hash_Table replaces by default.
    table, table_error := from_json(Table(string, int), "{\"k\":1,\"k\":2}");

    check(table_error.kind == .NONE && table.count == 1 && table_find_pointer(*table, "k").* == 2, "a Hash_Table keeps the last value");
    free_json(*table);
}

borrowing :: () {
    text := "{\"name\":\"plain\",\"tags\":[\"one\",\"tw\\u006f\"],\"extra\":{\"key\":\"v\",\"n\":12.50,\"es\\u0063\":\"a\\nb\"},\"table\":{\"tk\":\"tv\"}}";

    {
        value, error := from_json(Owner, text);

        check(error.kind == .NONE && !within(value.name, text), "strings are copied by default");

        copy_free := value.name;

        check(copy_free == "plain", "copied string");
        free_json(*value);
    }

    value, error := from_json(Owner, text, .{ borrow = true },, temp);

    check(error.kind == .NONE, tprint("borrowed decode: %", error_text(error)));
    check(value.name == "plain" && within(value.name, text), "a string without escapes points into the input");
    check(value.tags.count == 2 && value.tags[0] == "one" && within(value.tags[0], text), "array strings are borrowed");
    check(value.tags[1] == "two" && !within(value.tags[1], text), "a string with escapes is allocated");

    extra := value.extra;

    check(extra.kind == .OBJECT && extra.members.count == 3, "Json_Value decoded");
    check(extra.members[0].key == "key" && within(extra.members[0].key, text), "member keys are borrowed");
    check(extra.members[0].value.text == "v" && within(extra.members[0].value.text, text), "Json_Value strings are borrowed");
    check(extra.members[1].value.text == "12.50" && within(extra.members[1].value.text, text), "Json_Value numbers are borrowed");
    check(extra.members[2].key == "esc" && !within(extra.members[2].key, text), "an escaped key is copied");
    check(extra.members[2].value.text == "a\nb" && !within(extra.members[2].value.text, text), "an escaped Json_Value string is allocated");

    entry := table_find_pointer(*value.table, "tk");

    check(entry != null && entry.* == "tv" && within(entry.*, text), "table values are borrowed");

    for _, key: value.table check(within(key, text), "table keys are borrowed");

    // A failed borrowed decode must not free the input.
    _, failure := from_json(Owner, "{\"name\":\"zzz\",\"tags\":[\"q\",5]}", .{ borrow = true },, temp);

    check(failure.kind == .TYPE, "borrowed decode failure");

    plain, plain_error := from_json(Plain, "{\"a\":1}", .{ borrow = true });

    check(plain_error.kind == .NONE && plain.a == 1, "borrow with plain structs");
}

maybe :: () {
    value, error := from_json(Settings, "{\"theme\":\"t\",\"size\":null,\"note\":\"n\",\"items\":[\"a\",\"b\"]}");

    check(error.kind == .NONE, "maybe decodes");
    check(value.size.state == .NULL, "null is .NULL");
    check(value.note.state == .SET && value.note.value == "n", "a value is .SET");
    check(value.flag.state == .MISSING, "a missing key is .MISSING");
    check(value.items.state == .SET && value.items.value.count == 2, "arrays inside a Maybe");

    text := to_json(value);

    check(text == "{\"theme\":\"t\",\"size\":null,\"note\":\"n\",\"items\":[\"a\",\"b\"]}", tprint("encoding omits missing keys: %", text));
    free(text);

    pretty := to_json(value, pretty = true);

    check(find_index_from_left(pretty, "flag") < 0 && find_index_from_left(pretty, "\"size\": null") >= 0, "pretty output omits missing keys too");
    free(pretty);

    free_json(*value);
    check(value.note.state == .MISSING && value.note.value.count == 0, "free_json resets a Maybe");

    empty, empty_error := from_json(Settings, "{\"theme\":\"x\"}");

    check(empty_error.kind == .NONE && empty.size.state == .MISSING && empty.note.state == .MISSING, "no Maybe key is not an error");
    free_json(*empty);

    set, set_error := from_json(Settings, "{\"theme\":\"x\",\"size\":0,\"flag\":false}");

    check(set_error.kind == .NONE && set.size.state == .SET && set.size.value == 0 && set.flag.state == .SET && !set.flag.value, "falsy values are .SET");
    free_json(*set);

    _, wrong := from_json(Settings, "{\"theme\":\"x\",\"size\":\"big\"}");

    check(wrong.kind == .TYPE && wrong.path == "size", "a wrong type is an error");
    free_json(*wrong);

    // In arrays and at the top level.
    holder, holder_error := from_json(Holder, "{\"list\":[1,null,3],\"name\":\"h\"}");

    check(holder_error.kind == .NONE && holder.list.count == 3, "array of Maybe");
    check(holder.list[0].state == .SET && holder.list[0].value == 1 && holder.list[1].state == .NULL && holder.list[2].value == 3, "array element states");

    holder_text := to_json(holder);

    check(holder_text == "{\"list\":[1,null,3],\"name\":\"h\"}", tprint("array round trip: %", holder_text));
    free(holder_text);

    holder.list[0].state = .MISSING;
    holder_text = to_json(holder);
    check(holder_text == "{\"list\":[null,null,3],\"name\":\"h\"}", "a missing element encodes as null");
    free(holder_text);
    free_json(*holder);

    top, top_error := from_json(Json_Maybe(string), "null");

    check(top_error.kind == .NONE && top.state == .NULL, "top-level null");
    check(to_json(top) == "null", "top-level null encodes");

    top, top_error = from_json(Json_Maybe(string), "\"x\"");
    check(top_error.kind == .NONE && top.state == .SET && top.value == "x", "top-level value");
    text = to_json(top);
    check(text == "\"x\"", "top-level value encodes");
    free(text);
    free_json(*top);

    nested, nested_error := from_json([] Json_Maybe(Plain), "[{\"a\":1},null]");

    check(nested_error.kind == .NONE && nested.count == 2 && nested[0].state == .SET && nested[0].value.a == 1 && nested[1].state == .NULL, "views of Maybe structs");
    free_json(*nested);

    // A failure inside a Maybe releases what was read.
    tally: Tally;
    kind: Json_Error_Kind;

    {
        push_allocator(tally_proc, *tally);

        _, failure := from_json(Settings, "{\"theme\":\"a\",\"note\":\"n\",\"items\":[\"x\",\"y\",3]}");

        kind = failure.kind;
        free_json(*failure);
    }

    check(kind == .TYPE && tally.outstanding == 0, tprint("failure in a Maybe left % blocks", tally.outstanding));

    tally = .{};

    {
        push_allocator(tally_proc, *tally);

        ok, ok_error := from_json(Settings, "{\"theme\":\"a\",\"note\":\"n\",\"items\":[\"x\"]}");

        kind = ok_error.kind;
        free_json(*ok);
    }

    check(kind == .NONE && tally.outstanding == 0, tprint("free_json of a Maybe left % blocks", tally.outstanding));
}

paths :: () {
    check(json_path_text(.{}) == "$", "empty path");

    _, deep := from_json(Owner, "{\"inner\":{\"a\":\"x\"}}");

    check(json_path_text(deep) == "$.inner.a", tprint("keys: %", json_path_text(deep)));
    free_json(*deep);

    _, indexed := from_json([][] int, "[[1],[2,\"x\"]]");

    check(json_path_text(indexed) == "$[1][1]", tprint("indexes: %", json_path_text(indexed)));
    free_json(*indexed);

    _, mixed := from_json(Holder, "{\"list\":[1,\"x\"]}");

    check(json_path_text(mixed) == "$.list[1]", tprint("key and index: %", json_path_text(mixed)));
    free_json(*mixed);

    _, odd := from_json(Table(string, Plain), "{\"we ird\":{\"a\":true},\"x\":1}");

    check(odd.path == "\"we ird\".a" && json_path_text(odd) == "$.\"we ird\".a", tprint("quoted key: %", json_path_text(odd)));
    free_json(*odd);

    _, dotted := from_json(Table(string, Plain), "{\"a.b[0]\":{\"a\":true}}");

    check(json_path_text(dotted) == "$.\"a.b[0]\".a", tprint("dots and brackets: %", json_path_text(dotted)));
    free_json(*dotted);

    _, dashed := from_json(Table(string, Plain), "{\"user-id\":{\"a\":true},\"1st\":{\"a\":1}}");

    check(json_path_text(dashed) == "$.\"user-id\".a", tprint("hyphen: %", json_path_text(dashed)));
    check(dashed.path == "user-id.a", "plain keys stay bare in path");
    free_json(*dashed);

    error := Json_Error.{ path = "\"q\\\"x\"[3].k_1" };

    check(json_path_text(error) == "$.\"q\\\"x\"[3].k_1", tprint("escaped quote: %", json_path_text(error)));
}

main :: () {
    for pass: 0..1 {
        json_use_simd = pass == 1;
        json_index_min_size = 0;

        unknown_fields();
        missing_fields();
        duplicate_keys();
        borrowing();
        maybe();
        paths();
    }

    assert(failures == 0);
    print("ok\n");
}
