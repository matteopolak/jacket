// Variants in JSON: the four forms (externally tagged by default, internally and adjacently tagged,
// untagged), the notes that choose them, containers and nested variants, and the errors.
//
//     jaic stdlib/Extensions/Json/tests/variant.jai -interp

#import "Basic";
#import "Hash_Table";
#import "String";
#import "Extensions/Json";
#import "Extensions/Variant";

failures := 0;

check :: (condition: bool, what: string, loc := #caller_location) {
    if condition return;
    print("FAIL (line %): %\n", loc.line_number, what);
    failures += 1;
}

Circle :: struct {
    radius: int;
}

Rect :: struct {
    width: int;
    height: int;
}

Empty :: struct {}

Shape :: Variant(Circle, Rect, Empty);

Color :: enum {
    RED;
    GREEN;
}

Scalar :: Variant(s64, string, bool, float64, Color);

Number_Or_Text :: Variant(s64, string);

// Not every case is a scalar: externally tagged, with the types' names as tags.
Int_Or_Circle :: Variant(s64, Circle);

encode :: (value: $T) -> string {
    return to_json(value,, temp);
}

// Decodes, requires success, and returns the value.
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

contains :: (haystack: string, needle: string) -> bool {
    if needle.count == 0 return true;

    for 0..haystack.count - needle.count {
        if slice(haystack, it, needle.count) == needle return true;
    }

    return false;
}

top_level :: () {
    circle := variant(Shape, Circle.{ 3 });

    check(encode(circle) == "{\"circle\":{\"radius\":3}}", "externally tagged by default");
    check(encode(variant(Shape, Rect.{ 1, 2 })) == "{\"rect\":{\"width\":1,\"height\":2}}", "a second case");
    check(encode(variant(Shape, Empty.{})) == "{\"empty\":{}}", "a case without a payload");

    check(decode(Shape, "{\"circle\":{\"radius\":3}}") == circle, "decodes what it encodes");
    check(decode(Shape, "{\"rect\":{\"width\":1,\"height\":2}}") == variant(Shape, Rect.{ 1, 2 }), "decodes the second case");
    check(decode(Shape, " { \"empty\" : { } } ") == variant(Shape, Empty.{}), "whitespace everywhere");

    // Members in a different order, and fields the type does not have.
    check(decode(Shape, "{\"rect\":{\"height\":2,\"extra\":null,\"width\":1}}") == variant(Shape, Rect.{ 1, 2 }), "a payload in another order");

    pretty := to_json(circle, pretty = true,, temp);

    check(pretty == "{\n  \"circle\": {\n    \"radius\": 3\n  }\n}", "pretty output");
    check(decode(Shape, pretty) == circle, "pretty output decodes");

    // The zero value holds no case.
    zero: Shape;

    check(encode(zero) == "null", "the zero value");
    check(decode(Shape, "null").which == .NONE, "null decodes to no case");

    // Cases of the same type are told apart by their tags, never left untagged.
    Contact :: Variant(struct {
        email: string;
        phone: string;
    });

    phone: Contact;
    phone.which = .PHONE;
    phone.phone = "555";

    check(encode(phone) == "{\"phone\":\"555\"}", "two string cases are tagged");
    check(decode(Contact, "{\"phone\":\"555\"}").which == .PHONE, "and decode by tag");
}

