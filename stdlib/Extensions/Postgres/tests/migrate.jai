// Migrations against a live server (tests/start_server.sh); skipped when JAI_PG_TEST_PORT is not
// set. The split and naming checks run without one.
#import "Basic";
#import "String";
#import "Extensions/Environment";
Io :: #import "Extensions/Io";
Pg :: #import "Extensions/Postgres";

MIGRATIONS :: #run Pg.load_migrations("migrations");
BAD :: #run Pg.load_migrations("migrations_bad");

port: u16;

main :: () {
    test_split();
    test_loaded();

    port_text := environment_variable("JAI_PG_TEST_PORT");
    if port_text {
        value, valid := string_to_int(port_text);

        assert(valid, "bad JAI_PG_TEST_PORT");
        port = cast(u16) value;
        Io.io_run(() {
            test_migrate(port);
        });
    }

    print("ok\n");
}

test_split :: () {
    parts := Pg.split_statements("select 1; select ';' ; -- x;\n select $a$;$a$; /* ; /* ; */ */ select E'\\';'; ;");

    assert(parts.count == 4, "%", parts);
    assert(parts[0] == "select 1");
    assert(parts[1] == "select ';'");
    assert(parts[2] == "-- x;\n select $a$;$a$");
    assert(parts[3] == "/* ; /* ; */ */ select E'\\';'");
    assert(Pg.split_statements("-- only a comment\n").count == 0);
    assert(Pg.split_statements("select $1; select 2").count == 2);

    // The server counts characters from 1; a failure is reported by line and column.
    text := "select 1;\nselect é, x\nfrom t";
    line, column := Pg.line_column_of_position(text, 21);

    assert(line == 2 && column == 11, "% %", line, column);

    line, column = Pg.line_column_of_position(text, 1);
    assert(line == 1 && column == 1);

    line, column = Pg.line_column_of_position(text, 23);
    assert(line == 3 && column == 1, "% %", line, column);

    line, column = Pg.line_column_of_position(text, 0);
    assert(line == 0 && column == 0);

    failure := Pg.Migration_Failure.{ name = "0007_add_teams", line = 3, message = "relation \"team\" does not exist" };

    assert(Pg.describe_failure(failure) == "migration 0007_add_teams.sql failed at line 3: relation \"team\" does not exist");
    failure.line = 0;
    assert(Pg.describe_failure(failure) == "migration 0007_add_teams.sql failed: relation \"team\" does not exist");
}

test_loaded :: () {
    assert(MIGRATIONS.count == 3, "%", MIGRATIONS.count);
    assert(MIGRATIONS[0].version == 1 && MIGRATIONS[0].name == "0001_create_teams");
    assert(MIGRATIONS[1].version == 2 && MIGRATIONS[1].transaction);
    assert(MIGRATIONS[2].version == 3 && !MIGRATIONS[2].transaction);
    assert(MIGRATIONS[0].sha256.count == 64);
    hash := Pg.schema_hash(MIGRATIONS);

    assert(hash.count == 64 && hash != Pg.schema_hash(.[MIGRATIONS[0], MIGRATIONS[1]]));
}

test_migrate :: (port: u16) {
    config: Pg.Config;
    config.host = "127.0.0.1";
    config.port = port;
    config.user = "postgres";
    config.database = "postgres";
    config.ssl_mode = .DISABLE;

    db, error := Pg.connect(config);

    assert(error.kind == .NONE, "%", Pg.error_text(error));
    defer Pg.close(db);

    reset :: (db: *Pg.Connection) {
        error := Pg.execute_script(db, "drop table if exists mig_teams, mig_partial, jacket_migrations cascade; drop function if exists mig_team_count");
        assert(error.kind == .NONE, "%", Pg.error_text(error));
    }

    reset(db);
    defer reset(db);

    applied:, error = Pg.migrate(db, MIGRATIONS);

    assert(error.kind == .NONE, "%", Pg.error_text(error));
    assert(applied == 3, "%", applied);

    result:, error = Pg.query_dynamic(db, "select mig_team_count(), (select name from mig_teams), (select count(*) from pg_indexes where indexname = 'mig_teams_name')");

    assert(error.kind == .NONE, "%", Pg.error_text(error));
    count, _ := Pg.get(*result, 0, 0, s64);
    name, _ := Pg.get(*result, 0, 1, string);
    indexes, _ := Pg.get(*result, 0, 2, s64);

    assert(count == 1 && name == "it's; fine" && indexes == 1, "% % %", count, name, indexes);
    Pg.free_result(*result);

    // Nothing pending: a second run applies nothing.
    applied, error = Pg.migrate(db, MIGRATIONS);
    assert(error.kind == .NONE && applied == 0);

    // A changed file is refused, unless allowed.
    changed := array_copy(MIGRATIONS,, temp);
    changed[0].sha256 = "0000";
    status:, error = Pg.migration_status(db, changed);

    assert(status[0].state == .MODIFIED && status[1].state == .APPLIED);
    applied, error = Pg.migrate(db, changed);
    assert(error.kind == .USAGE && contains(error.message, "0001_create_teams"), "%", Pg.error_text(error));
    applied, error = Pg.migrate(db, changed, .{ allow_modified = true });
    assert(error.kind == .NONE && applied == 0);

    // A missing migration is refused.
    applied, error = Pg.migrate(db, .[MIGRATIONS[0], MIGRATIONS[1]]);
    assert(error.kind == .USAGE && contains(error.message, "not in the list"), "%", Pg.error_text(error));

    // A pending migration numbered below an applied one is out of order.
    status, error = Pg.migration_status(db, .[MIGRATIONS[0], MIGRATIONS[1], MIGRATIONS[2], .{ version = 2, name = "0002_other", sha256 = "x" }]);
    assert(error.kind == .NONE);

    // A failing migration is rolled back whole and named in the error.
    bad := BAD[0];
    bad.version = 4;
    applied, error = Pg.migrate(db, .[MIGRATIONS[0], MIGRATIONS[1], MIGRATIONS[2], bad]);
    assert(error.kind == .SERVER && applied == 0, "%", Pg.error_text(error));
    assert(begins_with(error.message, "migration 0001_fails failed: relation \"mig_no_such_table\""), "%", error.message);
    result, error = Pg.query_dynamic(db, "select to_regclass('mig_partial') is null, (select count(*) from jacket_migrations)");
    gone, _ := Pg.get(*result, 0, 0, bool);
    rows, _ := Pg.get(*result, 0, 1, s64);

    assert(gone && rows == 3, "% %", gone, rows);
    Pg.free_result(*result);

    // The failure says where in the file: the line the server's position falls on. An error
    // without a position (a utility statement) is traced to its statement.
    failure: Pg.Migration_Failure;

    applied, error = Pg.migrate_reporting(db, .[MIGRATIONS[0], MIGRATIONS[1], MIGRATIONS[2], bad], *failure);
    assert(error.kind == .SERVER && failure.name == "0001_fails" && failure.index == 3 && failure.line == 2 && failure.column == 15, "% % %", failure.name, failure.line, failure.column);

    quiet := bad;
    quiet.name = "0004_quiet";
    quiet.sql = "select 1;\n\nalter table mig_no_such_table add column x int;\nselect 2;";
    applied, error = Pg.migrate_reporting(db, .[MIGRATIONS[0], MIGRATIONS[1], MIGRATIONS[2], quiet], *failure);
    assert(error.kind == .SERVER && failure.name == "0004_quiet" && failure.line == 3, "% %", failure.name, failure.line);
}
