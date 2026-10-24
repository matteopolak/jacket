// Pipelining: send several statements without waiting for the answers, then read the answers in
// order. The semantics are libpq's pipeline mode.
//
//     p, error := Pg.pipeline_enter(db);
//     Pg.pipeline_query(p, "INSERT INTO log(line) VALUES ($1)", "one");
//     Pg.pipeline_query(p, "SELECT count(*) FROM log");
//     Pg.pipeline_sync(p); // sends everything and ends the batch
//     while true {
//         item, error := Pg.pipeline_next_result(p);
//         defer Pg.free_pipeline_result(*item);
//         if item.kind == .SYNC break;
//         // .RESULT: item.result holds the rows; .ERROR: item.error; .ABORTED: it was skipped
//     }
//     Pg.pipeline_exit(p);
//
// Statements queue into the connection's output buffer. They leave on `pipeline_sync`,
// `pipeline_flush` or `pipeline_flush_request`, or when a result is read. The server runs the
// statements between two sync points as one implicit transaction (unless the pipeline opens its
// own), so the first error ends the batch: its statement reports the error, the rest of the batch
// reports `.ABORTED`, and the sync point reports `.SYNC`, after which the pipeline is usable again.
//
// A pipeline sends without reading. If it queues so much that the server's answers fill the
// socket buffers before the client reads any, both sides wait on each other; read results (or
// sync) every few hundred kilobytes of queued statements.

Pipeline_Kind :: enum u8 {
    NONE; // nothing is waiting for a result
    RESULT; // a statement finished: its rows and command tag, or what a prepare/describe/close did
    ROWS; // chunked rows mode: some rows of a statement that goes on (`result.partial`)
    ERROR; // the server rejected the statement: `error`
    ABORTED; // the statement was skipped because an earlier one of its batch failed
    SYNC; // a sync point was reached: the batch is over and the server is ready again
}

// What one queued statement or sync point turned into.
Pipeline_Result :: struct {
    kind: Pipeline_Kind;
    // For RESULT and ROW: the columns, rows and command tag. After a describe, only the columns.
    result: Result;
    // After `pipeline_describe_statement`: the types the server inferred for `$1`, `$2`, ...
    parameter_types: [] u32;
    error: Pg_Error; // for ERROR
}

Pipeline_Step :: enum u8 {
    QUERY; // Parse (unless prepared), Bind, Describe portal, Execute
    PREPARE;
    DESCRIBE_STATEMENT;
    DESCRIBE_PORTAL;
    CLOSE;
    SYNC;
}

// The pipeline of a connection (`pipeline_enter`). While it is active, the connection answers no
// other request.
Pipeline :: struct {
    connection: *Connection;
    active: bool;
    // A statement failed and its batch has not reached its sync point yet: the rest of the batch
    // is skipped.
    aborted: bool;

    steps: [..] Pipeline_Step; // queued, whose results are not all read; from `head` on
    head: s64;
    unsynced: s64; // statements queued since the last sync point
    chunk_rows: s64; // the statement being read delivers this many rows per result (0: all at once)
    tag_ready: bool; // the statement's last rows were delivered, and its command tag is in `current`
    current: Pipeline_Result; // the result being assembled: a deadline in the middle loses nothing
}

// Starts pipeline mode on a connection. Entering again returns the same pipeline.
pipeline_enter :: (c: *Connection) -> *Pipeline, Pg_Error {
    if !c return null, make_error(.USAGE, "no connection");
    if c.pipeline.active return *c.pipeline, .{};
    if c.broken return null, make_error(.CLOSED, "the connection was broken by an earlier failure");
    if c.busy return null, make_error(.USAGE, "the connection is in use by another fiber; borrow one per fiber from a pool");
    if !Io.in_scheduler() return null, make_error(.USAGE, "Postgres calls must run on a fiber: call them inside Io.io_run");

    p := *c.pipeline;
    p.* = .{ connection = c, active = true };
    start_result(p);

    return p, .{};
}

// Ends pipeline mode. It fails, and the pipeline stays, while results are unread.
pipeline_exit :: (p: *Pipeline) -> Pg_Error {
    if !p.active return .{};
    if p.head < p.steps.count return make_error(.USAGE, "cannot exit pipeline mode with uncollected results");
    if p.connection.out.count return make_error(.USAGE, "cannot exit pipeline mode with unsent statements: flush or sync first");

    free_pipeline(p);

    return .{};
}

// Statements waiting for their results, sync points included.
pipeline_pending :: (p: *Pipeline) -> s64 {
    return p.steps.count - p.head;
}

