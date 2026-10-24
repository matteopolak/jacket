// Describing a statement against a live server: its parameter and column types, which columns can
// be NULL, and which tables, views, types and functions it depends on. This is the work of the
// describe service (describe_service.jai) and of the pg tool; the macros only read the answers.
//
// Requests and answers are JSON (`Json_Value`), so the service protocol can grow fields without
// breaking an older client. See docs/postgres/typed-queries.md.

Describe_Request :: struct {
    op: string; // "describe", "fingerprints" or "ping"
    url: string; // the database; empty with `managed`: the service's own managed server
    managed: bool; // no URL: describe in the managed server (needs `migrations`; see managed.jai)
    managed_version: string; // which release the managed server runs; empty: the default
    search_path: string; // set before describing; empty: the server's own
    sql: string; // the rewritten statement (`$1`, `$2`, ...)
    schema_hash: string; // of the migrations; copied into the entry
    keys: [] string; // "fingerprints": the dependency keys to fingerprint
    migrations: string; // the migrations folder: answer from its schema database
    setup: string; // set by the service: SQL run first inside the describe's transaction
    setup_marks: [] Setup_Mark; // set by the service: where each migration lies in `setup`
    stale: Migration_Failure; // set by the service: the schema is the one before this failed migration
    min_server_version: s64; // the oldest server major version the project supports; 0: any
}

// The complaint for a server older than the request allows, or an empty string.
server_too_old :: (db: *Connection, request: Describe_Request) -> string {
    major := db.server_version / VERSION_MAJOR;
    if major >= request.min_server_version return "";

    return tprint("the server is PostgreSQL %, but this project needs at least % (MIN_SERVER_VERSION)", major, request.min_server_version);
}

describe_request_json :: (request: Describe_Request) -> string {
    return Json_Module.to_json(request);
}

// Describes `request.sql` on `db` and returns the answer as JSON text:
//   {"kind":"ok","entry":{...}}                  the cache entry (see entry_to_json)
//   {"kind":"sql_error","error":{...}}           the server refused the statement
//   {"kind":"error","message":"..."}             anything else (the connection died, say)
//
// "ok" and "sql_error" carry `"stale_schema": {"name","file","line","column","message"}` when a
// migration failed and the statement was described against the schema before it. A failure with no
// schema to fall back on is `{"kind":"migration_error","migration":{...}}` (see migration_error_answer).
describe_answer :: (db: *Connection, request: Describe_Request) -> string {
    answer := describe_value(db, request);
    text := Json_Module.to_json(answer);
    Json_Module.free_json(*answer);

    return text;
}

// The answer to a `fingerprints` request: {"kind":"ok","server_major":17,"fingerprints":{key: hash}}.
fingerprints_answer :: (db: *Connection, request: Describe_Request) -> string {
    answer := json_object();
    too_old := server_too_old(db, request);

    if too_old {
        json_set(*answer, "kind", json_string("error"));
        json_set(*answer, "message", json_string(too_old));

        return Json_Module.to_json(answer);
    }

    error := apply_search_path(db, request.search_path);

    if error.kind != .NONE {
        json_set(*answer, "kind", json_string("error"));
        json_set(*answer, "message", json_string(error_text(error)));
        free_error(*error);

        return Json_Module.to_json(answer);
    }

    if request.stale.name return Json_Module.to_json(migration_error_answer(request.stale, request.migrations));

    if request.setup {
        stale := request.stale;
        used_setup: string;

        error = begin_with_setup(db, request, *stale, *used_setup);

        if error.kind != .NONE {
            json_set(*answer, "kind", json_string("error"));
            json_set(*answer, "message", json_string(error_text(error)));
            free_error(*error);

            return Json_Module.to_json(answer);
        }

        // Fingerprints of a schema that is not the project's would pass a stale entry as good.
        if stale.name {
            rolled_back := execute_script(db, "rollback");
            free_error(*rolled_back);

            return Json_Module.to_json(migration_error_answer(stale, request.migrations));
        }
    }

    prints, fail := fingerprints(db, request.keys);

    if request.setup {
        rolled_back := execute_script(db, "rollback");
        free_error(*rolled_back);
    }

    if fail.kind != .NONE {
        json_set(*answer, "kind", json_string("error"));
        json_set(*answer, "message", json_string(error_text(fail)));
        free_error(*fail);

        return Json_Module.to_json(answer);
    }

    json_set(*answer, "kind", json_string("ok"));
    json_set(*answer, "server_major", json_int(db.server_version / VERSION_MAJOR));

    object := json_object();
    for request.keys json_set(*object, it, json_string(prints[it_index]));

    json_set(*answer, "fingerprints", object);

    return Json_Module.to_json(answer);
}

// The answer for migrations that do not build and leave no earlier schema to describe against.
migration_error_text :: (failure: Migration_Failure, folder: string) -> string {
    answer := migration_error_answer(failure, folder);
    text := Json_Module.to_json(answer);
    Json_Module.free_json(*answer);

    return text;
}

#scope_module

