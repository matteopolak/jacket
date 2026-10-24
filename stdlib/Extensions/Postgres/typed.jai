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
//     users := Pg.fetch_all(*rows);   // or Pg.fetch_all(*rows, temp)
//     Pg.free_rows(*rows);
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

free_rows :: (rows: *Rows($T)) {
    free_result(*rows.result);
    free_error(*rows.error);
    rows.* = .{};
}

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
query :: (db: $D, $sql: string, args: ..Any, plan := Plan.AUTO, $call := #caller_code) -> Rows(#run row_type(sql, call, 1, #insert,scope(call) #run row_table_type(sql, call, 1, #insert,scope(call) #run row_table_exists(sql, call, 1)), Row(#run row_shape(sql, call, 1)))), Pg_Error #expand {
    #run check_statement(sql, call, 1, true);

    rows: Rows(#run row_type(sql, call, 1, #insert,scope(call) #run row_table_type(sql, call, 1, #insert,scope(call) #run row_table_exists(sql, call, 1)), Row(#run row_shape(sql, call, 1))));
    error: Pg_Error;
    rows.result, error = run_statement(db, #run rewritten_sql(sql, call, 1), #run binary_results(sql, call, 1), args, plan, #run expected_columns(sql, call, 1));

    return rows, error;
} @Interpolates("sql")

// The only row, or a `.NOT_FOUND` error when there is none. More rows are a `.USAGE` error. The row
// lives in temporary storage (until `reset_temporary_storage`).
query_one :: (db: $D, $sql: string, args: ..Any, plan := Plan.AUTO, $call := #caller_code) -> #run row_type(sql, call, 1, #insert,scope(call) #run row_table_type(sql, call, 1, #insert,scope(call) #run row_table_exists(sql, call, 1)), Row(#run row_shape(sql, call, 1))), Pg_Error #expand {
    #run check_statement(sql, call, 1, true);

    value: #run row_type(sql, call, 1, #insert,scope(call) #run row_table_type(sql, call, 1, #insert,scope(call) #run row_table_exists(sql, call, 1)), Row(#run row_shape(sql, call, 1)));
    result, error := run_statement(db, #run rewritten_sql(sql, call, 1), #run binary_results(sql, call, 1), args, plan, #run expected_columns(sql, call, 1));

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
query_opt :: (db: $D, $sql: string, args: ..Any, plan := Plan.AUTO, $call := #caller_code) -> #run row_type(sql, call, 1, #insert,scope(call) #run row_table_type(sql, call, 1, #insert,scope(call) #run row_table_exists(sql, call, 1)), Row(#run row_shape(sql, call, 1))), bool, Pg_Error #expand {
    #run check_statement(sql, call, 1, true);

    value: #run row_type(sql, call, 1, #insert,scope(call) #run row_table_type(sql, call, 1, #insert,scope(call) #run row_table_exists(sql, call, 1)), Row(#run row_shape(sql, call, 1)));
    result, error := run_statement(db, #run rewritten_sql(sql, call, 1), #run binary_results(sql, call, 1), args, plan, #run expected_columns(sql, call, 1));

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
execute :: (db: $D, $sql: string, args: ..Any, plan := Plan.AUTO, $call := #caller_code) -> s64, Pg_Error #expand {
    #run check_statement(sql, call, 1, false);

    result, error := run_statement(db, #run rewritten_sql(sql, call, 1), #run binary_results(sql, call, 1), args, plan, #run expected_columns(sql, call, 1));
    if error.kind != .NONE return 0, error;

    count := result.rows_affected;

    free_result(*result);

    return count, .{};
} @Interpolates("sql")

// `query` on a pipeline: the statement is queued, checked like `query`, and its rows come later
// from `take`. Queue several, then take them: they share one round trip.
//
//     p := Pg.pipeline_enter(c);
//     users := Pg.queue(p, "select id, name from users where id = $1", id);
//     count := Pg.queue_execute(p, "update users set seen = now() where id = $1", id);
//     rows, error := Pg.take(p, users);
//     changed, error = Pg.take(p, count);
//     Pg.pipeline_exit(p);
queue :: (p: *Pipeline, $sql: string, args: ..Any, $call := #caller_code) -> Pending(Rows(#run row_type(sql, call, 1, #insert,scope(call) #run row_table_type(sql, call, 1, #insert,scope(call) #run row_table_exists(sql, call, 1)), Row(#run row_shape(sql, call, 1))))), Pg_Error #expand {
    #run check_statement(sql, call, 1, true);

    pending: Pending(Rows(#run row_type(sql, call, 1, #insert,scope(call) #run row_table_type(sql, call, 1, #insert,scope(call) #run row_table_exists(sql, call, 1)), Row(#run row_shape(sql, call, 1)))));
    pending.serial = p.queued;
    error := queue_typed(p, #run rewritten_sql(sql, call, 1), "", args, ifx #run binary_results(sql, call, 1) then Format.BINARY else .TEXT);

    return pending, error;
} @Interpolates("sql")

// `execute` on a pipeline: `take` gives the count of rows the statement changed.
queue_execute :: (p: *Pipeline, $sql: string, args: ..Any, $call := #caller_code) -> Pending(s64), Pg_Error #expand {
    #run check_statement(sql, call, 1, false);

    pending: Pending(s64);
    pending.serial = p.queued;
    error := queue_typed(p, #run rewritten_sql(sql, call, 1), "", args, ifx #run binary_results(sql, call, 1) then Format.BINARY else .TEXT);

    return pending, error;
} @Interpolates("sql")

// The first column of the only row.
query_scalar :: (db: $D, $sql: string, args: ..Any, plan := Plan.AUTO, $call := #caller_code) -> type_of(Scalar(#run scalar_type(sql, call, 1)).value), Pg_Error #expand {
    #run check_statement(sql, call, 1, true);

    value: type_of(Scalar(#run scalar_type(sql, call, 1)).value);
    result, error := run_statement(db, #run rewritten_sql(sql, call, 1), #run binary_results(sql, call, 1), args, plan, #run expected_columns(sql, call, 1));

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
// says which type a json column holds.
query_as :: ($T: Type, db: $D, $sql: string, args: ..Any, plan := Plan.AUTO, $call := #caller_code) -> Rows(T), Pg_Error #expand {
    #run check_statement_as(T, sql, call, 2);

    rows: Rows(T);
    error: Pg_Error;

    #insert #run field_map_code(T, sql, call, 2);
    rows.result, error = run_statement(db, #run rewritten_sql(sql, call, 2), #run binary_results(sql, call, 2), args, plan, #run expected_columns(sql, call, 2));

    return rows, error;
} @Interpolates("sql")

// A one-field struct whose field has the type of a column: for `query_scalar`'s return type.
Scalar :: struct($type_text: string) {
    #insert tprint("value: %;", type_text);
}

#scope_module

run_statement :: (db: *Connection, sql: string, binary: bool, args: [] Any, plan: Plan, expected: string) -> Result, Pg_Error {
    result, error := run_extended(db, sql, args, ifx binary then Format.BINARY else .TEXT, plan == .AUTO, expected);

    return result, error;
}

run_statement :: (pool: *Pool, sql: string, binary: bool, args: [] Any, plan: Plan, expected: string) -> Result, Pg_Error {
    c, acquire_error := acquire(pool);
    if acquire_error.kind != .NONE return .{}, acquire_error;

    result, error := run_statement(c, sql, binary, args, plan, expected);

    release(pool, c);

    return result, error;
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
