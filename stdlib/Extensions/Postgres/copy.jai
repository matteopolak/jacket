// Bulk loading: `COPY ... FROM STDIN (FORMAT BINARY)` fed from an array of structs.

// Sends `rows` through `sql`, a `copy <table> (<columns>) from stdin (format binary)` statement,
// and returns how many rows the server stored. Each row sends the members named in `columns`, in
// that order, or every member when `columns` is empty. The columns' types must match the members'
// binary forms exactly (an `s32` for `int4`, an `s64` for `int8`): COPY casts nothing. A `Json(T)`
// member goes as `jsonb` unless it has the `@pg_json` note, which sends it as `json`.
//
// - c: the connection to run on.
// - sql: the `copy ... from stdin (format binary)` statement.
// - rows: the structs to send, one per row.
// - columns: the members to send, in column order. Empty sends every member.
//
// ```jai,ignore
// stored, error := Pg.copy_in(db, "copy users (email, score) from stdin (format binary)", users, .["email", "score"]);
// ```
//
// Not run: it needs a PostgreSQL server.
copy_in :: (c: *Connection, sql: string, rows: [] $Row, columns: [] string = .[]) -> s64, Pg_Error {
    #assert type_info(Row).type == .STRUCT "copy_in takes an array of structs";

    state: Copy_State;
    state.data = cast(*u8) rows.data;
    state.count = rows.count;
    state.stride = size_of(Row);
    state.put_member = #bake_arguments copy_put_erased(Row = Row);

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

// `copy_in` on a connection from the pool.
//
// - pool: the pool to take a connection from. It is returned afterwards.
// - sql: the `copy ... from stdin (format binary)` statement.
// - rows: the structs to send, one per row.
// - columns: the members to send, in column order. Empty sends every member.
copy_in :: (pool: *Pool, sql: string, rows: [] $Row, columns: [] string = .[]) -> s64, Pg_Error {
    c, error := acquire(pool);
    if error.kind != .NONE return 0, error;

    defer release(pool, c);

    stored:, error = copy_in(c, sql, rows, columns);

    return stored, error;
}

// Gets the data of a `COPY ... TO STDOUT` in the pieces the server sends it in. A piece is one row
// in the text and CSV formats, and any slice of the stream in binary; it is valid during the call.
Copy_Out_Proc :: #type (data: [] u8, user_data: *void);

// Runs a `COPY <table or (query)> TO STDOUT` statement and calls `proc` with each piece of its
// data, as it arrives, so a table of any size streams through. Returns how many rows the server
// sent. Any format works: text, CSV or binary.
//
// - c: the connection to run on.
// - sql: the `copy ... to stdout` statement, in any format.
// - proc: called with each piece of data as it arrives.
// - user_data: handed to `proc` as it is.
//
// ```jai,ignore
// count, error := Pg.copy_out(db, "copy (select email from users) to stdout (format csv)", write_chunk, file);
// ```
//
// Not run: it needs a PostgreSQL server.
copy_out :: (c: *Connection, sql: string, proc: Copy_Out_Proc, user_data: *void = null) -> s64, Pg_Error {
    state: Copy_Stream;
    state.proc = proc;
    state.user_data = user_data;
    state.hooks.user_data = *state;
    state.hooks.on_copy = stream_copy_info;
    state.hooks.copy_out = stream_copy_data;
    state.hooks.on_result = stream_copy_count;

    error := run_script(c, sql, *state.hooks);
    if error.kind != .NONE return 0, error;

    if !state.started return 0, make_error(.USAGE, "copy_out needs a COPY ... TO STDOUT statement");
    if state.incoming return 0, make_error(.USAGE, "copy_out needs COPY ... TO STDOUT, not FROM STDIN");

    return state.sent, .{};
}

// `copy_out` to a callback, on a connection from the pool.
//
// - pool: the pool to take a connection from. It is returned afterwards.
// - sql: the `copy ... to stdout` statement, in any format.
// - proc: called with each piece of data as it arrives.
// - user_data: handed to `proc` as it is.
copy_out :: (pool: *Pool, sql: string, proc: Copy_Out_Proc, user_data: *void = null) -> s64, Pg_Error {
    c, error := acquire(pool);
    if error.kind != .NONE return 0, error;

    defer release(pool, c);

    sent:, error = copy_out(c, sql, proc, user_data);

    return sent, error;
}

// The rows of a binary COPY. Strings, byte slices and what arrays and numerics hold belong to
// `result`: free them all with `free_copy_rows`.
Copy_Rows :: struct(Row: Type) {
    rows: [] Row;
    result: Result;
}

// Frees the rows of a binary COPY and everything they point to.
//
// - copied: the rows to free. It is empty afterwards.
free_copy_rows :: (copied: *Copy_Rows($Row)) {
    array_free(copied.rows);
    free_result(*copied.result);
    copied.* = .{};
}

// Runs a `copy <table or (query)> to stdout (format binary)` statement and decodes the rows into
// structs: the inverse of `copy_in`. Each row has the members named in `columns`, in the
// statement's column order, or every member when `columns` is empty. A member needs the type
// `copy_in` would send for the column (an `s32` for `int4`, a `Nullable` for a column that can be
// NULL, `@pg_json` for `json`). The whole result is read into memory.
//
// - c: the connection to run on.
// - sql: the `copy ... to stdout (format binary)` statement.
// - $Row: the struct each row is decoded into.
// - columns: the members to fill, in the statement's column order. Empty fills every member.
//
// ```jai,ignore
// copied, error := Pg.copy_out(db, "copy users (email, score) to stdout (format binary)", User_Row, .["email", "score"]);
// for copied.rows print("% %\n", it.email, it.score);
// Pg.free_copy_rows(*copied);
// ```
//
// Not run: it needs a PostgreSQL server.
copy_out :: (c: *Connection, sql: string, $Row: Type, columns: [] string = .[]) -> Copy_Rows(Row), Pg_Error {
    #assert type_info(Row).type == .STRUCT "copy_out takes a struct";

    copied: Copy_Rows(Row);
    copied.result.allocator = context.allocator;
    copied.result.json_options = c.json_options;

    state: Copy_Collect;
    state.data = *copied.result.data;
    state.hooks.user_data = *state;
    state.hooks.on_copy = collect_copy_info;
    state.hooks.copy_out = collect_copy_data;
    state.hooks.on_result = collect_copy_count;

    fields, fields_error := copy_read_fields(type_info(Row), columns);
    if fields_error.kind != .NONE return .{}, fields_error;

    error := run_script(c, sql, *state.hooks);

    if error.kind == .NONE && state.problem error = make_error(.USAGE, "%", state.problem);
    if error.kind == .NONE && !state.started error = make_error(.USAGE, "copy_out needs a COPY ... TO STDOUT statement");

    if error.kind == .NONE {
        error = decode_copy_rows(*copied, fields, state.stored);
    }

    if error.kind != .NONE {
        free_copy_rows(*copied);
        return .{}, error;
    }

    return copied, .{};
}

// `copy_out` into structs, on a connection from the pool.
//
// - pool: the pool to take a connection from. It is returned afterwards.
// - sql: the `copy ... to stdout (format binary)` statement.
// - $Row: the struct each row is decoded into.
// - columns: the members to fill, in the statement's column order. Empty fills every member.
copy_out :: (pool: *Pool, sql: string, $Row: Type, columns: [] string = .[]) -> Copy_Rows(Row), Pg_Error {
    c, error := acquire(pool);
    if error.kind != .NONE return .{}, error;

    defer release(pool, c);

    copied:, error = copy_out(c, sql, Row, columns);

    return copied, error;
}

#scope_module

// The pieces of `copy_out` to a callback.
Copy_Stream :: struct {
    hooks: Script_Hooks;
    proc: Copy_Out_Proc;
    user_data: *void;
    started: bool;
    incoming: bool;
    sent: s64;
}

stream_copy_info :: (hooks: *Script_Hooks, copy: *Copy_Info) {
    state := cast(*Copy_Stream) hooks.user_data;
    state.started = true;
    state.incoming = copy.incoming;
}

stream_copy_data :: (hooks: *Script_Hooks, copy: *Copy_Info, data: [] u8) {
    state := cast(*Copy_Stream) hooks.user_data;
    state.proc(data, state.user_data);
}

stream_copy_count :: (hooks: *Script_Hooks, result: *Result) {
    state := cast(*Copy_Stream) hooks.user_data;
    state.sent += result.rows_affected;
}

// The whole of a binary COPY, collected to be decoded.
Copy_Collect :: struct {
    hooks: Script_Hooks;
    data: *[..] u8;
    started: bool;
    stored: s64;
    problem: string;
}

collect_copy_info :: (hooks: *Script_Hooks, copy: *Copy_Info) {
    state := cast(*Copy_Collect) hooks.user_data;
    state.started = true;

    if copy.incoming state.problem = "copy_out needs COPY ... TO STDOUT, not FROM STDIN";
    else if copy.format != .BINARY state.problem = "copy_out into rows needs a statement with (format binary)";
}

collect_copy_data :: (hooks: *Script_Hooks, copy: *Copy_Info, data: [] u8) {
    state := cast(*Copy_Collect) hooks.user_data;
    if !state.problem put_bytes(state.data, data);
}

collect_copy_count :: (hooks: *Script_Hooks, result: *Result) {
    state := cast(*Copy_Collect) hooks.user_data;
    state.stored += result.rows_affected;
}

Copy_Read_Field :: struct {
    member: s64; // index in the struct's members
    oid: u32; // what `decode` is told the column is
}

// The members to fill, in column order.
copy_read_fields :: (info: *Type_Info, columns: [] string) -> [] Copy_Read_Field, Pg_Error {
    struct_info := cast(*Type_Info_Struct) info;
    fields: [..] Copy_Read_Field;
    fields.allocator = temp;

    if !columns.count {
        for *struct_info.members {
            if it.flags & .CONSTANT continue;
            array_add(*fields, .{ it_index, copy_read_oid(it.type, has_note(it, "pg_json")) });
        }

        return fields, .{};
    }

    for name: columns {
        found := -1;
        for *struct_info.members if it.name == name && !(it.flags & .CONSTANT) found = it_index;

        if found < 0 return .[], make_error(.USAGE, "% has no member \"%\" to read a column into", type_to_string(info,, temp), name);

        array_add(*fields, .{ found, copy_read_oid(struct_info.members[found].type, has_note(*struct_info.members[found], "pg_json")) });
    }

    return fields, .{};
}

// Binary values rarely depend on the OID. A string needs a text type, and a json document is told
// from a jsonb one by the version byte jsonb starts with.
copy_read_oid :: (info: *Type_Info, json_text: bool) -> u32 {
    inner := info;

    if is_nullable_info(info) {
        for (cast(*Type_Info_Struct) info).members if it.name == "value" inner = it.type;
    }

    if is_json_info(inner) || inner == type_info(Json_Raw) return xx ifx json_text then Oid.JSON else Oid.JSONB;

    return xx Oid.TEXT;
}

decode_copy_rows :: (copied: *Copy_Rows($Row), fields: [] Copy_Read_Field, expected: s64) -> Pg_Error {
    r := Cursor.{ data = copied.result.data };
    signature := get_bytes(*r, COPY_SIGNATURE.count);

    if r.bad || cast(string) signature != COPY_SIGNATURE return make_error(.PROTOCOL, "the COPY data does not start with the binary COPY header");

    get_u32(*r); // flags
    extension := get_u32(*r);

    get_bytes(*r, cast(s64) extension);

    rows: [..] Row;

    while !r.bad {
        count := get_s16(*r);

        if r.bad break;
        if count == -1 break;

        if count != fields.count {
            array_free(rows);
            return make_error(.USAGE, "the COPY sends % columns per row, and % were asked for", count, fields.count);
        }

        item: Row;

        for field, field_index: fields {
            length := get_s32(*r);
            bytes: [] u8;

            if length >= 0 bytes = get_bytes(*r, cast(s64) length);
            if r.bad break;

            why := copy_member(*item, field.member, bytes, length < 0, field.oid, *copied.result);

            if why {
                array_free(rows);
                return make_error(.DECODE, "row %, column %: %", rows.count + 1, field_index + 1, why);
            }
        }

        array_add(*rows, item);
    }

    if r.bad {
        array_free(rows);
        return make_error(.PROTOCOL, "the COPY data ends in the middle of a row");
    }

    if rows.count != expected {
        array_free(rows);
        return make_error(.PROTOCOL, "the COPY sent % rows, and its tag says %", rows.count, expected);
    }

    copied.rows = rows;

    return .{};
}

// Decodes one column's bytes into the member at `index`. What decoding allocates belongs to the
// result, as with `get`.
copy_member :: (row: *$Row, index: s64, bytes: [] u8, is_null: bool, oid: u32, result: *Result) -> string {
    #insert #run copy_member_code(Row);

    return "no such member";
}

copy_member_code :: (Row: Type) -> string {
    builder: String_Builder;
    builder.allocator = temp;

    info := cast(*Type_Info_Struct) type_info(Row);

    for *info.members {
        if it.flags & .CONSTANT continue;

        if is_nullable_info(it.type) {
            print_to_builder(*builder, "if index == % {\n    if is_null {\n        row.%.null = true;\n        return \"\";\n    }\n\n    row.%.null = false;\n    return copy_decode(*row.%.value, bytes, oid, result);\n}\n", it_index, it.name, it.name, it.name);
        } else {
            print_to_builder(*builder, "if index == % {\n    if is_null return \"the column is NULL: read it as a Nullable\";\n    return copy_decode(*row.%, bytes, oid, result);\n}\n", it_index, it.name);
        }
    }

    return builder_to_string(*builder);
}

copy_decode :: (out: *$T, bytes: [] u8, oid: u32, result: *Result) -> string {
    why: string;

    {
        push_allocator(.{ result_allocator, result });

        layers := Json_Layers.{ connection = result.json_options, borrow = true };

        why = decode(bytes, .BINARY, oid, out, *layers);
    }

    return why;
}

Copy_State :: struct {
    hooks: Script_Hooks;

    data: *u8;
    count: s64;
    stride: s64;
    fields: [..] Copy_Field;
    // Writes one member of a row of the struct being sent (`copy_put_member`, for that struct).
    put_member: (out: *[..] u8, row: *u8, field: Copy_Field) -> string;

    next: s64; // the next row to encode
    started: bool; // the header is sent
    buffer: [..] u8;
    problem: string;
    stored: s64;
}

Copy_Field :: struct {
    member: s64; // index in the struct's members
    name: string;
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
            array_add(*state.fields, .{ it_index, it.name, has_note(it, "pg_json") });
        }

        return .{};
    }

    for name: columns {
        found := -1;
        for *struct_info.members if it.name == name && !(it.flags & .CONSTANT) found = it_index;

        if found < 0 return make_error(.USAGE, "% has no member \"%\" to copy", type_to_string(info,, temp), name);

        array_add(*state.fields, .{ found, name, has_note(*struct_info.members[found], "pg_json") });
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
            why := state.put_member(*state.buffer, row, it);

            if why {
                // copy_in consumes and copies this error in the same fiber before its scratch storage is reset.
                state.problem = tprint("row %, member %: %", state.next + 1, it.name, why);
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

// Writes member `field.member` of `row`: its length (or -1 for null), then its binary form.
copy_put_erased :: ($Row: Type, out: *[..] u8, row: *u8, field: Copy_Field) -> string {
    return copy_put_member(out, cast(*Row) row, field);
}

// The body is written for `Row` when the program compiles: one case per member, each encoding its
// own type, so a row costs no type walk.
copy_put_member :: (out: *[..] u8, row: *$Row, field: Copy_Field) -> string {
    #insert #run copy_put_member_code(Row);

    return "";
}

copy_put_member_code :: (Row: Type) -> string {
    builder: String_Builder;
    builder.allocator = temp;

    append(*builder, "if field.member == {\n");

    for (cast(*Type_Info_Struct) type_info(Row)).members {
        if it.flags & .CONSTANT continue;

        if !it.name.count {
            compiler_report(tprint("Postgres: copy_in cannot send the unnamed member % of %", it_index, Row));
            continue;
        }

        print_to_builder(*builder, "    case %; return put_copy_value(out, *row.%, field.json_text);\n", it_index, it.name);
    }

    append(*builder, "}\n");

    return builder_to_string(*builder);
}

// One field: its length (or -1 for null), then its binary form. Every type is written by code
// made for it, so a field costs no walk of type information.
put_copy_value :: (out: *[..] u8, value: *$T, json_text: bool) -> string {
    #if #run is_nullable(T) {
        if value.null {
            put_u32(out, COPY_NULL);
            return "";
        }

        return put_copy_value(out, *value.value, json_text);
    } else #if type_info(T).type == .BOOL {
        put_u32(out, 1);
        put_u8(out, xx value.*);
    } else #if type_info(T).type == .INTEGER {
        SIGNED :: #run (cast(*Type_Info_Integer) type_info(T)).signed;
        v := cast,no_check(s64) value.*;

        #if size_of(T) <= 1 || (size_of(T) == 2 && SIGNED) {
            put_u32(out, 2);
            put_u16(out, cast,no_check(u16) v);
        } else #if size_of(T) <= 2 || (size_of(T) == 4 && SIGNED) {
            put_u32(out, 4);
            put_u32(out, cast,no_check(u32) v);
        } else {
            #if size_of(T) == 8 && !SIGNED {
                if value.* > xx S64_MAX return "a u64 parameter above S64_MAX does not fit in int8";
            }

            put_u32(out, 8);
            put_u64(out, cast,no_check(u64) v);
        }
    } else #if type_info(T).type == .FLOAT {
        #if size_of(T) == 4 {
            put_u32(out, 4);
            put_u32(out, (cast(*u32) value).*);
        } else {
            put_u32(out, 8);
            put_u64(out, (cast(*u64) value).*);
        }
    } else #if type_info(T).type == .STRING {
        if contains_zero(value.*) return "a string parameter holds a zero byte, which Postgres text cannot store";

        put_u32(out, cast(u32) value.count);
        put_bytes(out, value.*);
    } else #if type_info(T).type == .ENUM {
        label, found := enum_label_of(value.*);
        if !found return tprint("% holds a value that is none of its members", T);

        put_u32(out, cast(u32) label.count);
        put_bytes(out, label);
    } else #if T == Uuid {
        put_u32(out, 16);
        put_bytes(out, cast([] u8) value.bytes);
    } else #if T == Timestamp || T == Local_Timestamp || T == Time {
        put_u32(out, 8);
        put_u64(out, cast,no_check(u64) (cast(*s64) value).*);
    } else #if T == Date {
        put_u32(out, 4);
        put_u32(out, cast,no_check(u32) (cast(*s32) value).*);
    } else #if T == Interval {
        put_u32(out, 16);
        put_u64(out, cast,no_check(u64) value.microseconds);
        put_u32(out, cast,no_check(u32) value.days);
        put_u32(out, cast,no_check(u32) value.months);
    } else #if T == Numeric {
        bytes, why := numeric_binary(value.text);
        if why return why;

        put_u32(out, cast(u32) bytes.count);
        put_bytes(out, bytes);
    } else #if #run is_json_value(T) {
        text, why := json_text_of(value);
        if why return why;

        if json_text {
            put_u32(out, cast(u32) text.count);
        } else {
            // jsonb starts with its version byte.
            put_u32(out, cast(u32) text.count + 1);
            put_u8(out, 1);
        }

        put_bytes(out, text);
    } else #if #run is_bytes(T) {
        view: [] u8 = value.*;

        put_u32(out, cast(u32) view.count);
        put_bytes(out, view);
    } else #if #run is_enum_array(T) {
        return "COPY cannot send arrays of enums; insert them instead";
    } else #if type_info(T).type == .ARRAY {
        param, why := array_param_of(value.*);
        if why return why;

        put_u32(out, cast(u32) param.data.count);
        put_bytes(out, param.data);
    } else {
        return tprint("Postgres has no encoding for COPY members of type %", T);
    }

    return "";
}

// The label of an enum value as Postgres spells it, by a comparison per member written for `T`.
enum_label_of :: (value: $T) -> string, bool {
    #insert -> string {
        info := cast(*Type_Info_Enum) type_info(T);
        builder: String_Builder;
        builder.allocator = temp;

        for info.values {
            print_to_builder(*builder, "if cast(s64) value == % return %, true;\n", it, jai_string(enum_label(info.names[it_index]),, temp));
        }

        return builder_to_string(*builder);
    };

    return "", false;
}
