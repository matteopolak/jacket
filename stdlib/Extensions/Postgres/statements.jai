// Named prepared statements, cached per connection: a typed query is parsed by the server once per
// connection, and later runs send Bind and Execute only.

// Whether a connection keeps prepared statements (`Config.statement_cache`).
Statement_Cache_Mode :: enum u8 {
    ON;
    OFF; // unnamed statements only, for poolers that do not track named ones
}

// How the server plans a typed query (`plan = ` on `query`, `execute`, ...).
Plan :: enum u8 {
    AUTO; // a named statement: the server may switch to a generic plan after five runs
    CUSTOM; // the unnamed statement, planned again with the actual values on every run
}

// Runs one statement like `query_dynamic`, prepared under a name and kept in the connection's cache
// (unless `plan` is `.CUSTOM` or the cache is off): the first run parses it, later runs with the
// same text and parameter types only bind. Typed queries do this on their own.
query_cached :: (c: *Connection, sql: string, args: ..Any, plan := Plan.AUTO, binary := false) -> Result, Pg_Error {
    values := args; // `sql` is not a print format: keep it and `args` out of one call
    result, error := run_extended(c, sql, values, ifx binary then Format.BINARY else .TEXT, plan == .AUTO);

    return result, error;
}

// How many statements the connection has prepared on the server and remembers.
statement_cache_count :: (c: *Connection) -> s64 {
    return c.statements.entries.count;
}

// The server name of the statement a query with this text and these parameter types is prepared
// as (`j` and 16 hex digits), for looking it up in `pg_prepared_statements`.
statement_name :: (sql: string, parameter_types: [] u32 = .[]) -> string {
    return join("j", id_hex(cache_id(sql, parameter_types)),, temp);
}

#scope_module

Cached_Statement :: struct {
    id: u64;
    name: string;
    sql: string;
    types: [] u32;
    ready: bool; // the server has it and `columns` are known
    existing: bool; // the server may have it: send Close before the next Parse
    returns_rows: bool;
    columns: [] Column;
    used: s64; // `Statement_Cache.tick` at its last use
}

Statement_Cache :: struct {
    entries: [..] *Cached_Statement;
    tick: s64;
    mode: Statement_Cache_Mode;
    capacity: s64;
    verify: bool;
}

// What one request does with the cache.
Statement_Use :: struct {
    active: bool; // the request runs a named statement
    hit: bool; // the statement was ready: no Parse
    entry: *Cached_Statement;
    parsed: bool; // ParseComplete arrived
    described: bool; // the statement's columns arrived
}

cache_id :: (sql: string, types: [] u32) -> u64 {
    id := statement_id(sql);
    for types id = (id ^ it) * 0x100000001b3;

    return id;
}

free_cached_statement :: (entry: *Cached_Statement) {
    free(entry.name);
    free(entry.sql);
    array_free(entry.types);
    for entry.columns free(it.name);

    array_free(entry.columns);
    free(entry);
}

free_statement_cache :: (cache: *Statement_Cache) {
    for cache.entries free_cached_statement(it);

    array_free(cache.entries);
    cache.entries = .{};
}

statements_enabled :: (c: *Connection) -> bool {
    return c.statements.mode == .ON && c.statements.capacity > 0;
}

forget_statement :: (cache: *Statement_Cache, entry: *Cached_Statement) {
    for cache.entries {
        if it == entry {
            array_unordered_remove_by_index(*cache.entries, it_index);
            break;
        }
    }

    free_cached_statement(entry);
}

