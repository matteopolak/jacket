// Every typed query macro against a live database, and the nullability rules. Built by run.sh,
// which sets up the schema (schema.sql) and DATABASE_URL.
#import "Basic";
#import "String";
Io :: #import "Extensions/Io";
Pg :: #import "Extensions/Postgres";
#import "Extensions/Json";
#import "Extensions/Environment";

Settings :: struct {
    theme: string;
    font_size := 14; @json(default)
}

Theme_Only :: struct {
    theme: string;
}

Theme_Row :: struct {
    settings: Pg.Nullable(Pg.Json(Theme_Only));
}

User_Summary :: struct {
    id: s64;
    email: string;
    display_name: Pg.Nullable(string);
    settings: Pg.Nullable(Pg.Json(Settings));
    tags: [] string;
}

failures := 0;

check :: (ok: bool, what: string) {
    if ok return;

    failures += 1;
    print("FAILED: %\n", what);
}

main :: () {
    Io.io_run(() {
        config, error := Pg.parse_url(environment_variable("DATABASE_URL"));

        assert(error.kind == .NONE);
        db:, error = Pg.connect(config);

        assert(error.kind == .NONE, "%", Pg.error_text(error));
        defer Pg.close(db);

        test_query(db);
        test_one_and_opt(db);
        test_execute_and_scalar(db);
        test_nullability(db);
        test_arguments(db);
        test_query_as(db);
        test_json_and_arrays(db);
        test_statement_cache(db, config);
        test_pipeline(db);
        test_stream(db);
    });

    if failures {
        print("% failures\n", failures);
        exit(1);
    }

    print("ok\n");
}

test_query :: (db: *Pg.Connection) {
    user_id: s64 = 1;
    rows, error := Pg.query(db, "select u.id, u.email, u.display_name, count(p.id) as posts from users u left join posts p on p.author_id = u.id where u.id = % group by u.id", user_id);

    defer Pg.free_rows(*rows);
    check(error.kind == .NONE, "query");

    seen := 0;

    for rows {
        seen += 1;
        check(it.id == 1 && it.email == "a@example.com", "row values");
        check(!it.display_name.null && it.display_name.value == "Alice", "nullable value");
        check(it.posts == 2, "count is not nullable");
    }

    check(seen == 1, "one row");

    all, all_error := Pg.query(db, "select id, email from users order by id");

    defer Pg.free_rows(*all);
    check(all_error.kind == .NONE, "query without arguments");

    items := Pg.fetch_all(*all);

    defer array_free(items);
    check(items.count == 2 && items[1].email == "b@example.com", "fetch_all");

    // The copies outlive the rows: strings, arrays and json documents included.
    kept, kept_error := Pg.query(db, "select email, tags, settings from users order by id");

    check(kept_error.kind == .NONE, Pg.error_text(kept_error));

    copied := Pg.fetch_all(*kept);

    Pg.free_rows(*kept);

    defer array_free(copied);
    check(copied.count == 2 && copied[0].email == "a@example.com" && copied[0].tags.count == 2 && copied[0].tags[1] == "y", "fetch_all copies");

    copied_temp := Pg.fetch_all(*all, temp);

    check(copied_temp.count == 2 && copied_temp[0].email == "a@example.com", "fetch_all into temp");
}

test_one_and_opt :: (db: *Pg.Connection) {
    row, error := Pg.query_one(db, "select id, display_name from users where id = %", 2);

    check(error.kind == .NONE && row.id == 2 && row.display_name.null, "query_one");

    _, missing := Pg.query_one(db, "select id from users where id = %", 99);

    check(missing.kind == .NOT_FOUND, "query_one not found");

    found_row, found, opt_error := Pg.query_opt(db, "select email from users where id = %", 1);

    check(opt_error.kind == .NONE && found && found_row.email == "a@example.com", "query_opt found");

    _, found, opt_error = Pg.query_opt(db, "select email from users where id = %", 99);
    check(opt_error.kind == .NONE && !found, "query_opt missing");
}

