#import "Basic";
#import "String";
#import "Extensions/Testing";
#import "Extensions/Json";

positions_use_utf16_and_utf8_boundaries :: (t: *Test) {
    text := "aé😀z\r\n日x\n";
    pos := Lsp.position(text, 7);

    expect(pos.line == 0 && pos.character == 4);
    expect(Lsp.byte_offset(text, .{ line = 0, character = 4 }) == 7);
    expect(Lsp.byte_offset(text, .{ line = 0, character = 3 }) == 3);
    expect(Lsp.position(text, 5).character == 2);
    expect(Lsp.byte_offset(text, .{ line = 0, character = 100 }) == 8);
    expect(Lsp.byte_offset(text, .{ line = 1, character = 1 }) == 13);
    expect(Lsp.position(text, 13).line == 1);
    expect(Lsp.position(text, 13).character == 1);
    expect(Lsp.byte_offset(text, .{ line = 50, character = 0 }) == text.count);
} @test

framing_waits_for_every_byte_and_preserves_the_next_message :: (t: *Test) {
    client: Lsp.Client;

    defer Lsp.stop(*client);
    body := "{\"text\":\"é😀\"}";
    packet := tprint("Content-Length: %\r\nContent-Type: application/vscode-jsonrpc; charset=utf-8\r\n\r\n%", body.count, body);

    for packet {
        array_add(*client.input, it);
        result := Lsp.take_frame(*client);
        if it_index < packet.count - 1 expect(result.count == 0);
        else expect(result == body);

        free(result);
    }

    for packet array_add(*client.input, it);
    for packet array_add(*client.input, it);

    first := Lsp.take_frame(*client);
    second := Lsp.take_frame(*client);

    defer free(first);
    defer free(second);
    expect(first == body && second == body);
    expect(client.input.count == 0 && !client.failed);
} @test

bad_frame_lengths_fail_closed :: (t: *Test) {
    packets := string.["Content-Length: -1\r\n\r\n", "Content-Length: 999999999\r\n\r\n", "Content-Length: 1\r\nContent-Length: 1\r\n\r\nx", "Other: x\r\n\r\nx"];
    for packets {
        client: Lsp.Client;
        for it array_add(*client.input, it);

        body := Lsp.take_frame(*client);

        free(body);
        expect(client.failed);
        Lsp.stop(*client);
    }
} @test

request_ids_are_consumed_once_and_out_of_order :: (t: *Test) {
    client: Lsp.Client;

    defer Lsp.stop(*client);
    array_add(*client.pending, .{ id = 3, method = copy_string("hover"), uri = copy_string("file:///a"), version = 7, cursor = 9 });
    array_add(*client.pending, .{ id = 4, method = copy_string("definition"), uri = copy_string("file:///a"), version = 8, cursor = 2 });
    pending, found := Lsp.finish(*client, 4);

    expect(found && pending.version == 8 && pending.cursor == 2);
    Lsp.free_pending(*pending);
    _, found = Lsp.finish(*client, 4);
    expect(!found);
    Lsp.cancel_document(*client, "file:///a");
    expect(client.pending.count == 0);
    _, found = Lsp.finish(*client, 3);
    expect(!found);
} @test

file_uris_round_trip_unicode_spaces_and_percent :: (t: *Test) {
    path := "/tmp/a b/é😀%#.jai";
    uri := Lsp.uri(path);

    expect(contains(uri, "%20") && contains(uri, "%25") && contains(uri, "%23"));
    expect(Lsp.path_from_uri(uri) == path);
    expect(Lsp.path_from_uri("https://example/a") == "");
} @test

incremental_edits_round_trip_unicode_and_line_endings :: (t: *Test) {
    samples := string.["", "aé😀z\r\n日x\n", "aê😁z\r\n日x\n", "x\r\ny", "x\ry", "x\ny", "😀", "é", "終\n", "abc", "a\r\n", "a\r"];
    for old: samples for replacement : samples {
        encoded := Lsp.change(old, replacement);
        change, error := from_json(Json_Value, encoded);

        expect(error.kind == .NONE);
        range := Lsp.get(*change, "range");
        start := Lsp.byte_offset(old, Lsp.read_position(Lsp.get(range, "start")));
        end := Lsp.byte_offset(old, Lsp.read_position(Lsp.get(range, "end")));
        inserted := Lsp.text(Lsp.get(*change, "text"));
        after := tprint("%%%", slice(old, 0, start), inserted, slice(old, end, old.count - end));

        expect(after == replacement);
        free_json(*change);
        free_json(*error);
    }
} @test

positions_handle_lone_cr_and_the_middle_of_crlf :: (t: *Test) {
    expect(Lsp.position("a\r\nb", 2).line == 0);
    expect(Lsp.position("a\r\nb", 2).character == 1);
    expect(Lsp.position("a\r\nb", 3).line == 1);
    expect(Lsp.position("a\rb", 2).line == 1);
    expect(Lsp.byte_offset("a\rb", .{ line = 1, character = 1 }) == 3);
} @test

header_names_are_case_insensitive :: (t: *Test) {
    client: Lsp.Client;

    defer Lsp.stop(*client);
    for "content-length: 2\r\n\r\n{}" array_add(*client.input, it);

    body := Lsp.take_frame(*client);

    defer free(body);
    expect(body == "{}" && !client.failed);
} @test
