// The pg tool: keeps the committed query descriptions (`db/queries/*.json`) honest, and runs the
// project's migrations and generated schema.
//
//     jaic stdlib/Extensions/Postgres/Tool/main.jai -interp -- prepare [--root DIR] [--build COMMAND] [--prune]
//     jaic stdlib/Extensions/Postgres/Tool/main.jai -interp -- check   [--root DIR] [--build COMMAND]
//     jaic stdlib/Extensions/Postgres/Tool/main.jai -interp -- generate [--root DIR] [--out FILE]
//     jaic stdlib/Extensions/Postgres/Tool/main.jai -interp -- migrate|status [--root DIR]
//     jaic stdlib/Extensions/Postgres/Tool/main.jai -interp -- new NAME [--root DIR]
//     jaic stdlib/Extensions/Postgres/Tool/main.jai -interp -- diff [--root DIR] [--name NAME]
//
// `prepare` describes every entry again against DATABASE_URL (or JACKET_DATABASE_URL) and rewrites
// the ones that changed. `check` does the same without writing, and fails when an entry differs,
// is unused, or a compiled query has no entry. "Compiled" and "unused" come from
// `db/queries/used.txt`, which a build writes while JACKET_PG_CHECK is set; `--build COMMAND`
// runs your build that way first. See docs/postgres/typed-queries.md.
//
// `generate` reads the migrations' schema database on the server DATABASE_URL names and writes
// `db/schema.jai` (or FILE) from it. `migrate` applies db/migrations to DATABASE_URL itself,
// `status` lists each migration's state there, and `new NAME` creates the next
// `db/migrations/NNNN_name.sql`. `diff` compares the structs of a hand-edited `db/schema.jai`
// with the migrations' schema database and writes the next migration as a proposal; it applies
// nothing (docs/postgres/diff.md). See docs/postgres/schema.md.

#import "Basic";
#import "String";
#import "File";
Typed_Files :: #import "File_Utilities";
Io :: #import "Extensions/Io";
Json :: #import "Extensions/Json";
Pg :: #import "Extensions/Postgres";
#import "Extensions/Environment";

libc :: #system_library "libc";
system :: (command: *u8) -> s32 #foreign libc;

Options :: struct {
    command: string;
    root: string;
    build: string;
    prune: bool;
    out: string;
    name: string; // the argument of `new`
}

JSON_SUFFIX :: ".json";
COMMANDS :: string.["prepare", "check", "generate", "migrate", "status", "new", "diff"];

options: Options;
failures := 0;

main :: () {
    arguments := get_command_line_arguments();

    // The first argument is the program itself.
    i := 1;

    while i < arguments.count {
        argument := arguments[i];

        if argument == "--root" && i + 1 < arguments.count {
            options.root = arguments[i + 1];
            i += 1;
        } else if argument == "--build" && i + 1 < arguments.count {
            options.build = arguments[i + 1];
            i += 1;
        } else if argument == "--out" && i + 1 < arguments.count {
            options.out = arguments[i + 1];
            i += 1;
        } else if argument == "--name" && i + 1 < arguments.count {
            options.name = arguments[i + 1];
            i += 1;
        } else if argument == "--prune" {
            options.prune = true;
        } else if !options.command {
            options.command = argument;
        } else if options.command == "new" && !options.name {
            options.name = argument;
        } else {
            print("unexpected argument %\n", argument);
            exit(2);
        }

        i += 1;
    }

    if !array_contains(COMMANDS, options.command) || options.command == "new" && !options.name {
        print("usage: pg prepare [--root DIR] [--build COMMAND] [--prune]\n       pg check [--root DIR] [--build COMMAND]\n       pg generate [--root DIR] [--out FILE]\n       pg migrate|status [--root DIR]\n       pg new NAME [--root DIR]\n       pg diff [--root DIR] [--name NAME]\n");
        exit(2);
    }

    if !options.root options.root = Pg.find_project_root(join(get_working_directory(), "/x",, temp));

    Io.io_run(work);

    exit(ifx failures then 1 else 0);
}

problem :: (message: string) {
    print("%\n", message);
    failures += 1;
}

work :: () {
    if options.command == "generate" generate_schema_file();
    else if options.command == "migrate" migrate_database();
    else if options.command == "status" print_status();
    else if options.command == "new" new_migration();
    else if options.command == "diff" propose_diff();
    else describe_entries();
}