test_execute_and_scalar :: (db: *Pg.Connection) {
    affected, error := Pg.execute(db, "insert into posts (author_id, title) values (%, %)", 2, "temp");

    check(error.kind == .NONE && affected == 1, "execute insert");

    count, count_error := Pg.query_scalar(db, "select count(*) from posts where author_id = %", 2);

    check(count_error.kind == .NONE && count == 1, "query_scalar");

    affected, error = Pg.execute(db, "delete from posts where author_id = % and title = %", 2, "temp");
    check(error.kind == .NONE && affected == 1, "execute delete");

    name, name_error := Pg.query_scalar(db, "select display_name from users where id = %", 1);

    check(name_error.kind == .NONE && !name.null && name.value == "Alice", "nullable scalar");

    inserted, insert_error := Pg.query_one(db, "insert into posts (author_id, title) values (%, %) returning id, body", 2, "ret");

    check(insert_error.kind == .NONE && inserted.body.null, "returning");
    removed, _ := Pg.execute(db, "delete from posts where id = %", inserted.id);

    check(removed == 1, "cleanup");
}

test_nullability :: (db: *Pg.Connection) {
    // The nullable side of an outer join, though the column is NOT NULL in its table.
    rows, error := Pg.query(db, "select u.email, p.title, count(p.id) as n from users u left join posts p on p.author_id = u.id group by u.email, p.title order by u.email, p.title");

    defer Pg.free_rows(*rows);
    check(error.kind == .NONE, "left join");

    for rows {
        check(it.email.count > 0, "inner side is plain");
        if it.email == "b@example.com" check(it.title.null, "outer join side is NULL");
        else check(!it.title.null, "outer join side has a value");
    }

    // Computed columns: nullable unless they are on the short list; overrides win.
    row, row_error := Pg.query_one(db, "select 1 as one, 'a'::text as letter, coalesce(display_name, 'x') as name, exists(select 1 from posts) as any_post, id + 1 as next, id as \"forced!\", display_name as \"maybe?\" from users where id = 1");

    check(row_error.kind == .NONE, "computed");
    check(row.one == 1 && row.letter == "a" && row.name == "Alice" && row.any_post, "short list is not nullable");
    check(!row.next.null && row.next.value == 2, "other computations are nullable");
    check(row.forced == 1, "name! forces non-null");
    check(!row.maybe.null, "name? forces nullable");

    // An override of the Jai type.
    typed, typed_error := Pg.query_one(db, "select score as \"score: s64\", id as \"id: u32\" from users where id = 1");

    check(typed_error.kind == .NONE && typed.score == 0 && typed.id == 1, "type override");

    // A forced non-null column that is NULL is a decode error that names the column.
    _, decode_error := Pg.query_one(db, "select display_name as \"display_name!\" from users where id = 2");

    check(decode_error.kind == .DECODE && contains(decode_error.message, "display_name"), "forced non-null NULL is an error");
}

test_arguments :: (db: *Pg.Connection) {
    none: Pg.Nullable(string);
    none.null = true;

    row, error := Pg.query_one(db, "select count(*) as n from users where display_name is not distinct from %", none);

    check(error.kind == .NONE && row.n == 1, "Nullable(string) argument");

    row, error = Pg.query_one(db, "select count(*) as n from users where display_name is not distinct from %", Pg.NULL);
    check(error.kind == .NONE && row.n == 1, "Pg.NULL argument");

    ids := s64.[1, 2];

    row, error = Pg.query_one(db, "select count(*) as n from users where id = any(%)", ids);
    check(error.kind == .NONE && row.n == 2, "array argument");

    small: s32 = 1;

    row, error = Pg.query_one(db, "select count(*) as n from users where id = %", small);
    check(error.kind == .NONE && row.n == 1, "s32 widens to int8");

    uid: s64 = 1;
    inline_row, inline_error := Pg.query_one(db, "select email from users where id = %uid");

    check(inline_error.kind == .NONE && inline_row.email == "a@example.com", "inline hole");

    modulo, modulo_error := Pg.query_one(db, "select 7 %% 3 as r");

    check(modulo_error.kind == .NONE && modulo.r == 1, "%% is a literal percent");

    like, like_error := Pg.query_one(db, "select count(*) as n from users where email like '%@example.com'");

    check(like_error.kind == .NONE && like.n == 2, "percent inside a string is text");
}