// Adds a statement that is not on the server yet. When the cache is full the least recently used
// one goes first: its Close is queued now, ahead of the Parse that follows.
admit_statement :: (c: *Connection, id: u64, sql: string, types: [] u32) -> *Cached_Statement {
    cache := *c.statements;

    while cache.entries.count >= cache.capacity {
        oldest: *Cached_Statement;
        for cache.entries if !oldest || it.used < oldest.used oldest = it;

        if oldest.existing put_close(c, #char "S", oldest.name);
        forget_statement(cache, oldest);
    }

    entry := New(Cached_Statement);
    entry.id = id;
    entry.name = join("j", id_hex(id));
    entry.sql = copy_string(sql);
    entry.types = array_copy(types);
    array_add(*cache.entries, entry);

    return entry;
}

// Queues what a request needs to run `sql` as a named statement, up to Execute: Bind alone for a
// statement the server has, else Parse and Describe first. Returns false (nothing queued) when
// the request has to use the unnamed statement: a different text has the same id.
begin_cached :: (c: *Connection, sql: string, types: [] u32, params: [] Param, result_format: Format, expected: string, use: *Statement_Use) -> Pg_Error {
    cache := *c.statements;
    id := cache_id(sql, types);
    entry: *Cached_Statement;

    for cache.entries {
        if it.id != id continue;

        if it.sql != sql || !same_types(it.types, types) return .{};

        entry = it;
        break;
    }

    if !entry entry = admit_statement(c, id, sql, types);

    cache.tick += 1;
    entry.used = cache.tick;
    use.active = true;
    use.entry = entry;

    if !entry.ready && cache.verify && expected {
        error := prepare_checked(c, entry, expected);
        if error.kind != .NONE {
            use.active = false;
            return error;
        }
    }

    if entry.ready {
        use.hit = true;
    } else {
        // A statement the server may still hold under this name is replaced.
        if entry.existing put_close(c, #char "S", entry.name);

        put_parse(c, entry.name, sql, types);
        put_describe(c, #char "S", entry.name);
        entry.existing = true;
    }

    put_bind(c, "", entry.name, params, result_format);
    put_execute(c, "", 0);

    return .{};
}

same_types :: (a: [] u32, b: [] u32) -> bool {
    if a.count != b.count return false;
    for a if it != b[it_index] return false;

    return true;
}

// Parses and describes a statement in a round trip of its own, and compares its columns with the
// ones the query was compiled against, so a query never runs against a changed schema.
prepare_checked :: (c: *Connection, entry: *Cached_Statement, expected: string) -> Pg_Error {
    if entry.existing put_close(c, #char "S", entry.name);

    put_parse(c, entry.name, entry.sql, entry.types);
    put_describe(c, #char "S", entry.name);
    put_sync(c);
    entry.existing = true;

    error := send(c);
    if error.kind != .NONE return error;

    description: Result;
    failure: Pg_Error;
    parsed := false;
    described := false;

    while true {
        type, body, read_error := read_message(c);
        if read_error.kind != .NONE {
            free_result(*description);
            recover_interrupted(c, read_error);
            return error_before_close(*failure, read_error);
        }

        if handle_async(c, type, body) continue;

        if type == {
            case #char "1";
                parsed = true;

            case #char "3"; #through;
            case #char "t";

            case #char "n";
                described = true;

            case #char "T";
                error = read_columns(c, *description, body);
                if error.kind != .NONE {
                    free_result(*description);
                    return error;
                }

                described = true;
                description.returns_rows = true;

            case #char "E";
                if failure.kind == .NONE failure = parse_server_error(body);

            case #char "Z";
                c.status = transaction_status(ifx body.count then body[0] else 0);

                if failure.kind != .NONE || !parsed || !described {
                    free_result(*description);
                    if !parsed forget_statement(*c.statements, entry);

                    return failure;
                }

                problem := compare_columns(description.columns, expected);

                if problem {
                    free_result(*description);
                    return make_error(.USAGE, "statement % was compiled against a different schema: %", entry.name, problem);
                }

                keep_description(entry, *description);

                return .{};

            case;
                free_error(*failure);
                free_result(*description);

                return protocol_error(c, tprint("unexpected message % while preparing a statement", cast(string) u8.[type]));
        }
    }

    return .{};
}

// `expected` is the number of columns, then each name, one per line (see `expected_columns`). A
// column the query renamed with an alias suffix (`name!`) matches by its start.
compare_columns :: (columns: [] Column, expected: string) -> string {
    found, count_text, names := split_from_left(expected, #char "\n");
    count, valid := string_to_int(ifx found then count_text else expected);

    if !valid return "";

    if count != columns.count {
        return tprint("it has % column% now, and the query expects %", columns.count, ifx columns.count == 1 then "" else "s", count);
    }

    rest := names;

    for columns {
        more, name, after := split_from_left(rest, #char "\n");
        if !more name = rest;

        rest = after;

        if !begins_with(it.name, name) {
            return tprint("column % is \"%\" now, and the query expects \"%\"", it_index + 1, it.name, name);
        }
    }

    return "";
}

// Moves the columns a Describe returned into the cached statement, in the format the results use.
keep_description :: (entry: *Cached_Statement, description: *Result) {
    for entry.columns free(it.name);

    array_free(entry.columns);

    entry.columns = description.columns;
    entry.returns_rows = description.returns_rows;
    entry.ready = true;
    description.columns = .[];

    free_result(description);
}

// The columns a hit knows without asking the server, in the format of this request.
cached_columns :: (entry: *Cached_Statement, format: Format) -> [] Column {
    columns := array_copy(entry.columns);

    for *columns {
        it.name = copy_string(it.name);
        it.format = format;
    }

    return columns;
}

// The end of a request that used a named statement. Records what the server told us, and says
// whether the request should run again: a statement that went stale or is gone is dropped, and
// the retry prepares it afresh.
finish_cached :: (c: *Connection, use: *Statement_Use, result: *Result, failure: Pg_Error, may_retry: bool) -> bool {
    entry := use.entry;

    if !use.hit {
        if !use.parsed {
            forget_statement(*c.statements, entry);
            return false;
        }

        if use.described {
            for entry.columns free(it.name);

            array_free(entry.columns);

            entry.columns = array_copy(result.columns);
            for *entry.columns it.name = copy_string(it.name);

            entry.returns_rows = result.returns_rows;
            entry.ready = true;
        }
    }

    if failure.kind != .SERVER return false;

    stale := failure.code == .FEATURE_NOT_SUPPORTED && contains(failure.message, "cached plan must not change result type");
    gone := failure.code == .INVALID_SQL_STATEMENT_NAME;

    if !stale && !gone return false;

    // Close and Parse again on the next run. Inside a transaction the error has aborted it: the
    // caller sees the error, and the statement is fresh for the next transaction.
    entry.ready = false;
    entry.existing = true;

    return may_retry && use.hit && c.status != .FAILED;
}
