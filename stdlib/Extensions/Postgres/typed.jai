// Typed raw queries: SQL checked against the database while the program compiles.
//
//     rows, error := Pg.query(db, "select id, email from users where id = %", user_id);
//     for rows print("% %\n", it.id, it.email);
//     Pg.free_rows(*rows);
//
// `query`, `query_one`, `query_opt`, `execute`, `query_scalar` and `query_as` are macros: each
// reads its SQL string and argument types at compile time (resolve.jai), checks them against
// the described statement, and expands into a call that sends the rewritten statement. See
// docs/postgres/typed-queries.md.

// A typed SQL NULL for any parameter: `Pg.query(db, "update t set x = %", Pg.NULL)`.
Null_Marker :: struct {}

NULL :: Nullable(Null_Marker).{ null = true };

// The row of a described statement: one field per column, named by the column. `shape` is the
// struct body the describe step wrote (doc comments included), so equal columns give one type.
Row :: struct($shape: string) {
    #insert shape;
}

// The result of `query`: it owns the received data, and `for rows` hands out one row at a time.
// Strings and byte slices in a row point into that data and stay valid until `free_rows`.
Rows :: struct(T: Type) {
    result: Result;
    // For a hand-written row type (`query_as`): the column behind each field, in field order.
    map: [] s64;
    // Set when a row failed to decode: the loop stops, and the error is here.
    error: Pg_Error;
}

// Every row decoded and copied, with the strings, arrays and documents it points to, into one
// block from `allocator`. The rows stay valid after `free_rows`; `array_free(items)` frees all of
// it (with `temp`, nothing needs freeing). Check `rows.error` afterwards.
//
// - rows: the rows to decode; they are read from the start.
// - allocator: where the block of rows goes.
//
// ```jai,ignore
// users := Pg.fetch_all(*rows);   // or Pg.fetch_all(*rows, temp)
// Pg.free_rows(*rows);
// ```
//
// Not run: it needs a PostgreSQL server.
fetch_all :: (rows: *Rows($T), allocator := context.allocator) -> [] T {
    decoded: [..] T;
    decoded.allocator = temp;

    for 0..rows.result.row_count - 1 {
        item: T;
        error := read_row(rows, it, *item);

        if error.kind != .NONE {
            rows.error = error;
            break;
        }

        array_add(*decoded, item);
    }

    items: [] T;
    if !decoded.count return items;

    head := align_copy_size(decoded.count * size_of(T));
    size := head;

    for *decoded size += copy_size(it, type_info(T));

    block := cast(*u8) alloc(size,, allocator);

    memcpy(block, decoded.data, decoded.count * size_of(T));

    items.data = cast(*T) block;
    items.count = decoded.count;

    cursor := block + head;
    for *items copy_into(it, type_info(T), *cursor);

    return items;
}

#scope_module

align_copy_size :: (size: s64) -> s64 {
    return (size + 7) & ~7;
}

// The bytes `value` points to (not counting its own), each piece rounded up to 8.
copy_size :: (value: *void, info: *Type_Info) -> s64 {
    if info.type == {
        case .STRING;
            return align_copy_size((cast(*string) value).count);

        case .STRUCT;
            total := 0;

            for (cast(*Type_Info_Struct) info).members {
                if it.flags & .CONSTANT continue;
                total += copy_size((cast(*u8) value) + it.offset_in_bytes, it.type);
            }

            return total;

        case .ARRAY;
            array := cast(*Type_Info_Array) info;
            element := array.element_type;
            count, data := array_parts(value, array);
            total := ifx array.array_type == .FIXED then 0 else align_copy_size(count * element.runtime_size);

            for 0..count - 1 total += copy_size(data + it * element.runtime_size, element);

            return total;

        case .POINTER;
            target := (cast(*Type_Info_Pointer) info).pointer_to;
            pointer := (cast(**u8) value).*;

            if !pointer || !target || target.runtime_size <= 0 return 0;

            return align_copy_size(target.runtime_size) + copy_size(pointer, target);
    }

    return 0;
}

