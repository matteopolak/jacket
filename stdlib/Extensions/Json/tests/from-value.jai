// from_json on a Json_Value: decoding a parsed tree gives the value, the error kind, the message and
// the path that decoding its text gives, for every kind of field and for the options.
#import "Basic";
#import "Hash_Table";
#import "Extensions/Json";
#import "Extensions/Variant";

Color :: enum {
    RED;
    GREEN;
}

Point :: struct {
    x: s32;
    y: s32;
}

Circle :: struct {
    radius: s32;
}

Rect :: struct {
    width: s32;
    height: s32;
}

Shape :: Variant(Circle, Rect);

Scalar :: Variant(s64, string, Color);

Strict :: struct @json(deny_unknown_fields) {
    id: s32;
}

Doc :: struct @json(rename_all=camelCase) {
    id: u8;
    ratio: float32 = 1; @json(default)
    name: string;
    flag: bool; @json(default)
    color: Color; @json(default)
    point: Point; @json(default)
    tags: [] string; @json(default)
    coords: [3] float32; @json(default)
    list: [..] Point; @json(default)
    note: *Point; @json(default,skip_serializing_if=null)
    extras: Json_Value; @json(default,skip_serializing_if=null)
    maybe: Json_Maybe(s32);
    counts: Table(string, s32); @json(default)
    shape: Shape; @json(default)
    scalar: Scalar; @json(default)
    strict: Strict; @json(default)
    start_index: s32 = 4; @json(default,alias=first)
}

failures := 0;

check :: (condition: bool, what: string) {
    if condition return;

    print("FAIL: %\n", what);
    failures += 1;
}

// Decodes `text` directly and through a parsed tree, and requires the same outcome.
same :: ($T: Type, text: string, options := Json_Options.{}) {
    tree, parse_error := from_json(Json_Value, text);

    if parse_error.kind != .NONE {
        print("FAIL: % is not JSON\n", text);
        failures += 1;
        free_json(*parse_error);

        return;
    }

    defer free_json(*tree);

    direct, direct_error := from_json(T, text, options);
    through, through_error := from_json(T, *tree, options);

    defer free_json(*direct_error);
    defer free_json(*through_error);

    if direct_error.kind != through_error.kind || direct_error.message != through_error.message || direct_error.path != through_error.path {
        print("FAIL: % errors differ\n  text: % at %\n  tree: % at %\n", text, direct_error.message, direct_error.path, through_error.message, through_error.path);
        failures += 1;

        return;
    }

    if direct_error.kind != .NONE return;

    defer free_json(*direct);
    defer free_json(*through);

    left := to_json(direct,, temp);
    right := to_json(through,, temp);

    if left != right {
        print("FAIL: % values differ\n  text: %\n  tree: %\n", text, left, right);
        failures += 1;
    }
}

