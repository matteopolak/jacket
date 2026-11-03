// Editor-local JSON-RPC transport and LSP coordinate conversion. No UI dependency.
#import "Basic";
#import "String";
#import "Process";
#import "Thread";
#import "Unicode";
#import "Extensions/Json";

Position :: struct {
    line: int;
    character: int;
}

Pending :: struct {
    id: int;
    method: string;
    uri: string;
    version: int;
    cursor: int;
}

Client :: struct {
    process: Process; // address must remain stable until stop
    running: bool;
    ready: bool;
    failed: bool;
    incremental: bool;
    writer: Thread;
    mutex: Mutex;
    outgoing: [..] string;
    queued_bytes: int;
    stopping: bool;
    writer_failed: bool;
    started_at: int;
    next_id: int = 1;
    input: [..] u8;
    pending: [..] Pending;
}

MAX_MESSAGE :: 8 * 1024 * 1024;

// A byte inside a multibyte character rounds down to its start.
position :: (text: string, offset: int) -> Position {
    result: Position;
    at := 0;
    limit := clamp(offset, 0, text.count);

    while at < limit {
        code, width, _ := character_utf8_to_utf32(text.data + at, text.count - at);
        width = max(width, 1);
        if code == 13 && at + 1 < text.count && text[at + 1] == 10 width = 2;
        if at + width > limit break;

        if code == 10 || code == 13 {
            result.line += 1;
            result.character = 0;
        } else {
            result.character += ifx code > 0xffff then 2 else 1;
        }

        at += width;
    }

    return result;
}

// Invalid/out-of-line columns clamp to the line end, never into a UTF-8 sequence.
byte_offset :: (text: string, pos: Position) -> int {
    at := 0;
    line := 0;

    while at < text.count && line < pos.line {
        if text[at] == 10 || text[at] == 13 {
            line += 1;
            if text[at] == 13 && at + 1 < text.count && text[at + 1] == 10 at += 1;
        }

        at += 1;
    }

    column := 0;

    while at < text.count && text[at] != 10 && text[at] != 13 {
        code, width, _ := character_utf8_to_utf32(text.data + at, text.count - at);
        width = max(width, 1);
        units := ifx code > 0xffff then 2 else 1;
        if column + units > pos.character break;

        column += units;
        at += width;
    }

    return at;
}

quote :: (text: string) -> string {
    value := to_json(text);
    defer free(value);
    return copy_string(value,, temp);
}

// Percent encode every non-unreserved byte; slashes and drive colons remain separators.
uri :: (path: string) -> string {
    builder: String_Builder;
    builder.allocator = temp;
    append(*builder, "file://");
    if path.count && path[0] != 47 append(*builder, "/");

    hex :: "0123456789ABCDEF";

    for path {
        c := it;
        if c == 92 c = 47;

        if (c >= 65 && c <= 90) || (c >= 97 && c <= 122) || (c >= 48 && c <= 57) || contains("/-_.~:", string.{ 1, *c }) {
            append(*builder, string.{ 1, *c });
        } else {
            append(*builder, "%");
            append(*builder, slice(hex, c / 16, 1));
            append(*builder, slice(hex, c % 16, 1));
        }
    }

    return builder_to_string(*builder,, temp);
}

path_from_uri :: (value: string) -> string {
    if !begins_with(value, "file:///") return "";

    builder: String_Builder;
    builder.allocator = temp;
    at := 7;

    #if OS == .WINDOWS at = 8;

    while at < value.count {
        c := value[at];
        if c == 37 && at + 2 < value.count {
            high := hex_digit(value[at + 1]);
            low := hex_digit(value[at + 2]);

            if high < 0 || low < 0 return "";

            c = cast(u8) (high * 16 + low);
            if c == 0 return "";

            at += 2;
        }

        append(*builder, string.{ 1, *c });
        at += 1;
    }

    return builder_to_string(*builder,, temp);
}

hex_digit :: (c: u8) -> int {
    if c >= 48 && c <= 57 return c - 48;
    if c >= 65 && c <= 70 return c - 55;
    if c >= 97 && c <= 102 return c - 87;
    return -1;
}