describe_value :: (db: *Connection, request: Describe_Request) -> Json_Value {
    answer := json_object();

    too_old := server_too_old(db, request);

    if too_old return failure_answer(make_error(.USAGE, "%", too_old));

    error := apply_search_path(db, request.search_path);

    if error.kind != .NONE return failure_answer(error);

    stale := request.stale;
    used_setup: string;

    error = begin_with_setup(db, request, *stale, *used_setup);
    if error.kind != .NONE return failure_answer(error);

    info, describe_error := describe_statement(db, request.sql);

    if describe_error.kind != .NONE {
        rolled_back := execute_script(db, "rollback; deallocate all");

        free_error(*rolled_back);

        if describe_error.kind == .SERVER {
            refused := sql_error_answer(db, request.sql, describe_error);
            add_stale(*refused, stale, request.migrations);

            return refused;
        }

        return failure_answer(describe_error);
    }

    entry := Query_Entry.{ format = ENTRY_FORMAT, sql = request.sql, schema_hash = request.schema_hash, search_path = request.search_path };
    entry.server_major = db.server_version / VERSION_MAJOR;

    types, type_error := resolve_types(db, info);
    if type_error.kind != .NONE return abandon(db, type_error);

    for info.parameter_types array_add(*entry.params, types_entry(*types, it));

    attributes, attribute_error := load_attributes(db, info.columns);
    if attribute_error.kind != .NONE return abandon(db, attribute_error);

    plan := explain_plan(db, request.sql, info.parameter_types.count, info.columns.count);

    for column, index: info.columns {
        entry_column := column_entry(*types, *attributes, *plan, column, index);
        array_add(*entry.columns, entry_column);
    }

    rolled_back := execute_script(db, "rollback; deallocate all");

    free_error(*rolled_back);

    dependencies(db, request.sql, used_setup, info, *types, *entry);

    free_statement_info(*info);

    json_set(*answer, "kind", json_string("ok"));
    json_set(*answer, "entry", entry_to_json(entry));
    add_stale(*answer, stale, request.migrations);

    return answer;
}

// Opens the describe's transaction and runs the migrations newer than the schema database in it
// (`request.setup`). When one of them fails, the answer is still given, from the schema before it:
// the transaction is begun again with the migrations that came earlier, `stale` says which one
// failed and where, and `used_setup` is the part of the setup that ran. Any other failure is
// returned with the transaction rolled back.
begin_with_setup :: (db: *Connection, request: Describe_Request, stale: *Migration_Failure, used_setup: *string) -> Pg_Error {
    used_setup.* = request.setup;

    error := execute_script(db, "begin");
    if error.kind != .NONE return error;
    if !request.setup return .{};

    marks := request.setup_marks;

    if !marks.count {
        error = execute_script(db, request.setup);
        if error.kind == .NONE return .{};

        message := tprint("a migration newer than the schema database failed: %", error_text(error));

        free_error(*error);
        abandoned := execute_script(db, "rollback");

        free_error(*abandoned);

        return make_error(.USAGE, "%", message);
    }

    failed := -1;

    for marks {
        part := slice(request.setup, it.start, it.length);

        error = execute_script(db, part);
        if error.kind == .NONE continue;

        failed = it_index;
        stale.* = migration_failure(it.name, it.index, part, part, error);
        stale.file = migration_file(request.migrations, it.name);

        break;
    }

    if failed < 0 return .{};

    message := tprint("a migration newer than the schema database failed: %", error_text(error));
    from_server := error.kind == .SERVER;
    original := copy_string(error.message,, temp);

    free_error(*error);
    abandoned := execute_script(db, "rollback");

    free_error(*abandoned);

    if !from_server return make_error(.USAGE, "%", message);

    // Again, without the failed migration and the ones after it.
    error = execute_script(db, "begin");
    if error.kind != .NONE return error;

    for 0..failed - 1 {
        error = execute_script(db, slice(request.setup, marks[it].start, marks[it].length));

        if error.kind != .NONE {
            abandoned = execute_script(db, "rollback");
            free_error(*abandoned);

            return error;
        }
    }

    // The failure has no position: find the statement, from the state the migration started in.
    if stale.line == 0 {
        part := slice(request.setup, marks[failed].start, marks[failed].length);
        stale.* = locate_failing_statement(db, stale.name, stale.index, part, original, stale.*);
        stale.file = migration_file(request.migrations, stale.name);
    }

    used_setup.* = slice(request.setup, 0, marks[failed].start);

    return .{};
}

migration_file :: (folder: string, name: string) -> string {
    if !folder return tprint("%.sql", name);

    return join(folder, "/", name, ".sql",, temp);
}

failure_json :: (failure: Migration_Failure, folder: string) -> Json_Value {
    object := json_object();
    file := ifx failure.file then failure.file else migration_file(folder, failure.name);

    json_set(*object, "name", json_string(failure.name));
    json_set(*object, "file", json_string(file));
    json_set(*object, "line", json_int(failure.line));
    json_set(*object, "column", json_int(failure.column));
    json_set(*object, "message", json_string(failure.message));

    return object;
}

// No schema to describe against: the migrations themselves do not build.
migration_error_answer :: (failure: Migration_Failure, folder: string) -> Json_Value {
    answer := json_object();

    json_set(*answer, "kind", json_string("migration_error"));
    json_set(*answer, "migration", failure_json(failure, folder));

    return answer;
}

// Marks an answer that was given from the schema before a failed migration.
add_stale :: (answer: *Json_Value, stale: Migration_Failure, folder: string) {
    if !stale.name return;

    json_set(answer, "stale_schema", failure_json(stale, folder));
}

