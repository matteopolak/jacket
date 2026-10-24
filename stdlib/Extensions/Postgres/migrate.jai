// Forward-only SQL migrations, embedded in the program at compile time and applied at startup.
//
//     MIGRATIONS :: #run Pg.load_migrations("db/migrations");    // relative to this file
//     applied, error := Pg.migrate(db, MIGRATIONS);
//
// Files are named `NNNN_name.sql`. One whose first line is `-- jacket: no-transaction` runs
// outside a transaction, one statement at a time (`create index concurrently` needs that).

Migration :: struct {
    version: s64; // the number the file name starts with
    name: string; // the file name without `.sql`
    sql: string;
    sha256: string; // hex digest of `sql`
    transaction := true; // false for `-- jacket: no-transaction`
}

Migrate_Options :: struct {
    // Apply even when an applied migration's file changed or is missing from the list.
    allow_modified := false;
    // Apply migrations numbered below the newest applied one (merged from another branch).
    // Without it they are refused, since they were written against an older schema.
    allow_out_of_order := false;
}

NO_TRANSACTION_MARKER :: "-- jacket: no-transaction";
MIGRATIONS_TABLE :: "jacket_migrations";

// The migrations in `folder`, sorted by version, for embedding: `#run load_migrations(...)`.
// A relative folder starts at the calling file's folder. A badly named file or a repeated version
// stops the compile.
load_migrations :: (folder: string, location := #caller_location) -> [] Migration {
    migrations, problem := read_migrations(folder, location.fully_pathed_filename);
    if problem {
        if #compile_time Compiler.compiler_report(problem, location);
        else log_error("%", problem);
    }

    return migrations;
}

// `load_migrations` for tools: the migrations, or a message saying what is wrong with the folder.
// `relative_to` is a file whose folder relative paths start at; empty for the working directory.
read_migrations :: (folder: string, relative_to := "") -> [] Migration, problem: string {
    path := folder;

    if relative_to && !is_absolute_path(folder) {
        path = join(path_strip_filename(relative_to), folder,, temp);
    }

    migrations: [..] Migration;
    files := File_Utilities.file_list(path);

    for file: files {
        base := path_filename(file);
        if !ends_with(base, ".sql") continue;

        version, name_ok := parse_migration_name(base);
        if !name_ok return .[], tprint("migration file % must be named NNNN_name.sql", file);

        sql, read_ok := read_entire_file(file);
        if !read_ok return .[], tprint("cannot read migration %", file);

        migration: Migration;
        migration.version = version;
        migration.name = copy_string(slice(base, 0, base.count - 4));
        migration.sql = sql;
        migration.sha256 = sha256_hex(sql);
        migration.transaction = !begins_with(trim_left(sql), NO_TRANSACTION_MARKER);
        array_add(*migrations, migration);
    }

    quick_sort(migrations, (a: Migration, b: Migration) -> s64 {
        return a.version - b.version;
    });

    for 1..migrations.count - 1 {
        if migrations[it].version == migrations[it - 1].version {
            return .[], tprint("migrations % and % have the same version", migrations[it - 1].name, migrations[it].name);
        }
    }

    return migrations, "";
}

// The schema hash: SHA-256 chained over each migration's name and digest, in order, so it changes
// when any file is renamed, edited, added or removed. Hex.
schema_hash :: (migrations: [] Migration) -> string {
    if !migrations.count return "";

    hashes := prefix_hashes(migrations);
    return hashes[hashes.count - 1];
}

// The schema hash of each prefix of `migrations`: entry i covers migrations 0 to i.
prefix_hashes :: (migrations: [] Migration) -> [] string {
    hashes := NewArray(migrations.count, string,, temp);
    chained := "";

    for migrations {
        chained = sha256_hex(tprint("%\n%\n%", chained, it.name, it.sha256));
        hashes[it_index] = chained;
    }

    return hashes;
}

// Where one migration stands against a database.
Migration_State :: enum u8 {
    PENDING; // not applied yet
    APPLIED;
    MODIFIED; // applied, but the file changed since
    MISSING; // applied, but no longer in the list (the entry has only version and name)
    OUT_OF_ORDER; // pending, but numbered below the newest applied migration
}

Migration_Status :: struct {
    version: s64;
    name: string;
    state: Migration_State;
}

