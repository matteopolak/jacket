// json_remove: dropping a member from an object tree.
#scope_file
#import "Basic";
#import "Extensions/Testing";
Json :: #import "Extensions/Json";

#scope_export

remove_keeps_the_other_members_in_order :: () {
    object, error := Json.from_json(Json.Json_Value, "{\"a\":1,\"b\":[1,2],\"c\":{\"d\":null}}");

    require(error.kind == .NONE);
    defer Json.free_json(*object);

    expect(Json.json_remove(*object, "b"));
    expect_equal(Json.to_json(object,, temp), "{\"a\":1,\"c\":{\"d\":null}}");
    expect(Json.json_get(*object, "b") == null);
    expect_equal(Json.json_count(*object), 2);
} @test

remove_reports_a_missing_key_and_a_wrong_kind :: () {
    object := Json.json_object();

    defer Json.free_json(*object);
    Json.json_set(*object, "a", Json.json_int(1));

    expect(!Json.json_remove(*object, "missing"));
    expect_equal(Json.json_count(*object), 1);

    list := Json.json_array();

    defer Json.free_json(*list);
    Json.json_push(*list, Json.json_int(1));

    expect(!Json.json_remove(*list, "a"));
    expect_equal(Json.json_count(*list), 1);
} @test

// A document that repeats a key keeps every copy; removing the key removes them all, so `json_get`
// finds nothing afterwards.
remove_drops_every_duplicate :: () {
    object, error := Json.from_json(Json.Json_Value, "{\"a\":1,\"b\":2,\"a\":3}");

    require(error.kind == .NONE);
    defer Json.free_json(*object);

    expect(Json.json_remove(*object, "a"));
    expect(Json.json_get(*object, "a") == null);
    expect_equal(Json.to_json(object,, temp), "{\"b\":2}");
} @test

remove_leaves_the_object_usable :: () {
    object := Json.json_object();

    defer Json.free_json(*object);
    Json.json_set(*object, "a", Json.json_string("x"));
    expect(Json.json_remove(*object, "a"));
    expect_equal(Json.json_count(*object), 0);
    Json.json_set(*object, "a", Json.json_string("y"));
    expect_equal(Json.to_json(object,, temp), "{\"a\":\"y\"}");
} @test
