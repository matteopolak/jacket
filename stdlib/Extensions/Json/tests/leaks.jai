// A failed from_json frees everything it allocated, wherever it fails. A tracking allocator counts
// the blocks still outstanding after decoding a valid document cut off at every byte, with every
// byte replaced in turn, and followed by trailing data, in the scalar and the vector paths. It
// also checks free_json on the values that decoded.
//
//     jaic run stdlib/Extensions/Json/tests/leaks.jai

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
    allocations: int;
}

tally_proc :: (mode: Allocator_Mode, size: s64, old_size: s64, old_memory: *void, data: *void) -> *void {
    tally := cast(*Tally) data;
    backing := context.default_allocator;
    result := backing.proc(mode, size, old_size, old_memory, backing.data);

    if mode == .ALLOCATE && result {
        tally.outstanding += 1;
        tally.allocations += 1;
    } else if mode == .RESIZE && result && !old_memory {
        tally.outstanding += 1;
        tally.allocations += 1;
    } else if mode == .FREE && old_memory {
        tally.outstanding -= 1;
    }

    return result;
}

Leaf :: struct {
    label: string;
    values: [..] int;
}

Node :: struct {
    name: string;
    next: *Node;
    kids: [..] Node;
}

Config :: struct {
    title: string;
    nick := "guest"; @json(default) // absent from the document: free_json must leave the literal alone
    tags: [..] string;
    view: [] string;
    fixed: [2] string;
    leaves: [..] Leaf;
    link: *Node;
    by_name: Table(string, Leaf);
    lists: Table(string, [..] string);
    matrix: [..][..] string;
    extra: Json_Value;
    inner: Leaf;
    count: int;
}

Wrapped :: struct @json(transparent) {
    items: [] Leaf;
}

// One document that reaches every kind of allocation in Config.
DOCUMENT :: "{ \"title\": \"a title with an escape \\n and more text\", \"tags\": [\"one\", \"two\", \"three\"], \"view\": [\"v1\", \"v2\"], \"fixed\": [\"f1\", \"f2\"], \"leaves\": [{\"label\": \"l1\", \"values\": [1, 2, 3]}, {\"label\": \"l2\", \"values\": []}], \"link\": {\"name\": \"n1\", \"next\": {\"name\": \"n2\", \"next\": null, \"kids\": []}, \"kids\": [{\"name\": \"k1\", \"next\": null, \"kids\": [{\"name\": \"k2\", \"next\": null, \"kids\": []}]}]}, \"by_name\": {\"a\": {\"label\": \"x\", \"values\": [7]}, \"b\": {\"label\": \"y\", \"values\": [8, 9]}, \"a\": {\"label\": \"again\", \"values\": [10]}}, \"lists\": {\"p\": [\"q\", \"r\"], \"s\": []}, \"matrix\": [[\"m1\", \"m2\"], [], [\"m3\"]], \"extra\": {\"k\": [1, \"two\", {\"three\": null}], \"s\": \"text\"}, \"inner\": {\"label\": \"in\", \"values\": [4, 5]}, \"count\": 12 }";

Expectation :: enum {
    SUCCEEDS;
    FAILS;
    EITHER;
}

// Decodes under the tracking allocator; whatever came back is freed, and nothing may be left.
// Nothing is formatted inside the tracked scope: tprint's temporary storage would be counted.
decode_and_release :: (text: string, label: string, expectation: Expectation) {
    tally: Tally;
    succeeded: bool;
    kind: Json_Error_Kind;
    offset: int;
    default_result: bool;

    {
        push_allocator(tally_proc, *tally);

        value, error := from_json(Config, text);

        succeeded = error.kind == .NONE;
        kind = error.kind;
        offset = error.offset;

        if succeeded {
            free_json(*value);
        } else {
            default_result = value.title.count == 0 && value.tags.count == 0 && value.leaves.count == 0 && value.link == null && value.by_name.count == 0 && value.extra.kind == .NULL && value.nick == "guest";
            free_json(*error);
        }
    }

    if succeeded {
        check(expectation != .FAILS, tprint("% decoded", label));
    } else {
        check(expectation != .SUCCEEDS, tprint("% failed unexpectedly (%, offset %)", label, kind, offset));
        check(default_result, tprint("% returned a default value", label));
    }

    check(tally.outstanding == 0, tprint("% left % blocks allocated", label, tally.outstanding));
}

sweep :: () {
    text := DOCUMENT;

    decode_and_release(text, "the whole document", .SUCCEEDS);

    // Cut off at every offset: the decoder fails at every point of every kind of value.
    for length: 0..text.count - 1 {
        decode_and_release(slice(text, 0, length), tprint("the first % bytes", length), .FAILS);
    }

    // Every byte replaced by one that is never valid there. Some replacements are harmless (a
    // different letter in a string), so the result may succeed; either way nothing may leak.
    for offset: 0..text.count - 1 {
        copy := copy_string(text);

        copy[offset] = #char "@";
        decode_and_release(copy, tprint("byte % replaced", offset), .EITHER);
        free(copy);
    }

    decode_and_release(tprint("% x", text), "trailing data", .FAILS);
    decode_and_release(tprint("%{}", text), "a second value", .FAILS);
}

