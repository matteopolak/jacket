// The compile-time side of typed queries: from the SQL a macro was given to the description of its
// parameters and columns, through the offline cache and the describe service. Everything here
// runs inside `#run`, on behalf of one call site.
//
// The pipeline, per statement (docs/postgres/typed-queries.md):
//   1. rewrite `%` to `$n`, and hash the result into the statement id;
//   2. use `<QUERY_CACHE>/<id>.json` when it is still good: the migrations' hash matches, or every
//      dependency fingerprint still matches the database;
//   3. otherwise ask the describe service, and write the answer back to the cache;
//   4. with no database, a cache hit is all there is: a miss is an error (or "unknown", in the
//      editor, which asks the service and shows nothing until it answers).

#scope_module

Service :: #import "Extensions/Service";
String_Literal :: #import "Extensions/String_Literal";
Typed_Files :: #import "File_Utilities";
#import "Compiler";

DESCRIBE_SERVICE :: #run join(#filepath, "describe_service.jai");

// What one call site got: the rewritten statement and its description.
Prepared :: struct {
    key: string;
    rewrite: Rewrite;
    entry: Query_Entry;
    id: u64;
    sql_node: *Code_Node;
    call_node: *Code_Procedure_Call;
    project_file: string; // the file the project is found from (see `find_project_root`)
}

prepared_memo: [..] *Prepared;

// The managed server's binaries and cluster are ready (see `prepare_managed`).
managed_prepared: bool;

// Where to look, from the environment and the project: see `typed_settings`.
Typed_Settings :: struct {
    url: string; // JACKET_DATABASE_URL, else DATABASE_URL, from the environment or the project's .env
    managed: bool; // JACKET_PG_MANAGED (or jacket.toml), no URL, not offline: describe in the managed server
    managed_version: string; // the managed server's release (JACKET_PG_MANAGED_VERSION, or jacket.toml)
    search_path: string; // JACKET_PG_SEARCH_PATH
    offline: bool; // JACKET_PG_OFFLINE: never connect
    recording: bool; // JACKET_PG_CHECK: note every statement compiled
    root: string; // the project folder, which holds the migrations, schema and query cache
    migrations: string; // the migrations folder (MIGRATIONS), or empty when there is none
    schema_hash: string; // of the migrations, or empty
}

node_location :: (node: *Code_Node) -> Source_Code_Location {
    location: Source_Code_Location;
    if !node return location;

    if node.enclosing_load location.fully_pathed_filename = node.enclosing_load.fully_pathed_filename;

    location.line_number = node.l0;
    location.character_number = node.c0;

    return location;
}

// Reports at a byte of the SQL string, or at the string itself when it is not a literal.
report_at_byte :: (sql_node: *Code_Node, sql: string, byte: s64, message: string) {
    if sql_node && sql_node.kind == .LITERAL {
        literal := cast(*Code_Literal) sql_node;
        location, _ := String_Literal.location_of_byte(literal, byte);
        compiler_report(message, location);
        return;
    }

    compiler_report(message, node_location(sql_node));
}

report_at :: (node: *Code_Node, message: string) {
    compiler_report(message, node_location(node));
}

call_of :: (call: Code) -> *Code_Procedure_Call {
    root, _ := compiler_get_nodes(call);

    return cast(*Code_Procedure_Call) root;
}

#scope_export

// The project's migrations folder, schema file and query cache, from the module parameters
// MIGRATIONS, SCHEMA and QUERY_CACHE (paths relative to the project root).
migrations_path :: (root: string) -> string {
    return join(root, "/", MIGRATIONS,, temp);
}

schema_path :: (root: string) -> string {
    return join(root, "/", SCHEMA,, temp);
}

query_cache_path :: (root: string) -> string {
    return join(root, "/", QUERY_CACHE,, temp);
}

// MIN_SERVER_VERSION: the oldest server major version the project supports; 0 accepts any.
min_server_version :: () -> s64 {
    return MIN_SERVER_VERSION;
}

