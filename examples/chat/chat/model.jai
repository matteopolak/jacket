MAX_LINE :: 2048;
MAX_ROOMS :: 32;
MAX_USERS :: 256;

Request :: struct {
    verb: string;
    target: string;
    text: string;
}

valid_name :: (name: string) -> bool {
    if name.count < 1 || name.count > 24 return false;

    for name {
        if !(it >= #char "a" && it <= #char "z") && !(it >= #char "A" && it <= #char "Z") && !(it >= #char "0" && it <= #char "9") && it != #char "_" && it != #char "-" return false;
    }

    return true;
}

valid_text :: (text: string) -> bool {
    if text.count < 1 || text.count > 1024 return false;
    for text if it < 32 || it == 127 return false;
    return true;
}

// Views into line; the caller keeps it alive until the request is executed.
parse_request :: (line: string) -> Request, bool {
    request: Request;
    if line.count > MAX_LINE return request, false;

    found, verb, rest := split_from_left(line, #char " ");

    request.verb = verb;
    if !found request.verb = line;
    if request.verb == "LEAVE" || request.verb == "HISTORY" || request.verb == "QUIT" || request.verb == "PING" return request, !found;

    if request.verb == "NICK" || request.verb == "JOIN" {
        request.target = rest;
        return request, found && valid_name(rest);
    }

    if request.verb == "EMAIL" {
        request.target = rest;
        return request, valid_email(rest);
    }

    if request.verb == "SAY" {
        request.text = rest;
        return request, found && valid_text(rest);
    }

    if request.verb == "DM" {
        separated, target, text := split_from_left(rest, #char " ");
        request.target = target;
        request.text = text;
        return request, separated && valid_name(target) && valid_text(text);
    }

    return request, false;
}

valid_email :: (address: string) -> bool {
    if address.count < 3 || address.count > 128 return false;

    for address {
        if !(it >= #char "a" && it <= #char "z") && !(it >= #char "A" && it <= #char "Z") && !(it >= #char "0" && it <= #char "9") && it != #char "_" && it != #char "-" && it != #char "." && it != #char "@" && it != #char "+" return false;
    }

    return contains(address, "@");
}

Limiter :: struct {
    tokens: float64 = 20;
    last: float64;
}

allow :: (limiter: *Limiter, now: float64, rate: float64) -> bool {
    limiter.tokens = min(cast(float64) 20, limiter.tokens + max(cast(float64) 0, now - limiter.last) * rate);
    limiter.last = now;
    if limiter.tokens < 1 return false;

    limiter.tokens -= 1;

    return true;
}

Message :: struct {
    nick: string;
    text: string;
}

Room :: struct {
    name: string;
    history: [..] Message;
}

Model :: struct {
    rooms: [..] *Room;
    history_limit: s64 = 20;
    log: File;
    opened: bool;
    failed: bool;
}

room_find :: (model: *Model, name: string, create := true) -> *Room {
    for model.rooms if it.name == name return it;
    if !create || model.rooms.count >= MAX_ROOMS return null;

    room := New(Room);
    room.name = copy_string(name);
    array_add(*model.rooms, room);

    return room;
}

remember :: (model: *Model, room: *Room, nick: string, text: string) {
    if room.history.count >= model.history_limit {
        oldest := room.history[0];

        free(oldest.nick);
        free(oldest.text);
        for 1..room.history.count - 1 room.history[it - 1] = room.history[it];

        room.history.count -= 1;
    }

    array_add(*room.history, .{ copy_string(nick), copy_string(text) });
}

record_message :: (model: *Model, room: *Room, nick: string, text: string) -> bool {
    body := tprint("%\t%\t%", room.name, nick, text);
    digest := md5(body);

    defer free(digest);
    record := tprint("%\t%\n", body, digest);
    if model.failed || !file_write(*model.log, record) || !file_flush(model.log) {
        model.failed = true;
        return false;
    }

    remember(model, room, nick, text);

    return true;
}

// Only an unterminated final record is recoverable. A corrupt complete line fails closed.
model_open :: (model: *Model, path: string) -> bool {
    data, exists := read_entire_file(path);
    if !exists {
        probe, ok := file_open(path, for_writing = true, keep_existing_content = true);
        if !ok return false;
        file_close(*probe);
    }

    defer free(data);
    at := 0;
    while at < data.count {
        rest := slice(data, at, data.count - at);
        newline := find_index_from_left(rest, #char "\n");

        if newline < 0 {
            recovery := tprint("%.recover", path);
            if !write_entire_file(recovery, slice(data, 0, at)) || !atomic_file_replace_or_just_rename_if_destination_file_does_not_exist(recovery, path) return false;
            break;
        }

        line := slice(rest, 0, newline);
        fields := split(line, "\t",, temp);

        if fields.count != 4 || !valid_name(fields[0]) || !valid_name(fields[1]) || !valid_text(fields[2]) return false;

        body := slice(line, 0, line.count - fields[3].count - 1);
        digest := md5(body);
        correct := digest == fields[3];

        free(digest);
        if !correct return false;

        room := room_find(model, fields[0]);
        if !room return false;

        remember(model, room, fields[1], fields[2]);
        at += newline + 1;
    }

    model.log, model.opened = file_open(path, for_writing = true, keep_existing_content = true);

    return model.opened && file_seek(model.log, 0, .END);
}

model_free :: (model: *Model) {
    if model.opened file_close(*model.log);

    for model.rooms {
        for message: it.history {
            free(message.nick);
            free(message.text);
        }

        array_free(it.history);
        free(it.name);
        free(it);
    }

    array_free(model.rooms);
    model.* = .{};
}

// Mentions have token boundaries: @al must not match @alice.
mentions :: (text: string, nick: string) -> bool {
    needle := tprint("@%", nick);
    for start: 0..text.count - needle.count {
        if start > 0 && text[start - 1] != #char " " continue;
        if slice(text, start, needle.count) != needle continue;
        end := start + needle.count;
        if end == text.count || !valid_name(slice(text, end, 1)) return true;
    }

    return false;
}
