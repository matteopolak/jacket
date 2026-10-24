// The pg tool: keeps the committed query descriptions (`db/queries/*.json`) honest.
//
//     jaic stdlib/Extensions/Postgres/Tool/main.jai -interp -- prepare [--root DIR] [--build COMMAND] [--prune]
//     jaic stdlib/Extensions/Postgres/Tool/main.jai -interp -- check   [--root DIR] [--build COMMAND]
//
// `prepare` describes every entry again against DATABASE_URL (or JACKET_DATABASE_URL) and rewrites
// the ones that changed. `check` does the same without writing, and fails when an entry differs,
// is unused, or a compiled query has no entry. "Compiled" and "unused" come from
// `db/queries/used.txt`, which a build writes while JACKET_PG_CHECK is set; `--build COMMAND`
// runs your build that way first. See docs/postgres/typed-queries.md.

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
}

JSON_SUFFIX :: ".json";

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
        } else if argument == "--prune" {
            options.prune = true;
        } else if !options.command {
            options.command = argument;
        } else {
            print("unexpected argument %\n", argument);
            exit(2);
        }

        i += 1;
    }

    if options.command != "prepare" && options.command != "check" {
        print("usage: pg (prepare|check) [--root DIR] [--build COMMAND] [--prune]\n");
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

    url := environment_variable("JACKET_DATABASE_URL");
    if !url url = environment_variable("DATABASE_URL");

    if !url {
        problem("set DATABASE_URL (or JACKET_DATABASE_URL) to describe the queries");
        return;
    }

    config, config_error := Pg.parse_url(url);

    if config_error.kind != .NONE {
        problem(tprint("bad database URL: %", Pg.error_text(config_error)));
        return;
    }

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

    schema_hash := Pg.migrations_hash(options.root);
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