// The folder that holds the project's files: from the file with the query, the nearest folder up
// the tree with a `jacket.toml` or the first folder of MIGRATIONS (`db`, by default); if there is
// none, that file's own folder. JACKET_PG_ROOT overrides it.
find_project_root :: (file: string) -> string {
    forced := environment_variable("JACKET_PG_ROOT");
    if forced return forced;

    start := path_strip_filename(file);
    if start.count > 1 && start[start.count - 1] == #char "/" start.count -= 1;

    dir := start;

    while dir.count {
        if Typed_Files.file_exists(join(dir, "/jacket.toml",, temp)) return copy_string(dir,, temp);

        is_dir, ok := Typed_Files.is_directory(join(dir, "/", first_folder(MIGRATIONS),, temp));
        if ok && is_dir return copy_string(dir,, temp);

        slash := find_index_from_right(dir, #char "/");
        if slash <= 0 break;

        dir.count = slash;
    }

    return copy_string(start,, temp);
}

// The compile-time database's URL: JACKET_DATABASE_URL, else DATABASE_URL, from the environment;
// when it has neither, the same two names in the project's `.env` file. Empty when there is none.
database_url :: (root: string) -> string {
    url := environment_variable("JACKET_DATABASE_URL");
    if !url url = environment_variable("DATABASE_URL");
    if !url url = dotenv_database_url(root);

    return url;
}

#scope_module

// `db` for "db/migrations".
first_folder :: (path: string) -> string {
    slash := find_index_from_left(path, #char "/");

    return ifx slash > 0 then slice(path, 0, slash) else path;
}

typed_settings :: (file: string) -> Typed_Settings {
    settings: Typed_Settings;
    settings.root = find_project_root(file);
    settings.url = database_url(settings.root);

    settings.search_path = environment_variable("JACKET_PG_SEARCH_PATH");
    offline := environment_variable("JACKET_PG_OFFLINE");

    settings.offline = offline != "" && offline != "0";

    settings.recording = environment_variable("JACKET_PG_CHECK") != "";

    // JACKET_PG_MANAGED, else `[postgres] managed` in jacket.toml.
    managed := environment_variable("JACKET_PG_MANAGED");
    on := managed != "" && managed != "0";

    if !managed {
        from_project := project_managed_settings(settings.root);
        on = from_project == 1;
    }

    settings.managed = !settings.url && !settings.offline && on;
    if settings.managed settings.managed_version = managed_settings_for(settings.root).version;

    settings.migrations, settings.schema_hash = migrations_of(settings.root);

    return settings;
}

#scope_export

// The project's migrations folder and their schema hash (see `schema_hash`), or empty strings when
// there are none.
migrations_of :: (root: string) -> folder: string, hash: string {
    folder := migrations_path(root);
    is_dir, ok := Typed_Files.is_directory(folder);

    if !ok || !is_dir return "", "";

    migrations, problem := read_migrations(folder,, temp);
    if problem || !migrations.count return "", "";

    return folder, schema_hash(migrations);
}

#scope_module

read_if_exists :: (path: string) -> string, bool {
    if !Typed_Files.file_exists(path) return "", false;

    text, ok := read_entire_file(path,, temp);

    return text, ok;
}

#scope_export

// `id_hex` read back.
statement_id_of_hex :: (text: string) -> u64 {
    id: u64;

    for text {
        digit: u64 = ifx it >= #char "a" then cast(u64) (it - #char "a" + 10) else cast(u64) (it - #char "0");
        id = (id << 4) | digit;
    }

    return id;
}

cache_file :: (root: string, id: u64) -> string {
    return join(query_cache_path(root), "/", id_hex(id), ".json",, temp);
}

read_cache_entry :: (path: string, sql: string) -> Query_Entry, bool {
    if !Typed_Files.file_exists(path) return .{}, false;

    text, read := read_entire_file(path,, temp);
    if !read return .{}, false;

    document, error := Json_Module.from_json(Json_Value, text,, temp);
    if error.kind != .NONE return .{}, false;

    entry, ok := entry_from_json(*document);

    // The file name holds 64 bits of the hash: the statement itself must match too. An entry of
    // another format may mean something else by the same words: it is a miss.
    if !ok || entry.sql != sql || entry.format != ENTRY_FORMAT return .{}, false;

    return entry, true;
}

write_cache_entry :: (root: string, id: u64, entry: Query_Entry) {
    make_cache_folder(root);

    document := entry_to_json(entry);
    text := Json_Module.to_json(document, pretty = true,, temp);
    path := cache_file(root, id);
    existing, read := read_if_exists(path);

    // Do not touch a file that would not change: a build runs often, and a watcher may be looking.
    if read && trim(existing) == text return;

    write_entire_file(path, tprint("%\n", text));
}

#scope_module

// The query cache folder and every folder above it that is missing.
make_cache_folder :: (root: string) {
    path := root;

    for part: split(QUERY_CACHE, "/",, temp) {
        if !part continue;

        path = join(path, "/", part,, temp);
        make_directory_if_it_does_not_exist(path);
    }
}

// Remembers that this build compiled the statement (JACKET_PG_CHECK), for the tool's `check`.
note_used :: (root: string, id: u64) {
    path := join(query_cache_path(root), "/used.txt",, temp);
    line := id_hex(id);
    existing, read := read_if_exists(path);

    if read {
        rest := existing;

        while rest.count {
            newline := find_index_from_left(rest, #char "\n");
            word := ifx newline < 0 then rest else slice(rest, 0, newline);

            if word == line return;
            if newline < 0 break;

            rest = advance(rest, newline + 1);
        }
    }

    make_cache_folder(root);
    write_entire_file(path, tprint("%%\n", ifx read then existing else "", line));
}

// One round trip to the describe service. In the editor a late answer ends the compile-time call
// as unknown, and the editor asks again when it arrives.
ask_service :: (request: Describe_Request) -> Service.Status, string {
    status, reply := Service.call(DESCRIBE_SERVICE, describe_request_json(request));

    if status == .PENDING Service.unknown("waiting for the database to describe a query");

    return status, reply;
}

// Describes the statement from scratch, trusting nothing cached. The answer is an entry, or has
// reported an error.
describe_with_service :: (prepared: *Prepared, settings: Typed_Settings, sql: string) -> Query_Entry, ok: bool, unavailable: string, stale: bool {
    request := Describe_Request.{ op = "describe", url = settings.url, managed = settings.managed, managed_version = settings.managed_version, search_path = settings.search_path, sql = prepared.rewrite.sql, schema_hash = settings.schema_hash, migrations = settings.migrations, min_server_version = MIN_SERVER_VERSION };
    status, reply := ask_service(request);

    if status == .UNAVAILABLE return .{}, false, reply, false;

    if status == .FAILED {
        report_at(prepared.sql_node, tprint("the describe service failed: %", reply));
        return .{}, false, "", false;
    }

    document, error := Json_Module.from_json(Json_Value, reply,, temp);

    if error.kind != .NONE {
        report_at(prepared.sql_node, tprint("the describe service sent an answer that is not JSON: %", error.message));
        return .{}, false, "", false;
    }

    kind := string_member(*document, "kind");
    stale := json_get(*document, "stale_schema");

    if kind == "migration_error" {
        failure, _ := report_migration_failure(json_get(*document, "migration"));
        report_at(prepared.sql_node, tprint("the migrations do not build, so this query cannot be checked: %", failure));

        return .{}, false, "", false;
    }

    if kind == "sql_error" {
        report_sql_error(prepared, sql, json_get(*document, "error"), stale);
        return .{}, false, "", false;
    }

    answer := json_get(*document, "entry");

    if kind != "ok" || !answer {
        report_at(prepared.sql_node, tprint("the describe service could not describe the query: %", string_member(*document, "message")));
        return .{}, false, "", false;
    }

    entry, ok := entry_from_json(answer);

    if !ok {
        report_at(prepared.sql_node, "the describe service sent a description this compiler cannot read");
        return .{}, false, "", false;
    }

    // A migration failed: the description comes from the schema before it, which the cache must
    // never hold as the project's.
    if stale {
        _, first := report_migration_failure(stale);

        if first {
            compiler_report(tprint("this query and the ones after it are checked against the schema before the failed migration %.sql", string_member(stale, "name")), node_location(prepared.sql_node), .INFO);
        }
    }

    return entry, true, "", stale != null;
}

// The server's complaint, at the character it points to inside the SQL string.
report_sql_error :: (prepared: *Prepared, sql: string, details: *Json_Value, stale: *Json_Value) {
    if !details {
        report_at(prepared.sql_node, "the database rejected the query");
        return;
    }

    builder: String_Builder;
    builder.allocator = temp;

    print_to_builder(*builder, "%: %", string_member(details, "code"), string_member(details, "message"));

    detail := string_member(details, "detail");
    hint := string_member(details, "hint");
    suggestion := string_member(details, "suggestion");

    if detail print_to_builder(*builder, "\n  detail: %", detail);
    if hint print_to_builder(*builder, "\n  hint: %", hint);
    if suggestion && !hint print_to_builder(*builder, "\n  hint: did you mean \"%\"?", suggestion);

    if stale {
        failure, _ := report_migration_failure(stale);
        print_to_builder(*builder, "\n  note: checked against the schema before the failed migration; %", failure);
    }

    byte_node := json_get(details, "byte");
    position := json_get(details, "position");

    if position && byte_node {
        rewritten_byte, _ := json_as_s64(byte_node);
        source_byte := source_offset_of(*prepared.rewrite, rewritten_byte);
        report_at_byte(prepared.sql_node, sql, source_byte, builder_to_string(*builder));
    } else {
        report_at(prepared.sql_node, builder_to_string(*builder));
    }
}

// Failures already reported in this compile, so many queries do not repeat one error.
reported_failures: [..] string;

// Reports a failed migration at its line in the migration file, once per compile, and returns the
// text ("migration 0007_add_teams.sql failed at line 3: ...") and whether this call reported it.
// A report with a continuing error mode names the file and line itself and fails the build without
// ending the macro, so the queries that depend on the schema still get their own answers.
report_migration_failure :: (details: *Json_Value) -> string, first: bool {
    if !details return "a migration failed", false;

    name := string_member(details, "name");
    file := string_member(details, "file");
    message := string_member(details, "message");
    line := int_member(details, "line");
    column := int_member(details, "column");

    text := ifx line > 0 then tprint("migration %.sql failed at line %: %", name, line, message) else tprint("migration %.sql failed: %", name, message);
    key := tprint("%|%|%", file, line, message);

    for reported_failures if it == key return text, false;

    array_add(*reported_failures, copy_string(key));

    location: Source_Code_Location;
    location.fully_pathed_filename = file;
    location.line_number = cast(s32) max(line, 1);
    location.character_number = cast(s32) max(column, 1);

    compiler_report(text, location, .ERROR_CONTINUABLE);

    return text, true;
}

// Is the cached entry still right? Without a database to ask, only the schema hash can say.
Cache_Verdict :: enum u8 {
    GOOD; // use it
    STALE; // describe again
    UNCHECKED; // could not be checked (no database): use it if nothing better
}

check_cache_entry :: (prepared: *Prepared, settings: Typed_Settings, entry: Query_Entry, note: *string) -> Cache_Verdict {
    if entry.search_path != settings.search_path return .STALE;

    if settings.schema_hash && entry.schema_hash == settings.schema_hash return .GOOD;

    if settings.offline || !settings.url {
        // The migrations changed since the entry was described: it may be wrong.
        if settings.schema_hash != entry.schema_hash return .STALE;

        return .UNCHECKED;
    }

    if !entry.deps_complete return .STALE;

    request := Describe_Request.{ op = "fingerprints", url = settings.url, search_path = settings.search_path, migrations = settings.migrations, min_server_version = MIN_SERVER_VERSION };
    keys: [..] string;
    keys.allocator = temp;
    for entry.deps array_add(*keys, it.key);

    request.keys = keys;

    status, reply := ask_service(request);

    if status == .UNAVAILABLE {
        note.* = reply;
        return .UNCHECKED;
    }

    if status != .OK return .STALE;

    document, error := Json_Module.from_json(Json_Value, reply,, temp);
    if error.kind != .NONE || string_member(*document, "kind") != "ok" return .STALE;

    major := json_get(*document, "server_major");
    if major {
        value, _ := json_as_s64(major);
        if value != entry.server_major return .STALE;
    }

    prints := json_get(*document, "fingerprints");
    if !prints return .STALE;

    for entry.deps {
        current := string_member(prints, it.key);
        if current == "" || current != it.fingerprint return .STALE;
    }

    return .GOOD;
}

// The description of the statement at one call site, memoised for the compile. Reports errors
// itself; a result that is not known yet (the editor is still waiting) ends the compile-time call.
// `project_file` is the file the project is found from, when not the call's own: the ORM's
// statements are written inside the module, but belong to the file that called it.
prepare_query :: (sql: string, call: Code, sql_index: s64, project_file := "") -> *Prepared {
    call_node := call_of(call);
    sql_node: *Code_Node;

    if sql_index < call_node.arguments_sorted.count sql_node = call_node.arguments_sorted[sql_index];

    where := node_location(ifx sql_node then sql_node else cast(*Code_Node) call_node);
    file := ifx project_file then project_file else where.fully_pathed_filename;
    key := tprint("%|%:%:%|%", sql, where.fully_pathed_filename, where.line_number, where.character_number, file);

    for prepared_memo if it.key == key return it;

    prepared := New(Prepared);
    prepared.key = copy_string(key);
    prepared.call_node = call_node;
    prepared.sql_node = sql_node;
    prepared.project_file = copy_string(file);
    prepared.rewrite = rewrite_placeholders(sql);

    if prepared.rewrite.error {
        report_at_byte(sql_node, sql, prepared.rewrite.error_offset, prepared.rewrite.error);
        return prepared;
    }

    arguments := value_arguments(call_node, sql_index + 1).count;

    if arguments != prepared.rewrite.holes && !has_spread(call_node, sql_index + 1) {
        got := ifx arguments == 1 then "1 argument was" else tprint("% arguments were", arguments);
        message := tprint("the statement has % placeholder% but % given", prepared.rewrite.holes, ifx prepared.rewrite.holes == 1 then "" else "s", got);
        report_at(sql_node, message);
        return prepared;
    }

    prepared.id = statement_id(prepared.rewrite.sql);
    settings := typed_settings(file);

    if settings.recording note_used(settings.root, prepared.id);

    path := cache_file(settings.root, prepared.id);
    cached, found := read_cache_entry(path, prepared.rewrite.sql);

    unreachable := "";

    if found {
        verdict := check_cache_entry(prepared, settings, cached, *unreachable);

        if verdict == .GOOD {
            prepared.entry = cached;

            // The migrations moved but the query's tables did not: record the new hash.
            if cached.schema_hash != settings.schema_hash && settings.schema_hash {
                prepared.entry.schema_hash = settings.schema_hash;
                write_cache_entry(settings.root, prepared.id, prepared.entry);
            }

            array_add(*prepared_memo, prepared);

            return prepared;
        }

        if verdict == .UNCHECKED {
            if unreachable compiler_report(tprint("not re-checked: %", unreachable), node_location(sql_node), .INFO);

            prepared.entry = cached;
            array_add(*prepared_memo, prepared);

            return prepared;
        }
    }

    if settings.offline {
        report_at(sql_node, tprint("no up-to-date description of this query in %, and JACKET_PG_OFFLINE is set: run the pg tool's prepare against a database and commit the result", path));
        return prepared;
    }

    if !settings.url && !settings.managed {
        Service.unknown(tprint("the query at %:%:% has no cached description: set DATABASE_URL (or JACKET_DATABASE_URL), set JACKET_PG_MANAGED=1 to describe it in a managed server, or run the pg tool's prepare", where.fully_pathed_filename, where.line_number, where.character_number));
        return prepared;
    }

    // A managed server starts empty: the schema can only come from the migrations.
    if settings.managed && !settings.migrations {
        report_at(sql_node, tprint("managed mode is on, but there are no migrations to build the schema from: add SQL files to %, or set DATABASE_URL", migrations_path(settings.root)));
        return prepared;
    }

    // The first start downloads and unpacks the server, which takes longer than a service request
    // may (see docs/extensions/service.md): do that here, and leave only the start to the service.
    if settings.managed && !managed_prepared {
        problem := prepare_managed(managed_settings_for(settings.root));

        if problem {
            Service.unknown(tprint("cannot prepare the managed PostgreSQL server: %", problem));
            return prepared;
        }

        managed_prepared = true;
    }

    entry, ok, unavailable, stale := describe_with_service(prepared, settings, sql);

    if !ok {
        if unavailable {
            Service.unknown(tprint("the query at %:%:% has no usable cached description and the database is unreachable: %", where.fully_pathed_filename, where.line_number, where.character_number, unavailable));
        }

        return prepared;
    }

    prepared.entry = entry;
    if !stale write_cache_entry(settings.root, prepared.id, entry);

    array_add(*prepared_memo, prepared);

    return prepared;
}

// `..values` passed as an argument: the number of values is only known at run time.
has_spread :: (call_node: *Code_Procedure_Call, first: s64) -> bool {
    for first..call_node.arguments_sorted.count - 1 {
        node := call_node.arguments_sorted[it];
        if node.kind == .UNARY_OPERATOR return true;
        if node.type && node.type.type == .ARRAY && node.type == type_info([] Any) return true;
    }

    return false;
}