// Moves what `value` points to into the block at `cursor`, and points `value` there.
copy_into :: (value: *void, info: *Type_Info, cursor: **u8) {
    if info.type == {
        case .STRING;
            text := cast(*string) value;
            if !text.count return;

            memcpy(cursor.*, text.data, text.count);
            text.data = cursor.*;
            cursor.* += align_copy_size(text.count);

        case .STRUCT;
            for (cast(*Type_Info_Struct) info).members {
                if it.flags & .CONSTANT continue;
                copy_into((cast(*u8) value) + it.offset_in_bytes, it.type, cursor);
            }

        case .ARRAY;
            array := cast(*Type_Info_Array) info;
            element := array.element_type;
            count, data := array_parts(value, array);

            if array.array_type != .FIXED {
                if !count return;

                memcpy(cursor.*, data, count * element.runtime_size);
                data = cursor.*;
                cursor.* += align_copy_size(count * element.runtime_size);

                // A copied [..] array is a view into the block: it must not grow in place.
                view := cast(*Array_View_64) value;
                view.data = data;

                if array.array_type == .RESIZABLE {
                    resizable := cast(*Resizable_Array) value;
                    resizable.allocated = count;
                    resizable.allocator = .{};
                }
            }

            for 0..count - 1 copy_into(data + it * element.runtime_size, element, cursor);

        case .POINTER;
            target := (cast(*Type_Info_Pointer) info).pointer_to;
            pointer := cast(**u8) value;

            if !pointer.* || !target || target.runtime_size <= 0 return;

            memcpy(cursor.*, pointer.*, target.runtime_size);
            pointer.* = cursor.*;
            cursor.* += align_copy_size(target.runtime_size);

            copy_into(pointer.*, target, cursor);
    }
}

array_parts :: (value: *void, array: *Type_Info_Array) -> count: s64, data: *u8 {
    if array.array_type == .FIXED return array.array_count, cast(*u8) value;

    view := cast(*Array_View_64) value;

    return view.count, view.data;
}

#scope_export

// Frees the result data of `rows`.
//
// - rows: the rows to release; they are reset.
free_rows :: (rows: *Rows($T)) {
    free_result(*rows.result);
    free_error(*rows.error);
    rows.* = .{};
}