test_query_as :: (db: *Pg.Connection) {
    rows, error := Pg.query_as(User_Summary, db, "select id, email, display_name, settings, tags from users order by id");

    defer Pg.free_rows(*rows);
    check(error.kind == .NONE, "query_as");

    index := 0;

    for rows {
        if index == 0 {
            check(it.id == 1 && it.settings.value.value.theme == "dark" && it.settings.value.value.font_size == 16, "Json(T) field");
            check(it.tags.count == 2 && it.tags[1] == "y", "text[] field");
        } else {
            check(it.settings.null && it.tags.count == 0, "NULL json and empty array");
        }

        index += 1;
    }

    check(rows.error.kind == .NONE, "no decode error");
}

test_json_and_arrays :: (db: *Pg.Connection) {
    row, error := Pg.query_one(db, "select settings, mood, tags, created_at from users where id = 1");

    check(error.kind == .NONE, "json row");
    check(!row.settings.null && contains(row.settings.value.text, "dark"), "jsonb is Json_Raw");
    check(row.mood == "happy", "enum is a string");
    check(row.tags.count == 2, "array");
    check(row.created_at.microseconds > 0, "timestamptz");

    typed, typed_error := Pg.query_one(db, "select settings as \"settings!: Json(Json_Value)\" from users where id = 1");

    check(typed_error.kind == .NONE && typed.settings.value.kind == .OBJECT, "Json(T) override");

    // timestamps take spans: rows created in the last week are all of them, none in the next day
    since := Pg.operator -(Pg.now(), Pg.days(7));
    later := Pg.operator +(Pg.now(), Pg.days(1));

    recent, recent_error := Pg.query_scalar(db, "select count(*) from users where created_at > % and created_at < %", since, later);

    check(recent_error.kind == .NONE && recent == 2, "timestamp arithmetic as arguments");

    // a keyword argument sets the strictness of this query alone, and the error names the column
    lenient, lenient_error := Pg.query_as(Theme_Row, db, "select settings from users where id = 1");

    for lenient {}

    check(lenient_error.kind == .NONE && lenient.error.kind == .NONE, "unknown json keys are ignored by default");

    strict, _ := Pg.query_as(Theme_Row, db, "select settings from users where id = 1", json = .{ unknown_fields = .DENY });

    for strict {}

    check(strict.error.kind == .DECODE && contains(strict.error.message, "column \"settings\" (users.settings), row 0, at $"), "json options per query");
}

parses_sent := 0;

count_parses :: (c: *Pg.Connection, line: string) {
    if contains(line, "Parse\t") parses_sent += 1;
}

// Typed queries prepare once per connection and bind afterwards; `plan = .CUSTOM` opts out, and
// `verify_statements` notices a table that changed under the program.
test_stream :: (db: *Pg.Connection) {
    error: Pg.Pg_Error;
    sum := 0;
    seen := 0;

    for Pg.stream(db, *error, "select g as \"n!\" from generate_series(1, %) as g", 2500, batch = 1000) {
        check(it_index == seen, "it_index counts rows");
        sum += it.n;
        seen += 1;
    }

    check(error.kind == .NONE && seen == 2500 && sum == 2500 * 2501 / 2, "stream every row");

    // Leaving early cancels the rest, and the connection is usable again.
    seen = 0;

    for Pg.stream(db, *error, "select g as \"n!\" from generate_series(1, %) as g", 1000000, batch = 100) {
        seen += 1;
        if seen == 250 break;
    }

    check(error.kind == .NONE && seen == 250, "break out of a stream");

    count, count_error := Pg.query_scalar(db, "select count(*) from users where id > %", 0);

    check(count_error.kind == .NONE && count == 2, "the connection works after a break");

    // A failure part way: the rows before it arrive, then the error.
    seen = 0;
    for Pg.stream(db, *error, "select 1 / (1500 - g) as \"n!\" from generate_series(1, %) as g", 3000, batch = 100) seen += 1;

    check(error.kind == .SERVER && error.code == .DIVISION_BY_ZERO && seen >= 1400 && seen < 1500, "error in the middle of a stream");

    for Pg.stream(db, *error, "select id from users where id > %", 0, batch = 0) seen += 1;

    check(error.kind == .USAGE, "a batch of 0 is refused");

    _, after_error := Pg.query_scalar(db, "select count(*) from users where id > %", 0);

    check(after_error.kind == .NONE, "the connection works after a failed stream");
}

