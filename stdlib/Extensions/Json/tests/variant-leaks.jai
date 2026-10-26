// A failed from_json of a document with variants frees everything it allocated, wherever it fails,
// in every tagging form and in an untagged variant that tries several cases. A tracking allocator
// counts the blocks outstanding after decoding a valid document cut off at every byte and with
// every byte replaced in turn, in the scalar and the vector paths. free_json then releases the
// live cases of what decoded.
//
//     jaic stdlib/Extensions/Json/tests/variant-leaks.jai -interp

#import "Basic";
#import "String";
#import "Hash_Table";
#import "Extensions/Json";
#import "Extensions/Variant";

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

Named :: struct {
    name: string;
    tags: [..] string;
}

Listed :: struct {
    items: [..] string;
    extra: string;
}

Empty :: struct {}

// Cases that own memory at every depth.
Payload :: Variant(Empty, Named, Listed, string, [..] string);

Scalars :: Variant(s64, string, bool);

Plain :: struct {
    label: string;
    external: Payload;
    internal: Variant(Named, Listed); @json(tag=kind)
    adjacent: Payload; @json(tag=t,content=c)
    loose: Variant(Named, Listed); @json(untagged)
    scalar: Scalars;
    many: [..] Payload;
    maybe: Json_Maybe(Payload);
    by_name: Table(string, Payload);
    last: string;
}

DOCUMENT :: "{ \"label\": \"a label with an escape \\n inside\", \"external\": {\"Named\": {\"name\": \"external name\", \"tags\": [\"t1\", \"t2\"]}}, \"internal\": {\"kind\": \"Listed\", \"items\": [\"i1\", \"i2\"], \"extra\": \"more\"}, \"adjacent\": {\"c\": {\"items\": [\"a1\", \"a2\"], \"extra\": \"x\"}, \"t\": \"Listed\"}, \"loose\": {\"items\": [\"l1\"], \"extra\": \"loose text\"}, \"scalar\": \"a string scalar\", \"many\": [{\"string\": \"s1\"}, {\"Empty\": {}}, {\"[..] string\": [\"m1\", \"m2\"]}, {\"Named\": {\"name\": \"n\", \"tags\": []}}], \"maybe\": {\"string\": \"maybe text\"}, \"by_name\": {\"k\": {\"Listed\": {\"items\": [\"b1\"], \"extra\": \"e\"}}, \"k2\": {\"string\": \"again\"}, \"k\": {\"string\": \"replaced\"}}, \"last\": \"last text\" }";

Expectation :: enum {
    SUCCEEDS;
    FAILS;
    EITHER;
}

decode_and_release :: (text: string, label: string, expectation: Expectation) {
    tally: Tally;
    succeeded: bool;
    kind: Json_Error_Kind;
    offset: int;
    default_result: bool;

    {
        push_allocator(tally_proc, *tally);

        value, error := from_json(Plain, text);

        succeeded = error.kind == .NONE;
        kind = error.kind;
        offset = error.offset;

        if succeeded {
            free_json(*value);
        } else {
            default_result = value.label.count == 0 && value.many.count == 0 && value.external.index == 0 && value.by_name.count == 0;
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

    for length: 0..text.count - 1 {
        decode_and_release(slice(text, 0, length), tprint("the first % bytes", length), .FAILS);
    }

    for offset: 0..text.count - 1 {
        copy := copy_string(text);

        copy[offset] = #char "@";
        decode_and_release(copy, tprint("byte % replaced", offset), .EITHER);
        free(copy);
    }

    decode_and_release(tprint("% x", text), "trailing data", .FAILS);
}

shapes :: () {
    // Wrong tags and shapes after the payload has been read.
    decode_and_release("{\"external\": {\"Named\": {\"name\": \"n\"}, \"Empty\": {}}}", "two keys", .FAILS);
    decode_and_release("{\"external\": {\"Listed\": {\"items\": [\"a\"], \"extra\": 1}}}", "a bad field after strings", .FAILS);
    decode_and_release("{\"internal\": {\"items\": [\"a\", \"b\"], \"kind\": \"Named\", \"name\": \"n\"}}", "a missing field at the end of a case", .FAILS);
    decode_and_release("{\"internal\": {\"items\": [\"a\"], \"extra\": \"x\", \"kind\": \"Unknown\"}}", "an unknown internal tag", .FAILS);
    decode_and_release("{\"adjacent\": {\"t\": \"Named\", \"c\": {\"name\": \"n\", \"tags\": [\"a\"]}, \"c\": {\"name\": \"m\"}}}", "a repeated content", .FAILS);
    decode_and_release("{\"adjacent\": {\"c\": {\"name\": \"n\", \"tags\": [\"a\"]}, \"t\": \"Nowhere\"}}", "an unknown adjacent tag after the content", .FAILS);
    decode_and_release("{\"adjacent\": {\"t\": \"Named\", \"c\": {\"name\": \"n\"}, \"other\": [1,", "an unfinished adjacent object", .FAILS);
    decode_and_release("{\"loose\": {\"name\": 5}}", "no untagged case fits", .FAILS);
    decode_and_release("{\"loose\": {\"items\": [\"a\", 2]}}", "an untagged case that fails after allocating", .FAILS);
    decode_and_release("{\"scalar\": []}", "no scalar fits", .FAILS);
    decode_and_release("{\"many\": [{\"string\": \"ok\"}, {\"string\": 3}]}", "a bad element after a good one", .FAILS);
    decode_and_release("{\"maybe\": {\"Listed\": {\"items\": [\"a\"], \"extra\": []}}}", "a bad Json_Maybe payload", .FAILS);
    decode_and_release("{\"by_name\": {\"a\": {\"[..] string\": [\"x\", 1]}}}", "a bad table payload", .FAILS);
}

// free_json on what decoded: the live case of each variant is released, the variant zeroed, and a
// second call does nothing.
releasing :: () {
    tally: Tally;
    cleared: bool;

    {
        push_allocator(tally_proc, *tally);

        value := from_json(Plain, DOCUMENT);

        free_json(*value);
        cleared = value.external.index == 0 && value.internal.index == 0 && value.adjacent.index == 0 && value.loose.index == 0 && value.scalar.index == 0 && value.many.count == 0 && value.maybe.state == .MISSING;

        free_json(*value);
    }

    check(cleared, "free_json zeroes every variant");
    check(tally.outstanding == 0, tprint("free_json left % blocks allocated", tally.outstanding));
}

main :: () {
    releasing();

    for pass: 0..1 {
        json_use_simd = pass == 1;
        json_index_min_size = 0;

        sweep();
        shapes();
    }

    assert(failures == 0);
    print("ok\n");
}
