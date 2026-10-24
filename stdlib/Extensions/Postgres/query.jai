// Runtime queries: SQL known only at runtime, results read by column. The typed, compile-time
// checked queries (`query`, `query_one`, `execute`, ...) are built on the same messages.

Column :: struct {
    name: string;
    table_oid: u32; // the table the column comes from; 0 for computed columns
    column_number: s16; // its attribute number in that table; 0 for computed columns
    type_oid: u32; // see `Oid`
    type_size: s16; // pg_type.typlen: negative for variable-length types
    type_modifier: s32; // pg_attribute.atttypmod: varchar(n), numeric(p, s), ...
    format: Format;
}

// The rows of one statement. Strings and byte slices that `get` returns point into the result,
// and arrays and numerics it builds belong to it: all stay valid until `free_result`.
Result :: struct {
    columns: [] Column;
    row_count: s64;
    // The command tag ("SELECT 3", "INSERT 0 1", "UPDATE 7", "CREATE TABLE") and the row count
    // in it (0 when it has none).
    command: string;
    rows_affected: s64;
    // The statement returns rows (it sent a row description), even if there were none or it has
    // no columns: `select` returns one row of zero columns.
    returns_rows: bool;
    // More rows of the same statement follow (`Script_Hooks.chunk_rows`); `command` is not set yet.
    partial: bool;

    values: [..] Value_Span; // row_count * columns.count of them, row by row
    data: [..] u8;
    json_options: Json_Options; // for `Json(T)` columns; the connection's to begin with
    allocations: [..] *void;
    allocator: Allocator; // context.allocator when the query ran: it owns everything above
}

Value_Span :: struct {
    offset: s64;
    length: s64; // -1 for NULL
}

// Runs one statement with parameters `$1`, `$2`, ... taken from `args`, and returns its rows in
// text format. Arguments may be integers, floats, bools, strings, `[] u8` (bytea), `Uuid`,
// `Timestamp`, `Local_Timestamp`, `Date`, `Time`, `Interval`, `Numeric`, `Nullable` of any of
// those, and one-dimensional arrays of them.
query_dynamic :: (c: *Connection, sql: string, args: ..Any) -> Result, Pg_Error {
    values := args; // `sql` is not a print format: keep it and `args` out of one call
    result, error := run_extended(c, sql, values, .TEXT);
    return result, error;
}

// `query_dynamic` with binary results: smaller and faster to decode, but every column's type must
// be one `get` can decode in binary (see `Oid`).
query_binary :: (c: *Connection, sql: string, args: ..Any) -> Result, Pg_Error {
    values := args; // `sql` is not a print format: keep it and `args` out of one call
    result, error := run_extended(c, sql, values, .BINARY);
    return result, error;
}

// What the server makes of a statement without running it: the types it infers for `$1`, `$2`,
// ... and the columns it would return (none for a statement that returns no rows).
Statement_Info :: struct {
    parameter_types: [] u32;
    columns: [] Column;
}

