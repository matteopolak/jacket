// Quoted opaque notes can contain punctuation that otherwise separates note options.
#scope_file
#import "Basic";
#import "Extensions/Testing";
Json :: #import "Extensions/Json";

Named :: struct {
    value: string; @"json(rename=\"a,b) \\\"\\n\")"
}

Empty_Key :: struct {
    value: string; @"json(rename=\"\")"
}

#scope_export

quoted_note_names :: () {
    text := Json.to_json(Named.{ "ok" });

    defer free(text);
    expect_equal(text, "{\"a,b) \\\"\\n\":\"ok\"}");
    value, error := Json.from_json(Named, text);

    require(error.kind == .NONE);
    defer Json.free_json(*value);
    expect_equal(value.value, "ok");
    empty := Json.to_json(Empty_Key.{ "yes" });

    defer free(empty);
    expect_equal(empty, "{\"\":\"yes\"}");
} @test