// Applies the pending migrations in order, each in its own transaction, and returns how many it
// applied. An advisory lock keeps two processes from migrating at once. It refuses (applying
// nothing) when an applied migration changed or disappeared, or a pending one is numbered below
// the newest applied one, unless `options` allows it.
migrate :: (c: *Connection, migrations: [] Migration, options := Migrate_Options.{}) -> applied: s64, Pg_Error {
    error := execute_script(c, "select pg_advisory_lock(hashtext('jacket_migrations'))");
    if error.kind != .NONE return 0, error;

    defer {
        unlock := execute_script(c, "select pg_advisory_unlock(hashtext('jacket_migrations'))");
        free_error(*unlock);
    }

    status:, error = migration_status(c, migrations);

    if error.kind != .NONE return 0, error;

    refusal := check_status(status, options);
    if refusal.kind != .NONE return 0, refusal;

    applied := 0;

    for migration: migrations {
        state := status[it_index].state;
        if state != .PENDING && state != .OUT_OF_ORDER continue;

        error = apply_migration(c, migration);
        if error.kind != .NONE return applied, error;

        applied += 1;
    }

    return applied, .{};
}

// Each migration's state, in the list's order, then the applied ones missing from the list.
// Creates the migrations table if it does not exist.
migration_status :: (c: *Connection, migrations: [] Migration) -> [] Migration_Status, Pg_Error {
    // `if not exists` alone would send a notice on every run.
    error := execute_script(c, #string SQL
        do $$ begin if to_regclass('jacket_migrations') is null then create table jacket_migrations (
            version int8 primary key,
            name text not null,
            sha256 text not null,
            applied_at timestamptz not null default now(),
            duration_ms int8 not null
        ); end if; end $$
    SQL);
    if error.kind != .NONE return .[], error;

    result:, error = query_dynamic(c, "select version, name, sha256 from jacket_migrations order by version");

    if error.kind != .NONE return .[], error;

    defer free_result(*result);

    status := NewArray(migrations.count, Migration_Status, initialized = true,, temp);
    newest_applied := -1;

    for migrations status[it_index] = .{ it.version, it.name, .PENDING };

    missing: [..] Migration_Status;
    missing.allocator = temp;

    for row: 0..result.row_count - 1 {
        version, _ := get(*result, row, 0, s64);
        name, _ := get(*result, row, 1, string);
        digest, _ := get(*result, row, 2, string);

        newest_applied = max(newest_applied, version);

        index := find_version(migrations, version);
        if index < 0 {
            array_add(*missing, .{ version, copy_temporary_string(name), .MISSING });
        } else if migrations[index].sha256 != digest {
            status[index].state = .MODIFIED;
        } else {
            status[index].state = .APPLIED;
        }
    }

    for *status if it.state == .PENDING && it.version < newest_applied it.state = .OUT_OF_ORDER;

    all: [..] Migration_Status;
    all.allocator = temp;
    array_add(*all, ..status);
    array_add(*all, ..missing);

    return all, .{};
}

// Splits a script into its statements, skipping quoted text, dollar quotes and comments, so a `;`
// inside them does not end a statement. Empty statements are dropped. The pieces point into `sql`.
split_statements :: (sql: string) -> [] string {
    statements: [..] string;
    statements.allocator = temp;
    start := 0;
    i := 0;

    while i < sql.count {
        if sql[i] == #char ";" {
            add_statement(*statements, slice(sql, start, i - start));
            i += 1;
            start = i;
            continue;
        }

        i = skip_quoted(sql, i);
    }

    add_statement(*statements, slice(sql, start, sql.count - start));

    return statements;
}

#scope_file

File_Utilities :: #import "File_Utilities";
Compiler :: #import "Compiler";
#import "Sort";

parse_migration_name :: (base: string) -> version: s64, ok: bool {
    digits := 0;
    while digits < base.count && is_digit(base[digits]) digits += 1;
    if digits == 0 || digits >= base.count || base[digits] != #char "_" return 0, false;
    if digits + 1 >= base.count - 4 return 0, false; // no name before `.sql`

    version, ok := string_to_int(slice(base, 0, digits));

    return version, ok;
}

sha256_hex :: (text: string) -> string {
    digest := Sha.sha256(text);
    builder: String_Builder;
    builder.allocator = temp;
    for digest print_to_builder(*builder, "%", formatInt(it, base = 16, minimum_digits = 2));

    return builder_to_string(*builder,, temp);
}

find_version :: (migrations: [] Migration, version: s64) -> s64 {
    for migrations if it.version == version return it_index;
    return -1;
}

check_status :: (status: [] Migration_Status, options: Migrate_Options) -> Pg_Error {
    for status {
        if it.state == {
            case .MODIFIED;
                if options.allow_modified continue;

                return make_error(.USAGE, "migration % was applied, but its file has changed since; restore it, or migrate with allow_modified", it.name);
            case .MISSING;
                if options.allow_modified continue;

                return make_error(.USAGE, "migration % was applied, but it is not in the list; restore it, or migrate with allow_modified", it.name);
            case .OUT_OF_ORDER;
                if options.allow_out_of_order continue;

                return make_error(.USAGE, "migration % is numbered below one already applied; renumber it, or migrate with allow_out_of_order", it.name);
        }
    }

    return .{};
}