scalars :: () {
    // Every case a scalar: the payload by itself.
    check(encode(variant(Scalar, 5)) == "5", "an int");
    check(encode(variant(Scalar, "hi")) == "\"hi\"", "a string");
    check(encode(variant(Scalar, true)) == "true", "a bool");
    check(encode(variant(Scalar, cast(float64) 2.5)) == "2.5", "a float");
    check(encode(variant(Scalar, Color.GREEN)) == "\"GREEN\"", "an enum by name");

    check(decode(Scalar, "5") == variant(Scalar, 5), "decodes an int");
    check(decode(Scalar, "\"hi\"") == variant(Scalar, "hi"), "decodes a string");
    check(decode(Scalar, "true") == variant(Scalar, true), "decodes a bool");
    check(decode(Scalar, "2.5") == variant(Scalar, cast(float64) 2.5), "an int fails, so the float case matches");

    // A string is tried as a string before it is tried as an enum name.
    check(decode(Scalar, "\"GREEN\"") == variant(Scalar, "GREEN"), "the first case that matches wins");

    check(decode(Number_Or_Text, "7") == variant(Number_Or_Text, 7), "Number_Or_Text number");
    check(decode(Number_Or_Text, "\"7\"") == variant(Number_Or_Text, "7"), "Number_Or_Text text");

    // Not every case a scalar: tagged.
    check(encode(variant(Int_Or_Circle, 4)) == "{\"s64\":4}", "a scalar among structs is tagged by its type's name");
    check(encode(variant(Int_Or_Circle, Circle.{ 2 })) == "{\"circle\":{\"radius\":2}}", "and so is the struct");
    check(decode(Int_Or_Circle, "{\"s64\":4}") == variant(Int_Or_Circle, 4), "decodes it");
}

Holder :: struct {
    internal: Shape; @json(tag=kind)
    adjacent: Shape; @json(tag=t,content=c)
    untagged: Shape; @json(untagged)
    named: Shape; @json(tags=circle,rect,none)
    plain: Shape;
}

forms :: () {
    holder: Holder;

    set(*holder.internal, Rect.{ 1, 2 });
    set(*holder.adjacent, Circle.{ 3 });
    set(*holder.untagged, Rect.{ 4, 5 });
    set(*holder.named, Empty.{});
    set(*holder.plain, Circle.{ 6 });

    text := encode(holder);

    check(text == "{\"internal\":{\"kind\":\"rect\",\"width\":1,\"height\":2},\"adjacent\":{\"t\":\"circle\",\"c\":{\"radius\":3}},\"untagged\":{\"width\":4,\"height\":5},\"named\":{\"none\":{}},\"plain\":{\"circle\":{\"radius\":6}}}", "all five forms in one struct");

    back := decode(Holder, text);

    check(back.internal == holder.internal, "internally tagged round trip");
    check(back.adjacent == holder.adjacent, "adjacently tagged round trip");
    check(back.untagged == holder.untagged, "untagged round trip");
    check(back.named == holder.named, "renamed tags round trip");
    check(back.plain == holder.plain, "externally tagged round trip");

    free_json(*back);
}

orders :: () {
    // The tag can be anywhere in an internally tagged object.
    check(decode(Holder_Internal, "{\"shape\":{\"kind\":\"rect\",\"width\":1,\"height\":2}}").shape == variant(Shape, Rect.{ 1, 2 }), "the tag first");
    check(decode(Holder_Internal, "{\"shape\":{\"width\":1,\"height\":2,\"kind\":\"rect\"}}").shape == variant(Shape, Rect.{ 1, 2 }), "the tag last");
    check(decode(Holder_Internal, "{\"shape\":{\"width\":1,\"kind\":\"rect\",\"height\":2}}").shape == variant(Shape, Rect.{ 1, 2 }), "the tag between fields");
    check(decode(Holder_Internal, "{\"shape\":{\"kind\":\"empty\"}}").shape == variant(Shape, Empty.{}), "a case without fields");
    check(decode(Holder_Internal, "{\"shape\":{\"radius\":4,\"kind\":\"circle\",\"unknown\":[1,{\"a\":2}]}}").shape == variant(Shape, Circle.{ 4 }), "unknown fields are skipped");

    // The content can come before the tag in an adjacently tagged object.
    check(decode(Holder_Adjacent, "{\"shape\":{\"t\":\"circle\",\"c\":{\"radius\":4}}}").shape == variant(Shape, Circle.{ 4 }), "the tag first");
    check(decode(Holder_Adjacent, "{\"shape\":{\"c\":{\"radius\":4},\"t\":\"circle\"}}").shape == variant(Shape, Circle.{ 4 }), "the content first");
    check(decode(Holder_Adjacent, "{\"shape\":{\"other\":1,\"t\":\"empty\",\"c\":{}}}").shape == variant(Shape, Empty.{}), "unknown keys are skipped");

    // A tag with an escape is a tag all the same.
    check(decode(Holder_Adjacent, "{\"shape\":{\"t\":\"c\\u0069rcle\",\"c\":{\"radius\":1}}}").shape == variant(Shape, Circle.{ 1 }), "an escaped tag");
}

