// Row type identity: a query whose columns are exactly one table's returns that table's struct
// from db/schema.jai. Built by ../run.sh with JACKET_PG_ROOT set to a copy of this folder.
#import "Basic";
Io :: #import "Extensions/Io";
Pg :: #import "Extensions/Postgres";
#import "Extensions/Environment";
#load "db/schema.jai";

takes_user :: (user: User) -> string {
    return user.email;
}

main :: () {
    Io.io_run(() {
        config, error := Pg.parse_url(environment_variable("DATABASE_URL"));

        assert(error.kind == .NONE);
        db:, error = Pg.connect(config);

        assert(error.kind == .NONE, "%", Pg.error_text(error));
        defer Pg.close(db);

        // `select *` and `returning *` are `User`.
        user:, error = Pg.query_one(db, "select * from users where id = %", 1);

        #assert type_of(user) == User;
        assert(error.kind == .NONE && takes_user(user) == "a@example.com" && user.mood == .happy, "%", Pg.error_text(error));

        rows:, error = Pg.query(db, "select * from users order by id");

        #assert type_of(rows) == Pg.Rows(User);
        count := 0;
        for rows count += ifx takes_user(it) then 1;

        assert(rows.error.kind == .NONE && count == 2);
        Pg.free_rows(*rows);

        same:, found:, error = Pg.query_opt(db, "update users set score = score where id = % returning *", 2);

        #assert type_of(same) == User;
        assert(error.kind == .NONE && found && same.display_name.null);

        // Fewer columns, renamed ones and a table whose struct the caller cannot see are not.
        partial:, error = Pg.query_one(db, "select id, email from users where id = %", 1);

        #assert type_of(partial) != User;
        renamed:, error = Pg.query_one(db, "select id as user_id, email, display_name, created_at, settings, mood, score, tags from users where id = %", 1);

        #assert type_of(renamed) != User;
        post:, error = Pg.query_one(db, "select * from posts where id = %", 1);

        #assert type_of(post) != User;
        assert(error.kind == .NONE && post.title == "hello");
    });

    print("ok\n");
}
