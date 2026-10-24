// Managed PostgreSQL. The pure logic (platform choice, cache paths, the checksum table) always
// runs; with JAI_PG_TEST_MANAGED=1 it also downloads the pinned release into a temporary cache,
// starts a server, queries it, stops it, and starts it again on the same data folder.
#import "Basic";
#import "String";
#import "POSIX";
Io :: #import "Extensions/Io";
Pg :: #import "Extensions/Postgres";

main :: () {
    test_logic();

    if to_string(getenv("JAI_PG_TEST_MANAGED")) == "1" {
        Io.io_run(() {
            test_real();
        });
    }

    print("ok\n");
}

test_logic :: () {
    assert(Pg.managed_triple_for(.MACOS, true, false) == "aarch64-apple-darwin");
    assert(Pg.managed_triple_for(.MACOS, false, false) == "x86_64-apple-darwin");
    assert(Pg.managed_triple_for(.LINUX, false, false) == "x86_64-unknown-linux-gnu");
    assert(Pg.managed_triple_for(.LINUX, true, true) == "aarch64-unknown-linux-musl");
    assert(Pg.managed_triple_for(.WINDOWS, false, true) == "x86_64-pc-windows-msvc");

    assert(Pg.managed_cache_dir_for(.MACOS, "/Users/a", "", "") == "/Users/a/Library/Caches/jacket/postgres");
    assert(Pg.managed_cache_dir_for(.LINUX, "/home/a", "", "") == "/home/a/.cache/jacket/postgres");
    assert(Pg.managed_cache_dir_for(.LINUX, "/home/a", "/x/cache", "") == "/x/cache/jacket/postgres");
    assert(Pg.managed_cache_dir_for(.WINDOWS, "C:\\Users\\a", "", "C:\\L") == "C:\\L\\jacket\\postgres");
    assert(Pg.managed_cache_dir_for(.LINUX, "", "", "") == "");

    // Every platform the triple function can name has a pinned checksum, 64 hex digits long.
    for os: Pg.Managed_Os.[.MACOS, .LINUX, .WINDOWS] {
        for arm: bool.[false, true] {
            for musl: bool.[false, true] {
                triple := Pg.managed_triple_for(os, arm, musl);
                sum := Pg.managed_checksum(Pg.MANAGED_VERSION, triple);
                assert(sum.count == 64, "no checksum for %", triple);
            }
        }
    }

    here := Pg.managed_triple();

    assert(here != "", "no release for this platform");
    assert(Pg.managed_checksum(Pg.MANAGED_VERSION, here).count == 64);
    assert(Pg.managed_checksum("1.0.0", here) == "");
    assert(Pg.managed_cache_dir() != "");
}

test_real :: () {
    root := tprint("/tmp/jacket-managed-test-%", cast(s64) getpid());
    settings: Pg.Managed_Settings;
    settings.cache_dir = tprint("%/cache", root);
    settings.data_dir = tprint("%/data", root);
    settings.port = 54339;
    started := current_time_monotonic();

    server, error := Pg.start_managed(settings);

    assert(error == "", "start: %", error);
    print("first start: % ms\n", to_milliseconds(current_time_monotonic() - started));
    assert(server.started_here);
    check_server(server);

    // A second start finds the running server and reuses it.
    again, error2 := Pg.start_managed(settings);

    assert(error2 == "", "reuse: %", error2);
    assert(!again.started_here && again.port == server.port);
    assert(Pg.stop_managed(*again) == "");
    check_server(again);

    stop_error := Pg.stop_managed(*server);

    assert(stop_error == "", "stop: %", stop_error);

    // Stopped: the next start reuses the data folder and the downloaded binaries.
    started = current_time_monotonic();
    third, error3 := Pg.start_managed(settings);

    assert(error3 == "", "restart: %", error3);
    print("restart: % ms\n", to_milliseconds(current_time_monotonic() - started));
    assert(third.started_here);
    check_server(third);
    assert(Pg.stop_managed(*third) == "");

    // The mismatching-checksum path: an unknown version has no pin.
    unknown := settings;
    unknown.version = "1.0.0";
    _, bad := Pg.ensure_binaries(unknown);

    assert(bad != "");

    run_command_quiet(tprint("rm -rf %", root));
}

check_server :: (server: Pg.Managed_Server) {
    db, pg_error := Pg.connect(server.config);

    assert(pg_error.kind == .NONE, "connect: %", Pg.error_text(pg_error));
    defer Pg.close(db);

    result, query_error := Pg.query_dynamic(db, "select version(), current_setting('fsync')");

    assert(query_error.kind == .NONE, "query: %", Pg.error_text(query_error));
    defer Pg.free_result(*result);

    version := Pg.get(*result, 0, 0, string);

    assert(begins_with(version, "PostgreSQL 17."), "version: %", version);
    assert(Pg.get(*result, 0, 1, string) == "off");
}

run_command_quiet :: (command: string) {
    system(temp_c_string(command));
}