// Returns an owned body, or empty until complete. Bad/oversize frames fail closed.
take_frame :: (client: *Client) -> string {
    text := string.{ client.input.count, client.input.data };
    split := find_index_from_left(text, "\r\n\r\n");

    if split < 0 {
        if text.count > 8192 client.failed = true;
        return "";
    }

    if split > 8192 {
        client.failed = true;
        return "";
    }

    length := -1;
    start := 0;

    while start < split {
        end := find_index_from_left(slice(text, start, split + 2 - start), "\r\n");
        if end < 0 {
            client.failed = true;
            return "";
        }

        header := slice(text, start, end);
        if header.count >= 15 && equal_nocase(slice(header, 0, 15), "Content-Length:") {
            if length >= 0 {
                client.failed = true;
                return "";
            }

            length = 0;
            digits := false;

            for slice(header, 15, header.count - 15) {
                if it == 32 && !digits continue;

                if it < 48 || it > 57 {
                    client.failed = true;
                    return "";
                }

                digits = true;
                length = length * 10 + it - 48;
                if length > MAX_MESSAGE {
                    client.failed = true;
                    return "";
                }
            }

            if !digits {
                client.failed = true;
                return "";
            }
        }

        start += end + 2;
    }

    if length <= 0 {
        client.failed = true;
        return "";
    }

    count := split + 4 + length;
    if text.count < count return "";

    result := copy_string(slice(text, split + 4, length));
    remaining := text.count - count;

    for 0..remaining - 1 client.input[it] = client.input[count + it];

    client.input.count = remaining;

    return result;
}

send :: (client: *Client, method: string, params: string, id: int = 0) {
    if !client.running || client.failed return;

    body := ifx id > 0 then tprint("{\"jsonrpc\":\"2.0\",\"id\":%,\"method\":%,\"params\":%}", id, quote(method), params)
        else tprint("{\"jsonrpc\":\"2.0\",\"method\":%,\"params\":%}", quote(method), params);
    packet := tprint("Content-Length: %\r\n\r\n%", body.count, body);

    lock(*client.mutex);
    defer unlock(*client.mutex);

    if client.queued_bytes + packet.count > MAX_MESSAGE * 2 {
        client.writer_failed = true;
        return;
    }

    array_add(*client.outgoing, copy_string(packet));
    client.queued_bytes += packet.count;
}

// Only the writer thread can wait on pipe capacity. UI reads use timeout zero.
write_packets :: (thread: *Thread) -> s64 {
    client := cast(*Client) thread.data;

    while true {
        packet: string;

        lock(*client.mutex);
        stop := client.stopping;
        if !stop && client.outgoing.count {
            packet = client.outgoing[0];
            array_ordered_remove_by_index(*client.outgoing, 0);
            client.queued_bytes -= packet.count;
        }

        unlock(*client.mutex);
        if stop break;

        if !packet.count {
            sleep_milliseconds(2);
            continue;
        }

        at := 0;
        while at < packet.count {
            ok, count := write_to_process(*client.process, .{ packet.count - at, packet.data + at });
            if !ok || count <= 0 {
                lock(*client.mutex);
                client.writer_failed = true;
                unlock(*client.mutex);
                break;
            }

            at += count;
        }

        free(packet);
    }

    return 0;
}

request :: (client: *Client, method: string, params: string, uri: string = "", version: int = 0, cursor: int = 0) -> int {
    if !client.running || client.failed return 0;

    id := client.next_id;

    client.next_id += 1;
    array_add(*client.pending, .{ id = id, method = copy_string(method), uri = copy_string(uri), version = version, cursor = cursor });
    send(client, method, params, id);

    return id;
}

// Remove exactly once. Cancelled/unknown response ids are ignored.
finish :: (client: *Client, id: int) -> Pending, bool {
    for client.pending {
        if it.id != id continue;
        result := it;
        array_ordered_remove_by_index(*client.pending, it_index);
        return result, true;
    }

    return .{}, false;
}

free_pending :: (pending: *Pending) {
    free(pending.method);
    free(pending.uri);
    pending.* = .{};
}