Holder_Internal :: struct {
    shape: Shape; @json(tag=kind)
}

Holder_Adjacent :: struct {
    shape: Shape; @json(tag=t,content=c)
}

Holder_Untagged :: struct {
    shape: Shape; @json(untagged)
}

Snake_Holder :: struct @json(rename_all=snake_case) {
    first_shape: Shape;
    second_shape: Shape; @json(tag=kind)
    third_shape: Shape; @json(rename_all=SCREAMING_SNAKE_CASE)
}

rules :: () {
    // rename_all on the struct renames the tags as well as the keys.
    holder: Snake_Holder;

    set(*holder.first_shape, Circle.{ 1 });
    set(*holder.second_shape, Rect.{ 2, 3 });
    set(*holder.third_shape, Empty.{});

    text := encode(holder);

    check(text == "{\"first_shape\":{\"circle\":{\"radius\":1}},\"second_shape\":{\"kind\":\"rect\",\"width\":2,\"height\":3},\"third_shape\":{\"EMPTY\":{}}}", "tags follow the rename rules");

    back := decode(Snake_Holder, text);

    check(back.first_shape == holder.first_shape && back.second_shape == holder.second_shape && back.third_shape == holder.third_shape, "renamed tags decode");

    error := failure(Snake_Holder, "{\"first_shape\":{\"circle\":{\"radius\":1}},\"second_shape\":{\"kind\":\"rect\",\"width\":2,\"height\":3},\"third_shape\":{\"empty\":{}}}");

    check(error.kind == .UNKNOWN_VARIANT, "the case's own name is no longer a tag");
    check(contains(error.message, "CIRCLE, RECT, EMPTY"), "and the message lists the renamed tags");

    free_json(*error);
}

untagged :: () {
    shape := decode(Holder_Untagged, "{\"shape\":{\"radius\":4}}").shape;

    check(shape == variant(Shape, Circle.{ 4 }), "untagged: the first case that fits");
    check(decode(Holder_Untagged, "{\"shape\":{\"width\":1,\"height\":2}}").shape == variant(Shape, Rect.{ 1, 2 }), "untagged: the second case after the first fails");
    check(decode(Holder_Untagged, "{\"shape\":{\"radius\":4,\"width\":1,\"height\":2}}").shape == variant(Shape, Circle.{ 4 }), "untagged: extra fields do not stop the first case");

    // Nothing fits an array.
    error := failure(Holder_Untagged, "{\"shape\":[1]}");

    check(error.kind == .TYPE, "untagged: no case fits");
    check(contains(error.message, "Variant(Circle, Rect, Empty)") && contains(error.message, "`Circle`:") && contains(error.message, "`Rect`:") && contains(error.message, "`Empty`:"), "untagged: the message says why each case failed");
    check(error.path == "shape", "untagged: the path is the variant's");

    free_json(*error);

    // A case that fails halfway leaves nothing behind (see variant-leaks), and the next starts fresh.
    holder := decode(Holder_Pair, "{\"value\":{\"left\":1,\"right\":2}}");

    check(holder.value.pair.left == 1 && holder.value.pair.right == 2, "a failed case does not disturb the next");
}

Holder_Pair :: struct {
    value: Variant(Rect, Pair); @json(untagged)
}

Pair :: struct {
    left: int;
    right: int;
}

