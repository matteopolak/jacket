// `generate` and the schema check. The text of the generated file for a hand-made catalog, the
// enum spellings and the staleness check run anywhere; with JAI_PG_TEST_PORT set (see
// start_server.sh) it also migrates tests/schema/migrations into a scratch database, compares the
// generated file with tests/schema/expected_schema.jai and round-trips enums through the codecs.
#import "Basic";
#import "String";
#import "File";
#import "POSIX";
#import "Extensions/Environment";
Io :: #import "Extensions/Io";
Pg :: #import "Extensions/Postgres";

main :: () {
    test_enum_names();
    test_source();
    test_problems();
    test_staleness();

    port_text := environment_variable("JAI_PG_TEST_PORT");

    if port_text {
        _, valid := string_to_int(port_text);

        assert(valid, "bad JAI_PG_TEST_PORT");
        Io.io_run(() {
            test_server(cast(u16) string_to_int(environment_variable("JAI_PG_TEST_PORT")));
        });
    }

    print("ok\n");
}

test_enum_names :: () {
    labels :: string.["pending", "in progress", "if", "1st", "a__b", "_x", "ünï", "a-b", "string", "ok_20"];

    for labels {
        name := Pg.enum_member_name(it);

        assert(Pg.enum_label(name) == it, "% -> % -> %", it, name, Pg.enum_label(name));
        // Always a plain Jai name.
        for name assert((is_alnum(it) && it < 0x80) || it == #char "_", "% is not a name", name);
    }

    assert(Pg.enum_member_name("pending") == "pending");
    assert(Pg.enum_member_name("in progress") == "__in_20progress");
    assert(Pg.enum_member_name("if") == "__if");
    assert(Pg.enum_member_name("1st") == "__1st");
}

column :: (name: string, base: string, nullable := false) -> Pg.Catalog_Column {
    return .{ name = name, sql_type = base, base = base, nullable = nullable };
}

// An array literal of variables lives on the stack, so the catalog copies what it keeps.
keep :: ($T: Type, items: [] T) -> [] T {
    return array_copy(items);
}

catalog :: () -> Pg.Catalog {
    c: Pg.Catalog;

    mood := Pg.Catalog_Enum.{ key = "\"public\".\"mood\"", schema = "public", name = "mood", labels = .["sad", "in progress"] };

    c.enums = keep(Pg.Catalog_Enum, .[mood]);

    teams := Pg.Catalog_Relation.{ schema = "public", name = "teams" };
    id := column("id", "int8");
    id.primary_key = true;
    id.has_default = true;
    teams.columns = keep(Pg.Catalog_Column, .[id, column("name", "text")]);

    users := Pg.Catalog_Relation.{ schema = "public", name = "users" };
    mood_column := column("mood", "enum", true);
    mood_column.enum_key = mood.key;
    moods := column("past_moods", "enum[]");
    moods.enum_key = mood.key;
    users.columns = keep(Pg.Catalog_Column, .[id, column("team_id", "int8"), mood_column, moods, column("settings", "jsonb", true), column("tags", "text[]"), column("born", "date", true)]);

    audit := Pg.Catalog_Relation.{ schema = "audit", name = "users" };
    audit.columns = keep(Pg.Catalog_Column, .[id, column("at", "timestamptz")]);

    names := Pg.Catalog_Relation.{ schema = "public", name = "team_names", view = true };
    names.columns = keep(Pg.Catalog_Column, .[column("name", "text", true)]);

    c.relations = keep(Pg.Catalog_Relation, .[audit, names, teams, users]);
    c.foreign_keys = keep(Pg.Catalog_Foreign_Key, .[
        .{ schema = "public", table = "users", columns = .["team_id"], target_schema = "public", target_table = "teams", target_columns = .["id"] },
        .{ schema = "audit", table = "users", columns = .["id"], target_schema = "public", target_table = "users", target_columns = .["id"] },
    ]);

    return c;
}

read_fixture :: (name: string) -> string {
    text, ok := read_entire_file(join(#filepath, "schema/", name,, temp));
    assert(ok, "cannot read %", name);

    return text;
}

// The text with the generated file's trailing blank line trimmed, for a golden comparison.
test_source :: () {
    source, problem := Pg.schema_source(catalog(), "abc123");

    assert(problem == "", "%", problem);
    assert(trim(source) == trim(read_fixture("offline_expected.txt")), "the generated text differs from tests/schema/offline_expected.txt:\n%", source);
}

test_problems :: () {
    // A column named like a keyword cannot be a field.
    c: Pg.Catalog;
    keyword := Pg.Catalog_Relation.{ schema = "public", name = "t" };
    keyword.columns = keep(Pg.Catalog_Column, .[column("if", "int4")]);
    c.relations = keep(Pg.Catalog_Relation, .[keyword]);

    source, problem := Pg.schema_source(c, "h");

    assert(problem != "" && contains(problem, "public.t") && contains(problem, "\"if\"") && contains(problem, "cannot be a field name"), "%", problem);

    // `user` and `users` both want `User`: the second gets a longer name.
    one := Pg.Catalog_Relation.{ schema = "public", name = "user" };
    two := Pg.Catalog_Relation.{ schema = "public", name = "users" };

    c.relations = keep(Pg.Catalog_Relation, .[one, two]);
    source, problem = Pg.schema_source(c, "h");

    assert(problem == "", "%", problem);
    assert(contains(source, "User :: struct @pg_table(public.user)") && contains(source, "Users :: struct @pg_table(public.users)"), "%", source);

    // A table that is not a Jai name at all.
    odd := Pg.Catalog_Relation.{ schema = "public", name = "my-table" };

    c.relations = keep(Pg.Catalog_Relation, .[odd]);
    source, problem = Pg.schema_source(c, "h");

    assert(contains(problem, "my-table"), "%", problem);
}

test_staleness :: () {
    root := tprint("/tmp/jacket-schema-test-%", cast(s64) getpid());
    folder := tprint("%/db/migrations", root);
    first := tprint("%/0001_a.sql", folder);
    second := tprint("%/0002_b.sql", folder);
    path := tprint("%/db/schema.jai", root);

    make_directory_if_it_does_not_exist(folder, true);

    defer {
        for string.[first, second, path] file_delete(it);
        for string.[folder, tprint("%/db", root), root] delete_directory(it);
    }

    assert(write_entire_file(first, "create table a ();\n"));

    _, hash := Pg.migrations_of(root);

    assert(hash.count == 64);

    source, problem := Pg.schema_source(.{}, hash);

    assert(problem == "" && Pg.recorded_schema_hash(source) == hash);
    assert(write_entire_file(path, source));

    assert(Pg.schema_problem(path) == "", "%", Pg.schema_problem(path));

    // Editing, adding or removing a migration makes the file stale.
    assert(write_entire_file(first, "create table a (id int8);\n"));
    assert(contains(Pg.schema_problem(path), "stale"), "%", Pg.schema_problem(path));
    assert(write_entire_file(first, "create table a ();\n"));
    assert(Pg.schema_problem(path) == "");

    assert(write_entire_file(second, "create table b ();\n"));
    assert(contains(Pg.schema_problem(path), "stale") && contains(Pg.schema_problem(path), "generate"));

    // A file with no recorded hash is not trusted either.
    assert(write_entire_file(path, "A :: struct {}\n"));
    assert(contains(Pg.schema_problem(path), "no schema hash"));
}

Mood :: enum {
    sad;
    __in_20progress;
    __if;
}

Small :: enum {
    sad;
}

test_server :: (port: u16) {
    config: Pg.Config;
    config.host = "127.0.0.1";
    config.port = port;
    config.user = "postgres";
    config.database = "postgres";
    config.ssl_mode = .DISABLE;

    // The fixture migrations, through their schema database.
    source, problem := Pg.generate_schema(config, join(#filepath, "schema/migrations",, temp));

    assert(problem == "", "%", problem);
    assert(source == read_fixture("expected_schema.txt"), "the generated text differs from tests/schema/expected_schema.txt:\n%", source);

    source, problem = Pg.generate_schema(config, join(#filepath, "migrations_bad",, temp));
    assert(source == "" && contains(problem, "migration 0001_fails failed"), "%", problem);

    db, error := Pg.connect(config);

    assert(error.kind == .NONE, "%", Pg.error_text(error));
    defer Pg.close(db);

    leftovers:, error = Pg.query_dynamic(db, "select count(*) from pg_database where datname like 'jacket\\_build\\_%'");
    count, _ := Pg.get(*leftovers, 0, 0, s64);

    assert(error.kind == .NONE && count == 0, "% half-built schema databases were left behind", count);
    Pg.free_result(*leftovers);

    test_enums(db);
    test_json_raw(db);
}

query_one_row :: (db: *Pg.Connection, sql: string, args: ..Any) -> Pg.Result {
    values := args;
    result, error := Pg.query_dynamic(db, sql, ..values);
    assert(error.kind == .NONE, "%: %", sql, Pg.error_text(error));

    return result;
}

// A Jai enum is a Postgres enum's label, as a parameter, a column and an array, in both formats.
test_enums :: (db: *Pg.Connection) {
    error := Pg.execute_script(db, "set client_min_messages = warning; drop type if exists schema_test_mood; create type schema_test_mood as enum ('sad', 'in progress', 'if')");

    assert(error.kind == .NONE, "%", Pg.error_text(error));

    defer {
        dropped := Pg.execute_script(db, "drop type if exists schema_test_mood cascade");
        Pg.free_error(*dropped);
    }

    result := query_one_row(db, "select $1::schema_test_mood, $2::schema_test_mood[], null::schema_test_mood", Mood.__in_20progress, Mood.[.__if, .sad]);
    mood, e1 := Pg.get(*result, 0, 0, Mood);
    moods, e2 := Pg.get(*result, 0, 1, [] Mood);
    none, e3 := Pg.get(*result, 0, 2, Pg.Nullable(Mood));

    assert(e1.kind == .NONE && mood == .__in_20progress, "%", Pg.error_text(e1));
    assert(e2.kind == .NONE && moods.count == 2 && moods[0] == .__if && moods[1] == .sad, "%", Pg.error_text(e2));
    assert(e3.kind == .NONE && none.null);
    Pg.free_result(*result);

    // Binary results carry the label too.
    binary, binary_error := Pg.query_binary(db, "select 'if'::schema_test_mood, array['sad', 'in progress']::schema_test_mood[]");

    assert(binary_error.kind == .NONE, "%", Pg.error_text(binary_error));
    mood, e1 = Pg.get(*binary, 0, 0, Mood);
    moods, e2 = Pg.get(*binary, 0, 1, [] Mood);

    assert(e1.kind == .NONE && mood == .__if, "%", Pg.error_text(e1));
    assert(e2.kind == .NONE && moods.count == 2 && moods[1] == .__in_20progress, "%", Pg.error_text(e2));
    Pg.free_result(*binary);

    // A label the Jai enum does not have is a decode error naming it.
    result = query_one_row(db, "select 'if'::schema_test_mood");
    _, e1 = Pg.get(*result, 0, 0, Small);
    assert(e1.kind == .DECODE && contains(e1.message, "\"if\" is not a label"), "%", Pg.error_text(e1));
    Pg.free_result(*result);
}

// A json column nobody mapped is `Pg.Json(Pg.Json_Raw)`: the document's text, both ways.
test_json_raw :: (db: *Pg.Connection) {
    raw := Pg.Json(Pg.Json_Raw).{ value = .{ text = "{\"a\": [1, 2]}" } };
    result := query_one_row(db, "select $1::jsonb, $1::jsonb->'a'->>1", raw);
    document, e1 := Pg.get(*result, 0, 0, Pg.Json(Pg.Json_Raw));
    second, e2 := Pg.get(*result, 0, 1, string);

    assert(e1.kind == .NONE && contains(document.value.text, "\"a\"") && e2.kind == .NONE && second == "2", "%", Pg.error_text(e1));
    Pg.free_result(*result);
}