// A type error deep inside, wrong shapes, and the limits.
shapes :: () {
    decode_and_release("{\"tags\": [\"a\", 1]}", "a number among strings", .FAILS);
    decode_and_release("{\"fixed\": [\"a\", \"b\", \"c\"]}", "too many fixed elements", .FAILS);
    decode_and_release("{\"fixed\": [\"a\"]}", "too few fixed elements", .FAILS);
    decode_and_release("{\"link\": {\"name\": \"a\", \"next\": {\"name\": 5}}}", "a bad pointee", .FAILS);
    decode_and_release("{\"by_name\": {\"a\": {\"label\": \"x\"}, \"b\": 3}}", "a bad table entry", .FAILS);
    decode_and_release("{\"title\": \"t\", \"title\": \"u\"}", "a duplicate field", .FAILS);
    decode_and_release("{\"leaves\": [{\"label\": \"x\", \"values\": [1, 2, \"three\"]}]}", "a bad element of a nested array", .FAILS);
    decode_and_release("{\"extra\": [1, 2, tru]}", "a bad Json_Value", .FAILS);
    decode_and_release("{\"view\": [\"a\", \"b\", 3]}", "a bad view element", .FAILS);

    // Nesting past the limit.
    deep: String_Builder;

    for 1..50 append(*deep, "{\"name\": \"n\", \"kids\": [");
    for 1..50 append(*deep, "]}");

    deep_text := builder_to_string(*deep);

    tally: Tally;
    limit_kind: Json_Error_Kind;

    {
        push_allocator(tally_proc, *tally);

        node, error := from_json(Node, deep_text, Json_Options.{ max_depth = 20 });

        limit_kind = error.kind;
        free_json(*error);
        free_json(*node);
    }

    free(deep_text);
    check(limit_kind == .LIMIT, "the depth limit");
    check(tally.outstanding == 0, tprint("the depth limit left % blocks allocated", tally.outstanding));

    // Transparent wrappers and top-level types other than structs.
    tally = .{};

    bad_wrapped, bad_view, bad_pointer, good_wrapped, good_pointer, good_view: bool;

    {
        push_allocator(tally_proc, *tally);

        _, error := from_json(Wrapped, "[{\"label\": \"a\", \"values\": [1]}, {\"label\": \"b\", \"values\": [x]}]");

        bad_wrapped = error.kind != .NONE;
        free_json(*error);

        _, error2 := from_json([] string, "[\"a\", \"b\", 4]");

        bad_view = error2.kind != .NONE;
        free_json(*error2);

        pointer, error3 := from_json(*Leaf, "{\"label\": \"p\", \"values\": [1, \"2\"]}");

        bad_pointer = error3.kind != .NONE && pointer == null;
        free_json(*error3);

        ok_wrapped, error4 := from_json(Wrapped, "[{\"label\": \"a\", \"values\": [1]}]");

        good_wrapped = error4.kind == .NONE && ok_wrapped.items.count == 1;
        free_json(*ok_wrapped);

        ok_pointer, error5 := from_json(*Leaf, "{\"label\": \"p\", \"values\": [1]}");

        good_pointer = error5.kind == .NONE && ok_pointer != null;
        free_json(*ok_pointer);

        ok_names, error6 := from_json([] string, "[\"a\", \"b\"]");

        good_view = error6.kind == .NONE && ok_names.count == 2;
        free_json(*ok_names);
    }

    check(bad_wrapped, "a bad transparent element");
    check(bad_view, "a bad top-level view");
    check(bad_pointer, "a bad top-level pointer");
    check(good_wrapped, "a good transparent wrapper");
    check(good_pointer, "a good top-level pointer");
    check(good_view, "a good top-level view");
    check(tally.outstanding == 0, tprint("top-level types left % blocks allocated", tally.outstanding));
}

// A field that keeps its declared default is not freed, and a freed value can be freed again.
defaults :: () {
    value, error := from_json(Config, "{\"title\": \"t\", \"tags\": [], \"view\": [], \"fixed\": [\"\", \"\"], \"leaves\": [], \"link\": null, \"by_name\": {}, \"lists\": {}, \"matrix\": [], \"extra\": null, \"inner\": {\"label\": \"\", \"values\": []}, \"count\": 1}");

    check(error.kind == .NONE && value.nick == "guest", "the default survives");
    free_json(*value);
    check(value.nick == "guest" && value.title.count == 0, "freeing keeps the default");
    free_json(*value);
}

// `free_json` is one overload set: a decoded value of any type, a Json_Value and a Json_Error each
// leave nothing allocated and are reset, and freeing twice is harmless.
free_each_kind :: () {
    tally: Tally;
    typed_cleared, untyped_cleared, error_cleared: bool;
    had_error_text: bool;

    {
        push_allocator(tally_proc, *tally);

        typed, typed_error := from_json(Config, DOCUMENT);
        untyped, untyped_error := from_json(Json_Value, "{\"k\": [1, \"two\", {\"three\": null}], \"s\": \"text\"}");
        _, failure := from_json(Config, "{\"title\": 5}");

        had_error_text = typed_error.kind == .NONE && untyped_error.kind == .NONE && failure.message.count > 0 && failure.path.count > 0;

        free_json(*typed);
        free_json(*untyped);
        free_json(*failure);

        typed_cleared = typed.title.count == 0 && typed.tags.count == 0 && typed.link == null;
        untyped_cleared = untyped.kind == .NULL && untyped.members.count == 0;
        error_cleared = failure.kind == .NONE && failure.message.count == 0 && failure.path.count == 0;

        free_json(*typed);
        free_json(*untyped);
        free_json(*failure);
    }

    check(had_error_text, "the values decoded and the error has a message and a path");
    check(typed_cleared, "free_json on a decoded struct resets it");
    check(untyped_cleared, "free_json on a Json_Value resets it");
    check(error_cleared, "free_json on a Json_Error resets it");
    check(tally.outstanding == 0, tprint("free_json left % blocks allocated", tally.outstanding));
}

main :: () {
    free_each_kind();

    for pass: 0..1 {
        json_use_simd = pass == 1;
        json_index_min_size = 0;

        sweep();
        shapes();
        defaults();
    }

    assert(failures == 0);
    print("ok\n");
}