// Parses `sql` as the unnamed prepared statement and describes it (Parse, Describe, Sync). Free
// the result with `free_statement_info`.
describe_statement :: (c: *Connection, sql: string) -> Statement_Info, Pg_Error {
    error := begin_request(c, sql);
    if error.kind != .NONE return .{}, error;

    defer c.busy = false;

    begin_message(c, #char "P");
    put_cstring(*c.out, "");
    put_cstring(*c.out, sql);
    put_u16(*c.out, 0);
    end_message(c);

    begin_message(c, #char "D");
    put_u8(*c.out, #char "S");
    put_cstring(*c.out, "");
    end_message(c);

    begin_message(c, #char "S");
    end_message(c);

    error = send(c);
    if error.kind != .NONE return .{}, error;

    info: Statement_Info;
    rows: Result;
    failure: Pg_Error;

    while true {
        type, body, read_error := read_message(c);
        if read_error.kind != .NONE {
            free_result(*rows);
            free_statement_info(*info);
            recover_interrupted(c, read_error);

            return .{}, error_before_close(*failure, read_error);
        }

        if handle_async(c, type, body) continue;

        if type == {
            case #char "1"; #through;
            case #char "n";

            case #char "t";
                r := Cursor.{ data = body };
                count := cast(s64) get_u16(*r);

                array_free(info.parameter_types);
                info.parameter_types = NewArray(count, u32);
                for *info.parameter_types it.* = get_u32(*r);

                if r.bad {
                    free_statement_info(*info);
                    return .{}, protocol_error(c, "a malformed ParameterDescription");
                }

            case #char "T";
                error = read_columns(c, *rows, body);
                if error.kind != .NONE {
                    free_result(*rows);
                    free_statement_info(*info);
                    return .{}, error;
                }

            case #char "E";
                if failure.kind == .NONE failure = parse_server_error(body);

            case #char "Z";
                c.status = transaction_status(ifx body.count then body[0] else 0);
                info.columns = rows.columns;
                rows.columns = .[];
                free_result(*rows);

                if failure.kind != .NONE {
                    free_statement_info(*info);
                    return .{}, failure;
                }

                return info, .{};

            case;
                free_error(*failure);
                free_result(*rows);
                free_statement_info(*info);

                return .{}, protocol_error(c, tprint("unexpected message % while describing a statement", cast(string) u8.[type]));
        }
    }

    return .{}, .{};
}

free_statement_info :: (info: *Statement_Info) {
    for info.columns free(it.name);
    array_free(info.columns);
    array_free(info.parameter_types);
    info.* = .{};
}

// Runs SQL that may hold several statements, without parameters (the simple query protocol), and
// discards any rows. For migrations and scripts. Without an explicit BEGIN the whole string runs
// as one implicit transaction, so the first error stops it and undoes the statements before it.
execute_script :: (c: *Connection, sql: string) -> Pg_Error {
    return run_script(c, sql, null);
}

// What `run_script` does with each statement's outcome. Every hook is optional.
Script_Hooks :: struct {
    // Called once per statement that completes, in order, with its rows (when it returns rows) and
    // command tag. The result is freed after the call unless the hook moves it out (and leaves
    // `result.*` zeroed). Without this hook rows are not kept at all.
    on_result: (hooks: *Script_Hooks, result: *Result);

    // When positive, rows reach `on_result` in pieces of this many as they arrive, marked
    // `partial`, so a large result never has to fit in memory. The statement's last call has the
    // remaining rows (maybe none) and the command tag. A partial result must not be moved out:
    // its rows are dropped after the call and its columns kept for the next piece.
    chunk_rows: s64;

    // Called when the server starts a COPY, before any data moves.
    on_copy: (hooks: *Script_Hooks, copy: *Copy_Info);

    // COPY ... FROM STDIN: called until it says `done`, each time for the next piece of data (any
    // split; rows need not line up with calls). A non-empty `failure` aborts the COPY with that
    // message instead, and the server answers with an error. Without this hook the COPY fails.
    copy_in: (hooks: *Script_Hooks, copy: *Copy_Info) -> data : string, done: bool, failure: string;

    // COPY ... TO STDOUT: called with each piece of data the server sends (one row in text and CSV
    // formats). Without this hook the data is dropped.
    copy_out: (hooks: *Script_Hooks, copy: *Copy_Info, data: [] u8);

    user_data: *void;
}

// The shape of a COPY the server has started.
Copy_Info :: struct {
    incoming: bool; // FROM STDIN: the client sends; else TO STDOUT: the server sends
    format: Format; // the overall format: text (also CSV) or binary
    column_formats: [] Format;
}

// Runs SQL that may hold several statements (the simple query protocol) and reports each
// statement's outcome to `hooks`. The returned error is the one that stopped the script, after the
// results of the statements before it were reported. Without an explicit BEGIN the whole string
// runs as one implicit transaction.
run_script :: (c: *Connection, sql: string, hooks: *Script_Hooks) -> Pg_Error {
    error := begin_request(c, sql);
    if error.kind != .NONE return error;

    defer c.busy = false;
    begin_message(c, #char "Q");
    put_cstring(*c.out, sql);
    end_message(c);
    error = send(c);
    if error.kind != .NONE return error;

    keep_rows := hooks && hooks.on_result;
    failure: Pg_Error;
    result: Result;
    result.allocator = context.allocator;
    result.json_options = c.json_options;
    copy: Copy_Info;

    defer {
        free_result(*result);
        array_free(copy.column_formats);
    }

    while true {
        type, body, read_error := read_message(c);
        if read_error.kind != .NONE {
            recover_interrupted(c, read_error);
            return error_before_close(*failure, read_error);
        }

        if handle_async(c, type, body) continue;

        if type == {
            case #char "T";
                result.returns_rows = true;
                if !keep_rows continue;

                error = read_columns(c, *result, body);
                if error.kind != .NONE {
                    free_error(*failure);
                    return error;
                }

            case #char "D";
                if !keep_rows continue;

                error = read_row(c, *result, body);
                if error.kind != .NONE {
                    free_error(*failure);
                    return error;
                }

                if hooks.chunk_rows > 0 && result.row_count >= hooks.chunk_rows {
                    result.partial = true;
                    hooks.on_result(hooks, *result);
                    result.partial = false;
                    clear_rows(*result);
                }

            case #char "C";
                r := Cursor.{ data = body };
                tag := get_cstring(*r);

                free(result.command);
                result.command = copy_string(tag);
                result.rows_affected = rows_in_tag(tag);

                if keep_rows hooks.on_result(hooks, *result);

                free_result(*result);
                result.allocator = context.allocator;
                result.json_options = c.json_options;

            case #char "I";
            // An empty statement: nothing to report.

            case #char "G"; #through;
            case #char "H";
                error = read_copy_info(c, *copy, body);
                if error.kind != .NONE {
                    free_error(*failure);
                    return error;
                }

                copy.incoming = type == #char "G";
                if hooks && hooks.on_copy hooks.on_copy(hooks, *copy);

                if type == #char "G" {
                    error = send_copy_data(c, hooks, *copy);
                    if error.kind != .NONE {
                        free_error(*failure);
                        return error;
                    }
                }

            case #char "d";
                if hooks && hooks.copy_out hooks.copy_out(hooks, *copy, body);

            case #char "c";

            case #char "E";
                if failure.kind == .NONE failure = parse_server_error(body);

            case #char "Z";
                c.status = transaction_status(ifx body.count then body[0] else 0);
                return failure;

            case;
                free_error(*failure);
                return protocol_error(c, tprint("unexpected message % during a simple query", cast(string) u8.[type]));
        }
    }

    return .{};
}

// The value at `row` and `column` as a `T`. NULL is an error unless `T` is a `Nullable`.
get :: (r: *Result, row: s64, column: s64, $T: Type) -> T, Pg_Error {
    value: T;

    if row < 0 || row >= r.row_count || column < 0 || column >= r.columns.count {
        return value, make_error(.USAGE, "no value at row %, column % (the result has % rows and % columns)", row, column, r.row_count, r.columns.count);
    }

    span := r.values[row * r.columns.count + column];
    info := *r.columns[column];
    bytes := [] u8.{ span.length, r.data.data + span.offset };

    #if #run is_nullable(T) {
        if span.length < 0 {
            value.null = true;
            return value, .{};
        }
    } else {
        if span.length < 0 {
            return value, make_error(.DECODE, "column \"%\" of row % is NULL; read it as Nullable(%)", info.name, row, T);
        }
    }

    why: string;

    {
        // What decoding allocates belongs to the result.
        push_allocator(.{ result_allocator, r });
        // Decoded JSON strings may point into the result, as `string` columns do.
        options := r.json_options;
        options.borrow = true;

        #if #run is_nullable(T) why = decode(bytes, info.format, info.type_oid, *value.value, *options);
        else why = decode(bytes, info.format, info.type_oid, *value, *options);
    }

    if why return value, make_error(.DECODE, "column \"%\" (%) of row %: %", info.name, type_name(info.type_oid), row, why);

    return value, .{};
}

get :: (r: *Result, row: s64, column_name: string, $T: Type) -> T, Pg_Error {
    column := column_index(r, column_name);
    if column < 0 {
        value: T;
        return value, make_error(.USAGE, "the result has no column named \"%\"", column_name);
    }

    value, error := get(r, row, column, T);

    return value, error;
}

is_null :: (r: *Result, row: s64, column: s64) -> bool {
    if row < 0 || row >= r.row_count || column < 0 || column >= r.columns.count return false;
    return r.values[row * r.columns.count + column].length < 0;
}

// The first column with this name, or -1.
column_index :: (r: *Result, name: string) -> s64 {
    for r.columns if it.name == name return it_index;
    return -1;
}

// The undecoded bytes of a value, in the column's format; `null` for NULL.
raw_value :: (r: *Result, row: s64, column: s64) -> bytes: [] u8, null: bool {
    if row < 0 || row >= r.row_count || column < 0 || column >= r.columns.count return .[], true;
    span := r.values[row * r.columns.count + column];
    if span.length < 0 return .[], true;
    return .{ span.length, r.data.data + span.offset }, false;
}

// Drops the rows of a result and what was decoded from them, keeping its columns.
clear_rows :: (r: *Result) {
    r.row_count = 0;
    r.values.count = 0;
    r.data.count = 0;

    for r.allocations free(it,, r.allocator);

    r.allocations.count = 0;
}

free_result :: (r: *Result) {
    for r.columns free(it.name);

    array_free(r.columns);
    free(r.command);
    array_free(r.values);
    array_free(r.data);
    for r.allocations free(it,, r.allocator);

    array_free(r.allocations);
    r.* = .{};
}

#scope_module

run_extended :: (c: *Connection, sql: string, args: [] Any, result_format: Format, cache := false, expected := "") -> Result, Pg_Error {
    error := begin_request(c, sql);
    if error.kind != .NONE return .{}, error;

    defer c.busy = false;
    if args.count > 65535 return .{}, make_error(.USAGE, "% parameters; Postgres allows at most 65535", args.count);

    params := NewArray(args.count, Param,, temp);

    for args {
        why: string;
        params[it_index], why = encode_param(it);
        if why return .{}, make_error(.USAGE, "parameter $%: %", it_index + 1, why);
    }

    types := NewArray(params.count, u32,, temp);
    for params types[it_index] = it.oid;

    use_cache := cache && statements_enabled(c);
    retried := false;

    while true {
        result, failure, again := exchange(c, sql, params, types, result_format, use_cache, expected, !retried);

        if again {
            retried = true;
            continue;
        }

        return result, failure;
    }

    return .{}, .{};
}

#scope_file

// One trip to the server for a statement. `again` asks the caller to repeat it: the statement it
// used from the cache had gone stale.
exchange :: (c: *Connection, sql: string, params: [] Param, types: [] u32, result_format: Format, cache: bool, expected: string, may_retry: bool) -> Result, Pg_Error, again: bool {
    use: Statement_Use;

    if cache {
        error := begin_cached(c, sql, types, params, result_format, expected, *use);
        if error.kind != .NONE return .{}, error, false;
    }

    if !use.active {
        // Parse, Bind, Describe, Execute and Sync go out together: one round trip.
        put_parse(c, "", sql, types);
        put_bind(c, "", "", params, result_format);
        put_describe(c, #char "P", "");
        put_execute(c, "", 0);
    }

    put_sync(c);

    error := send(c);
    if error.kind != .NONE return .{}, error, false;

    result: Result;
    result.allocator = context.allocator;
    result.json_options = c.json_options;
    failure: Pg_Error;

    if use.hit {
        result.columns = cached_columns(use.entry, result_format);
        result.returns_rows = use.entry.returns_rows;
    }

    while true {
        type, body, read_error := read_message(c);
        if read_error.kind != .NONE {
            free_result(*result);
            recover_interrupted(c, read_error);
            return .{}, error_before_close(*failure, read_error), false;
        }

        if handle_async(c, type, body) continue;

        if type == {
            case #char "1";
                use.parsed = true;

            case #char "2"; #through;
            case #char "3"; #through;
            case #char "t"; #through;
            case #char "I"; #through;
            case #char "s";

            case #char "n";
                use.described = true;

            case #char "T";
                error = read_columns(c, *result, body);
                if error.kind != .NONE {
                    free_result(*result);
                    return .{}, error, false;
                }

                result.returns_rows = true;
                use.described = true;

                // A statement described before it was bound reports text for every column.
                if use.active && !use.hit {
                    for *result.columns it.format = result_format;
                }

            case #char "D";
                error = read_row(c, *result, body);
                if error.kind != .NONE {
                    free_result(*result);
                    return .{}, error, false;
                }

            case #char "C";
                r := Cursor.{ data = body };
                tag := get_cstring(*r);

                free(result.command);
                result.command = copy_string(tag);
                result.rows_affected = rows_in_tag(tag);

            case #char "E";
                if failure.kind == .NONE failure = parse_server_error(body);

            case #char "Z";
                c.status = transaction_status(ifx body.count then body[0] else 0);

                again := false;
                if use.active again = finish_cached(c, *use, *result, failure, may_retry);

                if failure.kind != .NONE {
                    free_result(*result);

                    if again {
                        free_error(*failure);
                        return .{}, .{}, true;
                    }

                    return .{}, failure, false;
                }

                return result, .{}, false;

            case;
                free_error(*failure);
                free_result(*result);
                return .{}, protocol_error(c, tprint("unexpected message % during a query", cast(string) u8.[type])), false;
        }
    }

    return .{}, .{}, false;
}

#scope_module

// A deadline or cancel ended the wait for an answer, with the request already sent. Asks the
// server to stop it and reads the rest of its answer, up to ReadyForQuery, so the connection stays
// in step and usable; it breaks if that does not happen within `cancel_timeout`. A statement that
// finished before the cancel arrived has still taken effect.
recover_interrupted :: (c: *Connection, error: Pg_Error) {
    if c.broken || (error.kind != .TIMED_OUT && error.kind != .CANCELED) return;

    recovered := false;

    Io.with_grace(c.cancel_timeout, #code {
        send_cancel_request(c); // whether it worked shows in the drain below
        recovered = drain_to_ready(c);
    });

    if !recovered c.broken = true;
}

// The error a failed read ends a request with. A server that stops on a FATAL error (a crash, an
// administrator's terminate) sends it and then closes, so that error says more than the close.
error_before_close :: (failure: *Pg_Error, read_error: Pg_Error) -> Pg_Error {
    if read_error.kind == .CLOSED && failure.kind == .SERVER {
        error := failure.*;
        failure.* = .{};
        return error;
    }

    free_error(failure);

    return read_error;
}

drain_to_ready :: (c: *Connection) -> bool {
    while true {
        type, body, error := read_message(c);
        if error.kind != .NONE return false;

        if handle_async(c, type, body) continue;

        if type == {
            case #char "Z";
                c.status = transaction_status(ifx body.count then body[0] else 0);
                return true;
            case #char "G";
                // COPY FROM STDIN waits for data: end it.
                begin_message(c, #char "f");
                put_cstring(*c.out, "canceled");
                end_message(c);
                if send(c).kind != .NONE return false;
        }
    }

    return false;
}

begin_request :: (c: *Connection, sql: string) -> Pg_Error {
    if !c return make_error(.USAGE, "no connection");
    if c.broken return make_error(.CLOSED, "the connection was broken by an earlier failure");
    if c.pipeline.active return make_error(.USAGE, "synchronous command execution functions are not allowed in pipeline mode");
    if c.busy return make_error(.USAGE, "the connection is in use by another fiber; borrow one per fiber from a pool");
    if !Io.in_scheduler() return make_error(.USAGE, "Postgres calls must run on a fiber: call them inside Io.io_run");
    if contains_zero(sql) return make_error(.USAGE, "the SQL holds a zero byte");

    c.busy = true;

    return .{};
}

read_columns :: (c: *Connection, result: *Result, body: [] u8) -> Pg_Error {
    r := Cursor.{ data = body };
    count := cast(s64) get_u16(*r);

    for result.columns free(it.name);

    array_free(result.columns);
    result.columns = NewArray(count, Column);
    for *result.columns {
        it.name = copy_string(get_cstring(*r));
        it.table_oid = get_u32(*r);
        it.column_number = get_s16(*r);
        it.type_oid = get_u32(*r);
        it.type_size = get_s16(*r);
        it.type_modifier = get_s32(*r);
        it.format = cast(Format) get_u16(*r);
    }

    if r.bad return protocol_error(c, "a malformed RowDescription");

    return .{};
}

read_row :: (c: *Connection, result: *Result, body: [] u8) -> Pg_Error {
    r := Cursor.{ data = body };
    count := cast(s64) get_u16(*r);

    if count != result.columns.count return protocol_error(c, "a DataRow with the wrong number of columns");

    for 0..count - 1 {
        length := cast(s64) get_s32(*r);
        if length < 0 {
            array_add(*result.values, .{ offset = 0, length = -1 });
            continue;
        }

        bytes := get_bytes(*r, length);

        if r.bad break;

        array_add(*result.values, .{ offset = result.data.count, length = length });
        put_bytes(*result.data, bytes);
    }

    if r.bad return protocol_error(c, "a malformed DataRow");

    result.row_count += 1;

    return .{};
}

read_copy_info :: (c: *Connection, copy: *Copy_Info, body: [] u8) -> Pg_Error {
    r := Cursor.{ data = body };

    copy.format = cast(Format) get_u8(*r);
    count := cast(s64) get_u16(*r);

    array_free(copy.column_formats);
    copy.column_formats = NewArray(count, Format);
    for *copy.column_formats it.* = cast(Format) get_u16(*r);

    if r.bad return protocol_error(c, "a malformed COPY response");

    return .{};
}

// Feeds COPY FROM STDIN from the hook, then ends it with CopyDone or CopyFail.
send_copy_data :: (c: *Connection, hooks: *Script_Hooks, copy: *Copy_Info) -> Pg_Error {
    if !hooks || !hooks.copy_in {
        begin_message(c, #char "f");
        put_cstring(*c.out, "COPY FROM STDIN needs Script_Hooks.copy_in");
        end_message(c);
        return send(c);
    }

    COPY_CHUNK :: 64 * 1024; // flush when this much is waiting

    while true {
        data, done, failure := hooks.copy_in(hooks, copy);

        if failure {
            begin_message(c, #char "f");
            put_cstring(*c.out, ifx contains_zero(failure) then "COPY aborted" else failure);
            end_message(c);
            return send(c);
        }

        if data {
            begin_message(c, #char "d");
            put_bytes(*c.out, cast([] u8) data);
            end_message(c);
        }

        if done break;

        if c.out.count >= COPY_CHUNK {
            error := send(c);
            if error.kind != .NONE return error;
        }
    }

    begin_message(c, #char "c");
    end_message(c);

    return send(c);
}

// "INSERT 0 5" is 5, "UPDATE 3" is 3, "CREATE TABLE" is 0.
rows_in_tag :: (tag: string) -> s64 {
    space := find_index_from_right(tag, #char " ");
    if space < 0 return 0;

    value, valid, rest := string_to_int(advance(tag, space + 1), 10, s64);
    if !valid || rest return 0;

    return value;
}

// Allocations made while decoding a value are recorded in the result and freed with it.
result_allocator :: (mode: Allocator_Mode, size: s64, old_size: s64, old_memory: *void, allocator_data: *void) -> *void {
    r := cast(*Result) allocator_data;

    if mode == {
        case .ALLOCATE;
            memory := alloc(size,, r.allocator);

            array_add(*r.allocations, memory,, r.allocator);
            return memory;
        case .RESIZE;
            memory := realloc(old_memory, size, old_size,, r.allocator);

            if old_memory {
                for *r.allocations if it.* == old_memory {
                    it.* = memory;
                    return memory;
                }
            }

            array_add(*r.allocations, memory,, r.allocator);

            return memory;
        case .FREE;
            // Freed with the result.
            return null;
    }

    return null;
}
