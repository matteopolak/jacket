// A `void` case of a Variant in JSON: `null` when untagged, the tag alone in the tagged forms,
// and `null` decodes to it in every form. A Variant without one rejects `null`.
//
//     jaic stdlib/Extensions/Json/tests/variant-void.jai -interp

#import "Basic";
#import "Extensions/Json";
#import "Extensions/Variant";

failures := 0;

check :: (condition: bool, what: string, loc := #caller_location) {
    if condition return;
    print("FAIL (line %): %\n", loc.line_number, what);
    failures += 1;
}

#load "../../test-support/counting-allocator.jai";

Circle :: struct {
    radius: int;
}

Rect :: struct {
    width: int;
    height: int;
}

// Scalar cases and a void case: untagged by default, like the same Variant without the void case.
Maybe_Number :: Variant(void, s64, string);

// Struct cases and a void case: externally tagged by default.
Shape :: Variant(void, Circle, Rect);

// The void case need not come first; then the zero value is another case.
Late :: Variant(struct {
    circle: Circle;
    nothing: void;
});

Holder :: struct {
    external: Shape;
    internal: Shape; @json(tag=kind)
    adjacent: Shape; @json(tag=t,content=c)
    loose: Shape; @json(untagged)
}

encode :: (value: $T) -> string {
    return to_json(value,, temp);
}

decode :: ($T: Type, text: string) -> T {
    value, error := from_json(T, text);

    check(error.kind == .NONE, tprint("decoding % failed: %", text, error.message));

    return value;
}

failure :: ($T: Type, text: string) -> Json_Error {
    _, error := from_json(T, text);

    check(error.kind != .NONE, tprint("decoding % should have failed", text));

    return error;
}

untagged :: () {
    none := variant(Maybe_Number);

    check(encode(none) == "null", "the void case of an untagged variant is null");
    check(encode(variant(Maybe_Number, cast(s64) 5)) == "5", "a number is itself");
    check(encode(variant(Maybe_Number, "x")) == "\"x\"", "and a string");

    check(decode(Maybe_Number, "null") == none, "null decodes to the void case");
    check(decode(Maybe_Number, "5") == variant(Maybe_Number, cast(s64) 5), "a number decodes");
    check(decode(Maybe_Number, "\"x\"") == variant(Maybe_Number, "x"), "a string decodes");

    zero: Maybe_Number;

    check(encode(zero) == "null" && decode(Maybe_Number, encode(zero)).which == .NONE, "the zero value round trips");

    check(failure(Maybe_Number, "true").kind == .TYPE, "nothing else matches");

    again := decode(Maybe_Number, "\"x\"");

    free_json(*again);
    check(again.which == .NONE, "free_json resets to the void case");
}

