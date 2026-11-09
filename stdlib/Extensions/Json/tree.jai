// Typed values from and into a parsed `Json_Value`.

// Decodes a parsed `Json_Value` into a `T`, the way [from_json] decodes text: the same notes,
// options, errors and paths, and the same ownership (the result is allocated from
// context.allocator and released with free_json; with `options.borrow` its strings are the
// tree's own). The generated decoder reads the tree directly, so nothing is written out and
// parsed again. The tree is not changed. Errors have no position in the text (offset 0, line 1,
// column 1); the kind, message and path match the ones decoding the same text reports.
//
// ```jai
// Point :: struct {
//     x: s32;
//     y: s32;
// }
//
// tree, _ := from_json(Json_Value, "{\"x\": 1, \"y\": 2}");
// point, error := from_json(Point, *tree);
// assert(error.kind == .NONE && point.x == 1 && point.y == 2);
// free_json(*tree);
//
// // The part of a document that is one type: decode a member by itself.
// document, _ := from_json(Json_Value, "{\"at\": {\"x\": 5, \"y\": \"six\"}}");
// _, error = from_json(Point, json_get(*document, "at"));
// assert(error.kind == .TYPE && error.path == "y");
// free_json(*error);
// free_json(*document);
// ```
from_json :: ($T: Type, value: *Json_Value, options := Json_Options.{}) -> T, Json_Error {
    empty: T;
    r: Json_Reader;

    reader_init_tree(*r, value, options);
    defer reader_free(*r);

    result: T;

    if !json_decode(*r, *result) return empty, finish_error(*r);

    return result, .{};
}

// Encodes `value` into a `Json_Value`, the inverse of [from_json] on a value: the tree holds what
// [to_json] writes, so it can be edited, stored in a larger tree with `json_set`, and decoded
// again. The encoder is the generated one; the tree is built by reading its output. Release the
// tree with free_json. The error is set only when the output cannot be read back (it nests
// deeper than 256 levels).
//
// ```jai
// Point :: struct {
//     x: s32;
//     y: s32;
// }
//
// tree, error := to_json_value(Point.{ 3, 4 });
// assert(error.kind == .NONE);
// assert(json_count(*tree) == 2);
// x, _ := json_as_s64(json_get(*tree, "x"));
// assert(x == 3);
//
// json_set(*tree, "y", json_int(9));
// point, _ := from_json(Point, *tree);
// assert(point.y == 9);
// free_json(*tree);
// ```
to_json_value :: (value: $T) -> Json_Value, Json_Error {
    text := to_json(value);

    defer free(text);

    return from_json(Json_Value, text);
}
