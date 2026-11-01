// The line protocol, independent of any socket: `parse_request` turns one line into a `Request`
// and `execute` runs it against a store and writes the reply lines.
//
//     GET key             ->  VALUE text | VALUE64 base64 | MISSING
//     SET key text        ->  OK                     (the text is the rest of the line)
//     SET64 key base64    ->  OK                     (for values with line breaks or bytes)
//     DEL key             ->  OK | MISSING
//     SCAN prefix         ->  ROW key text | ROW64 key base64, ..., END
//     COMPACT             ->  OK before after        (log bytes)
//     STATS               ->  STATS keys=N records=N log_bytes=N live_bytes=N
//     QUIT                ->  OK, then the server closes the connection
//     SHUTDOWN            ->  OK, then the server stops
//
// Any request that cannot be served is answered with `ERR message`. Verbs are case insensitive;
// SCAN with no prefix lists everything.

Verb :: enum {
    GET;
    SET;
    SET64;
    DEL;
    SCAN;
    COMPACT;
    STATS;
    QUIT;
    SHUTDOWN;
}

Request :: struct {
    verb: Verb;
    // The key, or the prefix for SCAN. Points into the line it was parsed from.
    key: string;
    // The value of SET, still Base64 for SET64.
    value: string;
}

// What the connection does after the reply.
Outcome :: enum {
    CONTINUE;
    CLOSE;
    SHUTDOWN;
}

// Parses one request line. The second result is empty on success and otherwise says what is wrong,
// in words fit for an `ERR` reply.
parse_request :: (line: string) -> Request, string {
    request: Request;
    _, word, rest := split_from_left(line, #char " ");

    verb, known := parse_verb(word);

    if !known return request, tprint("unknown command '%'", word);

    request.verb = verb;

    takes_key := verb == .GET || verb == .SET || verb == .SET64 || verb == .DEL;

    if verb == .SET || verb == .SET64 {
        found, key, value := split_from_left(rest, #char " ");

        // `SET key` stores the empty value.
        request.key = key;
        request.value = value;

        if !found request.value = "";
    } else if takes_key || verb == .SCAN {
        request.key = rest;
    } else if rest.count > 0 {
        return request, tprint("% takes no arguments", word);
    }

    if takes_key {
        if request.key.count == 0 return request, tprint("% needs a key", word);
        if contains(request.key, " ") return request, tprint("% takes one key", word);
        if !valid_key(request.key) return request, "a key is up to 1024 bytes with no spaces or control characters";
    }

    return request, "";
} @TempAllocates

// Runs `request` and appends the reply, one or more lines each ending in a line break, to `out`.
execute :: (store: *Store, request: Request, out: *String_Builder) -> Outcome {
    if request.verb == {
        case .GET;
            value, found := store_get(store, request.key);

            if found {
                append_value(out, "VALUE", "", value);
            } else {
                append(out, "MISSING\n");
            }

        case .SET;
            status := store_put(store, request.key, request.value);

            append_status(out, status);

        case .SET64;
            value, ok := base64_decode(request.value);

            defer free(value);

            if ok {
                append_status(out, store_put(store, request.key, value));
            } else {
                append(out, "ERR the value is not Base64\n");
            }

        case .DEL;
            status := store_delete(store, request.key);

            if status == .MISSING {
                append(out, "MISSING\n");
            } else {
                append_status(out, status);
            }

        case .SCAN;
            for store_scan(store, request.key) append_value(out, "ROW", it.key, it.value);

            append(out, "END\n");

        case .COMPACT;
            result := store_compact(store);

            if result.ok {
                print_to_builder(out, "OK % %\n", result.bytes_before, result.bytes_after);
            } else {
                append(out, "ERR compaction failed; the log is unchanged\n");
            }

        case .STATS;
            print_to_builder(out, "STATS keys=% records=% log_bytes=% live_bytes=%\n", store.index.count, store.log_records, store.log_size, store.live_bytes);

        case .QUIT;
            append(out, "OK\n");

            return .CLOSE;

        case .SHUTDOWN;
            append(out, "OK\n");

            return .SHUTDOWN;
    }

    return .CONTINUE;
}

#scope_file

parse_verb :: (word: string) -> Verb, bool {
    names := string.["GET", "SET", "SET64", "DEL", "SCAN", "COMPACT", "STATS", "QUIT", "SHUTDOWN"];

    for names {
        if equal_nocase(word, it) return cast(Verb) it_index, true;
    }

    return .GET, false;
}

append_status :: (out: *String_Builder, status: Write_Status) {
    if status == {
        case .OK; append(out, "OK\n");
        case .INVALID; append(out, "ERR the key or the value is not allowed\n");
        case; append(out, "ERR the write failed\n");
    }
}

// One `<tag> [key] value` line, in the Base64 form when the value would break the framing.
append_value :: (out: *String_Builder, tag: string, key: string, value: string) {
    plain := true;

    for value {
        if it == #char "\n" || it == #char "\r" plain = false;
    }

    encoded: string;

    if !plain {
        encoded = base64_encode(value);
    }

    defer free(encoded);

    append(out, tag);

    if !plain append(out, "64");
    if key.count > 0 print_to_builder(out, " %", key);

    print_to_builder(out, " %\n", ifx plain then value else encoded);
}