abandon :: (db: *Connection, error: Pg_Error) -> Json_Value {
    rolled_back := execute_script(db, "rollback; deallocate all");
    free_error(*rolled_back);

    return failure_answer(error);
}

failure_answer :: (error: Pg_Error) -> Json_Value {
    answer := json_object();
    json_set(*answer, "kind", json_string("error"));
    json_set(*answer, "message", json_string(error_text(error)));

    return answer;
}

// Only names and commas: a search path comes from the environment, and is not SQL.
apply_search_path :: (db: *Connection, search_path: string) -> Pg_Error {
    if !search_path return execute_script(db, "reset search_path");

    for search_path {
        if !(scan_is_word(it) || it == #char "," || it == #char " " || it == #char "\"") {
            return make_error(.CONFIG, "search_path \"%\" may hold only names and commas", search_path);
        }
    }

    return execute_script(db, tprint("set search_path to %", search_path));
}

// What the server said about a rejected statement, with a suggestion for a misspelt name.
sql_error_answer :: (db: *Connection, sql: string, error: Pg_Error) -> Json_Value {
    answer := json_object();
    details := json_object();

    json_set(*details, "code", json_string(sqlstate_text(error.code)));
    json_set(*details, "message", json_string(error.message));
    if error.detail json_set(*details, "detail", json_string(error.detail));
    if error.hint json_set(*details, "hint", json_string(error.hint));
    if error.position > 0 json_set(*details, "position", json_int(error.position));

    json_set(*details, "byte", json_int(byte_of_character(sql, error.position)));

    suggestion := suggest_name(db, error);
    if suggestion json_set(*details, "suggestion", json_string(suggestion));

    json_set(*answer, "kind", json_string("sql_error"));
    json_set(*answer, "error", details);

    return answer;
}