containers :: () {
    Many :: struct {
        list: [..] Shape;
        by_name: Table(string, Shape);
        maybe: Json_Maybe(Shape);
        pointer: *Shape;
        fixed: [2] Number_Or_Text;
    }

    text := "{\"list\":[{\"circle\":{\"radius\":1}},{\"empty\":{}},{\"rect\":{\"width\":2,\"height\":3}}],\"by_name\":{\"a\":{\"empty\":{}},\"b\":{\"circle\":{\"radius\":9}}},\"maybe\":{\"rect\":{\"width\":4,\"height\":5}},\"pointer\":{\"circle\":{\"radius\":6}},\"fixed\":[1,\"x\"]}";

    many := decode(Many, text);

    check(many.list.count == 3 && many.list[2] == variant(Shape, Rect.{ 2, 3 }), "an array of variants");
    check(many.by_name.count == 2 && table_find_pointer(*many.by_name, "b").* == variant(Shape, Circle.{ 9 }), "a table of variants");
    check(many.maybe.state == .SET && many.maybe.value == variant(Shape, Rect.{ 4, 5 }), "a Json_Maybe of a variant");
    check(many.pointer != null && many.pointer.* == variant(Shape, Circle.{ 6 }), "a pointer to a variant");
    check(many.fixed[0] == variant(Number_Or_Text, 1) && many.fixed[1] == variant(Number_Or_Text, "x"), "a fixed array of variants");

    again := encode(many);

    check(contains(again, "\"list\":[{\"circle\":{\"radius\":1}},{\"empty\":{}},{\"rect\":{\"width\":2,\"height\":3}}]"), "the array encodes");
    check(contains(again, "\"maybe\":{\"rect\":{\"width\":4,\"height\":5}}"), "the Json_Maybe encodes");
    check(contains(again, "\"pointer\":{\"circle\":{\"radius\":6}}"), "the pointer encodes");
    check(contains(again, "\"fixed\":[1,\"x\"]"), "the fixed array encodes");

    free_json(*many);

    // A Json_Maybe that is missing or null.
    Optional :: struct {
        shape: Json_Maybe(Shape);
    }

    check(decode(Optional, "{}").shape.state == .MISSING, "a missing variant");
    check(decode(Optional, "{\"shape\":null}").shape.state == .NULL, "a null variant");
    check(encode(Optional.{}) == "{}", "a missing variant is left out");

    // The notes reach the variants inside arrays and the like.
    Tagged :: struct {
        list: [..] Shape; @json(tag=kind)
        by_name: Table(string, Shape); @json(tag=t,content=c)
    }

    tagged := decode(Tagged, "{\"list\":[{\"kind\":\"circle\",\"radius\":1},{\"kind\":\"empty\"}],\"by_name\":{\"x\":{\"t\":\"rect\",\"c\":{\"width\":1,\"height\":1}}}}");

    check(tagged.list.count == 2 && tagged.list[0] == variant(Shape, Circle.{ 1 }), "the tag note applies to the elements");
    check(table_find_pointer(*tagged.by_name, "x").* == variant(Shape, Rect.{ 1, 1 }), "the note applies to the table values");

    free_json(*tagged);
}

Leaf :: struct {
    name: string;
}

Leaf_Or_Count :: Variant(Leaf, s64);

Group :: struct {
    items: [..] Leaf_Or_Count;
}

Tree :: Variant(struct {
    group: Group;
    leaf_or_count: Leaf_Or_Count;
});

nesting :: () {
    // A variant in a struct in a variant, and a variant as the case of another. (A variant cannot
    // contain itself: the compiler resolves a variadic type's arguments before the type.)
    text := "{\"group\":{\"items\":[{\"leaf\":{\"name\":\"a\"}},{\"s64\":5}]}}";

    tree := decode(Tree, text);
    group := tree.group;

    check(group.items.count == 2, "the group has two items");
    check(group.items[0].leaf.name == "a", "the first item is a leaf");
    check(group.items[1].s64_ == 5, "the second item is a count");
    check(encode(tree) == text, "the tree round trips");

    free_json(*tree);

    check(tree.which == .NONE, "freeing a tree empties it");

    // A variant as the case of another.
    nested := decode(Tree, "{\"leaf_or_count\":{\"s64\":9}}");

    check(nested.which == .LEAF_OR_COUNT && nested.leaf_or_count.s64_ == 9, "a variant inside a variant");

    free_json(*nested);
}