// Queues a statement with parameters `$1`, `$2`, ... from `args` (see `query_dynamic` for the types),
// results in text format.
pipeline_query :: (p: *Pipeline, sql: string, args: ..Any) -> Pg_Error {
    values := args; // `sql` is not a print format: keep it and `args` out of one call
    return queue_typed(p, sql, "", values, .TEXT);
}

// `pipeline_query` with results in binary format.
pipeline_query_binary :: (p: *Pipeline, sql: string, args: ..Any) -> Pg_Error {
    values := args;
    return queue_typed(p, sql, "", values, .BINARY);
}

// Queues a statement whose parameters are already text. `types` are the parameter OIDs to declare
// (0, or fewer than `values`, leaves the rest to the server).
pipeline_query_text :: (p: *Pipeline, sql: string, types: [] u32, values: [] string, result_format := Format.TEXT) -> Pg_Error {
    return queue_text(p, sql, "", types, values, result_format);
}

// Prepares a named statement. `types` are the parameter OIDs to declare.
pipeline_prepare :: (p: *Pipeline, name: string, sql: string, types: [] u32 = .[]) -> Pg_Error {
    error := check_queue(p);
    if error.kind != .NONE return error;
    if contains_zero(name) || contains_zero(sql) return make_error(.USAGE, "the name or SQL holds a zero byte");

    put_parse(p.connection, name, sql, types);
    add_step(p, .PREPARE);

    return .{};
}

// Runs a prepared statement with `args` (see `pipeline_query`), results in text format.
pipeline_execute_prepared :: (p: *Pipeline, name: string, args: ..Any) -> Pg_Error {
    values := args;
    return queue_typed(p, "", name, values, .TEXT);
}

// Runs a prepared statement with parameters that are already text.
pipeline_execute_prepared_text :: (p: *Pipeline, name: string, values: [] string, result_format := Format.TEXT) -> Pg_Error {
    return queue_text(p, "", name, .[], values, result_format);
}

// The result of these four is `.RESULT`; a describe also fills `result.columns`, and a statement
// describe `parameter_types`.
pipeline_describe_statement :: (p: *Pipeline, name: string) -> Pg_Error {
    return queue_named(p, .DESCRIBE_STATEMENT, name);
}

pipeline_describe_portal :: (p: *Pipeline, name: string) -> Pg_Error {
    return queue_named(p, .DESCRIBE_PORTAL, name);
}