// For an unknown table or column: the catalog name closest to the one in the message.
suggest_name :: (db: *Connection, error: Pg_Error) -> string {
    query: string;

    if error.code == .UNDEFINED_TABLE {
        query = "select distinct relname::text from pg_class where relkind in ('r', 'v', 'm', 'p', 'f') and pg_table_is_visible(oid)";
    } else if error.code == .UNDEFINED_COLUMN {
        query = "select distinct a.attname::text from pg_attribute a join pg_class c on c.oid = a.attrelid where a.attnum > 0 and not a.attisdropped and c.relkind in ('r', 'v', 'm', 'p', 'f') and pg_table_is_visible(c.oid)";
    } else {
        return "";
    }

    wanted := quoted_name(error.message);
    if !wanted return "";

    // `column "u.emal"`: only the last part is a name.
    dot := find_index_from_right(wanted, #char ".");
    if dot >= 0 wanted = advance(wanted, dot + 1);

    names, query_error := query_dynamic(db, query);

    if query_error.kind != .NONE {
        free_error(*query_error);
        return "";
    }

    best: string;
    best_distance := S64_MAX;

    for row: 0..names.row_count - 1 {
        name, _ := get(*names, row, 0, string);
        distance := edit_distance(wanted, name);

        if distance < best_distance {
            best_distance = distance;
            best = name;
        }
    }

    // Close enough to be a typo, not a different word.
    if best_distance > max(2, wanted.count / 3) best = "";

    result := copy_string(best,, temp);

    free_result(*names);

    return result;
}

// The first "quoted" part of a server message.
quoted_name :: (message: string) -> string {
    start := find_index_from_left(message, #char "\"");
    if start < 0 return "";

    rest := advance(message, start + 1);
    end := find_index_from_left(rest, #char "\"");

    if end < 0 return "";

    return slice(rest, 0, end);
}

lower_byte :: (c: u8) -> u8 {
    return ifx c >= #char "A" && c <= #char "Z" then c + 32 else c;
}

edit_distance :: (a: string, b: string) -> s64 {
    if !a.count return b.count;
    if !b.count return a.count;

    previous := NewArray(b.count + 1, s64,, temp);
    current := NewArray(b.count + 1, s64,, temp);

    for 0..b.count previous[it] = it;

    for i: 1..a.count {
        current[0] = i;

        for j: 1..b.count {
            cost := ifx lower_byte(a[i - 1]) == lower_byte(b[j - 1]) then 0 else 1;
            current[j] = min(min(previous[j] + 1, current[j - 1] + 1), previous[j - 1] + cost);
        }

        previous, current = current, previous;
    }

    return previous[b.count];
}

Described_Type :: struct {
    oid: u32;
    name: string; // pg_type.typname
    kind: u8; // typtype: 'b' base, 'd' domain, 'e' enum, 'c' composite, 'r' range, 'p' pseudo...
    category: u8; // typcategory: 'A' for arrays
    base_oid: u32; // a domain's base type
    element_oid: u32; // an array's element type
    shown: string; // format_type
    qualified: string; // "schema"."name", for dependency keys
    builtin: bool; // lives in pg_catalog
}

Described_Types :: struct {
    rows: [..] Described_Type;
}

// The names and shapes of every type the statement mentions, with the base types of its domains
// and the elements of its arrays.
resolve_types :: (db: *Connection, info: Statement_Info) -> Described_Types, Pg_Error {
    types: Described_Types;
    wanted: [..] u32;
    wanted.allocator = temp;

    for info.parameter_types array_add_if_missing(*wanted, it);
    for info.columns array_add_if_missing(*wanted, it.type_oid);

    // Follow domains and arrays to the types underneath; a few rounds always suffice.
    for 1..8 {
        missing: [..] u32;
        missing.allocator = temp;

        for wanted if !find_type(*types, it) array_add(*missing, it);
        if !missing.count break;

        builder: String_Builder;
        builder.allocator = temp;

        append(*builder, "select t.oid::int8, t.typname::text, t.typtype::text, t.typcategory::text, t.typbasetype::int8, t.typelem::int8, format_type(t.oid, null), quote_ident(n.nspname) || '.' || quote_ident(t.typname), (n.nspname = 'pg_catalog')::text from pg_type t join pg_namespace n on n.oid = t.typnamespace where t.oid in (");

        for missing {
            if it_index append(*builder, ", ");
            print_to_builder(*builder, "%", it);
        }

        append(*builder, ")");

        result, error := query_dynamic(db, builder_to_string(*builder));
        if error.kind != .NONE return types, error;

        for row: 0..result.row_count - 1 {
            type: Described_Type;
            oid, _ := get(*result, row, 0, s64);

            type.oid = cast(u32) oid;
            type.name = copy_string(text_cell(*result, row, 1));
            type.kind = first_byte(text_cell(*result, row, 2));
            type.category = first_byte(text_cell(*result, row, 3));
            base_oid, _ := get(*result, row, 4, s64);
            element_oid, _ := get(*result, row, 5, s64);

            type.base_oid = cast(u32) base_oid;
            type.element_oid = cast(u32) element_oid;
            type.shown = copy_string(text_cell(*result, row, 6));
            type.qualified = copy_string(text_cell(*result, row, 7));
            type.builtin = text_cell(*result, row, 8) == "true";

            array_add(*types.rows, type);

            if type.kind == #char "d" array_add_if_missing(*wanted, type.base_oid);
            if type.category == #char "A" && type.element_oid array_add_if_missing(*wanted, type.element_oid);
        }

        free_result(*result);
    }

    return types, .{};
}

array_add_if_missing :: (array: *[..] u32, value: u32) {
    for array.* if it == value return;
    array_add(array, value);
}

first_byte :: (text: string) -> u8 {
    return ifx text.count then text[0] else 0;
}

text_cell :: (result: *Result, row: s64, column: s64) -> string {
    value, error := get(result, row, column, Nullable(string));
    if error.kind != .NONE return "";

    return value.value;
}

find_type :: (types: *Described_Types, oid: u32) -> *Described_Type {
    for *types.rows if it.oid == oid return it;
    return null;
}

// What Jai maps a type to: a domain by its base type, an array by its element ("int4[]"), an enum
// as "enum", and a built-in or extension type by its name.
base_name :: (types: *Described_Types, oid: u32) -> string {
    type := find_type(types, oid);
    if !type return tprint("oid%", oid);

    if type.kind == #char "d" return base_name(types, type.base_oid);
    if type.category == #char "A" && type.element_oid return tprint("%[]", base_name(types, type.element_oid));
    if type.kind == #char "e" return "enum";
    if type.kind == #char "c" return "composite";
    if type.kind == #char "r" || type.kind == #char "m" return "range";
    if !type.builtin return "extension";

    return type.name;
}

shown_name :: (types: *Described_Types, oid: u32) -> string {
    type := find_type(types, oid);
    if !type return tprint("oid%", oid);

    return type.shown;
}

types_entry :: (types: *Described_Types, oid: u32) -> Query_Type {
    return .{ shown_name(types, oid), base_name(types, oid) };
}

Table_Attribute :: struct {
    table_oid: u32;
    number: s16;
    not_null: bool;
    origin: string; // schema.table.column
}

Table_Attributes :: struct {
    rows: [..] Table_Attribute;
}

// pg_attribute for the table columns among the result columns: whether each is NOT NULL, and where
// it comes from.
load_attributes :: (db: *Connection, columns: [] Column) -> Table_Attributes, Pg_Error {
    attributes: Table_Attributes;
    builder: String_Builder;
    builder.allocator = temp;

    count := 0;

    for columns {
        if !it.table_oid continue;

        append(*builder, ifx count then ", " else "");
        print_to_builder(*builder, "(%, %)", it.table_oid, it.column_number);
        count += 1;
    }

    if !count return attributes, .{};

    query := tprint("select a.attrelid::int8, a.attnum::int4, a.attnotnull::text, n.nspname || '.' || c.relname || '.' || a.attname from pg_attribute a join pg_class c on c.oid = a.attrelid join pg_namespace n on n.oid = c.relnamespace where (a.attrelid, a.attnum) in (%)", builder_to_string(*builder,, temp));
    result, error := query_dynamic(db, query);

    if error.kind != .NONE return attributes, error;

    for row: 0..result.row_count - 1 {
        attribute: Table_Attribute;
        oid, _ := get(*result, row, 0, s64);
        number, _ := get(*result, row, 1, s64);

        attribute.table_oid = cast(u32) oid;
        attribute.number = cast(s16) number;
        attribute.not_null = text_cell(*result, row, 2) == "true";
        attribute.origin = copy_string(text_cell(*result, row, 3));
        array_add(*attributes.rows, attribute);
    }

    free_result(*result);

    return attributes, .{};
}

// The generic plan of the statement, for the outer joins that make columns nullable and the
// expressions behind computed columns.
Query_Plan :: struct {
    found: bool;
    outputs: [..] string; // the top node's output expressions, one per result column
    nullable_aliases: [..] string; // relations on the nullable side of an outer join
}

explain_plan :: (db: *Connection, sql: string, parameters: s64, columns: s64) -> Query_Plan {
    plan: Query_Plan;
    if !columns return plan;

    // The savepoint keeps a statement that cannot be PREPAREd (a utility command) from aborting
    // the transaction.
    error := execute_script(db, "set local plan_cache_mode = force_generic_plan; savepoint jacket_explain");
    if error.kind != .NONE {
        free_error(*error);
        return plan;
    }

    error = execute_script(db, tprint("prepare jacket_describe as %", sql));

    if error.kind != .NONE {
        free_error(*error);
        rolled_back := execute_script(db, "rollback to savepoint jacket_explain");
        free_error(*rolled_back);

        return plan;
    }

    builder: String_Builder;
    builder.allocator = temp;

    append(*builder, "explain (verbose, format json) execute jacket_describe");
    if parameters append(*builder, "(");

    for 1..parameters append(*builder, ifx it > 1 then ", null" else "null");
    if parameters append(*builder, ")");

    result, explain_error := query_dynamic(db, builder_to_string(*builder));

    if explain_error.kind != .NONE {
        free_error(*explain_error);
        rolled_back := execute_script(db, "rollback to savepoint jacket_explain");
        free_error(*rolled_back);

        return plan;
    }

    text := text_cell(*result, 0, 0);
    document, parse_error := Json_Module.from_json(Json_Value, text);

    if parse_error.kind != .NONE {
        free_result(*result);
        return plan;
    }

    root := json_at(*document, 0);
    top: *Json_Value;

    if root top = json_get(root, "Plan");

    if top {
        plan.found = true;
        outputs := json_get(top, "Output");

        if outputs for 0..json_count(outputs) - 1 {
            array_add(*plan.outputs, copy_string(string_member_at(outputs, it)));
        }

        collect_nullable_aliases(top, false, *plan.nullable_aliases);
    }

    free_result(*result);

    return plan;
}

string_member :: (object: *Json_Value, key: string) -> string {
    member := json_get(object, key);
    if !member return "";

    text, _ := json_as_string(member);

    return text;
}

int_member :: (object: *Json_Value, key: string) -> s64 {
    member := json_get(object, key);
    if !member return 0;

    value, _ := json_as_s64(member);

    return value;
}

string_member_at :: (array: *Json_Value, index: s64) -> string {
    item := json_at(array, index);
    if !item return "";

    text, _ := json_as_string(item);

    return text;
}

// Every relation under the nullable side of an outer join, however deep. A left join makes its
// inner side nullable, a right join its outer side, a full join both. The planner may flip a join
// from the one written, so the node's own `Join Type` is what counts.
collect_nullable_aliases :: (node: *Json_Value, nullable: bool, aliases: *[..] string) {
    if nullable {
        alias := string_member(node, "Alias");
        if alias array_add(aliases, copy_string(alias));
    }

    join := string_member(node, "Join Type");
    children := json_get(node, "Plans");

    if !children return;

    for 0..json_count(children) - 1 {
        child := json_at(children, it);
        side := string_member(child, "Parent Relationship");

        below := nullable;

        if join == "Left" && side == "Inner" below = true;
        if join == "Right" && side == "Outer" below = true;
        if join == "Full" && (side == "Inner" || side == "Outer") below = true;

        collect_nullable_aliases(child, below, aliases);
    }
}

// Table, column and expression, as the planner prints a column: `p.id`.
output_alias :: (output: string) -> string {
    if !output.count || output[0] == #char "(" || output[0] == #char "'" return "";

    dot := find_index_from_left(output, #char ".");
    if dot <= 0 return "";

    // Only a plain reference: alias, dot, column, nothing else.
    for output {
        if it == #char "(" || it == #char " " || it == #char "," return "";
    }

    alias := slice(output, 0, dot);
    if alias.count > 1 && alias[0] == #char "\"" alias = slice(alias, 1, alias.count - 2);

    return alias;
}

column_entry :: (types: *Described_Types, attributes: *Table_Attributes, plan: *Query_Plan, column: Column, index: s64) -> Query_Column {
    entry: Query_Column;

    parse_alias(column.name, *entry);

    entry.name = copy_string(entry.name);
    entry.jai = copy_string(entry.jai);
    entry.type = shown_name(types, column.type_oid);
    entry.base = base_name(types, column.type_oid);
    entry.oid = column.type_oid;

    output := ifx index < plan.outputs.count then plan.outputs[index] else "";

    entry.nullable = true;

    attribute: *Table_Attribute;
    if column.table_oid for *attributes.rows if it.table_oid == column.table_oid && it.number == column.column_number attribute = it;

    if attribute {
        entry.origin = attribute.origin;
        entry.nullable = !attribute.not_null;

        if !entry.nullable && plan.found {
            alias := output_alias(output);
            if alias for plan.nullable_aliases if it == alias entry.nullable = true;
        }
    } else if plan.found {
        entry.nullable = !known_non_null(output);
    }

    if entry.forced == .NON_NULL entry.nullable = false;
    if entry.forced == .NULLABLE entry.nullable = true;

    return entry;
}

// The alias syntax: `name`, `name!` (never NULL), `name?` (may be NULL), and `name: Jai_Type`
// after either. The suffix and the type are stripped from the field name.
parse_alias :: (alias: string, entry: *Query_Column) {
    name := alias;
    colon := find_index_from_left(name, #char ":");

    if colon >= 0 {
        entry.jai = trim(advance(name, colon + 1));
        name = trim(slice(name, 0, colon));
    }

    if name.count > 1 && name[name.count - 1] == #char "!" {
        entry.forced = .NON_NULL;
        name.count -= 1;
    } else if name.count > 1 && name[name.count - 1] == #char "?" {
        entry.forced = .NULLABLE;
        name.count -= 1;
    }

    entry.name = name;
}

// An output expression that can never be NULL: the short list the design names. Anything else
// computed is nullable until the alias says otherwise.
known_non_null :: (expression: string) -> bool {
    e := trim(expression);

    // Peel parentheses that wrap the whole expression.
    while e.count > 1 && e[0] == #char "(" && matching_paren(e, 0) == e.count - 1 {
        e = trim(slice(e, 1, e.count - 2));
    }

    if begins_with(e, "count(") return true;
    if begins_with(e, "EXISTS(") || begins_with(e, "EXISTS (") return true;
    if begins_with(e, "row_number() OVER") || begins_with(e, "rank() OVER") || begins_with(e, "dense_rank() OVER") return true;
    if ends_with(e, " IS NULL") || ends_with(e, " IS NOT NULL") return true;
    if is_literal(e) return true;

    if begins_with(e, "COALESCE(") && ends_with(e, ")") {
        arguments := split_arguments(slice(e, 9, e.count - 10));
        if arguments.count return known_non_null(arguments[arguments.count - 1]);
    }

    return false;
}

is_literal :: (e: string) -> bool {
    if !e.count return false;
    if e == "true" || e == "false" return true;

    if scan_is_digit(e[0]) || (e[0] == #char "-" && e.count > 1 && scan_is_digit(e[1])) {
        for e if !(scan_is_digit(it) || it == #char "." || it == #char "-" || it == #char "e") return false;
        return true;
    }

    if e[0] != #char "'" return false;

    end := scan_skip_quoted(e, 0, #char "'", false);
    rest := advance(e, end);

    if !rest.count return true;

    // `'x'::text`, `'2020-01-01'::date`: a cast to a plain type name.
    if !begins_with(rest, "::") return false;

    for advance(rest, 2) if !(scan_is_word(it) || it == #char "[" || it == #char "]") return false;

    return true;
}

// The index of the parenthesis that closes the one at `start`, or -1.
matching_paren :: (text: string, start: s64) -> s64 {
    depth := 0;
    i := start;

    while i < text.count {
        c := text[i];

        if c == #char "'" {
            i = scan_skip_quoted(text, i, #char "'", false);
            continue;
        }

        if c == #char "(" depth += 1;

        if c == #char ")" {
            depth -= 1;
            if depth == 0 return i;
        }

        i += 1;
    }

    return -1;
}

// "a, f(b, c), 'x,y'" is three arguments.
split_arguments :: (text: string) -> [] string {
    arguments: [..] string;
    arguments.allocator = temp;

    depth := 0;
    start := 0;
    i := 0;

    while i < text.count {
        c := text[i];

        if c == #char "'" {
            i = scan_skip_quoted(text, i, #char "'", false);
            continue;
        }

        if c == #char "(" depth += 1;
        if c == #char ")" depth -= 1;

        if c == #char "," && depth == 0 {
            array_add(*arguments, trim(slice(text, start, i - start)));
            start = i + 1;
        }

        i += 1;
    }

    array_add(*arguments, trim(advance(text, start)));

    return arguments;
}

// The tables, views, types and functions the statement uses, found by letting the server create a
// function from it (inside the rolled-back transaction) and reading pg_depend. Views are followed
// to what they use. On failure the entry says so (`deps_complete` stays false) and relies on the
// schema hash alone.
dependencies :: (db: *Connection, sql: string, setup: string, info: Statement_Info, types: *Described_Types, entry: *Query_Entry) {
    error := execute_script(db, "begin");
    if error.kind != .NONE {
        free_error(*error);
        return;
    }

    if setup {
        error = execute_script(db, setup);

        if error.kind != .NONE {
            free_error(*error);
            rolled_back := execute_script(db, "rollback");
            free_error(*rolled_back);
            return;
        }
    }

    defer {
        rolled_back := execute_script(db, "rollback");
        free_error(*rolled_back);
    }

    keys: [..] string;
    keys.allocator = temp;

    // The described types that are not built in.
    for info.parameter_types add_type_key(*keys, types, it);
    for info.columns add_type_key(*keys, types, it.type_oid);

    body := sql;
    while body.count && (body[body.count - 1] == #char ";" || body[body.count - 1] == #char " " || body[body.count - 1] == #char "\n") body.count -= 1;

    builder: String_Builder;
    builder.allocator = temp;

    append(*builder, "create function pg_temp.jacket_q(");

    for info.parameter_types {
        if it_index append(*builder, ", ");
        print_to_builder(*builder, "%", shown_name(types, it));
    }

    append(*builder, ") returns ");

    if info.columns.count {
        append(*builder, "table (");

        for info.columns {
            if it_index append(*builder, ", ");
            print_to_builder(*builder, "c% %", it_index + 1, shown_name(types, it.type_oid));
        }

        append(*builder, ")");
    } else {
        append(*builder, "void");
    }

    print_to_builder(*builder, " language sql begin atomic %; end", body);

    error = execute_script(db, builder_to_string(*builder));

    if error.kind != .NONE {
        free_error(*error);
        finish_dependencies(db, keys, entry, false);
        return;
    }

    refine_from_body(db, entry);

    result, query_error := query_dynamic(db, DEPENDENCY_QUERY);

    if query_error.kind != .NONE {
        free_error(*query_error);
        finish_dependencies(db, keys, entry, false);
        return;
    }

    for row: 0..result.row_count - 1 {
        key := text_cell(*result, row, 0);
        add_key(*keys, key);
    }

    free_result(*result);
    finish_dependencies(db, keys, entry, true);
}

key_before :: (a: string, b: string) -> bool {
    for 0..min(a.count, b.count) - 1 {
        if a[it] != b[it] return a[it] < b[it];
    }

    return a.count < b.count;
}

// A computed column the plan could not vouch for may still be one the deparsed statement shows to
// be non-null: `exists(...)` of an uncorrelated subquery plans as an InitPlan, which the plan's
// output does not name. The deparsed text has the select list in plain form.
refine_from_body :: (db: *Connection, entry: *Query_Entry) {
    wanted := false;
    for entry.columns if it.nullable && !it.origin && it.forced == .NONE wanted = true;
    if !wanted return;

    result, error := query_dynamic(db, "select pg_get_function_sqlbody(p.oid) from pg_proc p where p.proname = 'jacket_q' and p.pronamespace = pg_my_temp_schema()");

    if error.kind != .NONE {
        free_error(*error);
        return;
    }

    body := text_cell(*result, 0, 0);
    start := find_index_from_left(body, "SELECT ");

    if start < 0 {
        free_result(*result);
        return;
    }

    list := advance(body, start + 7);
    end := top_level_from(list);

    list = slice(list, 0, end);
    items := split_arguments(list);

    if items.count == entry.columns.count {
        for *column: entry.columns {
            if !column.nullable || column.origin || column.forced != .NONE continue;

            item := items[it_index];
            as := last_top_level_as(item);

            if as >= 0 item = trim(slice(item, 0, as));

            if known_non_null(item) column.nullable = false;
        }
    }

    free_result(*result);
}

// Where the select list ends: the first top-level ` FROM `, or the end of the text.
top_level_from :: (text: string) -> s64 {
    depth := 0;
    i := 0;

    while i < text.count {
        c := text[i];

        if c == #char "'" {
            i = scan_skip_quoted(text, i, #char "'", false);
            continue;
        }

        if c == #char "(" depth += 1;
        if c == #char ")" depth -= 1;

        if depth == 0 && begins_with(advance(text, i), "\nFROM ") return i;
        if depth == 0 && begins_with(advance(text, i), " FROM ") return i;
        if depth == 0 && begins_with(advance(text, i), ";") return i;

        i += 1;
    }

    return text.count;
}

// The index of the ` AS ` that names a select-list item, or -1.
last_top_level_as :: (item: string) -> s64 {
    depth := 0;
    found := -1;
    i := 0;

    while i < item.count {
        c := item[i];

        if c == #char "'" {
            i = scan_skip_quoted(item, i, #char "'", false);
            continue;
        }

        if c == #char "(" depth += 1;
        if c == #char ")" depth -= 1;
        if depth == 0 && begins_with(advance(item, i), " AS ") found = i;

        i += 1;
    }

    return found;
}

add_type_key :: (keys: *[..] string, types: *Described_Types, oid: u32) {
    type := find_type(types, oid);
    if !type || type.builtin return;

    if type.category == #char "A" && type.element_oid {
        add_type_key(keys, types, type.element_oid);
        return;
    }

    add_key(keys, tprint("t:%", type.qualified));
}

add_key :: (keys: *[..] string, key: string) {
    for keys.* if it == key return;
    array_add(keys, copy_string(key,, temp));
}

finish_dependencies :: (db: *Connection, keys: [] string, entry: *Query_Entry, complete: bool) {
    entry.deps_complete = complete;
    sorted := array_copy(keys,, temp);

    // Insertion sort: a statement depends on a handful of things, and the order keeps cache files
    // stable.
    for i: 1..sorted.count - 1 {
        j := i;

        while j > 0 && key_before(sorted[j], sorted[j - 1]) {
            sorted[j], sorted[j - 1] = sorted[j - 1], sorted[j];
            j -= 1;
        }
    }

    prints, error := fingerprints(db, sorted);

    if error.kind != .NONE {
        free_error(*error);
        entry.deps_complete = false;
        return;
    }

    for sorted array_add(*entry.deps, .{ copy_string(it), prints[it_index] });
}

// Every relation, function and type under the function, through views, outside the system
// schemas. A view's rewrite rule lists what it reads; the recursion follows it down.
DEPENDENCY_QUERY :: #string DONE
with recursive d(classid, objid) as (
    select refclassid, refobjid from pg_depend
    where classid = 'pg_proc'::regclass
      and objid = (select oid from pg_proc where proname = 'jacket_q' and pronamespace = pg_my_temp_schema())
      and deptype = 'n'
      and refclassid in ('pg_class'::regclass, 'pg_proc'::regclass, 'pg_type'::regclass)
    union
    select dd.refclassid, dd.refobjid from d
    join pg_rewrite rw on d.classid = 'pg_class'::regclass and rw.ev_class = d.objid
    join pg_depend dd on dd.classid = 'pg_rewrite'::regclass and dd.objid = rw.oid and dd.deptype = 'n'
      and dd.refclassid in ('pg_class'::regclass, 'pg_proc'::regclass, 'pg_type'::regclass)
      and dd.refobjid <> d.objid
)
select distinct key from (
    select 'r:' || quote_ident(n.nspname) || '.' || quote_ident(c.relname) as key
    from d join pg_class c on d.classid = 'pg_class'::regclass and c.oid = d.objid
    join pg_namespace n on n.oid = c.relnamespace
    where n.nspname not in ('pg_catalog', 'information_schema', 'pg_toast') and n.nspname not like 'pg_temp%'
  union all
    select 'f:' || p.oid::regprocedure::text
    from d join pg_proc p on d.classid = 'pg_proc'::regclass and p.oid = d.objid
    join pg_namespace n on n.oid = p.pronamespace
    where n.nspname not in ('pg_catalog', 'information_schema') and n.nspname not like 'pg_temp%'
  union all
    select 't:' || quote_ident(n.nspname) || '.' || quote_ident(t.typname)
    from d join pg_type t on d.classid = 'pg_type'::regclass and t.oid = d.objid
    join pg_namespace n on n.oid = t.typnamespace
    where n.nspname not in ('pg_catalog', 'information_schema', 'pg_toast') and n.nspname not like 'pg_temp%' and t.typcategory <> 'A' and t.typtype <> 'c'
) k order by key
DONE

// One hash per dependency key ("r:public.users", "t:public.mood", "f:public.add(integer)"): SHA-256
// over the catalog rows that decide how a query against it is typed. A key that no longer exists
// hashes to the empty string.
fingerprints :: (db: *Connection, keys: [] string) -> [] string, Pg_Error {
    prints := NewArray(keys.count, string,, temp);
    if !keys.count return prints, .{};

    builder: String_Builder;
    builder.allocator = temp;

    append(*builder, "select k, coalesce(case left(k, 2) when 'r:' then (");
    append(*builder, RELATION_FINGERPRINT);
    append(*builder, ") when 't:' then (");
    append(*builder, TYPE_FINGERPRINT);
    append(*builder, ") when 'f:' then (");
    append(*builder, FUNCTION_FINGERPRINT);
    append(*builder, ") end, '') from unnest(array[");

    for key, key_index: keys {
        if key_index append(*builder, ", ");

        append(*builder, "'");

        for *key {
            if it.* == #char "'" append(*builder, "'");
            append(*builder, slice(key, it_index, 1));
        }

        append(*builder, "'");
    }

    append(*builder, "]::text[]) with ordinality as x(k, n) order by n");

    result, error := query_dynamic(db, builder_to_string(*builder));
    if error.kind != .NONE return prints, error;

    for row: 0..result.row_count - 1 {
        if row < prints.count prints[row] = copy_string(text_cell(*result, row, 1),, temp);
    }

    free_result(*result);

    return prints, .{};
}

RELATION_FINGERPRINT :: #string DONE
select encode(sha256(convert_to(c.relkind::text || '|' || coalesce((select string_agg(a.attnum::text || ':' || a.attname || ':' || format_type(a.atttypid, a.atttypmod) || ':' || a.attnotnull::text || ':' || a.attgenerated::text || ':' || a.attidentity::text, ',' order by a.attnum) from pg_attribute a where a.attrelid = c.oid and a.attnum > 0 and not a.attisdropped), '') || '|' || case when c.relkind in ('v', 'm') then coalesce(pg_get_viewdef(c.oid), '') else '' end, 'UTF8')), 'hex')
from pg_class c where c.oid = to_regclass(substr(k, 3))
DONE

TYPE_FINGERPRINT :: #string DONE
select encode(sha256(convert_to(t.typtype::text || '|' || format_type(t.typbasetype, t.typtypmod) || '|' || t.typnotnull::text || '|' || coalesce((select string_agg(e.enumlabel, ',' order by e.enumsortorder) from pg_enum e where e.enumtypid = t.oid), ''), 'UTF8')), 'hex')
from pg_type t where t.oid = to_regtype(substr(k, 3))
DONE

FUNCTION_FINGERPRINT :: #string DONE
select encode(sha256(convert_to(pg_get_function_result(p.oid) || '|' || pg_get_function_arguments(p.oid) || '|' || p.prokind::text || '|' || md5(coalesce(p.prosrc, '') || coalesce(pg_get_function_sqlbody(p.oid), '')), 'UTF8')), 'hex')
from pg_proc p where p.oid = to_regprocedure(substr(k, 3))
DONE
