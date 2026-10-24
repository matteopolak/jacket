// Schema databases: the describe service checks typed queries against a database built from the
// project's migrations, never against the developer's own database. Each one is named after the
// hash of the migrations it holds, `jacket_<first 12 hex digits>`, on the server the compile-time
// URL names (which needs CREATEDB).
//
// A request whose migrations have no database yet is answered from the database of the longest
// prefix that has one: the newer migrations run inside the describe's transaction, which is rolled
// back. A database for the whole set is built once the set has stayed the same for a while, so a
// migration being edited does not build one per keystroke. With no prefix database, or a newer
// migration that cannot run in a transaction, it is built right away. See
// docs/postgres/typed-queries.md.

Schema_Databases :: struct {
    // When each set of migrations (by schema hash) was first asked about without a database.
    first_seen: [..] Seen_Hash;
}

Seen_Hash :: struct {
    hash: string;
    milliseconds: s64;
}

// Where to describe: `database` on the same server (empty: the URL's own database) after running
// `setup` inside the transaction. `built` names a database this call created, so the caller can
// let go of connections to older ones and then `prune_schema_databases`.
Schema_Target :: struct {
    database: string;
    setup: string;
    built: bool;
}

// How long a set of migrations must stay unchanged before its database is built;
// `JACKET_PG_BUILD_DELAY_MS` overrides it.
SCHEMA_BUILD_DELAY_MS :: 2_000;
// Schema databases kept on the server, newest first.
SCHEMA_DATABASES_KEPT :: 3;

schema_database_name :: (hash: string) -> string {
    return tprint("jacket_%", slice(hash, 0, 12));
}

// Picks (and builds when it is time) the schema database for the migrations in `folder`. `db` is
// connected to the server's own database and `config` is how it was reached.
schema_target :: (state: *Schema_Databases, db: *Connection, config: Config, folder: string) -> Schema_Target, problem: string {
    migrations, problem := read_migrations(folder,, temp);
    if problem return .{}, problem;
    if !migrations.count return .{}, "";

    hashes := prefix_hashes(migrations);
    full := hashes[hashes.count - 1];

    existing, error := schema_database_names(db);

    if error.kind != .NONE return .{}, tprint("cannot list the schema databases: %", error_text(error));

    best := -1;

    for < hashes {
        if array_find(existing, schema_database_name(it)) {
            best = it_index;
            break;
        }
    }

    if best == migrations.count - 1 return .{ database = schema_database_name(full) }, "";

    must_build := best < 0;
    for best + 1..migrations.count - 1 if !migrations[it].transaction must_build = true;

    now := to_milliseconds(current_time_monotonic());
    seen := now;
    found := false;

    for state.first_seen if it.hash == full {
        seen = it.milliseconds;
        found = true;
    }

    if !found array_add(*state.first_seen, .{ hash = copy_string(full), milliseconds = now });

    delay := SCHEMA_BUILD_DELAY_MS;
    configured := environment_variable("JACKET_PG_BUILD_DELAY_MS");

    if configured {
        value, ok := string_to_int(configured);
        if ok delay = value;
    }

    if must_build || now - seen >= delay {
        build_problem := build_schema_database(db, config, migrations, schema_database_name(full));
        if build_problem return .{}, build_problem;

        return .{ database = schema_database_name(full), built = true }, "";
    }

    builder: String_Builder;
    builder.allocator = temp;

    for best + 1..migrations.count - 1 print_to_builder(*builder, "%\n;\n", migrations[it].sql);

    return .{ database = schema_database_name(hashes[best]), setup = builder_to_string(*builder) }, "";
}

// Drops schema databases beyond the newest `SCHEMA_DATABASES_KEPT`. One still in use (by another
// editor, say) is left for next time.
prune_schema_databases :: (db: *Connection) {
    names, error := schema_database_names(db);

    if error.kind != .NONE {
        free_error(*error);
        return;
    }

    for names {
        if it_index < SCHEMA_DATABASES_KEPT continue;

        dropped := execute_script(db, tprint("drop database if exists \"%\"", it));
        free_error(*dropped);
    }
}

#scope_module

// The schema databases on the server, newest first. Half-built ones (`jacket_build_...`) are not
// among them.
schema_database_names :: (db: *Connection) -> [] string, Pg_Error {
    result, error := query_dynamic(db, "select datname::text from pg_database where datname ~ '^jacket_[0-9a-f]{12}$' order by oid desc");
    if error.kind != .NONE return .[], error;

    names := NewArray(result.row_count, string,, temp);
    for 0..result.row_count - 1 names[it] = copy_string(text_cell(*result, it, 0),, temp);

    free_result(*result);

    return names, .{};
}

// Creates `name` from template0 under a temporary name, migrates it, and renames it, so another
// worker never sees it half built.
build_schema_database :: (db: *Connection, config: Config, migrations: [] Migration, name: string) -> problem: string {
    building := tprint("jacket_build_%_%", slice(name, 7, 12), to_milliseconds(current_time_monotonic()) % 1_000_000);

    error := execute_script(db, tprint("create database \"%\" template template0", building));

    if error.kind != .NONE return tprint("cannot create a schema database (the role needs CREATEDB): %", error_text(error));

    target := config;
    target.database = building;

    migrated, connect_error := connect(target);

    if connect_error.kind != .NONE {
        drop_built(db, building);
        return tprint("cannot connect to the new schema database: %", error_text(connect_error));
    }

    _, migrate_error := migrate(migrated, migrations);

    close(migrated);

    if migrate_error.kind != .NONE {
        drop_built(db, building);
        return tprint("the migrations failed on a fresh database: %", error_text(migrate_error));
    }

    error = execute_script(db, tprint("alter database \"%\" rename to \"%\"", building, name));

    // Another worker may have built the same one meanwhile.
    if error.kind != .NONE {
        free_error(*error);
        drop_built(db, building);
    }

    return "";
}

drop_built :: (db: *Connection, name: string) {
    error := execute_script(db, tprint("drop database if exists \"%\"", name));
    free_error(*error);
}
