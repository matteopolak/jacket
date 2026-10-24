// The ORM against tests/typed/schema.sql. Built by run.sh with DATABASE_URL and JACKET_PG_ROOT set.
#import "Basic";
Io :: #import "Extensions/Io";
Pg :: #import "Extensions/Postgres";
#import "Extensions/Environment";
#load "db/schema.jai";

failures := 0;

check :: (ok: bool, what: string, loc := #caller_location) {
    if ok return;

    failures += 1;
    print("FAILED (line %): %\n", loc.line_number, what);
}

main :: () {
    Io.io_run(() {
        config, error := Pg.parse_url(environment_variable("DATABASE_URL"));

        assert(error.kind == .NONE);
        db:, error = Pg.connect(config);

        assert(error.kind == .NONE, "%", Pg.error_text(error));
        defer Pg.close(db);

        test_select(db);
        test_json_and_sql(db);
        test_writes(db);
    });

    if failures {
        print("% failures\n", failures);
        exit(1);
    }

    print("ok\n");
}

test_select :: (db: *Pg.Connection) {
    // Every row, in order.
    users, error := Pg.select(db, User, order_by = #code it.id);
    all := Pg.fetch_all(*users);

    check(error.kind == .NONE && all.count == 2 && all[0].email == "a@example.com" && all[1].display_name.null, Pg.error_text(error));
    Pg.free_rows(*users);

    // Columns, literals and parameters (an s64 against an int4 column).
    min_score := 0;
    domain := "%@example.com";

    users, error = Pg.select(db, User, where = #code it.score >= min_score && Pg.like(it.email, domain) && it.display_name != null);
    count := 0;

    for users {
        count += 1;
        check(it.email == "a@example.com", it.email);
    }

    check(error.kind == .NONE && count == 1, Pg.error_text(error));
    Pg.free_rows(*users);

    // Descending, limit and offset.
    users, error = Pg.select(db, User, order_by = #code it.id, desc = true, limit = 1);
    all = Pg.fetch_all(*users);
    check(error.kind == .NONE && all.count == 1 && all[0].email == "b@example.com", Pg.error_text(error));
    Pg.free_rows(*users);

    // Pg.in, Pg.lower, negation and arithmetic.
    ids := s64.[1, 2, 3];

    users, error = Pg.select(db, User, where = #code Pg.in(it.id, ids) && !(Pg.lower(it.email) == "b@example.com") && it.id * 2 < 10);
    all = Pg.fetch_all(*users);
    check(error.kind == .NONE && all.count == 1 && all[0].id == 1, Pg.error_text(error));
    Pg.free_rows(*users);

    // A json field, by its document key.
    users, error = Pg.select(db, User, where = #code it.settings.theme == "dark" && it.settings.font_size > 10);
    all = Pg.fetch_all(*users);
    check(error.kind == .NONE && all.count == 1 && all[0].id == 1, Pg.error_text(error));
    Pg.free_rows(*users);

    // An optional filter: no value matches everything.
    wanted := Pg.Nullable(s64).{ null = true };

    users, error = Pg.select(db, User, where = #code Pg.if_some(wanted, #code it.id == wanted));
    all = Pg.fetch_all(*users);
    check(error.kind == .NONE && all.count == 2, Pg.error_text(error));
    Pg.free_rows(*users);

    wanted = .{ value = 2 };
    users, error = Pg.select(db, User, where = #code Pg.if_some(wanted, #code it.id == wanted));
    all = Pg.fetch_all(*users);
    check(error.kind == .NONE && all.count == 1 && all[0].id == 2, Pg.error_text(error));
    Pg.free_rows(*users);
}

Theme_Only :: struct {
    theme: string;
}

ids_of :: (rows: [] User) -> [..] s64 {
    ids: [..] s64;
    ids.allocator = temp;

    for rows array_add(*ids, it.id);

    return ids;
}

// The jsonb operators, raw fragments and a raw source.
test_json_and_sql :: (db: *Pg.Connection) {
    dark := Theme_Only.{ theme = "dark" };

    users, error := Pg.select(db, User, where = #code Pg.contains(it.settings, dark));
    all := Pg.fetch_all(*users);

    check(error.kind == .NONE && all.count == 1 && all[0].id == 1, Pg.error_text(error));
    Pg.free_rows(*users);

    users, error = Pg.select(db, User, where = #code Pg.has_key(it.settings, "font_size") || Pg.json_path(it.settings, "$.theme ? (@ == \"light\")"));
    all = Pg.fetch_all(*users);
    check(error.kind == .NONE && all.count == 1 && all[0].id == 1, Pg.error_text(error));
    Pg.free_rows(*users);

    // A fragment with a column, a named parameter and an expression.
    email := "A@EXAMPLE.COM";

    users, error = Pg.select(db, User, where = #code it.id > 0 && Pg.sql("lower(%(it.email)) = lower(%email) and %(it.settings.font_size) > %(10 + 2)"));
    all = Pg.fetch_all(*users);
    check(error.kind == .NONE && all.count == 1 && all[0].id == 1, Pg.error_text(error));
    Pg.free_rows(*users);

    // A raw source returning the table's columns.
    first := 2;

    users, error = Pg.select(db, User, from_sql = "select * from users where id >= %first", order_by = #code it.id);
    all = Pg.fetch_all(*users);
    ids := ids_of(all);

    check(error.kind == .NONE && ids.count == 1 && ids[0] == 2, Pg.error_text(error));
    Pg.free_rows(*users);
}

test_writes :: (db: *Pg.Connection) {
    user := User.{ email = "orm@example.com" };
    user.display_name = .{ value = "Orm" };

    inserted, error := Pg.insert(db, user);

    check(error.kind == .NONE && inserted.id > 2 && inserted.email == "orm@example.com" && inserted.mood == .ok && inserted.score == 0, Pg.error_text(error));

    bump := 5;
    changed:, error = Pg.update(db, User, set = #code {
        it.score = bump;
        it.display_name = null;
    }, where = #code it.id == inserted.id);

    check(error.kind == .NONE && changed == 1, Pg.error_text(error));

    // One assignment needs no braces.
    changed, error = Pg.update(db, User, set = #code it.score = it.score + 1, where = #code it.id == inserted.id);
    check(error.kind == .NONE && changed == 1, Pg.error_text(error));

    users:, error = Pg.select(db, User, where = #code it.id == inserted.id);
    all := Pg.fetch_all(*users);

    check(error.kind == .NONE && all.count == 1 && all[0].score == 6 && all[0].display_name.null, Pg.error_text(error));
    Pg.free_rows(*users);

    gone:, error = Pg.delete(db, User, where = #code it.id == inserted.id);

    check(error.kind == .NONE && gone == 1, Pg.error_text(error));
}