cancel_document :: (client: *Client, uri: string) {
    i := client.pending.count - 1;
    while i >= 0 {
        if client.pending[i].uri == uri {
            send(client, "$/cancelRequest", tprint("{\"id\":%}", client.pending[i].id));
            free_pending(*client.pending[i]);
            array_ordered_remove_by_index(*client.pending, i);
        }

        i -= 1;
    }
}

start :: (client: *Client, executable: string, root: string) -> bool {
    client.running = create_process(*client.process, executable, working_directory = root, capture_and_return_output = true);
    if !client.running return false;

    init(*client.mutex);

    if !thread_init(*client.writer, write_packets) {
        destroy(*client.mutex);
        kill_process(*client.process);
        deinit(*client.process);
        client.running = false;

        return false;
    }

    client.writer.data = client;
    thread_start(*client.writer);
    client.started_at = to_milliseconds(current_time_monotonic());
    request(client, "initialize", tprint("{\"processId\":null,\"rootUri\":%,\"capabilities\":{\"general\":{\"positionEncodings\":[\"utf-16\"]},\"textDocument\":{\"completion\":{\"completionItem\":{\"snippetSupport\":false}},\"publishDiagnostics\":{\"versionSupport\":true}}},\"initializationOptions\":{\"trusted\":false}}", quote(uri(root))));

    return true;
}

// Bounded work per frame; drain stderr too so logging cannot fill the server's pipe.
poll :: (client: *Client) {
    if !client.running || client.failed return;

    lock(*client.mutex);
    if client.writer_failed client.failed = true;

    unlock(*client.mutex);
    if !client.ready && to_milliseconds(current_time_monotonic()) - client.started_at > 15000 client.failed = true;

    output: [16384] u8;
    errors: [4096] u8;

    for 0..7 {
        ok, count, error_count := read_from_process(*client.process, output, errors, 0);
        if !ok {
            client.failed = true;
            return;
        }

        for 0..count - 1 array_add(*client.input, output[it]);

        if client.input.count > MAX_MESSAGE + 8192 {
            client.failed = true;
            return;
        }

        if count == 0 && error_count == 0 break;
    }

    if client.process.output.eof client.failed = true;
}

stop :: (client: *Client) {
    if client.running {
        lock(*client.mutex);
        client.stopping = true;
        unlock(*client.mutex);
        kill_process(*client.process);
        thread_is_done(*client.writer, -1);
        thread_deinit(*client.writer);
        destroy(*client.mutex);
        deinit(*client.process);
    }

    for client.outgoing free(it);

    array_free(client.outgoing);
    for *client.pending free_pending(it);

    array_free(client.pending);
    array_free(client.input);
    client.* = .{};
}

get :: (value: *Json_Value, name: string) -> *Json_Value {
    if !value return null;
    return json_get(value, name);
}

number :: (value: *Json_Value) -> int {
    if !value return 0;
    n, _ := json_as_s64(value);
    return n;
}

text :: (value: *Json_Value) -> string {
    if !value || value.kind != .STRING return "";
    return value.text;
}

read_position :: (value: *Json_Value) -> Position {
    return .{ line = number(get(value, "line")), character = number(get(value, "character")) };
}

// One minimal replacement, widened to complete UTF-8 characters and CRLF endings.
change :: (old: string, text: string) -> string {
    start := 0;
    while start < min(text.count, old.count) && text[start] == old[start] start += 1;
    while split_boundary(text, start) || split_boundary(old, start) start -= 1;

    old_end := old.count;
    new_end := text.count;

    while old_end > start && new_end > start && old[old_end - 1] == text[new_end - 1] {
        old_end -= 1;
        new_end -= 1;
    }

    while split_boundary(old, old_end) || split_boundary(text, new_end) {
        old_end += 1;
        new_end += 1;
    }

    a := position(old, start);
    b := position(old, old_end);

    return tprint("{\"range\":{\"start\":{\"line\":%,\"character\":%},\"end\":{\"line\":%,\"character\":%}},\"text\":%}", a.line, a.character, b.line, b.character, quote(slice(text, start, new_end - start)));
}

split_boundary :: (text: string, at: int) -> bool {
    if at <= 0 || at >= text.count return false;
    return (text[at] & 0xc0) == 0x80 || (text[at - 1] == 13 && text[at] == 10);
}