// Loops over the rows, decoding each into `it`. A row that fails to decode stops the loop and
// sets `rows.error`.
//
// - rows: the rows to loop over.
// - body: the loop body.
// - flags: the loop's flags.
for_expansion :: (rows: *Rows($T), body: Code, flags: For_Flags) #expand {
    for `it_index: 0..rows.result.row_count - 1 {
        `it: T;
        error := read_row(rows, it_index, *`it);

        if error.kind != .NONE {
            rows.error = error;
            break;
        }

        #insert body;
    }
}

// Runs the statement and returns its rows. `Rows(Row(...))` has one field per column; when the
// columns are exactly one table's, it is `Rows(Table)` with that table's struct from db/schema.jai
// (row type identity, see `row_type`).
//
// - db: a `*Connection` or a `*Pool`.
// - sql: the statement, written at the call as a string: `%` marks each argument, `%%` is a literal `%`.
// - args: one value per `%`, in order.
// - plan: `.AUTO` runs it as a cached named statement; `.CUSTOM` plans it again on every run.
// - json: decoding options for json columns in this query; they apply over the connection's and the column's.
// - call: the code of the call, which lets the query text be checked while compiling. Leave the default.
query :: (db: $D, $sql: string, args: ..Any, plan := Plan.AUTO, json := Json_Options.{}, $call := #caller_code) -> Rows(#run row_type(sql, call, 1, #insert,scope(call) #run row_table_type(sql, call, 1, #insert,scope(call) #run row_table_exists(sql, call, 1)), Row(#run row_shape(sql, call, 1)))), Pg_Error #expand {
    #run check_statement(sql, call, 1, true);

    rows: Rows(#run row_type(sql, call, 1, #insert,scope(call) #run row_table_type(sql, call, 1, #insert,scope(call) #run row_table_exists(sql, call, 1)), Row(#run row_shape(sql, call, 1))));
    error: Pg_Error;
    rows.result, error = run_statement(db, #run rewritten_sql(sql, call, 1), #run binary_results(sql, call, 1), args, plan, json, #run expected_columns(sql, call, 1));

    return rows, error;
} @Interpolates("sql")

// The only row, or a `.NOT_FOUND` error when there is none. More rows are a `.USAGE` error. The row
// lives in temporary storage (until `reset_temporary_storage`).
//
// - db: a `*Connection` or a `*Pool`.
// - sql: the statement, written at the call as a string: `%` marks each argument, `%%` is a literal `%`.
// - args: one value per `%`, in order.
// - plan: `.AUTO` runs it as a cached named statement; `.CUSTOM` plans it again on every run.
// - json: decoding options for json columns in this query; they apply over the connection's and the column's.
// - call: the code of the call, which lets the query text be checked while compiling. Leave the default.
query_one :: (db: $D, $sql: string, args: ..Any, plan := Plan.AUTO, json := Json_Options.{}, $call := #caller_code) -> #run row_type(sql, call, 1, #insert,scope(call) #run row_table_type(sql, call, 1, #insert,scope(call) #run row_table_exists(sql, call, 1)), Row(#run row_shape(sql, call, 1))), Pg_Error #expand {
    #run check_statement(sql, call, 1, true);

    value: #run row_type(sql, call, 1, #insert,scope(call) #run row_table_type(sql, call, 1, #insert,scope(call) #run row_table_exists(sql, call, 1)), Row(#run row_shape(sql, call, 1)));
    result, error := run_statement(db, #run rewritten_sql(sql, call, 1), #run binary_results(sql, call, 1), args, plan, json, #run expected_columns(sql, call, 1));

    if error.kind != .NONE return value, error;

    if result.row_count != 1 {
        count := result.row_count;
        free_result(*result);

        if count == 0 return value, make_error(.NOT_FOUND, "no rows");

        return value, make_error(.USAGE, "expected one row (the query returned % rows)", count);
    }

    rows := Rows(type_of(value)).{ result = detach_to_temp(*result) };

    error = read_row(*rows, 0, *value);

    return value, error;
} @Interpolates("sql")

// The first row and whether there was one.
//
// - db: a `*Connection` or a `*Pool`.
// - sql: the statement, written at the call as a string: `%` marks each argument, `%%` is a literal `%`.
// - args: one value per `%`, in order.
// - plan: `.AUTO` runs it as a cached named statement; `.CUSTOM` plans it again on every run.
// - json: decoding options for json columns in this query; they apply over the connection's and the column's.
// - call: the code of the call, which lets the query text be checked while compiling. Leave the default.
query_opt :: (db: $D, $sql: string, args: ..Any, plan := Plan.AUTO, json := Json_Options.{}, $call := #caller_code) -> #run row_type(sql, call, 1, #insert,scope(call) #run row_table_type(sql, call, 1, #insert,scope(call) #run row_table_exists(sql, call, 1)), Row(#run row_shape(sql, call, 1))), bool, Pg_Error #expand {
    #run check_statement(sql, call, 1, true);

    value: #run row_type(sql, call, 1, #insert,scope(call) #run row_table_type(sql, call, 1, #insert,scope(call) #run row_table_exists(sql, call, 1)), Row(#run row_shape(sql, call, 1)));
    result, error := run_statement(db, #run rewritten_sql(sql, call, 1), #run binary_results(sql, call, 1), args, plan, json, #run expected_columns(sql, call, 1));

    if error.kind != .NONE return value, false, error;

    if result.row_count == 0 {
        free_result(*result);
        return value, false, .{};
    }

    rows := Rows(type_of(value)).{ result = detach_to_temp(*result) };

    error = read_row(*rows, 0, *value);

    return value, error.kind == .NONE, error;
} @Interpolates("sql")

// Runs a statement and returns the number of rows it affected.
//
// `project_file` names the file whose project (migrations, query cache, `.env`) the statement
// belongs to, when it is not the file of the call: for code that writes statements for others.
//
// - db: a `*Connection` or a `*Pool`.
// - sql: the statement, written at the call as a string: `%` marks each argument, `%%` is a literal `%`.
// - args: one value per `%`, in order.
// - plan: `.AUTO` runs it as a cached named statement; `.CUSTOM` plans it again on every run.
// - json: decoding options for json columns in this query; they apply over the connection's and the column's.
// - project_file: the file whose project the statement belongs to; empty for the file of the call.
// - call: the code of the call, which lets the query text be checked while compiling. Leave the default.
execute :: (db: $D, $sql: string, args: ..Any, plan := Plan.AUTO, json := Json_Options.{}, $project_file := "", $call := #caller_code) -> s64, Pg_Error #expand {
    #run check_statement(sql, call, 1, false, project_file);

    result, error := run_statement(db, #run rewritten_sql(sql, call, 1, project_file), #run binary_results(sql, call, 1, project_file), args, plan, json, #run expected_columns(sql, call, 1, project_file));
    if error.kind != .NONE return 0, error;

    count := result.rows_affected;

    free_result(*result);

    return count, .{};
} @Interpolates("sql")

// `query` on a pipeline: the statement is queued, checked like `query`, and its rows come later
// from `take`. Queue several, then take them: they share one round trip.
//
// - p: the pipeline to queue on.
// - sql: the statement, as for `query`.
// - args: one value per `%`, in order.
// - call: the code of the call, which lets the query text be checked while compiling. Leave the default.
//
// ```jai,ignore
// p := Pg.pipeline_enter(c);
// users := Pg.queue(p, "select id, name from users where id = $1", id);
// count := Pg.queue_execute(p, "update users set seen = now() where id = $1", id);
// rows, error := Pg.take(p, users);
// changed, error = Pg.take(p, count);
// Pg.pipeline_exit(p);
// ```
//
// Not run: it needs a PostgreSQL server.
queue :: (p: *Pipeline, $sql: string, args: ..Any, $call := #caller_code) -> Pending(Rows(#run row_type(sql, call, 1, #insert,scope(call) #run row_table_type(sql, call, 1, #insert,scope(call) #run row_table_exists(sql, call, 1)), Row(#run row_shape(sql, call, 1))))), Pg_Error #expand {
    #run check_statement(sql, call, 1, true);

    pending: Pending(Rows(#run row_type(sql, call, 1, #insert,scope(call) #run row_table_type(sql, call, 1, #insert,scope(call) #run row_table_exists(sql, call, 1)), Row(#run row_shape(sql, call, 1)))));
    pending.serial = p.queued;
    error := queue_typed(p, #run rewritten_sql(sql, call, 1), "", args, ifx (#run binary_results(sql, call, 1)).binary then Format.BINARY else .TEXT);

    return pending, error;
} @Interpolates("sql")

// `execute` on a pipeline: `take` gives the count of rows the statement changed.
//
// - p: the pipeline to queue on.
// - sql: the statement, as for `execute`.
// - args: one value per `%`, in order.
// - call: the code of the call, which lets the query text be checked while compiling. Leave the default.
queue_execute :: (p: *Pipeline, $sql: string, args: ..Any, $call := #caller_code) -> Pending(s64), Pg_Error #expand {
    #run check_statement(sql, call, 1, false);

    pending: Pending(s64);
    pending.serial = p.queued;
    error := queue_typed(p, #run rewritten_sql(sql, call, 1), "", args, ifx (#run binary_results(sql, call, 1)).binary then Format.BINARY else .TEXT);

    return pending, error;
} @Interpolates("sql")

// `query` for results too big to hold: the rows arrive `batch` at a time and `for` decodes them
// one by one, so at most one batch is in memory. The statement is sent here, and the connection is
// busy until the loop ends; leaving the loop early cancels the rest. A failure, before or during the
// loop, lands in `error` (cleared first). Strings in `it` live until the next batch: copy what you
// keep.
//
// - c: the connection to run on; it is busy until the loop ends.
// - error: set to the failure, if any, once the loop ends.
// - sql: the statement, as for `query`.
// - args: one value per `%`, in order.
// - batch: how many rows arrive at a time.
// - call: the code of the call, which lets the query text be checked while compiling. Leave the default.
//
// ```jai,ignore
// error: Pg.Pg_Error;
// for Pg.stream(c, *error, "select id, body from events where day = %", day, batch = 5000) {
//     total += it.body.count;
// }
// if error.kind != .NONE ...
// ```
//
// Not run: it needs a PostgreSQL server.
stream :: (c: *Connection, error: *Pg_Error, $sql: string, args: ..Any, batch := 1000, $call := #caller_code) -> Row_Stream(#run row_type(sql, call, 2, #insert,scope(call) #run row_table_type(sql, call, 2, #insert,scope(call) #run row_table_exists(sql, call, 2)), Row(#run row_shape(sql, call, 2)))) #expand {
    #run check_statement(sql, call, 2, true);

    meta :: #run binary_results(sql, call, 2);
    out: Row_Stream(#run row_type(sql, call, 2, #insert,scope(call) #run row_table_type(sql, call, 2, #insert,scope(call) #run row_table_exists(sql, call, 2)), Row(#run row_shape(sql, call, 2))));

    error.* = .{};
    out.error = error;
    out.meta = meta;
    error.* = start_stream(c, *out.pipeline, #run rewritten_sql(sql, call, 2), args, ifx meta.binary then Format.BINARY else .TEXT, batch);

    return out;
} @Interpolates("sql")

Row_Stream :: struct(T: Type) {
    pipeline: *Pipeline; // null once the statement is over
    error: *Pg_Error;
    meta: Statement_Meta;
}

// Loops over the streamed rows, fetching a batch when the last one is used up.
//
// - stream: the stream to loop over.
// - body: the loop body.
// - flags: the loop's flags.
for_expansion :: (stream: *Row_Stream($T), body: Code, flags: For_Flags) #expand {
    rows: Rows(T);
    row := 0;
    finished := false;

    `it_index := -1;

    defer {
        free_result(*rows.result);
        end_stream(stream.pipeline, stream.error);
    }

    while stream.pipeline {
        if row >= rows.result.row_count {
            free_result(*rows.result);
            rows.result = .{};
            row = 0;

            if finished break;

            more, failure := next_batch(stream.pipeline, *rows.result);

            apply_statement_meta(*rows.result, stream.meta, .{});

            if failure.kind != .NONE {
                stream.error.* = failure;
                break;
            }

            finished = !more;

            continue;
        }

        `it: T;
        failure := read_row(*rows, row, *`it);

        if failure.kind != .NONE {
            stream.error.* = failure;
            break;
        }

        row += 1;
        `it_index += 1;

        #insert body;
    }
}

// The first column of the only row.
//
// - db: a `*Connection` or a `*Pool`.
// - sql: the statement, written at the call as a string: `%` marks each argument, `%%` is a literal `%`.
// - args: one value per `%`, in order.
// - plan: `.AUTO` runs it as a cached named statement; `.CUSTOM` plans it again on every run.
// - json: decoding options for json columns in this query; they apply over the connection's and the column's.
// - call: the code of the call, which lets the query text be checked while compiling. Leave the default.
query_scalar :: (db: $D, $sql: string, args: ..Any, plan := Plan.AUTO, json := Json_Options.{}, $call := #caller_code) -> type_of(Scalar(#run scalar_type(sql, call, 1)).value), Pg_Error #expand {
    #run check_statement(sql, call, 1, true);

    value: type_of(Scalar(#run scalar_type(sql, call, 1)).value);
    result, error := run_statement(db, #run rewritten_sql(sql, call, 1), #run binary_results(sql, call, 1), args, plan, json, #run expected_columns(sql, call, 1));

    if error.kind != .NONE return value, error;

    if result.row_count != 1 {
        count := result.row_count;
        free_result(*result);

        if count == 0 return value, make_error(.NOT_FOUND, "no rows");

        return value, make_error(.USAGE, "expected one row (the query returned % rows)", count);
    }

    detached := detach_to_temp(*result);
    got, read_error := get(*detached, 0, 0, type_of(value));

    return got, read_error;
} @Interpolates("sql")

// `query` into your own struct. The columns are matched to fields by name, and checked: a column
// that can be NULL needs a `Nullable` field, the types must be assignable, and a `Json(T)` field
// says which type a json column holds. `project_file` is as for `execute`.
//
// - T: the struct each row is read into.
// - db: a `*Connection` or a `*Pool`.
// - sql: the statement, written at the call as a string: `%` marks each argument, `%%` is a literal `%`.
// - args: one value per `%`, in order.
// - plan: `.AUTO` runs it as a cached named statement; `.CUSTOM` plans it again on every run.
// - json: decoding options for json columns in this query; they apply over the connection's and the column's.
// - project_file: the file whose project the statement belongs to; empty for the file of the call.
// - call: the code of the call, which lets the query text be checked while compiling. Leave the default.
query_as :: ($T: Type, db: $D, $sql: string, args: ..Any, plan := Plan.AUTO, json := Json_Options.{}, $project_file := "", $call := #caller_code) -> Rows(T), Pg_Error #expand {
    #run check_statement_as(T, sql, call, 2, project_file);

    rows: Rows(T);
    error: Pg_Error;

    #insert #run field_map_code(T, sql, call, 2, project_file);
    rows.result, error = run_statement(db, #run rewritten_sql(sql, call, 2, project_file), #run binary_results(sql, call, 2, project_file), args, plan, json, #run expected_columns(sql, call, 2, project_file));

    return rows, error;
} @Interpolates("sql")

// A one-field struct whose field has the type of a column: for `query_scalar`'s return type.
Scalar :: struct($type_text: string) {
    #insert tprint("value: %;", type_text);
}

#scope_module

run_statement :: (db: *Connection, sql: string, meta: Statement_Meta, args: [] Any, plan: Plan, json: Json_Options, expected: string) -> Result, Pg_Error {
    result, error := run_extended(db, sql, args, ifx meta.binary then Format.BINARY else .TEXT, plan == .AUTO, expected);

    apply_statement_meta(*result, meta, json);

    return result, error;
}

// What the description knows about a result's columns, for decode errors: where each comes from,
// and the query's own JSON options.
apply_statement_meta :: (result: *Result, meta: Statement_Meta, json: Json_Options) {
    result.json_query = json;

    rest := meta.origins;
    index := 0;

    while index < result.columns.count {
        newline := find_index_from_left(rest, #char "\n");

        result.columns[index].origin = ifx newline < 0 then rest else slice(rest, 0, newline);
        index += 1;

        if newline < 0 break;

        rest = advance(rest, newline + 1);
    }
}

run_statement :: (pool: *Pool, sql: string, meta: Statement_Meta, args: [] Any, plan: Plan, json: Json_Options, expected: string) -> Result, Pg_Error {
    c, acquire_error := acquire(pool);
    if acquire_error.kind != .NONE return .{}, acquire_error;

    result, error := run_statement(c, sql, meta, args, plan, json, expected);

    release(pool, c);

    return result, error;
}

// Enters pipeline mode, queues the statement and its sync point, and asks for its rows in batches.
start_stream :: (c: *Connection, pipeline: **Pipeline, sql: string, args: [] Any, result_format: Format, batch: s64) -> Pg_Error {
    if batch < 1 return make_error(.USAGE, "batch must be at least 1, not %", batch);
    if c && c.pipeline.active return make_error(.USAGE, "the connection is already in pipeline mode");

    p, error := pipeline_enter(c);
    if error.kind != .NONE return error;

    error = queue_typed(p, sql, "", args, result_format);
    if error.kind == .NONE error = pipeline_sync(p);
    if error.kind == .NONE && !pipeline_chunked_rows(p, batch) error = make_error(.USAGE, "the statement's rows could not be asked for in batches");

    if error.kind != .NONE {
        pipeline_abandon(p);
        return error;
    }

    pipeline.* = p;

    return .{};
}

// The next rows of a stream into `out`; false once the statement is over (`out` may still hold its
// last rows).
next_batch :: (p: *Pipeline, out: *Result) -> bool, Pg_Error {
    item, error := pipeline_next_result(p);
    if error.kind != .NONE return false, error;

    if item.kind == .ROWS || item.kind == .RESULT {
        out.* = item.result;
        array_free(item.parameter_types);

        return item.kind == .ROWS, .{};
    }

    if item.kind == .ERROR {
        failure := item.error;
        item.error = .{};
        free_pipeline_result(*item);

        return false, failure;
    }

    free_pipeline_result(*item);

    return false, make_error(.PROTOCOL, "the stream's statement ended without a result");
}

// After the loop: reads the sync point and leaves pipeline mode, or, when the loop stopped early,
// cancels what is left.
end_stream :: (p: *Pipeline, error: *Pg_Error) {
    if !p return;

    if p.head < p.steps.count && p.steps[p.head] == .SYNC && !p.chunk_rows {
        item, failure := pipeline_next_result(p);
        free_pipeline_result(*item);

        if failure.kind == .NONE && pipeline_exit(p).kind == .NONE return;
        if error.kind == .NONE error.* = failure;
    }

    pipeline_abandon(p);
}

// A result moved into temporary storage, so a value read from it outlives the result's owner.
detach_to_temp :: (result: *Result) -> Result {
    detached := result.*;

    detached.columns = NewArray(result.columns.count, Column,, temp);

    for result.columns {
        detached.columns[it_index] = it;
        detached.columns[it_index].name = copy_string(it.name,, temp);
    }

    detached.command = copy_string(result.command,, temp);
    detached.values = .{};
    detached.data = .{};
    detached.allocations = .{};
    detached.allocator = temp;

    array_resize(*detached.values, result.values.count,, temp);
    array_resize(*detached.data, result.data.count,, temp);
    if result.values.count memcpy(detached.values.data, result.values.data, result.values.count * size_of(Value_Span));
    if result.data.count memcpy(detached.data.data, result.data.data, result.data.count);

    free_result(result);

    return detached;
}

// Decodes row `index` into `out`: field i comes from column `map[i]`, or column i without a map.
read_row :: (rows: *Rows($T), index: s64, out: *T) -> Pg_Error {
    #insert #run read_row_code(T);

    return .{};
}

read_row_code :: (T: Type) -> string {
    builder: String_Builder;
    builder.allocator = temp;

    info := cast(*Type_Info_Struct) type_info(T);
    field := 0;

    for info.members {
        if (it.flags & .CONSTANT) || (it.flags & .USING) continue;

        print_to_builder(*builder, "{\n    value, error := get(*rows.result, index, ifx rows.map.count then rows.map[%] else %, type_of(out.%));\n    if error.kind != .NONE return error;\n    out.% = value;\n}\n", field, field, it.name, it.name);
        field += 1;
    }

    return builder_to_string(*builder);
}