// The compile-time database's settings from the environment, or a message on `problem`.
database_config :: () -> Pg.Config, bool {
    url := environment_variable("JACKET_DATABASE_URL");
    if !url url = environment_variable("DATABASE_URL");

    if !url {
        problem("set DATABASE_URL (or JACKET_DATABASE_URL) to reach the database");
        return .{}, false;
    }

    config, error := Pg.parse_url(url);

    if error.kind != .NONE {
        problem(tprint("bad database URL: %", Pg.error_text(error)));
        return .{}, false;
    }

    return config, true;
}

describe_entries :: () {
    check_only := options.command == "check";
    queries := join(options.root, "/db/queries",, temp);
    used_path := join(queries, "/used.txt",, temp);

    if options.build {
        if Typed_Files.file_exists(used_path) file_delete(used_path);

        status := system(temp_c_string(tprint("JACKET_PG_CHECK=1 JACKET_PG_ROOT=\"%\" %", options.root, options.build)));

        if status != 0 {
            problem(tprint("the build failed (%)", status));
            return;
        }
    }

    config, config_ok := database_config();
    if !config_ok return;

    db, error := Pg.connect(config);

    if error.kind != .NONE {
        problem(tprint("cannot connect: %", Pg.error_text(error)));
        return;
    }

    defer Pg.close(db);

    have_used := Typed_Files.file_exists(used_path);
    used: [..] string;

    if have_used {
        text, _ := read_entire_file(used_path);

        for split(text, "\n") if trim(it) array_add(*used, trim(it));
    }

    _, schema_hash := Pg.migrations_of(options.root);
    present: [..] string;
    changed := 0;

    if Typed_Files.is_directory(queries) {
        for Typed_Files.file_list(queries) {
            if !ends_with(it, ".json") continue;

            id := path_filename(it);
            id.count -= JSON_SUFFIX.count;
            array_add(*present, id);

            if have_used && !array_contains(used, id) {
                if check_only {
                    problem(tprint("unused entry %.json: no compiled query has this id", id));
                } else if options.prune {
                    file_delete(it);
                    print("removed unused %.json\n", id);
                } else {
                    print("unused entry %.json (--prune removes it)\n", id);
                }

                continue;
            }

            if refresh(db, it, id, schema_hash, check_only) changed += 1;
        }
    }

    if have_used for used if !array_contains(present, it) {
        problem(tprint("the query % was compiled but has no entry: run `prepare`", it));
    } else if !have_used && check_only {
        print("note: no db/queries/used.txt, so unused and missing entries were not checked (use --build)\n");
        break;
    }

    print("% entr% %\n", present.count, ifx present.count == 1 then "y" else "ies", ifx check_only then tprint("checked, % differ", changed) else tprint("described, % rewritten", changed));
}

array_contains :: (array: [] string, value: string) -> bool {
    for array if it == value return true;
    return false;
}

// Describes one entry again. True when it differs (and, unless checking, was rewritten).
refresh :: (db: *Pg.Connection, path: string, id: string, schema_hash: string, check_only: bool) -> bool {
    text, read := read_entire_file(path);

    if !read {
        problem(tprint("cannot read %", path));
        return false;
    }

    document, parse_error := Json.from_json(Json.Json_Value, text);

    if parse_error.kind != .NONE {
        problem(tprint("%: not JSON: %", path, parse_error.message));
        return false;
    }

    old, ok := Pg.entry_from_json(*document);

    if !ok {
        problem(tprint("%: not a query description", path));
        return false;
    }

    request := Pg.Describe_Request.{ op = "describe", sql = old.sql, search_path = old.search_path, schema_hash = schema_hash };
    answer_text := Pg.describe_answer(db, request);
    answer, answer_error := Json.from_json(Json.Json_Value, answer_text);

    if answer_error.kind != .NONE {
        problem(tprint("%: unreadable answer", id));
        return false;
    }

    kind := member_text(*answer, "kind");

    if kind != "ok" {
        details := Json.json_get(*answer, "error");
        message := ifx details then member_text(details, "message") else member_text(*answer, "message");
        problem(tprint("%.json: %", id, message));

        return false;
    }

    fresh, fresh_ok := Pg.entry_from_json(Json.json_get(*answer, "entry"));

    if !fresh_ok {
        problem(tprint("%.json: the description is not readable", id));
        return false;
    }

    // The schema hash alone does not make an entry differ.
    probe := fresh;
    probe.schema_hash = old.schema_hash;
    same := Json.to_json(Pg.entry_to_json(probe)) == Json.to_json(Pg.entry_to_json(old));

    if same && fresh.schema_hash == old.schema_hash return false;

    if check_only {
        if !same problem(tprint("%.json differs from the database: run `prepare` and commit", id));
        return !same;
    }

    Pg.write_cache_entry(options.root, Pg.statement_id_of_hex(id), fresh);
    if !same print("rewrote %.json\n", id);

    return !same;
}

