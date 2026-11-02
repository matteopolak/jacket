#scope_file
#import "Basic";
#import "String";
#import "Extensions/Testing";
Json :: #import "Extensions/Json";
Api :: #import "Extensions/Protobuf/Schema"(TEXT = "syntax='proto3'; message M { string type=1 [json_name='a,b) \\\"\\n']; } message E { string type=1 [json_name='']; }");

#scope_export

json_name_notes :: () {
    value := Api.M.{ type_ = "ok" };
    text := Json.to_json(value);

    defer free(text);
    empty := Json.to_json(Api.E.{ type_ = "yes" });

    defer free(empty);
    expect(begins_with(empty, "{\"\":\"yes\""));
    // Json also sees the preserved unknown byte array. The rename must be one complete key.
    expect(begins_with(text, "{\"a,b) \\\"\\n\":\"ok\""));
} @test