errors :: () {
    error := failure(Shape, "{\"Triangle\":{}}");

    check(error.kind == .UNKNOWN_VARIANT, "an unknown tag");
    check(error.message == "unknown variant `Triangle`, expected one of: circle, rect, empty", "names the tag and the choices");
    check(error.offset == 1 && error.line == 1 && error.column == 2, "points at the tag");

    free_json(*error);

    error = failure(Holder_Internal, "{\"shape\":{\"kind\":\"Oval\",\"radius\":1}}");

    check(error.kind == .UNKNOWN_VARIANT && error.path == "shape", "an unknown internal tag, in a path");
    check(contains(error.message, "`Oval`") && contains(error.message, "circle, rect, empty"), "an unknown internal tag, in a message");

    free_json(*error);

    error = failure(Holder_Adjacent, "{\"shape\":{\"t\":\"Oval\",\"c\":{}}}");

    check(error.kind == .UNKNOWN_VARIANT && error.path == "shape", "an unknown adjacent tag");

    free_json(*error);

    error = failure(Holder_Internal, "{\"shape\":{\"radius\":1}}");

    check(error.kind == .MISSING_FIELD && contains(error.message, "`kind`") && error.path == "shape", "no tag");

    free_json(*error);

    error = failure(Holder_Internal, "{\"shape\":{\"kind\":3}}");

    check(error.kind == .TYPE && error.path == "shape", "a tag that is not a string");

    free_json(*error);

    error = failure(Holder_Internal, "{\"shape\":[]}");

    check(error.kind == .TYPE && contains(error.message, "object"), "an internally tagged variant must be an object");

    free_json(*error);

    error = failure(Holder_Adjacent, "{\"shape\":{\"t\":\"circle\"}}");

    check(error.kind == .MISSING_FIELD && contains(error.message, "`c`"), "no content");

    free_json(*error);

    error = failure(Holder_Adjacent, "{\"shape\":{\"t\":\"circle\",\"c\":{\"radius\":1},\"c\":{\"radius\":2}}}");

    check(error.kind == .DUPLICATE_FIELD, "two contents");

    free_json(*error);

    value, duplicates := from_json(Holder_Adjacent, "{\"shape\":{\"t\":\"circle\",\"c\":{\"radius\":1},\"c\":{\"radius\":2}}}", Json_Options.{ duplicate_keys = .LAST });

    check(duplicates.kind == .NONE && value.shape == variant(Shape, Circle.{ 2 }), "the last content wins with duplicate_keys = .LAST");

    error = failure(Shape, "{}");

    check(error.kind == .TYPE && contains(error.message, "one key"), "an empty object for an externally tagged variant");

    free_json(*error);

    error = failure(Shape, "{\"circle\":{\"radius\":1},\"rect\":{}}");

    check(error.kind == .TYPE && contains(error.message, "exactly one key"), "two keys for an externally tagged variant");

    free_json(*error);

    error = failure(Shape, "\"circle\"");

    check(error.kind == .TYPE, "a string for an externally tagged variant");

    free_json(*error);

    error = failure(Shape, "{\"rect\":{\"width\":1,\"height\":\"tall\"}}");

    check(error.kind == .TYPE && error.path == "rect.height", "a bad payload: the path goes through the tag");

    free_json(*error);

    error = failure(Holder_Internal, "{\"shape\":{\"kind\":\"rect\",\"width\":1,\"height\":\"tall\"}}");

    check(error.kind == .TYPE && error.path == "shape.height", "a bad internally tagged payload");

    free_json(*error);

    error = failure(Holder_Adjacent, "{\"shape\":{\"t\":\"rect\",\"c\":{\"width\":true}}}");

    check(error.kind == .TYPE && error.path == "shape.c.width", "a bad adjacently tagged payload");

    free_json(*error);

    strict := Json_Options.{ deny_unknown_fields = true };

    strict_value, strict_error := from_json(Holder_Internal, "{\"shape\":{\"kind\":\"circle\",\"radius\":1,\"extra\":2}}", strict);

    check(strict_error.kind == .UNKNOWN_FIELD && strict_error.path == "shape", "deny_unknown_fields reaches an internally tagged case, but not its tag");

    free_json(*strict_error);

    strict_value, strict_error = from_json(Holder_Internal, "{\"shape\":{\"kind\":\"circle\",\"radius\":1}}", strict);

    check(strict_error.kind == .NONE, "the tag is not an unknown field");

    adjacent_value, adjacent_error := from_json(Holder_Adjacent, "{\"shape\":{\"t\":\"circle\",\"c\":{\"radius\":1},\"extra\":2}}", strict);

    check(adjacent_error.kind == .UNKNOWN_FIELD, "deny_unknown_fields reaches an adjacently tagged variant");

    free_json(*adjacent_error);
    free_json(*strict_value);
    free_json(*adjacent_value);

    // The tag key is no unknown field, and a decoder still needs the fields it has.
    error = failure(Holder_Internal, "{\"shape\":{\"kind\":\"rect\",\"width\":1}}");

    check(error.kind == .MISSING_FIELD && contains(error.message, "`height`"), "a missing field of an internally tagged case");

    free_json(*error);

    error = failure(Number_Or_Text, "[]");

    check(error.kind == .TYPE && contains(error.message, "`s64`") && contains(error.message, "`string`"), "untagged scalars name every case in the error");

    free_json(*error);

    error = failure(Shape, "{\"circle\":{\"radius\":1}} 7");

    check(error.kind == .SYNTAX, "trailing data");

    free_json(*error);
}