apply_migration :: (c: *Connection, migration: Migration) -> Pg_Error {
    started := current_time_monotonic();

    if migration.transaction {
        error := execute_script(c, "begin");
        if error.kind != .NONE return error;

        error = execute_script(c, migration.sql);
        if error.kind == .NONE error = record_migration(c, migration, started);
        if error.kind == .NONE error = execute_script(c, "commit");

        if error.kind != .NONE {
            rollback := execute_script(c, "rollback");
            free_error(*rollback);
            return failed_migration(migration, error);
        }

        return .{};
    }

    for split_statements(migration.sql) {
        error := execute_script(c, it);
        if error.kind != .NONE return failed_migration(migration, error);
    }

    return record_migration(c, migration, started);
}

record_migration :: (c: *Connection, migration: Migration, started: Apollo_Time) -> Pg_Error {
    duration := to_milliseconds(current_time_monotonic() - started);
    result, error := query_dynamic(c, "insert into jacket_migrations (version, name, sha256, duration_ms) values ($1, $2, $3, $4)", migration.version, migration.name, migration.sha256, duration);
    free_result(*result);
    return error;
}

// The server's error, with the migration's name in front of its message.
failed_migration :: (migration: Migration, error: Pg_Error) -> Pg_Error {
    if error.kind != .SERVER return error;

    named := make_error(.SERVER, "migration % failed: %", migration.name, error.message);
    named.code = error.code;
    named.position = error.position;
    free_error(*error);

    return named;
}

add_statement :: (statements: *[..] string, text: string) {
    trimmed := trim(text);
    if only_comments(trimmed) return;
    array_add(statements, trimmed);
}

only_comments :: (text: string) -> bool {
    i := 0;
    while i < text.count {
        if is_space(text[i]) {
            i += 1;
            continue;
        }

        next := skip_quoted(text, i);
        if next == i + 1 return false; // an ordinary character
        if text[i] != #char "-" && text[i] != #char "/" return false; // quoted text

        i = next;
    }

    return true;
}

// The index after the quoted text, comment or dollar quote at `i`, or `i + 1` for anything else.
skip_quoted :: (sql: string, i: s64) -> s64 {
    c := sql[i];
    next := ifx i + 1 < sql.count then sql[i + 1] else 0;

    if c == #char "'" || c == #char "\"" {
        backslashes := c == #char "'" && i > 0 && (sql[i - 1] == #char "E" || sql[i - 1] == #char "e");
        j := i + 1;

        while j < sql.count {
            if backslashes && sql[j] == #char "\\" {
                j += 2;
                continue;
            }

            if sql[j] == c {
                if j + 1 < sql.count && sql[j + 1] == c {
                    j += 2;
                    continue;
                }

                return j + 1;
            }

            j += 1;
        }

        return sql.count;
    }

    if c == #char "-" && next == #char "-" {
        j := i;
        while j < sql.count && sql[j] != #char "\n" j += 1;
        return j;
    }

    if c == #char "/" && next == #char "*" {
        depth := 0;
        j := i;

        while j < sql.count {
            if sql[j] == #char "/" && j + 1 < sql.count && sql[j + 1] == #char "*" {
                depth += 1;
                j += 2;
            } else if sql[j] == #char "*" && j + 1 < sql.count && sql[j + 1] == #char "/" {
                depth -= 1;
                j += 2;
                if depth == 0 return j;
            } else {
                j += 1;
            }
        }

        return sql.count;
    }

    if c == #char "$" {
        tag_end := dollar_tag_end(sql, i);
        if tag_end > 0 {
            tag := slice(sql, i, tag_end - i);
            body := slice(sql, tag_end, sql.count - tag_end);
            found := find_index_from_left(body, tag);

            if found < 0 return sql.count;

            return tag_end + found + tag.count;
        }
    }

    return i + 1;
}

// For a dollar-quote opener `$tag$` at `i`, the index after it; 0 when `i` does not start one
// (`$1` is a parameter, and a tag cannot follow an identifier character).
dollar_tag_end :: (sql: string, i: s64) -> s64 {
    if i > 0 && (is_alnum(sql[i - 1]) || sql[i - 1] == #char "_") return 0;

    j := i + 1;
    while j < sql.count && (is_alpha(sql[j]) || sql[j] == #char "_" || (j > i + 1 && is_digit(sql[j]))) j += 1;
    if j < sql.count && sql[j] == #char "$" return j + 1;

    return 0;
}