test_pipeline :: (db: *Pg.Connection) {
    p, enter_error := Pg.pipeline_enter(db);

    check(enter_error.kind == .NONE, "enter pipeline");

    first, error_a := Pg.queue(p, "select id, email from users where id = %", 1);
    touched, error_b := Pg.queue_execute(p, "update users set email = email where id > %", 0);
    second, error_c := Pg.queue(p, "select email from users where id = %", 2);

    check(error_a.kind == .NONE && error_b.kind == .NONE && error_c.kind == .NONE, "queue typed statements");

    // Taken out of order: the first and the update are read on the way and kept.
    rows_two, take_c := Pg.take(p, second);

    check(take_c.kind == .NONE && rows_two.result.row_count == 1, "take the last one first");
    for rows_two check(it.email == "b@example.com", "the last one's row");

    Pg.free_rows(*rows_two);

    count, take_b := Pg.take(p, touched);

    check(take_b.kind == .NONE && count == 2, "take an execute");

    rows_one, take_a := Pg.take(p, first);

    check(take_a.kind == .NONE && rows_one.result.row_count == 1, "take the first one last");
    for rows_one check(it.id == 1 && it.email == "a@example.com", "the first one's row");

    Pg.free_rows(*rows_one);

    _, again := Pg.take(p, first);

    check(again.kind == .USAGE, "a result is taken once");

    // A failing statement fails its batch: the next one is skipped.
    bad, _ := Pg.queue(p, "select id from users where id = 1 / %", 0);
    after, _ := Pg.queue(p, "select id from users where id = %", 1);

    _, bad_error := Pg.take(p, bad);

    check(bad_error.kind == .SERVER && bad_error.code == .DIVISION_BY_ZERO, "division by zero");

    _, after_error := Pg.take(p, after);

    check(after_error.kind == .ABORTED, "the rest of the batch is skipped");

    check(Pg.pipeline_exit(p).kind == .NONE, "exit after taking everything");
}

test_statement_cache :: (db: *Pg.Connection, config: Pg.Config) {
    Pg.set_trace(db, count_parses);
    defer Pg.set_trace(db, null);

    parses_sent = 0;

    for 1..3 {
        row, error := Pg.query_one(db, "select email from users where id = % and display_name is not distinct from display_name", 1);

        check(error.kind == .NONE, "cached query");
        check(row.email == "a@example.com", "cached query row");
    }

    check(parses_sent == 1, "three runs of one query parse once");

    before := parses_sent;

    for 1..2 {
        count, error := Pg.query_scalar(db, "select count(*) from users where id > %", 0, plan = .CUSTOM);
        check(error.kind == .NONE && count == 2, "custom plan");
    }

    check(parses_sent == before + 2, "a custom plan parses every time");

    verifying := config;
    verifying.verify_statements = true;

    checked, connect_error := Pg.connect(verifying);

    check(connect_error.kind == .NONE, "connect with verify_statements");
    if connect_error.kind != .NONE return;

    defer Pg.close(checked);

    for 1..2 {
        row, error := Pg.query_one(checked, "select a, b from drift");
        check(error.kind == .NONE && row.a == 1 && row.b == 2, "verified query");
    }

    // The table gains a column that the compiled query does not know of.
    _, alter_error := Pg.execute(db, "alter table drift add column c int8");

    check(alter_error.kind == .NONE, "alter drift");

    fresh, fresh_error := Pg.connect(verifying);

    check(fresh_error.kind == .NONE, "connect again");
    if fresh_error.kind != .NONE return;

    defer Pg.close(fresh);

    _, mismatch := Pg.query_one(fresh, "select * from drift");

    check(mismatch.kind == .USAGE && contains(mismatch.message, "different schema"), "a changed table is reported");

    plain, plain_error := Pg.connect(config);

    check(plain_error.kind == .NONE, "connect without verify");

    if plain_error.kind == .NONE {
        _, unchecked := Pg.query_one(plain, "select * from drift");
        check(unchecked.kind == .NONE, "no check without verify_statements");
        Pg.close(plain);
    }

    _, restore_error := Pg.execute(db, "alter table drift drop column c");

    check(restore_error.kind == .NONE, "restore drift");
}