tagged :: () {
    none := variant(Shape);

    check(encode(none) == "\"none\"", "externally tagged: the tag alone");
    check(encode(variant(Shape, Circle.{ 2 })) == "{\"circle\":{\"radius\":2}}", "the other cases as before");

    check(decode(Shape, "\"none\"") == none, "the tag decodes");
    check(decode(Shape, " \"none\" ") == none, "with whitespace");
    check(decode(Shape, "null") == none, "null decodes too");
    check(decode(Shape, "{\"none\":null}") == none, "and the tag as a key with null");
    check(decode(Shape, "{\"circle\":{\"radius\":2}}") == variant(Shape, Circle.{ 2 }), "the other cases decode");

    check(failure(Shape, "\"circle\"").kind == .UNKNOWN_VARIANT, "the tag of a case with a payload is not enough");
    check(failure(Shape, "\"square\"").kind == .UNKNOWN_VARIANT, "an unknown tag");
    check(failure(Shape, "{\"none\":1}").kind == .SYNTAX, "the key form needs null");
    check(failure(Shape, "5").kind == .TYPE, "a number");

    holder: Holder;

    holder.external = none;
    holder.internal = none;
    holder.adjacent = none;
    holder.loose = none;

    text := encode(holder);

    check(text == "{\"external\":\"none\",\"internal\":{\"kind\":\"none\"},\"adjacent\":{\"t\":\"none\"},\"loose\":null}", "each form");

    back := decode(Holder, text);

    check(back.external == none && back.internal == none && back.adjacent == none && back.loose == none, "and decodes");

    holder.external = variant(Shape, Rect.{ 1, 2 });
    holder.internal = variant(Shape, Rect.{ 1, 2 });
    holder.adjacent = variant(Shape, Rect.{ 1, 2 });
    holder.loose = variant(Shape, Rect.{ 1, 2 });

    other := decode(Holder, encode(holder));

    check(other.external == holder.external && other.internal == holder.internal && other.adjacent == holder.adjacent && other.loose == holder.loose, "the other cases round trip in each form");

    // Null for any form, and other forms of the tag.
    nulls := decode(Holder, "{\"external\":null,\"internal\":null,\"adjacent\":null,\"loose\":null}");

    check(nulls.external == none && nulls.internal == none && nulls.adjacent == none && nulls.loose == none, "null is the void case in every form");

    check(decode(Holder, "{\"external\":\"none\",\"internal\":{\"extra\":[1],\"kind\":\"none\"},\"adjacent\":{\"c\":null,\"t\":\"none\"},\"loose\":null}").internal == none, "a tag in any position, with members to skip");
    check(decode(Holder, "{\"external\":\"none\",\"internal\":{\"kind\":\"none\"},\"adjacent\":{\"c\":null,\"t\":\"none\"},\"loose\":null}").adjacent == none, "adjacent: null content is allowed");

    check(failure(Holder, "{\"external\":\"none\",\"internal\":{\"kind\":\"none\"},\"adjacent\":{\"t\":\"none\",\"c\":1},\"loose\":null}").kind == .SYNTAX, "adjacent: content must be null");
    check(failure(Holder, "{\"external\":\"none\",\"internal\":{\"kind\":\"none\"},\"adjacent\":{\"t\":\"circle\"},\"loose\":null}").kind == .MISSING_FIELD, "adjacent: another case needs its content");
    check(failure(Holder, "{\"external\":\"none\",\"internal\":{\"kind\":\"circle\"},\"adjacent\":{\"t\":\"none\"},\"loose\":null}").kind == .MISSING_FIELD, "internal: another case needs its fields");

    strict, error := from_json(Holder, "{\"external\":\"none\",\"internal\":{\"kind\":\"none\",\"extra\":1},\"adjacent\":{\"t\":\"none\"},\"loose\":null}", .{ deny_unknown_fields = true });

    check(error.kind == .UNKNOWN_FIELD, "internal: unknown members are refused when asked");
    free_json(*error);
    free_json(*strict);
}

late :: () {
    zero: Late;

    check(zero.which == .CIRCLE, "the zero value is the first case");
    check(encode(zero) == "{\"circle\":{\"radius\":0}}", "and encodes as it");

    nothing := variant(Late);

    check(encode(nothing) == "\"nothing\"", "a void case under its own name is its tag");
    check(decode(Late, "\"nothing\"") == nothing, "and decodes");
    check(decode(Late, "null") == nothing, "null decodes to it");
    check(decode(Late, "{\"circle\":{\"radius\":4}}") == variant(Late, Circle.{ 4 }), "the other case");
}

leaks :: () {
    tally: Tally;

    {
        push_allocator(tally_proc, *tally);

        a := decode(Holder, "{\"external\":{\"circle\":{\"radius\":1}},\"internal\":{\"kind\":\"none\"},\"adjacent\":null,\"loose\":null}");

        free_json(*a);

        // A failure after a void case was read, and one before it.
        _, error := from_json(Holder, "{\"external\":\"none\",\"internal\":null,\"adjacent\":{\"t\":\"none\"},\"loose\":5}");

        check(error.kind != .NONE, "a bad untagged value fails");
        free_json(*error);

        _, error2 := from_json(Holder, "{\"external\":\"circle\"}");

        check(error2.kind == .UNKNOWN_VARIANT, "a bare tag of a case with a payload");
        free_json(*error2);
    }

    check(tally.outstanding == 0, tprint("left % blocks allocated", tally.outstanding));
}

main :: () {
    for pass: 0..1 {
        json_use_simd = pass == 1;
        json_index_min_size = 0;

        untagged();
        tagged();
        late();
        leaks();
    }

    assert(failures == 0);
    print("ok\n");
}
