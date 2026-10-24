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
        test_relations(db);
        test_writes(db);
        test_json_maybe(db);
        test_copy(db);
        test_each(db);
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

// Filters through a to-one relation, and loading both directions in one query each.
test_relations :: (db: *Pg.Connection) {
    owner := "a@example.com";

    posts, error := Pg.select(db, Post, where = #code it.author.email == owner && it.author.settings.theme == "dark", order_by = #code it.id);
    all_posts := Pg.fetch_all(*posts);

    check(error.kind == .NONE && all_posts.count == 2 && all_posts[0].title == "hello", Pg.error_text(error));

    authors:, error = Pg.load_one(db, all_posts, "author");

    check(error.kind == .NONE && authors.items.count == 2 && !authors.items[0].null && authors.items[1].value.email == "a@example.com", Pg.error_text(error));
    Pg.free_loaded(*authors);
    Pg.free_rows(*posts);

    users:, error = Pg.select(db, User, order_by = #code it.id);
    all_users := Pg.fetch_all(*users);

    // Alice twice: parents sharing a key share their items.
    parents := User.[all_users[0], all_users[1], all_users[0]];
    written:, error = Pg.load_many(db, parents, "posts");

    check(error.kind == .NONE && written.items.count == 3, Pg.error_text(error));
    check(written.items[0].count == 2 && written.items[1].count == 0 && written.items[2].count == 2, "post counts");
    check(written.items[0][0].author_id == all_users[0].id && written.items[0][1].title != "", "post rows");
    Pg.free_loaded(*written);
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

// A Json_Maybe field's three states. User 1's settings have no "beta" key, user 2 has no settings.
test_json_maybe :: (db: *Pg.Connection) {
    users, error := Pg.select(db, User, where = #code it.settings.beta.state == .MISSING);
    all := Pg.fetch_all(*users);

    check(error.kind == .NONE && all.count == 1 && all[0].id == 1 && all[0].settings.value.value.beta.state == .MISSING, Pg.error_text(error));
    Pg.free_rows(*users);

    user := User.{ email = "maybe@example.com" };
    user.settings = .{ value = .{ value = .{ theme = "x", beta = .{ state = .NULL } } } };

    with_null, error = := Pg.insert(db, user);
    check(error.kind == .NONE, Pg.error_text(error));

    users, error = Pg.select(db, User, where = #code it.settings.beta.state == .NULL);
    all = Pg.fetch_all(*users);
    check(error.kind == .NONE && all.count == 1 && all[0].id == with_null.id && all[0].settings.value.value.beta.state == .NULL, Pg.error_text(error));
    Pg.free_rows(*users);

    user.email = "set@example.com";
    user.settings.value.value.beta = .{ state = .SET, value = true };

    with_value, error = := Pg.insert(db, user);
    check(error.kind == .NONE, Pg.error_text(error));

    // `.value` is the key itself.
    users, error = Pg.select(db, User, where = #code it.settings.beta.state == .SET && it.settings.beta.value == true);
    all = Pg.fetch_all(*users);
    check(error.kind == .NONE && all.count == 1 && all[0].id == with_value.id, Pg.error_text(error));
    Pg.free_rows(*users);

    // User 2 (no settings) has no state, so it matches every `!=`.
    users, error = Pg.select(db, User, where = #code it.settings.beta.state != .SET, order_by = #code it.id);
    all = Pg.fetch_all(*users);
    ids := ids_of(all);

    check(error.kind == .NONE && ids.count == 3 && ids[0] == 1 && ids[1] == 2 && ids[2] == with_null.id, Pg.error_text(error));
    Pg.free_rows(*users);

    gone:, error = Pg.delete(db, User, where = #code it.id == with_null.id || it.id == with_value.id);

    check(error.kind == .NONE && gone == 2, Pg.error_text(error));
}

Copy_Row :: struct {
    n: Pg.Numeric;
    j: Pg.Json(Settings); @pg_json
    jb: Pg.Nullable(Pg.Json(Settings));
    b: [] u8;
    at: Pg.Timestamp;
    a: [] s64;
    m: Mood;
    f: float64;
    i: s32;
    t: Pg.Nullable(string);
}

// Binary COPY: the ORM's copy_many, and copy_in over a table with one column of each kind.
test_copy :: (db: *Pg.Connection) {
    users: [3] User;
    for *users {
        it.email = tprint("copy%@example.com", it_index);
        it.display_name.null = true;
        it.settings.null = true;
    }

    users[1].display_name = .{ value = "Copied" };
    users[1].settings = .{ value = .{ value = .{ theme = "copied" } } };

    stored, error := Pg.copy_many(db, users);

    check(error.kind == .NONE && stored == 3, Pg.error_text(error));

    copied:, error = Pg.select(db, User, where = #code Pg.like(it.email, "copy%@example.com"), order_by = #code it.email);
    all := Pg.fetch_all(*copied);

    check(error.kind == .NONE && all.count == 3 && all[1].display_name.value == "Copied" && all[0].display_name.null && all[2].mood == .ok, Pg.error_text(error));
    Pg.free_rows(*copied);

    gone:, error = Pg.delete(db, User, where = #code Pg.like(it.email, "copy%@example.com"));

    check(error.kind == .NONE && gone == 3, Pg.error_text(error));

    error = Pg.execute_script(db, "create temp table copy_kinds (n numeric, j json, jb jsonb, b bytea, at timestamptz, a int8[], m mood, f float8, i int4, t text)");
    check(error.kind == .NONE, Pg.error_text(error));

    rows: [2] Copy_Row;
    rows[0] = .{ n = .{ "-12345.0067" }, b = .[1, 0, 255], at = .{ 86_400_000_000 }, a = .[3, -4], m = .happy, f = 2.5, i = -7 };
    rows[0].j.value = .{ theme = "dark" };
    rows[0].jb = .{ value = .{ value = .{ theme = "light", font_size = 9 } } };
    rows[0].t = .{ value = "tab\there" };
    rows[1] = .{ n = .{ "0.000" }, m = .sad, i = 2_000_000_000 };
    rows[1].jb.null = true;
    rows[1].t.null = true;

    stored, error = Pg.copy_in(db, "copy copy_kinds from stdin (format binary)", rows);
    check(error.kind == .NONE && stored == 2, Pg.error_text(error));

    text, error = := Pg.query_dynamic(db, "select n::text, j::text, jb->>'theme', encode(b, 'hex'), (at at time zone 'UTC')::date::text, a::text, m::text, f::text, i::text, t from copy_kinds order by i");
    defer Pg.free_result(*text);
    check(error.kind == .NONE && text.row_count == 2, Pg.error_text(error));

    if text.row_count == 2 {
        expected := string.["-12345.0067", "{\"theme\":\"dark\",\"font_size\":14}", "light", "0100ff", "2000-01-02", "{3,-4}", "happy", "2.5", "-7", "tab\there"];

        for expected {
            got := Pg.get(*text, 0, it_index, string);
            check(got == it, tprint("column %: % instead of %", it_index, got, it));
        }

        check(Pg.get(*text, 1, 0, string) == "0.000" && Pg.is_null(*text, 1, 2) && Pg.is_null(*text, 1, 9), "the second row");
    }

    // Enough rows for several chunks.
    many := NewArray(5000, Copy_Row,, temp);
    for *many it.* = rows[0];

    stored, error = Pg.copy_in(db, "copy copy_kinds from stdin (format binary)", many);
    check(error.kind == .NONE && stored == many.count, Pg.error_text(error));

    // A wrong width is the server's error, not a corrupt row.
    Wide :: struct {
        i: s64;
    }

    wide := Wide.[.{ 1 }];

    stored, error = Pg.copy_in(db, "copy copy_kinds (i) from stdin (format binary)", wide);
    check(error.kind == .SERVER, "an s64 into int4 should fail");
}

// Pg.each reads only the columns its loop body uses (run.sh checks the cached statement).
test_each :: (db: *Pg.Connection) {
    error: Pg.Pg_Error;
    emails: [..] string;
    emails.allocator = temp;
    min := 0;

    for Pg.each(db, User, *error, where = #code it.score >= min, order_by = #code it.id) {
        array_add(*emails, copy_string(it.email,, temp));
        check(it.id == it_index + 1, "ids in order");
    }

    check(error.kind == .NONE && emails.count == 2 && emails[0] == "a@example.com", Pg.error_text(error));

    // `it` used whole fetches everything.
    users: [..] User;
    users.allocator = temp;

    // (Its strings read from the result, which the loop frees, so check them inside.)
    for Pg.each(db, User, *error, where = #code it.id == 1) {
        array_add(*users, it);
        check(users[0].settings.value.value.theme == "dark", "the whole row");
    }

    check(error.kind == .NONE && users.count == 1 && users[0].mood == .happy, Pg.error_text(error));

    // A failure lands in `error`, and the body never runs.
    zero := 0;
    ran := false;

    for Pg.each(db, User, *error, where = #code it.score / zero > 1) ran = it.id > 0;

    check(error.kind == .SERVER && !ran, "a division by zero should fail");
}
