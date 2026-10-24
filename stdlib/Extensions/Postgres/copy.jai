// Bulk loading: `COPY ... FROM STDIN (FORMAT BINARY)` fed from an array of structs.

// Sends `rows` through `sql`, a `copy <table> (<columns>) from stdin (format binary)` statement,
// and returns how many rows the server stored. Each row sends the members named in `columns`, in
// that order, or every member when `columns` is empty. The columns' types must match the members'
// binary forms exactly (an `s32` for `int4`, an `s64` for `int8`): COPY casts nothing. A `Json(T)`
// member goes as `jsonb` unless it has the `@pg_json` note, which sends it as `json`.
//
//     stored, error := Pg.copy_in(db, "copy users (email, score) from stdin (format binary)", users, .["email", "score"]);
copy_in :: (c: *Connection, sql: string, rows: [] $Row, columns: [] string = .[]) -> s64, Pg_Error {
    #assert type_info(Row).type == .STRUCT "copy_in takes an array of structs";

    state: Copy_State;
    state.data = cast(*u8) rows.data;
    state.count = rows.count;
    state.stride = size_of(Row);

    error := copy_fields(*state, type_info(Row), columns);
    if error.kind != .NONE return 0, error;

    state.hooks.user_data = *state;
    state.hooks.copy_in = copy_next_chunk;
    state.hooks.on_result = copy_count;

    error = run_script(c, sql, *state.hooks);
    array_free(state.buffer);

    if error.kind != .NONE return 0, error;
    if state.problem return 0, make_error(.USAGE, "%", state.problem);

    return state.stored, .{};
}

copy_in :: (pool: *Pool, sql: string, rows: [] $Row, columns: [] string = .[]) -> s64, Pg_Error {
    c, error := acquire(pool);
    if error.kind != .NONE return 0, error;

    defer release(pool, c);

    stored:, error = copy_in(c, sql, rows, columns);

    return stored, error;
}

#scope_module

Copy_State :: struct {
    hooks: Script_Hooks;

    data: *u8;
    count: s64;
    stride: s64;
    fields: [..] Copy_Field;

    next: s64; // the next row to encode
    started: bool; // the header is sent
    buffer: [..] u8;
    problem: string;
    stored: s64;
}

Copy_Field :: struct {
    member: *Type_Info_Struct_Member;
    json_text: bool; // `@pg_json`: a json document goes as `json`, without jsonb's version byte
}

COPY_SIGNATURE :: "PGCOPY\n\xff\r\n\0";
COPY_NULL :: 0xFFFF_FFFF; // a field length of -1
COPY_END :: 0xFFFF; // a field count of -1 ends the data
COPY_CHUNK_BYTES :: 64 * 1024;

copy_fields :: (state: *Copy_State, info: *Type_Info, columns: [] string) -> Pg_Error {
    struct_info := cast(*Type_Info_Struct) info;

    state.fields.allocator = temp;

    if !columns.count {
        for *struct_info.members {
            if it.flags & .CONSTANT continue;
            array_add(*state.fields, .{ member = it, json_text = has_note(it, "pg_json") });
        }

        return .{};
    }

    for name: columns {
        found: *Type_Info_Struct_Member;
        for *struct_info.members if it.name == name && !(it.flags & .CONSTANT) found = it;

        if !found return make_error(.USAGE, "% has no member \"%\" to copy", type_to_string(info,, temp), name);

        array_add(*state.fields, .{ member = found, json_text = has_note(found, "pg_json") });
    }

    return .{};
}

// Encodes rows until about a chunk is waiting, then hands it to the connection.
copy_next_chunk :: (hooks: *Script_Hooks, copy: *Copy_Info) -> data: string, done: bool, failure: string {
    state := cast(*Copy_State) hooks.user_data;
    state.buffer.count = 0;

    if copy.format != .BINARY {
        state.problem = "copy_in needs a statement with (format binary)";
        return "", false, state.problem;
    }

    if !state.started {
        put_bytes(*state.buffer, COPY_SIGNATURE);
        put_u32(*state.buffer, 0); // flags
        put_u32(*state.buffer, 0); // header extension length
        state.started = true;
    }

    while state.next < state.count && state.buffer.count < COPY_CHUNK_BYTES {
        row := state.data + state.next * state.stride;

        put_u16(*state.buffer, cast(u16) state.fields.count);

        for state.fields {
            why := put_copy_value(*state.buffer, Any.{ it.member.type, row + it.member.offset_in_bytes }, it.json_text);

            if why {
                state.problem = tprint("row %, member %: %", state.next + 1, it.member.name, why);
                return "", false, state.problem;
            }
        }

        state.next += 1;
    }

    done := state.next >= state.count;
    if done put_u16(*state.buffer, COPY_END);

    return cast(string) state.buffer, done, "";
}

copy_count :: (hooks: *Script_Hooks, result: *Result) {
    state := cast(*Copy_State) hooks.user_data;
    state.stored += result.rows_affected;
}

// One field: its length (or -1 for null), then its binary form.
put_copy_value :: (out: *[..] u8, value: Any, json_text: bool) -> string {
    info := value.type;
    data := value.value_pointer;

    if is_nullable_info(info) {
        s := cast(*Type_Info_Struct) info;
        null_member, inner_member: *Type_Info_Struct_Member;

        for *s.members {
            if it.name == "null" null_member = it;
            if it.name == "value" inner_member = it;
        }

        if (cast(*bool) (data + null_member.offset_in_bytes)).* {
            put_u32(out, COPY_NULL);
            return "";
        }

        return put_copy_value(out, Any.{ inner_member.type, data + inner_member.offset_in_bytes }, json_text);
    }

    bytes: string;

    if info.type == .ENUM {
        label, found := enum_value_label(value);
        if !found return tprint("% holds a value that is none of its members", type_to_string(info,, temp));

        bytes = label;
    } else if info.type == .ARRAY && (cast(*Type_Info_Array) info).element_type == type_info(u8) {
        bytes = cast(string) any_array_view(value);
    } else if info.type == .ARRAY {
        if (cast(*Type_Info_Array) info).element_type.type == .ENUM return "COPY cannot send arrays of enums; insert them instead";

        param, why := encode_array(value);
        if why return why;

        bytes = param.data;
    } else if info == type_info(Numeric) {
        why: string;
        bytes, why = numeric_binary((cast(*Numeric) data).text);
        if why return why;
    } else if json_text && (is_json_info(info) || info == type_info(Json_Raw)) {
        text, why := json_parameter_text(value);
        if why return why;

        bytes = text;
    } else {
        param, why := encode_binary(value);
        if why return why;

        bytes = param.data;
    }

    put_u32(out, cast(u32) bytes.count);
    put_bytes(out, bytes);

    return "";
}