defaults :: () {
    Config :: struct @json(default) {
        name: string;
        shape: Shape;
        count: int;
    }

    config := decode(Config, "{\"name\":\"n\"}");

    check(config.shape.which == .NONE && config.count == 0, "a variant left out keeps its zero value");

    // Required when nothing says otherwise.
    Required :: struct {
        shape: Shape;
    }

    error := failure(Required, "{}");

    check(error.kind == .MISSING_FIELD, "a missing variant is an error");

    free_json(*error);
    free_json(*config);
}

freeing :: () {
    Owner :: struct {
        text: Variant(s64, string);
        list: Variant(struct {
            empty: Empty;
            list: [..] string;
        });
        shape: Shape;
    }

    owner := decode(Owner, "{\"text\":\"owned\",\"list\":{\"list\":[\"a\",\"b\"]},\"shape\":{\"empty\":{}}}");

    check(owner.text.string_ == "owned" && owner.list.list.count == 2, "decoded");

    free_json(*owner);

    check(owner.text.which == .NONE && owner.list.which == .NONE && owner.shape.which == .NONE, "free_json zeroes the variants");

    free_json(*owner);

    // A variant holding its declared default is not freed: it may be a literal.
    Defaulted :: struct @json(default) {
        text := Number_Or_Text.{};
        other: Number_Or_Text;
    }

    defaulted := decode(Defaulted, "{\"other\":\"heap\"}");

    free_json(*defaulted);

    check(defaulted.other.which == .NONE, "a decoded variant is freed");
}

top_level_free :: () {
    text := decode(Number_Or_Text, "\"standalone\"");

    check(text.string_ == "standalone", "a top-level variant");

    free_json(*text);

    check(text.which == .NONE, "a top-level variant is freed and zeroed");
}

main :: () {
    for pass: 0..1 {
        json_use_simd = pass == 1;
        json_index_min_size = 0;

        top_level();
        scalars();
        forms();
        orders();
        rules();
        untagged();
        containers();
        nesting();
        errors();
        defaults();
        freeing();
        top_level_free();
    }

    assert(failures == 0);
    print("ok\n");
}