// Closing a statement or portal that does not exist is not an error.
pipeline_close_statement :: (p: *Pipeline, name: string) -> Pg_Error {
    return queue_named(p, .CLOSE, name, #char "S");
}

pipeline_close_portal :: (p: *Pipeline, name: string) -> Pg_Error {
    return queue_named(p, .CLOSE, name, #char "P");
}

// Queues a sync point and sends everything queued.
pipeline_sync :: (p: *Pipeline) -> Pg_Error {
    error := pipeline_queue_sync(p);
    if error.kind != .NONE return error;

    return send(p.connection);
}

// Queues a sync point without sending.
pipeline_queue_sync :: (p: *Pipeline) -> Pg_Error {
    error := check_queue(p);
    if error.kind != .NONE return error;

    put_sync(p.connection);
    array_add(*p.steps, .SYNC);
    p.unsynced = 0;

    return .{};
}

// Asks the server to send the answers so far without ending the batch, and sends everything
// queued. Without a sync point, a batch that fails is not rolled back until the next one.
pipeline_flush_request :: (p: *Pipeline) -> Pg_Error {
    error := check_queue(p);
    if error.kind != .NONE return error;

    put_flush(p.connection);

    return send(p.connection);
}

// Sends everything queued.
pipeline_flush :: (p: *Pipeline) -> Pg_Error {
    error := check_queue(p);
    if error.kind != .NONE return error;

    return send(p.connection);
}

// Makes the next statement to be read deliver its rows `count` at a time (`.ROWS`; the last piece
// may have fewer), then end with a `.RESULT` that has no rows but the command tag, so a large
// result never has to fit in memory. It only works before the statement's first result is read.
// Reverts at the statement's end.
pipeline_chunked_rows :: (p: *Pipeline, count: s64) -> bool {
    if !p.active || p.head >= p.steps.count || count < 1 return false;
    if p.steps[p.head] != .QUERY || p.current.result.row_count > 0 return false;

    p.chunk_rows = count;

    return true;
}

// `pipeline_chunked_rows` with one row per result.
pipeline_single_row :: (p: *Pipeline) -> bool {
    return pipeline_chunked_rows(p, 1);
}

// Reads the result of the oldest queued statement or sync point, sending what is still unsent
// first. Blocks the fiber until the answer is in, within any `Io.with_deadline`: after a deadline
// the pipeline keeps what it had read, and the call can be repeated. The result's kind is NONE
// when nothing is queued. The returned error is for failures of the connection; the server's
// errors come as `.ERROR` results. Free the result with `free_pipeline_result`.
pipeline_next_result :: (p: *Pipeline) -> Pipeline_Result, Pg_Error {
    error := check_queue(p);
    if error.kind != .NONE return .{}, error;

    if p.head >= p.steps.count return .{}, .{};

    c := p.connection;

    error = send(c);
    if error.kind != .NONE return .{}, error;

    while true {
        step := p.steps[p.head];

        if p.aborted && step != .SYNC {
            next_step(p);

            return .{ kind = .ABORTED }, .{};
        }

        outcome := Outcome.DONE;

        if p.tag_ready {
            p.tag_ready = false;
            p.current.kind = .RESULT;
        } else {
            type, body, read_error := read_message(c);
            if read_error.kind != .NONE return .{}, read_error;
            if handle_async(c, type, body) continue;

            outcome, error = consume(p, step, type, body);
            if error.kind != .NONE return .{}, error;
            if outcome == .MORE continue;
        }

        item := p.current;

        p.current = .{};
        start_result(p);

        if outcome == .DONE {
            next_step(p);
        } else if item.kind == .ROWS {
            // The statement goes on: its next rows need the columns too, and maybe its tag.
            p.current.result.columns = copy_columns(item.result.columns);
            p.current.result.returns_rows = true;

            if p.tag_ready {
                p.current.result.command = item.result.command;
                p.current.result.rows_affected = item.result.rows_affected;
                item.result.command = string.{};
                item.result.rows_affected = 0;
            }
        }

        return item, .{};
    }

    return .{}, .{};
}

free_pipeline_result :: (item: *Pipeline_Result) {
    free_result(*item.result);
    array_free(item.parameter_types);
    free_error(*item.error);
    item.* = .{};
}

// Gives up on everything queued: asks the server to cancel what runs, reads and drops all answers
// up to the last sync point (queuing one if needed), and leaves pipeline mode. The connection
// breaks if the server does not finish within `cancel_timeout`.
pipeline_abandon :: (p: *Pipeline) {
    if !p.active return;

    c := p.connection;

    if !c.broken {
        if p.unsynced > 0 || c.out.count > 0 pipeline_queue_sync(p);

        syncs := 0;
        for p.head..p.steps.count - 1 if p.steps[it] == .SYNC syncs += 1;

        recovered := false;

        Io.with_grace(c.cancel_timeout, #code {
            if send(c).kind == .NONE {
                send_cancel_request(c); // whether it worked shows in the drain below
                recovered = true;

                for 1..syncs {
                    if !drain_to_ready(c) {
                        recovered = false;
                        break;
                    }
                }
            }
        });

        if !recovered c.broken = true;
    }

    free_pipeline(p);
}

// Describes a prepared statement by name, without a pipeline (Describe, Sync).
describe_prepared :: (c: *Connection, name: string) -> Statement_Info, Pg_Error {
    item, error := run_named(c, .DESCRIBE_STATEMENT, name);
    if error.kind != .NONE return .{}, error;

    info := Statement_Info.{ parameter_types = item.parameter_types, columns = item.result.columns };

    // The info owns these now.
    item.parameter_types = .[];
    item.result.columns = .[];
    free_pipeline_result(*item);

    return info, .{};
}

// Describes a portal (a cursor, say) by name: the columns it returns.
describe_portal :: (c: *Connection, name: string) -> Statement_Info, Pg_Error {
    item, error := run_named(c, .DESCRIBE_PORTAL, name);
    if error.kind != .NONE return .{}, error;

    info := Statement_Info.{ columns = item.result.columns };

    item.result.columns = .[];
    free_pipeline_result(*item);

    return info, .{};
}

// Drops a prepared statement by name. One that does not exist is not an error.
close_prepared :: (c: *Connection, name: string) -> Pg_Error {
    item, error := run_named(c, .CLOSE, name, #char "S");
    free_pipeline_result(*item);

    return error;
}

// Drops a portal by name. One that does not exist is not an error.
close_portal :: (c: *Connection, name: string) -> Pg_Error {
    item, error := run_named(c, .CLOSE, name, #char "P");
    free_pipeline_result(*item);

    return error;
}

#scope_module

Outcome :: enum u8 {
    MORE; // the statement has more messages
    EMIT; // return `current`, and stay on the same statement
    DONE; // return `current`: the statement is over
}

check_queue :: (p: *Pipeline) -> Pg_Error {
    if !p.active return make_error(.USAGE, "the connection is not in pipeline mode");
    if p.connection.broken return make_error(.CLOSED, "the connection was broken by an earlier failure");
    if !Io.in_scheduler() return make_error(.USAGE, "Postgres calls must run on a fiber: call them inside Io.io_run");

    return .{};
}

start_result :: (p: *Pipeline) {
    p.current = .{};
    p.current.result.allocator = context.allocator;
    p.current.result.json_options = p.connection.json_options;
}

add_step :: (p: *Pipeline, step: Pipeline_Step) {
    array_add(*p.steps, step);
    p.unsynced += 1;
}

next_step :: (p: *Pipeline) {
    p.head += 1;
    p.chunk_rows = 0;

    if p.head >= p.steps.count {
        p.steps.count = 0;
        p.head = 0;
    }
}

free_pipeline :: (p: *Pipeline) {
    free_pipeline_result(*p.current);
    array_free(p.steps);
    p.* = .{};
}

copy_columns :: (columns: [] Column) -> [] Column {
    copy := NewArray(columns.count, Column);

    for columns {
        copy[it_index] = it;
        copy[it_index].name = copy_string(it.name);
    }

    return copy;
}

queue_typed :: (p: *Pipeline, sql: string, statement: string, args: [] Any, result_format: Format) -> Pg_Error {
    error := check_queue(p);
    if error.kind != .NONE return error;
    if args.count > 65535 return make_error(.USAGE, "% parameters; Postgres allows at most 65535", args.count);
    if contains_zero(sql) || contains_zero(statement) return make_error(.USAGE, "the name or SQL holds a zero byte");

    params := NewArray(args.count, Param,, temp);
    types := NewArray(args.count, u32,, temp);

    for args {
        why: string;
        params[it_index], why = encode_param(it);
        if why return make_error(.USAGE, "parameter $%: %", it_index + 1, why);

        types[it_index] = params[it_index].oid;
    }

    queue_query(p, sql, statement, types, params, result_format);

    return .{};
}

queue_text :: (p: *Pipeline, sql: string, statement: string, types: [] u32, values: [] string, result_format: Format) -> Pg_Error {
    error := check_queue(p);
    if error.kind != .NONE return error;
    if values.count > 65535 return make_error(.USAGE, "% parameters; Postgres allows at most 65535", values.count);
    if contains_zero(sql) || contains_zero(statement) return make_error(.USAGE, "the name or SQL holds a zero byte");

    params := NewArray(values.count, Param,, temp);

    for values {
        if contains_zero(it) return make_error(.USAGE, "parameter $% holds a zero byte", it_index + 1);
        params[it_index] = .{ format = .TEXT, data = it };
    }

    queue_query(p, sql, statement, types, params, result_format);

    return .{};
}

// Parse (when there is SQL), Bind, Describe and Execute: one statement with a row description.
queue_query :: (p: *Pipeline, sql: string, statement: string, types: [] u32, params: [] Param, result_format: Format) {
    c := p.connection;

    if sql put_parse(c, statement, sql, types);

    put_bind(c, "", statement, params, result_format);
    put_describe(c, #char "P", "");
    put_execute(c, "", 0);
    add_step(p, .QUERY);
}

queue_named :: (p: *Pipeline, step: Pipeline_Step, name: string, kind := #char "S") -> Pg_Error {
    error := check_queue(p);
    if error.kind != .NONE return error;
    if contains_zero(name) return make_error(.USAGE, "the name holds a zero byte");

    if step == .CLOSE {
        put_close(p.connection, cast(u8) kind, name);
    } else {
        put_describe(p.connection, ifx step == .DESCRIBE_STATEMENT then #char "S" else #char "P", name);
    }

    add_step(p, step);

    return .{};
}

// Takes the messages of one statement and says whether the result is complete.
consume :: (p: *Pipeline, step: Pipeline_Step, type: u8, body: [] u8) -> Outcome, Pg_Error {
    c := p.connection;
    out := *p.current;

    if type == #char "E" {
        free_result(*out.result);
        out.kind = .ERROR;
        out.error = parse_server_error(body);
        p.aborted = true;

        // An error while syncing (a commit that fails) comes before the sync's own answer.
        if step == .SYNC return .EMIT, .{};

        return .DONE, .{};
    }

    if step == {
        case .SYNC;
            if type != #char "Z" return .MORE, unexpected(c, type, "a sync point");

            c.status = transaction_status(ifx body.count then body[0] else 0);
            p.aborted = false;
            out.kind = .SYNC;

            return .DONE, .{};
        case .PREPARE;
            if type != #char "1" return .MORE, unexpected(c, type, "a prepare");

            out.kind = .RESULT;

            return .DONE, .{};
        case .CLOSE;
            if type != #char "3" return .MORE, unexpected(c, type, "a close");

            out.kind = .RESULT;

            return .DONE, .{};
        case .DESCRIBE_STATEMENT; #through;
        case .DESCRIBE_PORTAL;
            return consume_describe(p, step, type, body);
        case .QUERY;
            return consume_query(p, type, body);
    }

    return .MORE, .{};
}

consume_describe :: (p: *Pipeline, step: Pipeline_Step, type: u8, body: [] u8) -> Outcome, Pg_Error {
    c := p.connection;
    out := *p.current;

    if type == {
        case #char "t";
            if step != .DESCRIBE_STATEMENT return .MORE, unexpected(c, type, "a portal description");

            r := Cursor.{ data = body };
            count := cast(s64) get_u16(*r);

            array_free(out.parameter_types);
            out.parameter_types = NewArray(count, u32);
            for *out.parameter_types it.* = get_u32(*r);

            if r.bad return .MORE, protocol_error(c, "a malformed ParameterDescription");
        case #char "T";
            error := read_columns(c, *out.result, body);
            if error.kind != .NONE return .MORE, error;

            out.result.returns_rows = true;
            out.kind = .RESULT;

            return .DONE, .{};
        case #char "n";
            out.kind = .RESULT;

            return .DONE, .{};
        case;
            return .MORE, unexpected(c, type, "a description");
    }

    return .MORE, .{};
}

consume_query :: (p: *Pipeline, type: u8, body: [] u8) -> Outcome, Pg_Error {
    c := p.connection;
    out := *p.current;

    if type == {
        case #char "1"; #through;
        case #char "2"; #through;
        case #char "n";
        case #char "T";
            error := read_columns(c, *out.result, body);
            if error.kind != .NONE return .MORE, error;

            out.result.returns_rows = true;
        case #char "D";
            error := read_row(c, *out.result, body);
            if error.kind != .NONE return .MORE, error;

            if p.chunk_rows > 0 && out.result.row_count >= p.chunk_rows {
                out.kind = .ROWS;
                out.result.partial = true;
                return .EMIT, .{};
            }
        case #char "C";
            r := Cursor.{ data = body };
            tag := get_cstring(*r);

            free(out.result.command);
            out.result.command = copy_string(tag);
            out.result.rows_affected = rows_in_tag(tag);
            out.kind = .RESULT;

            if p.chunk_rows > 0 && out.result.row_count > 0 {
                // The rows left over go first; the tag follows on its own.
                out.kind = .ROWS;
                out.result.partial = true;
                p.tag_ready = true;

                return .EMIT, .{};
            }

            return .DONE, .{};
        case #char "I"; #through;
        case #char "s";
            // No statement to tag: an empty query, or a portal that stopped at its row limit.
            out.kind = .RESULT;

            return .DONE, .{};
        case #char "G"; #through;
        case #char "H"; #through;
        case #char "W";
            // As in libpq, the connection ends: a COPY would need the rest of the pipeline.
            return .MORE, protocol_error(c, "COPY in a pipeline is not supported, aborting connection");
        case;
            return .MORE, unexpected(c, type, "a statement");
    }

    return .MORE, .{};
}

unexpected :: (c: *Connection, type: u8, what: string) -> Pg_Error {
    return protocol_error(c, tprint("unexpected message % during %", cast(string) u8.[type], what));
}

// One statement run outside any pipeline: it, a Sync, and both answers.
run_named :: (c: *Connection, step: Pipeline_Step, name: string, kind := #char "S") -> Pipeline_Result, Pg_Error {
    if c && c.pipeline.active return .{}, make_error(.USAGE, "synchronous command execution functions are not allowed in pipeline mode");

    p, error := pipeline_enter(c);
    if error.kind != .NONE return .{}, error;

    error = queue_named(p, step, name, kind);
    if error.kind == .NONE error = pipeline_sync(p);

    item: Pipeline_Result;

    if error.kind == .NONE item, error = pipeline_next_result(p);

    if error.kind == .NONE {
        sync: Pipeline_Result;
        sync, error = pipeline_next_result(p);
        free_pipeline_result(*sync);
    }

    if error.kind != .NONE {
        free_pipeline_result(*item);
        pipeline_abandon(p);

        return .{}, error;
    }

    pipeline_exit(p);

    if item.kind == .ERROR {
        failure := item.error;

        item.error = .{};
        free_pipeline_result(*item);

        return .{}, failure;
    }

    return item, .{};
}