main :: () {
    // Valid documents.
    same(Doc, "{\"id\":7,\"name\":\"a\",\"maybe\":null}");
    same(Doc, "{\"id\":7,\"name\":\"a\\n\\u00e9\",\"maybe\":-3,\"ratio\":0.25,\"flag\":true,\"color\":\"GREEN\",\"point\":{\"x\":1,\"y\":-2},\"tags\":[\"p\",\"q\"],\"coords\":[1,2.5,3e2],\"list\":[{\"x\":1,\"y\":2},{\"x\":3,\"y\":4}],\"note\":{\"x\":9,\"y\":8},\"extras\":{\"a\":[1,{\"b\":null}],\"c\":\"d\"},\"counts\":{\"one\":1,\"two\":2},\"shape\":{\"rect\":{\"width\":2,\"height\":3}},\"scalar\":\"GREEN\",\"strict\":{\"id\":5},\"first\":9}");
    same(Doc, "{\"id\":7,\"name\":\"\",\"maybe\":1,\"note\":null,\"shape\":{\"circle\":{\"radius\":1}},\"scalar\":12,\"extras\":null}");
    same(Doc, "{\"id\":7,\"name\":\"a\",\"maybe\":1,\"scalar\":\"text\"}");
    same(Doc, "{\"id\":7,\"name\":\"a\",\"maybe\":1,\"unknown\":[1,2,{}]}");

    // Wrong shapes, with their paths.
    same(Doc, "[]");
    same(Doc, "5");
    same(Doc, "null");
    same(Doc, "{\"name\":\"a\",\"maybe\":1}");
    same(Doc, "{\"id\":300,\"name\":\"a\",\"maybe\":1}");
    same(Doc, "{\"id\":-1,\"name\":\"a\",\"maybe\":1}");
    same(Doc, "{\"id\":1.5,\"name\":\"a\",\"maybe\":1}");
    same(Doc, "{\"id\":\"x\",\"name\":\"a\",\"maybe\":1}");
    same(Doc, "{\"id\":1,\"name\":5,\"maybe\":1}");
    same(Doc, "{\"id\":1,\"name\":\"a\",\"maybe\":\"x\"}");
    same(Doc, "{\"id\":1,\"name\":\"a\",\"maybe\":1,\"ratio\":1e999}");
    same(Doc, "{\"id\":1,\"name\":\"a\",\"maybe\":1,\"flag\":\"yes\"}");
    same(Doc, "{\"id\":1,\"name\":\"a\",\"maybe\":1,\"color\":\"BLUE\"}");
    same(Doc, "{\"id\":1,\"name\":\"a\",\"maybe\":1,\"color\":1}");
    same(Doc, "{\"id\":1,\"name\":\"a\",\"maybe\":1,\"point\":{\"x\":1}}");
    same(Doc, "{\"id\":1,\"name\":\"a\",\"maybe\":1,\"point\":{\"x\":1,\"y\":true}}");
    same(Doc, "{\"id\":1,\"name\":\"a\",\"maybe\":1,\"tags\":[\"a\",2]}");
    same(Doc, "{\"id\":1,\"name\":\"a\",\"maybe\":1,\"coords\":[1,2]}");
    same(Doc, "{\"id\":1,\"name\":\"a\",\"maybe\":1,\"coords\":[1,2,3,4]}");
    same(Doc, "{\"id\":1,\"name\":\"a\",\"maybe\":1,\"list\":[{\"x\":1,\"y\":2},{\"x\":\"3\",\"y\":4}]}");
    same(Doc, "{\"id\":1,\"name\":\"a\",\"maybe\":1,\"note\":[]}");
    same(Doc, "{\"id\":1,\"name\":\"a\",\"maybe\":1,\"counts\":{\"a\":1,\"b\":\"c\"}}");
    same(Doc, "{\"id\":1,\"name\":\"a\",\"maybe\":1,\"shape\":{\"circle\":{\"radius\":\"a\"}}}");
    same(Doc, "{\"id\":1,\"name\":\"a\",\"maybe\":1,\"shape\":{\"hex\":{}}}");
    same(Doc, "{\"id\":1,\"name\":\"a\",\"maybe\":1,\"scalar\":2.5}");
    same(Doc, "{\"id\":1,\"name\":\"a\",\"maybe\":1,\"scalar\":[]}");
    same(Doc, "{\"id\":1,\"name\":\"a\",\"maybe\":1,\"strict\":{\"id\":1,\"more\":2}}");
    same(Doc, "{\"id\":1,\"id\":2,\"name\":\"a\",\"maybe\":1}");

    // Top-level types that are not structs.
    same([] s64, "[1,2,3]");
    same([] s64, "[1,\"2\"]");
    same(string, "\"text\"");
    same(string, "5");
    same(Json_Value, "{\"a\":[1,2,{\"b\":true}]}");
    same(Table(string, [] s32), "{\"a\":[1],\"b\":[]}");
    same(Json_Maybe(s32), "null");
    same(Shape, "{\"circle\":{\"radius\":2}}");
    same(Scalar, "7");

    // Options.
    strict := Json_Options.{ deny_unknown_fields = true };

    same(Doc, "{\"id\":1,\"name\":\"a\",\"maybe\":1,\"unknown\":1}", strict);

    defaults := Json_Options.{ missing_fields_default = true };

    same(Doc, "{}", defaults);

    last := Json_Options.{ duplicate_keys = .LAST };

    same(Doc, "{\"id\":1,\"id\":2,\"name\":\"a\",\"maybe\":1}", last);

    shallow := Json_Options.{ max_depth = 3 };

    same(Doc, "{\"id\":1,\"name\":\"a\",\"maybe\":1,\"extras\":[[[[1]]]]}", shallow);
    same(Doc, "{\"id\":1,\"name\":\"a\",\"maybe\":1,\"unknown\":[[[[1]]]]}", shallow);
    same(Doc, "{\"id\":1,\"name\":\"a\",\"maybe\":1,\"list\":[{\"x\":1,\"y\":1}]}", shallow);

    // Borrowing: the strings are the tree's own.
    tree, _ := from_json(Json_Value, "{\"id\":1,\"name\":\"borrowed\",\"maybe\":null}");
    borrowed, borrow_error := from_json(Doc, *tree, Json_Options.{ borrow = true });

    check(borrow_error.kind == .NONE, "borrowed decode");
    check(borrowed.name == "borrowed" && borrowed.name.data == json_get(*tree, "name").text.data, "borrowed strings point into the tree");
    free_json(*tree);

    // The result does not depend on the tree: free it first.
    tree, _ = from_json(Json_Value, "{\"id\":1,\"name\":\"owned\",\"maybe\":2,\"extras\":{\"k\":[1,2]},\"tags\":[\"x\"]}");
    owned, owned_error := from_json(Doc, *tree);

    free_json(*tree);
    check(owned_error.kind == .NONE && owned.name == "owned" && owned.tags[0] == "x", "owned decode");
    check(json_count(json_get(*owned.extras, "k")) == 2, "extras are copied");
    free_json(*owned);

    // to_json_value: encoding into a tree and decoding it again gives the value back.
    source, source_error := from_json(Doc, "{\"id\":7,\"name\":\"a\",\"maybe\":-3,\"ratio\":0.25,\"point\":{\"x\":1,\"y\":-2},\"list\":[{\"x\":1,\"y\":2}],\"note\":{\"x\":9,\"y\":8},\"extras\":{\"a\":[1,null]},\"counts\":{\"one\":1},\"shape\":{\"rect\":{\"width\":2,\"height\":3}},\"scalar\":\"GREEN\"}");

    check(source_error.kind == .NONE, "source document");

    encoded, encode_error := to_json_value(source);

    check(encode_error.kind == .NONE, "to_json_value");

    text := to_json(source,, temp);
    parsed, _ := from_json(Json_Value, text);
    again := to_json(encoded,, temp);

    check(again == to_json(parsed,, temp), "the tree holds what to_json writes");
    free_json(*parsed);

    back, back_error := from_json(Doc, *encoded);

    check(back_error.kind == .NONE && to_json(back,, temp) == text, "a tree encoded from a value decodes to it");
    free_json(*back);
    free_json(*encoded);
    free_json(*source);

    if failures {
        print("% failures\n", failures);
        exit(1);
    }

    print("ok\n");
}