member_text :: (object: *Json.Json_Value, key: string) -> string {
    member := Json.json_get(object, key);
    if !member return "";

    value, _ := Json.json_as_string(member);

    return value;
}

migrations_folder :: () -> string {
    return join(options.root, "/db/migrations",, temp);
}

// The project's migrations, or a problem reported.
project_migrations :: () -> [] Pg.Migration, bool {
    migrations, why := Pg.read_migrations(migrations_folder());

    if why {
        problem(why);
        return .[], false;
    }

    return migrations, true;
}

connect_database :: () -> *Pg.Connection {
    config, config_ok := database_config();
    if !config_ok return null;

    db, error := Pg.connect(config);

    if error.kind != .NONE {
        problem(tprint("cannot connect: %", Pg.error_text(error)));
        return null;
    }

    return db;
}

generate_schema_file :: () {
    config, config_ok := database_config();
    if !config_ok return;

    source, why := Pg.generate_schema(config, migrations_folder());

    if why {
        problem(why);
        return;
    }

    path := options.out;
    if !path path = join(options.root, "/db/schema.jai",, temp);

    if !write_entire_file(path, source) {
        problem(tprint("cannot write %", path));
        return;
    }

    print("wrote %\n", path);
}

migrate_database :: () {
    migrations, ok := project_migrations();
    if !ok return;

    db := connect_database();
    if !db return;

    defer Pg.close(db);

    applied, error := Pg.migrate(db, migrations);

    if error.kind != .NONE {
        problem(Pg.error_text(error));
        return;
    }

    print("applied % migration%\n", applied, ifx applied == 1 then "" else "s");
}

print_status :: () {
    migrations, ok := project_migrations();
    if !ok return;

    db := connect_database();
    if !db return;

    defer Pg.close(db);

    statuses, error := Pg.migration_status(db, migrations);

    if error.kind != .NONE {
        problem(Pg.error_text(error));
        return;
    }

    for statuses print("%  %\n", it.name, to_lower_copy(tprint("%", it.state),, temp));
}

// Creates `db/migrations/NNNN_name.sql`.
new_migration :: () {
    path, ok := next_migration_path(options.name);
    if !ok return;

    if !write_entire_file(path, tprint("-- %\n", slug(options.name))) {
        problem(tprint("cannot write %", path));
        return;
    }

    print("created %\n", path);
}

// Writes the migration that would bring the migrated schema to the structs of db/schema.jai,
// unless nothing differs.
propose_diff :: () {
    config, config_ok := database_config();
    if !config_ok return;

    proposal, why := Pg.diff_schema(config, migrations_folder(), join(options.root, "/db/schema.jai",, temp));

    if why {
        problem(why);
        return;
    }

    if !proposal.sql {
        print("db/schema.jai matches the migrations: nothing to propose\n");
        return;
    }

    path, ok := next_migration_path(ifx options.name then options.name else "schema changes");
    if !ok return;

    if !write_entire_file(path, proposal.sql) {
        problem(tprint("cannot write %", path));
        return;
    }

    print("created % (% change%, % TODO%): review it, then run `migrate`\n", path, proposal.changes, ifx proposal.changes == 1 then "" else "s", proposal.todos, ifx proposal.todos == 1 then "" else "s");
}

// The path of `db/migrations/NNNN_name.sql`, numbered after the newest migration and as wide as
// it; or a problem reported.
next_migration_path :: (label: string) -> string, bool {
    name := slug(label);

    if !name {
        problem(tprint("\"%\" has no letters or digits to name a migration with", label));
        return "", false;
    }

    folder := migrations_folder();
    migrations, why := Pg.read_migrations(folder);

    if why {
        problem(why);
        return "", false;
    }

    next := 1;
    width := 4;

    for migrations {
        if it.version >= next next = it.version + 1;
        digits := find_index_from_left(it.name, #char "_");
        if digits > width width = digits;
    }

    number := tprint("%", next);
    while number.count < width number = tprint("0%", number);

    make_directory_if_it_does_not_exist(folder, true);

    return join(folder, "/", number, "_", name, ".sql",, temp), true;
}

// "Add users table" as "add_users_table".
slug :: (text: string) -> string {
    builder: String_Builder;
    builder.allocator = temp;
    lowered := to_lower_copy(text,, temp);
    pending := false;

    for lowered {
        if is_alnum(it) && it < 0x80 {
            if pending && builder_string_length(*builder) append(*builder, "_");
            pending = false;
            print_to_builder(*builder, "%", slice(lowered, it_index, 1));
        } else {
            pending = true;
        }
    }

    return builder_to_string(*builder,, temp);
}
